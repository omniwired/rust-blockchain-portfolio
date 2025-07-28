.PHONY: help build test demo clean lint fmt

# Default target
help:
	@echo "IBC-Mini: Rust Light-Client + zk-Proof Bridge"
	@echo ""
	@echo "Available commands:"
	@echo "  make demo     - Run the full demonstration" 
	@echo "  make build    - Build all workspace crates"
	@echo "  make test     - Run all tests"
	@echo "  make lint     - Run clippy linting"
	@echo "  make fmt      - Format code with rustfmt"
	@echo "  make clean    - Clean build artifacts"
	@echo ""

# Build all crates
build:
	@echo "🔨 Building all workspace crates..."
	cargo build --release --workspace

# Run all tests
test:
	@echo "🧪 Running tests..."
	cargo test --workspace

# Run the demo
demo:
	@echo "🚀 Running IBC-Mini demonstration..."
	cargo run --bin ibc-mini demo

# Generate a specific proof 
prove:
	@echo "⚡ Generating proof for height 1000..."
	cargo run --bin ibc-mini prove --height 1000

# Lint with clippy
lint:
	@echo "🔍 Running clippy..."
	cargo clippy --workspace -- -D warnings

# Format code
fmt:
	@echo "🎨 Formatting code..."
	cargo fmt --all

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean

# Development workflow
dev: fmt lint test
	@echo "✅ Development checks passed!"