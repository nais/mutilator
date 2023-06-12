Mutilator
=========

A mutating admission controller for Kubernetes that will enforce certain fields in resources belonging to the [Aiven Operator](https://github.com/aiven/aiven-operator).

Things to mutate

- [X] projectVpcId
- [X] terminationProtection: true
- [X] cloudName: google-{{ .Values.location }}
- [X] tags: (environment, tenant, team)

## Building Docker image w/Earthly

Mutilator can use [earthly](https://earthly.dev) for building.
If you don't have earthly installed, you can use the wrapper at `./earthlyw`, which downloads the latest version for you.

* `earthly ls` to list targets
* `earthly +docker` to build primary target
* `earthly +aiven-types` to generate rust models for Aiven CRDs (see below)

## Building Docker Image w/Nix
1. Use `earthly +aiven-types` to generate rust models
1. Use `nix build .#docker` to build docker image
1. Load docker image into Docker Daemon w/`docker load < result`

## Adding new types

Add your new service type in the `Earthfile` and run `earthly +aiven-types` to generate rust models.
These will be placed in `src/aiven_types/`.
Then figure out how to make your changes.
