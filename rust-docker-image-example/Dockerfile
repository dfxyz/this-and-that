FROM rust:alpine as builder

WORKDIR /build/
COPY . .
RUN cargo build --release

FROM alpine
COPY --from=builder /build/target/release/rust-docker-image-example /usr/local/bin/
CMD ["rust-docker-image-example"]