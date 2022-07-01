FROM rust:alpine as builder

WORKDIR /app
RUN apk add --no-cache musl-dev libc6-compat

COPY . .
RUN cargo build --release

FROM scratch
WORKDIR /app
COPY --from=builder /app/target/release/tempx /app/tempx

EXPOSE 3000
ENTRYPOINT ["/app/tempx"]