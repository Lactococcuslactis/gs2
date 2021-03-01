FROM rust as cacher
WORKDIR app
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates libssl-dev pkg-config && \
    apt-get autoremove --purge -y

RUN cargo install cargo-chef
COPY recipe.json .
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust as builder
WORKDIR app
COPY --from=cacher /app/target target
COPY . .
RUN cargo build --release --bin genshin-signin

FROM debian:buster-slim as runtime
WORKDIR app
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates libssl1.1 pkg-config && \
    apt-get autoremove --purge -y
COPY --from=builder /app/target/release/genshin-signin /usr/local/bin
ENTRYPOINT ["/usr/local/bin/genshin-signin"]