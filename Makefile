.PHONY: build release clean install help
.DEFAULT_GOAL := build

BINARY_NAME=tx-tx-tx
DEBUG_BINARY=target/debug/$(BINARY_NAME)
RELEASE_BINARY=target/release/$(BINARY_NAME)
INSTALL_PATH=./$(BINARY_NAME)

build: build-debug create-wrapper

build-debug:
	cargo build

release: build-release create-wrapper

build-release:
	cargo build --release

create-wrapper:
	@if [ -f "$(DEBUG_BINARY)" ]; then \
		echo '#!/bin/sh' > "$(INSTALL_PATH)"; \
		echo 'exec "$$(dirname "$$0")/target/debug/$(BINARY_NAME)" "$$@"' >> "$(INSTALL_PATH)"; \
		chmod +x "$(INSTALL_PATH)"; \
		echo "✓ Wrapper created for debug binary"; \
	elif [ -f "$(RELEASE_BINARY)" ]; then \
		echo '#!/bin/sh' > "$(INSTALL_PATH)"; \
		echo 'exec "$$(dirname "$$0")/target/release/$(BINARY_NAME)" "$$@"' >> "$(INSTALL_PATH)"; \
		chmod +x "$(INSTALL_PATH)"; \
		echo "✓ Wrapper created for release binary"; \
	else \
		echo "✗ Binary not found. Run 'make' or 'make release' first."; \
		exit 1; \
	fi

clean:
	cargo clean
	rm -f "$(INSTALL_PATH)"
	@echo "✓ Cleaned build artifacts and installed binary"

help:
	@echo "Available targets:"
	@echo "  make           - Build debug version and create wrapper script (default)"
	@echo "  make build     - Build debug version and create wrapper script"
	@echo "  make release   - Build release version and create wrapper script"
	@echo "  make clean     - Remove build artifacts and wrapper script"
	@echo "  make help      - Show this help message"
