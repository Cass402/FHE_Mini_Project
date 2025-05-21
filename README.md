# Fully Homomorphic Encryption (FHE) for Biosample Data

A demonstration project showcasing the power of FHE for privacy-preserving computation on sensitive biosample data, implemented in Rust.

![FHE Workflow](outputs/fhe_workflow.png)

## Overview

This project demonstrates the application of Fully Homomorphic Encryption (FHE) to biobanking and medical data analysis. It shows how statistical computations can be performed directly on encrypted biosample data without ever decrypting the underlying sensitive information.

### Purpose

AminoChain is building a decentralized biobanking platform that emphasizes:

- Patient data ownership
- Privacy and security
- Regulatory compliance (HIPAA, GDPR)
- Transparent, efficient data sharing

This demo showcases how FHE can enable secure computation while preserving these values.

## Features

- Generation of realistic synthetic biosample metadata
- Fully homomorphic encryption of numerical and categorical data
- Statistical computations on encrypted data:
  - Mean/average calculations
  - Threshold-based counting
  - Categorical data analysis
- Visualization of results and performance metrics
- Comparison between encrypted and plaintext computations
- Interactive demo through command-line interface

## Technical Details

### FHE Implementation

This project uses the TFHE-rs library (Rust implementation of TFHE) for fully homomorphic encryption. The implementation supports:

- Encrypting floating-point and categorical data
- Homomorphic addition and multiplication operations
- Approximate comparison operations
- Result verification and error analysis

### Dependencies

This project relies on:

- `tfhe` - Rust implementation of the TFHE fully homomorphic encryption scheme
- `csv`, `serde` - For data handling
- `rand`, `rand_distr` - For synthetic data generation
- `plotters` - For visualization
- `clap` - For command-line interface
- Other utility crates for error handling and logging

## Installation

### Prerequisites

- Rust 1.60+ and Cargo (install via [rustup](https://rustup.rs/))
- Standard build tools for your platform

### Building the Project

```bash
# Clone the repository
git clone https://github.com/yourusername/fhe-biosample-demo.git
cd fhe-biosample-demo

# Build the project
cargo build --release
```

## Running the Demo

### Command-Line Demo

```bash
# Run with default settings
cargo run --release

# Run with custom settings
cargo run --release -- --samples 2000 --seed 123

# See all available options
cargo run --release -- --help
```

### Interactive Demo

```bash
# Run the interactive demo
cargo run --release --example interactive_demo
```

## Project Structure

```
fhe-biosample-demo/
├── Cargo.toml                # Rust package manifest
├── README.md                 # Project documentation
├── data/
│   └── sample_data.csv       # Synthetic biosample data
├── src/
│   ├── main.rs               # Entry point
│   ├── data_generator.rs     # Creates synthetic data
│   ├── encryption.rs         # FHE encryption/decryption
│   ├── computations.rs       # FHE operations
│   ├── visualization.rs      # Result visualization
│   └── utils.rs              # Helper functions
├── examples/
│   └── interactive_demo.rs   # Interactive demo example
├── tests/                    # Unit tests
└── outputs/                  # Generated visualizations
```

## Understanding FHE and Its Applications

### What is Fully Homomorphic Encryption?

Fully Homomorphic Encryption (FHE) is a form of encryption that allows computations to be performed directly on encrypted data without requiring decryption first. The results of these computations, when decrypted, match the results that would have been obtained by performing the same operations on the unencrypted data.

### Why FHE for Biosample Data?

Biosample data contains sensitive personal and medical information that must be protected. Traditional approaches require decrypting data for analysis, creating potential privacy vulnerabilities. FHE enables:

1. **Privacy-Preserving Analytics**: Researchers can analyze data without accessing raw information
2. **Secure Multi-Party Computation**: Multiple institutions can collaborate without sharing sensitive data
3. **Regulatory Compliance**: Helps meet HIPAA, GDPR, and other regulatory requirements
4. **Patient Control**: Supports AminoChain's mission of patient data ownership

### Limitations and Considerations

1. **Performance Overhead**: FHE operations are computationally intensive and slower than plaintext operations
2. **Operation Constraints**: Complex operations like division and comparison are challenging in FHE
3. **Noise Management**: FHE introduces computational noise that must be managed
4. **Implementation Complexity**: Proper implementation requires cryptographic expertise

## Relevance to AminoChain

This demonstration aligns with AminoChain's mission by showing how:

1. **Patient Privacy**: Sensitive biosample data can be utilized while remaining encrypted
2. **Secure Sharing**: Data can be shared for research without compromising privacy
3. **Transparency**: Analysis can be conducted with full transparency about what computations are performed
4. **Regulatory Compliance**: Processing can comply with regulations without limiting utility

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- TFHE-rs team for providing the Rust implementation of TFHE
- The broader FHE research community for advancing privacy-preserving computation techniques
