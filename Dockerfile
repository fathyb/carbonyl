# Using rust:1.67 for cross-compilation
FROM rust:1.67 AS cross-compile

# Update the package lists, install dependencies, and clean up in one step
RUN apt-get update && \
    apt-get install -y \
        zip g++-aarch64-linux-gnu g++-x86-64-linux-gnu libc6-dev-arm64-cross libc6-dev-amd64-cross && \
    rustup target add aarch64-unknown-linux-gnu x86_64-unknown-linux-gnu && \
    rustup toolchain install stable-aarch64-unknown-linux-gnu stable-x86_64-unknown-linux-gnu && \
    rm -rf /var/lib/apt/lists/*

# Setting environmental variables for cross-compilation
ENV AR_AARCH64_UNKNOWN_LINUX_GNU=aarch64-linux-gnu-ar \
    CC_AARCH64_UNKNOWN_LINUX_GNU=aarch64-linux-gnu-gcc \
    CXX_AARCH64_UNKNOWN_LINUX_GNU=aarch64-linux-gnu-g++ \
    CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
    AR_X86_64_UNKNOWN_LINUX_GNU=x86_64-linux-gnu-ar \
    CC_X86_64_UNKNOWN_LINUX_GNU=x86_64-linux-gnu-gcc \
    CXX_X86_64_UNKNOWN_LINUX_GNU=x86_64-linux-gnu-g++ \
    CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc

# Starting with a clean debian:bullseye-slim image
FROM debian:bullseye-slim

# Set up carbonyl user, directories, and required packages in a single step
RUN groupadd -r carbonyl && \
    useradd -r -g carbonyl carbonyl && \
    mkdir -p /carbonyl/data && \
    chown -R carbonyl:carbonyl /carbonyl && \
    apt-get update && \
    apt-get install -y libasound2 libexpat1 libfontconfig1 libnss3 && \
    rm -rf /var/lib/apt/lists/*

# Using user carbonyl
USER carbonyl

# Setting the working directory
VOLUME /carbonyl/data

# Setting environment variable for home directory
ENV HOME=/carbonyl/data

# Copy everything to the carbonyl directory
COPY . /carbonyl

# Check carbonyl version
RUN /carbonyl/carbonyl --version

# Set the entrypoint for the container
ENTRYPOINT ["/carbonyl/carbonyl", "--no-sandbox", "--disable-dev-shm-usage", "--user-data-dir=/carbonyl/data"]
