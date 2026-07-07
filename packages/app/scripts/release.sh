#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$APP_ROOT/../.." && pwd)"

KIND="${1:-patch}"
FROM_BRANCH="${FROM_BRANCH:-master}"

case "$KIND" in
  major|minor|patch) ;;
  --dry-run) KIND="patch"; DRY_RUN=1 ;;
  *) echo "Usage: release.sh [major|minor|patch|--dry-run]" >&2; exit 1 ;;
esac

DRY_RUN="${DRY_RUN:-0}"
TAURI_CONF="$APP_ROOT/src-tauri/tauri.conf.json"
CURRENT="$(node -e "console.log(JSON.parse(require('fs').readFileSync('$TAURI_CONF')).version)")"

if [ "$DRY_RUN" = "1" ]; then
  NEXT="$(node -e "import('$SCRIPT_DIR/bump-version.mjs').then(m => console.log(m.nextVersion('$CURRENT', '$KIND')))")"
  echo "Dry release: v$CURRENT -> v$NEXT from origin/$FROM_BRANCH"
  exit 0
fi

if ! git -C "$REPO_ROOT" diff --quiet || ! git -C "$REPO_ROOT" diff --cached --quiet; then
  echo "Working tree is not clean. Commit or stash changes before releasing." >&2
  exit 1
fi

command -v gh >/dev/null || { echo "gh CLI is required." >&2; exit 1; }
gh auth status >/dev/null || { echo "gh CLI is not authenticated." >&2; exit 1; }

git -C "$REPO_ROOT" fetch origin --prune
git -C "$REPO_ROOT" switch -c "release/tmp-$$" "origin/$FROM_BRANCH"

NEXT="$(cd "$APP_ROOT" && node scripts/bump-version.mjs "$KIND")"
RELEASE_BRANCH="release/v$NEXT"

git -C "$REPO_ROOT" branch -m "$RELEASE_BRANCH"
git -C "$REPO_ROOT" add package.json packages/app/package.json packages/app/src-tauri/tauri.conf.json packages/app/src-tauri/Cargo.toml packages/app/src-tauri/Cargo.lock
git -C "$REPO_ROOT" commit -m "chore: bump version to v$NEXT [skip ci]"
git -C "$REPO_ROOT" push -u origin "$RELEASE_BRANCH"

gh pr create \
  --base master \
  --head "$RELEASE_BRANCH" \
  --title "Release v$NEXT" \
  --body "Prepare Clarus v$NEXT for signed desktop release."

echo "Release branch pushed. Merge the PR, then tag the master merge commit:"
echo "  git switch master && git pull origin master"
echo "  git tag v$NEXT && git push origin v$NEXT"
