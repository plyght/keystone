.PHONY: build test clean install docker help

help:
	@echo "Birch - Secret Rotation Tool"
	@echo ""
	@echo "Available targets:"
	@echo "  build      - Build debug binary"
	@echo "  release    - Build release binary"
	@echo "  test       - Run all tests"
	@echo "  clean      - Clean build artifacts"
	@echo "  install    - Install to /usr/local/bin"
	@echo "  docker     - Build Docker image"
	@echo "  dist       - Build for all platforms"
	@echo "  fmt        - Format code"
	@echo "  lint       - Run clippy"

build:
	cargo build

release:
	cargo build --release

test:
	cargo test

clean:
	cargo clean
	rm -rf dist/

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

