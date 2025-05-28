FROM oven/bun:latest AS builder

WORKDIR /build
COPY . .
RUN bun install && bun run build

FROM caddy:latest

WORKDIR /app
COPY --from=builder /build/build .
COPY Dockerfile_Caddyfile Caddyfile
ENTRYPOINT ["caddy", "run", "--config", "Caddyfile"]