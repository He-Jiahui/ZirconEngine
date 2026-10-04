"""Feature-provider generated manifest metadata schema validation."""

from __future__ import annotations

from typing import Any

from .plugin_validate_common import plugin_validate_optional_trimmed_string, plugin_validate_string_array
from .plugin_validate_feature_provider_manifest_description import plugin_validate_feature_provider_manifest_description_projection
from .plugin_validate_feature_provider_manifest_metadata_values import plugin_validate_feature_provider_manifest_metadata_values
from .plugin_validate_feature_provider_manifest_required_metadata import (
    PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_ARRAY_FIELDS,
    PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_STRING_FIELDS,
    validate_plugin_feature_provider_manifest_required_metadata,
)


def plugin_validate_feature_provider_manifest_metadata_schema(
    manifest: dict[str, Any], package_id: str, diagnostics: list[str]
) -> None:
    label = f"plugin {package_id} generated manifest"
    validate_plugin_feature_provider_manifest_required_metadata(manifest, package_id, diagnostics)
    for field_name in PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_STRING_FIELDS:
        plugin_validate_optional_trimmed_string(
            manifest, field_name, f"{label}.{field_name}", diagnostics
        )
    for field_name in PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_ARRAY_FIELDS:
        if field_name in {"capabilities", "default_packaging"}:
            continue
        if field_name in manifest:
            plugin_validate_string_array(
                manifest, field_name, f"{label}.{field_name}", diagnostics
            )
    plugin_validate_feature_provider_manifest_metadata_values(manifest, package_id, diagnostics)


def plugin_validate_feature_provider_manifest_projection_consistency(manifest: dict[str, Any], package_id: str, diagnostics: list[str]) -> None:
    feature_extension = _single_feature_extension(manifest)
    if feature_extension is not None:
        plugin_validate_feature_provider_manifest_display_name_projection(
            manifest, feature_extension, package_id, diagnostics
        )
        plugin_validate_feature_provider_manifest_description_projection(manifest, feature_extension, package_id, diagnostics)
        plugin_validate_feature_provider_manifest_supported_targets_projection(
            manifest, feature_extension, package_id, diagnostics
        )
        _compare_generated_manifest_array(
            manifest,
            feature_extension,
            "capabilities",
            "generated feature_extensions[0].capabilities",
            package_id,
            diagnostics,
        )
        _compare_generated_manifest_array(
            manifest,
            feature_extension,
            "default_packaging",
            "generated feature_extensions[0].default_packaging",
            package_id,
            diagnostics,
        )
    distribution = manifest.get("distribution")
    if isinstance(distribution, dict):
        _compare_generated_manifest_array(
            manifest,
            distribution,
            "default_packaging",
            "generated distribution.default_packaging",
            package_id,
            diagnostics,
        )


def plugin_validate_feature_provider_manifest_supported_targets_projection(
    manifest: dict[str, Any],
    feature_extension: dict[str, Any],
    package_id: str,
    diagnostics: list[str],
) -> None:
    modules = feature_extension.get("modules")
    if not isinstance(modules, list) or len(modules) != 1 or not isinstance(modules[0], dict):
        return
    manifest_targets = _valid_string_array(manifest.get("supported_targets"))
    module_targets = _valid_string_array(modules[0].get("target_modes"))
    if manifest_targets is None or module_targets is None or manifest_targets == module_targets:
        return
    diagnostics.append(
        f"plugin {package_id} generated manifest.supported_targets must match "
        "generated feature_extensions[0].modules[0].target_modes"
    )


def plugin_validate_feature_provider_manifest_display_name_projection(
    manifest: dict[str, Any],
    feature_extension: dict[str, Any],
    package_id: str,
    diagnostics: list[str],
) -> None:
    manifest_display_name = _valid_trimmed_string(manifest.get("display_name"))
    extension_display_name = _valid_trimmed_string(feature_extension.get("display_name"))
    if manifest_display_name is None or extension_display_name is None:
        return
    expected_display_name = f"{extension_display_name} Provider"
    if manifest_display_name != expected_display_name:
        diagnostics.append(
            f"plugin {package_id} generated manifest.display_name must equal "
            "generated feature_extensions[0].display_name + Provider"
        )


def _single_feature_extension(manifest: dict[str, Any]) -> dict[str, Any] | None:
    feature_extensions = manifest.get("feature_extensions")
    if not isinstance(feature_extensions, list) or len(feature_extensions) != 1:
        return None
    feature_extension = feature_extensions[0]
    return feature_extension if isinstance(feature_extension, dict) else None


def _compare_generated_manifest_array(
    manifest: dict[str, Any],
    generated_table: dict[str, Any],
    field: str,
    expected_label: str,
    package_id: str,
    diagnostics: list[str],
) -> None:
    manifest_value = _valid_string_array(manifest.get(field))
    generated_value = _valid_string_array(generated_table.get(field))
    if manifest_value is None or generated_value is None:
        return
    if manifest_value != generated_value:
        diagnostics.append(
            f"plugin {package_id} generated manifest.{field} "
            f"must match {expected_label}"
        )


def _valid_trimmed_string(value: object) -> str | None:
    if not isinstance(value, str) or not value.strip() or value.strip() != value:
        return None
    return value


def _valid_string_array(value: object) -> list[str] | None:
    if not isinstance(value, list) or not value:
        return None
    for item in value:
        if not isinstance(item, str) or not item.strip() or item.strip() != item:
            return None
    return value
