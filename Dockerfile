# Base
FROM docker.io/rust:1-slim-bookworm AS base

WORKDIR /app

COPY rust-toolchain.toml rust-toolchain.toml
RUN rustup toolchain install

RUN rm -f /etc/apt/apt.conf.d/docker-clean && apt-get update
RUN apt-get -y --no-install-recommends install curl mold clang

# Sccache
FROM base AS sccache

RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN apt-get -y --no-install-recommends install pkg-config libssl-dev
RUN cargo binstall sccache --no-confirm
ENV RUSTC_WRAPPER=sccache SCCACHE_DIR=/sccache

# Chef
FROM sccache AS chef

RUN --mount=type=cache,target=/usr/local/cargo/registry \
	--mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
	cargo binstall cargo-chef --no-confirm

# Planner
FROM chef AS planner

COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
	--mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
	cargo chef prepare --recipe-path recipe.json

# Builder
FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry \
	--mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
	cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
	--mount=type=cache,target=$SCCACHE_DIR,sharing=locked \
	cargo build --release -p git_diff_sync_server

# Application
FROM debian:bookworm-slim AS runtime

ARG server_package=git_diff_sync_server

WORKDIR /app

# Copy the main binary
COPY --from=builder /app/target/release/$server_package /usr/local/bin
# Copy static assets
COPY --from=builder /app/crates/$server_package/Rocket.tom[l] ./static
COPY --from=builder /app/crates/$server_package/stati[c] ./static
COPY --from=builder /app/crates/$server_package/template[s] ./templates

ENTRYPOINT ["/usr/local/bin/git_diff_sync_server"]
