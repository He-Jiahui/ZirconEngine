"""Root package kind validation for standalone plugin manifests."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .native_build_workspace import read_toml
from .plugin_validate_common import plugin_validate_optional_trimmed_string


Diagnostics = list[str]
Manifest = dict[str, Any]

PLUGIN_VALIDATE_PACKAGE_KIND_VALUES = ("standard", "feature_extension")


def validate_plugin_package_kind(
    *,
    plugin_manifest_path: Path | None,
    package_id: str,
    diagnostics: Diagnostics,
) -> None:
    if plugin_manifest_path is None:
        return
    manifest = read_toml(plugin_manifest_path, diagnostics)
    if manifest is None:
        return
    package_kind = "standard"
    if "package_kind" in manifest:
        declared_kind = plugin_validate_optional_trimmed_string(
            manifest,
            "package_kind",
            f"plugin {package_id} package_kind",
            diagnostics,
        )
        if declared_kind is None:
            return
        package_kind = declared_kind
    if package_kind not in PLUGIN_VALIDATE_PACKAGE_KIND_VALUES:
        diagnostics.append(
            f"plugin {package_id} package_kind {package_kind} "
            "should be standard or feature_extension"
        )
        return
    validate_plugin_package_kind_coherence(
        package_kind,
        package_id,
        plugin_validate_table_array_row_count(
            manifest, "optional_features", package_id, diagnostics
        ),
        plugin_validate_table_array_row_count(
            manifest, "feature_extensions", package_id, diagnostics
        ),
        diagnostics,
    )


def validate_plugin_package_kind_coherence(
    package_kind: str,
    package_id: str,
    optional_feature_count: int,
    feature_extension_count: int,
    diagnostics: Diagnostics,
) -> None:
    if package_kind == "standard" and feature_extension_count:
        diagnostics.append(
            f"plugin {package_id} standard package_kind "
            "should not declare feature_extensions rows"
        )
    if package_kind == "feature_extension":
        if feature_extension_count == 0:
            diagnostics.append(
                f"plugin {package_id} package_kind feature_extension "
                "should declare at least one feature_extensions row"
            )
        if optional_feature_count:
            diagnostics.append(
                f"plugin {package_id} package_kind feature_extension "
                "should not declare optional_features rows"
            )


def plugin_validate_table_array_row_count(
    manifest: Manifest,
    field: str,
    package_id: str,
    diagnostics: Diagnostics,
) -> int:
    rows = manifest.get(field)
    if rows is None:
        return 0
    if not isinstance(rows, list):
        diagnostics.append(f"plugin {package_id} {field} must be an array")
        return 0
    for index, row in enumerate(rows):
        if not isinstance(row, dict):
            diagnostics.append(f"plugin {package_id} {field}[{index}] must be a table")
    return len(rows)
