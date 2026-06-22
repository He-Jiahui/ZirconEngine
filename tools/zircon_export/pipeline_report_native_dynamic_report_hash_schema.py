"""Schema helpers for NativeDynamic report content hashes."""

from __future__ import annotations

from typing import Any

from .export_template import is_sha256_hex


def native_dynamic_content_hash_is_schema_clean(value: object) -> bool:
    return (
        isinstance(value, str)
        and bool(value.strip())
        and value.strip() == value
        and is_sha256_hex(value)
    )


def native_dynamic_report_content_hash_schema_diagnostics(
    label: str,
    report: dict[str, Any],
) -> list[str]:
    value = report.get("content_hash")
    if (
        isinstance(value, str)
        and value.strip()
        and value.strip() == value
        and not is_sha256_hex(value)
    ):
        return [f"{label} content_hash must be a SHA-256 hex digest"]
    return []
