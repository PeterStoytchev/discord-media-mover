FROM rust:latest as builder

WORKDIR /usr/src/app
COPY . .

# Standard build (Release mode)
RUN cargo install --path .

FROM debian:bookworm-slim

# Install curl
RUN apt-get update && \
    apt-get install -y curl ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/cargo/bin/discord-media-mover /usr/local/bin/discord-media-mover

CMD ["discord-media-mover"]
