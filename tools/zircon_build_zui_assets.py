"""Staged .zui asset validation for zircon_build."""

from __future__ import annotations

from pathlib import Path

try:
    from .zircon_export.plugin_validate_distribution_zui_assets import (
        validate_plugin_distribution_zui_asset,
    )
except ImportError:  # pragma: no cover - exercised when zircon_build.py is run directly.
    from zircon_export.plugin_validate_distribution_zui_assets import (
        validate_plugin_distribution_zui_asset,
    )


def validate_staged_engine_asset_suffix(relative: Path, source: Path) -> None:
    normalized = relative.as_posix()
    if normalized.startswith("ui/") and normalized.endswith(".ui.toml"):
        raise SystemExit(
            "Legacy UI document suffix is not stageable after .zui cutover: "
            f"{source}. Rename the asset to .zui."
        )
    if normalized.startswith("ui/") and relative.suffix == ".zui":
        diagnostics: list[str] = []
        validate_plugin_distribution_zui_asset(
            item_label="staged engine asset",
            relative_source=relative,
            source_path=source,
            diagnostics=diagnostics,
        )
        if diagnostics:
            raise SystemExit("; ".join(diagnostics))
