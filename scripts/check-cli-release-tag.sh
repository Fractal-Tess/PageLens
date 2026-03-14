#!/usr/bin/env bash

set -euo pipefail

if [[ $# -ne 1 ]]; then
  printf 'usage: %s <tag>\n' "$0" >&2
  exit 1
fi

tag="$1"
repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
expected_version="$(python3 "$repo_root/scripts/get-workspace-version.py")"
expected_tag="cli-v${expected_version}"

if [[ "$tag" != "$expected_tag" ]]; then
  printf 'release tag mismatch: expected %s but got %s\n' "$expected_tag" "$tag" >&2
  exit 1
fi

printf 'validated CLI release tag %s\n' "$tag"
