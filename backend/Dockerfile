FROM rust:1.88.0 AS builder
LABEL "dev.ethanchang.maintainer"="Ethan Chang"
LABEL "dev.ethanchang.project"="Mockomatic"
LABEL version="0.1.0"

WORKDIR /app
# for highs
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
ARG RUST_PORT=8080
ENV PORT=$RUST_PORT
EXPOSE $PORT 
CMD ["./backend"]
