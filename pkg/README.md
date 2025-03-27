# CRT.neo Visualization Tool

A tool to convert CRT.neo format files into interactive D3.js visualizations. This project supports both:
1. Command-line usage for generating static HTML visualizations
2. WebAssembly (WASM) integration for in-browser editing and visualization

## What is CRT.neo format?

CRT.neo is a simple text format for describing conditional relationships between nodes. The format looks like:

```
This is a node
-then-> This is a connected node
-then-> And another one
-and-> This is a parallel node

Another starting node
-then-> Connected to that one
```

## Features

- Parse CRT.neo format and convert to interactive graph visualizations
- Interactive D3.js visualization with:
  - Force-directed graph layout
  - Node selection and filtering
  - Path visualization in a tabbed interface
  - Customizable physics properties
- WebAssembly support for browser-based editing
- Command-line interface for generating static visualizations

## Building the Project

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) and Cargo
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) for WebAssembly compilation

### Build for Command Line

To build the command-line version:

```bash
cargo build --release
```

The binary will be available at `target/release/crt_to_cypher`.

### Build for WebAssembly

To build the WebAssembly module:

```bash
wasm-pack build --target web
```

This creates a `pkg` directory containing the compiled WebAssembly module and JavaScript bindings.

## Usage

### Command Line

```bash
# Basic usage
crt_to_cypher -i input.neo -o output.html

# Help
crt_to_cypher --help
```

### WebAssembly/Browser

1. Build the WebAssembly module as described above
2. Serve the directory containing `index.html` and the `pkg` directory using any web server
3. Open `index.html` in a browser

For a simple local server, you can use:

```bash
# Using Python
python -m http.server

# Or using Node.js
npx serve
```

Then open http://localhost:8000 (or the appropriate port) in your browser.

## Web Editor Features

- Real-time editing and visualization
- Drag and resize interface
- Download visualizations as standalone HTML files
- Interactive graph with node selection
- Path visualization with tabbed interface

## License

MIT

## Contributors

- Your Name Here 