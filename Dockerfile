FROM rust:1.77-slim-bookworm

# Install mingw-w64 for Windows cross-compilation and pkg-config/libudev for serialport
RUN apt-get update && apt-get install -y \
    mingw-w64 \
    pkg-config \
    libudev-dev \
    libxdo-dev \
    && rm -rf /var/lib/apt/lists/*

# Add the Windows target
RUN rustup target add x86_64-pc-windows-gnu

# Configure Cargo to use the MinGW linker for Windows
RUN mkdir -p /.cargo && \
    echo '[target.x86_64-pc-windows-gnu]\nlinker = "x86_64-w64-mingw32-gcc"\n' > /.cargo/config.toml

WORKDIR /app
CMD ["cargo", "build", "--release", "--target", "x86_64-pc-windows-gnu"]
