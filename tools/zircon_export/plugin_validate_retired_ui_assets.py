"""Repository-level retired UI asset checks for plugin validation."""

from __future__ import annotations

from collections.abc import Iterable
from pathlib import Path

from .plugin_validate_distribution_assets import (
    PLUGIN_VALIDATE_RETIRED_UI_ASSET_SUFFIXES,
    plugin_validate_retired_ui_asset_suffix,
)


PLUGIN_VALIDATE_RETIRED_UI_ASSET_SCAN_ROOTS = (
    "zircon_editor",
    "zircon_plugins",
    "zircon_runtime",
)


def validate_plugin_retired_ui_asset_files(
    repo_root: Path,
    diagnostics: list[str],
) -> None:
    for relative_path in _plugin_validate_retired_ui_asset_files(
        repo_root,
        roots=(
            repo_root / scan_root
            for scan_root in PLUGIN_VALIDATE_RETIRED_UI_ASSET_SCAN_ROOTS
        ),
    ):
        suffix = plugin_validate_retired_ui_asset_suffix(relative_path)
        if suffix is None:
            continue
        diagnostics.append(
            "plugin validate --all retired UI asset file "
            f"{relative_path.as_posix()} uses retired UI asset suffix {suffix}; use .zui"
        )


def validate_plugin_target_retired_ui_asset_files(
    *,
    plugin_manifest_path: Path | None,
    package_id: str,
    diagnostics: list[str],
) -> None:
    if plugin_manifest_path is None:
        return
    package_root = plugin_manifest_path.parent
    for relative_path in _plugin_validate_retired_ui_asset_files(
        package_root,
        roots=(package_root,),
    ):
        suffix = plugin_validate_retired_ui_asset_suffix(relative_path)
        if suffix is None:
            continue
        diagnostics.append(
            f"plugin {package_id} retired UI asset file {relative_path.as_posix()} "
            f"uses retired UI asset suffix {suffix}; use .zui"
        )


def _plugin_validate_retired_ui_asset_files(
    base_root: Path,
    *,
    roots: Iterable[Path],
) -> list[Path]:
    retired_paths: list[Path] = []
    for root in roots:
        if not root.exists():
            continue
        for path in root.rglob("*"):
            if not path.is_file():
                continue
            relative_path = path.relative_to(base_root)
            if plugin_validate_retired_ui_asset_suffix(relative_path) is not None:
                retired_paths.append(relative_path)
    return sorted(retired_paths, key=lambda path: path.as_posix())
