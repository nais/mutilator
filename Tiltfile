load('ext://cert_manager', 'deploy_cert_manager')
load('ext://helm_resource', 'helm_resource', 'helm_repo')
load('ext://local_output', 'local_output')

APP_NAME="mutilator"

deploy_cert_manager()

helm_repo('aiven', 'https://aiven.github.io/aiven-charts')
helm_resource('aiven-operator-crds', 'aiven/aiven-operator-crds', resource_deps=['aiven'], pod_readiness="ignore")

config.define_bool("debugger", usage="Enable directing webhook requests out of the cluster to your locally running instance")
cfg = config.parse()

ignore = str(read_file(".earthignore")).split("\n")
host_ip = local_output("/sbin/ip route show default | awk '/default/ { print $9 }'")
mutilator_objects = [
    "chart-mutilator:mutatingwebhookconfiguration",
    "chart-mutilator:networkpolicy",
    "chart-mutilator:certificate",
    "chart-mutilator:issuer",
]
if cfg.get("debugger", False):
    mutilator_objects.append("chart-mutilator:endpointslice")
    mutilator_objects.append("chart-mutilator:service")

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
    "debugger.enabled={}".format("true" if cfg.get("debugger", False) else "false"),
    "debugger.host={}".format(host_ip),
]))
k8s_resource(
    workload="chart-{}".format(APP_NAME),
    resource_deps=["aiven-operator-crds"],
    objects=mutilator_objects,
)

# Update locally stored certificates from cluster
cert_cmd = "kubectl get secret chart-mutilator-certs -o jsonpath='{.data.tls\\.crt}' | base64 -d > tls.crt && kubectl get secret chart-mutilator-certs -o jsonpath='{.data.tls\\.key}' | base64 -d > tls.key"
local_resource("certificates", cmd=cert_cmd, resource_deps=["chart-mutilator"])