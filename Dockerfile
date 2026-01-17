FROM rust:1.75-alpine AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
COPY templates ./templates

RUN apk add --no-cache musl-dev

RUN cargo build --release

FROM alpine:latest

WORKDIR /app

COPY --from=builder /app/target/release/oxide-paste .
COPY --from=builder /app/templates ./templates

RUN adduser -D -u 1000 oxide && \
    chown -R oxide:oxide /app

USER oxide

EXPOSE 3000

ENV DATABASE_URL=sqlite:./oxide-paste.db?mode=rwc

CMD ["./oxide-paste"]
