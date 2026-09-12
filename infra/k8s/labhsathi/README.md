# LabhSathi Helm chart

Namespace `labhsathi`. `values.yaml` is the hackathon-scope default (single-broker Kafka, CPU-based HPA) -- a bare `helm install` works with no override file. `values-kind.yaml` and `values-prod.yaml` are overlays; see the comments in each.

## Prerequisites

This chart never templates a Secret from values -- two Secrets must already exist in the namespace before `helm install`, each referenced by name (`secretName` / `postgres.secretName` in `values.yaml`):

```bash
kubectl create namespace labhsathi
kubectl create secret generic labhsathi-anthropic-secret \
  --namespace labhsathi \
  --from-literal=ANTHROPIC_API_KEY="$ANTHROPIC_API_KEY"
kubectl create secret generic labhsathi-postgres-secret \
  --namespace labhsathi \
  --from-literal=POSTGRES_PASSWORD="$(openssl rand -hex 20)"
```

(Or, if Doppler is wired up: `doppler secrets download --no-file --format env | kubectl create secret generic labhsathi-anthropic-secret -n labhsathi --from-env-file=/dev/stdin`.)

## Local kind demo

```bash
kind create cluster --name labhsathi
# build + load images (see repo root Dockerfiles under services/*/Dockerfile and frontend/Dockerfile)
kind load docker-image labhsathi/api-gateway:latest --name labhsathi
kind load docker-image labhsathi/ocr-worker:latest --name labhsathi
kind load docker-image labhsathi/catalog-service:latest --name labhsathi
kind load docker-image labhsathi/frontend:latest --name labhsathi

helm install labhsathi . -f values-kind.yaml
kubectl get pods -n labhsathi
kubectl get hpa -n labhsathi
```

## Generating load for the HPA demo

`values-kind.yaml` already raises the upload rate limit for this cluster specifically (200 uploads/sec instead of the ~5/hour public-deployment default -- see `apiGateway.uploadRateLimit` in both values files and GitHub issue #5) so this doesn't need discovering live on camera.

Port-forward api-gateway, then fire concurrent uploads at it with one of `docs/sample-documents/*.jpg` -- 30 requests at 8 in flight is enough to push CPU over the 70% HPA threshold and watch replicas climb:

```bash
kubectl port-forward -n labhsathi svc/labhsathi-api-gateway-svc 8080:8080 &

seq 1 30 | xargs -P 8 -I{} curl -s -o /dev/null -X POST http://localhost:8080/api/documents \
  -F "document=@../../docs/sample-documents/specimen-income-certificate.jpg"

watch kubectl get hpa,pods -n labhsathi
```

`ocrWorker.maxConcurrentExtractions` (default 4, per pod) is also Helm-settable if a smaller/larger load is needed to hit the threshold predictably on the hardware being demoed on.
