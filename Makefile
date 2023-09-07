# Default target to run the program in dev mode
.PHONY: all
all: dev

# Build and run in dev mode
.PHONY: dev
dev:
	@echo "Building and running in dev mode..."
	@cargo build -q

# Build and run in dev mode with dev arg
.PHONY: dev-r
dev-r:
	@echo "Building in dev mode and running dev..."
	@cargo run -q -- -d

# Build and run in dev mode with config args
.PHONY: dev-c
dev-c:
	@echo "Building in dev mode and running config..."
	@cargo run -q -- -c --project-id <PROJEC_ID> --access-token <ACCESS_TOKEN> --api-key <API_KEY>

# Build and run in dev mode with install arg
.PHONY: dev-i
dev-i:
	@echo "Building in dev mode and running install..."
	@cargo run -q -- -i

# Build and run in dev mode with help arg (short)
.PHONY: dev-h
dev-h:
	@echo "Building in dev mode and running help (short)..."
	@cargo run -q -- -h

# Build and run in dev mode with help arg (long)
.PHONY: dev-hf
dev-hf:
	@echo "Building in dev mode and running help (long)..."
	@cargo run -q -- --help

# Build and run in release mode
.PHONY: release
release:
	@echo "Building and running in release mode..."
	@cargo run -q -r -- -p "test-files/Winch (2004).pdf"

# Build and run in dev mode with config args
.PHONY: config
config:
	@echo "Building and running config..."
	@cargo run -r -q -- -c --project-id <PROJEC_ID> --api-key <API_KEY> --access-token <ACCESS_TOKEN>

# Run tests
.PHONY: test
test:
	@echo "Running tests..."
	@cargo test

# Clean build artifacts
.PHONY: clean
clean:
	@echo "Cleaning up..."
	@cargo clean

# Run clippy linter
.PHONY: lint
lint:
	@echo "Running clippy..."
	@cargo clippy --all-targets --all-features -- -D warnings
