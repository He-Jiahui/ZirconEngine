"""Feature-provider package projection validation."""

from __future__ import annotations

import tomllib
from pathlib import Path
from typing import Any

from .native_build import read_toml
from .plugin_validate_common import (
    PLUGIN_VALIDATE_FEATURE_SOURCE,
    plugin_validate_int,
    plugin_validate_manifest_target_id,
    plugin_validate_optional_trimmed_string,
    plugin_validate_selected_feature,
    plugin_validate_string_array,
    plugin_validate_trimmed_string,
)


def validate_plugin_feature_provider_package_projection(
    *,
    plugin_manifest_path: Path | None,
    package_manifest_text: str | None,
    requested_plugin_id: str,
    package_id: str,
    diagnostics: list[str],
) -> None:
    if package_manifest_text is None:
        return
    generated_manifest = plugin_validate_generated_package_manifest(
        package_manifest_text,
        package_id,
        diagnostics,
    )
    if generated_manifest is None:
        return
    if generated_manifest.get("id") != package_id:
        diagnostics.append(f"plugin {package_id} generated id must equal {package_id}")
    if generated_manifest.get("package_kind") != PLUGIN_VALIDATE_FEATURE_SOURCE:
        diagnostics.append(
            f"plugin {package_id} generated package_kind must equal "
            f"{PLUGIN_VALIDATE_FEATURE_SOURCE}"
        )
    generated_distribution = generated_manifest.get("distribution")
    if not isinstance(generated_distribution, dict):
        diagnostics.append(f"plugin {package_id} generated distribution must be a table")
        generated_distribution = None

    feature_extensions = generated_manifest.get("feature_extensions")
    if not isinstance(feature_extensions, list) or len(feature_extensions) != 1:
        diagnostics.append(
            f"plugin {package_id} generated feature_extensions must contain "
            "exactly one table"
        )
        return
    feature_extension = feature_extensions[0]
    if not isinstance(feature_extension, dict):
        diagnostics.append(
            f"plugin {package_id} generated feature_extensions[0] must be a table"
        )
        return
    validate_plugin_feature_extension_projection(
        plugin_manifest_path=plugin_manifest_path,
        generated_distribution=generated_distribution,
        generated_feature=feature_extension,
        requested_plugin_id=requested_plugin_id,
        package_id=package_id,
        diagnostics=diagnostics,
    )


def plugin_validate_generated_package_manifest(
    package_manifest_text: str,
    package_id: str,
    diagnostics: list[str],
) -> dict[str, Any] | None:
    try:
        manifest = tomllib.loads(package_manifest_text)
    except tomllib.TOMLDecodeError as error:
        diagnostics.append(
            f"plugin {package_id} generated package manifest is invalid TOML: {error}"
        )
        return None
    if not isinstance(manifest, dict):
        diagnostics.append(f"plugin {package_id} generated package manifest must be a table")
        return None
    return manifest


def validate_plugin_feature_extension_projection(
    *,
    plugin_manifest_path: Path | None,
    generated_distribution: dict[str, Any] | None,
    generated_feature: dict[str, Any],
    requested_plugin_id: str,
    package_id: str,
    diagnostics: list[str],
) -> None:
    if plugin_manifest_path is None:
        return
    owner_manifest = read_toml(plugin_manifest_path, diagnostics)
    if owner_manifest is None:
        return
    owner_id = plugin_validate_manifest_target_id(
        owner_manifest,
        f"plugin {package_id} owner manifest id",
        diagnostics,
    )
    selected_feature = plugin_validate_selected_feature(
        owner_manifest,
        requested_plugin_id,
        package_id,
    )
    if selected_feature is None:
        diagnostics.append(
            f"plugin {package_id} generated feature_extension was not found "
            "in owner manifest"
        )
        return
    feature_id = plugin_validate_manifest_target_id(
        selected_feature,
        f"plugin {package_id} selected feature id",
        diagnostics,
    )
    if feature_id is not None and generated_feature.get("id") != feature_id:
        diagnostics.append(
            f"plugin {package_id} generated feature_extensions[0].id "
            f"must equal {feature_id}"
        )
    if owner_id is not None and generated_feature.get("owner_plugin_id") != owner_id:
        diagnostics.append(
            f"plugin {package_id} generated feature_extensions[0].owner_plugin_id "
            f"must equal {owner_id}"
        )
    validate_plugin_feature_provider_distribution_projection(
        selected_feature=selected_feature,
        generated_distribution=generated_distribution,
        package_id=package_id,
        diagnostics=diagnostics,
    )
    validate_plugin_feature_provider_capabilities(
        selected_feature=selected_feature,
        generated_feature=generated_feature,
        package_id=package_id,
        diagnostics=diagnostics,
    )
    validate_plugin_feature_provider_dependencies(
        selected_feature=selected_feature,
        generated_feature=generated_feature,
        package_id=package_id,
        diagnostics=diagnostics,
    )


def validate_plugin_feature_provider_distribution_projection(
    *,
    selected_feature: dict[str, Any],
    generated_distribution: dict[str, Any] | None,
    package_id: str,
    diagnostics: list[str],
) -> None:
    owner_distribution = selected_feature.get("distribution")
    if not isinstance(owner_distribution, dict):
        diagnostics.append(
            f"plugin {package_id} optional feature distribution must be a table"
        )
        return
    if generated_distribution is None:
        return
    for field in ("forms", "default_packaging"):
        plugin_validate_compare_string_array_projection(
            owner_table=owner_distribution,
            generated_table=generated_distribution,
            field=field,
            package_id=package_id,
            diagnostics=diagnostics,
        )
    plugin_validate_compare_int_projection(
        owner_table=owner_distribution,
        generated_table=generated_distribution,
        field="abi_version",
        package_id=package_id,
        diagnostics=diagnostics,
    )
    for field in ("engine_compat", "dist_crate", "descriptor_symbol"):
        plugin_validate_compare_required_string_projection(
            owner_table=owner_distribution,
            generated_table=generated_distribution,
            field=field,
            package_id=package_id,
            diagnostics=diagnostics,
        )
    for field in ("runtime_entry", "editor_entry"):
        plugin_validate_compare_optional_string_projection(
            owner_table=owner_distribution,
            generated_table=generated_distribution,
            field=field,
            package_id=package_id,
            diagnostics=diagnostics,
        )
    plugin_validate_compare_optional_string_array_projection(
        owner_table=owner_distribution,
        generated_table=generated_distribution,
        field="assets",
        package_id=package_id,
        diagnostics=diagnostics,
    )


def plugin_validate_compare_string_array_projection(
    *,
    owner_table: dict[str, Any],
    generated_table: dict[str, Any],
    field: str,
    package_id: str,
    diagnostics: list[str],
) -> None:
    owner_label = f"plugin {package_id} optional feature distribution.{field}"
    generated_label = f"plugin {package_id} generated distribution.{field}"
    diagnostic_count = len(diagnostics)
    owner_value = plugin_validate_string_array(
        owner_table,
        field,
        owner_label,
        diagnostics,
    )
    generated_value = plugin_validate_string_array(
        generated_table,
        field,
        generated_label,
        diagnostics,
    )
    if len(diagnostics) != diagnostic_count:
        return
    if generated_value != owner_value:
        diagnostics.append(
            f"{generated_label} must equal owner optional feature distribution.{field}"
        )


def plugin_validate_compare_int_projection(
    *,
    owner_table: dict[str, Any],
    generated_table: dict[str, Any],
    field: str,
    package_id: str,
    diagnostics: list[str],
) -> None:
    owner_label = f"plugin {package_id} optional feature distribution.{field}"
    generated_label = f"plugin {package_id} generated distribution.{field}"
    diagnostic_count = len(diagnostics)
    owner_value = plugin_validate_int(owner_table, field, owner_label, diagnostics)
    generated_value = plugin_validate_int(
        generated_table,
        field,
        generated_label,
        diagnostics,
    )
    if len(diagnostics) != diagnostic_count:
        return
    if generated_value != owner_value:
        diagnostics.append(
            f"{generated_label} must equal owner optional feature distribution.{field}"
        )


def plugin_validate_compare_required_string_projection(
    *,
    owner_table: dict[str, Any],
    generated_table: dict[str, Any],
    field: str,
    package_id: str,
    diagnostics: list[str],
) -> None:
    owner_label = f"plugin {package_id} optional feature distribution.{field}"
    generated_label = f"plugin {package_id} generated distribution.{field}"
    diagnostic_count = len(diagnostics)
    owner_value = plugin_validate_trimmed_string(
        owner_table,
        field,
        owner_label,
        diagnostics,
    )
    generated_value = plugin_validate_trimmed_string(
        generated_table,
        field,
        generated_label,
        diagnostics,
    )
    if len(diagnostics) != diagnostic_count:
        return
    if generated_value != owner_value:
        diagnostics.append(
            f"{generated_label} must equal owner optional feature distribution.{field}"
        )


def plugin_validate_compare_optional_string_projection(
    *,
    owner_table: dict[str, Any],
    generated_table: dict[str, Any],
    field: str,
    package_id: str,
    diagnostics: list[str],
) -> None:
    owner_label = f"plugin {package_id} optional feature distribution.{field}"
    generated_label = f"plugin {package_id} generated distribution.{field}"
    owner_has_field = field in owner_table
    generated_has_field = field in generated_table
    diagnostic_count = len(diagnostics)
    owner_value = plugin_validate_optional_trimmed_string(
        owner_table,
        field,
        owner_label,
        diagnostics,
    )
    generated_value = plugin_validate_optional_trimmed_string(
        generated_table,
        field,
        generated_label,
        diagnostics,
    )
    if len(diagnostics) != diagnostic_count:
        return
    owner_field = f"owner optional feature distribution.{field}"
    if owner_has_field and not generated_has_field:
        diagnostics.append(f"{generated_label} is required when {owner_field} is present")
        return
    if not owner_has_field and generated_has_field:
        diagnostics.append(f"{generated_label} must be omitted when {owner_field} is absent")
        return
    if owner_has_field and generated_value != owner_value:
        diagnostics.append(f"{generated_label} must equal {owner_field}")


def plugin_validate_compare_optional_string_array_projection(
    *,
    owner_table: dict[str, Any],
    generated_table: dict[str, Any],
    field: str,
    package_id: str,
    diagnostics: list[str],
) -> None:
    owner_label = f"plugin {package_id} optional feature distribution.{field}"
    generated_label = f"plugin {package_id} generated distribution.{field}"
    owner_has_field = field in owner_table
    generated_has_field = field in generated_table
    diagnostic_count = len(diagnostics)
    owner_value = (
        plugin_validate_string_array(owner_table, field, owner_label, diagnostics)
        if owner_has_field
        else None
    )
    generated_value = (
        plugin_validate_string_array(
            generated_table,
            field,
            generated_label,
            diagnostics,
        )
        if generated_has_field
        else None
    )
    if len(diagnostics) != diagnostic_count:
        return
    owner_field = f"owner optional feature distribution.{field}"
    if owner_has_field and not generated_has_field:
        diagnostics.append(f"{generated_label} is required when {owner_field} is present")
        return
    if not owner_has_field and generated_has_field:
        diagnostics.append(f"{generated_label} must be omitted when {owner_field} is absent")
        return
    if owner_has_field and generated_value != owner_value:
        diagnostics.append(f"{generated_label} must equal {owner_field}")


def validate_plugin_feature_provider_capabilities(
    *,
    selected_feature: dict[str, Any],
    generated_feature: dict[str, Any],
    package_id: str,
    diagnostics: list[str],
) -> None:
    diagnostic_count = len(diagnostics)
    expected_capabilities = plugin_validate_string_array(
        selected_feature,
        "capabilities",
        f"plugin {package_id} optional feature capabilities",
        diagnostics,
    )
    if len(diagnostics) != diagnostic_count or expected_capabilities is None:
        return
    generated_capabilities = plugin_validate_string_array(
        generated_feature,
        "capabilities",
        f"plugin {package_id} generated feature_extensions[0].capabilities",
        diagnostics,
    )
    if len(diagnostics) != diagnostic_count or generated_capabilities is None:
        return
    if generated_capabilities != expected_capabilities:
        diagnostics.append(
            f"plugin {package_id} generated feature_extensions[0].capabilities "
            "must match owner optional feature capabilities"
        )


def validate_plugin_feature_provider_dependencies(
    *,
    selected_feature: dict[str, Any],
    generated_feature: dict[str, Any],
    package_id: str,
    diagnostics: list[str],
) -> None:
    diagnostic_count = len(diagnostics)
    expected_dependencies = plugin_validate_feature_dependencies(
        selected_feature.get("dependencies", []),
        f"plugin {package_id} optional feature dependencies",
        diagnostics,
    )
    if len(diagnostics) != diagnostic_count:
        return
    generated_dependencies = plugin_validate_feature_dependencies(
        generated_feature.get("dependencies", []),
        f"plugin {package_id} generated feature_extensions[0].dependencies",
        diagnostics,
    )
    if len(diagnostics) != diagnostic_count:
        return
    if generated_dependencies != expected_dependencies:
        diagnostics.append(
            f"plugin {package_id} generated feature_extensions[0].dependencies "
            "must match owner optional feature dependencies"
        )


def plugin_validate_feature_dependencies(
    dependencies: Any,
    label: str,
    diagnostics: list[str],
) -> list[dict[str, object]]:
    if dependencies in (None, []):
        return []
    if not isinstance(dependencies, list):
        diagnostics.append(f"{label} must be an array")
        return []
    parsed: list[dict[str, object]] = []
    for index, dependency in enumerate(dependencies):
        dependency_label = f"{label}[{index}]"
        if not isinstance(dependency, dict):
            diagnostics.append(f"{dependency_label} must be a table")
            continue
        plugin_id = plugin_validate_trimmed_string(
            dependency,
            "plugin_id",
            f"{dependency_label}.plugin_id",
            diagnostics,
        )
        capability = plugin_validate_trimmed_string(
            dependency,
            "capability",
            f"{dependency_label}.capability",
            diagnostics,
        )
        primary = dependency.get("primary", False)
        if "primary" in dependency and not isinstance(primary, bool):
            diagnostics.append(f"{dependency_label}.primary must be a bool")
            continue
        if plugin_id is None or capability is None:
            continue
        parsed.append(
            {
                "plugin_id": plugin_id,
                "capability": capability,
                "primary": primary is True,
            }
        )
    return parsed
