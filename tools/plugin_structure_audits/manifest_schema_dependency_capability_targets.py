from __future__ import annotations

from typing import Any, Iterable

from .manifest_schema_asset_importer_capability_gates import (
    collect_declared_capability_values,
    is_non_empty_trimmed_string,
    table_rows,
)


ManifestEntry = tuple[str, dict[str, Any]]
CapabilityTargetIndex = dict[str, set[str]]


def collect_dependency_capability_target_violations(
    manifests: Iterable[ManifestEntry],
    violations: list[str],
) -> None:
    manifest_entries = list(manifests)
    capability_targets = dependency_capability_target_index(manifest_entries)
    for display_path, manifest in manifest_entries:
        collect_manifest_dependency_capability_target_violations(
            display_path,
            manifest,
            capability_targets,
            violations,
        )


def dependency_capability_target_index(
    manifests: Iterable[ManifestEntry],
) -> CapabilityTargetIndex:
    targets: CapabilityTargetIndex = {}
    for _display_path, manifest in manifests:
        package_id = manifest.get("id")
        if not is_non_empty_trimmed_string(package_id):
            continue
        capabilities = targets.setdefault(package_id, set())
        collect_declared_capability_values(manifest, capabilities)
        for feature in table_rows(manifest.get("optional_features")):
            collect_declared_capability_values(feature, capabilities)
        for extension in table_rows(manifest.get("feature_extensions")):
            collect_declared_capability_values(extension, capabilities)
    return targets


def collect_manifest_dependency_capability_target_violations(
    display_path: str,
    manifest: dict[str, Any],
    capability_targets: CapabilityTargetIndex,
    violations: list[str],
) -> None:
    dependencies = manifest.get("dependencies")
    if isinstance(dependencies, list):
        for dependency_index, dependency in enumerate(dependencies):
            if not isinstance(dependency, dict):
                continue
            collect_single_dependency_capability_target_violations(
                display_path,
                f"dependencies[{dependency_index}]",
                dependency,
                capability_targets,
                violations,
            )
    collect_feature_dependency_capability_target_violations(
        display_path,
        "optional_features",
        manifest.get("optional_features"),
        capability_targets,
        violations,
    )
    collect_feature_dependency_capability_target_violations(
        display_path,
        "feature_extensions",
        manifest.get("feature_extensions"),
        capability_targets,
        violations,
    )


def collect_single_dependency_capability_target_violations(
    display_path: str,
    dependency_label: str,
    dependency: dict[str, Any],
    capability_targets: CapabilityTargetIndex,
    violations: list[str],
) -> None:
    capability = dependency.get("capability")
    if not is_non_empty_trimmed_string(capability):
        return
    dependency_id = dependency.get("id")
    if not is_non_empty_trimmed_string(dependency_id):
        return
    collect_dependency_capability_target_violation(
        display_path,
        dependency_label,
        dependency_id,
        capability,
        capability_targets,
        violations,
    )


def collect_feature_dependency_capability_target_violations(
    display_path: str,
    table_label: str,
    tables: object,
    capability_targets: CapabilityTargetIndex,
    violations: list[str],
) -> None:
    if not isinstance(tables, list):
        return
    for table_index, table in enumerate(tables):
        if not isinstance(table, dict):
            continue
        dependencies = table.get("dependencies")
        if not isinstance(dependencies, list):
            continue
        for dependency_index, dependency in enumerate(dependencies):
            if not isinstance(dependency, dict):
                continue
            collect_single_feature_dependency_capability_target_violations(
                display_path,
                f"{table_label}[{table_index}].dependencies[{dependency_index}]",
                dependency,
                capability_targets,
                violations,
            )


def collect_single_feature_dependency_capability_target_violations(
    display_path: str,
    dependency_label: str,
    dependency: dict[str, Any],
    capability_targets: CapabilityTargetIndex,
    violations: list[str],
) -> None:
    capability = dependency.get("capability")
    if not is_non_empty_trimmed_string(capability):
        return
    plugin_id = dependency.get("plugin_id")
    if not is_non_empty_trimmed_string(plugin_id):
        return
    collect_dependency_capability_target_violation(
        display_path,
        dependency_label,
        plugin_id,
        capability,
        capability_targets,
        violations,
    )


def collect_dependency_capability_target_violation(
    display_path: str,
    dependency_label: str,
    target_id: str,
    capability: str,
    capability_targets: CapabilityTargetIndex,
    violations: list[str],
) -> None:
    target_capabilities = capability_targets.get(target_id)
    if target_capabilities is not None:
        if capability in target_capabilities:
            return
        violations.append(
            f"{display_path}: {dependency_label}.capability {capability} "
            "should be declared by the referenced static plugin package or "
            "one of its feature rows"
        )
        return
    if dependency_capability_is_host_owned(capability):
        return
    violations.append(
        f"{display_path}: {dependency_label}.capability {capability} "
        "references no static plugin package and should use a "
        "runtime.module.* or runtime.capability.* host namespace"
    )


def dependency_capability_is_host_owned(capability: str) -> bool:
    return capability.startswith("runtime.module.") or capability.startswith(
        "runtime.capability."
    )
