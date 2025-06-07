#[cfg(test)]
mod proptests {
    use crate::encryption::*;
    use proptest::prelude::*;
    use std::collections::HashSet;

    // Custom strategies for generating test data with smaller sizes for performance
    
    /// Strategy for generating reasonable f64 values for biosample data
    fn reasonable_f64() -> impl Strategy<Value = f64> {
        prop_oneof![
            // Normal range values (smaller range for performance)
            -100.0..100.0,
            // Edge cases
            Just(0.0),
            Just(-0.0),
            // Small values
            -1.0..1.0,
        ]
    }

    /// Strategy for generating small f64 vectors for performance
    fn f64_vector() -> impl Strategy<Value = Vec<f64>> {
        prop::collection::vec(reasonable_f64(), 0..5) // Reduced size for performance
    }

    /// Strategy for generating small boolean vectors for performance
    fn bool_vector() -> impl Strategy<Value = Vec<bool>> {
        prop::collection::vec(any::<bool>(), 0..5) // Reduced size for performance
    }

    /// Strategy for generating small categorical string vectors
    fn categorical_vector() -> impl Strategy<Value = Vec<String>> {
        let categories = vec!["A+", "B+", "O+", "AB+"]; // Reduced categories for performance
        prop::collection::vec(
            prop::sample::select(categories).prop_map(|s| s.to_string()),
            0..3 // Reduced size for performance
        )
    }

    /// Strategy for generating scale factors
    fn scale_factor() -> impl Strategy<Value = f64> {
        prop_oneof![
            Just(1.0),
            Just(10.0),
            Just(100.0),
        ]
    }

    proptest! {
        // Reduce the number of test cases for performance
        #![proptest_config(ProptestConfig::with_cases(10))]

        /// Property: F64 vector encryption/decryption should be reversible
        #[test]
        fn prop_f64_encrypt_decrypt_roundtrip(
            values in f64_vector(),
            scale in scale_factor()
        ) {
            let fhe = BiosampleFHE::new();
            
            // Skip if scale is too small to avoid precision issues
            prop_assume!(scale >= 1.0);
            
            let encrypted = fhe.encrypt_f64_vector(&values, scale);
            let decrypted = fhe.decrypt_f64_vector(&encrypted, scale);
            
            // Verify length preservation
            prop_assert_eq!(encrypted.length, values.len());
            prop_assert_eq!(decrypted.len(), values.len());
            
            // Verify values are approximately equal (accounting for floating point precision)
            for (original, decrypted_val) in values.iter().zip(decrypted.iter()) {
                let tolerance = 1.0 / scale + 0.01; // Scale-dependent tolerance
                prop_assert!(
                    (original - decrypted_val).abs() < tolerance,
                    "Original: {}, Decrypted: {}, Tolerance: {}", 
                    original, decrypted_val, tolerance
                );
            }
        }

        /// Property: Boolean vector encryption/decryption should be perfectly reversible
        #[test]
        fn prop_bool_encrypt_decrypt_roundtrip(values in bool_vector()) {
            let fhe = BiosampleFHE::new();
            
            let encrypted = fhe.encrypt_bool_vector(&values);
            let decrypted = fhe.decrypt_bool_vector(&encrypted);
            
            // Boolean encryption should be exact
            prop_assert_eq!(encrypted.length, values.len());
            prop_assert_eq!(decrypted, values);
        }

        /// Property: Categorical encryption should preserve category information
        #[test]
        fn prop_categorical_encrypt_preserves_categories(values in categorical_vector()) {
            let fhe = BiosampleFHE::new();
            
            let encrypted_categorical = fhe.encrypt_categorical(&values);
            
            // Extract unique categories from input
            let expected_categories: HashSet<String> = values.iter().cloned().collect();
            let actual_categories: HashSet<String> = encrypted_categorical.categories.iter().cloned().collect();
            
            // Categories should match
            prop_assert_eq!(actual_categories, expected_categories);
            
            // Number of vectors should match number of categories
            prop_assert_eq!(encrypted_categorical.vectors.len(), encrypted_categorical.categories.len());
            
            // Each vector should have the same length as input
            for vector in &encrypted_categorical.vectors {
                prop_assert_eq!(vector.length, values.len());
            }
        }

        /// Property: Empty vectors should be handled correctly
        #[test]
        fn prop_empty_vectors_handled_correctly(scale in scale_factor()) {
            let fhe = BiosampleFHE::new();
            
            // Test empty f64 vector
            let empty_f64: Vec<f64> = vec![];
            let encrypted_f64 = fhe.encrypt_f64_vector(&empty_f64, scale);
            let decrypted_f64 = fhe.decrypt_f64_vector(&encrypted_f64, scale);
            
            prop_assert_eq!(encrypted_f64.length, 0);
            prop_assert_eq!(decrypted_f64.len(), 0);
            
            // Test empty bool vector
            let empty_bool: Vec<bool> = vec![];
            let encrypted_bool = fhe.encrypt_bool_vector(&empty_bool);
            let decrypted_bool = fhe.decrypt_bool_vector(&encrypted_bool);
            
            prop_assert_eq!(encrypted_bool.length, 0);
            prop_assert_eq!(decrypted_bool.len(), 0);
            
            // Test empty categorical vector
            let empty_categorical: Vec<String> = vec![];
            let encrypted_categorical = fhe.encrypt_categorical(&empty_categorical);
            
            prop_assert_eq!(encrypted_categorical.categories.len(), 0);
            prop_assert_eq!(encrypted_categorical.vectors.len(), 0);
        }

        /// Property: Encrypted vectors should be serializable and deserializable
        #[test]
        fn prop_encrypted_vector_serialization(values in f64_vector(), scale in scale_factor()) {
            prop_assume!(scale >= 1.0);
            prop_assume!(!values.is_empty()); // Skip empty vectors for this test
            
            let fhe = BiosampleFHE::new();
            let encrypted = fhe.encrypt_f64_vector(&values, scale);
            
            // Test JSON serialization
            let serialized = serde_json::to_string(&encrypted);
            prop_assert!(serialized.is_ok());
            
            let deserialized: Result<EncryptedVector, _> = serde_json::from_str(&serialized.unwrap());
            prop_assert!(deserialized.is_ok());
            
            let deserialized = deserialized.unwrap();
            prop_assert_eq!(encrypted.length, deserialized.length);
            prop_assert_eq!(encrypted.data.len(), deserialized.data.len());
            
            // Verify that deserialized data can be decrypted correctly
            let decrypted = fhe.decrypt_f64_vector(&deserialized, scale);
            prop_assert_eq!(decrypted.len(), values.len());
            
            for (original, decrypted_val) in values.iter().zip(decrypted.iter()) {
                let tolerance = 1.0 / scale + 0.01;
                prop_assert!((original - decrypted_val).abs() < tolerance);
            }
        }

        /// Property: Scale factor should affect precision but not correctness
        #[test]
        fn prop_scale_factor_affects_precision(
            values in prop::collection::vec(-10.0..10.0, 1..3), // Smaller range and size
        ) {
            let fhe = BiosampleFHE::new();
            let scale1 = 1.0;
            let scale2 = 100.0;
            
            let encrypted1 = fhe.encrypt_f64_vector(&values, scale1);
            let encrypted2 = fhe.encrypt_f64_vector(&values, scale2);
            
            let decrypted1 = fhe.decrypt_f64_vector(&encrypted1, scale1);
            let decrypted2 = fhe.decrypt_f64_vector(&encrypted2, scale2);
            
            // Both should be approximately correct
            for ((original, dec1), dec2) in values.iter().zip(decrypted1.iter()).zip(decrypted2.iter()) {
                let error1 = (original - dec1).abs();
                let error2 = (original - dec2).abs();
                
                // Both should be reasonably close
                prop_assert!(error1 < 2.0, "Low scale error too large: {}", error1);
                prop_assert!(error2 < 0.2, "High scale error too large: {}", error2);
            }
        }
    }
}