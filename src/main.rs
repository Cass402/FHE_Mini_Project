mod computations;
mod data_generator;
mod encryption;
mod visualization;

// Required libraries
use std::collections::HashMap; // HashMap is used for storing key-value pairs
use std::error::Error; // Error trait is used for handling errors
use std::fs; // fs module is used for file system operations
use std::path::{Path, PathBuf}; // Path and PathBuf are used for handling file paths
use std::time::Instant; // Instant is used for measuring time

use clap::{ArgAction, Parser}; // clap is used for command-line argument parsing

// Importing the modules
use computations::{compute_encrypted_mean, run_biosample_analysis, verify_computation};
use data_generator::{generate_biosample_data, load_biosample_data, save_biosample_data};
use encryption::{encrypt_biosample_data, BiosampleFHE};
use visualization::{plot_comparison, plot_performance_metrics, visualize_fhe_workflow};

/// FHE Demo for secure computation on biosample data
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Number of biosample records to generate
    #[clap(short, long, default_value_t = 1000)]
    samples: usize,

    /// Random seed for reproducibility
    #[clap(short, long, default_value_t = 42)]
    seed: u64,

    /// Regenerate data even if it exists
    #[clap(short, long, action=ArgAction::SetTrue)]
    regenerate: bool,

    /// Skip visualization generation
    #[clap(long, action=ArgAction::SetTrue)]
    no_visualize: bool,

    /// Output directory for visualization
    #[clap(short, long, default_value = "outputs")]
    output_dir: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    // Parse command-line arguments
    let args = Args::parse();

    println!("{}", "=".repeat(80));
    println!(
        "{:^80}",
        "Fully Homophoric Encryption Demo for Biosample Data"
    );
    println!("{}", "=".repeat(80));

    // Create output directory if it doesn't exist
    let data_dir = Path::new("data");
    fs::create_dir_all(data_dir)?;

    let output_dir = PathBuf::from(&args.output_dir);
    fs::create_dir_all(&output_dir)?;

    // Generate or load biosample data
    let data_file = data_dir.join("biosample_data.csv");

    let records = if !data_file.exists() || args.regenerate {
        println!("\n[1/5] Generating synthetic biosample data...");
        let records = generate_biosample_data(args.samples, args.seed)?;
        save_biosample_data(&records, &data_file)?;
        records
    } else {
        println!("\n[1/5] Loading existing biosample data...");
        let records = load_biosample_data(&data_file)?;
        println!("{} biosample records loaded.", records.len());
        records
    };

    // Display the first 5 records
    println!("\nSample data preview:");
    for (i, record) in records.iter().enumerate().take(5) {
        println!(
            "Record #{}: Age: {}, Blood Type: {}, Glucose: {:.1}, Cholesterol: {:.1}",
            i + 1,
            record.age,
            record.blood_type,
            record.glucose_level,
            record.cholesterol_level
        );
    }
    println!("...[{}] more records", records.len() - 5);

    // Initialize FHE encryption and encrypt data
    println!("\n[2/5] Encrypting biosample data using FHE...");
    let encryption_start = Instant::now();
    let fhe = BiosampleFHE::new();

    // Encrypt the biosample data
    println!("Encrypting numerical and categorical data...");
    let encrypted_data = encrypt_biosample_data(&fhe, &records)?;

    let encryption_time = encryption_start.elapsed();
    println!(
        "Encryption completed in {:.2}",
        encryption_time.as_secs_f64()
    );

    // Perform computations on encrypted data
    println!("\n[3/5] Performing computations on encrypted data...");
    let computation_start = Instant::now();

    // Track performance metrics
    let mut performance_metrics = HashMap::new();

    // Average Age
    println!("Computing average age...");
    let start = Instant::now();
    let encrypted_avg_age = match encrypted_data.get("age") {
        Some(age_data) => compute_encrypted_mean(age_data, fhe.server_key())?,
        None => return Err("Age data not found".into()),
    };
    performance_metrics.insert("Average Age".to_string(), start.elapsed());

    // Average Glucose Level
    println!("Computing average glucose level...");
    let start = Instant::now();
    let encrypted_avg_glucose = match encrypted_data.get("glucose_level") {
        Some(glucose_data) => compute_encrypted_mean(glucose_data, fhe.server_key())?,
        None => return Err("Glucose data not found".into()),
    };
    performance_metrics.insert("Average Glucose Level".to_string(), start.elapsed());

    // Average Cholesterol Level
    println!("Computing average cholesterol level...");
    let start = Instant::now();
    let encrypted_avg_cholesterol = match encrypted_data.get("cholesterol_level") {
        Some(cholesterol_data) => compute_encrypted_mean(cholesterol_data, fhe.server_key())?,
        None => return Err("Cholesterol data not found".into()),
    };
    performance_metrics.insert("Average Cholesterol Level".to_string(), start.elapsed());

    // Run full analysis
    println!("Running complete biosample analysis...");
    let start = Instant::now();
    let _encrypted_results = run_biosample_analysis(&encrypted_data, fhe.server_key())?;
    performance_metrics.insert("Full Analysis".to_string(), start.elapsed());

    let computation_time = computation_start.elapsed();
    println!(
        "Computation completed in {:.2}",
        computation_time.as_secs_f64()
    );

    // Decrypt and verify results
    println!("\n[4/5] Decrypting and verifying results...");

    // Calculate plaintext_results for verification
    let scale = 100.0;
    let plaintext_results = {
        let mut results = HashMap::new();

        // Average Age
        let avg_age = records.iter().map(|r| r.age as f64).sum::<f64>() / records.len() as f64;
        results.insert("Average Age".to_string(), avg_age);

        // Average Glucose Level
        let avg_glucose =
            records.iter().map(|r| r.glucose_level).sum::<f64>() / records.len() as f64;
        results.insert("Average Glucose Level".to_string(), avg_glucose);

        // Average Cholesterol Level
        let avg_cholesterol =
            records.iter().map(|r| r.cholesterol_level).sum::<f64>() / records.len() as f64;
        results.insert("Average Cholesterol Level".to_string(), avg_cholesterol);

        results
    };

    // Decrypt results
    let decryption_start = Instant::now();

    let mut encrypted_result_map = HashMap::new();
    encrypted_result_map.insert("Average Age".to_string(), encrypted_avg_age);
    encrypted_result_map.insert("Average Glucose Level".to_string(), encrypted_avg_glucose);
    encrypted_result_map.insert(
        "Average Cholesterol Level".to_string(),
        encrypted_avg_cholesterol,
    );

    let mut decrypted_results = HashMap::new();
    for (key, enc_result) in &encrypted_result_map {
        println!("Decrypting {}...", key);

        // Decrypt
        let decrypted_raw = fhe.decrypt_f64_vector(enc_result, scale);

        // Process the result
        let decrypted = decrypted_raw[0] / records.len() as f64;
        decrypted_results.insert(key.clone(), decrypted);

        // Get plaintext result for verification
        let plaintext = plaintext_results[key];
        let is_verified = verify_computation(decrypted, plaintext, 0.05);
        let error = (decrypted - plaintext).abs();
        let error_pct = if plaintext != 0.0 {
            error / plaintext * 100.0
        } else {
            0.0
        };

        println!("Plaintext result: {:.2}", plaintext);
        println!("Decrypted result: {:.2}", decrypted);
        println!(
            "Verification status: {}",
            if is_verified { "PASS" } else { "FAIL" }
        );
        println!("Error: {:.2}", error);
        println!("Error percentage: {:.2}%", error_pct);
    }

    let decryption_time = decryption_start.elapsed();
    println!(
        "Decryption completed in {:.2}",
        decryption_time.as_secs_f64()
    );

    // Generate visiualizations
    if !args.no_visualize {
        println!("\n[5/5] Generating visualizations...");

        // Create output directory for visualizations
        fs::create_dir_all(&output_dir)?;

        // Plot comparison of plaintext and encrypted results
        println!("Plotting comparison of plaintext and encrypted results...");
        plot_comparison(
            &plaintext_results,
            &decrypted_results,
            "FHE vs Plaintext Computation Results",
            &output_dir.join("results_comparision.png"),
        )?;

        // Plot performance metrics
        println!("  Creating performance metrics chart...");
        let mut perf_metrics = performance_metrics.clone();
        perf_metrics.insert("Encryption".to_string(), encryption_time);
        perf_metrics.insert("Decryption".to_string(), decryption_time);
        plot_performance_metrics(
            &perf_metrics,
            "FHE Operation Performance",
            &output_dir.join("performance_metrics.png"),
        )?;

        // Plot FHE workflow
        println!("  Creating FHE workflow visualization...");
        visualize_fhe_workflow(&output_dir.join("fhe_workflow.png"))?;

        println!("✓ Visualizations saved to {}/", args.output_dir);
    } else {
        println!("\n[5/5] Visualization skipped");
    }

    // Summary
    println!("\n{}", "=".repeat(80));
    println!("{:^80}", "Demo Summary");
    println!("{}", "=".repeat(80));
    println!("Data size: {} biosample records", records.len());
    println!(
        "Total time: {:.2} seconds",
        (encryption_time + computation_time + decryption_time).as_secs_f64()
    );
    println!(
        "  - Encryption: {:.2} seconds",
        encryption_time.as_secs_f64()
    );
    println!(
        "  - Computation: {:.2} seconds",
        computation_time.as_secs_f64()
    );
    println!(
        "  - Decryption: {:.2} seconds",
        decryption_time.as_secs_f64()
    );
    println!("\nAccuracy:");
    for key in plaintext_results.keys() {
        let error_pct = (decrypted_results[key] - plaintext_results[key]).abs()
            / plaintext_results[key]
            * 100.0;
        println!("  - {}: {:.2}% error", key, error_pct);
    }

    println!("\n{}", "=".repeat(80));
    println!("{:^80}", "FHE Demo Complete!");
    println!("{}", "=".repeat(80));

    Ok(())
}
