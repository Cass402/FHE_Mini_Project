use fhe_mini_project::{
    computations::compute_encrypted_mean,
    data_generator::generate_biosample_data,
    encryption::BiosampleFHE,
    visualization::{plot_comparison, visualize_fhe_workflow},
};

use std::collections::HashMap;
use std::error::Error;
use std::io::{self, Write};
use std::path::Path;
use std::time::Instant;

/// Prints a formatted header with the given text centered.
///
/// The header consists of a line of '=' characters, the centered text,
/// and another line of '=' characters, creating a visually distinct section.
fn print_header(text: &str) {
    println!("\n{}", "=".repeat(80));
    println!("{:^80}", text);
    println!("{}", "=".repeat(80));
}

/// Pauses the program execution and waits for the user to press Enter.
///
/// This function displays a prompt to the user, flushes stdout to ensure
/// the prompt is displayed immediately, and then waits for the user to
/// press Enter before continuing program execution.
fn pause() {
    print!("\nPress Enter to continue...");
    io::stdout().flush().unwrap();
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).unwrap();
}

/// Runs an interactive demonstration of Fully Homomorphic Encryption (FHE) for biosample data analysis.
///
/// This demo guides users through the process of:
/// 1. Generating synthetic biosample data
/// 2. Encrypting the data using FHE
/// 3. Performing statistical analysis on the encrypted data
/// 4. Decrypting only the results (not the original data)
/// 5. Verifying the accuracy against plaintext computations
///
/// The demo includes visualizations of the results and workflow, saved to the 'outputs' directory.
///
/// # Returns
///
/// A `Result` indicating success or an error that occurred during the demo.
fn main() -> Result<(), Box<dyn Error>> {
    print_header("FHE Biosample Data Analysis Interactive Demo");

    println!("\nThis demo will guide you through the process of performing secure computations");
    println!("on encrypted biosample data using Fully Homomorphic Encryption (FHE).");
    println!("\nThe demo will:");
    println!("1. Generate synthetic biosample data");
    println!("2. Encrypt the data using FHE");
    println!("3. Perform statistical analysis on the encrypted data");
    println!("4. Decrypt only the results (not the original data)");
    println!("5. Verify the accuracy against plaintext computations");

    pause();

    // Step 1: Generate synthetic data
    print_header("Step 1: Generate Synthetic Biosample Data");

    println!("Generating 100 synthetic biosample records...");
    let records = generate_biosample_data(100, 42)?;

    println!("\nSample of generated data:");
    for record in records.iter().take(5) {
        println!(
            "Patient {}: Age={}, Blood Type={}, Glucose={:.1}, Cholesterol={:.1}, Marker={}",
            record.patient_id,
            record.age,
            record.blood_type,
            record.glucose_level,
            record.cholesterol_level,
            if record.marker_alpha {
                "Positive"
            } else {
                "Negative"
            }
        );
    }
    println!("... [plus {} more records]", records.len() - 5);

    pause();

    // Step 2: Initialize FHE system and encrypt data
    print_header("Step 2: Initialize FHE System and Encrypt Data");

    println!("Initializing the FHE system...");
    let start = Instant::now();
    let fhe = BiosampleFHE::new();
    println!(
        "FHE system initialized in {:.2} seconds",
        start.elapsed().as_secs_f64()
    );

    println!("\nEncrypting numerical data...");
    let start = Instant::now();

    // Extract and scale numerical data
    let scale = 100.0; // Scale for floating-point values

    // Encrypt age
    let ages: Vec<f64> = records.iter().map(|r| r.age as f64).collect();
    let encrypted_age = fhe.encrypt_f64_vector(&ages, scale);
    println!("Encrypted ages");

    // Encrypt glucose levels
    let glucose: Vec<f64> = records.iter().map(|r| r.glucose_level).collect();
    let encrypted_glucose = fhe.encrypt_f64_vector(&glucose, scale);
    println!("Encrypted glucose levels");

    // Encrypt cholesterol
    let cholesterol: Vec<f64> = records.iter().map(|r| r.cholesterol_level).collect();
    let encrypted_cholesterol = fhe.encrypt_f64_vector(&cholesterol, scale);
    println!("Encrypted cholesterol values");

    println!(
        "\nEncryption completed in {:.2} seconds",
        start.elapsed().as_secs_f64()
    );

    pause();

    // Step 3: Perform computations on encrypted data
    print_header("Step 3: Perform Computations on Encrypted Data");

    println!("Computing statistics on encrypted data...");
    let start = Instant::now();

    // Compute average age
    println!("\nComputing average age on encrypted data...");
    let compute_start = Instant::now();
    let encrypted_avg_age = compute_encrypted_mean(&encrypted_age, fhe.server_key())?;
    let age_time = compute_start.elapsed();
    println!("Computation took {:.2} seconds", age_time.as_secs_f64());

    // Compute average glucose level
    println!("\nComputing average glucose level on encrypted data...");
    let compute_start = Instant::now();
    let encrypted_avg_glucose = compute_encrypted_mean(&encrypted_glucose, fhe.server_key())?;
    let glucose_time = compute_start.elapsed();
    println!("Computation took {:.2} seconds", glucose_time.as_secs_f64());

    // Compute average cholesterol samples
    println!("\nComputing average cholesterol level on encrypted data...");
    let compute_start = Instant::now();
    let encrypted_high_cholesterol =
        compute_encrypted_mean(&encrypted_cholesterol, fhe.server_key())?;
    let cholesterol_time = compute_start.elapsed();
    println!(
        "Computation took {:.2} seconds",
        cholesterol_time.as_secs_f64()
    );

    println!(
        "\nAll computations completed in {:.2} seconds",
        start.elapsed().as_secs_f64()
    );
    println!("\nIMPORTANT: These computations were performed directly on encrypted data!");
    println!("At no point was the original data decrypted during the computation process.");

    pause();

    // Step 4: Decrypt and verify results
    print_header("Step 4: Decrypt and Verify Results");

    println!("Now we'll decrypt ONLY the computation results (not the original data)");
    println!("and compare with plaintext computations for verification.");

    // Calculate plaintext results for comparison
    let plaintext_avg_age = ages.iter().sum::<f64>() / ages.len() as f64;
    let plaintext_avg_glucose = glucose.iter().sum::<f64>() / glucose.len() as f64;
    let plaintext_high_cholesterol = cholesterol.iter().filter(|&&c| c > 200.0).count() as f64;

    // Decrypt and verify average age
    println!("\nDecrypting average age result...");
    let decrypted_age_raw = fhe.decrypt_f64_vector(&encrypted_avg_age, scale);
    let decrypted_avg_age = decrypted_age_raw[0] / records.len() as f64;
    let age_error = (decrypted_avg_age - plaintext_avg_age).abs();
    let age_error_pct = age_error / plaintext_avg_age * 100.0;

    println!("Plaintext average age: {:.2}", plaintext_avg_age);
    println!("Encrypted+decrypted average age: {:.2}", decrypted_avg_age);
    println!("Error: {:.4} ({:.2}%)", age_error, age_error_pct);

    // Decrypt and verify average glucose
    println!("\nDecrypting average glucose result...");
    let decrypted_glucose_raw = fhe.decrypt_f64_vector(&encrypted_avg_glucose, scale);
    let decrypted_avg_glucose = decrypted_glucose_raw[0] / records.len() as f64;
    let glucose_error = (decrypted_avg_glucose - plaintext_avg_glucose).abs();
    let glucose_error_pct = glucose_error / plaintext_avg_glucose * 100.0;

    println!("Plaintext average glucose: {:.2}", plaintext_avg_glucose);
    println!(
        "Encrypted+decrypted average glucose: {:.2}",
        decrypted_avg_glucose
    );
    println!("Error: {:.4} ({:.2}%)", glucose_error, glucose_error_pct);

    // Decrypt and verify high cholesterol count
    println!("\nDecrypting high cholesterol count result...");
    let decrypted_high_chol_raw = fhe.decrypt_f64_vector(&encrypted_high_cholesterol, scale);
    let decrypted_high_cholesterol = decrypted_high_chol_raw[0];
    let chol_error = (decrypted_high_cholesterol - plaintext_high_cholesterol).abs();

    println!(
        "Plaintext high cholesterol count: {:.0}",
        plaintext_high_cholesterol
    );
    println!(
        "Encrypted+decrypted high cholesterol count: {:.2}",
        decrypted_high_cholesterol
    );
    println!("Error: {:.4}", chol_error);

    // Store results for visualization
    let mut plaintext_results = HashMap::new();
    plaintext_results.insert("Average Age".to_string(), plaintext_avg_age);
    plaintext_results.insert("Average Glucose".to_string(), plaintext_avg_glucose);
    plaintext_results.insert(
        "High Cholesterol Count".to_string(),
        plaintext_high_cholesterol,
    );

    let mut decrypted_results = HashMap::new();
    decrypted_results.insert("Average Age".to_string(), decrypted_avg_age);
    decrypted_results.insert("Average Glucose".to_string(), decrypted_avg_glucose);
    decrypted_results.insert(
        "High Cholesterol Count".to_string(),
        decrypted_high_cholesterol,
    );

    pause();

    // Step 5: Visualize results
    print_header("Step 5: Visualize Results");

    println!("Generating visualizations to show FHE results compared to plaintext...");

    // Ensure output directory exists
    std::fs::create_dir_all("outputs")?;

    // Plot comparison chart
    println!("Creating comparison chart...");
    plot_comparison(
        &plaintext_results,
        &decrypted_results,
        "FHE vs Plaintext Results",
        Path::new("outputs/interactive_results.png"),
    )?;

    // Visualize FHE workflow
    println!("Creating FHE workflow visualization...");
    visualize_fhe_workflow(Path::new("outputs/interactive_workflow.png"))?;

    println!("\nVisualizations have been saved to the 'outputs' directory:");
    println!("- outputs/interactive_results.png");
    println!("- outputs/interactive_workflow.png");

    // Final summary
    print_header("Demo Summary");

    println!("This demonstration has shown how Fully Homomorphic Encryption (FHE)");
    println!("can be used to perform computations on encrypted biosample data while");
    println!("preserving privacy and maintaining accuracy.");

    println!("\nKey takeaways:");
    println!("1. FHE allows computation directly on encrypted data without decryption");
    println!("2. Only the final results need to be decrypted, not the original data");
    println!("3. Results are accurate with minimal error due to the encryption process");
    println!("4. This approach enables privacy-preserving data analysis for biospecimens");

    println!("\nIn an AminoChain context, this technology would allow:");
    println!("- Secure sharing of biospecimen data while maintaining patient privacy");
    println!("- Compliance with regulations like HIPAA and GDPR");
    println!("- Enabling collaborative research without exposing sensitive information");
    println!("- Supporting patient data ownership by minimizing data exposure");

    print_header("End of Interactive Demo");
    println!("\nThank you for exploring the FHE Biosample Demo!");

    Ok(())
}
