# Build stage
FROM rust:1.75-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev postgresql-dev

# Set working directory
WORKDIR /app

# Copy cargo files first for better caching
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src
COPY migrations ./migrations

# Build release binary
RUN cargo build --release

# Runtime stage
FROM alpine:3.19

# Install runtime dependencies
RUN apk add --no-cache \
    openssl \
    postgresql-client \
    ca-certificates

# Create non-root user
RUN addgroup -g 1000 mevshield && \
    adduser -D -u 1000 -G mevshield mevshield

# Set working directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/mev-shield /app/mev-shield
COPY --from=builder /app/target/release/mev-shield-cli /app/mev-shield-cli

# Copy configuration and migrations
COPY config.toml ./config.toml
COPY migrations ./migrations

# Change ownership
RUN chown -R mevshield:mevshield /app

# Switch to non-root user
USER mevshield

# Expose ports
EXPOSE 8080 9090

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD ["/app/mev-shield-cli", "monitor"]

# Run the application
ENTRYPOINT ["/app/mev-shield"]
