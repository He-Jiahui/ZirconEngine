"""SourceTemplate path helpers shared by stage and plan owners."""

from __future__ import annotations

import os
from pathlib import Path


def default_repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def resolve_source_template_optional_path(
    value: object,
    label: str,
    diagnostics: list[str],
) -> Path | None:
    if value is None:
        return None
    if not isinstance(value, (str, os.PathLike)):
        diagnostics.append(f"{label} argument must be a path-like value")
        return None
    try:
        return resolve_user_path(value)
    except OSError as error:
        diagnostics.append(f"{label} {value} could not be resolved: {error}")
        return None


def resolve_user_path(path: str | os.PathLike[str]) -> Path:
    return Path(path).expanduser().resolve()
