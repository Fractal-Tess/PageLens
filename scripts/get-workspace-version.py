#!/usr/bin/env python3

from __future__ import annotations

import pathlib
import sys
import tomllib


def main() -> int:
    repo_root = pathlib.Path(__file__).resolve().parent.parent
    cargo_toml_path = repo_root / "Cargo.toml"

    with cargo_toml_path.open("rb") as cargo_file:
        cargo = tomllib.load(cargo_file)

    version = cargo.get("workspace", {}).get("package", {}).get("version")
    if not isinstance(version, str) or not version:
        raise SystemExit("workspace.package.version is missing from Cargo.toml")

    print(version)
    return 0


if __name__ == "__main__":
    sys.exit(main())
