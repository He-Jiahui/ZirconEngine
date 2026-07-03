"""Feature extension row validation for standalone plugin manifests."""

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

PLUGIN_VALIDATE_FEATURE_EXTENSION_FIELDS = {
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


def validate_plugin_feature_extensions(
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
    feature_extensions = manifest.get("feature_extensions")
    if feature_extensions is None:
        return
    label = f"plugin {package_id} feature_extensions"
    if not isinstance(feature_extensions, list):
        diagnostics.append(f"{label} must be an array")
        return
    if not feature_extensions:
        diagnostics.append(f"{label} must not be empty when declared")
        return
    seen_ids: dict[str, int] = {}
    for index, feature in enumerate(feature_extensions):
        feature_label = f"{label}[{index}]"
        if not isinstance(feature, dict):
            diagnostics.append(f"{feature_label} must be a table")
            continue
        validate_plugin_feature_extension_row(
            feature,
            feature_label,
            seen_ids,
            diagnostics,
        )


def validate_plugin_feature_extension_row(
    feature: Manifest,
    feature_label: str,
    seen_ids: dict[str, int],
    diagnostics: Diagnostics,
) -> None:
    validate_plugin_feature_extension_known_fields(feature, feature_label, diagnostics)
    owner_plugin_id = plugin_validate_trimmed_string(
        feature, "owner_plugin_id", f"{feature_label}.owner_plugin_id", diagnostics
    )
    if owner_plugin_id is not None:
        validate_plugin_feature_extension_owner_package_token(
            owner_plugin_id,
            f"{feature_label}.owner_plugin_id",
            diagnostics,
        )
    feature_id = validate_plugin_feature_extension_id(
        feature, f"{feature_label}.id", owner_plugin_id, diagnostics
    )
    if feature_id is not None:
        previous_index = seen_ids.get(feature_id)
        if previous_index is not None:
            diagnostics.append(
                f"{feature_label}.id {feature_id} "
                f"duplicates feature extension id feature_extensions[{previous_index}]"
            )
        else:
            seen_ids[feature_id] = int(feature_label.rsplit("[", 1)[-1].rstrip("]"))
    plugin_validate_trimmed_string(
        feature, "display_name", f"{feature_label}.display_name", diagnostics
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


def validate_plugin_feature_extension_known_fields(
    feature: Manifest,
    feature_label: str,
    diagnostics: Diagnostics,
) -> None:
    for field_name in sorted(feature):
        if field_name not in PLUGIN_VALIDATE_FEATURE_EXTENSION_FIELDS:
            diagnostics.append(
                f"{feature_label}.{field_name} "
                "is not a known feature extension field"
            )


def validate_plugin_feature_extension_id(
    feature: Manifest,
    label: str,
    owner_plugin_id: str | None,
    diagnostics: Diagnostics,
) -> str | None:
    value = plugin_validate_trimmed_string(feature, "id", label, diagnostics)
    if value is None:
        return None
    validate_plugin_feature_extension_dot_namespace(value, label, diagnostics)
    if owner_plugin_id is not None:
        expected_prefix = f"{owner_plugin_id}."
        if not value.startswith(expected_prefix):
            diagnostics.append(
                f"{label} {value} should stay under owner namespace {expected_prefix}"
            )
    return value


def validate_plugin_feature_extension_dot_namespace(
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


def validate_plugin_feature_extension_owner_package_token(
    value: str,
    label: str,
    diagnostics: Diagnostics,
) -> None:
    if not ("a" <= value[0] <= "z"):
        diagnostics.append(f"{label} {value} should start with a lowercase ASCII letter")
    if not all(
        char.isascii() and (char.islower() or char.isdigit() or char == "_")
        for char in value
    ):
        diagnostics.append(
            f"{label} {value} should contain only lowercase ASCII letters, "
            "digits, and underscores"
        )
    if value.endswith("_") or "__" in value:
        diagnostics.append(
            f"{label} {value} should not end with an underscore "
            "or contain repeated underscores"
        )
