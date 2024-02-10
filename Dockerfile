# Use a smaller and more secure base image
FROM rust:slim AS builder

# Set the working directory inside the container
WORKDIR /app

# Copy only the necessary files for building the application
COPY Cargo.toml Cargo.lock ./
COPY src ./src/

# Build the Rust application with optimizations
# RUN cargo build --release
RUN apt-get update && apt-get install -y musl-tools
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --target x86_64-unknown-linux-musl --release

# Use a minimal base image for the runtime
FROM debian:buster-slim

# Set the working directory inside the container
WORKDIR /app

# Copy the built executable from the builder stage
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/rinha /app/rinha

# Set the default command to run the application when the container starts
CMD ["./rinha"]
