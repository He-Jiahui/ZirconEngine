#!/usr/bin/env python3
"""Read-only audit for ZirconEngine plan output-record placement."""

from __future__ import annotations

import argparse
import re
import tempfile
from dataclasses import dataclass
from pathlib import Path


NOTICE = "请将产出记录放置在子计划中，此处仅展示当前现状的概述"
H2_RE = re.compile(r"^##\s+.+$", re.MULTILINE)
OUTPUT_HEADING_RE = re.compile(r"产出记录|完成阶段记录")
CHILD_PLAN_RE = re.compile(r"^(?P<number>\d{2})-.+\.md$")
ENGINE_CODE_RE = re.compile(r"^engine-code-.+\.md$")
ARCHIVE_LINK_RE = re.compile(
    r"\[[^\]]+\]\((?P<target>\d{2}/\d{4}-\d{2}-\d{2}-[^)#]+\.md)(?:#[^)]+)?\)"
)
DATE_RE = re.compile(r"\b20\d{2}-\d{2}-\d{2}\b")
CONCRETE_SIGNATURE_RE = re.compile(
    r"_static_(?:passed|failed)|_cargo_(?:passed|failed|deferred)|"
    r"Cargo gate deferred|focused Cargo|SHA256|target/cargo-target|"
    r"\bpassed\s+\d+/\d+|\d+/\d+\s+passed|\.log`|验证日志|验证图",
    re.IGNORECASE,
)
MIGRATION_LINE_RE = re.compile(r"迁入记录|迁入产出记录|具体记录已迁入")


@dataclass(frozen=True)
class Violation:
    path: Path
    line: int
    code: str
    message: str


@dataclass(frozen=True)
class Section:
    heading: str
    start_line: int
    body: str


def output_sections(text: str) -> list[Section]:
    headings = list(H2_RE.finditer(text))
    sections: list[Section] = []
    for index, match in enumerate(headings):
        heading = match.group(0)
        if not OUTPUT_HEADING_RE.search(heading):
            continue
        end = headings[index + 1].start() if index + 1 < len(headings) else len(text)
        start_line = text.count("\n", 0, match.start()) + 1
        sections.append(Section(heading, start_line, text[match.start() : end]))
    return sections


def is_separator_row(line: str) -> bool:
    cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
    return bool(cells) and all(re.fullmatch(r":?-{3,}:?", cell or "") for cell in cells)


def count_table_records(section: str, forbidden_target: bool) -> int:
    lines = section.splitlines()
    total = 0
    index = 0
    while index + 1 < len(lines):
        header = lines[index].strip()
        separator = lines[index + 1].strip()
        if not (
            header.startswith("|")
            and separator.startswith("|")
            and is_separator_row(separator)
        ):
            index += 1
            continue
        header_is_concrete = any(
            key in header for key in ("完成日期", "证据", "验证", "状态锚", "切片")
        )
        index += 2
        rows = 0
        row_has_signature = False
        while index < len(lines) and lines[index].strip().startswith("|"):
            row = lines[index]
            if not is_separator_row(row):
                rows += 1
                row_has_signature = row_has_signature or bool(
                    DATE_RE.search(row) or CONCRETE_SIGNATURE_RE.search(row)
                )
            index += 1
        if not forbidden_target or header_is_concrete or row_has_signature:
            total += rows
    return total


def count_list_records(section: str) -> int:
    total = 0
    for line in section.splitlines():
        stripped = line.strip()
        if not re.match(r"(?:[-*]|\d+[.)])\s+", stripped):
            continue
        if MIGRATION_LINE_RE.search(stripped):
            continue
        if DATE_RE.search(stripped) or CONCRETE_SIGNATURE_RE.search(stripped):
            total += 1
    return total


def record_count(section: str, forbidden_target: bool) -> int:
    return count_table_records(section, forbidden_target) + count_list_records(section)


def audit_repo(repo_root: Path) -> list[Violation]:
    plans_root = repo_root / "docs" / "plans"
    if not plans_root.is_dir():
        return [Violation(plans_root, 1, "plans-root-missing", "docs/plans does not exist")]

    violations: list[Violation] = []
    for path in sorted(plans_root.rglob("*.md")):
        text = path.read_text(encoding="utf-8")
        sections = output_sections(text)
        forbidden_target = path.name == "index.md" or bool(
            ENGINE_CODE_RE.fullmatch(path.name)
        )
        child_match = CHILD_PLAN_RE.fullmatch(path.name)

        if forbidden_target:
            for section in sections:
                if NOTICE not in section.body:
                    violations.append(
                        Violation(
                            path,
                            section.start_line,
                            "missing-notice",
                            f"output-record position must contain the exact notice: {NOTICE}",
                        )
                    )
                count = record_count(section.body, forbidden_target=True)
                if count:
                    violations.append(
                        Violation(
                            path,
                            section.start_line,
                            "forbidden-concrete-records",
                            f"found {count} concrete record(s) in a forbidden overview file",
                        )
                    )
            if MIGRATION_LINE_RE.search(text) and NOTICE not in text and not sections:
                violations.append(
                    Violation(
                        path,
                        1,
                        "missing-notice",
                        "migration links exist without the exact notice",
                    )
                )
            for line_number, line in enumerate(text.splitlines(), 1):
                if NOTICE in line or MIGRATION_LINE_RE.search(line):
                    continue
                if CONCRETE_SIGNATURE_RE.search(line):
                    violations.append(
                        Violation(
                            path,
                            line_number,
                            "forbidden-concrete-signature",
                            "concrete status or validation evidence appears in a forbidden overview file",
                        )
                    )

        if child_match:
            child_number = child_match.group("number")
            for section in sections:
                count = record_count(section.body, forbidden_target=False)
                if count > 10:
                    violations.append(
                        Violation(
                            path,
                            section.start_line,
                            "child-record-limit",
                            f"child plan contains {count} direct records; move all records to {child_number}/",
                        )
                    )
                archive_links = list(ARCHIVE_LINK_RE.finditer(section.body))
                if archive_links and NOTICE not in section.body:
                    violations.append(
                        Violation(
                            path,
                            section.start_line,
                            "missing-archive-notice",
                            "numbered archive links require the exact notice in the child plan",
                        )
                    )
                for link in archive_links:
                    target = link.group("target")
                    target_number = target.split("/", 1)[0]
                    link_line = section.start_line + section.body.count(
                        "\n", 0, link.start()
                    )
                    if target_number != child_number:
                        violations.append(
                            Violation(
                                path,
                                link_line,
                                "archive-number-mismatch",
                                f"archive link {target} does not match child prefix {child_number}",
                            )
                        )
                    if not (path.parent / target).is_file():
                        violations.append(
                            Violation(
                                path,
                                link_line,
                                "archive-link-missing",
                                f"archive link does not exist: {target}",
                            )
                        )

    return violations


def run_self_test() -> None:
    with tempfile.TemporaryDirectory(prefix="write-plan-output-records-") as temp_dir:
        root = Path(temp_dir)
        plans = root / "docs" / "plans" / "runtime"
        plans.mkdir(parents=True)
        (plans / "index.md").write_text(
            "# Runtime\n\n## 状态与产出记录\n\n"
            f"> {NOTICE}\n\n当前仅展示概述。\n",
            encoding="utf-8",
        )
        child = plans / "01-runtime.md"
        child.write_text(
            "# Runtime 01\n\n## 状态与产出记录\n\n"
            "| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |\n"
            "|---|---|---|---|---|\n"
            "| M1 | a | `passed` | 2026-07-10 | check |\n"
            "| M1 | b | `passed` | 2026-07-10 | test |\n",
            encoding="utf-8",
        )
        assert not audit_repo(root), "valid fixture should pass"

        rows = "".join(
            f"| M1 | slice-{index} | `passed` | 2026-07-10 | test |\n"
            for index in range(11)
        )
        child.write_text(
            "# Runtime 01\n\n## 状态与产出记录\n\n"
            "| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |\n"
            "|---|---|---|---|---|\n"
            + rows,
            encoding="utf-8",
        )
        assert any(item.code == "child-record-limit" for item in audit_repo(root))

        archive = plans / "01" / "2026-07-10-runtime-validation.md"
        archive.parent.mkdir()
        archive.write_text("# Runtime validation\n", encoding="utf-8")
        child.write_text(
            "# Runtime 01\n\n## 状态与产出记录\n\n"
            "- [迁入记录](01/2026-07-10-runtime-validation.md)\n",
            encoding="utf-8",
        )
        assert any(item.code == "missing-archive-notice" for item in audit_repo(root))

        (plans / "index.md").write_text(
            "# Runtime\n\n## 状态与产出记录\n\n"
            f"> {NOTICE}\n\n"
            "| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |\n"
            "|---|---|---|---|---|\n"
            "| M1 | forbidden | `x_static_passed` | 2026-07-10 | test |\n",
            encoding="utf-8",
        )
        codes = {item.code for item in audit_repo(root)}
        assert "forbidden-concrete-records" in codes
        assert "forbidden-concrete-signature" in codes


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()

    if args.self_test:
        run_self_test()
        print("self-test passed")
        return 0

    violations = audit_repo(args.repo_root.resolve())
    for item in violations:
        print(f"{item.path}:{item.line}: {item.code}: {item.message}")
    if violations:
        print(f"audit failed: {len(violations)} violation(s)")
        return 1
    print("audit passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
