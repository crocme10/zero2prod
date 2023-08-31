ARG RUST_VERSION="1.72"
ARG DEBIAN_VERSION="bookworm"

FROM rust:${RUST_VERSION}-${DEBIAN_VERSION} as builder

WORKDIR /home/zero2prod

ENV DEBIAN_FRONTEND noninteractive

COPY . .

RUN cargo build --release -p zero2prod-backend

# Extract binary from build cache
RUN mkdir bin

RUN cp target/release/zero2prod bin/

ARG DEBIAN_VERSION

FROM debian:${DEBIAN_VERSION}-slim

WORKDIR /srv

ENV DEBIAN_FRONTEND noninteractive

ARG DEBIAN_VERSION

COPY --from=builder /home/zero2prod/bin/zero2prod /srv/zero2prod/bin/backend

ENTRYPOINT ["/srv/zero2prod/bin/backend"]
