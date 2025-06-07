use serde::{Deserialize, Serialize};
/// Encryption module for handling data encryption and decryption
/// This module provides functions to encrypt and decrypt biosample data
/// using a tfhe fully homomorphic encryption scheme.
// Required libraries
use std::collections::HashMap; // For HashMap
use std::error::Error; // For error handling
use std::fs::File; // For file handling
use std::io::{Read, Write}; // For reading and writing files
use std::path::Path; // For path handling
                     // For serialization and deserialization
use tfhe::integer::{ServerKey, SignedRadixCiphertext}; // For integer encryption
use tfhe::shortint::parameters::PARAM_MESSAGE_2_CARRY_2;

// use the BiosampleRecord struct from the data_generator module
use crate::data_generator::BiosampleRecord;

/// Number of bits to use for integer encodings
const FHE_INT_BITS: usize = 8;

/// Represents a structure for handling Fully Homomorphic Encryption operations on biosample data
///
/// This structure contains the client key for encryption/decryption and the server key
/// for performing homomorphic operations on encrypted data without decryption.
#[derive(Clone)]
pub struct BiosampleFHE {
    client_key: tfhe::integer::ClientKey,
    server_key: ServerKey,
}

/// Represents an encrypted vector of data
///
/// This structure contains serialized ciphertexts and the length of the vector,
/// allowing for storage and transmission of encrypted vector data.
#[derive(Serialize, Deserialize, Clone)]
pub struct EncryptedVector {
    pub data: Vec<Vec<u8>>, // Serialized ciphertexts
    pub length: usize,      // Length of the vector
}

/// Represents an encrypted categorical variable
///
/// This structure contains the categories of the categorical variable and
/// the corresponding encrypted vectors for each category, allowing for
/// homomorphic operations on categorical data.
#[derive(Serialize, Deserialize, Clone)]
pub struct EncryptedCategorical {
    pub categories: Vec<String>, // Categories of the categorical variable
    pub vectors: Vec<EncryptedVector>, // Encrypted vectors for each category
}

/// Implements the Default trait for BiosampleFHE
///
/// This implementation allows creating a BiosampleFHE instance using the default() method,
/// which simply calls the new() method to generate fresh encryption keys.
///
/// # Returns
///
/// A new BiosampleFHE instance with initialized keys
impl Default for BiosampleFHE {
    fn default() -> Self {
        Self::new()
    }
}

impl BiosampleFHE {
    /// Creates a new instance of BiosampleFHE with freshly generated keys
    ///
    /// This function initializes a new BiosampleFHE structure by generating a new client key
    /// for encryption/decryption and a server key for performing homomorphic operations.
    ///
    /// # Returns
    ///
    /// A new BiosampleFHE instance with initialized keys
    pub fn new() -> Self {
        // Generate client key
        let client_key = tfhe::integer::ClientKey::new(PARAM_MESSAGE_2_CARRY_2);
        // Generate server key for homomorphic operations
        let server_key = ServerKey::new_radix_server_key(&client_key);

        Self {
            client_key,
            server_key,
        }
    }

    /// Encrypts a vector of floating-point values using FHE
    ///
    /// This function takes a slice of f64 values, scales them by the provided factor,
    /// converts them to integers, and encrypts each value using the client key.
    /// # Arguments
    ///
    /// * `values` - A slice of f64 values to encrypt
    /// * `scale` - A scaling factor to convert floating-point values to integers
    ///
    /// # Returns
    ///
    /// An `EncryptedVector` containing the encrypted values
    pub fn encrypt_f64_vector(&self, values: &[f64], scale: f64) -> EncryptedVector {
        // Scale and convert to integers
        let scaled_values: Vec<i64> = values.iter().map(|&v| (v * scale).round() as i64).collect();

        // Encrypt each value
        let encrypted_data: Vec<Vec<u8>> = scaled_values
            .iter()
            .map(|&v| {
                let ciphertext = self.client_key.encrypt_signed_radix(v, FHE_INT_BITS);
                bincode::serialize(&ciphertext).unwrap()
            })
            .collect();

        EncryptedVector {
            data: encrypted_data,
            length: values.len(),
        }
    }

    /// Encrypts a vector of boolean values using FHE
    ///
    /// This function takes a slice of boolean values, converts them to integers (1 for true, 0 for false),
    /// and encrypts each value using the client key.
    ///
    /// # Arguments
    ///
    /// * `values` - A slice of boolean values to encrypt
    ///
    /// # Returns
    ///
    /// An `EncryptedVector` containing the encrypted values
    pub fn encrypt_bool_vector(&self, values: &[bool]) -> EncryptedVector {
        // Convert bools to integers
        let int_values: Vec<i64> = values.iter().map(|&v| if v { 1 } else { 0 }).collect();

        // Encrypt each value
        let encrypted_data: Vec<Vec<u8>> = int_values
            .iter()
            .map(|&v| {
                let ciphertext = self.client_key.encrypt_signed_radix(v, FHE_INT_BITS);
                bincode::serialize(&ciphertext).unwrap()
            })
            .collect();
        EncryptedVector {
            data: encrypted_data,
            length: values.len(),
        }
    }

    /// Encrypts a vector of categorical values using FHE
    ///
    /// This function takes a slice of string values representing categories,
    /// converts them to one-hot encoded vectors, and encrypts each vector.
    ///
    /// # Arguments
    ///
    /// * `values` - A slice of String values to encrypt
    ///
    /// # Returns
    ///
    /// An `EncryptedCategorical` containing the encrypted one-hot vectors and category names
    pub fn encrypt_categorical(&self, values: &[String]) -> EncryptedCategorical {
        // Find unique categories
        let mut categories: Vec<String> = values
            .iter()
            .cloned()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        categories.sort(); // Sort categories for consistency

        //Create a one-hot encoded vectors
        let mut one_hot_vectors: Vec<Vec<bool>> = Vec::with_capacity(categories.len());

        for category in &categories {
            let one_hot: Vec<bool> = values.iter().map(|v| v == category).collect();
            one_hot_vectors.push(one_hot);
        }

        // Encrypt each one-hot vector
        let encrypted_vectors: Vec<EncryptedVector> = one_hot_vectors
            .iter()
            .map(|v| self.encrypt_bool_vector(v))
            .collect();

        EncryptedCategorical {
            categories,
            vectors: encrypted_vectors,
        }
    }

    /// Decrypts a vector of encrypted floating-point values
    ///
    /// # Arguments
    ///
    /// * `encrypted` - An `EncryptedVector` containing the encrypted values
    /// * `scale` - The scaling factor used during encryption
    ///
    /// # Returns
    ///
    /// A vector of decrypted f64 values
    pub fn decrypt_f64_vector(&self, encrypted: &EncryptedVector, scale: f64) -> Vec<f64> {
        encrypted
            .data
            .iter()
            .map(|data| {
                // Use RadixCiphertext instead of BaseSignedRadixCiphertext
                let ciphertext: SignedRadixCiphertext = bincode::deserialize(data).unwrap();
                let decrypted_value: i64 = self.client_key.decrypt_signed_radix(&ciphertext);
                decrypted_value as f64 / scale
            })
            .collect()
    }

    /// Decrypts a vector of encrypted boolean values
    ///
    /// # Arguments
    ///
    /// * `encrypted` - An `EncryptedVector` containing the encrypted values
    ///
    /// # Returns
    ///
    /// A vector of decrypted boolean values
    #[allow(dead_code)]
    pub fn decrypt_bool_vector(&self, encrypted: &EncryptedVector) -> Vec<bool> {
        encrypted
            .data
            .iter()
            .map(|data| {
                let ciphertext: SignedRadixCiphertext = bincode::deserialize(data).unwrap();
                let decrypted_value: i64 = self.client_key.decrypt_signed_radix(&ciphertext);
                decrypted_value != 0
            })
            .collect()
    }

    /// Returns a reference to the server key
    ///
    /// # Returns
    ///
    /// A reference to the `ServerKey` used for homomorphic operations
    pub fn server_key(&self) -> &ServerKey {
        &self.server_key
    }

    /// Saves the encryption keys to disk
    ///
    /// # Arguments
    ///
    /// * `client_key_path` - The path where the client key will be saved
    /// * `server_key_path` - The path where the server key will be saved
    ///
    /// # Returns
    ///
    /// A Result containing () if successful, or an error if the keys could not be saved
    #[allow(dead_code)]
    pub fn save_keys(
        &self,
        client_key_path: &Path,
        server_key_path: &Path,
    ) -> Result<(), Box<dyn Error>> {
        // Save the client key
        let mut client_key_file = File::create(client_key_path)?;
        let client_key_bytes = bincode::serialize(&self.client_key)?;
        client_key_file.write_all(&client_key_bytes)?;

        // Save the server key
        let mut server_key_file = File::create(server_key_path)?;
        let server_key_bytes = bincode::serialize(&self.server_key)?;
        server_key_file.write_all(&server_key_bytes)?;

        Ok(())
    }

    /// Loads encryption keys from disk
    ///
    /// # Arguments
    ///
    /// * `client_key_path` - The path from which the client key will be loaded
    /// * `server_key_path` - The path from which the server key will be loaded
    ///
    /// # Returns
    ///
    /// A Result containing a new `Self` instance if successful, or an error if the keys could not be loaded
    #[allow(dead_code)]
    pub fn load_keys(
        client_key_path: &Path,
        server_key_path: &Path,
    ) -> Result<Self, Box<dyn Error>> {
        // Load the client key
        let mut client_key_file = File::open(client_key_path)?;
        let mut client_key_bytes = Vec::new();
        client_key_file.read_to_end(&mut client_key_bytes)?;
        let client_key: tfhe::integer::ClientKey = bincode::deserialize(&client_key_bytes)?;

        // Load the server key
        let mut server_key_file = File::open(server_key_path)?;
        let mut server_key_bytes = Vec::new();
        server_key_file.read_to_end(&mut server_key_bytes)?;
        let server_key: ServerKey = bincode::deserialize(&server_key_bytes)?;

        Ok(Self {
            client_key,
            server_key,
        })
    }
}
pub fn encrypt_biosample_data(
    fhe: &BiosampleFHE,
    records: &[BiosampleRecord],
) -> Result<HashMap<String, EncryptedVector>, Box<dyn Error>> {
    let mut encrypted_data = HashMap::new();

    // Extract and scale the numerical data
    let scale = 100.0; // Scale for floating-point values

    // Encrypt age field
    let ages: Vec<f64> = records.iter().map(|r| r.age as f64).collect();
    encrypted_data.insert("age".to_string(), fhe.encrypt_f64_vector(&ages, scale));

    // Encrypt glucose levels
    let glucose: Vec<f64> = records.iter().map(|r| r.glucose_level).collect();
    encrypted_data.insert(
        "glucose".to_string(),
        fhe.encrypt_f64_vector(&glucose, scale),
    );

    // Encrypt cholesterol levels
    let cholesterol: Vec<f64> = records.iter().map(|r| r.cholesterol_level).collect();
    encrypted_data.insert(
        "cholesterol".to_string(),
        fhe.encrypt_f64_vector(&cholesterol, scale),
    );

    // Encrypt marker (boolean) field
    let marker: Vec<bool> = records.iter().map(|r| r.marker_alpha).collect();
    encrypted_data.insert("marker".to_string(), fhe.encrypt_bool_vector(&marker));

    // For categorical data, we can use the encrypt_categorical method
    // Blood types
    let blood_types: Vec<String> = records.iter().map(|r| r.blood_type.clone()).collect();
    let encrypted_blood_types = fhe.encrypt_categorical(&blood_types);

    // Store each blood type vector seperately.
    for (i, blood_type) in encrypted_blood_types.categories.iter().enumerate() {
        let key = format!("blood_type_{}", blood_type);
        encrypted_data.insert(key, encrypted_blood_types.vectors[i].clone());
    }

    Ok(encrypted_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_generator::{generate_biosample_data, BiosampleRecord};
    use std::collections::HashSet;
    use tempfile::tempdir;

    /// Helper function to create test biosample records
    fn create_test_records() -> Vec<BiosampleRecord> {
        vec![
            BiosampleRecord {
                patient_id: "P001".to_string(),
                age: 25,
                gender: "Male".to_string(),
                blood_type: "A+".to_string(),
                glucose_level: 95.5,
                cholesterol_level: 180.0,
                marker_alpha: true,
                collection_date: "2023-01-01".to_string(),
                facility_id: 1,
            },
            BiosampleRecord {
                patient_id: "P002".to_string(),
                age: 45,
                gender: "Female".to_string(),
                blood_type: "O-".to_string(),
                glucose_level: 110.2,
                cholesterol_level: 220.5,
                marker_alpha: false,
                collection_date: "2023-01-02".to_string(),
                facility_id: 2,
            },
            BiosampleRecord {
                patient_id: "P003".to_string(),
                age: 65,
                gender: "Male".to_string(),
                blood_type: "B+".to_string(),
                glucose_level: 88.7,
                cholesterol_level: 160.3,
                marker_alpha: true,
                collection_date: "2023-01-03".to_string(),
                facility_id: 1,
            },
        ]
    }

    #[test]
    fn test_biosample_fhe_new() {
        let fhe = BiosampleFHE::new();
        // Test that keys are properly initialized (we can't directly test the keys,
        // but we can test that the struct is created without panicking)
        assert!(!std::ptr::addr_of!(fhe.client_key).is_null());
        assert!(!std::ptr::addr_of!(fhe.server_key).is_null());
    }

    #[test]
    fn test_biosample_fhe_default() {
        let fhe = BiosampleFHE::default();
        // Test that default implementation works
        assert!(!std::ptr::addr_of!(fhe.client_key).is_null());
        assert!(!std::ptr::addr_of!(fhe.server_key).is_null());
    }

    #[test]
    fn test_encrypt_decrypt_f64_vector() {
        let fhe = BiosampleFHE::new();
        let test_values = vec![1.5, 2.7, std::f64::consts::PI, -1.2, 0.0];
        let scale = 100.0;

        // Encrypt the values
        let encrypted = fhe.encrypt_f64_vector(&test_values, scale);
        
        // Verify encrypted vector structure
        assert_eq!(encrypted.length, test_values.len());
        assert_eq!(encrypted.data.len(), test_values.len());

        // Decrypt the values
        let decrypted = fhe.decrypt_f64_vector(&encrypted, scale);
        
        // Verify decrypted values match original (with some tolerance for floating point precision)
        assert_eq!(decrypted.len(), test_values.len());
        for (original, decrypted_val) in test_values.iter().zip(decrypted.iter()) {
            assert!((original - decrypted_val).abs() < 0.01, 
                   "Original: {}, Decrypted: {}", original, decrypted_val);
        }
    }

    #[test]
    fn test_encrypt_decrypt_f64_vector_empty() {
        let fhe = BiosampleFHE::new();
        let test_values: Vec<f64> = vec![];
        let scale = 100.0;

        let encrypted = fhe.encrypt_f64_vector(&test_values, scale);
        assert_eq!(encrypted.length, 0);
        assert_eq!(encrypted.data.len(), 0);

        let decrypted = fhe.decrypt_f64_vector(&encrypted, scale);
        assert_eq!(decrypted.len(), 0);
    }

    #[test]
    fn test_encrypt_decrypt_f64_vector_large_values() {
        let fhe = BiosampleFHE::new();
        let test_values = vec![1000.0, -500.0, 999.99];
        let scale = 10.0;

        let encrypted = fhe.encrypt_f64_vector(&test_values, scale);
        let decrypted = fhe.decrypt_f64_vector(&encrypted, scale);
        
        for (original, decrypted_val) in test_values.iter().zip(decrypted.iter()) {
            assert!((original - decrypted_val).abs() < 0.1, 
                   "Original: {}, Decrypted: {}", original, decrypted_val);
        }
    }

    #[test]
    fn test_encrypt_decrypt_bool_vector() {
        let fhe = BiosampleFHE::new();
        let test_values = vec![true, false, true, true, false];

        // Encrypt the values
        let encrypted = fhe.encrypt_bool_vector(&test_values);
        
        // Verify encrypted vector structure
        assert_eq!(encrypted.length, test_values.len());
        assert_eq!(encrypted.data.len(), test_values.len());

        // Decrypt the values
        let decrypted = fhe.decrypt_bool_vector(&encrypted);
        
        // Verify decrypted values match original
        assert_eq!(decrypted, test_values);
    }

    #[test]
    fn test_encrypt_decrypt_bool_vector_empty() {
        let fhe = BiosampleFHE::new();
        let test_values: Vec<bool> = vec![];

        let encrypted = fhe.encrypt_bool_vector(&test_values);
        assert_eq!(encrypted.length, 0);
        assert_eq!(encrypted.data.len(), 0);

        let decrypted = fhe.decrypt_bool_vector(&encrypted);
        assert_eq!(decrypted.len(), 0);
    }

    #[test]
    fn test_encrypt_decrypt_bool_vector_all_true() {
        let fhe = BiosampleFHE::new();
        let test_values = vec![true; 5];

        let encrypted = fhe.encrypt_bool_vector(&test_values);
        let decrypted = fhe.decrypt_bool_vector(&encrypted);
        
        assert_eq!(decrypted, test_values);
    }

    #[test]
    fn test_encrypt_decrypt_bool_vector_all_false() {
        let fhe = BiosampleFHE::new();
        let test_values = vec![false; 5];

        let encrypted = fhe.encrypt_bool_vector(&test_values);
        let decrypted = fhe.decrypt_bool_vector(&encrypted);
        
        assert_eq!(decrypted, test_values);
    }

    #[test]
    fn test_encrypt_categorical() {
        let fhe = BiosampleFHE::new();
        let test_values = vec![
            "A+".to_string(),
            "B+".to_string(),
            "A+".to_string(),
            "O-".to_string(),
            "B+".to_string(),
        ];

        let encrypted_categorical = fhe.encrypt_categorical(&test_values);
        
        // Verify categories are extracted correctly
        let expected_categories: HashSet<String> = test_values.iter().cloned().collect();
        let actual_categories: HashSet<String> = encrypted_categorical.categories.iter().cloned().collect();
        assert_eq!(actual_categories, expected_categories);
        
        // Verify number of vectors matches number of categories
        assert_eq!(encrypted_categorical.vectors.len(), encrypted_categorical.categories.len());
        
        // Verify each vector has the correct length
        for vector in &encrypted_categorical.vectors {
            assert_eq!(vector.length, test_values.len());
        }
    }

    #[test]
    fn test_encrypt_categorical_empty() {
        let fhe = BiosampleFHE::new();
        let test_values: Vec<String> = vec![];

        let encrypted_categorical = fhe.encrypt_categorical(&test_values);
        
        assert_eq!(encrypted_categorical.categories.len(), 0);
        assert_eq!(encrypted_categorical.vectors.len(), 0);
    }

    #[test]
    fn test_encrypt_categorical_single_category() {
        let fhe = BiosampleFHE::new();
        let test_values = vec!["A+".to_string(); 3];

        let encrypted_categorical = fhe.encrypt_categorical(&test_values);
        
        assert_eq!(encrypted_categorical.categories.len(), 1);
        assert_eq!(encrypted_categorical.categories[0], "A+");
        assert_eq!(encrypted_categorical.vectors.len(), 1);
        assert_eq!(encrypted_categorical.vectors[0].length, 3);
    }

    #[test]
    fn test_encrypt_categorical_consistency() {
        let fhe = BiosampleFHE::new();
        let test_values = vec![
            "Type1".to_string(),
            "Type2".to_string(),
            "Type1".to_string(),
        ];

        let encrypted_categorical = fhe.encrypt_categorical(&test_values);
        
        // Find the index of "Type1" in categories
        let type1_index = encrypted_categorical.categories.iter()
            .position(|x| x == "Type1").unwrap();
        
        // Decrypt the corresponding vector
        let type1_vector = fhe.decrypt_bool_vector(&encrypted_categorical.vectors[type1_index]);
        
        // Should be [true, false, true] for "Type1"
        assert_eq!(type1_vector, vec![true, false, true]);
    }

    #[test]
    fn test_server_key_access() {
        let fhe = BiosampleFHE::new();
        let server_key = fhe.server_key();
        
        // Test that we can access the server key
        assert!(!std::ptr::addr_of!(*server_key).is_null());
    }

    #[test]
    fn test_save_and_load_keys() {
        let fhe = BiosampleFHE::new();
        
        // Create temporary directory for test files
        let temp_dir = tempdir().unwrap();
        let client_key_path = temp_dir.path().join("client_key.bin");
        let server_key_path = temp_dir.path().join("server_key.bin");
        
        // Save keys
        let save_result = fhe.save_keys(&client_key_path, &server_key_path);
        assert!(save_result.is_ok());
        
        // Verify files were created
        assert!(client_key_path.exists());
        assert!(server_key_path.exists());
        
        // Load keys
        let loaded_fhe = BiosampleFHE::load_keys(&client_key_path, &server_key_path);
        assert!(loaded_fhe.is_ok());
        
        let loaded_fhe = loaded_fhe.unwrap();
        
        // Test that loaded keys work by encrypting and decrypting
        let test_values = vec![1.0, 2.0, 3.0];
        let scale = 100.0;
        
        let encrypted = loaded_fhe.encrypt_f64_vector(&test_values, scale);
        let decrypted = loaded_fhe.decrypt_f64_vector(&encrypted, scale);
        
        for (original, decrypted_val) in test_values.iter().zip(decrypted.iter()) {
            assert!((original - decrypted_val).abs() < 0.01);
        }
    }

    #[test]
    fn test_save_keys_invalid_path() {
        let fhe = BiosampleFHE::new();
        
        // Try to save to an invalid path
        let invalid_path = Path::new("/invalid/path/that/does/not/exist/key.bin");
        let result = fhe.save_keys(invalid_path, invalid_path);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_load_keys_nonexistent_files() {
        let nonexistent_path = Path::new("nonexistent_key.bin");
        let result = BiosampleFHE::load_keys(nonexistent_path, nonexistent_path);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_biosample_data() {
        let fhe = BiosampleFHE::new();
        let test_records = create_test_records();
        
        let encrypted_result = encrypt_biosample_data(&fhe, &test_records);
        assert!(encrypted_result.is_ok());
        
        let encrypted_data = encrypted_result.unwrap();
        
        // Verify all expected fields are present
        assert!(encrypted_data.contains_key("age"));
        assert!(encrypted_data.contains_key("glucose"));
        assert!(encrypted_data.contains_key("cholesterol"));
        assert!(encrypted_data.contains_key("marker"));
        
        // Verify blood type fields are present
        let blood_types: HashSet<String> = test_records.iter()
            .map(|r| r.blood_type.clone())
            .collect();
        
        for blood_type in blood_types {
            let key = format!("blood_type_{}", blood_type);
            assert!(encrypted_data.contains_key(&key), "Missing key: {}", key);
        }
        
        // Verify vector lengths
        assert_eq!(encrypted_data["age"].length, test_records.len());
        assert_eq!(encrypted_data["glucose"].length, test_records.len());
        assert_eq!(encrypted_data["cholesterol"].length, test_records.len());
        assert_eq!(encrypted_data["marker"].length, test_records.len());
    }

    #[test]
    fn test_encrypt_biosample_data_empty() {
        let fhe = BiosampleFHE::new();
        let test_records: Vec<BiosampleRecord> = vec![];
        
        let encrypted_result = encrypt_biosample_data(&fhe, &test_records);
        assert!(encrypted_result.is_ok());
        
        let encrypted_data = encrypted_result.unwrap();
        
        // Should still have the basic fields, but with zero length
        assert!(encrypted_data.contains_key("age"));
        assert!(encrypted_data.contains_key("glucose"));
        assert!(encrypted_data.contains_key("cholesterol"));
        assert!(encrypted_data.contains_key("marker"));
        
        assert_eq!(encrypted_data["age"].length, 0);
        assert_eq!(encrypted_data["glucose"].length, 0);
        assert_eq!(encrypted_data["cholesterol"].length, 0);
        assert_eq!(encrypted_data["marker"].length, 0);
    }

    #[test]
    fn test_encrypt_biosample_data_roundtrip() {
        let fhe = BiosampleFHE::new();
        let test_records = create_test_records();
        
        // Encrypt the data
        let encrypted_data = encrypt_biosample_data(&fhe, &test_records).unwrap();
        
        // Decrypt and verify age data
        let scale = 100.0;
        let decrypted_ages = fhe.decrypt_f64_vector(&encrypted_data["age"], scale);
        let expected_ages: Vec<f64> = test_records.iter().map(|r| r.age as f64).collect();
        
        for (expected, actual) in expected_ages.iter().zip(decrypted_ages.iter()) {
            assert!((expected - actual).abs() < 0.01);
        }
        
        // Decrypt and verify glucose data
        let decrypted_glucose = fhe.decrypt_f64_vector(&encrypted_data["glucose"], scale);
        let expected_glucose: Vec<f64> = test_records.iter().map(|r| r.glucose_level).collect();
        
        for (expected, actual) in expected_glucose.iter().zip(decrypted_glucose.iter()) {
            assert!((expected - actual).abs() < 0.01);
        }
        
        // Decrypt and verify marker data
        let decrypted_marker = fhe.decrypt_bool_vector(&encrypted_data["marker"]);
        let expected_marker: Vec<bool> = test_records.iter().map(|r| r.marker_alpha).collect();
        
        assert_eq!(decrypted_marker, expected_marker);
    }

    #[test]
    fn test_encrypt_biosample_data_with_generated_data() {
        let fhe = BiosampleFHE::new();
        
        // Generate test data using the data generator
        let generated_records = generate_biosample_data(10, 12345).unwrap();
        
        let encrypted_result = encrypt_biosample_data(&fhe, &generated_records);
        assert!(encrypted_result.is_ok());
        
        let encrypted_data = encrypted_result.unwrap();
        
        // Verify all vectors have the correct length
        for (key, vector) in &encrypted_data {
            assert_eq!(vector.length, generated_records.len(), 
                      "Vector {} has incorrect length", key);
        }
    }

    #[test]
    fn test_encrypted_vector_serialization() {
        let fhe = BiosampleFHE::new();
        let test_values = vec![1.0, 2.0, 3.0];
        let scale = 100.0;
        
        let encrypted = fhe.encrypt_f64_vector(&test_values, scale);
        
        // Test that EncryptedVector can be serialized and deserialized
        let serialized = serde_json::to_string(&encrypted).unwrap();
        let deserialized: EncryptedVector = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(encrypted.length, deserialized.length);
        assert_eq!(encrypted.data.len(), deserialized.data.len());
        
        // Verify that deserialized data can be decrypted correctly
        let decrypted = fhe.decrypt_f64_vector(&deserialized, scale);
        for (original, decrypted_val) in test_values.iter().zip(decrypted.iter()) {
            assert!((original - decrypted_val).abs() < 0.01);
        }
    }

    #[test]
    fn test_encrypted_categorical_serialization() {
        let fhe = BiosampleFHE::new();
        let test_values = vec!["A+".to_string(), "B+".to_string(), "A+".to_string()];
        
        let encrypted_categorical = fhe.encrypt_categorical(&test_values);
        
        // Test that EncryptedCategorical can be serialized and deserialized
        let serialized = serde_json::to_string(&encrypted_categorical).unwrap();
        let deserialized: EncryptedCategorical = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(encrypted_categorical.categories, deserialized.categories);
        assert_eq!(encrypted_categorical.vectors.len(), deserialized.vectors.len());
        
        for (original, deserialized_vec) in encrypted_categorical.vectors.iter()
            .zip(deserialized.vectors.iter()) {
            assert_eq!(original.length, deserialized_vec.length);
            assert_eq!(original.data.len(), deserialized_vec.data.len());
        }
    }

    #[test]
    fn test_biosample_fhe_clone() {
        let fhe = BiosampleFHE::new();
        let fhe_clone = fhe.clone();
        
        // Test that both instances can encrypt/decrypt independently
        let test_values = vec![1.0, 2.0, 3.0];
        let scale = 100.0;
        
        let encrypted_original = fhe.encrypt_f64_vector(&test_values, scale);
        let encrypted_clone = fhe_clone.encrypt_f64_vector(&test_values, scale);
        
        // Both should be able to decrypt their own encrypted data
        let decrypted_original = fhe.decrypt_f64_vector(&encrypted_original, scale);
        let decrypted_clone = fhe_clone.decrypt_f64_vector(&encrypted_clone, scale);
        
        // Results should match original values
        for (original, decrypted_val) in test_values.iter().zip(decrypted_original.iter()) {
            assert!((original - decrypted_val).abs() < 0.01);
        }
        for (original, decrypted_val) in test_values.iter().zip(decrypted_clone.iter()) {
            assert!((original - decrypted_val).abs() < 0.01);
        }
    }

    #[test]
    fn test_edge_case_zero_values() {
        let fhe = BiosampleFHE::new();
        let test_values = vec![0.0; 5];
        let scale = 100.0;
        
        let encrypted = fhe.encrypt_f64_vector(&test_values, scale);
        let decrypted = fhe.decrypt_f64_vector(&encrypted, scale);
        
        for decrypted_val in decrypted.iter() {
            assert!(decrypted_val.abs() < 0.01);
        }
    }

    #[test]
    fn test_edge_case_negative_values() {
        let fhe = BiosampleFHE::new();
        let test_values = vec![-1.0, -2.5, -10.0];
        let scale = 100.0;
        
        let encrypted = fhe.encrypt_f64_vector(&test_values, scale);
        let decrypted = fhe.decrypt_f64_vector(&encrypted, scale);
        
        for (original, decrypted_val) in test_values.iter().zip(decrypted.iter()) {
            assert!((original - decrypted_val).abs() < 0.01);
        }
    }
}
