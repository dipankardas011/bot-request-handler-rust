
FROM rust:latest as builder
WORKDIR /app
COPY . .
RUN cargo build -v --release

EXPOSE 3000
CMD ["/app/target/release/rust-http"]
