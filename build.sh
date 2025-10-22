#!/bin/bash

# Build script to compile all binaries and organize them
echo "Building CLI shell and command binaries..."

# Build all binaries
cargo build --release

# Create bin directory for our commands
mkdir -p bin

# Copy command binaries to bin directory
cp target/release/echo bin/

echo "Binaries built and copied to bin/ directory"