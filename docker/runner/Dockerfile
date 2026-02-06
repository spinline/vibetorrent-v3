# Use a base image that supports multi-arch (x64 and arm64)
# We use debian:bookworm-slim as base to install everything manually
# and then install the act_runner binary.
FROM debian:bookworm-slim

ENV DEBIAN_FRONTEND=noninteractive
ENV PATH="/root/.cargo/bin:/root/.node/bin:/root/zig:${PATH}"

# 1. Install Basic Dependencies
RUN apt-get update && apt-get install -y \
    curl \
    git \
    build-essential \
    ca-certificates \
    wget \
    xz-utils \
    libssl-dev \
    pkg-config \
    # Needed for some crate compilations
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# 2. Install Node.js v20 (Manual install to support multi-arch cleanly)
RUN ARCH=$(dpkg --print-architecture) && \
    if [ "$ARCH" = "amd64" ]; then NODE_ARCH="x64"; \
    elif [ "$ARCH" = "arm64" ]; then NODE_ARCH="arm64"; fi && \
    NODE_VER="v20.11.1" && \
    curl -fsSL "https://nodejs.org/dist/$NODE_VER/node-$NODE_VER-linux-$NODE_ARCH.tar.xz" -o node.tar.xz && \
    mkdir -p /root/.node && \
    tar -xJf node.tar.xz -C /root/.node --strip-components=1 && \
    rm node.tar.xz

# 3. Install Rust (Nightly) + Targets
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly --profile minimal && \
    . "$HOME/.cargo/env" && \
    rustup target add wasm32-unknown-unknown && \
    rustup component add rust-src

# 4. Install Zig (for Cross Compilation)
RUN ARCH=$(dpkg --print-architecture) && \
    if [ "$ARCH" = "amd64" ]; then ZIG_ARCH="x86_64"; \
    elif [ "$ARCH" = "arm64" ]; then ZIG_ARCH="aarch64"; fi && \
    ZIG_VER="0.13.0" && \
    curl -fsSL "https://ziglang.org/download/$ZIG_VER/zig-linux-$ZIG_ARCH-$ZIG_VER.tar.xz" -o zig.tar.xz && \
    tar -xf zig.tar.xz && \
    mv "zig-linux-$ZIG_ARCH-$ZIG_VER" /root/zig && \
    rm zig.tar.xz

# 5. Fix: Create libatomic.a shim for MIPS
# MIPS musl static build often misses __atomic_is_lock_free.
# We create a minimal implementation to satisfy the linker.
# We rename the function to avoid builtin conflict and alias it via linker flags later.
RUN . "$HOME/.cargo/env" && \
    echo '#include <stdbool.h>' > atomic.c && \
    echo 'bool __atomic_is_lock_free_shim(unsigned int size, const volatile void *ptr) { return size <= 4; }' >> atomic.c && \
    /root/zig/zig cc -target mips-linux-musl -msoft-float -c -o atomic.o atomic.c && \
    /root/zig/zig ar rcs libatomic.a atomic.o && \
    RUST_SYSROOT=$(rustc --print sysroot) && \
    TARGET_LIB_DIR="$RUST_SYSROOT/lib/rustlib/mips-unknown-linux-musl/lib" && \
    mkdir -p "$TARGET_LIB_DIR" && \
    cp libatomic.a "$TARGET_LIB_DIR/" && \
    rm atomic.c atomic.o libatomic.a

# 6. Install Tools (Trunk, cargo-zigbuild, wasm-bindgen protocol aligned)
# We install trunk binary to save time, others via cargo
RUN . "$HOME/.cargo/env" && \
    # Install cargo-zigbuild
    cargo install cargo-zigbuild && \
    # Install trunk (Binary)
    ARCH=$(dpkg --print-architecture) && \
    if [ "$ARCH" = "amd64" ]; then TRUNK_ARCH="x86_64-unknown-linux-gnu"; \
    elif [ "$ARCH" = "arm64" ]; then TRUNK_ARCH="aarch64-unknown-linux-gnu"; fi && \
    wget -qO- "https://github.com/trunk-rs/trunk/releases/download/v0.21.5/trunk-$TRUNK_ARCH.tar.gz" | tar -xzf - -C /root/.cargo/bin/ && \
    chmod +x /root/.cargo/bin/trunk && \
    # Install wasm-bindgen-cli (Compiling from source to avoid glibc issues, doing it ONCE here)
    cargo install wasm-bindgen-cli --version 0.2.108

# 6. Fix: Create dummy libatomic.a for MIPS
# openssl-sys demands -latomic, but Zig provides the symbols in compiler-rt.
# We create an empty archive to satisfy the linker.
RUN . "$HOME/.cargo/env" && \
    echo '' > atomic.c && \
    /root/zig/zig cc -target mips-linux-musl -c -o atomic.o atomic.c && \
    /root/zig/zig ar rcs libatomic.a atomic.o && \
    RUST_SYSROOT=$(rustc --print sysroot) && \
    TARGET_LIB_DIR="$RUST_SYSROOT/lib/rustlib/mips-unknown-linux-musl/lib" && \
    mkdir -p "$TARGET_LIB_DIR" && \
    cp libatomic.a "$TARGET_LIB_DIR/" && \
    rm atomic.c atomic.o libatomic.a

# 7. Install Gitea Act Runner Binary
# We fetch the binary directly to run as the entrypoint
RUN ARCH=$(dpkg --print-architecture) && \
    VERSION="0.2.11" && \
    curl -fsSL -o /usr/local/bin/act_runner "https://dl.gitea.com/act_runner/$VERSION/act_runner-$VERSION-linux-$ARCH" && \
    chmod +x /usr/local/bin/act_runner

# Create a volume for registration data
VOLUME /data
WORKDIR /data

# Define entrypoint to run the registration or daemon
# We will use a script to handle auto-registration if env vars are present
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

ENTRYPOINT ["/entrypoint.sh"]
