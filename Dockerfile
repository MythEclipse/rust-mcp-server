# Use the official Rust nightly image as the base image
FROM rustlang/rust:nightly-slim as builder

# Set the working directory
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Copy the source code
COPY src ./src

# Build the application in release mode
RUN cargo build --release

# Create a new stage for the runtime image
FROM debian:bookworm-slim

# Install ca-certificates for HTTPS requests (if needed)
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/rust-mcp-server /usr/local/bin/rust-mcp-server

# Set the binary as executable
RUN chmod +x /usr/local/bin/rust-mcp-server

# Expose any ports if needed (MCP uses stdio, so no ports needed)
# EXPOSE 3000

# Set the default command
CMD ["rust-mcp-server"]