# Use the official Rust image as the base image
FROM rust:latest

# Set the working directory inside the container
WORKDIR /app

# Copy the Rust project's files into the container
COPY . .

# Build the Rust application
RUN cargo build --release

# Set the default command to run the application when the container starts
CMD ["./target/release/rinha"]
