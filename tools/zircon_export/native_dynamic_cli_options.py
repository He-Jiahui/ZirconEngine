"""NativeDynamic CLI option normalization helpers."""

from __future__ import annotations

from pathlib import Path


def native_dynamic_cli_optional_trimmed_string(
    value: object,
    field: str,
    diagnostics: list[str],
) -> str | None:
    if value is None:
        return None
    if not isinstance(value, str):
        diagnostics.append(f"{field} must be a string")
        return None
    if not value or value.strip() != value:
        diagnostics.append(f"{field} must be a non-empty trimmed string")
        return None
    return value


def native_dynamic_cli_string_array(
    value: object,
    field: str,
    diagnostics: list[str],
    *,
    lowercase: bool = False,
) -> list[str]:
    if value is None:
        return []
    if not isinstance(value, list):
        value = [value]
    values: list[str] = []
    seen: set[str] = set()
    for index, item in enumerate(value):
        if not isinstance(item, str):
            diagnostics.append(f"{field}[{index}] must be a string")
            continue
        if not item or item.strip() != item:
            diagnostics.append(f"{field}[{index}] must be a non-empty trimmed string")
            continue
        normalized = item.lower() if lowercase else item
        if normalized in seen:
            continue
        values.append(normalized)
        seen.add(normalized)
    return values


def native_dynamic_signing_profile(
    value: object,
    field: str,
    diagnostics: list[str],
) -> str | None:
    return native_dynamic_cli_optional_trimmed_string(value, field, diagnostics)


def native_dynamic_signing_platforms(
    value: object,
    field: str,
    diagnostics: list[str],
) -> list[str]:
    return native_dynamic_cli_string_array(value, field, diagnostics, lowercase=True)


def default_repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def resolve_user_path(path: str | Path) -> Path:
    return Path(path).expanduser().resolve()
