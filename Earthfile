VERSION 0.7
FROM rust:1

prepare:
    FROM rust:1
    RUN cargo install cargo-chef kopium
    RUN apt-get --yes update && apt-get --yes install cmake musl-tools
    RUN rustup target add x86_64-unknown-linux-musl
    SAVE IMAGE --push ghcr.io/nais/mutilator/cache:prepare

chef-planner:
    FROM +prepare
    COPY --dir src Cargo.lock Cargo.toml .
    RUN cargo chef prepare --recipe-path recipe.json
    SAVE ARTIFACT recipe.json

chef-cook:
    FROM +prepare
    COPY +chef-planner/recipe.json recipe.json
    RUN cargo chef cook --recipe-path recipe.json --release --target x86_64-unknown-linux-musl
    SAVE IMAGE --push ghcr.io/nais/mutilator/cache:chef-cook

build:
    FROM +chef-cook

    COPY --dir src Cargo.lock Cargo.toml .
    RUN cargo build --release --target x86_64-unknown-linux-musl

    SAVE ARTIFACT target/x86_64-unknown-linux-musl/release/mutilator mutilator
    SAVE IMAGE --push ghcr.io/nais/mutilator/cache:build

aiven-types:
    FROM +prepare
    RUN echo 'WARNING!!! This will overwrite the generated types. Have you checked that https://github.com/aiven/aiven-operator/pull/413 has been merged?' && exit 1
    RUN for type in redis; do \
            curl -sSL https://raw.githubusercontent.com/aiven/aiven-operator/main/config/crd/bases/aiven.io_${type}.yaml | kopium -Af - > aiven_${type}.rs; \
        done
    SAVE ARTIFACT aiven_*.rs AS LOCAL src/aiven_types/

docker:
    FROM cgr.dev/chainguard/static
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
