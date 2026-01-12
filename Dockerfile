FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev pkgconfig

WORKDIR /app
COPY . .

RUN cargo build --release

FROM alpine:latest

RUN apk add --no-cache ca-certificates curl

COPY --from=builder /app/target/release/discord-media-mover /usr/local/bin/discord-media-mover

# Set the binary as the entrypoint
CMD ["discord-media-mover"]
