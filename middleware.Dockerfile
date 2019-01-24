FROM rust:1.32.0 as build

# maybe we can optimize this but easiest way to get nightly
RUN rustup default nightly
RUN USER=root cargo new --bin aepp-middleware
WORKDIR /aepp-middleware

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs && rm -rf target/release/deps/aepp_middleware*
COPY src/ ./src/
RUN cargo build --release

# since rust:1.32.0 is based on
FROM debian:stretch-slim
RUN apt-get update && apt-get -y install libpq5 libcurl3 libcurl3-gnutls
COPY --from=build /aepp-middleware/target/release/aepp-middleware /app/aepp-middleware
WORKDIR /app
COPY ./Rocket.example.toml ./Rocket.toml
COPY ./.env ./.env
ENTRYPOINT ["/app/aepp-middleware"]
CMD ["-p", "-s"]
EXPOSE 8000:8000
EXPOSE 80:80