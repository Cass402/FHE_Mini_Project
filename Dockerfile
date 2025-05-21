# Build stage
FROM rust:1.86-bookworm as builder

WORKDIR /usr/src/app

# Install dependencies with security updates
RUN apt-get update && \
    apt-get upgrade -y && \
    apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Create empty project for caching dependencies
RUN cargo new --bin fhe-mini-project
WORKDIR /usr/src/app/fhe-mini-project
COPY Cargo.toml Cargo.lock ./

# Build dependencies only (this layer will be cached if dependencies don't change)
RUN cargo build --release && \
    rm src/*.rs

# Copy source code
COPY . .

# Build application
RUN cargo build --release

# Final stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies with security updates
RUN apt-get update && \
    apt-get upgrade -y && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 && \
    rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /usr/src/app/fhe-mini-project/target/release/fhe-mini-project /app/
COPY --from=builder /usr/src/app/fhe-mini-project/target/release/examples/interactive_demo /app/

# Create directory for outputs
RUN mkdir -p /app/data /app/outputs
COPY --from=builder /usr/src/app/fhe-mini-project/README.md /app/

# Set environment variables
ENV RUST_LOG=info

# Command to run
ENTRYPOINT ["/app/fhe-mini-project"]