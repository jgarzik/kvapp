FROM rust:1.61 as build

RUN apt-get update && apt-get -y install protobuf-compiler

# create a new empty shell project
RUN USER=root mkdir -p /usr/src && cd /usr/src && cargo new --bin kvapp
WORKDIR /usr/src/kvapp

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
RUN cp src/main.rs src/tester.rs

# this build step will cache your dependencies
RUN cargo update && cargo fetch
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./src ./src

# build for release
RUN rm ./target/release/deps/kvapp* ./target/release/deps/tester*
RUN cargo build --release
RUN cargo install --path .

# our final base
#FROM rust:1.49

# copy the build artifact from the build stage
#COPY --from=build /kvapp/target/release/kvapp .

# set the startup command to run your binary
CMD ["kvapp"]
