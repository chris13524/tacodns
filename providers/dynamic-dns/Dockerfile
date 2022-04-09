# syntax=docker/dockerfile:1.2

FROM ubuntu as dumb-init
RUN rm -f /etc/apt/apt.conf.d/docker-clean; echo 'Binary::apt::APT::Keep-Downloaded-Packages "true";' > /etc/apt/apt.conf.d/keep-cache
RUN --mount=type=cache,target=/var/cache/apt \
    --mount=type=cache,target=/var/lib/apt \
    apt update && apt-get --no-install-recommends install -y ca-certificates wget
RUN wget -O /usr/local/bin/dumb-init https://github.com/Yelp/dumb-init/releases/download/v1.2.5/dumb-init_1.2.5_x86_64 && \
    chmod +x /usr/local/bin/dumb-init

FROM rust as builder
RUN apt update && DEBIAN_FRONTEND=noninteractive apt-get install -y musl-tools
RUN rustup target add x86_64-unknown-linux-musl
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry/cache/ \
    --mount=type=cache,target=/usr/local/cargo/registry/index/ \
    --mount=type=cache,target=/usr/local/cargo/git/db/ \
    --mount=type=cache,target=./target/ \
    cargo install --target x86_64-unknown-linux-musl --path .

FROM scratch
COPY --from=dumb-init /usr/local/bin/dumb-init /usr/local/bin/
ENTRYPOINT ["/usr/local/bin/dumb-init"]
COPY --from=builder /usr/local/cargo/bin/dynamic-dns-provider /usr/local/bin/
CMD ["/usr/local/bin/dynamic-dns-provider"]
ENV RUST_LOG=dynamic_dns_provider
ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=80
