# Use a smaller and more secure base image
FROM rust:bullseye as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src/
RUN cargo build --release

FROM debian:buster-slim
WORKDIR /app
COPY --from=builder /app/target/release/rinha /app/rinha
CMD ["./rinha"]
