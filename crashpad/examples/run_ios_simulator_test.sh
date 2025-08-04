#!/bin/bash

# Script to build and run iOS simulator test
# Usage: ./run_ios_simulator_test.sh

set -e

echo "Building for iOS Simulator..."

# Build for iOS simulator (x86_64 for Intel Macs, aarch64 for Apple Silicon)
if [[ $(uname -m) == "arm64" ]]; then
    TARGET="aarch64-apple-ios-sim"
else
    TARGET="x86_64-apple-ios"
fi

echo "Target: $TARGET"

# Build the iOS simulator test
cargo build --target $TARGET --example ios_simulator_test

echo "Build complete!"
echo ""
echo "To run in iOS Simulator:"
echo "1. The binary is at: target/$TARGET/debug/examples/ios_simulator_test"
echo "2. You can test it with xcrun simctl or deploy to a simulator"
echo ""
echo "Note: iOS uses in-process crash handling, so no separate handler binary is needed."