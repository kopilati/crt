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

### Agent Backend (Python)

A lightweight FastAPI service lives in `agents/`; it exposes a single endpoint backed by the OpenAI Agents SDK.

1. Create a virtual environment:

    ```bash
    python -m venv .venv
    source .venv/bin/activate
    pip install -e agents
    ```

2. Configure credentials and launch:

    ```bash
    export OPENAI_API_KEY=sk-...
    export OPENAI_MODEL=gpt-4o-mini   # optional
    export BACKEND_PORT=3000          # optional

    python -m agents.main
    ```

Create a virtual environment and install the service:

```bash
python -m venv .venv
source .venv/bin/activate
pip install -e services/agents
```

Then configure credentials and start the API:

```bash
export OPENAI_API_KEY=sk-...
export OPENAI_MODEL=gpt-4o-mini   # optional
export BACKEND_PORT=3000          # optional

python -m crt_agents.main
# or use the console script: crt-agents-service
```

The server exposes `POST /agents/{agent_name}/run`, which loads the matching agent YAML definition (for example `goldratt.yml`), forwards the request to the OpenAI Agents SDK, and returns the agent response and run id.

Agent definitions live under `services/agents/src/crt_agents/config/` and must be named `<agent_name>.yml` (or `.yaml`). Each file should provide at least an `instructions` field, with optional `name` and `model` overrides.

#### Connecting the front-end to the backend

When you open `index.html` from a static file server (or straight off disk), it needs to know where the backend is running. By default, the UI assumes `http://localhost:8080`. You can override this at runtime from the browser console:

```js
// In the devtools console
setBackendBaseUrl('http://localhost:3000');
```

The value is stored in `localStorage` so the page remembers it between reloads. To reset back to the default, call `setBackendBaseUrl(null)`.

Environment variables can also be provided via a `.env` file (repository root or `services/agents/`) and are loaded automatically when the server starts.

### Rust Proxy Backend

The `services/backend` crate exposes the original `/api/refine` endpoint but now proxies every request to the Goldratt agent service. It expects the agent service to be reachable (e.g., the Python backend above running on port 3000).

```bash
cargo run -p crt-backend --manifest-path services/backend/Cargo.toml
```

Configuration (via environment variables):

- `AGENT_SERVICE_URL` – base URL of the agent service (`http://127.0.0.1:3000` by default)
- `AGENT_NAME` – agent identifier (matches `<name>.yml` under `services/agents/src/crt_agents/config/`, defaults to `goldratt`)
- `BACKEND_ADDR` – bind address for the Rust proxy (`0.0.0.0:8080` by default)

`POST /api/refine` accepts `{ "content": "..." }`, forwards the request to `{AGENT_SERVICE_URL}/agents/{AGENT_NAME}/run`, and relays the agent's JSON back to the caller. If the agent returns plain text, the proxy wraps it in `{ "output_text": ..., "run_id": ... }`.

If you host both the UI and backend behind the same origin/port, the UI will automatically target that origin.

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
