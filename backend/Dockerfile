FROM rust:1.82.0 AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y \
    cmake \
    clang \
    libclang-dev \
    && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src
COPY . .
RUN cargo build --release
FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/backend .
RUN useradd -m -r appuser && chown appuser:appuser /app/backend
USER appuser
ARG PORT=8080
ENV PORT=$RUST_PORT
EXPOSE $RUST_PORT 
CMD ["./backend"]
