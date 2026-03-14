#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
version="$(python3 "$repo_root/scripts/get-workspace-version.py")"
tag="cli-v${version}"

git -C "$repo_root" rev-parse --verify HEAD >/dev/null

if git -C "$repo_root" rev-parse --verify --quiet "refs/tags/${tag}" >/dev/null; then
  printf 'tag %s already exists\n' "$tag" >&2
  exit 1
fi

if git -C "$repo_root" ls-remote --exit-code --tags origin "refs/tags/${tag}" >/dev/null 2>&1; then
  printf 'remote tag %s already exists on origin\n' "$tag" >&2
  exit 1
fi

git -C "$repo_root" tag -a "$tag" -m "Release ${tag}"
git -C "$repo_root" push origin "$tag"

printf 'pushed %s\n' "$tag"
