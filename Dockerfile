# Build stage
FROM rust:1.70-alpine AS builder

# Install dependencies
RUN apk add --no-cache musl-dev sqlite-dev

WORKDIR /app

# Copy dependency files
COPY Cargo.toml Cargo.lock ./

# Create dummy source to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy real source and build
COPY src ./src
COPY migrations ./migrations
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache sqlite ca-certificates

WORKDIR /app

# Copy binary and migrations
COPY --from=builder /app/target/release/dnd-scheduler-bot .
COPY --from=builder /app/migrations ./migrations

# Create data directory
RUN mkdir -p data

EXPOSE 3000

CMD ["./dnd-scheduler-bot"]
