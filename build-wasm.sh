#!/bin/bash

set -e

echo "🦀 Building Rust ECS for WebAssembly..."

# Build the WebAssembly module
wasm-pack build --target web --out-dir www/pkg --dev

echo "✅ WebAssembly build complete!"
echo "📁 Files generated in www/pkg/"
echo "🌐 Open www/index.html in a web server to view the demo"
echo ""
echo "To serve locally, you can use:"
echo "  cd www && python3 -m http.server 8000"
echo "  Then visit: http://localhost:8000"