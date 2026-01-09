# ---- build ----
FROM rust:latest AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

# ---- run ----
FROM debian:bookworm-slim
WORKDIR /app

# (optional) add certs if you ever call out to https endpoints
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/homey /app/homey
COPY config.json /app/config.json
COPY static /app/static

EXPOSE 3000
CMD ["/app/homey"]
