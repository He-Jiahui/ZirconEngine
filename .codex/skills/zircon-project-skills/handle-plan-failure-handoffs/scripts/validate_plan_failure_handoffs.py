from __future__ import annotations

import argparse
import re
import sys
from datetime import date
from pathlib import Path, PurePosixPath


CANONICAL_NAME = re.compile(
    r"^(failure|fixed)-(\d{4}-\d{2}-\d{2})-([a-z0-9]+(?:-[a-z0-9]+)*)\.md$"
)
DATE_FIRST_HANDOFF = re.compile(
    r"^\d{4}-\d{2}-\d{2}-.+(?:failure|fixed).*(?:handoff)?\.md$",
    re.IGNORECASE,
)
PLAN_NAME = re.compile(r"^(\d+)-.+\.md$")
SLUG = re.compile(r"^[a-z0-9]+(?:-[a-z0-9]+)*$")
MARKDOWN_LINK = re.compile(r"\[[^\]]*\]\(([^)]+)\)")
WINDOWS_ABSOLUTE = re.compile(r"^[A-Za-z]:[\\/]")
REQUIRED_HEADINGS = (
    "## 来源执行者",
    "## 失败现象与复现证据",
    "## 最低共享层根因",
    "## 架构修复验收",
    "## 禁止临时方案",
    "## 修复结果与回传",
)
BASE_KEYS = (
    "handoff_kind",
    "status",
    "created_at",
    "summary_slug",
    "origin_plan",
    "fixing_plan",
    "origin_child_dir",
    "fixing_child_dir",
)


def _relative(path: Path, root: Path) -> str:
    try:
        return path.relative_to(root).as_posix()
    except ValueError:
        return path.as_posix()


def _parse_frontmatter(path: Path, content: str) -> tuple[dict[str, str], list[str]]:
    errors: list[str] = []
    lines = content.splitlines()
    if not lines or lines[0].strip() != "---":
        return {}, [f"{path}: missing YAML frontmatter"]
    try:
        end = next(index for index, line in enumerate(lines[1:], start=1) if line.strip() == "---")
    except StopIteration:
        return {}, [f"{path}: unterminated YAML frontmatter"]

    metadata: dict[str, str] = {}
    for line in lines[1:end]:
        if not line or line[0].isspace() or ":" not in line:
            continue
        key, value = line.split(":", 1)
        metadata[key.strip()] = value.strip().strip('"\'')
    return metadata, errors


def _repo_path(value: str, *, field: str, artifact: Path, root: Path, errors: list[str]) -> Path | None:
    if not value:
        return None
    normalized = value.replace("\\", "/")
    pure = PurePosixPath(normalized)
    if pure.is_absolute() or WINDOWS_ABSOLUTE.match(normalized) or ".." in pure.parts:
        errors.append(f"{_relative(artifact, root)}: {field} must be a repo-relative path")
        return None
    return root.joinpath(*pure.parts)


def _derived_child_dir(
    plan: Path | None,
    *,
    field: str,
    artifact: Path,
    root: Path,
    errors: list[str],
) -> Path | None:
    if plan is None:
        return None
    if not plan.is_file():
        errors.append(f"{_relative(artifact, root)}: {field} does not exist: {_relative(plan, root)}")
        return None
    match = PLAN_NAME.match(plan.name)
    if not match:
        errors.append(f"{_relative(artifact, root)}: {field} must name a numbered child plan")
        return None
    return plan.parent / match.group(1)


def _parse_date(value: str, *, field: str, artifact: Path, root: Path, errors: list[str]) -> None:
    try:
        date.fromisoformat(value)
    except ValueError:
        errors.append(f"{_relative(artifact, root)}: {field} must use YYYY-MM-DD")


def _link_destination(raw_target: str, source: Path) -> tuple[Path | None, bool]:
    target = raw_target.strip()
    if target.startswith("<") and target.endswith(">"):
        target = target[1:-1]
    target = target.split("#", 1)[0].strip()
    if not target:
        return None, False
    if re.match(r"^[a-z][a-z0-9+.-]*://", target, re.IGNORECASE):
        return None, False
    absolute = target.startswith(('/', '\\')) or WINDOWS_ABSOLUTE.match(target) is not None
    candidate = Path(target.replace("/", str(Path('/'))))
    if absolute:
        return candidate.resolve(), True
    return (source.parent / candidate).resolve(), False


def _validate_plan_link(
    plan: Path | None,
    artifact: Path,
    *,
    role: str,
    kind: str,
    root: Path,
    errors: list[str],
) -> None:
    if plan is None or not plan.is_file():
        return
    content = plan.read_text(encoding="utf-8")
    artifact_resolved = artifact.resolve()
    matching_line: str | None = None
    absolute_match = False
    for line in content.splitlines():
        for raw_target in MARKDOWN_LINK.findall(line):
            destination, absolute = _link_destination(raw_target, plan)
            if destination == artifact_resolved:
                matching_line = line
                absolute_match = absolute
                break
        if matching_line is not None:
            break

    plan_name = _relative(plan, root)
    if matching_line is None:
        errors.append(
            f"{plan_name}: {role} plan must retain a relative Markdown link to {_relative(artifact, root)}"
        )
        return
    if absolute_match:
        errors.append(
            f"{plan_name}: {role} plan must use a relative Markdown link to {_relative(artifact, root)}"
        )
    expected = ("open", "待修复") if kind == "failure" else ("fixed", "已修复")
    if not any(token in matching_line.lower() if token.isascii() else token in matching_line for token in expected):
        errors.append(
            f"{plan_name}: handoff link line must include a concise {expected[0]}/{expected[1]} status summary"
        )


def _candidate_files(root: Path) -> list[Path]:
    plans_root = root / "docs/plans"
    if not plans_root.is_dir():
        return []
    candidates: list[Path] = []
    for path in plans_root.rglob("*.md"):
        name = path.name
        lowered = name.lower()
        if (
            lowered.startswith("failure-")
            or lowered.startswith("fixed-")
            or DATE_FIRST_HANDOFF.match(name)
            or "failure-handoff" in lowered
        ):
            candidates.append(path)
    return sorted(candidates)


def validate_repository(root: Path) -> list[str]:
    root = root.resolve()
    errors: list[str] = []
    lifecycle_keys: dict[tuple[str, str, str], list[Path]] = {}

    for artifact in _candidate_files(root):
        artifact_name = _relative(artifact, root)
        name_match = CANONICAL_NAME.match(artifact.name)
        if not name_match:
            errors.append(f"{artifact_name}: noncanonical handoff filename; use failure-{{date}}-{{summary}}.md or fixed-{{date}}-{{summary}}.md")

        content = artifact.read_text(encoding="utf-8")
        metadata, metadata_errors = _parse_frontmatter(artifact, content)
        errors.extend(_relative_error(error, root) for error in metadata_errors)
        for key in BASE_KEYS:
            if not metadata.get(key):
                errors.append(f"{artifact_name}: missing frontmatter key '{key}'")

        kind = name_match.group(1) if name_match else metadata.get("handoff_kind", "")
        filename_date = name_match.group(2) if name_match else ""
        filename_slug = name_match.group(3) if name_match else ""
        if metadata.get("handoff_kind") and metadata["handoff_kind"] != kind:
            errors.append(f"{artifact_name}: handoff_kind must match the filename prefix")
        expected_status = "open" if kind == "failure" else "fixed"
        if kind in {"failure", "fixed"} and metadata.get("status") != expected_status:
            errors.append(f"{artifact_name}: {kind} artifacts require status: {expected_status}")
        if kind == "fixed" and not metadata.get("resolved_at"):
            errors.append(f"{artifact_name}: missing frontmatter key 'resolved_at'")

        slug = metadata.get("summary_slug", "")
        if slug and not SLUG.fullmatch(slug):
            errors.append(f"{artifact_name}: summary_slug must be lowercase hyphenated text")
        if filename_slug and slug and filename_slug != slug:
            errors.append(f"{artifact_name}: filename summary must equal summary_slug")

        if metadata.get("created_at"):
            _parse_date(metadata["created_at"], field="created_at", artifact=artifact, root=root, errors=errors)
        if metadata.get("resolved_at"):
            _parse_date(metadata["resolved_at"], field="resolved_at", artifact=artifact, root=root, errors=errors)
        expected_date = metadata.get("created_at") if kind == "failure" else metadata.get("resolved_at")
        if filename_date and expected_date and filename_date != expected_date:
            date_field = "created_at" if kind == "failure" else "resolved_at"
            errors.append(f"{artifact_name}: filename date must equal {date_field}")

        origin_plan = _repo_path(
            metadata.get("origin_plan", ""), field="origin_plan", artifact=artifact, root=root, errors=errors
        )
        fixing_plan = _repo_path(
            metadata.get("fixing_plan", ""), field="fixing_plan", artifact=artifact, root=root, errors=errors
        )
        origin_child = _repo_path(
            metadata.get("origin_child_dir", ""), field="origin_child_dir", artifact=artifact, root=root, errors=errors
        )
        fixing_child = _repo_path(
            metadata.get("fixing_child_dir", ""), field="fixing_child_dir", artifact=artifact, root=root, errors=errors
        )
        derived_origin = _derived_child_dir(
            origin_plan, field="origin_plan", artifact=artifact, root=root, errors=errors
        )
        derived_fixing = _derived_child_dir(
            fixing_plan, field="fixing_plan", artifact=artifact, root=root, errors=errors
        )
        if origin_child and derived_origin and origin_child.resolve() != derived_origin.resolve():
            errors.append(f"{artifact_name}: origin_child_dir must match the origin plan number")
        if fixing_child and derived_fixing and fixing_child.resolve() != derived_fixing.resolve():
            errors.append(f"{artifact_name}: fixing_child_dir must match the fixing plan number")
        if (
            derived_origin
            and derived_fixing
            and derived_origin.resolve() == derived_fixing.resolve()
        ):
            errors.append(
                f"{artifact_name}: origin and fixing owners must belong to different numbered child plans"
            )

        if kind == "failure" and fixing_child and artifact.parent.resolve() != fixing_child.resolve():
            errors.append(f"{artifact_name}: failure artifact must be stored in fixing_child_dir")
        if kind == "fixed" and origin_child and artifact.parent.resolve() != origin_child.resolve():
            errors.append(f"{artifact_name}: fixed artifact must be returned to origin_child_dir")

        for heading in REQUIRED_HEADINGS:
            if heading not in content:
                errors.append(f"{artifact_name}: missing required heading '{heading}'")
        source_section = content.split("## 来源执行者", 1)[-1].split(
            "## 失败现象与复现证据", 1
        )[0]
        source_values: dict[str, str] = {}
        for label in ("来源计划：", "来源执行切片：", "修复责任计划：", "交接原因："):
            match = re.search(
                rf"^\s*(?:[-*]\s*)?{re.escape(label)}\s*(\S.*)?$",
                source_section,
                re.MULTILINE,
            )
            if match is None or not (match.group(1) or "").strip():
                errors.append(
                    f"{artifact_name}: source executor field '{label}' requires a non-empty value"
                )
            else:
                source_values[label] = (match.group(1) or "").strip().strip("`")
        expected_provenance = (
            ("来源计划：", metadata.get("origin_plan", "")),
            ("修复责任计划：", metadata.get("fixing_plan", "")),
        )
        for label, expected_path in expected_provenance:
            actual_path = source_values.get(label, "").replace("\\", "/")
            if actual_path and expected_path and actual_path != expected_path.replace("\\", "/"):
                errors.append(
                    f"{artifact_name}: source executor field '{label}' must match frontmatter provenance"
                )
        if kind == "fixed":
            result_section = content.split("## 修复结果与回传", 1)[-1]
            for marker in ("待修复", "status: open", "open /", "open状态"):
                if marker.lower() in result_section.lower():
                    errors.append(
                        f"{artifact_name}: fixed artifact still contains open-state marker '{marker}'"
                    )
            for label in ("根因：", "架构修复：", "验证：", "回传："):
                match = re.search(
                    rf"^\s*(?:[-*]\s*)?{re.escape(label)}\s*(\S.*)?$",
                    result_section,
                    re.MULTILINE,
                )
                if match is None:
                    errors.append(
                        f"{artifact_name}: fixed result section missing required field '{label}'"
                    )
                elif not (match.group(1) or "").strip():
                    errors.append(
                        f"{artifact_name}: fixed result field '{label}' requires a non-empty value"
                    )

        _validate_plan_link(
            origin_plan, artifact, role="origin", kind=kind, root=root, errors=errors
        )
        _validate_plan_link(
            fixing_plan, artifact, role="fixing", kind=kind, root=root, errors=errors
        )

        if origin_plan and fixing_plan and metadata.get("summary_slug"):
            key = (
                origin_plan.resolve().as_posix().casefold(),
                fixing_plan.resolve().as_posix().casefold(),
                metadata["summary_slug"],
            )
            lifecycle_keys.setdefault(key, []).append(artifact)

    for key, artifacts in lifecycle_keys.items():
        if len(artifacts) > 1:
            paths = ", ".join(_relative(path, root) for path in artifacts)
            errors.append(f"duplicate canonical handoff lifecycle {key}: {paths}")

    return errors


def _relative_error(error: str, root: Path) -> str:
    root_prefix = f"{root}{Path('/')}"
    return error.replace(root_prefix, "")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Validate ZirconEngine plan failure/fixed handoffs.")
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    args = parser.parse_args(argv)

    candidates = _candidate_files(args.repo_root.resolve())
    errors = validate_repository(args.repo_root)
    if errors:
        for error in errors:
            print(f"ERROR: {error}")
        print(f"Validated {len(candidates)} handoff artifact(s): {len(errors)} error(s).")
        return 1
    print(f"Validated {len(candidates)} handoff artifact(s): 0 errors.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
