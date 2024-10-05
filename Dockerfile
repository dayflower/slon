# syntax=docker/dockerfile:1

ARG NAME=slon


#### Stage 1: Build the application

FROM --platform=$BUILDPLATFORM messense/rust-musl-cross:${TARGETARCH}-musl AS builder
# messense/rust-musl-cross does not support tags commonly used by docker build (buildx), so you cannot use simply `--platform` with it. refer to the README for more information.
# ref. https://github.com/rust-cross/rust-musl-cross/issues/133

ARG TARGETARCH
ARG NAME

ARG ARCHFILE=/tmp/arch

# translate TARGETARCH to rust-musl-cross arch
RUN rm -f /tmp/arch && \
    if [ "$TARGETARCH" = "amd64" ]; then \
        echo "x86_64" > $ARCHFILE; \
    elif [ "$TARGETARCH" = "arm64" ]; then \
        echo "aarch64" > $ARCHFILE; \
    else \
        echo "Unsupported architecture: $TARGETARCH"; \
        exit 1; \
    fi

## install dependencies
RUN --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    mkdir -p src && \
    echo 'fn main() { }' > src/main.rs && \
    cargo build --release --target $(cat $ARCHFILE)-unknown-linux-musl

## build
RUN --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=bind,source=src,target=src \
    CARGO_BUILD_INCREMENTAL=true cargo build --release --target $(cat $ARCHFILE)-unknown-linux-musl && \
    cp target/$(cat $ARCHFILE)-unknown-linux-musl/release/${NAME} target/release/app


#### Stage 2: Setup the runtime environment

FROM gcr.io/distroless/static-debian12:nonroot

ARG NAME

WORKDIR /app

USER nonroot

COPY --from=builder /home/rust/src/target/release/app /app/${NAME}

ENV SLACK_TOKEN=

# we cannot use ARG expand in ENTRYPOINT
ENTRYPOINT [ "/app/slon" ]
