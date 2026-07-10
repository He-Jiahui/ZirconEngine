#!/usr/bin/env python3
from __future__ import annotations

import argparse
import re
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Iterable


REFERENCE_SETS = {
    "zircon": {
        "code": [
            "zircon_core",
            "zircon_module",
            "zircon_server",
            "zircon_script",
            "zircon_graphics",
            "zircon_editor",
        ],
        "tests": [
            "zircon_core",
            "zircon_module",
            "zircon_server",
            "zircon_script",
            "zircon_graphics",
            "zircon_editor",
            ".github/workflows/ci.yml",
        ],
    },
    "unreal": {
        "code": [
            "dev/UnrealEngine/Engine/Source/Runtime",
            "dev/UnrealEngine/Engine/Source/Editor",
            "dev/UnrealEngine/Engine/Source/Developer",
            "dev/UnrealEngine/Engine/Source/Programs",
        ],
        "tests": [
            "dev/UnrealEngine/Engine/Source/Runtime",
            "dev/UnrealEngine/Engine/Source/Editor",
            "dev/UnrealEngine/Engine/Source/Developer",
            "dev/UnrealEngine/Engine/Source/Programs",
            "dev/UnrealEngine/Samples",
            "dev/UnrealEngine/Templates",
        ],
    },
    "bevy": {
        "code": [
            "dev/bevy/crates",
            "dev/bevy/src",
        ],
        "tests": [
            "dev/bevy/tests",
            "dev/bevy/tests-integration",
            "dev/bevy/examples",
        ],
    },
    "fyrox": {
        "code": [
            "dev/Fyrox/fyrox",
            "dev/Fyrox/fyrox-graphics",
            "dev/Fyrox/fyrox-resource",
            "dev/Fyrox/fyrox-scripts",
            "dev/Fyrox/editor",
        ],
        "tests": [
            "dev/Fyrox/fyrox",
            "dev/Fyrox/fyrox-graphics",
            "dev/Fyrox/fyrox-resource",
            "dev/Fyrox/fyrox-scripts",
            "dev/Fyrox/editor",
        ],
    },
    "godot": {
        "code": [
            "dev/godot/core",
            "dev/godot/scene",
            "dev/godot/modules",
            "dev/godot/editor",
        ],
        "tests": [
            "dev/godot/tests",
            "dev/godot/modules/gdscript/tests",
        ],
    },
    "graphics": {
        "code": [
            "dev/Graphics/Packages/com.unity.render-pipelines.core",
            "dev/Graphics/Packages/com.unity.render-pipelines.universal",
            "dev/Graphics/Packages/com.unity.render-pipelines.high-definition",
            "dev/Graphics/Packages/com.unity.shadergraph",
            "dev/Graphics/Packages/com.unity.visualeffectgraph",
            "dev/Graphics/com.unity.postprocessing",
        ],
        "tests": [
            "dev/Graphics/Tests",
            "dev/Graphics/TestProjects",
            "dev/Graphics/Packages/com.unity.render-pipelines.core",
            "dev/Graphics/Packages/com.unity.render-pipelines.universal",
            "dev/Graphics/Packages/com.unity.render-pipelines.high-definition",
            "dev/Graphics/Packages/com.unity.shadergraph",
            "dev/Graphics/Packages/com.unity.visualeffectgraph",
        ],
    },
    "piccolo": {
        "code": ["dev/Piccolo/engine/source"],
        "tests": ["dev/Piccolo/engine/source"],
    },
    "slint": {
        "code": [
            "dev/slint/api",
            "dev/slint/internal",
            "dev/slint/ui-libraries",
        ],
        "tests": [
            "dev/slint/tests",
            "dev/slint/examples",
            "dev/slint/demos",
        ],
    },
    "theatre": {
        "code": [
            "dev/theatre/packages/core",
            "dev/theatre/packages/studio",
            "dev/theatre/packages/react",
            "dev/theatre/packages/utils",
        ],
        "tests": [
            "dev/theatre/compat-tests",
            "dev/theatre/examples",
            "dev/theatre/packages",
        ],
    },
}

LANGUAGE_ALIASES = {
    "ue": "unreal",
    "ue5": "unreal",
    "unrealengine": "unreal",
    "unreal-engine": "unreal",
    "unity": "graphics",
    "unity-srp": "graphics",
    "srp": "graphics",
}

TEXT_SUFFIXES = {
    ".c",
    ".cc",
    ".cpp",
    ".compute",
    ".cxx",
    ".cs",
    ".cts",
    ".h",
    ".hh",
    ".hlsl",
    ".hpp",
    ".html",
    ".inl",
    ".ini",
    ".java",
    ".js",
    ".json",
    ".jsx",
    ".lua",
    ".md",
    ".mjs",
    ".py",
    ".rs",
    ".shader",
    ".sh",
    ".slint",
    ".toml",
    ".txt",
    ".tsx",
    ".uasset",
    ".usf",
    ".ush",
    ".uxml",
    ".yaml",
    ".yml",
}


def find_repo_root(start: Path) -> Path:
    resolved = start.resolve()
    for parent in resolved.parents:
        if (parent / ".codex").exists() and (parent / "Cargo.toml").exists():
            return parent
    raise RuntimeError(f"Could not locate repository root from {start}")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Search the bundled reference engine trees for feature evidence."
    )
    parser.add_argument("pattern", help="Regex pattern to search for.")
    parser.add_argument(
        "--languages",
        help=(
            "Comma-separated subset of engines: "
            "zircon,unreal,bevy,fyrox,godot,graphics,piccolo,slint,theatre "
            "(aliases: unrealengine, ue, ue5, unity, srp)"
        ),
    )
    parser.add_argument(
        "--tests-only",
        action="store_true",
        help="Search only configured test locations.",
    )
    parser.add_argument(
        "--code-only",
        action="store_true",
        help="Search only configured source locations.",
    )
    parser.add_argument(
        "--max-count",
        type=int,
        default=8,
        help="Maximum number of matches to print per language.",
    )
    parser.add_argument(
        "--case-sensitive",
        action="store_true",
        help="Make the search case-sensitive.",
    )
    return parser.parse_args()


def selected_languages(raw: str | None) -> list[str]:
    if not raw:
        return list(REFERENCE_SETS)
    items = [
        LANGUAGE_ALIASES.get(item.strip().lower(), item.strip().lower())
        for item in raw.split(",")
        if item.strip()
    ]
    unknown = [item for item in items if item not in REFERENCE_SETS]
    if unknown:
        raise ValueError(f"Unknown languages: {', '.join(unknown)}")
    return items


def selected_paths(repo_root: Path, language: str, tests_only: bool, code_only: bool) -> list[Path]:
    if tests_only and code_only:
        raise ValueError("--tests-only and --code-only cannot be used together")
    groups = []
    if tests_only:
        groups = ["tests"]
    elif code_only:
        groups = ["code"]
    else:
        groups = ["code", "tests"]

    paths: list[Path] = []
    for group in groups:
        for raw in REFERENCE_SETS[language][group]:
            path = repo_root / raw
            if path.exists():
                paths.append(path)
    return paths


def ripgrep_search(pattern: str, paths: list[Path], max_count: int, case_sensitive: bool) -> list[str]:
    if not paths:
        return []
    command = [
        "rg",
        "--line-number",
        "--with-filename",
        "--color",
        "never",
        "--max-count",
        str(max_count),
    ]
    if not case_sensitive:
        command.append("--ignore-case")
    command.append(pattern)
    command.extend(str(path) for path in paths)
    result = subprocess.run(command, capture_output=True, text=False, check=False)
    if result.returncode not in (0, 1):
        stderr = result.stderr.decode("utf-8", errors="replace").strip()
        raise RuntimeError(stderr or "ripgrep search failed")
    if result.returncode == 1:
        return []
    stdout = result.stdout.decode("utf-8", errors="replace")
    return [line for line in stdout.splitlines() if line.strip()][:max_count]


def file_candidates(paths: Iterable[Path]) -> Iterable[Path]:
    for path in paths:
        if path.is_file():
            yield path
            continue
        for child in path.rglob("*"):
            if child.is_file() and child.suffix.lower() in TEXT_SUFFIXES:
                yield child


def python_search(pattern: str, paths: list[Path], max_count: int, case_sensitive: bool) -> list[str]:
    flags = 0 if case_sensitive else re.IGNORECASE
    regex = re.compile(pattern, flags)
    matches: list[str] = []
    for candidate in file_candidates(paths):
        try:
            with candidate.open("r", encoding="utf-8", errors="ignore") as handle:
                for line_number, line in enumerate(handle, start=1):
                    if regex.search(line):
                        matches.append(f"{candidate}:{line_number}:{line.rstrip()}")
                        if len(matches) >= max_count:
                            return matches
        except OSError:
            continue
    return matches


def main() -> int:
    args = parse_args()
    try:
        languages = selected_languages(args.languages)
        repo_root = find_repo_root(Path(__file__))
    except (RuntimeError, ValueError) as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2

    use_ripgrep = shutil.which("rg") is not None

    for language in languages:
        try:
            paths = selected_paths(repo_root, language, args.tests_only, args.code_only)
        except ValueError as exc:
            print(f"error: {exc}", file=sys.stderr)
            return 2

        print(f"[{language}]")
        if not paths:
            print("  no configured paths found")
            print()
            continue

        if use_ripgrep:
            try:
                matches = ripgrep_search(args.pattern, paths, args.max_count, args.case_sensitive)
            except (OSError, RuntimeError):
                matches = python_search(args.pattern, paths, args.max_count, args.case_sensitive)
        else:
            matches = python_search(args.pattern, paths, args.max_count, args.case_sensitive)
        if not matches:
            print("  no matches")
            print()
            continue

        for match in matches:
            print(f"  {match}")
        print()

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
