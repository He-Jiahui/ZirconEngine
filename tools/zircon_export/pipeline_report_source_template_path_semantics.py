"""Shared SourceTemplate path and string semantics for final report checks."""

from __future__ import annotations

from pathlib import Path
from typing import Any


def resolve_source_template_path_or_diagnostic(
    path: str | Path,
    diagnostics: list[str],
    label: str,
) -> Path | None:
    try:
        return Path(path).expanduser().resolve()
    except OSError as error:
        diagnostics.append(f"{label} {path} could not be resolved: {error}")
        return None


def source_template_is_non_empty_trimmed_string(value: Any) -> bool:
    return isinstance(value, str) and bool(value.strip()) and value.strip() == value


def source_template_generated_file_path(
    project_dir: Path,
    relative_path: str,
    diagnostics: list[str],
    *,
    kind: str = "SourceTemplate generated file path",
) -> Path | None:
    file_path = Path(relative_path)
    if file_path.is_absolute():
        diagnostics.append(f"{kind} {relative_path} must be relative")
        return None
    resolved_project = resolve_source_template_path_or_diagnostic(
        project_dir,
        diagnostics,
        f"{kind} project",
    )
    if resolved_project is None:
        return None
    resolved_path = resolve_source_template_path_or_diagnostic(
        resolved_project / file_path,
        diagnostics,
        f"{kind} {relative_path}",
    )
    if resolved_path is None:
        return None
    try:
        resolved_path.relative_to(resolved_project)
    except ValueError:
        diagnostics.append(f"{kind} {relative_path} escapes the project")
        return None
    return resolved_path
