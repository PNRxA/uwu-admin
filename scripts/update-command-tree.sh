#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CONTINUWUITY_DIR="$REPO_ROOT/../continuwuity"
FORK_URL="https://github.com/PNRxA/continuwuity.git"
UPSTREAM_URL="https://forgejo.ellis.link/continuwuation/continuwuity.git"

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

# Pull latest from upstream and rebase
echo "Fetching upstream..."
git fetch upstream
echo "Rebasing on upstream/main..."
git rebase upstream/main

# Generate the command tree
echo "Generating command-tree.json..."
cargo xtask generate-command-tree --output "$REPO_ROOT/shared/command-tree.json"
echo "Done."
