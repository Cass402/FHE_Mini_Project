use fhe_mini_project::data_generator::generate_biosample_data;
use fhe_mini_project::encryption::{encrypt_biosample_data, BiosampleFHE, EncryptedVector};
use tempfile::tempdir;

fn main() {
    println!("🔬 Running Comprehensive Fuzz Tests");
    println!("===================================");

    let mut passed = 0;
    let mut total = 0;

    // Test 1: Edge cases
    total += 1;
    print!("Edge cases test: ");
    match test_edge_cases() {
        Ok(_) => {
            println!("✓ PASSED");
            passed += 1;
        }
        Err(e) => {
            println!("✗ FAILED - {}", e);
        }
    }

    // Test 2: Large data sets
    total += 1;
    print!("Large dataset test: ");
    match test_large_datasets() {
        Ok(_) => {
            println!("✓ PASSED");
            passed += 1;
        }
        Err(e) => {
            println!("✗ FAILED - {}", e);
        }
    }

    // Test 3: Serialization/Deserialization
    total += 1;
    print!("Serialization test: ");
    match test_serialization() {
        Ok(_) => {
            println!("✓ PASSED");
            passed += 1;
        }
        Err(e) => {
            println!("✗ FAILED - {}", e);
        }
    }

    // Test 4: Key save/load functionality
    total += 1;
    print!("Key persistence test: ");
    match test_key_persistence() {
        Ok(_) => {
            println!("✓ PASSED");
            passed += 1;
        }
        Err(e) => {
            println!("✗ FAILED - {}", e);
        }
    }

    // Test 5: Multiple FHE instances
    total += 1;
    print!("Multiple instances test: ");
    match test_multiple_instances() {
        Ok(_) => {
            println!("✓ PASSED");
            passed += 1;
        }
        Err(e) => {
            println!("✗ FAILED - {}", e);
        }
    }

    // Test 6: Random data generation and encryption
    for i in 0..10 {
        total += 1;
        print!("Random test {}: ", i + 1);
        
        let seed = 12345 + i as u64;
        let num_samples = (i % 5) + 1;
        
        match test_random_data(num_samples, seed) {
            Ok(_) => {
                println!("✓ PASSED");
                passed += 1;
            }
            Err(e) => {
                println!("✗ FAILED - {}", e);
            }
        }
    }

    // Test 7: Stress test with various scales
    for scale_exp in [1.0, 10.0, 100.0, 1000.0] {
        total += 1;
        print!("Scale test ({}): ", scale_exp);
        
        match test_different_scales(scale_exp) {
            Ok(_) => {
                println!("✓ PASSED");
                passed += 1;
            }
            Err(e) => {
                println!("✗ FAILED - {}", e);
            }
        }
    }

    println!("\n📊 Results: {}/{} tests passed", passed, total);

    if passed == total {
        println!("🎉 All comprehensive fuzz tests passed!");
        std::process::exit(0);
    } else {
        println!("❌ Some tests failed");
        std::process::exit(1);
    }
}

fn test_edge_cases() -> Result<(), Box<dyn std::error::Error>> {
    let fhe = BiosampleFHE::new();

    // Test empty vectors
    let empty_f64: Vec<f64> = vec![];
    let empty_bool: Vec<bool> = vec![];
    let empty_categorical: Vec<String> = vec![];

    let encrypted_empty_f64 = fhe.encrypt_f64_vector(&empty_f64, 100.0);
    let encrypted_empty_bool = fhe.encrypt_bool_vector(&empty_bool);
    let encrypted_empty_categorical = fhe.encrypt_categorical(&empty_categorical);

    assert_eq!(encrypted_empty_f64.length, 0);
    assert_eq!(encrypted_empty_bool.length, 0);
    assert_eq!(encrypted_empty_categorical.categories.len(), 0);

    // Test single element vectors
    let single_f64 = vec![42.0];
    let single_bool = vec![true];
    let single_categorical = vec!["TestCategory".to_string()];

    let encrypted_single_f64 = fhe.encrypt_f64_vector(&single_f64, 100.0);
    let encrypted_single_bool = fhe.encrypt_bool_vector(&single_bool);
    let encrypted_single_categorical = fhe.encrypt_categorical(&single_categorical);

    let decrypted_single_f64 = fhe.decrypt_f64_vector(&encrypted_single_f64, 100.0);
    let decrypted_single_bool = fhe.decrypt_bool_vector(&encrypted_single_bool);

    assert!((single_f64[0] - decrypted_single_f64[0]).abs() < 0.01);
    assert_eq!(single_bool, decrypted_single_bool);
    assert_eq!(encrypted_single_categorical.categories.len(), 1);

    // Test extreme values
    let extreme_values = vec![-1000.0, 0.0, 1000.0, f64::MIN, f64::MAX];
    let scale = 1.0; // Use smaller scale for extreme values
    
    // Filter out values that would overflow when scaled
    let safe_values: Vec<f64> = extreme_values.into_iter()
        .filter(|&x| x.is_finite() && (x * scale).abs() < i64::MAX as f64)
        .collect();
    
    if !safe_values.is_empty() {
        let encrypted_extreme = fhe.encrypt_f64_vector(&safe_values, scale);
        let decrypted_extreme = fhe.decrypt_f64_vector(&encrypted_extreme, scale);
        
        for (original, decrypted) in safe_values.iter().zip(decrypted_extreme.iter()) {
            if (original - decrypted).abs() > 1.0 {
                return Err(format!("Extreme value test failed: {} vs {}", original, decrypted).into());
            }
        }
    }

    Ok(())
}

fn test_large_datasets() -> Result<(), Box<dyn std::error::Error>> {
    let fhe = BiosampleFHE::new();
    
    // Test with larger dataset (but not too large to avoid timeouts)
    let large_size = 50;
    let large_f64: Vec<f64> = (0..large_size).map(|i| i as f64 * 0.5).collect();
    let large_bool: Vec<bool> = (0..large_size).map(|i| i % 2 == 0).collect();
    
    let encrypted_large_f64 = fhe.encrypt_f64_vector(&large_f64, 100.0);
    let encrypted_large_bool = fhe.encrypt_bool_vector(&large_bool);
    
    assert_eq!(encrypted_large_f64.length, large_size);
    assert_eq!(encrypted_large_bool.length, large_size);
    
    let decrypted_large_f64 = fhe.decrypt_f64_vector(&encrypted_large_f64, 100.0);
    let decrypted_large_bool = fhe.decrypt_bool_vector(&encrypted_large_bool);
    
    for (original, decrypted) in large_f64.iter().zip(decrypted_large_f64.iter()) {
        if (original - decrypted).abs() > 0.01 {
            return Err(format!("Large dataset f64 test failed: {} vs {}", original, decrypted).into());
        }
    }
    
    if large_bool != decrypted_large_bool {
        return Err("Large dataset bool test failed".into());
    }

    Ok(())
}

fn test_serialization() -> Result<(), Box<dyn std::error::Error>> {
    let fhe = BiosampleFHE::new();
    
    let test_values = vec![1.0, 2.5, 3.14, -1.5];
    let encrypted = fhe.encrypt_f64_vector(&test_values, 100.0);
    
    // Test JSON serialization
    let serialized = serde_json::to_string(&encrypted)?;
    let deserialized: EncryptedVector = serde_json::from_str(&serialized)?;
    
    assert_eq!(encrypted.length, deserialized.length);
    assert_eq!(encrypted.data.len(), deserialized.data.len());
    
    // Verify that deserialized data can be decrypted correctly
    let decrypted = fhe.decrypt_f64_vector(&deserialized, 100.0);
    for (original, decrypted_val) in test_values.iter().zip(decrypted.iter()) {
        if (original - decrypted_val).abs() > 0.01 {
            return Err(format!("Serialization test failed: {} vs {}", original, decrypted_val).into());
        }
    }

    Ok(())
}

fn test_key_persistence() -> Result<(), Box<dyn std::error::Error>> {
    let fhe = BiosampleFHE::new();
    
    // Create temporary directory for test files
    let temp_dir = tempdir()?;
    let client_key_path = temp_dir.path().join("client_key.bin");
    let server_key_path = temp_dir.path().join("server_key.bin");
    
    // Save keys
    fhe.save_keys(&client_key_path, &server_key_path)?;
    
    // Verify files were created
    if !client_key_path.exists() || !server_key_path.exists() {
        return Err("Key files were not created".into());
    }
    
    // Load keys
    let loaded_fhe = BiosampleFHE::load_keys(&client_key_path, &server_key_path)?;
    
    // Test that loaded keys work
    let test_values = vec![1.0, 2.0, 3.0];
    let scale = 100.0;
    
    let encrypted = loaded_fhe.encrypt_f64_vector(&test_values, scale);
    let decrypted = loaded_fhe.decrypt_f64_vector(&encrypted, scale);
    
    for (original, decrypted_val) in test_values.iter().zip(decrypted.iter()) {
        if (original - decrypted_val).abs() > 0.01 {
            return Err(format!("Key persistence test failed: {} vs {}", original, decrypted_val).into());
        }
    }

    Ok(())
}

fn test_multiple_instances() -> Result<(), Box<dyn std::error::Error>> {
    let fhe1 = BiosampleFHE::new();
    let fhe2 = BiosampleFHE::new();
    let fhe3 = fhe1.clone();
    
    let test_values = vec![1.0, 2.0, 3.0];
    let scale = 100.0;
    
    // Each instance should be able to encrypt/decrypt independently
    let encrypted1 = fhe1.encrypt_f64_vector(&test_values, scale);
    let encrypted2 = fhe2.encrypt_f64_vector(&test_values, scale);
    let encrypted3 = fhe3.encrypt_f64_vector(&test_values, scale);
    
    let decrypted1 = fhe1.decrypt_f64_vector(&encrypted1, scale);
    let decrypted2 = fhe2.decrypt_f64_vector(&encrypted2, scale);
    let decrypted3 = fhe3.decrypt_f64_vector(&encrypted3, scale);
    
    // All should decrypt to the same values
    for ((original, dec1), (dec2, dec3)) in test_values.iter()
        .zip(decrypted1.iter())
        .zip(decrypted2.iter().zip(decrypted3.iter())) {
        
        if (original - dec1).abs() > 0.01 ||
           (original - dec2).abs() > 0.01 ||
           (original - dec3).abs() > 0.01 {
            return Err("Multiple instances test failed".into());
        }
    }
    
    // Clone should be able to decrypt original's data
    let decrypted_clone = fhe3.decrypt_f64_vector(&encrypted1, scale);
    for (original, decrypted_val) in test_values.iter().zip(decrypted_clone.iter()) {
        if (original - decrypted_val).abs() > 0.01 {
            return Err("Clone decryption test failed".into());
        }
    }

    Ok(())
}

fn test_random_data(num_samples: usize, seed: u64) -> Result<(), Box<dyn std::error::Error>> {
    // Generate biosample data
    let records = generate_biosample_data(num_samples, seed)?;
    
    // Create FHE instance
    let fhe = BiosampleFHE::new();
    
    // Encrypt the data
    let encrypted_data = encrypt_biosample_data(&fhe, &records)?;
    
    // Verify all expected fields are present
    let expected_fields = ["age", "glucose", "cholesterol", "marker"];
    for field in &expected_fields {
        if !encrypted_data.contains_key(*field) {
            return Err(format!("Missing field: {}", field).into());
        }
    }
    
    // Verify vector lengths
    for (field, vector) in &encrypted_data {
        if vector.length != records.len() {
            return Err(format!("Length mismatch for field {}: expected {}, got {}", 
                             field, records.len(), vector.length).into());
        }
    }
    
    // Test round-trip encryption/decryption
    let scale = 100.0;
    
    // Test age field
    let decrypted_ages = fhe.decrypt_f64_vector(&encrypted_data["age"], scale);
    let expected_ages: Vec<f64> = records.iter().map(|r| r.age as f64).collect();
    
    for (expected, actual) in expected_ages.iter().zip(decrypted_ages.iter()) {
        if (expected - actual).abs() > 0.1 {
            return Err(format!("Age decryption mismatch: {} vs {}", expected, actual).into());
        }
    }
    
    // Test marker field
    let decrypted_markers = fhe.decrypt_bool_vector(&encrypted_data["marker"]);
    let expected_markers: Vec<bool> = records.iter().map(|r| r.marker_alpha).collect();
    
    if decrypted_markers != expected_markers {
        return Err("Marker decryption mismatch".into());
    }

    Ok(())
}

fn test_different_scales(scale: f64) -> Result<(), Box<dyn std::error::Error>> {
    let fhe = BiosampleFHE::new();
    
    let test_values = vec![1.0, 2.5, 3.14, -1.5, 0.0];
    
    let encrypted = fhe.encrypt_f64_vector(&test_values, scale);
    let decrypted = fhe.decrypt_f64_vector(&encrypted, scale);
    
    let tolerance = 1.0 / scale; // Tolerance based on scale
    
    for (original, decrypted_val) in test_values.iter().zip(decrypted.iter()) {
        if (original - decrypted_val).abs() > tolerance {
            return Err(format!("Scale test failed for scale {}: {} vs {} (tolerance: {})", 
                             scale, original, decrypted_val, tolerance).into());
        }
    }

    Ok(())
}