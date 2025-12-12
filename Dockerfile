# Multi-stage Dockerfile for tx-tx-tx blockchain CLI tool

# ============================================
# Builder Stage - Includes Rust + Foundry
# ============================================
FROM rust:1.83-slim AS builder

# Install nightly Rust toolchain for edition 2024 support
RUN rustup toolchain install nightly && \
    rustup default nightly

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    curl \
    git \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Install Foundry directly
RUN export FOUNDRY_VERSION=nightly-5c28b5b5c0c0b0b0b0b0b0b0b0b0b0b0b0b0b0b && \
    curl -L "https://github.com/foundry-rs/foundry/releases/download/nightly/foundry_nightly_linux_amd64.tar.gz" | \
    tar xz -C /usr/local/bin forge cast chisel anvil && \
    chmod +x /usr/local/bin/forge /usr/local/bin/cast /usr/local/bin/chisel /usr/local/bin/anvil

# Verify Foundry installation
RUN forge --version && cast --version

# Set working directory
WORKDIR /app

# Copy Cargo files for dependency caching
COPY Cargo.toml ./
COPY Cargo.lock ./
COPY crates/ ./crates/

# Copy Cargo files for dependency caching
COPY Cargo.toml ./
COPY Cargo.lock ./
COPY crates/ ./crates/

# Copy other source files
COPY contracts/ ./contracts/
COPY data/ ./data/
COPY lib/ ./lib/
COPY foundry.toml ./
COPY remappings.txt ./

# Build the application
RUN cargo build --release

# ============================================
# Runtime Stage - Minimal image
# ============================================
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    git \
    && rm -rf /var/lib/apt/lists/*

# Install Foundry in runtime stage
RUN export FOUNDRY_VERSION=nightly-5c28b5b5c0c0b0b0b0b0b0b0b0b0b0b0b0b0b0b && \
    curl -L "https://github.com/foundry-rs/foundry/releases/download/nightly/foundry_nightly_linux_amd64.tar.gz" | \
    tar xz -C /usr/local/bin forge cast chisel anvil && \
    chmod +x /usr/local/bin/forge /usr/local/bin/cast /usr/local/bin/chisel /usr/local/bin/anvil

# Create non-root user for security
RUN useradd -m -u 1000 txuser

# Set working directory
WORKDIR /app

# Copy compiled binary from builder stage
COPY --from=builder /app/target/release/tx-tx-tx /usr/local/bin/tx-tx-tx

# Copy contracts and data directories
COPY --from=builder /app/contracts/ ./contracts/
COPY --from=builder /app/data/ ./data/
COPY --from=builder /app/lib/ ./lib/

# Copy remappings and foundry config
COPY --from=builder /app/remappings.txt ./
COPY --from=builder /app/foundry.toml ./

# Set permissions
RUN chown -R txuser:txuser /app
RUN chmod +x /usr/local/bin/tx-tx-tx

# Switch to non-root user
USER txuser

# Set environment variables
ENV RUST_LOG=info
ENV PATH="/root/.foundry/bin:$PATH"

# Expose default port (if needed for future web interface)
EXPOSE 8545

# Default entry point - interactive UI mode
ENTRYPOINT ["tx-tx-tx"]

# Default command - shows interactive menu
CMD []