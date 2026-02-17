FROM rust:slim-bookworm AS builder

WORKDIR /src

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
  && apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    git \
    pkg-config \
  && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY augustinus_ascii.txt ./
COPY crates ./crates
COPY bin ./bin

RUN cargo build -p augustinus --release

FROM debian:bookworm-slim AS runtime

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
  && apt-get install -y --no-install-recommends \
    bash \
    ca-certificates \
    git \
    nodejs \
    npm \
  && rm -rf /var/lib/apt/lists/*

ENV TERM=xterm-256color

COPY --from=builder /src/target/release/augustinus /usr/local/bin/augustinus
COPY --chmod=755 docker/entrypoint.sh /entrypoint.sh

ENTRYPOINT ["bash", "/entrypoint.sh"]
