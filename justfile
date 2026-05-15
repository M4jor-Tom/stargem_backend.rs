# stargem-backend dev commands — run from server/ directory

default:
  @just --list

# Build backend (debug)
build-dev:
  cargo build

# Build backend release (via crane)
build:
  nix build .

# Run tests
test:
  cargo test

# Run linter
lint:
  cargo clippy -- -D warnings

# Format code
fmt:
  cargo fmt

# Check formatting
fmt-check:
  cargo fmt --check
