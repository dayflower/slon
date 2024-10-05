# syntax=docker/dockerfile-upstream:master

ARG NAME=slon

ARG RUSTARCH1=${TARGETARCH/arm64/aarch64}
ARG RUSTARCH=${RUSTARCH1/amd64/x86_64}

#### Stage 1: Build the application

FROM --platform=$BUILDPLATFORM messense/rust-musl-cross:${RUSTARCH}-musl AS builder

ARG TARGETARCH
ARG RUSTARCH
ARG NAME

## install dependencies
RUN --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    mkdir -p src && \
    echo 'fn main() { }' > src/main.rs && \
    cargo build --release --target ${RUSTARCH}-unknown-linux-musl

## build
RUN --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=bind,source=src,target=src \
    CARGO_BUILD_INCREMENTAL=true cargo build --release --target ${RUSTARCH}-unknown-linux-musl && \
    cp target/${RUSTARCH}-unknown-linux-musl/release/${NAME} target/release/app


#### Stage 2: Setup the runtime environment

# platform is TARGETPLATFORM (as default)
FROM gcr.io/distroless/static-debian12:nonroot

ARG NAME

WORKDIR /app

USER nonroot

COPY --from=builder /home/rust/src/target/release/app /app/${NAME}

ENV SLACK_TOKEN=

# we cannot use ARG expand in ENTRYPOINT
ENTRYPOINT [ "/app/slon" ]
