FROM rust:1-alpine3.16 as builder
RUN apk add --no-cache cargo
ENV HOME=/root
WORKDIR /app/
COPY . /app/
RUN cargo build --release --target=x86_64-unknown-linux-musl --color never
RUN ls /app/target/x86_64-unknown-linux-musl/release/

FROM alpine:3.16
LABEL org.opencontainers.image.source="https://github.com/ngerakines/countdown-metrics-rs"
LABEL org.opencontainers.image.description="A daemon that publishes the number of seconds until a given date to a statsd sink."
LABEL org.opencontainers.image.authors="Nick Gerakines <nick.gerakines@gmail.com>"
LABEL org.opencontainers.image.licenses="MIT"
RUN apk add --no-cache ca-certificates
ENV RUST_LOG="warning"
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/countdown-metrics-rs /usr/local/bin/countdown-metrics-rs
ENTRYPOINT ["/usr/local/bin/countdown-metrics-rs"]
