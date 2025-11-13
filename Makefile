.PHONY: build test clean install docker help build-all test-all build-cli test-cli build-sdk test-sdk build-docs check-bun check-cargo

help:
	@echo "Birch - Secret Rotation Tool"
	@echo ""
	@echo "Available targets:"
	@echo "  Unified Commands:"
	@echo "    build-all  - Build CLI + SDK + docs"
	@echo "    test-all   - Run all tests (CLI + SDK)"
	@echo ""
	@echo "  CLI Commands:"
	@echo "    build      - Build debug binary"
	@echo "    build-cli  - Build CLI (alias for release)"
	@echo "    release    - Build release binary"
	@echo "    test       - Run Rust tests"
	@echo "    test-cli   - Run Rust tests (alias)"
	@echo "    fmt        - Format Rust code"
	@echo "    lint       - Run clippy"
	@echo ""
	@echo "  SDK Commands:"
	@echo "    build-sdk  - Build TypeScript SDK"
	@echo "    test-sdk   - Run SDK tests"
	@echo ""
	@echo "  Docs Commands:"
	@echo "    build-docs - Build documentation site"
	@echo ""
	@echo "  Other:"
	@echo "    clean      - Clean build artifacts"
	@echo "    install    - Install to /usr/local/bin"
	@echo "    docker     - Build Docker image"
	@echo "    dist       - Build for all platforms"
	@echo "    dev        - Build and show help"

check-cargo:
	@which cargo > /dev/null || (echo "Error: cargo not found. Install Rust toolchain." && exit 1)

check-bun:
	@which bun > /dev/null || (echo "Error: bun not found. Install from https://bun.sh" && exit 1)

build-all: check-cargo check-bun
	@echo "Building Rust CLI..."
	@cargo build --release
	@echo "✅ CLI built"
	@echo ""
	@echo "Building TypeScript SDK..."
	@cd packages/client && bun install && bun run build
	@echo "✅ SDK built"
	@echo ""
	@echo "Building documentation..."
	@cd docs && bun install && bun run build
	@echo "✅ Docs built"
	@echo ""
	@echo "✅ All components built successfully"

test-all: check-cargo check-bun
	@echo "Running Rust tests..."
	@cargo test
	@echo "✅ Rust tests passed"
	@echo ""
	@echo "Running SDK tests..."
	@cd packages/client && bun test
	@echo "✅ SDK tests passed"
	@echo ""
	@echo "✅ All tests passed"

build-cli: release

test-cli: test

build-sdk: check-bun
	@echo "Building TypeScript SDK..."
	@cd packages/client && bun install && bun run build
	@echo "✅ SDK built"

test-sdk: check-bun
	@echo "Running SDK tests..."
	@cd packages/client && bun test
	@echo "✅ SDK tests passed"

build-docs: check-bun
	@echo "Building documentation..."
	@cd docs && bun install && bun run build
	@echo "✅ Docs built"

build:
	cargo build

release:
	cargo build --release

test:
	cargo test

clean:
	cargo clean
	rm -rf dist/
	rm -rf packages/client/dist
	rm -rf docs/.next

install: release
	sudo cp target/release/birch /usr/local/bin/
	@echo "Installed to /usr/local/bin/birch"

docker:
	docker build -t birch:latest .

dist:
	./build.sh

fmt:
	cargo fmt

lint:
	cargo clippy -- -D warnings

dev: build
	./target/debug/birch --help

