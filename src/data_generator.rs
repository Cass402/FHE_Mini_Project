/// Data generator for synthetic biosample data
/// This module generates synthetic biosample data for testing and development purposes.
/// It includes functions to generate random values for various biosample attributes.
// Required libraries
use chrono::{Duration, Utc}; // For generating random dates
use csv::Writer; // For writing CSV files
use rand::prelude::*; // For generating random numbers
use rand_distr::{Distribution, Normal}; // For generating normally distributed random numbers
use serde::{Serialize, Deserialize}; // For serializing and deserializing data (e.g., to/from CSV)
use std::error::Error; // For error handling
use std::fs::File; // For file operations
use std::path::Path; // For path operations

/// Represents a biosample record with patient and medical information
/// 
/// This struct contains various attributes of a biosample including patient identifiers,
/// demographic information, medical measurements, and collection metadata.
/// It is used for generating and storing synthetic biosample data.
#[derive(Serialize, Deserialize, Debug)]
pub struct BiosampleRecord {
    pub patient_id: String,
    pub age: u32,
    pub gender: String,
    pub blood_type: String,
    pub glucose_level: f64,
    pub cholesterol_level: f64,
    pub marker_alpha: bool,
    pub collection_date: String,
    pub facility_id: u32
}

/// Generates a vector of synthetic biosample records for testing and development
///
/// This function creates a specified number of biosample records with randomized but realistic
/// values for patient attributes such as age, gender, blood type, glucose levels, etc.
/// The random number generator is seeded to ensure reproducible results.
///
/// # Arguments
/// * `num_samples` - The number of biosample records to generate
/// * `seed` - A seed value for the random number generator to ensure reproducibility
///
/// # Returns
/// * `Result<Vec<BiosampleRecord>, Box<dyn Error>>` - A vector of generated biosample records or an error
pub fn generate_biosample_data(num_samples: usize, seed: u64) -> Result<Vec<BiosampleRecord>, Box<dyn Error>> {
    // Initialize a random number generator with a seed
    let mut random_num_gen = StdRng::seed_from_u64(seed);

    // Distribution for normally distributed age, glucose, and cholesterol levels
    let age_dist = Normal::new(45.0, 15.0)?; // Mean 45, StdDev 15
    let glucose_dist = Normal::new(100.0, 25.0)?; // Mean 100, StdDev 25
    let cholesterol_dist = Normal::new(180.0, 40.0)?; // Mean 180, StdDev 40

    // Approximate real-world frequency of blood types distribution
    let blood_types = ["A+", "A-", "B+", "B-", "AB+", "AB-", "O+", "O-"];
    let blood_type_weights = [0.34, 0.06, 0.09, 0.02, 0.03, 0.01, 0.38, 0.07]; // Approximate frequencies

    let base_date = Utc::now() - Duration::days(365); // Base date for collection

    // Generate the biosample records
    let mut biosample_records = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        // Generate patient age between 18 and 90
        let age_f64 = f64::round(age_dist.sample(&mut random_num_gen));
        let age = age_f64.clamp(18.0, 90.0) as u32;
        
        // Generate patient gender
        let gender = if random_num_gen.gen_bool(0.5) { "Male" } else { "Female" };
        // Generate blood type based on weighted random selection
        let blood_type_index = {
            let mut cumulative = 0.0;
            let r: f64 = random_num_gen.gen();
            let mut selected = 0;

            for(i, &weight) in blood_type_weights.iter().enumerate() {
                cumulative += weight;
                if r < cumulative {
                    selected = i;
                    break;
                }
            }
            selected
        };

        let blood_type = blood_types[blood_type_index];

        // Generate glucose and cholesterol levels
        let glucose_level = glucose_dist.sample(&mut random_num_gen);
        let cholesterol_level = cholesterol_dist.sample(&mut random_num_gen);

        // Generate marker alpha (boolean)
        let marker_alpha = random_num_gen.gen_bool(0.3); // 30% chance of being true

        // Generate collection date within the last year
        let days_offset = random_num_gen.gen_range(0..365);
        let collection_date = (base_date + Duration::days(days_offset)).format("%Y-%m-%d").to_string();

        // Generate facility ID
        let facility_id = random_num_gen.gen_range(1..6); 

        // Create a new biosample record
        let biosample_record = BiosampleRecord {
            patient_id: format!("P{:06}", i + 1), // Patient ID
            age,
            gender: gender.to_string(),
            blood_type: blood_type.to_string(),
            glucose_level,
            cholesterol_level,
            marker_alpha,
            collection_date,
            facility_id
        };

        // Add the record to the vector
        biosample_records.push(biosample_record);
    }

    // Return the generated biosample records
    Ok(biosample_records)
}

/// Saves a collection of biosample records to a CSV file.
/// 
/// # Arguments
/// 
/// * `biosample_records` - A slice of BiosampleRecord structs to be saved
/// * `path` - The file path where the CSV will be written
/// 
/// # Returns
/// 
/// * `Result<(), Box<dyn Error>>` - Ok(()) on success, or an error if the operation fails
pub fn save_biosample_data(biosample_records: &[BiosampleRecord], path: &Path) -> Result<(), Box<dyn Error>> {
    // Create a CSV writer
    let file = File::create(path)?;
    let mut wtr = Writer::from_writer(file);

    // Write the records to the CSV file
    for record in biosample_records {
        wtr.serialize(record)?;
    }

    // Flush and finalize the writer
    wtr.flush()?;
    Ok(())
}

/// Loads biosample records from a CSV file.
/// 
/// # Arguments
/// 
/// * `path` - The file path from which to read the CSV
/// 
/// # Returns
/// 
/// * `Result<Vec<BiosampleRecord>, Box<dyn Error>>` - A vector of BiosampleRecord on success, or an error if the operation fails
pub fn load_biosample_data(path: &Path) -> Result<Vec<BiosampleRecord>, Box<dyn Error>> {
    // Open the CSV file
    let file = File::open(path)?;
    let mut rdr = csv::Reader::from_reader(file);

    // Deserialize the records into a vector
    let mut biosample_records = Vec::new();
    for result in rdr.deserialize() {
        let record: BiosampleRecord = result?;
        biosample_records.push(record);
    }

    // Return the loaded biosample records
    Ok(biosample_records)
}