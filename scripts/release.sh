#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

if [ -z "${1:-}" ]; then
  echo "Usage: $0 <version-tag>  (e.g. v0.5.6 or v0.5.6-1)" >&2
  exit 1
fi

VERSION="$1"

if [[ ! "$VERSION" =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9a-zA-Z.]+)?$ ]]; then
  echo "Error: version must match vX.Y.Z or vX.Y.Z-suffix (got '$VERSION')" >&2
  exit 1
fi

BARE_VERSION="${VERSION#v}"

cd "$REPO_ROOT"

# Check for uncommitted changes
if [ -n "$(git status --porcelain)" ]; then
  echo "Error: working tree is dirty — commit or stash changes first" >&2
  exit 1
fi

# Check tag doesn't already exist
if git rev-parse "$VERSION" &>/dev/null; then
  echo "Error: tag $VERSION already exists" >&2
  exit 1
fi

echo "Updating version to $BARE_VERSION..."

# web/package.json
jq --arg v "$BARE_VERSION" '.version = $v' web/package.json > web/package.json.tmp \
  && mv web/package.json.tmp web/package.json

# api/Cargo.toml
sed -i '/^\[package\]/,/^\[/{s/^version = ".*"/version = "'"$BARE_VERSION"'"/}' api/Cargo.toml

# README.md — update Docker image tag (use base version as the floating tag)
BASE_VERSION=$(echo "$VERSION" | sed 's/-[0-9]*$//')
sed -i "s|pnrxa/uwu-admin:v[0-9][0-9.a-zA-Z-]*|pnrxa/uwu-admin:${BASE_VERSION}|g" README.md

# Show what changed
echo ""
git diff --stat
echo ""
git diff

echo ""
read -rp "Commit, tag $VERSION, and push? [y/N] " confirm
if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
  echo "Aborted. Reverting changes..."
  git checkout -- web/package.json api/Cargo.toml README.md
  exit 1
fi

git add web/package.json api/Cargo.toml README.md
git commit -m "Release $VERSION"
git tag "$VERSION"
git push origin HEAD "$VERSION"

echo "Released $VERSION"
