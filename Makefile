# Default target to run the program in dev mode
.PHONY: all
all: dev

# Build and run in dev mode
.PHONY: dev
dev:
	@echo "Building and running in dev mode..."
	@cargo run -q -- -d

# Build and run in dev mode with config args
.PHONY: dev-c
dev-c:
	@echo "Building in dev mode and running config..."
	@cargo run -q -- -c --project-id prefab-mountain-398012 --access-token ya29.a0AfB_byBvCT9oK6vRMwjXhLG7PQH1uBMNwJgT39B5uk1R3PyGZ9FZc1vT1kyj0XGowi3zx-xmiiuv9AEwbXYi0NqpjsWkk1OGaxvRK01dHaO_czreKc2_Y91hQpfDpHPqk6VVBs5mde7-YCS2kumD4OqFJ_9sRqrBgySQVL6qQcsaCgYKAbYSARESFQHsvYls1P-84IlAVOTlcoWaK9rJvQ0178

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
	@cargo run -r -q -- -c --project-id prefab-mountain-398012 --api-key AIzaSyCz8G31yxztpayKtQy1A0VXOZdh2B0ETtM --access-token $(gcloud auth print-access-token)

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
