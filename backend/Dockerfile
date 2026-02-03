# Build stage
FROM rust:1.75-slim as builder

WORKDIR /app

# Copy workspace configuration
COPY Cargo.toml Cargo.lock ./

# Copy backend source
COPY backend ./backend

# Build the application
RUN cargo build --release --package falkordb-importer-backend

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/falkordb-importer-backend /app/backend

# Set the startup command
CMD ["/app/backend"]

EXPOSE 3000
