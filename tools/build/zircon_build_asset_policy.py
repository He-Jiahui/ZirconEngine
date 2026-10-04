"""Product-specific asset inclusion policy for ZirconEngine staging."""

from __future__ import annotations

from pathlib import Path


SHIPPING_RUNTIME_EXCLUDED_TOP_LEVEL = frozenset({"cache"})
SHIPPING_RUNTIME_EXCLUDED_PREFIXES = (
    "fonts/editor-ui-sources/",
    "fonts/editor-ui.font.toml",
    "fonts/editor-ui.ttc",
    "font/editor-ui-sources/",
    "font/editor-ui.font.toml",
    "font/editor-ui.ttc",
    "ui/compiled_artifacts/",
    "ui/editor/",
)


def shipping_runtime_asset_excluded(relative: Path) -> bool:
    """Return whether a runtime asset is editor-only or transient cache data."""
    key = relative.as_posix().lower()
    if relative.parts and relative.parts[0].lower() in SHIPPING_RUNTIME_EXCLUDED_TOP_LEVEL:
        return True
    return any(
        _matches_path_prefix(key, prefix)
        for prefix in SHIPPING_RUNTIME_EXCLUDED_PREFIXES
    )


def _matches_path_prefix(relative_key: str, prefix: str) -> bool:
    """Match a path prefix on a component boundary, not by raw substring."""
    normalized = prefix.rstrip("/").lower()
    return relative_key == normalized or relative_key.startswith(normalized + "/")
