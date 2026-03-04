#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CONTINUWUITY_DIR="$REPO_ROOT/../continuwuity"
FORK_URL="https://github.com/PNRxA/continuwuity.git"
UPSTREAM_URL="https://forgejo.ellis.link/continuwuation/continuwuity.git"

PUSH=false
CREATE_PR=false
while getopts "wp" opt; do
  case $opt in
    w) PUSH=true ;;
    p) CREATE_PR=true ;;
    *) echo "Usage: $0 [-w] [-p] [tag]"; exit 1 ;;
  esac
done
shift $((OPTIND - 1))

# -p requires a version tag
if [ "$CREATE_PR" = true ]; then
  if [ -z "${1:-}" ] || [ "$1" = "main" ]; then
    echo "Error: -p requires a version tag argument (e.g. v0.5.6)" >&2; exit 1
  fi
  if ! command -v gh &>/dev/null; then
    echo "Error: gh CLI is required for -p" >&2; exit 1
  fi
  if ! gh auth status &>/dev/null; then
    echo "Error: gh CLI is not authenticated (run 'gh auth login')" >&2; exit 1
  fi
fi

# Clone the fork if it doesn't exist
if [ ! -d "$CONTINUWUITY_DIR" ]; then
  echo "Cloning continuwuity fork..."
  git clone "$FORK_URL" "$CONTINUWUITY_DIR"
fi

cd "$CONTINUWUITY_DIR"

# Add upstream remote if missing
if ! git remote get-url upstream &>/dev/null; then
  echo "Adding upstream remote..."
  git remote add upstream "$UPSTREAM_URL"
fi

# Determine target ref (tag or branch)
TARGET_REF="${1:-main}"

# Ensure we're on main before doing anything
git checkout main

# Pull latest from upstream (including tags)
echo "Fetching upstream (with tags)..."
git fetch upstream --tags
echo "Rebasing on upstream/main..."
git rebase upstream/main

if [ "$PUSH" = true ]; then
  echo "Pushing tags to origin..."
  git push origin --tags
  echo "Pushing main to origin..."
  git push --force-with-lease origin main
fi

# If a tag is specified, rebase fork commits onto it in a temp branch
if [ "$TARGET_REF" != "main" ]; then
  if ! git tag -l "$TARGET_REF" | grep -q .; then
    echo "Error: tag '$TARGET_REF' not found" >&2; exit 1
  fi
  echo "Creating temp branch with fork commits on tag: $TARGET_REF..."
  git checkout -b _generate-tmp
  git rebase --onto "refs/tags/$TARGET_REF" upstream/main
fi

# Generate the command tree
echo "Generating command-tree.json..."
cargo xtask generate-command-tree --output "$REPO_ROOT/shared/command-tree.json"

# Clean up temp branch if we created one
if [ "$TARGET_REF" != "main" ]; then
  echo "Returning to main..."
  git checkout main
  git branch -D _generate-tmp
fi

# Create PR if requested
if [ "$CREATE_PR" = true ]; then
  cd "$REPO_ROOT"

  # Update version references
  BARE_VERSION="${TARGET_REF#v}-0"   # v0.5.6 → 0.5.6-0

  # web/package.json — use jq to set .version
  jq --arg v "$BARE_VERSION" '.version = $v' web/package.json > web/package.json.tmp \
    && mv web/package.json.tmp web/package.json

  # api/Cargo.toml — replace version under [package]
  sed -i '/^\[package\]/,/^\[/{s/^version = ".*"/version = "'"$BARE_VERSION"'"/}' api/Cargo.toml

  # README.md — update Docker image tag (use base version as the floating tag)
  sed -i "s|pnrxa/uwu-admin:v[0-9][0-9.a-zA-Z-]*|pnrxa/uwu-admin:${TARGET_REF}|g" README.md

  if [ -z "$(git status --porcelain shared/command-tree.json web/package.json api/Cargo.toml README.md)" ]; then
    echo "No changes detected — skipping PR creation."
    echo "Done."
    exit 0
  fi

  PR_BRANCH="update/continuwuity-${TARGET_REF}"

  # Close any existing PR for this branch before recreating
  git checkout main
  EXISTING_PR=$(gh pr list --head "$PR_BRANCH" --state open --json number --jq '.[0].number' 2>/dev/null || true)
  if [ -n "$EXISTING_PR" ]; then
    echo "Closing superseded PR #$EXISTING_PR..."
    gh pr close "$EXISTING_PR" --comment "Superseded by a new run for $TARGET_REF."
  fi
  git branch -D "$PR_BRANCH" 2>/dev/null || true
  git push origin --delete "$PR_BRANCH" 2>/dev/null || true

  echo "Creating PR branch: $PR_BRANCH"
  git checkout -b "$PR_BRANCH"
  git add shared/command-tree.json web/package.json api/Cargo.toml README.md
  git commit -m "Update to $TARGET_REF"
  git push -u origin "$PR_BRANCH"

  gh pr create \
    --title "Update to $TARGET_REF" \
    --body "Updates command tree, package versions, and Docker tag to match continuwuity [$TARGET_REF](https://forgejo.ellis.link/continuwuation/continuwuity/releases/tag/$TARGET_REF)." \
    --base main \
    --assignee PNRxA \
    --reviewer PNRxA

  git checkout main
  git branch -D "$PR_BRANCH"
fi

echo "Done."
