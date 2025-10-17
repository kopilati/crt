# CRT Analysis Tool - Workspace

A Current Reality Tree analysis tool with system analysis capabilities, built as a Rust workspace with shared core library.

## Architecture

This project is organized as a Rust workspace with three main packages:

- **`crt-core`**: Shared types, validation, and DORA metric translation logic
- **`crt-backend`**: REST API server using Axum
- **`crt-frontend`**: WASM-based frontend with web bindings

## Quick Start

### Prerequisites

- Rust (latest stable)
- wasm-pack (`cargo install wasm-pack`)
- Node.js (for serving the frontend)

### Build Everything

```bash
# Build all packages
./build.sh

# Or build individually:
cd crates/crt-core && cargo build
cd crates/crt-backend && cargo build
cd crates/crt-frontend && wasm-pack build --target web --out-dir ../../pkg --dev
```

### Run Backend

```bash
cd crates/crt-backend
cargo run
```

The backend will start on `http://localhost:8080`.

### Serve Frontend

```bash
# Serve the WASM-enabled frontend
python3 -m http.server 8000
# Then open http://localhost:8000/index-wasm.html
```

## Package Details

### crt-core

Shared library containing:

- **Types**: All request/response structs (`AnalyseRequest`, `AnalysisResult`, etc.)
- **Validation**: `Validate` trait implementation for all request types
- **DORA Metrics**: Translation logic from 0-1 slider values to real-world units
- **WASM Bindings**: Optional web bindings for frontend integration

Key features:
- Strong typing throughout (no `serde_json::Value` in public APIs)
- Comprehensive validation with clear error messages
- Configurable DORA metric ranges and units
- Extensive unit tests for metric translation

### crt-backend

REST API server with endpoints:

- `POST /api/analyse` - Run system analysis
- `POST /api/evaluate_analysis` - Evaluate analysis results
- `POST /api/analyse_with_feedback` - Refine analysis with evaluation feedback
- `POST /api/refine` - Refine CRT content

Features:
- Request validation as first step in all handlers
- DORA metric translation for agent consumption
- Graceful fallback for agent response parsing
- CORS enabled for frontend integration

### crt-frontend

WASM-based frontend with:

- **Native Rust Types**: Direct use of `crt-core` types
- **DORA Sliders**: Interactive sliders with real-time metric translation
- **Split-Screen Analysis**: Analysis results and editable evaluation
- **Refinement Flow**: Preserve user edits during analysis refinement

Features:
- WASM module for client-side validation and metric translation
- Responsive design with modern CSS
- Real-time feedback and error handling
- Integration with backend REST API

## Development

### Adding New Types

1. Add types to `crt-core/src/types.rs`
2. Implement `Validate` trait in `crt-core/src/validation.rs`
3. Add WASM bindings in `crt-core/src/wasm.rs` (if needed)
4. Update backend handlers to use new types
5. Update frontend to use new types

### Testing

```bash
# Run all tests
cargo test

# Run core library tests specifically
cd crates/crt-core && cargo test

# Run backend tests
cd crates/crt-backend && cargo test
```

### WASM Development

```bash
# Build WASM package
cd crates/crt-frontend
wasm-pack build --target web --out-dir ../../pkg --dev

# For production builds
wasm-pack build --target web --out-dir ../../pkg --release
```

## Migration from Monolith

The original monolith structure has been preserved in the root directory for reference:

- `src/` - Original Rust source
- `services/backend/` - Original backend
- `index.html` - Original frontend

The new workspace structure provides:

1. **Better Organization**: Clear separation of concerns
2. **Shared Logic**: Common types and validation in core library
3. **Type Safety**: Strong typing throughout the entire stack
4. **WASM Integration**: Native Rust types in the frontend
5. **Maintainability**: Easier to extend and modify individual components

## Configuration

### Environment Variables

- `AGENT_BASE_URL`: URL for agent service (default: `http://localhost:8000`)

### DORA Metrics

Metric ranges and units are configured in `crt-core/src/dora.rs`:

```rust
const DORA_METRIC_CONFIGS: &[(&str, DoraMetricConfig)] = &[
    ("deployment_frequency", DoraMetricConfig {
        min_value: 0.025,
        max_value: 10.0,
        unit: "deployments/day",
        inverted: false,
    }),
    // ... other metrics
];
```

## API Documentation

### Analyse Request

```rust
{
    "crt": "string",
    "dora_metrics": {
        "deployment_frequency": 0.25,  // 0-1 slider value
        "lead_time": 0.5,
        "change_failure_rate": 0.5,
        "mttr": 0.5
    },
    "extended_engineering_metrics": {
        "commit_frequency": 0.25,
        "branch_lifetime": 0.5,
        "pbis_delivered_per_sprint_per_team": 0.25
    },
    "westrum": 3.0,  // 0-7 scale
    "time_allocation": {
        "meetings": 20,
        "unplanned": 15,
        "bugs": 15,
        "feature": 30,
        "tech_debt": 20
    }
}
```

### Analysis Response

```rust
{
    "run_id": "string",
    "result": {
        "executive_summary": "string",
        "core_systemic_issues": [...],
        "leverage_points": [...],
        "systemic_relationships": [...],
        "assumptions": [...],
        "analysis_confidence": "string",
        "analysis_metadata": {...}
    }
}
```

## Contributing

1. Make changes to the appropriate package
2. Run tests: `cargo test`
3. Build: `./build.sh`
4. Test end-to-end functionality
5. Update documentation as needed

## License

MIT License - see LICENSE file for details.
