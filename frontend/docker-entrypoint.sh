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

exec nginx -g "daemon off;"
