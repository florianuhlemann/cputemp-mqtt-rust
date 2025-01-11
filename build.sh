#/bin/bash

set -e  # Exit on any command failure

# Build the project in release mode
echo "Building the project in release mode..."
cargo build --release

# Copy the built binary to the releases directory
echo "Copying the built binary to the releases directory..."
cp ./target/release/cputemp-rust ./releases/cputemp-rust

echo "Build and copy completed successfully."
