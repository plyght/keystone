FROM rust:1.75 as builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY services/api/Cargo.toml ./services/api/
COPY src ./src
COPY services/api/src ./services/api/src

RUN cargo build --release --bin birch-api

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/birch-api /usr/local/bin/birch-api

ENV API_HOST=0.0.0.0
ENV API_PORT=3000

EXPOSE 3000

CMD ["birch-api"]

