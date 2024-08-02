VERSION 0.8
FROM rust:1

ARG --global PUSH_CACHE=true

prepare:
    FROM rust:1
    WORKDIR /code

    RUN apt-get --yes update && apt-get --yes install cmake musl-tools

    ENV CARGO_BUILD_TARGET=x86_64-unknown-linux-musl
    RUN rustup target add "${CARGO_BUILD_TARGET}"

    RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
    RUN cargo binstall --secure --no-confirm --no-cleanup cargo-chef cargo-nextest

    IF ${PUSH_CACHE} == "true"
        SAVE IMAGE --push ghcr.io/nais/mutilator/cache:prepare
    ELSE
        SAVE IMAGE ghcr.io/nais/mutilator/cache:prepare
    END

chef-planner:
    FROM +prepare
    COPY --dir src .config Cargo.lock Cargo.toml .
    RUN cargo chef prepare --recipe-path recipe.json
    SAVE ARTIFACT recipe.json

chef-cook:
    FROM +prepare
    COPY +chef-planner/recipe.json recipe.json
    RUN cargo chef cook --recipe-path recipe.json --release --tests
    RUN cargo chef cook --recipe-path recipe.json --release

    IF ${PUSH_CACHE} == "true"
        SAVE IMAGE --push ghcr.io/nais/mutilator/cache:chef-cook
    ELSE
        SAVE IMAGE ghcr.io/nais/mutilator/cache:chef-cook
    END

build:
    FROM +chef-cook

    COPY --dir src .config Cargo.lock Cargo.toml .
    RUN cargo build --release
    RUN cargo nextest run --profile ci --release

    SAVE ARTIFACT target/${CARGO_BUILD_TARGET}/release/mutilator mutilator
    SAVE ARTIFACT target/nextest/ci/junit.xml AS LOCAL target/nextest/ci/junit.xml

    IF ${PUSH_CACHE} == "true"
        SAVE IMAGE --push ghcr.io/nais/mutilator/cache:build
    ELSE
        SAVE IMAGE ghcr.io/nais/mutilator/cache:build
    END

docker:
    FROM gcr.io/distroless/static-debian11:nonroot
    # Explicitly build these targets so that the cache images are pushed
    BUILD +prepare
    BUILD +chef-cook
    BUILD +build

    WORKDIR /
    COPY +build/mutilator /

    CMD ["/mutilator"]

    # builtins must be declared
    ARG EARTHLY_GIT_SHORT_HASH

    ARG REGISTRY=europe-north1-docker.pkg.dev/nais-io/nais/images
    ARG image=${REGISTRY}/mutilator
    ARG VERSION=$EARTHLY_GIT_SHORT_HASH
    SAVE IMAGE --push ${image}:${VERSION} ${image}:latest
