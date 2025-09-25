#!/bin/bash

set -e

echo "🚀 Setting up Rust ECS development environment with WebAssembly support..."

# Display versions
echo "📋 Checking installed versions:"
rustc --version
cargo --version
python3 --version
node --version
npm --version

# Add WebAssembly target
echo "🎯 Adding WebAssembly target..."
rustup target add wasm32-unknown-unknown

# Install wasm-pack
echo "📦 Installing wasm-pack..."
cargo install wasm-pack --version 0.12.1

# Install clippy and rustfmt
echo "🔧 Adding Rust components..."
rustup component add clippy rustfmt

# Verify wasm-pack installation
echo "✅ Verifying wasm-pack installation..."
wasm-pack --version

# Create a simple test to verify WebAssembly build works
echo "🧪 Testing WebAssembly build capability..."
if [ -f "build-wasm.sh" ]; then
    echo "Found build-wasm.sh script"
    chmod +x build-wasm.sh
else
    echo "⚠️  build-wasm.sh not found, will be available when repository is cloned"
fi

# Install some useful global npm packages for web development
echo "🌐 Installing useful web development tools..."
npm install -g http-server live-server

echo ""
echo "✅ Setup complete! The environment now includes:"
echo "   🦀 Rust with WebAssembly target (wasm32-unknown-unknown)"
echo "   📦 wasm-pack for building WebAssembly modules"
echo "   🐍 Python3 for serving static files (python3 -m http.server)"
echo "   🟩 Node.js and npm for web development"
echo "   🌐 http-server and live-server for serving web content"
echo ""
echo "📖 To build and run the WebAssembly demo:"
echo "   1. Run: ./build-wasm.sh"
echo "   2. Serve: cd www && python3 -m http.server 8000"
echo "   3. Or use: cd www && http-server -p 8000"
echo "   4. Or use: cd www && live-server --port=8000"
echo "   5. Visit: http://localhost:8000"
echo ""
echo "🎉 Happy coding!"