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
