# Build stage
FROM rust:1.82-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev sqlite-dev pkgconfig openssl-dev

WORKDIR /app

# Copy dependency files first for better layer caching
COPY Cargo.toml Cargo.lock ./

# Create dummy source to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy real source code
COPY src ./src
COPY migrations ./migrations

# Build the actual application
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache sqlite ca-certificates curl

# Create non-root user
RUN addgroup -g 1001 -S appgroup && \
    adduser -u 1001 -S appuser -G appgroup

WORKDIR /app

# Copy binaries and migrations from builder
COPY --from=builder /app/target/release/dnd-scheduler-bot /app/dnd-scheduler-bot
COPY --from=builder /app/target/release/migrate /app/migrate
COPY --from=builder /app/migrations /app/migrations

# Create data directory with proper permissions
RUN mkdir -p /app/data && chown -R appuser:appgroup /app

# Switch to non-root user
USER appuser

# Expose the health check port
EXPOSE 3000

# Add health check using the health endpoint
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health/live || exit 1

# Run the application
CMD ["/app/dnd-scheduler-bot"]
