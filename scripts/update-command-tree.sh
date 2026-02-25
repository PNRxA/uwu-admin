#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CONTINUWUITY_DIR="$REPO_ROOT/../continuwuity"
FORK_URL="https://github.com/PNRxA/continuwuity.git"
UPSTREAM_URL="https://forgejo.ellis.link/continuwuation/continuwuity.git"

PUSH=false
while getopts "w" opt; do
  case $opt in
    w) PUSH=true ;;
    *) echo "Usage: $0 [-w] [tag]"; exit 1 ;;
  esac
done
shift $((OPTIND - 1))

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

# Validate and check out the target tag for generation if specified
if [ "$TARGET_REF" != "main" ]; then
  if ! git tag -l "$TARGET_REF" | grep -q .; then
    echo "Error: tag '$TARGET_REF' not found" >&2; exit 1
  fi
  echo "Checking out tag: $TARGET_REF..."
  git checkout "refs/tags/$TARGET_REF"
fi

# Generate the command tree
echo "Generating command-tree.json..."
cargo xtask generate-command-tree --output "$REPO_ROOT/shared/command-tree.json"

# Return to main if we checked out a tag
if [ "$TARGET_REF" != "main" ]; then
  echo "Returning to main..."
  git checkout main
fi

echo "Done."
