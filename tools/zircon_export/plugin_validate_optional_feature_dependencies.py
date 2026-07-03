"""Root optional feature and feature extension dependency validation."""

from __future__ import annotations

from pathlib import Path
from collections.abc import Callable
from typing import Any

from .native_build_workspace import read_toml
from .plugin_validate_common import plugin_validate_trimmed_string
from .plugin_validate_dependency_capabilities import plugin_validate_dependency_capability_target_index
from .plugin_validate_optional_feature_dependency_capabilities import validate_plugin_optional_feature_dependency_capability_gate

Diagnostics = list[str]
Manifest = dict[str, Any]
DependencyRow = tuple[str, str, bool]
PrimaryTarget = tuple[str, set[str] | None, str, str]
KnownFieldsValidator = Callable[[Manifest, str, Diagnostics], None]

PLUGIN_VALIDATE_OPTIONAL_FEATURE_DEPENDENCY_FIELDS = frozenset("capability plugin_id primary".split())


def validate_plugin_optional_feature_dependencies(
    *, plugin_manifest_path: Path | None, plugin_root: Path | None,
    package_id: str, diagnostics: Diagnostics,
) -> None:
    if plugin_manifest_path is None:
        return
    manifest = read_toml(plugin_manifest_path, diagnostics)
    if manifest is None:
        return
    package_capabilities = plugin_validate_package_capabilities(manifest)
    capability_targets = plugin_validate_dependency_capability_target_index(plugin_root, diagnostics)
    optional_features = manifest.get("optional_features")
    if optional_features is not None:
        validate_plugin_optional_feature_dependency_list(
            optional_features, package_id, package_capabilities, capability_targets, diagnostics,
        )
    validate_plugin_feature_extension_dependencies(manifest, package_id, package_capabilities, capability_targets, diagnostics)


def validate_plugin_optional_feature_dependency_list(
    optional_features: Any, package_id: str, package_capabilities: set[str],
    capability_targets: dict[str, set[str]], diagnostics: Diagnostics,
) -> None:
    label = f"plugin {package_id} optional_features"
    if not isinstance(optional_features, list):
        diagnostics.append(f"{label} must be an array")
        return
    primary_target: PrimaryTarget = (
        package_id,
        package_capabilities,
        "primary dependency plugin_id must match package id",
        "primary dependency capability must be a package capability",
    )
    for index, feature in enumerate(optional_features):
        feature_label = f"{label}[{index}]"
        if not isinstance(feature, dict):
            diagnostics.append(f"{feature_label} must be a table")
            continue
        validate_plugin_optional_feature_dependency_rows(
            feature, feature_label, primary_target, capability_targets, diagnostics,
        )


def validate_plugin_feature_extension_dependencies(
    manifest: Manifest, package_id: str, package_capabilities: set[str],
    capability_targets: dict[str, set[str]], diagnostics: Diagnostics,
) -> None:
    feature_extensions = manifest.get("feature_extensions")
    if not isinstance(feature_extensions, list):
        return
    label = f"plugin {package_id} feature_extensions"
    for index, feature in enumerate(feature_extensions):
        feature_label = f"{label}[{index}]"
        if not isinstance(feature, dict):
            continue
        owner_plugin_id = plugin_validate_feature_extension_owner_id(feature)
        if owner_plugin_id is None:
            continue
        owner_capabilities = package_capabilities if owner_plugin_id == package_id else capability_targets.get(owner_plugin_id)
        primary_target: PrimaryTarget = (
            owner_plugin_id,
            owner_capabilities,
            "primary dependency plugin_id must match owner plugin id",
            "primary dependency capability must be an owner plugin capability",
        )
        validate_plugin_optional_feature_dependency_rows(
            feature, feature_label, primary_target, capability_targets, diagnostics,
        )


def validate_plugin_optional_feature_dependency_rows(feature: Manifest, feature_label: str, primary_target: PrimaryTarget, capability_targets: dict[str, set[str]], diagnostics: Diagnostics) -> None:
    validate_plugin_optional_feature_dependency_rows_at_label(
        feature.get("dependencies"), f"{feature_label}.dependencies",
        primary_target, capability_targets, diagnostics,
    )


def validate_plugin_optional_feature_dependency_rows_at_label(
    dependencies: Any, label: str, primary_target: PrimaryTarget,
    capability_targets: dict[str, set[str]], diagnostics: Diagnostics,
    *, known_fields_validator: KnownFieldsValidator | None = None,
    validate_capability_targets: bool = True,
) -> None:
    if dependencies is None:
        diagnostics.append(f"{label} is required")
        return
    if not isinstance(dependencies, list):
        diagnostics.append(f"{label} must be an array")
        return
    if not dependencies:
        diagnostics.append(f"{label} should declare at least one dependency")
        return

    primary_count = 0
    seen: dict[tuple[str, str], int] = {}
    for index, dependency in enumerate(dependencies):
        dependency_label = f"{label}[{index}]"
        row = validate_plugin_optional_feature_dependency_row(
            dependency, dependency_label, diagnostics,
            known_fields_validator=known_fields_validator,
        )
        if row is None:
            continue
        plugin_id, capability, primary = row
        if validate_capability_targets:
            validate_plugin_optional_feature_dependency_capability_gate(
                plugin_id, capability, dependency_label, capability_targets,
                diagnostics,
            )
        identity = plugin_validate_optional_feature_dependency_identity(plugin_id, capability)
        previous_index = seen.get(identity)
        if previous_index is not None:
            diagnostics.append(f"{dependency_label} duplicates dependency row {previous_index}")
        else:
            seen[identity] = index
        if primary:
            primary_count += 1
            validate_plugin_optional_feature_primary_dependency(
                dependency_label, plugin_id, capability, primary_target, diagnostics,
            )
    if primary_count != 1:
        diagnostics.append(f"{label} should declare exactly one primary dependency")


def validate_plugin_optional_feature_dependency_row(
    dependency: Any, dependency_label: str, diagnostics: Diagnostics,
    *, known_fields_validator: KnownFieldsValidator | None = None,
) -> DependencyRow | None:
    if not isinstance(dependency, dict):
        diagnostics.append(f"{dependency_label} must be a table")
        return None
    if known_fields_validator is None:
        known_fields_validator = validate_plugin_optional_feature_dependency_known_fields
    known_fields_validator(dependency, dependency_label, diagnostics)
    plugin_id = plugin_validate_trimmed_string(dependency, "plugin_id", f"{dependency_label}.plugin_id", diagnostics)
    capability = plugin_validate_trimmed_string(dependency, "capability", f"{dependency_label}.capability", diagnostics)
    primary = dependency.get("primary")
    if type(primary) is not bool:
        diagnostics.append(f"{dependency_label}.primary must be a bool")
        return None
    if plugin_id is None or capability is None:
        return None
    return (plugin_id, capability, primary)


def validate_plugin_optional_feature_dependency_known_fields(dependency: Manifest, dependency_label: str, diagnostics: Diagnostics) -> None:
    for field_name in sorted(dependency):
        if field_name not in PLUGIN_VALIDATE_OPTIONAL_FEATURE_DEPENDENCY_FIELDS:
            diagnostics.append(f"{dependency_label}.{field_name} is not a known optional feature dependency field")


def validate_plugin_optional_feature_primary_dependency(
    dependency_label: str, plugin_id: str, capability: str,
    primary_target: PrimaryTarget, diagnostics: Diagnostics,
) -> None:
    target_plugin_id, target_capabilities, plugin_id_message, capability_message = primary_target
    if plugin_id != target_plugin_id:
        diagnostics.append(f"{dependency_label} {plugin_id_message} {target_plugin_id}")
    if target_capabilities is not None and capability not in target_capabilities:
        diagnostics.append(f"{dependency_label} {capability_message}")


def plugin_validate_optional_feature_dependency_identity(plugin_id: str, capability: str) -> tuple[str, str]:
    return (plugin_id, capability)


def plugin_validate_package_capabilities(manifest: Manifest) -> set[str]:
    values = manifest.get("capabilities")
    if not isinstance(values, list):
        return set()
    return {value for value in values if isinstance(value, str) and value.strip() and value.strip() == value}


def plugin_validate_feature_extension_owner_id(feature: Manifest) -> str | None:
    value = feature.get("owner_plugin_id")
    if not isinstance(value, str) or not value.strip() or value.strip() != value:
        return None
    return value
