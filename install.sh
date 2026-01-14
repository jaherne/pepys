#!/usr/bin/env bash
set -e

echo "Building pepys for release..."
cargo build --release

echo ""
echo "Build complete! Binary location: target/release/pepys"
echo ""

# Check if /usr/local/bin is writable
if [ -w /usr/local/bin ]; then
    echo "Installing to /usr/local/bin/pepys..."
    cp target/release/pepys /usr/local/bin/pepys
    echo "✓ Installation complete!"
else
    echo "Installing to /usr/local/bin/pepys (requires sudo)..."
    sudo cp target/release/pepys /usr/local/bin/pepys
    echo "✓ Installation complete!"
fi

echo ""
echo "Pepys has been installed successfully!"
echo ""
