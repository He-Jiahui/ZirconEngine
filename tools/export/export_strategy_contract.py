"""Shared export strategy contract for Zircon export reports."""

from __future__ import annotations

SUPPORTED_EXPORT_STRATEGIES = (
    "library_embed",
    "native_dynamic",
    "source_template",
)

EXPORT_STRATEGY_ALIASES = {
    "LibraryEmbed": "library_embed",
    "NativeDynamic": "native_dynamic",
    "SourceTemplate": "source_template",
}


def normalize_export_strategy(value: object) -> str | None:
    if not isinstance(value, str):
        return None
    normalized = value.strip().replace("-", "_").replace(" ", "_")
    aliased = EXPORT_STRATEGY_ALIASES.get(normalized)
    if aliased is not None:
        return aliased
    lowered = normalized.lower()
    if lowered in SUPPORTED_EXPORT_STRATEGIES:
        return lowered
    return None
