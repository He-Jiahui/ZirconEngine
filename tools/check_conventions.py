from __future__ import annotations

import argparse
from collections import Counter
import json
import re
import stat
import subprocess
from dataclasses import asdict, dataclass
from pathlib import Path, PurePosixPath, PureWindowsPath
from typing import Iterable

if __package__:
    from .convention_exemptions import (
        audit_rust_exemptions as audit_rust_exemptions_from_catalog,
    )
else:  # pragma: no cover - direct script import path.
    from convention_exemptions import (
        audit_rust_exemptions as audit_rust_exemptions_from_catalog,
    )


DOCUMENT_PATH_FIELDS = ("implementation_files", "related_code")
FRONT_MATTER_LIST_FIELDS = (*DOCUMENT_PATH_FIELDS, "tests")
CONVENTION_RULES_PATH = Path(
    "docs/plans/zircon_runtime/frameworks/development-conventions.md"
)
# The convention document owns these rule families. A new family must update
# the runner contract in the same change instead of silently widening what an
# unrelated Markdown table can look like.
RULE_ID_PATTERN = r"(?:GEN-[A-Z][0-9]+|(?:RT|ED|PL|IF|WF)-[0-9]+)"
RULE_ROW_PATTERN = re.compile(
    rf"^\|\s*({RULE_ID_PATTERN})\s*\|\s*(MUST|SHOULD)\s*\|\s*(.*?)\s*\|\s*(.*?)\s*\|\s*$"
)
RULE_ROW_CANDIDATE_PATTERN = re.compile(rf"^\|\s*{RULE_ID_PATTERN}\s*\|")
RULE_TABLE_HEADER = "| ID | 级别 | 规则 | 守卫 |"
RULE_TABLE_SEPARATOR_PATTERN = re.compile(
    r"^\|\s*-+\s*\|\s*-+\s*\|\s*-+\s*\|\s*-+\s*\|$"
)
EMPTY_GUARD_VALUES = {"", "-", "—", "none", "tbd", "无", "无守卫", "待定"}
VALID_GUARD_VALUES = frozenset(
    {
        "CI",
        "G1",
        "G1 + 评审",
        "G2",
        "G3",
        "G3(clippy 配置)",
        "G4",
        "G5",
        "G5 + 契约测试",
        "G5 + 审计",
        "G6",
        "G7",
        "内核 `CoreError` + 单测",
        "契约测试",
        "审计",
        "流程",
        "评审",
        "评审 + perf 计数断言",
        "评审 + 契约测试",
        "评审→G2",
        "评审→G3",
    }
)


@dataclass(frozen=True)
class ConventionCommand:
    name: str
    argv: tuple[str, ...]


def convention_commands() -> list[ConventionCommand]:
    return [
        ConventionCommand(
            "layering",
            (
                "python",
                "-m",
                "unittest",
                "tools.tests.test_frameworks_05_layer_direction",
                "-v",
            ),
        ),
        ConventionCommand(
            "structure",
            (
                "cargo",
                "+1.94.1",
                "test",
                "-p",
                "zircon_runtime",
                "--lib",
                "structure_convention",
                "--locked",
                "--jobs",
                "1",
            ),
        ),
        ConventionCommand("fmt", ("cargo", "+1.94.1", "fmt", "--all", "--check")),
        ConventionCommand(
            "clippy",
            (
                "cargo",
                "+1.94.1",
                "clippy",
                "-p",
                "zircon_runtime_interface",
                "-p",
                "zircon_app",
                "--all-targets",
                "--no-deps",
                "--locked",
                "--jobs",
                "1",
                "--",
                "-D",
                "warnings",
            ),
        ),
    ]


def audit_rule_guard_coverage(repo_root: Path) -> dict[str, object]:
    repo_root = repo_root.resolve()
    document = repo_root / CONVENTION_RULES_PATH
    if not document.is_file():
        return {
            "schema_version": 1,
            "document": CONVENTION_RULES_PATH.as_posix(),
            "rule_count": 0,
            "must_rule_count": 0,
            "rule_ids": [],
            "must_rule_ids": [],
            "guard_counts": {},
            "violation_count": 1,
            "violations": [
                {
                    "rule_id": "<document>",
                    "line": 0,
                    "reason": "missing convention rules document",
                }
            ],
        }

    seen_rule_ids: set[str] = set()
    rule_ids: list[str] = []
    must_rule_ids: list[str] = []
    rule_count = 0
    must_rule_count = 0
    guard_counts: Counter[str] = Counter()
    violations: list[dict[str, object]] = []
    in_rule_table = False
    expects_rule_table_separator = False
    rule_table_header_line = 0
    for line_number, line in enumerate(
        document.read_text(encoding="utf-8-sig").splitlines(), start=1
    ):
        stripped = line.strip()
        if in_rule_table and expects_rule_table_separator:
            if RULE_TABLE_SEPARATOR_PATTERN.match(stripped):
                expects_rule_table_separator = False
                continue
            violations.append(
                {
                    "rule_id": "<malformed>",
                    "line": line_number,
                    "reason": "missing rule table separator",
                }
            )
            in_rule_table = False
            expects_rule_table_separator = False
        if stripped == RULE_TABLE_HEADER:
            if in_rule_table:
                violations.append(
                    {
                        "rule_id": "<malformed>",
                        "line": line_number,
                        "reason": "unexpected rule table header",
                    }
                )
                continue
            in_rule_table = True
            expects_rule_table_separator = True
            rule_table_header_line = line_number
            continue
        if not in_rule_table:
            if RULE_ROW_CANDIDATE_PATTERN.match(stripped):
                violations.append(
                    {
                        "rule_id": "<malformed>",
                        "line": line_number,
                        "reason": "rule row outside recognized table",
                    }
                )
            continue
        if not stripped.startswith("|"):
            in_rule_table = False
            continue
        if RULE_TABLE_SEPARATOR_PATTERN.match(stripped):
            violations.append(
                {
                    "rule_id": "<malformed>",
                    "line": line_number,
                    "reason": "unexpected rule table separator",
                }
            )
            continue

        match = RULE_ROW_PATTERN.match(stripped)
        if match is None:
            violations.append(
                {
                    "rule_id": "<malformed>",
                    "line": line_number,
                    "reason": "malformed rule row",
                }
            )
            continue
        rule_id, level, rule_text, guard = match.groups()
        rule_count += 1
        rule_ids.append(rule_id)
        if rule_id in seen_rule_ids:
            violations.append(
                {
                    "rule_id": rule_id,
                    "line": line_number,
                    "reason": "duplicate rule id",
                }
            )
        else:
            seen_rule_ids.add(rule_id)

        if not rule_text.strip():
            violations.append(
                {
                    "rule_id": rule_id,
                    "line": line_number,
                    "reason": "empty rule text",
                }
            )

        normalized_guard = guard.strip().strip("`*_ ")
        guard_is_empty = normalized_guard.casefold() in EMPTY_GUARD_VALUES
        if level == "MUST":
            must_rule_count += 1
            must_rule_ids.append(rule_id)
        if level == "MUST" and guard_is_empty:
            violations.append(
                {
                    "rule_id": rule_id,
                    "line": line_number,
                    "reason": "missing guard",
                }
            )
        elif not guard_is_empty and normalized_guard not in VALID_GUARD_VALUES:
            violations.append(
                {
                    "rule_id": rule_id,
                    "line": line_number,
                    "reason": "unknown guard",
                }
            )
        elif level == "MUST":
            guard_counts[normalized_guard] += 1

    if expects_rule_table_separator:
        violations.append(
            {
                "rule_id": "<malformed>",
                "line": rule_table_header_line,
                "reason": "missing rule table separator",
            }
        )

    if rule_count == 0:
        violations.append(
            {
                "rule_id": "<document>",
                "line": 0,
                "reason": "no convention rule rows",
            }
        )

    violations.sort(key=lambda item: (int(item["line"]), str(item["reason"])))
    return {
        "schema_version": 1,
        "document": CONVENTION_RULES_PATH.as_posix(),
        "rule_count": rule_count,
        "must_rule_count": must_rule_count,
        "rule_ids": rule_ids,
        "must_rule_ids": must_rule_ids,
        "guard_counts": dict(sorted(guard_counts.items())),
        "violation_count": len(violations),
        "violations": violations,
    }


def audit_document_paths(repo_root: Path) -> dict[str, object]:
    validator = _RepositoryPathValidator(repo_root)
    repo_root = validator.repo_root
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
            "resolution_metrics": validator.resolution_metrics(),
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
        for field in FRONT_MATTER_LIST_FIELDS:
            declared_paths = fields.get(field, [])
            if field == "tests":
                declared_paths = [
                    path
                    for entry in declared_paths
                    for path in _test_reference_paths(entry)
                ]
            for declared_path in declared_paths:
                checked_path_count += 1
                if declared_path not in path_reason_cache:
                    path_reason_cache[declared_path] = validator.violation_reason(
                        declared_path
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
        "resolution_metrics": validator.resolution_metrics(),
        "violations": violations,
    }


def audit_rust_exemptions(repo_root: Path) -> dict[str, object]:
    rule_report = audit_rule_guard_coverage(repo_root)
    return audit_rust_exemptions_from_catalog(
        repo_root,
        rule_report=rule_report,
        convention_rules_path=CONVENTION_RULES_PATH,
        rule_id_pattern=RULE_ID_PATTERN,
    )


def _front_matter_path_fields(document: Path) -> dict[str, list[str]] | None:
    lines = document.read_text(encoding="utf-8-sig").splitlines()
    if not lines or lines[0].strip() != "---":
        return None
    try:
        end = next(index for index, line in enumerate(lines[1:], start=1) if line.strip() == "---")
    except StopIteration:
        return None

    fields = {field: [] for field in FRONT_MATTER_LIST_FIELDS}
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


def _test_reference_paths(entry: str) -> list[str]:
    if not entry:
        return []
    path = entry.split("::", maxsplit=1)[0].rstrip(":")
    path = re.sub(r":\d+(?::\d+)?$", "", path).replace("\\", "/")
    if any(marker in path for marker in ("*", "?", "[", "]", "{", "}", "<", ">")):
        return []
    repository_path = path[2:] if path.startswith("./") else path
    if re.search(r"(?:^|/)[^/]+\.[A-Za-z0-9]+$", path) is None:
        return []
    windows_path = PureWindowsPath(path)
    if windows_path.is_absolute() or windows_path.drive:
        return [repository_path]
    if re.match(r"^[A-Za-z][A-Za-z0-9+.-]*://", path):
        return []
    posix_path = PurePosixPath(path)
    if posix_path.is_absolute() or ".." in posix_path.parts:
        return [repository_path]
    if any(character.isspace() for character in entry):
        return []
    if any(part in {"build", "target"} for part in PurePosixPath(repository_path).parts):
        return []
    return [repository_path]


class _RepositoryPathValidator:
    def __init__(self, repo_root: Path) -> None:
        self.repo_root = repo_root.resolve()
        self._resolved_parents: dict[Path, Path] = {}
        self._unique_path_count = 0
        self._parent_resolution_count = 0
        self._reparse_leaf_resolution_count = 0
        self._relative_segment_resolution_count = 0

    def violation_reason(self, declared_path: str) -> str | None:
        self._unique_path_count += 1
        windows_path = PureWindowsPath(declared_path)
        normalized_path = declared_path.replace("\\", "/")
        posix_path = PurePosixPath(normalized_path)
        if windows_path.is_absolute() or windows_path.drive or posix_path.is_absolute():
            return "absolute path"

        candidate = self.repo_root / Path(normalized_path)
        exists = candidate.exists()
        if ".." in posix_path.parts:
            resolved_candidate = candidate.resolve()
            self._relative_segment_resolution_count += 1
        elif self._is_reparse_leaf(candidate):
            resolved_candidate = candidate.resolve()
            self._reparse_leaf_resolution_count += 1
        else:
            lexical_parent = candidate.parent
            resolved_parent = self._resolved_parents.get(lexical_parent)
            if resolved_parent is None:
                resolved_parent = lexical_parent.resolve()
                self._resolved_parents[lexical_parent] = resolved_parent
                self._parent_resolution_count += 1
            resolved_candidate = resolved_parent / candidate.name

        try:
            resolved_candidate.relative_to(self.repo_root)
        except ValueError:
            return "repository escape"
        if not exists:
            return "missing path"
        return None

    def resolution_metrics(self) -> dict[str, int]:
        return {
            "unique_path_count": self._unique_path_count,
            "full_resolution_count": (
                self._parent_resolution_count
                + self._reparse_leaf_resolution_count
                + self._relative_segment_resolution_count
            ),
            "parent_resolution_count": self._parent_resolution_count,
            "reparse_leaf_resolution_count": self._reparse_leaf_resolution_count,
            "relative_segment_resolution_count": (
                self._relative_segment_resolution_count
            ),
        }

    @staticmethod
    def _is_reparse_leaf(candidate: Path) -> bool:
        try:
            metadata = candidate.lstat()
        except OSError:
            return False
        return stat.S_ISLNK(metadata.st_mode) or bool(
            getattr(metadata, "st_file_attributes", 0)
            & stat.FILE_ATTRIBUTE_REPARSE_POINT
        )


def run_conventions(
    repo_root: Path,
    selected: Iterable[str],
    *,
    dry_run: bool,
    capture_output: bool = False,
) -> dict[str, object]:
    selected_set = set(selected)
    report: dict[str, object] = {
        "schema_version": 1,
        "repo_root": str(repo_root.resolve()),
        "docs": None,
        "exemptions": None,
        "guards": None,
        "commands": [],
        "passed": True,
    }
    if "docs" in selected_set:
        docs_report = audit_document_paths(repo_root)
        report["docs"] = docs_report
        if docs_report["violation_count"]:
            report["passed"] = False
    if "guards" in selected_set:
        guards_report = audit_rule_guard_coverage(repo_root)
        report["guards"] = guards_report
        if guards_report["violation_count"]:
            report["passed"] = False
    if "exemptions" in selected_set:
        exemptions_report = audit_rust_exemptions(repo_root)
        report["exemptions"] = exemptions_report
        if exemptions_report["violation_count"]:
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
            try:
                completed = subprocess.run(
                    command.argv,
                    cwd=repo_root,
                    check=False,
                    capture_output=capture_output,
                    text=capture_output,
                    encoding="utf-8" if capture_output else None,
                    errors="replace" if capture_output else None,
                )
            except OSError as error:
                command_report["launch_error"] = {
                    "kind": type(error).__name__,
                    "message": str(error),
                }
                if capture_output:
                    command_report["stdout"] = ""
                    command_report["stderr"] = str(error)
                report["passed"] = False
            else:
                command_report["exit_code"] = completed.returncode
                if capture_output:
                    command_report["stdout"] = completed.stdout
                    command_report["stderr"] = completed.stderr
                if completed.returncode != 0:
                    report["passed"] = False
        command_reports.append(command_report)
    report["commands"] = command_reports
    return report


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Run Zircon documentation, exemption, formatting, and scoped lint gates."
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
        choices=(
            "docs",
            "guards",
            "exemptions",
            "layering",
            "structure",
            "fmt",
            "clippy",
        ),
        help="Run only the selected gate; repeat for multiple gates.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print command plans without running subprocess gates.",
    )
    parser.add_argument("--json", action="store_true", help="Emit the complete machine-readable report.")
    args = parser.parse_args()

    selected = args.only or [
        "docs",
        "guards",
        "exemptions",
        "layering",
        "structure",
        "fmt",
        "clippy",
    ]
    report = run_conventions(
        args.repo_root.resolve(),
        selected,
        dry_run=args.dry_run,
        capture_output=args.json,
    )
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
        guards_report = report.get("guards")
        if isinstance(guards_report, dict):
            print(
                "guards: "
                f"{guards_report['must_rule_count']} MUST / "
                f"{guards_report['rule_count']} rules, "
                f"{guards_report['violation_count']} violations"
            )
            for violation in guards_report["violations"]:
                print(
                    f"  {guards_report['document']}:{violation['line']} "
                    f"{violation['rule_id']} ({violation['reason']})"
                )
        exemptions_report = report.get("exemptions")
        if isinstance(exemptions_report, dict):
            print(
                "exemptions: "
                f"{exemptions_report['valid_exemption_count']} valid / "
                f"{exemptions_report['allow_attribute_count']} allow attributes, "
                f"{exemptions_report['violation_count']} violations"
            )
            for violation in exemptions_report["violations"]:
                print(
                    f"  {violation['path']}:{violation['line']} "
                    f"{violation['member']} ({violation['reason']})"
                )
        for command in report["commands"]:
            rendered = " ".join(command["argv"])
            status = "planned" if not command["executed"] else f"exit {command['exit_code']}"
            print(f"{command['name']}: {status}: {rendered}")
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
