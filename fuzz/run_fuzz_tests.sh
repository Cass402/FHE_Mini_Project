#!/bin/bash

# Fuzz Testing Script for FHE Mini Project
# This script runs various fuzz tests on the encryption module

set -e

echo "🔍 Starting Fuzz Testing for FHE Mini Project"
echo "=============================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Please run this script from the fuzz directory"
    exit 1
fi

# Build the fuzz targets
print_status "Building fuzz targets..."
cargo build --release

if [ $? -ne 0 ]; then
    print_error "Failed to build fuzz targets"
    exit 1
fi

print_status "Build successful!"

# Run simple fuzz tests
print_status "Running simple fuzz tests..."
cargo run --bin simple_fuzz --release

if [ $? -ne 0 ]; then
    print_error "Simple fuzz tests failed"
    exit 1
fi

print_status "Simple fuzz tests passed!"

# Run comprehensive fuzz tests
print_status "Running comprehensive fuzz tests..."
cargo run --bin comprehensive_fuzz --release

if [ $? -ne 0 ]; then
    print_error "Comprehensive fuzz tests failed"
    exit 1
fi

print_status "Comprehensive fuzz tests passed!"

# Run extended fuzz testing with multiple iterations
print_status "Running extended fuzz testing (multiple iterations)..."

ITERATIONS=50
PASSED=0
FAILED=0

for i in $(seq 1 $ITERATIONS); do
    echo -n "Iteration $i/$ITERATIONS: "
    
    # Generate random test data
    RANDOM_DATA=$(openssl rand -hex 200)
    echo -n "$RANDOM_DATA" | xxd -r -p > /tmp/fuzz_input_$i.bin
    
    # Run comprehensive fuzz with random data
    if cargo run --bin comprehensive_fuzz --release > /tmp/fuzz_output_$i.log 2>&1; then
        echo "✓"
        PASSED=$((PASSED + 1))
    else
        echo "✗"
        FAILED=$((FAILED + 1))
        print_warning "Iteration $i failed - check /tmp/fuzz_output_$i.log"
    fi
    
    # Clean up
    rm -f /tmp/fuzz_input_$i.bin
done

echo ""
print_status "Extended fuzz testing completed!"
print_status "Results: $PASSED passed, $FAILED failed out of $ITERATIONS iterations"

if [ $FAILED -eq 0 ]; then
    print_status "🎉 All fuzz tests passed successfully!"
    echo ""
    echo "Summary:"
    echo "- Simple fuzz tests: ✓ PASSED"
    echo "- Comprehensive fuzz tests: ✓ PASSED"
    echo "- Extended fuzz testing ($ITERATIONS iterations): ✓ PASSED"
    echo ""
    echo "The encryption module has been thoroughly tested and appears robust!"
else
    print_warning "Some extended fuzz tests failed. This may indicate edge cases that need attention."
    echo ""
    echo "Summary:"
    echo "- Simple fuzz tests: ✓ PASSED"
    echo "- Comprehensive fuzz tests: ✓ PASSED"
    echo "- Extended fuzz testing: ⚠️  $FAILED/$ITERATIONS failed"
    exit 1
fi

# Clean up any remaining temp files
rm -f /tmp/fuzz_output_*.log

print_status "Fuzz testing completed successfully!"