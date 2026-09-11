#!/bin/sh
set -eu

# Vite env vars are baked in at build time, which would mean a separate
# image per deploy target (Compose / kind / Railway). Instead this writes
# the one runtime knob the frontend needs -- where api-gateway lives -- from
# an env var set at container start, so one built image works everywhere.
# API_BASE_URL defaults to same-origin "/api", which is correct whenever
# nginx (or the platform's ingress) proxies /api through to api-gateway.
cat > /usr/share/nginx/html/env.js <<EOF
window.__LABHSATHI_CONFIG__ = { apiBaseUrl: "${API_BASE_URL:-/api}" };
EOF

# API_PROXY_PASS must be a bare "http://host:port" with NO trailing path --
# nginx's proxy_pass forwards the full matched URI (including "/api/...")
# unchanged only when the directive itself has no path component. Set by
# the Helm chart to the in-cluster api-gateway service for kind, where
# there's no ingress/absolute URL doing this job instead (GitHub issue #2).
# The default is a connection nginx will always refuse -- an unconfigured
# proxy should 502 loudly, not silently forward /api/* to the wrong place.
export API_PROXY_PASS="${API_PROXY_PASS:-http://127.0.0.1:1}"

envsubst '${API_PROXY_PASS}' < /etc/nginx/conf.d/default.conf.template > /etc/nginx/conf.d/default.conf

exec nginx -g "daemon off;"
