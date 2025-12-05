.PHONY: build release clean install help
.DEFAULT_GOAL := build

BINARY_NAME=tx-tx-tx
DEBUG_BINARY=target/debug/$(BINARY_NAME)
RELEASE_BINARY=target/release/$(BINARY_NAME)
INSTALL_PATH=./tx

build: build-debug create-wrapper-debug

build-debug:
	cargo build

release: build-release create-wrapper-release

build-release:
	cargo build --release

create-wrapper-debug:
	@echo '#!/bin/sh' > "$(INSTALL_PATH)"; \
	echo 'exec "$$(dirname "$$0")/target/debug/$(BINARY_NAME)" "$$@"' >> "$(INSTALL_PATH)"; \
	chmod +x "$(INSTALL_PATH)"; \
	echo "✓ Wrapper created for debug binary"

create-wrapper-release:
	@echo '#!/bin/sh' > "$(INSTALL_PATH)"; \
	echo 'exec "$$(dirname "$$0")/target/release/$(BINARY_NAME)" "$$@"' >> "$(INSTALL_PATH)"; \
	chmod +x "$(INSTALL_PATH)"; \
	echo "✓ Wrapper created for release binary"

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
