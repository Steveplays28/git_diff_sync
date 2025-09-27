# Base
FROM docker.io/rust:1-slim-bookworm AS base

RUN rustup toolchain install

# Chef
FROM base AS chef

RUN cargo install cargo-chef

RUN --mount=target=/var/lib/apt/lists,type=cache,sharing=locked \
	--mount=target=/var/cache/apt,type=cache,sharing=locked \
	rm -f /etc/apt/apt.conf.d/docker-clean \
	&& apt-get update \
	&& apt-get -y --no-install-recommends install pkg-config libssl-dev
RUN cargo install sccache
ENV RUSTC_WRAPPER=sccache SCCACHE_DIR=/sccache

WORKDIR /app

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
	cargo build --release

# Application
FROM debian:bookworm-slim AS runtime

ARG server_package=git_diff_sync_server

WORKDIR /app

# Copy the main binary
COPY --from=builder /app/target/release/$server_package /usr/local/bin
# Copy static assets
COPY --from=builder /app/$server_package/Rocket.tom[l] ./static
COPY --from=builder /app/$server_package/stati[c] ./static
COPY --from=builder /app/$server_package/template[s] ./templates

ENTRYPOINT ["/usr/local/bin/git_diff_sync_server"]
