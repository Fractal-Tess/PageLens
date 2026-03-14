#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
version="$(python3 "$repo_root/scripts/get-workspace-version.py")"
tag="cli-v${version}"
remote_name="$(git -C "$repo_root" rev-parse --abbrev-ref --symbolic-full-name '@{upstream}' 2>/dev/null | cut -d/ -f1 || true)"

if [[ -z "$remote_name" ]]; then
  if git -C "$repo_root" remote get-url origin >/dev/null 2>&1; then
    remote_name="origin"
  elif git -C "$repo_root" remote get-url github >/dev/null 2>&1; then
    remote_name="github"
  else
    printf 'could not determine git remote for pushing %s\n' "$tag" >&2
    exit 1
  fi
fi

git -C "$repo_root" rev-parse --verify HEAD >/dev/null

if git -C "$repo_root" rev-parse --verify --quiet "refs/tags/${tag}" >/dev/null; then
  printf 'tag %s already exists\n' "$tag" >&2
  exit 1
fi

if git -C "$repo_root" ls-remote --exit-code --tags "$remote_name" "refs/tags/${tag}" >/dev/null 2>&1; then
  printf 'remote tag %s already exists on %s\n' "$tag" "$remote_name" >&2
  exit 1
fi

git -C "$repo_root" tag -a "$tag" -m "Release ${tag}"
git -C "$repo_root" push "$remote_name" "$tag"

printf 'pushed %s to %s\n' "$tag" "$remote_name"
