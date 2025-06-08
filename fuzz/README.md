# Fuzz Testing for FHE Mini Project

This directory contains comprehensive fuzz tests for the FHE Mini Project's encryption module.

## Overview

The fuzz testing suite includes:

1. **Simple Fuzz Tests** - Basic deterministic tests with edge cases
2. **Comprehensive Fuzz Tests** - Advanced tests including serialization, key persistence, and stress testing
3. **Extended Fuzz Testing** - Multiple iterations with random data generation

## Running Fuzz Tests

### Method 1: Using the Fuzz Script (Recommended)

Run the complete fuzz testing suite:

```bash
cd fuzz
./run_fuzz_tests.sh
```

This script will:
- Build all fuzz targets
- Run simple fuzz tests
- Run comprehensive fuzz tests  
- Run extended fuzz testing with 50 iterations
- Provide detailed results and summary

### Method 2: Running Individual Fuzz Tests

You can also run individual fuzz test binaries:

```bash
cd fuzz

# Run simple fuzz tests
cargo run --bin simple_fuzz --release

# Run comprehensive fuzz tests
cargo run --bin comprehensive_fuzz --release
```

### Method 3: Using cargo-fuzz (Requires Nightly Rust)

If you have nightly Rust installed, you can use cargo-fuzz:

```bash
# List available fuzz targets
cargo fuzz list

# Run a specific fuzz target (requires nightly Rust)
rustup run nightly cargo fuzz run simple_fuzz
```

Note: cargo-fuzz requires nightly Rust and may have compatibility issues on some systems.

## Test Coverage

### Simple Fuzz Tests
- Empty data handling
- Single and multiple record processing
- Edge cases with boundary values
- Generated data validation
- Basic encryption/decryption round-trips

### Comprehensive Fuzz Tests
- Edge cases (empty vectors, single elements, extreme values)
- Large dataset processing
- Serialization/deserialization of encrypted data
- Key persistence (save/load functionality)
- Multiple FHE instances
- Random data generation and encryption
- Different scaling factors

### Extended Fuzz Testing
- 50 iterations of comprehensive tests
- Random data generation for each iteration
- Stress testing with various input sizes
- Robustness validation

## Test Results

A successful run will show:
```
Summary:
- Simple fuzz tests: ✓ PASSED
- Comprehensive fuzz tests: ✓ PASSED  
- Extended fuzz testing (50 iterations): ✓ PASSED

The encryption module has been thoroughly tested and appears robust!
```

## Dependencies

The fuzz tests use:
- `arbitrary` - For generating structured test data
- `tempfile` - For temporary file operations in key persistence tests
- `serde_json` - For serialization testing

## Troubleshooting

### Script Permission Issues
If you get permission denied:
```bash
chmod +x run_fuzz_tests.sh
```

### Build Issues
Make sure you're in the fuzz directory and the parent project builds successfully:
```bash
cd ..
cargo build --release
cd fuzz
cargo build --release
```

### Timeout Issues
The script removes timeout commands for macOS compatibility. If tests seem to hang, you can manually interrupt them with Ctrl+C.

## Adding New Fuzz Tests

To add new fuzz tests:

1. Create a new binary target in `Cargo.toml`
2. Add the corresponding Rust file in `fuzz_targets/`
3. Update `run_fuzz_tests.sh` to include the new test
4. Follow the existing patterns for error handling and test structure

## Performance Notes

- Tests are designed to complete quickly while providing good coverage
- Large dataset tests are limited to reasonable sizes to avoid timeouts
- Extended testing uses 50 iterations by default (configurable in the script)
- All tests use release builds for better performance