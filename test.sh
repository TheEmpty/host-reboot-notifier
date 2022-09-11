#!/bin/bash

set -ex

cargo fmt
cargo clippy -- -D warnings
cargo build --release

USER="theempty"
NAME="host-reboot-notifier"
BUILDX="pensive_albattani"
PLATFORMS="linux/amd64,linux/arm64"

TAGS=(
192.168.7.7:5000/${USER}/${NAME}
)

function join_tags {
    for tag in "${TAGS[@]}"; do
        printf %s " -t $tag"
    done
}

docker buildx build --builder ${BUILDX} $(join_tags) --push --platform=${PLATFORMS} .

kubectl rollout restart daemonset/${NAME} || true
kubectl exec -n registry $(kubectl get po -n registry -l app=registry -o=name) -- bin/registry garbage-collect /etc/docker/registry/config.yml || true
