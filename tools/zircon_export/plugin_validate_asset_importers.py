"""asset_importers full_suffixes checks for plugin validation."""

from __future__ import annotations

from pathlib import Path

from .native_build_workspace import read_toml
from .plugin_validate_distribution_assets import (
    plugin_validate_retired_ui_asset_pattern_suffix,
)
from .plugin_validate_asset_importer_ids import validate_plugin_asset_importer_id
from .plugin_validate_asset_importer_metadata_arrays import validate_plugin_asset_importer_metadata_arrays
from .plugin_validate_asset_importer_numbers import validate_plugin_asset_importer_numbers
from .plugin_validate_asset_importer_required_capabilities import validate_plugin_asset_importer_required_capabilities
from .plugin_validate_asset_importer_required_capability_gates import (
    plugin_validate_static_declared_capabilities,
    validate_plugin_asset_importer_required_capability_gates,
)
from .plugin_validate_asset_importer_schema import validate_plugin_asset_importer_schema


def validate_plugin_asset_importers(
    *,
    plugin_manifest_path: Path | None,
    plugin_root: Path | None,
    package_id: str,
    diagnostics: list[str],
) -> None:
    if plugin_manifest_path is None:
        return
    manifest = read_toml(plugin_manifest_path, diagnostics)
    if manifest is None:
        return
    asset_importers = manifest.get("asset_importers")
    if asset_importers is None:
        return
    label = f"plugin {package_id} asset_importers"
    if not isinstance(asset_importers, list):
        diagnostics.append(f"{label} must be an array")
        return
    if not asset_importers:
        diagnostics.append(f"{label} must not be empty when declared")
        return
    declared_capabilities = plugin_validate_static_declared_capabilities(plugin_root, diagnostics)
    for index, importer in enumerate(asset_importers):
        importer_label = f"{label}[{index}]"
        if not isinstance(importer, dict):
            diagnostics.append(f"{importer_label} must be a table")
            continue
        validate_plugin_asset_importer_schema(importer, importer_label, package_id, diagnostics)
        validate_plugin_asset_importer_id(importer, importer_label, diagnostics)
        validate_plugin_asset_importer_metadata_arrays(
            importer, importer_label, diagnostics
        )
        validate_plugin_asset_importer_numbers(importer, importer_label, diagnostics)
        validate_plugin_asset_importer_required_capabilities(importer, importer_label, diagnostics)
        validate_plugin_asset_importer_required_capability_gates(
            importer, importer_label, declared_capabilities, diagnostics
        )
        validate_plugin_asset_importer_source_selector(importer, importer_label, diagnostics)
        validate_plugin_asset_importer_full_suffixes(importer, importer_label, diagnostics)


def validate_plugin_asset_importer_source_selector(
    importer: dict[str, object],
    importer_label: str,
    diagnostics: list[str],
) -> None:
    selector_fields = ("source_extensions", "full_suffixes")
    if all(field not in importer for field in selector_fields):
        diagnostics.append(
            f"{importer_label} must declare source_extensions or full_suffixes"
        )
    for field in selector_fields:
        value = importer.get(field)
        if isinstance(value, list) and not value:
            diagnostics.append(f"{importer_label}.{field} must not be empty when declared")


def validate_plugin_asset_importer_full_suffixes(
    importer: dict[str, object],
    importer_label: str,
    diagnostics: list[str],
) -> None:
    full_suffixes = importer.get("full_suffixes")
    if full_suffixes is None:
        return
    label = f"{importer_label}.full_suffixes"
    if not isinstance(full_suffixes, list):
        diagnostics.append(f"{label} must be an array")
        return
    seen: dict[str, int] = {}
    for index, raw_suffix in enumerate(full_suffixes):
        item_label = f"{label}[{index}]"
        if not isinstance(raw_suffix, str) or not raw_suffix.strip():
            diagnostics.append(f"{item_label} must be a non-empty string")
            continue
        if raw_suffix.strip() != raw_suffix:
            diagnostics.append(f"{item_label} must be trimmed")
            continue
        duplicate_index = seen.get(raw_suffix)
        if duplicate_index is not None:
            diagnostics.append(f"{item_label} duplicates entry {duplicate_index}")
            continue
        seen[raw_suffix] = index
        if not raw_suffix.startswith(".") or raw_suffix == ".":
            diagnostics.append(f"{item_label} must be a dotted suffix")
            continue
        if raw_suffix != raw_suffix.lower():
            diagnostics.append(f"{item_label} must be lowercase")
            continue
        retired_suffix = plugin_validate_retired_ui_asset_pattern_suffix(raw_suffix)
        if retired_suffix is not None:
            diagnostics.append(
                f"{item_label} declares retired UI asset suffix "
                f"{retired_suffix}; use .zui"
            )
