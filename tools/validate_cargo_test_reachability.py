from __future__ import annotations

import argparse
import json
import re
import subprocess
from collections.abc import Iterable
from pathlib import Path
from typing import Any


INLINE_TEST_PATTERN = re.compile(r"#\s*\[\s*test\s*\]")
MODULE_DECLARATION_PATTERN = re.compile(
    r"^(?P<attributes>(?:[ \t]*#\s*\[[^\]\r\n]+\][ \t]*\r?\n)*)"
    r"[ \t]*(?:pub(?:\([^)]*\))?\s+)?mod\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)"
    r"\s*;[ \t]*(?://[^\r\n]*)?$",
    re.MULTILINE,
)
PATH_ATTRIBUTE_PATTERN = re.compile(r'#\s*\[\s*path\s*=\s*"(?P<path>[^"]+)"\s*\]')
INCLUDE_PATTERN = re.compile(r'\binclude!\s*\(\s*"(?P<path>[^"]+)"\s*\)')


def audit_test_reachability(metadata: dict[str, object]) -> dict[str, object]:
    """Reject workspace targets whose inline tests Cargo will not execute."""

    workspace_members = set(metadata.get("workspace_members", []))
    packages = metadata.get("packages", [])
    violations: list[dict[str, object]] = []
    checked_target_count = 0

    for package in packages:
        if not isinstance(package, dict) or package.get("id") not in workspace_members:
            continue
        package_name = str(package.get("name", "<unknown>"))
        for target in _test_targets(package.get("targets", [])):
            checked_target_count += 1
            if target.get("test", True):
                continue

            source_path = Path(str(target["src_path"])).resolve()
            inline_test_sources = _inline_test_sources(source_path)
            inline_test_count = sum(
                source[1] for source in inline_test_sources
            )
            if inline_test_count == 0:
                continue

            violations.append(
                {
                    "package": package_name,
                    "target": str(target.get("name", "<unknown>")),
                    "target_kind": ",".join(str(kind) for kind in target["kind"]),
                    "source": source_path.as_posix(),
                    "inline_test_count": inline_test_count,
                    "inline_test_sources": [
                        path.as_posix() for path, _ in inline_test_sources
                    ],
                    "reason": (
                        "Cargo target disables tests while its module graph "
                        "contains inline #[test] cases"
                    ),
                }
            )

    return {
        "schema_version": 1,
        "checked_target_count": checked_target_count,
        "violation_count": len(violations),
        "violations": violations,
        "passed": not violations,
    }


def _test_targets(targets: object) -> Iterable[dict[str, Any]]:
    if not isinstance(targets, list):
        return ()
    return (
        target
        for target in targets
        if isinstance(target, dict)
        and isinstance(target.get("kind"), list)
        and "custom-build" not in target["kind"]
        and "src_path" in target
    )


def _inline_test_sources(source_path: Path) -> list[tuple[Path, int]]:
    pending = [source_path]
    visited: set[Path] = set()
    sources: list[tuple[Path, int]] = []

    while pending:
        candidate = pending.pop()
        if candidate in visited or not candidate.is_file():
            continue
        visited.add(candidate)
        source = candidate.read_text(encoding="utf-8")
        inline_test_count = len(INLINE_TEST_PATTERN.findall(source))
        if inline_test_count:
            sources.append((candidate, inline_test_count))
        pending.extend(_child_sources(candidate, source))

    return sorted(sources, key=lambda entry: entry[0].as_posix())


def _child_sources(source_path: Path, source: str) -> Iterable[Path]:
    module_directory = (
        source_path.parent
        if source_path.name in {"lib.rs", "main.rs", "mod.rs"}
        else source_path.parent / source_path.stem
    )
    for match in MODULE_DECLARATION_PATTERN.finditer(source):
        path_attribute = PATH_ATTRIBUTE_PATTERN.search(match.group("attributes"))
        if path_attribute is not None:
            attributed_source = source_path.parent / path_attribute.group("path")
            if attributed_source.is_file():
                yield attributed_source
            continue

        module_name = match.group("name")
        direct_module = module_directory / f"{module_name}.rs"
        nested_module = module_directory / module_name / "mod.rs"
        if direct_module.is_file():
            yield direct_module
        elif nested_module.is_file():
            yield nested_module

    for include_path in INCLUDE_PATTERN.findall(source):
        included_source = source_path.parent / include_path
        if included_source.is_file():
            yield included_source


def _cargo_metadata(manifest_path: Path) -> dict[str, object]:
    try:
        completed = subprocess.run(
            [
                "cargo",
                "metadata",
                "--locked",
                "--no-deps",
                "--format-version",
                "1",
                "--manifest-path",
                str(manifest_path),
            ],
            check=True,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
    except subprocess.CalledProcessError as error:
        detail = (error.stderr or error.stdout or "<no Cargo diagnostics>").strip()
        raise RuntimeError(f"cargo metadata failed for {manifest_path}: {detail}") from error
    metadata = json.loads(completed.stdout)
    if not isinstance(metadata, dict):
        raise ValueError("cargo metadata did not return an object")
    return metadata


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Reject Cargo targets that hide inline Rust tests from Cargo."
    )
    parser.add_argument("--manifest-path", type=Path, default=Path("Cargo.toml"))
    parser.add_argument("--json", action="store_true")
    arguments = parser.parse_args(argv)

    report = audit_test_reachability(_cargo_metadata(arguments.manifest_path))
    if arguments.json:
        print(json.dumps(report, ensure_ascii=True, indent=2, sort_keys=True))
    elif report["passed"]:
        print(f"Cargo test reachability passed for {report['checked_target_count']} libraries.")
    else:
        for violation in report["violations"]:
            print(
                f"{violation['package']}::{violation['target']}: "
                f"{violation['inline_test_count']} inline tests are unreachable"
            )
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
