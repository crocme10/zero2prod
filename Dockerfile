ARG RUST_VERSION="1.70"
ARG DEBIAN_VERSION="bullseye"

FROM rust:${RUST_VERSION}-${DEBIAN_VERSION} as builder

WORKDIR /home

ENV DEBIAN_FRONTEND noninteractive

RUN USER=root cargo new zero2prod
RUN USER=root cargo install --locked trunk
RUN USER=root cargo install cargo-xtask

WORKDIR /home/zero2prod

COPY . .

RUN cargo xtask frontend
RUN cargo build --release --bin zero2prod

# Extract binary from build cache
RUN mkdir bin

RUN cp target/release/zero2prod bin/

ARG DEBIAN_VERSION

FROM debian:${DEBIAN_VERSION}-slim

WORKDIR /srv

ENV DEBIAN_FRONTEND noninteractive

ARG DEBIAN_VERSION

COPY ./config /srv/zero2prod/etc/zero2prod
COPY ./docker/zero2prod/entrypoint.sh /srv/zero2prod/bin/
COPY ./dist /srv/zero2prod/var/http
RUN chmod +x /srv/zero2prod/bin/entrypoint.sh
RUN mkdir -p /srv/zero2prod/var/log
COPY --from=builder /home/zero2prod/bin/zero2prod /srv/zero2prod/bin/zero2prod

ENTRYPOINT ["/srv/zero2prod/bin/entrypoint.sh"]

EXPOSE 8000

CMD [ "run" ]
