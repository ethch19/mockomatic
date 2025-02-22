FROM rust:1.82.0 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src
COPY . .
RUN cargo build --release
FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/backend .
ARG PORT=8080
ENV PORT=$RUST_PORT
EXPOSE $RUST_PORT 
CMD ["./backend"]
