# Build stage
FROM rust:1 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

# Runtime stage
FROM debian:trixie-slim
COPY --from=builder /app/target/release/breezy /usr/local/bin/breezy
ENTRYPOINT ["/usr/local/bin/breezy"]
