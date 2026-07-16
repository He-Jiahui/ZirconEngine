from __future__ import annotations

import re
from pathlib import Path


MARKDOWN_LINK_TARGET = re.compile(r"\]\(\s*<?([^)>\s]+)>?(?:\s+[^)]*)?\)")


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def file_line_count(path: Path) -> int:
    return len(read_text(path).splitlines())


def file_entries(root: Path, files: tuple[str, ...]) -> tuple[list[dict[str, object]], list[str]]:
    entries: list[dict[str, object]] = []
    missing: list[str] = []
    for file_name in files:
        path = root / file_name
        if not path.exists():
            missing.append(file_name)
            continue
        entries.append({"path": file_name, "lines": file_line_count(path)})
    return entries, missing


def missing_snippets(sources: tuple[str, ...], snippets: tuple[str, ...]) -> list[str]:
    return [
        snippet
        for snippet in snippets
        if not any(snippet in source for source in sources)
    ]


def frontmatter_value(source: str, prefix: str) -> str | None:
    lines = source.splitlines()
    if not lines or lines[0] != "---":
        return None
    for line in lines[1:]:
        if line == "---":
            break
        if line.startswith(prefix):
            return line.removeprefix(prefix).strip()
    return None


def max_iso_date(source: str) -> str | None:
    dates = re.findall(r"\d{4}-\d{2}-\d{2}", source)
    return max(dates) if dates else None


def markdown_table_cells(row: str) -> list[str]:
    return [cell.strip() for cell in row.strip().strip("|").split("|")]


def markdown_repo_link_targets(root: Path, source_path: Path, source: str) -> set[str]:
    resolved_root = root.resolve()
    targets: set[str] = set()
    for match in MARKDOWN_LINK_TARGET.finditer(source):
        raw_target = match.group(1).split("#", 1)[0].split("?", 1)[0]
        if (
            not raw_target
            or raw_target.startswith(("/", "\\"))
            or re.match(r"^[A-Za-z][A-Za-z0-9+.-]*:", raw_target)
        ):
            continue
        resolved_target = (source_path.parent / raw_target).resolve(strict=False)
        try:
            targets.add(resolved_target.relative_to(resolved_root).as_posix())
        except ValueError:
            continue
    return targets


def runtime_subplans(root: Path) -> list[tuple[str, str]]:
    plan_dir = root / "docs/plans/zircon_runtime/runtime"
    subplans: list[tuple[str, str]] = []
    for path in sorted(plan_dir.glob("[0-9][0-9]-*.md")):
        subplans.append((path.name, read_text(path)))
    return subplans


def runtime_numbered_archives(root: Path) -> dict[str, list[tuple[str, str]]]:
    active_plan_dir = root / "docs/plans/zircon_runtime/runtime"
    canonical_archive_dir = root / "docs/plans/_archive/zircon_runtime/runtime"
    archives: dict[str, list[tuple[str, str]]] = {}
    for number in (f"{value:02d}" for value in range(1, 16)):
        entries: list[tuple[str, str]] = []
        for source_dir in (
            active_plan_dir / number,
            canonical_archive_dir / number,
        ):
            if not source_dir.exists():
                continue
            for path in sorted(source_dir.glob("*.md")):
                entries.append((path.relative_to(root).as_posix(), read_text(path)))
        archives[number] = entries
    return archives


def runtime_index_problem_rows(index_source: str) -> list[str]:
    rows: list[str] = []
    for line in index_source.splitlines():
        cells = markdown_table_cells(line)
        if (
            len(cells) == 4
            and cells[0].startswith("P")
            and cells[0][1:].isdigit()
        ):
            rows.append(line)
    return rows


def runtime_index_subplan_rows(index_source: str) -> list[str]:
    rows: list[str] = []
    for line in index_source.splitlines():
        cells = markdown_table_cells(line)
        if (
            len(cells) == 4
            and cells[0][:2].isdigit()
            and "`" in cells[1]
            and cells[1].endswith(".md`")
        ):
            rows.append(line)
    return rows


def runtime_index_backlog_rows(index_source: str) -> list[str]:
    start = index_source.find("### 3.1 已知但暂不立项的缺口")
    if start == -1:
        return []
    section = index_source[start:]
    end = section.find("阶段划分:")
    if end != -1:
        section = section[:end]
    rows: list[str] = []
    for line in section.splitlines():
        cells = markdown_table_cells(line)
        if len(cells) == 3 and cells[0] not in {"缺口", "---"} and cells[0]:
            if not all(character == "-" for character in cells[0]):
                rows.append(line)
    return rows


def status_rows(source: str) -> list[str]:
    start = source.find("## 状态与产出记录")
    if start == -1:
        return []
    section = source[start:]
    next_heading = section.find("\n## ", len("## 状态与产出记录"))
    if next_heading != -1:
        section = section[:next_heading]
    rows: list[str] = []
    for line in section.splitlines():
        cells = markdown_table_cells(line)
        if (
            len(cells) == 5
            and cells[0] not in {"里程碑", "---"}
            and cells[0]
            and not all(character == "-" for character in cells[0])
        ):
            rows.append(line)
    return rows


def archive_status_rows(source: str) -> list[str]:
    rows: list[str] = []
    for line in source.splitlines():
        cells = markdown_table_cells(line)
        if (
            len(cells) == 5
            and cells[0] not in {"里程碑", "日期", "---"}
            and cells[0]
            and not all(character == "-" for character in cells[0])
        ):
            rows.append(line)
    return rows
