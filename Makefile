.PHONY: build release clean install help docker-build docker-secret docker-run docker-clean
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

# Docker targets
docker-build:
	docker build -t tx-tx-tx:latest .
	@echo "✓ Docker image built successfully"

docker-secret:
	@if [ -f ".eth_secret" ]; then \
		echo "✓ Local secret file already exists"; \
	else \
		echo "Setting up local Docker secret file..."; \
		echo "Please enter your Ethereum private key:"; \
		bash -c 'printf "Enter private key: " && read -s secret && echo "$$secret" > .eth_secret && chmod 600 .eth_secret'; \
		echo "✓ Secret file created at .eth_secret"; \
	fi

docker-run:
	@if [ -f "$(PWD)/.eth_secret" ]; then \
		echo "Running UI mode (interactive)"; \
		docker run -it --rm \
			--name tx-tx-tx-ui \
			-e ETH_PRIVATE_KEY_FILE=/run/secrets/eth_private_key \
			-v $(PWD)/.eth_secret:/run/secrets/eth_private_key:ro \
			-v $(PWD)/contracts:/app/contracts:ro \
			tx-tx-tx:latest; \
	else \
		echo "❌ Error: .eth_secret file not found!"; \
		echo "Please create .eth_secret file with: make docker-secret"; \
		echo "Example: echo 'your_private_key_here' > .eth_secret"; \
		exit 1; \
	fi

docker-clean:
	docker rmi -f tx-tx-tx:latest 2>/dev/null || true
	docker volume rm tx-tx-tx_tx_artifacts 2>/dev/null || true
	docker volume rm tx-tx-tx_foundry_cache 2>/dev/null || true
	@echo "✓ Docker resources cleaned"

help:
	@echo "Available targets:"
	@echo "  make           - Build debug version and create wrapper script (default)"
	@echo "  make build     - Build debug version and create wrapper script"
	@echo "  make release   - Build release version and create wrapper script"
	@echo "  make clean     - Remove build artifacts and wrapper script"
	@echo ""
	@echo "Docker targets:"
	@echo "  make docker-build  - Build Docker image"
	@echo "  make docker-secret - Create local .eth_secret file"
	@echo "  make docker-run    - Run container in UI mode (interactive)"
	@echo "  make docker-clean  - Remove Docker resources"
	@echo ""
	@echo "Usage:"
	@echo "  Setup:  make docker-secret"
	@echo "  Run:    make docker-run"
	@echo "  Help:   make help"
