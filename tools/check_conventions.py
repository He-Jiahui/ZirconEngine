from __future__ import annotations

import argparse
from collections import Counter
import json
import subprocess
from dataclasses import asdict, dataclass
from pathlib import Path, PureWindowsPath
from typing import Iterable


DOCUMENT_PATH_FIELDS = ("implementation_files", "related_code")


@dataclass(frozen=True)
class ConventionCommand:
    name: str
    argv: tuple[str, ...]


def convention_commands() -> list[ConventionCommand]:
    return [
        ConventionCommand("fmt", ("cargo", "fmt", "--all", "--check")),
        ConventionCommand(
            "clippy",
            (
                "cargo",
                "clippy",
                "-p",
                "zircon_runtime_interface",
                "-p",
                "zircon_app",
                "--all-targets",
                "--no-deps",
                "--locked",
                "--",
                "-D",
                "warnings",
            ),
        ),
    ]


def audit_document_paths(repo_root: Path) -> dict[str, object]:
    repo_root = repo_root.resolve()
    docs_root = repo_root / "docs"
    document_count = 0
    checked_path_count = 0
    violations: list[dict[str, object]] = []
    path_reason_cache: dict[str, str | None] = {}

    if not docs_root.is_dir():
        return {
            "schema_version": 1,
            "document_count": 0,
            "checked_path_count": 0,
            "affected_document_count": 1,
            "violation_count": 1,
            "reason_counts": {"missing docs root": 1},
            "path_root_counts": {"docs": 1},
            "violations": [
                {
                    "document": "docs",
                    "field": "docs_root",
                    "path": "docs",
                    "reason": "missing docs root",
                }
            ],
        }

    for document in sorted(docs_root.rglob("*.md")):
        fields = _front_matter_path_fields(document)
        if fields is None:
            continue
        document_count += 1
        for field in DOCUMENT_PATH_FIELDS:
            for declared_path in fields.get(field, []):
                checked_path_count += 1
                if declared_path not in path_reason_cache:
                    path_reason_cache[declared_path] = _path_violation_reason(
                        repo_root, declared_path
                    )
                reason = path_reason_cache[declared_path]
                if reason is not None:
                    violations.append(
                        {
                            "document": document.relative_to(repo_root).as_posix(),
                            "field": field,
                            "path": declared_path,
                            "reason": reason,
                        }
                    )

    violations.sort(
        key=lambda item: (item["document"], item["field"], item["path"], item["reason"])
    )
    reason_counts = Counter(str(item["reason"]) for item in violations)
    path_root_counts = Counter(
        str(item["path"]).replace("\\", "/").split("/", maxsplit=1)[0]
        for item in violations
    )
    return {
        "schema_version": 1,
        "document_count": document_count,
        "checked_path_count": checked_path_count,
        "affected_document_count": len({item["document"] for item in violations}),
        "violation_count": len(violations),
        "reason_counts": dict(sorted(reason_counts.items())),
        "path_root_counts": dict(
            sorted(path_root_counts.items(), key=lambda item: (-item[1], item[0]))
        ),
        "violations": violations,
    }


def _front_matter_path_fields(document: Path) -> dict[str, list[str]] | None:
    lines = document.read_text(encoding="utf-8-sig").splitlines()
    if not lines or lines[0].strip() != "---":
        return None
    try:
        end = next(index for index, line in enumerate(lines[1:], start=1) if line.strip() == "---")
    except StopIteration:
        return None

    fields = {field: [] for field in DOCUMENT_PATH_FIELDS}
    active_field: str | None = None
    for line in lines[1:end]:
        if line and not line[0].isspace():
            key, separator, _ = line.partition(":")
            active_field = key if separator and key in fields else None
            continue
        stripped = line.strip()
        if active_field is None or not stripped.startswith("- "):
            continue
        value = stripped[2:].strip().strip("'\"").strip("`")
        if value:
            fields[active_field].append(value)
    return fields


def _path_violation_reason(repo_root: Path, declared_path: str) -> str | None:
    windows_path = PureWindowsPath(declared_path)
    if windows_path.is_absolute() or windows_path.drive:
        return "absolute path"
    candidate = (repo_root / Path(declared_path.replace("\\", "/"))).resolve()
    try:
        candidate.relative_to(repo_root)
    except ValueError:
        return "repository escape"
    if not candidate.exists():
        return "missing path"
    return None


def run_conventions(
    repo_root: Path,
    selected: Iterable[str],
    *,
    dry_run: bool,
) -> dict[str, object]:
    selected_set = set(selected)
    report: dict[str, object] = {
        "schema_version": 1,
        "repo_root": str(repo_root.resolve()),
        "docs": None,
        "commands": [],
        "passed": True,
    }
    if "docs" in selected_set:
        docs_report = audit_document_paths(repo_root)
        report["docs"] = docs_report
        if docs_report["violation_count"]:
            report["passed"] = False

    command_reports: list[dict[str, object]] = []
    for command in convention_commands():
        if command.name not in selected_set:
            continue
        command_report: dict[str, object] = {
            **asdict(command),
            "exit_code": None,
            "executed": not dry_run,
        }
        if not dry_run:
            completed = subprocess.run(command.argv, cwd=repo_root, check=False)
            command_report["exit_code"] = completed.returncode
            if completed.returncode != 0:
                report["passed"] = False
        command_reports.append(command_report)
    report["commands"] = command_reports
    return report


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Run Zircon documentation, formatting, and scoped lint convention gates."
    )
    parser.add_argument(
        "--repo-root",
        type=Path,
        default=Path(__file__).resolve().parents[1],
        help="Repository root override.",
    )
    parser.add_argument(
        "--only",
        action="append",
        choices=("docs", "fmt", "clippy"),
        help="Run only the selected gate; repeat for multiple gates.",
    )
    parser.add_argument("--dry-run", action="store_true", help="Print command plans without running Cargo.")
    parser.add_argument("--json", action="store_true", help="Emit the complete machine-readable report.")
    args = parser.parse_args()

    selected = args.only or ["docs", "fmt", "clippy"]
    report = run_conventions(args.repo_root.resolve(), selected, dry_run=args.dry_run)
    if args.json:
        print(json.dumps(report, ensure_ascii=False, indent=2, sort_keys=True))
    else:
        docs_report = report.get("docs")
        if isinstance(docs_report, dict):
            print(
                "docs: "
                f"{docs_report['checked_path_count']} paths in "
                f"{docs_report['document_count']} documents, "
                f"{docs_report['violation_count']} violations"
            )
            for violation in docs_report["violations"]:
                print(
                    f"  {violation['document']}: {violation['field']} "
                    f"`{violation['path']}` ({violation['reason']})"
                )
        for command in report["commands"]:
            rendered = " ".join(command["argv"])
            status = "planned" if not command["executed"] else f"exit {command['exit_code']}"
            print(f"{command['name']}: {status}: {rendered}")
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
