FROM mcr.microsoft.com/devcontainers/rust:1-bullseye

# Install additional packages
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
    build-essential \
    cmake \
    pkg-config \
    libssl-dev \
    lldb

# Install additional Rust components
RUN rustup component add clippy rustfmt

# Set working directory
WORKDIR /workspace

# Create a non-root user with sudo access
ARG USERNAME=vscode
ARG USER_UID=1000
ARG USER_GID=$USER_UID

# Install dependencies for development
RUN cargo install cargo-expand cargo-edit cargo-watch

# Create the user
RUN groupadd --gid $USER_GID $USERNAME \
    && useradd --uid $USER_UID --gid $USER_GID -m $USERNAME \
    && apt-get update \
    && apt-get install -y sudo \
    && echo $USERNAME ALL=\(root\) NOPASSWD:ALL > /etc/sudoers.d/$USERNAME \
    && chmod 0440 /etc/sudoers.d/$USERNAME