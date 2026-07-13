from __future__ import annotations

import argparse
import json
import sys
import tomllib
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
PRESET_PATH = REPO_ROOT / "zircon_runtime" / "runtime-feature-presets.toml"


def load_profiles() -> list[dict[str, object]]:
    document = tomllib.loads(PRESET_PATH.read_text(encoding="utf-8"))
    if document.get("schema_version") != 1:
        raise ValueError(f"unsupported runtime feature preset schema in {PRESET_PATH}")
    profiles = document.get("profiles")
    if not isinstance(profiles, list) or not profiles:
        raise ValueError(f"runtime feature preset source has no profiles: {PRESET_PATH}")
    return profiles


def profile_by_id(profile_id: str) -> dict[str, object]:
    for profile in load_profiles():
        if profile["id"] == profile_id:
            return profile
    raise ValueError(f"unknown runtime profile {profile_id!r}")


def profile_matrix() -> dict[str, object]:
    return {
        "include": [
            {
                "profile": profile["id"],
                "cargo_feature": profile["cargo_feature"],
            }
            for profile in load_profiles()
        ]
    }


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Read the canonical Zircon runtime profile Cargo feature presets."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    feature_parser = subparsers.add_parser("feature")
    feature_parser.add_argument("profile")

    subparsers.add_parser("matrix")
    return parser.parse_args(argv)


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    if args.command == "feature":
        print(profile_by_id(args.profile)["cargo_feature"])
        return 0
    if args.command == "matrix":
        print(json.dumps(profile_matrix(), separators=(",", ":")))
        return 0
    raise AssertionError(f"unhandled command {args.command}")


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except (OSError, KeyError, TypeError, ValueError, tomllib.TOMLDecodeError) as error:
        print(f"runtime profile feature preset error: {error}", file=sys.stderr)
        raise SystemExit(2) from error
