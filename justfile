# Load PUBLIC_API_URL etc. from .env (compile-time env for trunk/cargo)
set dotenv-load

# List available recipes
default:
    @just --list

# Start the dev server (localhost:5173)
dev:
    nix develop -c trunk serve

# Development build into dist/
build:
    nix develop -c trunk build

# Production build into dist/
release:
    nix develop -c trunk build --release

# Type-check for the wasm target
check:
    nix develop -c cargo check --target wasm32-unknown-unknown

# Run the native test suite (cdk WalletDatabase conformance + unit tests)
test:
    nix develop -c cargo test

# Lint with clippy
clippy:
    nix develop -c cargo clippy --target wasm32-unknown-unknown

# Format the code
fmt:
    nix develop -c cargo fmt

# Reproducible package build with nix
nix-build:
    nix build .#athenut-frontend

# Serve a built site (default: dist/) with SPA fallback
serve root="dist" port="3000":
    nix develop -c static-web-server --port {{port}} --root {{root}} --page-fallback {{root}}/index.html
