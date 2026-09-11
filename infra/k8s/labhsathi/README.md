# LabhSathi Helm chart

Namespace `labhsathi`. `values.yaml` is the hackathon-scope default (single-broker Kafka, CPU-based HPA) -- a bare `helm install` works with no override file. `values-kind.yaml` and `values-prod.yaml` are overlays; see the comments in each.

## Prerequisites

This chart never templates a Secret from values -- the `ANTHROPIC_API_KEY` Secret must already exist in the namespace before `helm install`, referenced by name (`secretName` in `values.yaml`, default `labhsathi-anthropic-secret`):

```bash
kubectl create namespace labhsathi
kubectl create secret generic labhsathi-anthropic-secret \
  --namespace labhsathi \
  --from-literal=ANTHROPIC_API_KEY="$ANTHROPIC_API_KEY"
```

(Or, if Doppler is wired up: `doppler secrets download --no-file --format env | kubectl create secret generic labhsathi-anthropic-secret -n labhsathi --from-env-file=/dev/stdin`.)

## Local kind demo

```bash
kind create cluster --name labhsathi
# build + load images (see repo root Dockerfiles under services/*/Dockerfile and frontend/Dockerfile)
kind load docker-image labhsathi/api-gateway:latest --name labhsathi
kind load docker-image labhsathi/ocr-worker:latest --name labhsathi
kind load docker-image labhsathi/frontend:latest --name labhsathi

helm install labhsathi . -f values-kind.yaml
kubectl get pods -n labhsathi
kubectl get hpa -n labhsathi
```

To generate load against ocr-worker for the HPA demo, raise the api-gateway upload rate limit for this session first (`services/api-gateway/src/rate_limit.rs` is a ~5/hour default) rather than discovering the limit live on camera.
