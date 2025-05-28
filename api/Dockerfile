FROM rust:alpine AS builder

WORKDIR /build
COPY . .
RUN apk add musl-dev
RUN --mount=type=cache,target=target cargo build --release --target x86_64-unknown-linux-musl --features postgres
RUN --mount=type=cache,target=target cp target/x86_64-unknown-linux-musl/release/rutwt .

FROM alpine:latest

WORKDIR /app
COPY --from=builder /build/rutwt .
RUN apk add ffmpeg
ENTRYPOINT ["/app/rutwt"]
