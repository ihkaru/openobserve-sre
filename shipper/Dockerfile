# Multi-stage build for ultra-small (~15MB) container footprint
FROM rust:1-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /usr/src/shipper
COPY . .
RUN cargo build --release

FROM alpine:3.20
RUN apk add --no-cache ca-certificates
WORKDIR /app
COPY --from=builder /usr/src/shipper/target/release/openobserve-shipper /app/shipper
COPY config.yaml /app/config.yaml
ENV CONFIG_PATH=/app/config.yaml
EXPOSE 8089
ENTRYPOINT ["/app/shipper"]
