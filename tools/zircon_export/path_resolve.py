"""Diagnostics-aware path resolution helpers for export stages."""

from __future__ import annotations

import os
from pathlib import Path


def resolve_stage_optional_path(
    value: object,
    label: str,
    diagnostics: list[str],
    *,
    prefix: str,
) -> Path | None:
    if value is None:
        return None
    if not isinstance(value, (str, os.PathLike)) or not str(value):
        diagnostics.append(f"{prefix} {label} argument must be a non-empty path")
        return None
    try:
        return Path(value).expanduser().resolve()
    except OSError as error:
        diagnostics.append(f"{prefix} {label} {value} could not be resolved: {error}")
        return None
