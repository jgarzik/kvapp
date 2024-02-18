# Build Stage
FROM rust:1.76 as builder

# Create a new empty shell project
RUN USER=root cargo new --bin kvapp
WORKDIR /kvapp

# Copy our manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# This step caches our dependencies
RUN cargo build --release
RUN rm src/*.rs

# Now that the dependencies are built, copy your source code
COPY ./src ./src

# Build for release.
RUN rm ./target/release/deps/kvapp*
RUN cargo build --release

# Final Stage
FROM debian:bookworm-slim

# Copy the build artifact from the build stage
COPY --from=builder /kvapp/target/release/kvapp .

# Set the binary as the entrypoint of the container
ENTRYPOINT ["./kvapp"]

# Your application listens on port 8080.
EXPOSE 8080
