#!/usr/bin/env bash
# Verifies every scheme's source_url in the catalog:
#   1. its domain is on the committed allowlist (scripts/trusted-source-domains.txt)
#      -- catches a typo'd or fraudulent domain slipping into schemes.json,
#      since an unlisted domain fails closed instead of silently passing.
#   2. it actually resolves and returns HTTP 200 (following redirects)
#   3. any redirect it follows lands on a domain that's *also* on the
#      allowlist -- catches a legitimate-looking source getting hijacked or
#      redirected somewhere unexpected between one run and the next.
#
# This is intentionally simple: curl + jq, no new service, no new dependency.
# Real network calls to real government sites -- run it when you actually
# want current answers, not as part of every commit.
#
# Usage: scripts/verify-source-links.sh [path/to/schemes.json]

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCHEMES_JSON="${1:-$REPO_ROOT/services/labhsathi-core/data/schemes.json}"
ALLOWLIST="$REPO_ROOT/scripts/trusted-source-domains.txt"
TIMEOUT_SECS=10

if [[ ! -f "$SCHEMES_JSON" ]]; then
  echo "error: schemes file not found at $SCHEMES_JSON" >&2
  exit 2
fi
if [[ ! -f "$ALLOWLIST" ]]; then
  echo "error: allowlist not found at $ALLOWLIST" >&2
  exit 2
fi

domain_of() {
  # Strips scheme, path, port, credentials -- leaves just the host.
  local url="$1"
  url="${url#*://}"
  url="${url%%/*}"
  url="${url##*@}"
  url="${url%%:*}"
  echo "$url"
}

is_allowlisted() {
  grep -Fxq "$1" "$ALLOWLIST"
}

total=0
ok=0
bad_status=0
domain_not_allowlisted=0
redirected_off_allowlist=0
network_error=0

printf '%-24s %-32s %-8s %s\n' "SCHEME_ID" "DOMAIN" "STATUS" "VERDICT"
printf '%s\n' "----------------------------------------------------------------------------------"

while IFS=$'\t' read -r id source_url; do
  [[ -z "$source_url" || "$source_url" == "null" ]] && continue
  total=$((total + 1))

  domain="$(domain_of "$source_url")"

  if ! is_allowlisted "$domain"; then
    printf '%-24s %-32s %-8s %s\n' "$id" "$domain" "-" "DOMAIN_NOT_ALLOWLISTED"
    domain_not_allowlisted=$((domain_not_allowlisted + 1))
    continue
  fi

  response="$(curl -sS -L --max-time "$TIMEOUT_SECS" -o /dev/null \
      -w '%{http_code}|%{url_effective}' "$source_url" 2>/dev/null)"

  if [[ -z "$response" ]]; then
    printf '%-24s %-32s %-8s %s\n' "$id" "$domain" "ERR" "NETWORK_ERROR"
    network_error=$((network_error + 1))
    continue
  fi

  status="${response%%|*}"
  effective_url="${response#*|}"
  effective_domain="$(domain_of "$effective_url")"

  if [[ "$status" != "200" ]]; then
    printf '%-24s %-32s %-8s %s\n' "$id" "$domain" "$status" "BAD_STATUS"
    bad_status=$((bad_status + 1))
  elif ! is_allowlisted "$effective_domain"; then
    printf '%-24s %-32s %-8s %s\n' "$id" "$domain" "$status" "REDIRECTED_OFF_ALLOWLIST -> $effective_domain"
    redirected_off_allowlist=$((redirected_off_allowlist + 1))
  else
    printf '%-24s %-32s %-8s %s\n' "$id" "$domain" "$status" "OK"
    ok=$((ok + 1))
  fi
done < <(jq -r '.[] | [.id, (.source_url // "null")] | @tsv' "$SCHEMES_JSON")

printf '%s\n' "----------------------------------------------------------------------------------"
echo "checked $total schemes: $ok OK, $bad_status bad status, $domain_not_allowlisted domain not allowlisted, $redirected_off_allowlist redirected off allowlist, $network_error network errors"

if [[ $((bad_status + domain_not_allowlisted + redirected_off_allowlist + network_error)) -gt 0 ]]; then
  exit 1
fi
exit 0
