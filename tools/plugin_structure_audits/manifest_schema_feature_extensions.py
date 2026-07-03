from __future__ import annotations

from typing import Any

from .manifest_schema import (
    PACKAGING_VALUES,
    REQUIRED_FEATURE_FIELDS,
    collect_allowed_string_array_values,
    collect_feature_dependency_duplicate_identity_violations,
    collect_feature_dependency_primary_count_violation,
    collect_feature_dependency_primary_target_violations,
    collect_feature_dependency_schema_violations,
    collect_module_schema_violations,
    collect_optional_trimmed_string_field_violation,
    collect_required_field_violation,
    is_non_empty_trimmed_string,
    optional_feature_dependency_primary_target,
)
from .manifest_schema_distribution import collect_feature_distribution_schema_violations


FEATURE_EXTENSION_FIELDS = {
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


def collect_feature_extensions_schema_violations(
    display_path: str,
    manifest: dict[str, Any],
    feature_extensions: object,
    violations: list[str],
    *,
    module_seen_names: dict[str, str] | None = None,
    module_supported_targets: set[str] | None = None,
) -> None:
    if not isinstance(feature_extensions, list):
        violations.append(f"{display_path}: feature_extensions must be an array of tables")
        return
    if not feature_extensions:
        violations.append(
            f"{display_path}: feature_extensions must not be empty when declared"
        )
        return
    seen_ids: dict[str, int] = {}
    for feature_index, feature in enumerate(feature_extensions):
        if not isinstance(feature, dict):
            violations.append(
                f"{display_path}: feature_extensions[{feature_index}] must be a table"
            )
            continue
        collect_feature_extension_schema_violations(
            display_path,
            feature_index,
            feature,
            violations,
        )
        collect_feature_extension_identity_violations(
            display_path,
            feature_index,
            feature,
            seen_ids,
            violations,
        )
        collect_feature_extension_capability_violations(
            display_path,
            feature_index,
            feature,
            violations,
        )
        feature_dependencies = feature.get("dependencies")
        if feature_dependencies is None:
            violations.append(
                f"{display_path}: missing feature_extensions[{feature_index}].dependencies"
            )
        elif not isinstance(feature_dependencies, list):
            violations.append(
                f"{display_path}: feature_extensions[{feature_index}].dependencies "
                "must be an array of tables"
            )
        elif not feature_dependencies:
            violations.append(
                f"{display_path}: feature_extensions[{feature_index}].dependencies "
                "should declare at least one dependency"
            )
        else:
            for dependency_index, dependency in enumerate(feature_dependencies):
                collect_feature_dependency_schema_violations(
                    display_path,
                    f"feature_extensions[{feature_index}]"
                    f".dependencies[{dependency_index}]",
                    dependency,
                    violations,
                )
            collect_feature_dependency_primary_count_violation(
                display_path,
                f"feature_extensions[{feature_index}].dependencies",
                feature_dependencies,
                violations,
            )
            collect_feature_dependency_duplicate_identity_violations(
                display_path,
                f"feature_extensions[{feature_index}].dependencies",
                feature_dependencies,
                violations,
            )
            collect_feature_extension_dependency_primary_target_violations(
                display_path,
                feature_index,
                manifest,
                feature,
                feature_dependencies,
                violations,
            )
        feature_modules = feature.get("modules")
        if feature_modules is not None:
            collect_feature_extension_modules_schema_violations(
                display_path,
                feature_index,
                feature,
                feature_modules,
                violations,
                module_seen_names=module_seen_names,
                module_supported_targets=module_supported_targets,
            )
        feature_distribution = feature.get("distribution")
        if feature_distribution is not None:
            collect_feature_distribution_schema_violations(
                display_path,
                f"feature_extensions[{feature_index}].distribution",
                feature_distribution,
                violations,
            )


def collect_feature_extension_schema_violations(
    display_path: str,
    feature_index: int,
    feature: dict[str, Any],
    violations: list[str],
) -> None:
    field_label = f"feature_extensions[{feature_index}]"
    for field in sorted(feature):
        if field not in FEATURE_EXTENSION_FIELDS:
            violations.append(
                f"{display_path}: {field_label}.{field} "
                "is not a known feature extension field"
            )
    for field in REQUIRED_FEATURE_FIELDS:
        collect_required_field_violation(
            display_path,
            f"{field_label}.{field}",
            feature,
            violations,
            field_name=field,
        )
    collect_optional_trimmed_string_field_violation(
        display_path,
        f"{field_label}.provider_package_id",
        feature,
        "provider_package_id",
        violations,
    )
    collect_allowed_string_array_values(
        display_path,
        f"{field_label}.default_packaging",
        feature,
        "default_packaging",
        PACKAGING_VALUES,
        violations,
    )


def collect_feature_extension_identity_violations(
    display_path: str,
    feature_index: int,
    feature: dict[str, Any],
    seen_ids: dict[str, int],
    violations: list[str],
) -> None:
    field_label = f"feature_extensions[{feature_index}]"
    owner_plugin_id = feature.get("owner_plugin_id")
    if is_non_empty_trimmed_string(owner_plugin_id):
        collect_feature_extension_owner_package_token_violations(
            display_path,
            f"{field_label}.owner_plugin_id",
            owner_plugin_id,
            violations,
        )
    feature_id = feature.get("id")
    if not is_non_empty_trimmed_string(feature_id):
        return
    collect_feature_extension_dot_namespace_violations(
        display_path,
        f"{field_label}.id",
        feature_id,
        violations,
    )
    if is_non_empty_trimmed_string(owner_plugin_id):
        expected_prefix = f"{owner_plugin_id}."
        if not feature_id.startswith(expected_prefix):
            violations.append(
                f"{display_path}: {field_label}.id {feature_id} "
                f"should stay under owner namespace {expected_prefix}"
            )
    previous_index = seen_ids.get(feature_id)
    if previous_index is not None:
        violations.append(
            f"{display_path}: {field_label}.id {feature_id} "
            f"duplicates feature extension id feature_extensions[{previous_index}]"
        )
        return
    seen_ids[feature_id] = feature_index


def collect_feature_extension_dot_namespace_violations(
    display_path: str,
    label: str,
    value: str,
    violations: list[str],
) -> None:
    segments = value.split(".")
    if len(segments) < 2:
        violations.append(
            f"{display_path}: {label} {value} "
            "should use owner.feature dot namespace form"
        )
    if any(not segment for segment in segments):
        violations.append(
            f"{display_path}: {label} {value} "
            "should not contain empty namespace segments"
        )
    if not all(
        char.isascii() and (char.islower() or char.isdigit() or char in {"_", "."})
        for char in value
    ):
        violations.append(
            f"{display_path}: {label} {value} should contain only "
            "lowercase ASCII letters, digits, underscores, and dots"
        )


def collect_feature_extension_owner_package_token_violations(
    display_path: str,
    label: str,
    value: str,
    violations: list[str],
) -> None:
    if not ("a" <= value[0] <= "z"):
        violations.append(
            f"{display_path}: {label} {value} "
            "should start with a lowercase ASCII letter"
        )
    if not all(
        char.isascii() and (char.islower() or char.isdigit() or char == "_")
        for char in value
    ):
        violations.append(
            f"{display_path}: {label} {value} should contain only "
            "lowercase ASCII letters, digits, and underscores"
        )
    if value.endswith("_") or "__" in value:
        violations.append(
            f"{display_path}: {label} {value} "
            "should not end with an underscore or contain repeated underscores"
        )


def collect_feature_extension_capability_violations(
    display_path: str,
    feature_index: int,
    feature: dict[str, Any],
    violations: list[str],
) -> None:
    capabilities = feature.get("capabilities")
    if not isinstance(capabilities, list) or not capabilities:
        return
    seen: dict[str, int] = {}
    for capability_index, capability in enumerate(capabilities):
        if not is_non_empty_trimmed_string(capability):
            continue
        item_label = f"feature_extensions[{feature_index}].capabilities[{capability_index}]"
        collect_feature_extension_capability_namespace_violations(
            display_path,
            item_label,
            capability,
            violations,
        )
        previous_index = seen.get(capability)
        if previous_index is not None:
            violations.append(
                f"{display_path}: {item_label} {capability} "
                f"duplicates capabilities capabilities[{previous_index}]"
            )
            continue
        seen[capability] = capability_index


def collect_feature_extension_capability_namespace_violations(
    display_path: str,
    label: str,
    value: str,
    violations: list[str],
) -> None:
    segments = value.split(".")
    if len(segments) < 2:
        violations.append(
            f"{display_path}: {label} {value} "
            "should use at least two dot-separated namespace segments"
        )
    if any(not segment for segment in segments):
        violations.append(
            f"{display_path}: {label} {value} "
            "should not contain empty namespace segments"
        )
    if not all(
        char.isascii() and (char.islower() or char.isdigit() or char in {"_", "."})
        for char in value
    ):
        violations.append(
            f"{display_path}: {label} {value} should contain only "
            "lowercase ASCII letters, digits, underscores, and dots"
        )


def collect_feature_extension_dependency_primary_target_violations(
    display_path: str,
    feature_index: int,
    manifest: dict[str, Any],
    feature: dict[str, Any],
    dependencies: list[object],
    violations: list[str],
) -> None:
    owner_plugin_id = feature.get("owner_plugin_id")
    if not is_non_empty_trimmed_string(owner_plugin_id):
        return
    package_target = optional_feature_dependency_primary_target(manifest)
    owner_capabilities: set[str] | None = None
    if package_target is not None:
        package_id, package_capabilities = package_target
        if owner_plugin_id == package_id:
            owner_capabilities = package_capabilities
    collect_feature_dependency_primary_target_violations(
        display_path,
        f"feature_extensions[{feature_index}].dependencies",
        dependencies,
        owner_plugin_id,
        owner_capabilities,
        "primary dependency plugin_id must match owner plugin id",
        "primary dependency capability must be an owner plugin capability",
        violations,
    )


def collect_feature_extension_modules_schema_violations(
    display_path: str,
    feature_index: int,
    feature: dict[str, Any],
    modules: object,
    violations: list[str],
    *,
    module_seen_names: dict[str, str] | None = None,
    module_supported_targets: set[str] | None = None,
) -> None:
    if not isinstance(modules, list):
        violations.append(
            f"{display_path}: feature_extensions[{feature_index}].modules "
            "must be an array of tables"
        )
        return
    if not modules:
        violations.append(
            f"{display_path}: feature_extensions[{feature_index}].modules "
            "must not be empty when declared"
        )
        return
    feature_id = feature.get("id")
    namespace_id = feature_id if is_non_empty_trimmed_string(feature_id) else None
    for module_index, module in enumerate(modules):
        module_label = f"feature_extensions[{feature_index}].modules[{module_index}]"
        collect_module_schema_violations(
            display_path,
            module_label,
            module,
            violations,
            namespace_id=namespace_id,
            supported_targets=module_supported_targets,
            seen_names=module_seen_names,
            row_identity=module_label,
        )
