VERSION 0.7
FROM rust:1

prepare:
    FROM rust:1
    WORKDIR /code

    ENV CARGO_BUILD_TARGET=x86_64-unknown-linux-musl

    RUN apt-get --yes update && apt-get --yes install cmake musl-tools
    RUN rustup target add "${CARGO_BUILD_TARGET}"

    RUN curl -LsSf https://github.com/cargo-bins/cargo-binstall/releases/latest/download/cargo-binstall-x86_64-unknown-linux-musl.tgz | tar zxf - -C ${CARGO_HOME:-~/.cargo}/bin
    RUN curl -LsSf https://github.com/kube-rs/kopium/releases/latest/download/kopium-linux-amd64 > ${CARGO_HOME:-~/.cargo}/bin/kopium && chmod a+x ${CARGO_HOME:-~/.cargo}/bin/kopium
    RUN curl -LsSf https://get.nexte.st/latest/linux | tar zxf - -C ${CARGO_HOME:-~/.cargo}/bin
    RUN cargo binstall --no-confirm --no-cleanup cargo-chef
    SAVE IMAGE --push ghcr.io/nais/mutilator/cache:prepare

chef-planner:
    FROM +prepare
    COPY --dir src .config Cargo.lock Cargo.toml .
    RUN cargo chef prepare --recipe-path recipe.json
    SAVE ARTIFACT recipe.json

chef-cook:
    FROM +prepare
    COPY +chef-planner/recipe.json recipe.json
    RUN cargo chef cook --recipe-path recipe.json --release
    SAVE IMAGE --push ghcr.io/nais/mutilator/cache:chef-cook

build:
    FROM +chef-cook

    COPY --dir src .config Cargo.lock Cargo.toml .
    RUN cargo build --release
    RUN cargo nextest run --profile ci --release

    SAVE ARTIFACT target/${CARGO_BUILD_TARGET}/release/mutilator mutilator
    SAVE ARTIFACT target/nextest/ci/junit.xml AS LOCAL target/nextest/ci/junit.xml
    SAVE IMAGE --push ghcr.io/nais/mutilator/cache:build

aiven-types:
    FROM +prepare
    RUN for type in redis; do \
            curl -sSL https://raw.githubusercontent.com/aiven/aiven-operator/main/config/crd/bases/aiven.io_${type}.yaml | kopium -Af - > aiven_${type}.rs; \
        done
    SAVE ARTIFACT aiven_*.rs AS LOCAL src/aiven_types/

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
