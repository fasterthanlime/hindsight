# Hindsight build tasks

# Build everything and serve with seed data
serve: build-wasm copy-wasm build-server
    cargo run --release --bin hindsight -- serve --seed

# Build WASM frontend
build-wasm:
    cd crates/hindsight-wasm && wasm-pack build --target web --release

# Copy WASM to server static directory
copy-wasm:
    mkdir -p crates/hindsight-server/static/wasm
    cp -r crates/hindsight-wasm/pkg/* crates/hindsight-server/static/wasm/

# Build server binary
build-server:
    cargo build --release --bin hindsight

# Clean build artifacts
clean:
    cargo clean
    rm -rf crates/hindsight-wasm/pkg
    rm -rf crates/hindsight-server/static/wasm

# Development build (faster, unoptimized)
dev: build-wasm-dev copy-wasm
    cargo run --bin hindsight -- serve --seed

# Build WASM in dev mode (faster)
build-wasm-dev:
    cd crates/hindsight-wasm && wasm-pack build --target web --dev
