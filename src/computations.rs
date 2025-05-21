/// Perform computations on tfhe encrypted data
/// This module contains the functions that perform computations on the encrypted data
/// The computations are performed using the TFHE library
// Required libraries
use std::collections::HashMap;
use std::error::Error;
use tfhe::integer::{ServerKey, SignedRadixCiphertext};

// Import the encryption module
use crate::encryption::EncryptedVector;

/// Deserializes a vector of encrypted ciphertexts from an EncryptedVector
///
/// This function converts the binary data in an EncryptedVector back into
/// SignedRadixCiphertext objects that can be used for computation.
///
/// # Arguments
/// * `encrypted_vector` - The EncryptedVector containing serialized ciphertexts
///
/// # Returns
/// A vector of deserialized SignedRadixCiphertext objects
fn deserialize_ciphertexts(encrypted_vector: &EncryptedVector) -> Vec<SignedRadixCiphertext> {
    encrypted_vector
        .data
        .iter()
        .map(|data| bincode::deserialize(data).unwrap())
        .collect()
}

/// Serializes a vector of SignedRadixCiphertext objects into an EncryptedVector
///
/// This function converts SignedRadixCiphertext objects into binary data
/// that can be stored in an EncryptedVector for transmission or storage.
///
/// # Arguments
/// * `ciphertexts` - A vector of SignedRadixCiphertext objects to serialize
///
/// # Returns
/// An EncryptedVector containing the serialized ciphertexts
fn serialize_ciphertexts(ciphertexts: Vec<SignedRadixCiphertext>) -> EncryptedVector {
    let data: Vec<Vec<u8>> = ciphertexts
        .iter()
        .map(|ciphertext| bincode::serialize(ciphertext).unwrap())
        .collect();
    EncryptedVector {
        data,
        length: ciphertexts.len(),
    }
}

/// Computes the sum of encrypted values in a vector
///
/// This function takes an encrypted vector, deserializes the ciphertexts,
/// and computes their sum using homomorphic addition.
///
/// # Arguments
/// * `encrypted_vector` - The EncryptedVector containing serialized ciphertexts
/// * `server_key` - The ServerKey used for homomorphic operations
///
/// # Returns
/// * `Result<SignedRadixCiphertext, Box<dyn Error>>` - The encrypted sum or an error
///   if the vector is empty or if addition fails
pub fn compute_encrypted_sum(
    encrypted_vector: &EncryptedVector,
    server_key: &ServerKey,
) -> Result<SignedRadixCiphertext, Box<dyn Error>> {
    // Deserialize the ciphertexts
    let ciphertexts = deserialize_ciphertexts(encrypted_vector);

    // check if the ciphertexts are empty
    if ciphertexts.is_empty() {
        return Err("Cannot compute sum of empty vector".into());
    }

    // Start with the first ciphertext
    let mut sum = ciphertexts[0].clone();

    // Iterate over the rest of the ciphertexts and add them to the sum
    for ciphertext in &ciphertexts[1..] {
        sum = server_key.checked_add(&sum, ciphertext)?;
    }

    Ok(sum)
}

/// Computes the mean of encrypted values in a vector
///
/// This function calculates the sum of encrypted values and returns it
/// in a serialized form. The actual division to compute the mean is performed
/// after decryption, as homomorphic division is complex.
///
/// # Arguments
/// * `encrypted_vector` - The EncryptedVector containing serialized ciphertexts
/// * `server_key` - The ServerKey used for homomorphic operations
///
/// # Returns
/// * `Result<EncryptedVector, Box<dyn Error>>` - The encrypted sum in a serialized form,
///   or an error if computation fails
pub fn compute_encrypted_mean(
    encrypted_vector: &EncryptedVector,
    server_key: &ServerKey,
) -> Result<EncryptedVector, Box<dyn Error>> {
    // Compute the sum
    let sum = compute_encrypted_sum(encrypted_vector, server_key)?;

    // For division, we'll use a trick: instead of dividing the encrypted sum (which is complex),
    // we'll return the sum and divide after decryption
    // In a more advanced implementation, we would use bootstrapping and server-side division

    Ok(serialize_ciphertexts(vec![sum]))
}

/*
/// Count values in a vector that are approximately above a threshold
/// Note: This is an approximation as direct comparisons are not easily done in FHE
pub fn compute_encrypted_threshold_count(encrypted_vector: &EncryptedVector, server_key: &ServerKey, threshold_scaled: i64) -> Result<EncryptedVector, Box<dyn Error>> {
    // Deserialize the ciphertexts
    let ciphertexts = deserialize_ciphertexts(encrypted_vector);

    // For each value, we'll compute a score that's higher when the value exceeds the threshold
    // This is a simplified approach and not a true comparison

    // Encrypt the threshold
    let threshold_cipher = server_key.create_trivial_radix(threshold_scaled, 8);

    // For each cipthertext, compute it is greater than the threshold
    let mut count_ciphers = Vec::new();

    for cipher in ciphertexts {
        // Subtract the threshold from the ciphertext
        let diff = server_key.checked_sub(&cipher, &threshold_cipher)?;

        // If difference is positive, it's above threshold
        // We'll encode a "soft" count using the sign bit trick
        // In real FHE, this would use more sophisticated polynomials

        // This is a simplification - in practice you'd use a better approach
        let shifted = server_key.unchecked_scalar_right_shift(&diff, 7);
        count_ciphers.push(shifted);
    }

    // Sum the counts
    let mut count_sum = count_ciphers[0].clone();
    for cipher in &count_ciphers[1..] {
        count_sum = server_key.checked_add(&count_sum, cipher)?;
    }

    Ok(serialize_ciphertexts(vec![count_sum]))

}
*/

/// Computes the count of each category in a map of encrypted category vectors
///
/// # Arguments
/// * `encrypted_categories` - A map of category names to encrypted vectors where each vector
///   contains binary indicators (0 or 1) for category membership
/// * `server_key` - The server key used for homomorphic operations
///
/// # Returns
/// * A map of category names to encrypted counts, where each count is the sum of the binary indicators
///
/// # Errors
/// * Returns an error if any of the homomorphic operations fail
pub fn compute_encrypted_category_counts(
    encrypted_categories: &HashMap<String, EncryptedVector>,
    server_key: &ServerKey,
) -> Result<HashMap<String, EncryptedVector>, Box<dyn Error>> {
    let mut category_counts = HashMap::new();

    for (category, encrypted_vector) in encrypted_categories {
        if category.starts_with("blood_type_") {
            let sum = compute_encrypted_sum(encrypted_vector, server_key)?;
            category_counts.insert(category.clone(), serialize_ciphertexts(vec![sum]));
        }
    }

    Ok(category_counts)
}

/// Verifies that an encrypted computation result is close enough to the plaintext result
///
/// # Arguments
/// * `encrypted_result` - The result obtained through homomorphic encryption
/// * `plaintext_result` - The expected result computed on plaintext data
/// * `tolerance` - The relative error tolerance (as a fraction)
///
/// # Returns
/// * `true` if the encrypted result is within the specified tolerance of the plaintext result
pub fn verify_computation(encrypted_result: f64, plaintext_result: f64, tolerance: f64) -> bool {
    // Check if the encrypted result is within the tolerance of the plaintext result
    (encrypted_result - plaintext_result).abs() <= tolerance * plaintext_result.abs()
}

/// Runs analysis on encrypted biosample data
///
/// # Arguments
/// * `encrypted_data` - A map of feature names to encrypted vectors containing the data
/// * `server_key` - The server key used for homomorphic operations
///
/// # Returns
/// * A map of analysis results, including average age, glucose, cholesterol, and blood type counts
///
/// # Errors
/// * Returns an error if any of the homomorphic operations fail
pub fn run_biosample_analysis(
    encrypted_data: &HashMap<String, EncryptedVector>,
    server_key: &ServerKey,
) -> Result<HashMap<String, EncryptedVector>, Box<dyn Error>> {
    let mut results = HashMap::new();

    // Compute average age
    if let Some(age_data) = encrypted_data.get("age") {
        let mean = compute_encrypted_mean(age_data, server_key)?;
        results.insert("avg_age".to_string(), mean);
    }

    // Compute average glucose levels
    if let Some(glucose_data) = encrypted_data.get("glucose") {
        let mean = compute_encrypted_mean(glucose_data, server_key)?;
        results.insert("avg_glucose".to_string(), mean);
    }

    // Compute average cholesterol levels
    if let Some(cholesterol_data) = encrypted_data.get("cholesterol") {
        let mean = compute_encrypted_mean(cholesterol_data, server_key)?;
        results.insert("avg_cholesterol".to_string(), mean);
    }

    // Count blood types
    let blood_type_keys: Vec<String> = encrypted_data
        .keys()
        .filter(|k| k.starts_with("blood_type_"))
        .cloned()
        .collect();

    if !blood_type_keys.is_empty() {
        let blood_type_data: HashMap<String, EncryptedVector> = blood_type_keys
            .iter()
            .map(|k| (k.clone(), encrypted_data[k].clone()))
            .collect();
        let blood_counts = compute_encrypted_category_counts(&blood_type_data, server_key)?;

        for (key, value) in blood_counts {
            results.insert(key, value);
        }
    }

    Ok(results)
}
