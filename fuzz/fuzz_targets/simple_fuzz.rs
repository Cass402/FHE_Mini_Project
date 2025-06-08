use fhe_mini_project::data_generator::{generate_biosample_data, BiosampleRecord};
use fhe_mini_project::encryption::{encrypt_biosample_data, BiosampleFHE};

fn main() {
    // Simple deterministic test cases
    let test_cases = vec![
        // Empty data
        vec![],
        // Single record
        vec![1, 2, 3, 4, 5],
        // Multiple records
        vec![10, 20, 30, 40, 50, 60, 70, 80, 90, 100],
        // Edge cases
        vec![0, 255],
        // Random data
        (0..50).collect::<Vec<u8>>(),
    ];

    println!("🧪 Running Simple Fuzz Tests");
    println!("============================");

    let mut passed = 0;
    let mut total = 0;

    for (i, test_data) in test_cases.iter().enumerate() {
        total += 1;
        print!("Test {}: ", i + 1);

        match run_simple_test(test_data) {
            Ok(_) => {
                println!("✓ PASSED");
                passed += 1;
            }
            Err(e) => {
                println!("✗ FAILED - {}", e);
            }
        }
    }

    // Test with generated data
    for num_samples in [1, 5, 10] {
        for seed in [12345, 54321, 98765] {
            total += 1;
            print!("Generated data test (samples: {}, seed: {}): ", num_samples, seed);

            match test_generated_data(num_samples, seed) {
                Ok(_) => {
                    println!("✓ PASSED");
                    passed += 1;
                }
                Err(e) => {
                    println!("✗ FAILED - {}", e);
                }
            }
        }
    }

    println!("\n📊 Results: {}/{} tests passed", passed, total);

    if passed == total {
        println!("🎉 All simple fuzz tests passed!");
        std::process::exit(0);
    } else {
        println!("❌ Some tests failed");
        std::process::exit(1);
    }
}

fn run_simple_test(data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    if data.is_empty() {
        // Test empty data handling
        let fhe = BiosampleFHE::new();
        let empty_records: Vec<BiosampleRecord> = vec![];
        let _encrypted = encrypt_biosample_data(&fhe, &empty_records)?;
        return Ok(());
    }

    // Create FHE instance
    let fhe = BiosampleFHE::new();

    // Test basic encryption/decryption with simple data
    let f64_values: Vec<f64> = data.iter().map(|&x| x as f64).collect();
    let bool_values: Vec<bool> = data.iter().map(|&x| x % 2 == 0).collect();
    let categorical_values: Vec<String> = data.iter()
        .map(|&x| format!("Category{}", x % 4))
        .collect();

    // Test f64 vector encryption/decryption
    let scale = 100.0;
    let encrypted_f64 = fhe.encrypt_f64_vector(&f64_values, scale);
    let decrypted_f64 = fhe.decrypt_f64_vector(&encrypted_f64, scale);

    // Verify decryption accuracy
    for (original, decrypted) in f64_values.iter().zip(decrypted_f64.iter()) {
        if (original - decrypted).abs() > 0.1 {
            return Err(format!("F64 decryption mismatch: {} vs {}", original, decrypted).into());
        }
    }

    // Test bool vector encryption/decryption
    let encrypted_bool = fhe.encrypt_bool_vector(&bool_values);
    let decrypted_bool = fhe.decrypt_bool_vector(&encrypted_bool);

    if bool_values != decrypted_bool {
        return Err("Bool decryption mismatch".into());
    }

    // Test categorical encryption
    let _encrypted_categorical = fhe.encrypt_categorical(&categorical_values);

    Ok(())
}

fn test_generated_data(num_samples: usize, seed: u64) -> Result<(), Box<dyn std::error::Error>> {
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

    // Test decryption of a few fields
    let scale = 100.0;
    let decrypted_ages = fhe.decrypt_f64_vector(&encrypted_data["age"], scale);
    let expected_ages: Vec<f64> = records.iter().map(|r| r.age as f64).collect();

    for (expected, actual) in expected_ages.iter().zip(decrypted_ages.iter()) {
        if (expected - actual).abs() > 0.1 {
            return Err(format!("Age decryption mismatch: {} vs {}", expected, actual).into());
        }
    }

    Ok(())
}