FROM rust:1.89-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/hng-xiv-be-1 /usr/local/bin/hng-xiv-be-1

ENV PORT=8080
ENV RUST_LOG=info

EXPOSE 8080

CMD ["/usr/local/bin/hng-xiv-be-1"]