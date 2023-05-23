Mutilator
=========

A mutating admission controller for Kubernetes that will enforce certain fields in resources belonging to the [Aiven Operator](https://github.com/aiven/aiven-operator).

Things to mutate

- [X] projectVpcId
- [ ] plan
- [ ] serviceIntegrations (prometheus ?)
- [X] terminationProtection: true
- [ ] cloudName: google-{{ .Values.location }}
- [ ] tags: (environment, tenant, team)

## Building 

Mutilator uses [earthly](https://earthly.dev) for building. 
If you don't have earthly installed, you can use the wrapper at `./earthlyw`, which downloads the latest version for you.

* `earthly ls` to list targets
* `earthly +docker` to build primary target
* `earthly +aiven-types` to generate rust models for Aiven CRDs (see below)

## Adding new types

Add your new service type in the `Earthfile` and run `earthly +aiven-types` to generate rust models.
These will be placed in `src/aiven_types/`.
Then figure out how to make your changes.
