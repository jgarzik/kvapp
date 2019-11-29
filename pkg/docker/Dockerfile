FROM rust:1.38.0 as builder
ENV NAME=kvapp

# fixme
# First build a dummy project with our dependencies to cache them in Docker
# WORKDIR /usr/src
# RUN cargo new --bin ${NAME}
# WORKDIR /usr/src/${NAME}
# COPY ./Cargo.lock ./Cargo.lock
# COPY ./Cargo.toml ./Cargo.toml
# RUN cargo build --release
# RUN rm src/*.rs

# Now copy the sources and do the real build
WORKDIR /usr/src/${NAME}
COPY . .
RUN cargo test -j9
RUN cargo build --release -j9

# Second stage putting the build result into a debian stable-slim image
FROM debian:stable-slim
ENV NAME=kvapp

COPY --from=builder /usr/src/${NAME}/target/release/${NAME} /usr/local/bin/${NAME}
EXPOSE 8080
CMD ${NAME}
