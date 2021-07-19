# syntax=docker/dockerfile:1.2

FROM ubuntu as dumb-init
RUN rm -f /etc/apt/apt.conf.d/docker-clean; echo 'Binary::apt::APT::Keep-Downloaded-Packages "true";' > /etc/apt/apt.conf.d/keep-cache
RUN --mount=type=cache,target=/var/cache/apt \
    --mount=type=cache,target=/var/lib/apt \
    apt update && apt-get --no-install-recommends install -y ca-certificates wget
RUN wget -O /usr/local/bin/dumb-init https://github.com/Yelp/dumb-init/releases/download/v1.2.5/dumb-init_1.2.5_x86_64 && \
    chmod +x /usr/local/bin/dumb-init

FROM ekidd/rust-musl-builder as builder
WORKDIR /usr/local/src/tacodns-adapter
COPY . .
RUN --mount=type=cache,target=/root/.cargo cargo install --path .

FROM scratch
COPY --from=dumb-init /usr/local/bin/dumb-init /usr/local/bin/
ENTRYPOINT ["/usr/local/bin/dumb-init"]
COPY --from=builder /home/rust/.cargo/bin/tacodns-adapter /usr/local/bin/
CMD ["/usr/local/bin/tacodns-adapter"]
ENV RUST_LOG=tacodns_adapter
