FROM rust:1-alpine AS builder

RUN apk add --no-cache build-base libressl-dev musl-dev pkgconfig sqlite sqlite-dev

WORKDIR /app

COPY . .

RUN cargo build --release

FROM alpine:latest

RUN apk add --no-cache libressl sqlite sqlite-dev

WORKDIR /app

COPY --from=builder /app/target/release/tourist /app/tourist

EXPOSE 3000

VOLUME ["/data"]

ENTRYPOINT ["/app/tourist"]
