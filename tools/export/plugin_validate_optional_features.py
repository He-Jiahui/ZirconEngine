"""Optional feature row validation for standalone plugin manifests."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .native_build_workspace import read_toml
from .plugin_validate_capabilities import validate_plugin_capability_values
from .plugin_validate_common import (
    plugin_validate_optional_trimmed_string,
    plugin_validate_string_array,
    plugin_validate_trimmed_string,
)

Diagnostics = list[str]
Manifest = dict[str, Any]

PLUGIN_VALIDATE_OPTIONAL_FEATURE_FIELDS = {
    "capabilities",
    "default_packaging",
    "dependencies",
    "distribution",
    "display_name",
    "enabled_by_default",
    "id",
    "modules",
    "owner_plugin_id",
    "provider_package_id",
}


def validate_plugin_optional_features(
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
    optional_features = manifest.get("optional_features")
    if optional_features is None:
        return
    label = f"plugin {package_id} optional_features"
    if not isinstance(optional_features, list):
        diagnostics.append(f"{label} must be an array")
        return
    if not optional_features:
        diagnostics.append(f"{label} must not be empty when declared")
        return
    seen_ids: dict[str, int] = {}
    for index, feature in enumerate(optional_features):
        feature_label = f"{label}[{index}]"
        if not isinstance(feature, dict):
            diagnostics.append(f"{feature_label} must be a table")
            continue
        validate_plugin_optional_feature_row(
            feature,
            feature_label,
            package_id,
            seen_ids,
            diagnostics,
        )


def validate_plugin_optional_feature_row(
    feature: Manifest,
    feature_label: str,
    package_id: str,
    seen_ids: dict[str, int],
    diagnostics: Diagnostics,
) -> None:
    validate_plugin_optional_feature_known_fields(feature, feature_label, diagnostics)
    feature_id = validate_plugin_optional_feature_id(
        feature, f"{feature_label}.id", package_id, diagnostics
    )
    if feature_id is not None:
        previous_index = seen_ids.get(feature_id)
        if previous_index is not None:
            diagnostics.append(
                f"{feature_label}.id {feature_id} "
                f"duplicates optional feature id optional_features[{previous_index}]"
            )
        else:
            seen_ids[feature_id] = int(feature_label.rsplit("[", 1)[-1].rstrip("]"))
    plugin_validate_trimmed_string(
        feature, "display_name", f"{feature_label}.display_name", diagnostics
    )
    owner_plugin_id = plugin_validate_trimmed_string(
        feature, "owner_plugin_id", f"{feature_label}.owner_plugin_id", diagnostics
    )
    if owner_plugin_id is not None and owner_plugin_id != package_id:
        diagnostics.append(
            f"{feature_label}.owner_plugin_id {owner_plugin_id} "
            f"should match package id {package_id}"
        )
    plugin_validate_optional_trimmed_string(
        feature, "provider_package_id", f"{feature_label}.provider_package_id", diagnostics
    )
    capabilities = plugin_validate_string_array(
        feature, "capabilities", f"{feature_label}.capabilities", diagnostics
    )
    if capabilities is not None:
        validate_plugin_capability_values(
            capabilities, f"{feature_label}.capabilities", diagnostics
        )
    if type(feature.get("enabled_by_default")) is not bool:
        diagnostics.append(f"{feature_label}.enabled_by_default must be a bool")


def validate_plugin_optional_feature_known_fields(
    feature: Manifest,
    feature_label: str,
    diagnostics: Diagnostics,
) -> None:
    for field_name in sorted(feature):
        if field_name not in PLUGIN_VALIDATE_OPTIONAL_FEATURE_FIELDS:
            diagnostics.append(
                f"{feature_label}.{field_name} "
                "is not a known optional feature field"
            )


def validate_plugin_optional_feature_id(
    feature: Manifest,
    label: str,
    package_id: str,
    diagnostics: Diagnostics,
) -> str | None:
    value = plugin_validate_trimmed_string(feature, "id", label, diagnostics)
    if value is None:
        return None
    validate_plugin_optional_feature_dot_namespace(value, label, diagnostics)
    expected_prefix = f"{package_id}."
    if not value.startswith(expected_prefix):
        diagnostics.append(
            f"{label} {value} should stay under package namespace {expected_prefix}"
        )
    return value


def validate_plugin_optional_feature_dot_namespace(
    value: str,
    label: str,
    diagnostics: Diagnostics,
) -> None:
    segments = value.split(".")
    if len(segments) < 2:
        diagnostics.append(f"{label} {value} should use owner.feature dot namespace form")
    if any(not segment for segment in segments):
        diagnostics.append(f"{label} {value} should not contain empty namespace segments")
    if not all(
        char.isascii() and (char.islower() or char.isdigit() or char in {"_", "."})
        for char in value
    ):
        diagnostics.append(
            f"{label} {value} should contain only lowercase ASCII letters, "
            "digits, underscores, and dots"
        )
