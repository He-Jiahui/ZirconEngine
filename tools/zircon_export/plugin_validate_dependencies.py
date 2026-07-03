"""Top-level dependency validation for standalone plugin manifests."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .native_build_workspace import read_toml
from .plugin_validate_common import plugin_validate_trimmed_string
from .plugin_validate_dependency_capabilities import (
    plugin_validate_dependency_capability_target_index,
    validate_plugin_dependency_capability_gate,
)
from .plugin_validate_interfaces import (
    plugin_validate_dependency_interfaces,
    validate_plugin_provided_interfaces,
)

Diagnostics = list[str]
DependencyRow = dict[str, Any]
DependencyIdentity = tuple[str, str]
PLUGIN_VALIDATE_DEPENDENCY_FIELDS = frozenset(("id", "required", "capability", "interfaces"))


def validate_plugin_dependencies(
    *,
    plugin_manifest_path: Path | None,
    plugin_root: Path | None,
    package_id: str,
    diagnostics: Diagnostics,
) -> None:
    if plugin_manifest_path is None:
        return
    manifest = read_toml(plugin_manifest_path, diagnostics)
    if manifest is None:
        return
    validate_plugin_provided_interfaces(manifest, package_id, diagnostics)
    dependencies = manifest.get("dependencies")
    if dependencies is None:
        return
    label = f"plugin {package_id} dependencies"
    if not isinstance(dependencies, list):
        diagnostics.append(f"{label} must be an array")
        return
    if not dependencies:
        diagnostics.append(f"{label} must not be empty when declared")
        return
    capability_targets = plugin_validate_dependency_capability_target_index(plugin_root, diagnostics)
    seen: dict[DependencyIdentity, int] = {}
    for index, dependency in enumerate(dependencies):
        dependency_label = f"{label}[{index}]"
        if not isinstance(dependency, dict):
            diagnostics.append(f"{dependency_label} must be a table")
            continue
        validate_plugin_dependency_known_fields(dependency, dependency_label, diagnostics)
        identity = validate_plugin_dependency_row(dependency, dependency_label, diagnostics)
        validate_plugin_dependency_capability_gate(dependency, dependency_label, capability_targets, diagnostics)
        if identity is None:
            continue
        previous_index = seen.get(identity)
        if previous_index is not None:
            diagnostics.append(
                f"{dependency_label} duplicates dependency row {previous_index}"
            )
            continue
        seen[identity] = index


def validate_plugin_dependency_known_fields(
    dependency: DependencyRow, dependency_label: str, diagnostics: Diagnostics,
) -> None:
    for field in sorted(dependency):
        if field not in PLUGIN_VALIDATE_DEPENDENCY_FIELDS:
            diagnostics.append(f"{dependency_label}.{field} is not a known dependency field")


def validate_plugin_dependency_row(
    dependency: DependencyRow,
    dependency_label: str,
    diagnostics: Diagnostics,
) -> DependencyIdentity | None:
    dependency_id = plugin_validate_trimmed_string(
        dependency, "id", f"{dependency_label}.id", diagnostics
    )
    if type(dependency.get("required")) is not bool:
        diagnostics.append(f"{dependency_label}.required must be a bool")
    return plugin_validate_dependency_row_identity(
        dependency, dependency_label, dependency_id, diagnostics
    )


def plugin_validate_dependency_row_identity(
    dependency: DependencyRow,
    dependency_label: str,
    dependency_id: str | None,
    diagnostics: Diagnostics,
) -> DependencyIdentity | None:
    if "capability" in dependency:
        capability = plugin_validate_trimmed_string(
            dependency, "capability", f"{dependency_label}.capability", diagnostics
        )
        if dependency_id is None or capability is None:
            return None
        return (dependency_id, f"capability:{capability}")

    interfaces = plugin_validate_dependency_interfaces(
        dependency.get("interfaces"), f"{dependency_label}.interfaces", diagnostics
    )
    if dependency_id is None or interfaces is None:
        return None
    return (dependency_id, "interfaces:" + ",".join(interfaces))
