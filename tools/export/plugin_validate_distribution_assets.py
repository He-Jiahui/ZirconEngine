"""distribution.assets glob checks for plugin validation."""

from __future__ import annotations

from pathlib import Path, PureWindowsPath
from typing import Any

from .plugin_validate_distribution_asset_matches import (
    plugin_validate_distribution_asset_matches,
)
from .plugin_validate_distribution_zui_assets import (
    validate_plugin_distribution_zui_asset,
)


PLUGIN_VALIDATE_RETIRED_UI_ASSET_SUFFIXES = (".v2.ui.toml", ".ui.toml")


def _is_plugin_relative_asset_glob(pattern: str) -> bool:
    pattern_path = Path(pattern)
    windows_pattern_path = PureWindowsPath(pattern)
    return not (
        pattern_path.is_absolute()
        or pattern_path.anchor
        or windows_pattern_path.anchor
        or ".." in pattern_path.parts
        or ".." in windows_pattern_path.parts
    )


def plugin_validate_retired_ui_asset_pattern_suffix(pattern: str) -> str | None:
    return _plugin_validate_retired_ui_asset_suffix_text(pattern)


def plugin_validate_retired_ui_asset_suffix(path: Path) -> str | None:
    return _plugin_validate_retired_ui_asset_suffix_text(path.as_posix())


def _plugin_validate_retired_ui_asset_suffix_text(asset_path: str) -> str | None:
    for suffix in PLUGIN_VALIDATE_RETIRED_UI_ASSET_SUFFIXES:
        if asset_path.endswith(suffix):
            return suffix
    return None


def plugin_validate_distribution_assets(
    distribution: dict[str, Any],
    package_id: str,
    diagnostics: list[str],
    *,
    plugin_manifest_path: Path | None = None,
    distribution_label: str | None = None,
) -> None:
    assets = distribution.get("assets")
    if assets is None:
        return
    distribution_label = distribution_label or f"plugin {package_id} distribution"
    label = f"{distribution_label}.assets"
    if not isinstance(assets, list):
        diagnostics.append(f"{label} must be an array")
        return
    plugin_root = plugin_manifest_path.parent if plugin_manifest_path is not None else None
    resolved_plugin_root = plugin_root.resolve() if plugin_root is not None else None
    for index, raw_pattern in enumerate(assets):
        item_label = f"{label}[{index}]"
        if not isinstance(raw_pattern, str) or not raw_pattern.strip():
            diagnostics.append(f"{item_label} must be a non-empty string")
            continue
        if raw_pattern.strip() != raw_pattern:
            diagnostics.append(f"{item_label} must be trimmed")
            continue
        pattern_path = Path(raw_pattern)
        if not _is_plugin_relative_asset_glob(raw_pattern):
            diagnostics.append(f"{item_label} must be a plugin-relative glob")
            continue
        if plugin_validate_retired_ui_asset_pattern_suffix(raw_pattern) is not None:
            diagnostics.append(
                f"{item_label} targets retired UI asset suffix "
                f"{pattern_path.as_posix()}; use .zui"
            )
            continue
        if plugin_root is None or resolved_plugin_root is None:
            continue
        matches = plugin_validate_distribution_asset_matches(
            pattern=raw_pattern,
            plugin_root=plugin_root,
            resolved_plugin_root=resolved_plugin_root,
            item_label=item_label,
            diagnostics=diagnostics,
        )
        for source_path, relative_source in matches:
            if plugin_validate_retired_ui_asset_suffix(relative_source) is not None:
                diagnostics.append(
                    f"{item_label} matched retired UI asset suffix "
                    f"{relative_source.as_posix()}; use .zui"
                )
                continue
            validate_plugin_distribution_zui_asset(
                item_label=item_label,
                relative_source=relative_source,
                source_path=source_path,
                diagnostics=diagnostics,
            )
