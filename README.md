Mutilator
=========

A mutating admission controller for Kubernetes that will enforce certain fields in resources belonging to the [Aiven Operator](https://github.com/aiven/aiven-operator).

Things to mutate

- [X] projectVpcId
- [ ] plan
- [ ] serviceIntegrations (prometheus ?)
- [ ] terminationProtection: true
- [ ] cloudName: google-{{ .Values.location }}
- [ ] tags: (environment, tenant, team)
