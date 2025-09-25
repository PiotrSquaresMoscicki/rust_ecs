# WebAssembly Support

This document describes how to build and run the Rust ECS framework in WebAssembly for use in web browsers.

## Overview

The Rust ECS framework now supports compilation to WebAssembly (WASM), allowing the ECS system to run directly in web browsers. This enables creating web-based games and applications using the ECS framework.

## Features

- ✅ **Hello World Demo**: Simple WebAssembly "Hello World" example
- ✅ **ECS World Creation**: Create and manage ECS worlds in the browser
- ✅ **Entity Management**: Create entities and observe them in the browser console
- ✅ **Interactive Functions**: JavaScript can call Rust functions directly
- ✅ **Console Output**: Rust code outputs directly to browser console

## Prerequisites

1. **Rust with WebAssembly target**:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

2. **wasm-pack** (for building WebAssembly modules):
   ```bash
   cargo install wasm-pack
   ```

## Building for WebAssembly

### Quick Build

Use the provided build script:

```bash
./build-wasm.sh
```

### Manual Build

If you prefer to build manually:

```bash
wasm-pack build --target web --out-dir www/pkg --dev
```

This will generate WebAssembly files in the `www/pkg/` directory:
- `rust_ecs.js` - JavaScript bindings
- `rust_ecs_bg.wasm` - WebAssembly binary
- `rust_ecs.d.ts` - TypeScript definitions
- Other supporting files

## Running the Demo

1. **Build the WebAssembly module**:
   ```bash
   ./build-wasm.sh
   ```

2. **Start a local web server**:
   ```bash
   cd www
   python3 -m http.server 8000
   ```

3. **Open in browser**:
   Navigate to `http://localhost:8000` in your web browser.

## Demo Functionality

The WebAssembly demo includes:

### Interactive Features
- **Greet Function**: Enter your name and get a personalized greeting from Rust
- **ECS Demo**: Creates an ECS World and Entity, demonstrating basic functionality
- **Console Output**: All Rust code output appears in the browser console

### Code Structure

#### Entry Point (`src/wasm.rs`)
```rust
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    console_log!("Hello World from Rust and WebAssembly!");
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    console_log!("Hello, {}! Welcome to Rust ECS in WebAssembly!", name);
}

#[wasm_bindgen]
pub fn run_simple_demo() {
    let mut world = crate::World::new();
    let entity = world.create_entity();
    console_log!("Created entity: {:?}", entity);
}
```

#### HTML Interface (`www/index.html`)
- Responsive web design with clean UI
- Interactive buttons for testing Rust functions
- Real-time console output display
- Input field for customizing greetings

## Technical Details

### Conditional Compilation

The code uses conditional compilation to handle platform differences:

```rust
// Only compile main.rs for non-WASM targets
#![cfg(not(target_arch = "wasm32"))]

// Platform-specific dependencies in Cargo.toml
[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
ctrlc = "3.4"

[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = { version = "0.2", features = ["serde-serialize"] }
web-sys = "0.3"
console_error_panic_hook = "0.1"
getrandom = { version = "0.2", features = ["js"] }
```

### Dependencies

WebAssembly-specific dependencies:
- `wasm-bindgen`: Rust-JavaScript interop
- `web-sys`: Web API bindings
- `console_error_panic_hook`: Better error messages
- `getrandom` with "js" feature: Random number generation

### Limitations

Current WebAssembly implementation:
- No advanced ECS systems (movement, physics, etc.)
- No game loop or advanced demos
- Console output only (no canvas rendering yet)
- Limited to basic World and Entity operations

## Future Enhancements

Potential improvements for the WebAssembly implementation:
- Canvas-based rendering system
- Interactive game demos
- Full ECS system demonstrations
- Animation loop integration
- Component visualization
- Performance benchmarking tools

## Troubleshooting

### Common Issues

1. **"wasm-pack not found"**: Install wasm-pack with `cargo install wasm-pack`
2. **"target wasm32-unknown-unknown not found"**: Add target with `rustup target add wasm32-unknown-unknown`
3. **CORS errors**: Make sure to serve files through a web server, not file:// protocol
4. **getrandom errors**: Ensure `getrandom` has the "js" feature enabled

### Build Errors

If you encounter build errors:
1. Check that all conditional compilation attributes are correct
2. Ensure ctrlc usage is properly wrapped in `#[cfg(not(target_arch = "wasm32"))]`
3. Verify that WebAssembly-specific dependencies are correctly specified

## File Structure

```
rust_ecs/
├── src/
│   ├── wasm.rs          # WebAssembly bindings and entry points
│   ├── lib.rs           # Updated to include wasm module
│   └── ...              # Other source files
├── www/
│   ├── index.html       # HTML demo page
│   └── pkg/             # Generated WebAssembly files (gitignored)
├── build-wasm.sh        # Build script
└── WEBASSEMBLY.md       # This documentation
```