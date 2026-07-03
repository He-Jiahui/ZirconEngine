"""Root and embedded-feature default packaging validation."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .native_build_workspace import read_toml
from .plugin_validate_common import (
    PLUGIN_VALIDATE_DEFAULT_PACKAGING,
    plugin_validate_allowed_string_values,
    plugin_validate_string_array,
)


Diagnostics = list[str]
Manifest = dict[str, Any]

PLUGIN_VALIDATE_DEFAULT_PACKAGING_METADATA_DUPLICATE_MESSAGE = "duplicates default_packaging"


def validate_plugin_default_packaging(
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
    validate_plugin_default_packaging_values(
        manifest,
        "default_packaging",
        f"plugin {package_id} default_packaging",
        diagnostics,
    )
    validate_plugin_optional_feature_default_packaging(
        manifest.get("optional_features"),
        package_id,
        diagnostics,
    )
    validate_plugin_feature_extension_default_packaging(
        manifest.get("feature_extensions"),
        package_id,
        diagnostics,
    )


def validate_plugin_optional_feature_default_packaging(
    optional_features: Any,
    package_id: str,
    diagnostics: Diagnostics,
) -> None:
    if optional_features is None or not isinstance(optional_features, list):
        return
    for index, feature in enumerate(optional_features):
        if not isinstance(feature, dict):
            continue
        validate_plugin_default_packaging_values(
            feature,
            "default_packaging",
            f"plugin {package_id} optional_features[{index}].default_packaging",
            diagnostics,
        )


def validate_plugin_feature_extension_default_packaging(
    feature_extensions: Any,
    package_id: str,
    diagnostics: Diagnostics,
) -> None:
    if feature_extensions is None or not isinstance(feature_extensions, list):
        return
    for index, feature in enumerate(feature_extensions):
        if not isinstance(feature, dict):
            continue
        validate_plugin_default_packaging_values(
            feature,
            "default_packaging",
            f"plugin {package_id} feature_extensions[{index}].default_packaging",
            diagnostics,
        )


def validate_plugin_default_packaging_values(
    table: Manifest,
    field: str,
    label: str,
    diagnostics: Diagnostics,
) -> None:
    values = plugin_validate_string_array(table, field, label, diagnostics)
    if values is None:
        return
    plugin_validate_allowed_string_values(
        values, label, PLUGIN_VALIDATE_DEFAULT_PACKAGING, diagnostics
    )
    seen: dict[str, int] = {}
    for index, value in enumerate(values):
        previous_index = seen.get(value)
        if previous_index is not None:
            diagnostics.append(
                f"{label}[{index}] {value} "
                f"{PLUGIN_VALIDATE_DEFAULT_PACKAGING_METADATA_DUPLICATE_MESSAGE}[{previous_index}]"
            )
            continue
        seen[value] = index
