# DevContainer Configuration

This directory contains the development container configuration for the Rust ECS project with WebAssembly support.

## Features

The devcontainer provides a complete development environment with:

### 🦀 Rust Development
- Latest stable Rust toolchain
- WebAssembly target (`wasm32-unknown-unknown`)
- Clippy and rustfmt components
- Rust Analyzer for VS Code

### 📦 WebAssembly Build Tools
- **wasm-pack** (version 0.12.1) - for building WebAssembly modules
- Proper caching and version pinning for consistent builds

### 🌐 Web Development Tools
- **Python 3.11** - for serving static files (`python3 -m http.server`)
- **Node.js LTS** - for modern web development tooling
- **http-server** - alternative static file server
- **live-server** - development server with live reload

### 🔧 VS Code Extensions
- **rust-analyzer** - Rust language support
- **vscode-lldb** - debugging support
- **Live Server** - quick web server for development
- Additional extensions for TOML, JSON, and web development

### 🚪 Port Forwarding
- **Port 8000** - WebAssembly demo server (default)
- **Port 3000** - Development server
- **Port 5000** - Alternative server

## Quick Start

1. **Open in GitHub Codespaces or VS Code with Dev Containers extension**
2. **Wait for the setup to complete** (installs dependencies automatically)
3. **Build the WebAssembly demo**:
   ```bash
   ./build-wasm.sh
   ```
4. **Serve the demo**:
   ```bash
   cd www && python3 -m http.server 8000
   # or
   cd www && http-server -p 8000
   # or  
   cd www && live-server --port=8000
   ```
5. **Open the forwarded port** in your browser

## Development Workflow

### Building WebAssembly
```bash
# Quick build with the provided script
./build-wasm.sh

# Manual build
wasm-pack build --target web --out-dir www/pkg --release
```

### Serving the Web Demo
```bash
# Python (simple, always available)
cd www && python3 -m http.server 8000

# http-server (more features)
cd www && http-server -p 8000 -c-1

# live-server (auto-reload)
cd www && live-server --port=8000 --no-browser
```

### Testing
```bash
# Run Rust tests
cargo test

# Run clippy for linting
cargo clippy

# Format code
cargo fmt
```

## Customization

The devcontainer configuration can be customized by editing:
- `.devcontainer/devcontainer.json` - main configuration
- `.devcontainer/setup.sh` - installation script

## Troubleshooting

### WebAssembly Build Issues
- Ensure `wasm32-unknown-unknown` target is installed: `rustup target list --installed`
- Verify wasm-pack version: `wasm-pack --version`
- Try rebuilding: `rm -rf www/pkg && ./build-wasm.sh`

### Port Issues
- Check if ports are forwarded in VS Code/Codespaces
- Try alternative ports: 3000, 5000, or 8080
- Use `lsof -i :8000` to check if port is in use

### Dependencies
- Rerun setup: `./.devcontainer/setup.sh`
- Check installations: `rustc --version && wasm-pack --version && python3 --version`