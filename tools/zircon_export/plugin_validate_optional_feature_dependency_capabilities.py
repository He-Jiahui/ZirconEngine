"""Optional feature dependency capability resolution checks."""

from __future__ import annotations

from .plugin_validate_dependency_capabilities import (
    CapabilityIndex,
    plugin_validate_dependency_capability_is_host_owned,
)

Diagnostics = list[str]


def validate_plugin_optional_feature_dependency_capability_gate(
    plugin_id: str,
    capability: str,
    dependency_label: str,
    capability_targets: CapabilityIndex,
    diagnostics: Diagnostics,
) -> None:
    target_capabilities = capability_targets.get(plugin_id)
    if target_capabilities is not None:
        if capability in target_capabilities:
            return
        diagnostics.append(
            f"{dependency_label}.capability {capability} should be declared by "
            "the referenced static plugin package or one of its feature rows"
        )
        return
    if plugin_validate_dependency_capability_is_host_owned(capability):
        return
    diagnostics.append(
        f"{dependency_label}.capability {capability} references no static "
        "plugin package and should use a runtime.module.* or runtime.capability.* "
        "host namespace"
    )
