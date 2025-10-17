#!/bin/bash

# CRT Build Script
# Builds the core library, backend, and frontend packages

set -e

echo "🚀 Building CRT workspace..."

# Build core library first
echo "📦 Building crt-core..."
cd crates/crt-core
cargo build --lib
cargo test
cd ../..

# Build backend
echo "🖥️  Building crt-backend..."
cd crates/crt-backend
cargo build --bin crt-backend
cd ../..

# Build frontend WASM
echo "🌐 Building crt-frontend (WASM)..."
cd crates/crt-frontend
wasm-pack build --target web --out-dir ../../pkg --dev
cd ../..

echo "✅ Build complete!"
echo ""
echo "📁 Generated files:"
echo "  - Backend binary: crates/crt-backend/target/debug/crt-backend"
echo "  - WASM package: pkg/"
echo ""
echo "🔧 To run the backend:"
echo "  cd crates/crt-backend && cargo run"
echo ""
echo "🌐 To use the frontend:"
echo "  Include pkg/crt_frontend.js in your HTML"
