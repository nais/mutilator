load('ext://cert_manager', 'deploy_cert_manager')
load('ext://helm_resource', 'helm_resource', 'helm_repo')

APP_NAME="mutilator"

deploy_cert_manager()

helm_repo('aiven', 'https://aiven.github.io/aiven-charts')
helm_resource('aiven-operator-crds', 'aiven/aiven-operator-crds', resource_deps=['aiven'], pod_readiness="ignore")

ignore = str(read_file(".earthignore")).split("\n")

custom_build(
    ref=APP_NAME,
    command="earthly +docker --VERSION=$EXPECTED_TAG --REGISTRY=$EXPECTED_REGISTRY --PUSH_CACHE=false",
    deps=["src", "Cargo.*", ".config"],
    skips_local_docker=False,
    ignore=ignore,
)

# Deployed to the cluster
k8s_yaml(helm("charts/{}".format(APP_NAME), set=[
    # Make sure the chart refers to the same image ref as the one we built
    "image.repository={}".format(APP_NAME),
    # Application configure for testing
    "project_vpc_id=00000000-0000-0000-0000-000000000000",
    # Kubernetes configuration for testing
    "autoscaling.enabled=false",
    "autoscaling.minReplicas=1",
    "replicaCount=1",
    "debugger.enabled=true",
]))
k8s_resource(
    workload="chart-{}".format(APP_NAME),
    resource_deps=["aiven-operator-crds"],
    objects=[
        "chart-mutilator:mutatingwebhookconfiguration",
        "chart-mutilator:networkpolicy",
        "chart-mutilator:certificate",
        "chart-mutilator:issuer",
        "chart-mutilator:service",
        "chart-mutilator:endpointslice",
    ],
)

# Update locally stored certificates from cluster
cert_cmd = "kubectl get secret chart-mutilator-certs -o jsonpath='{.data.tls\\.crt}' | base64 -d > tls.crt && kubectl get secret chart-mutilator-certs -o jsonpath='{.data.tls\\.key}' | base64 -d > tls.key"
local_resource("certificates", cmd=cert_cmd, resource_deps=["chart-mutilator"])