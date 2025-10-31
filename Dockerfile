#
# Stage 1: Builder
#
# This stage prepares the application environment. It installs all necessary
# build dependencies, including Rust nightly, and then builds the Rust binary
# in release mode.
#
FROM rustlang/rust:nightly AS builder

# ARG TARGETARCH is automatically set by Docker to the architecture of the
# build machine (e.g., 'amd64', 'arm64').
ARG TARGETARCH

# Set a non-interactive frontend for Debian's package manager to prevent
# prompts from blocking the build process.
ENV DEBIAN_FRONTEND=noninteractive

# Install build dependencies.
# - build-essential: Provides compilers (gcc, etc.) needed for Rust crates
#   that contain C/C++ extensions.
# - curl: Utility for downloading dependencies or additional tools if needed.
# - pkg-config, libcairo2-dev: Required for building plotters crate dependencies.
RUN apt-get update && apt-get install -y \
    build-essential \
    curl \
    pkg-config \
    libcairo2-dev

# Set the working directory for the application source code.
WORKDIR /app

# Copy the project's dependency definition file into the builder stage. This
# leverages Docker's layer caching, so dependencies are only fetched if
# this file changes.
COPY Cargo.toml .

# Create a dummy main.rs to allow Cargo to fetch and build dependencies first.
# This is the equivalent of a compilation step, preparing a self-contained
# build environment.
RUN mkdir -p src && echo "fn main() {}" > src/main.rs

# Fetch all dependencies specified in Cargo.toml without building the project yet.
RUN cargo fetch

# Copy the entire build context (source code, modules, etc.) into the
# builder stage.
COPY . .

# Build the Rust project in release mode. This produces the 'bigmul' binary.
RUN cargo build --release

#
# Stage 2: Final Image
#
# This stage creates the final, minimal production image. It copies the
# compiled binary from the builder stage and includes only the necessary
# runtime system libraries, resulting in a smaller and more secure image.
#
FROM debian:bookworm-slim

# Set a non-interactive frontend for Debian's package manager.
ENV DEBIAN_FRONTEND=noninteractive

# Install runtime dependencies and clean up the apt cache to minimize image size.
# - libcairo2: Required for runtime dependencies of the plotters crate.
RUN apt-get update && apt-get install -y --no-install-recommends \
    libcairo2 \
    && rm -rf /var/lib/apt/lists/*

# Set the working directory for the application.
WORKDIR /app

# Copy the compiled Rust binary from the builder stage into the final image.
COPY --from=builder /app/target/release/bigmul .

# Set the container's entrypoint to run the Rust application executable.
ENTRYPOINT ["./bigmul"]

# Set the default command to run the benchmark. This is
# executed when the container is run without any additional arguments.
CMD []
