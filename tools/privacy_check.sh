#!/usr/bin/env bash
# Privacy / public-safety check for tracked files (and dist/ if present).
# Exit 0 and print "PRIVACY CHECK: PASS" when clean; otherwise list findings and exit 1.
# This script is excluded from content scanning because it necessarily contains the patterns.
set -uo pipefail

cd "$(git rev-parse --show-toplevel)" || exit 2
SELF="tools/privacy_check.sh"
fail=0

mapfile -t FILES < <(git ls-files --cached --others --exclude-standard | grep -vxF "$SELF")

# name|extended-regex|allowlist-regex (matching lines are ignored; empty = none)
CHECKS=(
  "private key|-----BEGIN [A-Z ]*PRIVATE KEY-----|"
  "cloud key|AKIA[0-9A-Z]{16}|"
  "git host token|gh[pousr]_[A-Za-z0-9]{20,}|github_pat_[A-Za-z0-9_]{20,}|"
  "chat token|xox[abprs]-[A-Za-z0-9-]{10,}|"
  "generic secret|(api[_-]?key|secret|passw(or)?d|auth[_-]?token)[\"']?[[:space:]]*[:=][[:space:]]*[\"'][^\"'[:space:]]{8,}|"
  "email address|[A-Za-z0-9._%+-]+@[A-Za-z0-9-]+(\.[A-Za-z0-9-]+)*\.[A-Za-z]{2,}|[0-9]+\+[A-Za-z0-9-]+@users\.noreply\.github\.com"
  "IPv4 address|(^|[^0-9.])([0-9]{1,3}\.){3}[0-9]{1,3}([^0-9.]|$)|"
  "MAC address|([0-9A-Fa-f]{2}[:-]){5}[0-9A-Fa-f]{2}|"
  "GPS coordinate|(lat(itude)?|lon(gitude)?|lng)[\"']?[[:space:]]*[:=][[:space:]]*-?[0-9]{1,3}\.[0-9]{4,}|"
  "host path|(/home/[A-Za-z0-9_.-]+|/Users/[A-Za-z0-9_.-]+|[A-Za-z]:\\\\Users\\\\)|"
  "external URL|(https?|wss?|ftp)://|registry\+https://github\.com/rust-lang/crates\.io-index"
  "protocol-relative resource|(src|href)=[\"']//|url\([\"']?//|"
  "tracker/CDN|(google-analytics|googletagmanager|gtag\(|fonts\.googleapis|fonts\.gstatic|jsdelivr|unpkg|cdnjs|sentry|mixpanel|segment\.io|hotjar|plausible|posthog)|"
  "runtime network/storage API|(fetch\(|XMLHttpRequest|sendBeacon|new WebSocket|EventSource|localStorage|sessionStorage|indexedDB)|"
)

DIST_ALLOW='^[0-9]+:[[:space:]]*// https://developer\.mozilla\.org/|^[0-9]+:[[:space:]]*module_or_path = fetch\(module_or_path\);$'

scan() {
  local label="$1"; shift
  local f entry name re allow hits
  for entry in "${CHECKS[@]}"; do
    name="${entry%%|*}"; rest="${entry#*|}"
    re="${rest%|*}"; allow="${rest##*|}"
    for f in "$@"; do
      [ -f "$f" ] || continue
      grep -Iq . "$f" 2>/dev/null || continue   # skip binary / empty
      hits=$(grep -nEi -- "$re" "$f" || true)
      [ -n "$allow" ] && hits=$(grep -vEi -- "$allow" <<< "$hits" || true)
      # Generated wasm-bindgen glue: a doc-comment link and the same-origin wasm load.
      [ "$label" = dist ] && hits=$(grep -vE -- "$DIST_ALLOW" <<< "$hits" || true)
      if [ -n "$hits" ]; then
        fail=1
        while IFS= read -r line; do
          printf '  [%s] %s (%s): %s\n' "$label" "$f" "$name" "${line:0:160}"
        done <<< "$hits"
      fi
    done
  done
}

echo "Scanning ${#FILES[@]} tracked/unignored files..."
scan tracked "${FILES[@]}"

if [ -d dist ]; then
  mapfile -t DIST < <(find dist -type f \( -name '*.html' -o -name '*.js' -o -name '*.css' \))
  echo "Scanning ${#DIST[@]} built files in dist/..."
  scan dist "${DIST[@]}"
fi

if [ "$fail" -eq 0 ]; then
  echo "PRIVACY CHECK: PASS"
else
  echo "PRIVACY CHECK: FAIL"
  exit 1
fi
