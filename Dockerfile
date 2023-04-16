FROM rust:1.68.2 AS cross-compile

RUN apt-get update && \
    apt-get install -y \
        zip g++-aarch64-linux-gnu g++-x86-64-linux-gnu libc6-dev-arm64-cross libc6-dev-amd64-cross && \
    rustup target add aarch64-unknown-linux-gnu x86_64-unknown-linux-gnu && \
    rustup toolchain install stable-aarch64-unknown-linux-gnu stable-x86_64-unknown-linux-gnu && \
    rm -rf /var/lib/apt/lists/*

ENV AR_AARCH64_UNKNOWN_LINUX_GNU=aarch64-linux-gnu-ar
ENV CC_AARCH64_UNKNOWN_LINUX_GNU=aarch64-linux-gnu-gcc
ENV CXX_AARCH64_UNKNOWN_LINUX_GNU=aarch64-linux-gnu-g++
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc

ENV AR_X86_64_UNKNOWN_LINUX_GNU=x86_64-linux-gnu-ar
ENV CC_X86_64_UNKNOWN_LINUX_GNU=x86_64-linux-gnu-gcc
ENV CXX_X86_64_UNKNOWN_LINUX_GNU=x86_64-linux-gnu-g++
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc

FROM debian:bullseye-slim

RUN groupadd -r carbonyl && \
    useradd -r -g carbonyl carbonyl && \
    mkdir -p /carbonyl/data && \
    chown -R carbonyl:carbonyl /carbonyl && \
    apt-get update && \
    apt-get install -y libasound2 libexpat1 libfontconfig1 libnss3 && \
    rm -rf /var/lib/apt/lists/*

USER carbonyl
VOLUME /carbonyl/data
ENV HOME=/carbonyl/data

COPY . /carbonyl

RUN /carbonyl/carbonyl --version

ENTRYPOINT ["/carbonyl/carbonyl", "--no-sandbox", "--disable-dev-shm-usage", "--user-data-dir=/carbonyl/data"]
