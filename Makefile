# SSH Key Manager - Makefile
# Provides convenient shortcuts for common development tasks

.PHONY: all build test clean install fmt lint check release run dev help

# Default target
all: check build

# Build the project in debug mode
build:
	@echo "🔨 Building debug version..."
	cargo build

# Build optimized release binary
release:
	@echo "📦 Building release version..."
	cargo build --release

# Run tests
test:
	@echo "🧪 Running tests..."
	cargo test

# Run tests with output
test-verbose:
	@echo "🧪 Running tests (verbose)..."
	cargo test -- --nocapture

# Run only unit tests
test-unit:
	@echo "🧪 Running unit tests..."
	cargo test --lib

# Run only integration tests
test-integration:
	@echo "🧪 Running integration tests..."
	cargo test --test integration_tests

# Format code
fmt:
	@echo "🎨 Formatting code..."
	cargo fmt

# Check formatting without modifying
fmt-check:
	@echo "🔍 Checking code formatting..."
	cargo fmt -- --check

# Run clippy lints
lint:
	@echo "🔍 Running clippy..."
	cargo clippy -- -D warnings

# Run all checks (format, clippy, test)
check: fmt-check lint test
	@echo "✅ All checks passed!"

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean

# Install locally (requires ~/.cargo/bin in PATH)
install: release
	@echo "📥 Installing to ~/.cargo/bin..."
	cp target/release/skm ~/.cargo/bin/
	@echo "✅ Installed! Run 'skm' to start."

# Uninstall
uninstall:
	@echo "🗑️  Uninstalling from ~/.cargo/bin..."
	rm -f ~/.cargo/bin/skm
	@echo "✅ Uninstalled!"

# Run in TUI mode
run:
	@echo "🚀 Running skm..."
	cargo run

# Run in debug mode with backtrace
dev:
	@echo "🐛 Running in debug mode..."
	RUST_BACKTRACE=1 cargo run

# Quick build and run
quick: build run

# Build for multiple targets (requires cross-compilation setup)
build-all:
	@echo "📦 Building for multiple targets..."
	cargo build --release --target x86_64-unknown-linux-gnu || true
	cargo build --release --target x86_64-apple-darwin || true
	cargo build --release --target aarch64-apple-darwin || true

# Generate documentation
doc:
	@echo "📚 Generating documentation..."
	cargo doc --no-deps --open

# Run benchmarks (if any)
bench:
	@echo "⚡ Running benchmarks..."
	cargo bench

# Update dependencies
update:
	@echo "⬆️  Updating dependencies..."
	cargo update

# Security audit (requires cargo-audit)
audit:
	@echo "🔒 Running security audit..."
	cargo audit

# Show help
help:
	@echo "SSH Key Manager - Available Commands:"
	@echo ""
	@echo "  make build          - Build debug version"
	@echo "  make release        - Build optimized release binary"
	@echo "  make run            - Run the application (TUI mode)"
	@echo "  make dev            - Run with debug backtrace enabled"
	@echo ""
	@echo "  make test           - Run all tests"
	@echo "  make test-verbose   - Run tests with output"
	@echo "  make test-unit      - Run only unit tests"
	@echo "  make test-integration - Run only integration tests"
	@echo ""
	@echo "  make fmt            - Format code"
	@echo "  make fmt-check      - Check code formatting"
	@echo "  make lint           - Run clippy lints"
	@echo "  make check          - Run all checks (fmt + lint + test)"
	@echo ""
	@echo "  make install        - Install to ~/.cargo/bin"
	@echo "  make uninstall      - Remove from ~/.cargo/bin"
	@echo "  make clean          - Clean build artifacts"
	@echo ""
	@echo "  make doc            - Generate and open documentation"
	@echo "  make update         - Update dependencies"
	@echo "  make audit          - Run security audit (needs cargo-audit)"
	@echo ""
	@echo "  make help           - Show this help message"
