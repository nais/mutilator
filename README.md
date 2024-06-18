Mutilator
=========

A mutating admission controller for Kubernetes that will enforce certain fields in resources belonging to the [Aiven Operator](https://github.com/aiven/aiven-operator).

Things we mutate

- projectVpcId
- terminationProtection: true
- cloudName: google-{{ .Values.location }}
- tags: (environment, tenant, team)

## Building

### Earthly

Mutilator can use [earthly](https://earthly.dev) for building.
If you don't have earthly installed, you can use the wrapper at `./earthlyw`, which downloads the latest version for you.

* `earthly ls` to list targets
* `earthly +docker` to build primary target

### Nix

1. Use `nix build .#docker` to build docker image
2. Load docker image into Docker Daemon w/`docker load < result`

## Development

Mutilator is a mutating webhook, which means the requests can be difficult to handcraft when testing.
For that reason, there is a Tiltfile that installs the webhook into a local kind cluster and sets a service that points out of the cluster to your locally running instance.
This way you can run mutilator in a debugger, and trigger mutation by applying resources in the kind cluster.

To use this, you need to have [Tilt](https://tilt.dev) installed.
It is also recommended to use [ctlptl](https://github.com/tilt-dev/ctlptl) to manage your local cluster.

1. Start the cluster: `ctlptl create cluster kind --registry=ctlptl-registry`
2. Start tilt: `tilt up --stream`
3. Run mutilator in your debugger, with these environment variables:

    | Variable                           | Value                                  |
    |------------------------------------|----------------------------------------|
    | `MUTILATOR__PROJECT_VPC_ID`        | `00000000-0000-0000-0000-000000000000` |
    | `MUTILATOR__WEB__CERTIFICATE_PATH` | `tls.crt`                              |
    | `MUTILATOR__WEB__PRIVATE_KEY_PATH` | `tls.key`                              |
4. Apply suitable resources to trigger mutations: `kubectl apply -f develop/`
