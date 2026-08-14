#!/usr/bin/env bash
# Generates the Ferrofin plugin-repository manifest (Jellyfin PackageInfo[]
# format) from this repo's GitHub Releases, writing site/manifest.json.
#
# Run by the release workflow after each release publishes; the result is
# deployed to GitHub Pages, so the repository URL an admin adds to Ferrofin is
#   https://<owner>.github.io/<repo>/manifest.json
#
# Identity comes from [package.metadata.ferrofin] in Cargo.toml (the same
# values the plugin compiles in via build.rs); targetAbi comes from the
# vendored WIT contract. Each released version carries the .wasm asset's
# download URL, its MD5 (the Jellyfin-standard `checksum`) and its SHA-256
# (Ferrofin's stronger extension field, preferred by the server).
#
# Requires: gh (authenticated), jq, md5sum, sha256sum.
set -euo pipefail
cd "$(dirname "$0")/.."

REPO="${GITHUB_REPOSITORY:-$(gh repo view --json nameWithOwner -q .nameWithOwner)}"

# ── identity from Cargo.toml's [package.metadata.ferrofin] ──
meta() {
  awk -v key="$1" '
    /^\[package\.metadata\.ferrofin\]/ { insec = 1; next }
    /^\[/ { insec = 0 }
    insec && $1 == key {
      sub(/^[^=]*=[[:space:]]*"/, ""); sub(/"[[:space:]]*$/, ""); print; exit
    }' Cargo.toml
}
GUID="$(meta guid)"
NAME="$(meta name)"
DESCRIPTION="$(meta description)"
OWNER="$(meta owner)"
CATEGORY="$(meta category)"
[ -n "$GUID" ] || { echo "missing [package.metadata.ferrofin] guid" >&2; exit 1; }

# ── targetAbi from the vendored contract ──
TARGET_ABI="$(sed -n 's/^package \(ferrofin:plugin@[0-9.]*\);$/\1/p' wit/ferrofin-plugin.wit)"
[ -n "$TARGET_ABI" ] || { echo "could not read the world version from wit/" >&2; exit 1; }

# ── one VersionInfo per release with a .wasm asset ──
# Fetch the release list FIRST and fail loudly if the API call itself fails:
# a silent failure here would republish an EMPTY manifest over a good one.
releases="$(gh release list --repo "$REPO" --json tagName,publishedAt \
              -q '.[] | [.tagName, .publishedAt] | @tsv')" \
  || { echo "gh release list failed — refusing to publish an empty manifest" >&2; exit 1; }

mkdir -p site
versions='[]'
while IFS=$'\t' read -r tag published; do
  [ -n "$tag" ] || continue
  asset="$(gh release view "$tag" --repo "$REPO" \
    --json assets -q '.assets[] | select(.name | endswith(".wasm")) | .name' | head -n1)"
  if [ -z "$asset" ]; then
    echo "skip $tag: no .wasm asset" >&2
    continue
  fi
  tmp="$(mktemp -d)"
  gh release download "$tag" --repo "$REPO" --pattern "$asset" --dir "$tmp"
  md5="$(md5sum "$tmp/$asset" | cut -d' ' -f1)"
  sha="$(sha256sum "$tmp/$asset" | cut -d' ' -f1)"
  rm -rf "$tmp"
  versions="$(jq -c \
    --arg version "${tag#v}" \
    --arg abi "$TARGET_ABI" \
    --arg url "https://github.com/$REPO/releases/download/$tag/$asset" \
    --arg md5 "$md5" --arg sha "$sha" --arg ts "$published" \
    --arg rname "$REPO" --arg rurl "https://github.com/$REPO" \
    '. + [{
      version: $version, targetAbi: $abi, sourceUrl: $url,
      checksum: $md5, sha256: $sha, timestamp: $ts,
      repositoryName: $rname, repositoryUrl: $rurl
    }]' <<<"$versions")"
  echo "manifest: $tag → $asset (md5 $md5)" >&2
done <<<"$releases"

jq -n \
  --arg name "$NAME" --arg desc "$DESCRIPTION" --arg owner "$OWNER" \
  --arg cat "$CATEGORY" --arg guid "$GUID" --argjson versions "$versions" \
  '[{
    name: $name, description: $desc, overview: $desc, owner: $owner,
    category: $cat, guid: $guid, versions: $versions
  }]' > site/manifest.json

echo "wrote site/manifest.json ($(jq '.[0].versions | length' site/manifest.json) version(s))" >&2
