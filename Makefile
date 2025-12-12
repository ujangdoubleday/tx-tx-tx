.PHONY: build release clean install help docker-build docker-secret docker-run docker-clean docker-swarm docker-logs docker-secret-remove
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
		if [ -n "$(ARGS)" ]; then \
			echo "Running CLI mode with arguments: $(ARGS)"; \
			docker run --rm \
				--name tx-tx-tx-cli \
				-e ETH_PRIVATE_KEY_FILE=/run/secrets/eth_private_key \
				-v $(PWD)/.eth_secret:/run/secrets/eth_private_key:ro \
				-v $(PWD)/contracts:/app/contracts:ro \
				tx-tx-tx:latest $(ARGS); \
		else \
			if [ -t 0 ]; then \
				echo "Running UI mode (interactive)"; \
				docker run -it --rm \
					--name tx-tx-tx-ui \
					-e ETH_PRIVATE_KEY_FILE=/run/secrets/eth_private_key \
					-v $(PWD)/.eth_secret:/run/secrets/eth_private_key:ro \
					-v $(PWD)/contracts:/app/contracts:ro \
					tx-tx-tx:latest; \
			else \
				echo "❌ Error: Not in interactive terminal!"; \
				echo "UI mode requires interactive terminal. Use CLI mode instead:"; \
				echo "  ARGS=\"--help\" make docker-run"; \
				echo "  ARGS=\"sign --help\" make docker-run"; \
				exit 1; \
			fi; \
		fi; \
	else \
		echo "❌ Error: .eth_secret file not found!"; \
		echo "Please create .eth_secret file with your private key or use 'make docker-swarm' for Docker secrets."; \
		echo "Example: echo 'your_private_key_here' > .eth_secret"; \
		exit 1; \
	fi

docker-run-swarm:
	docker run -it --rm \
		--name tx-tx-tx-swarm \
		-e ETH_PRIVATE_KEY_FILE=/run/secrets/eth_private_key \
		--secret eth_private_key \
		-v $(PWD)/contracts:/app/contracts:ro \
		tx-tx-tx:latest

docker-clean:
	docker rmi -f tx-tx-tx:latest 2>/dev/null || true
	docker volume rm tx-tx-tx_tx_artifacts 2>/dev/null || true
	docker volume rm tx-tx-tx_foundry_cache 2>/dev/null || true
	@echo "✓ Docker resources cleaned"

docker-swarm:
	docker swarm init 2>/dev/null || echo "Swarm already initialized"
	@echo "Checking for existing secret..."
	@if docker secret inspect eth_private_key >/dev/null 2>&1; then \
		echo "Secret eth_private_key already exists. Using existing secret."; \
	else \
		echo "Creating new secret..."; \
		echo "Please enter your Ethereum private key:"; \
		bash -c 'printf "Enter your Ethereum private key: " && read -s secret && echo "$$secret" | docker secret create eth_private_key -' || exit 1; \
		echo "✓ Secret created successfully"; \
	fi
	@echo "Deploying to Swarm..."
	docker stack deploy -c docker-compose.yml tx-stack
	@echo "✓ Deployed to Docker Swarm"

docker-logs:
	docker service logs tx-stack_tx-tx-tx -f

docker-down:
	docker stack rm tx-stack
	@echo "✓ Stack removed from Swarm"

docker-secret-remove:
	@if docker secret inspect eth_private_key >/dev/null 2>&1; then \
		echo "Removing existing secret eth_private_key..."; \
		docker secret rm eth_private_key; \
		echo "✓ Secret removed"; \
	else \
		echo "No existing secret found"; \
	fi

help:
	@echo "Available targets:"
	@echo "  make           - Build debug version and create wrapper script (default)"
	@echo "  make build     - Build debug version and create wrapper script"
	@echo "  make release   - Build release version and create wrapper script"
	@echo "  make clean     - Remove build artifacts and wrapper script"
	@echo ""
	@echo "Docker targets:"
	@echo "  make docker-build        - Build Docker image"
	@echo "  make docker-secret       - Create local .eth_secret file for Docker"
	@echo "  make docker-secret-remove - Remove existing Docker Swarm secret"
	@echo "  make docker-run          - Run container (UI mode by default, CLI with args)"
	@echo "  make docker-clean        - Remove Docker resources"
	@echo "  make docker-swarm        - Deploy to Docker Swarm with secrets"
	@echo "  make docker-logs         - View Swarm service logs"
	@echo "  make docker-down         - Remove Swarm stack"
	@echo ""
	@echo "Usage:"
	@echo "  Setup secrets:     make docker-secret"
	@echo "  UI Mode:           make docker-run"
	@echo "  CLI Commands:      ARGS=\"sign --message hello\" make docker-run"
	@echo "  Production:        make docker-swarm"
	@echo ""
	@echo "Note: Local Docker uses .eth_secret file, Swarm uses Docker secrets"
	@echo "  make help      - Show this help message"
