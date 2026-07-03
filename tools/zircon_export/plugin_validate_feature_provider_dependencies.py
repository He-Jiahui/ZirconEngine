"""Feature-provider dependency projection validation."""

from __future__ import annotations

from typing import Any

from .plugin_validate_optional_feature_dependencies import PrimaryTarget, validate_plugin_optional_feature_dependency_rows_at_label

PLUGIN_VALIDATE_FEATURE_PROVIDER_DEPENDENCY_FIELDS = frozenset("capability plugin_id primary".split())
PLUGIN_VALIDATE_FEATURE_PROVIDER_DEPENDENCY_UNKNOWN_FIELD_MESSAGE = "is not a known feature provider dependency field"


def validate_plugin_feature_provider_dependencies(
    *, selected_feature: dict[str, Any], generated_feature: dict[str, Any],
    package_id: str, owner_plugin_id: str | None,
    owner_package_capabilities: set[str] | None, diagnostics: list[str],
) -> None:
    diagnostic_count = len(diagnostics)
    primary_target: PrimaryTarget = (
        owner_plugin_id or "",
        owner_package_capabilities,
        "primary dependency plugin_id must match owner plugin id",
        "primary dependency capability must be an owner plugin capability",
    )
    expected_dependencies = plugin_validate_feature_dependencies(
        selected_feature.get("dependencies", []),
        f"plugin {package_id} optional feature dependencies",
        primary_target,
        diagnostics,
    )
    if len(diagnostics) != diagnostic_count:
        return
    generated_dependencies = plugin_validate_feature_dependencies(
        generated_feature.get("dependencies", []),
        f"plugin {package_id} generated feature_extensions[0].dependencies",
        primary_target,
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
    dependencies: Any, label: str, primary_target: PrimaryTarget,
    diagnostics: list[str],
) -> list[dict[str, object]]:
    diagnostic_count = len(diagnostics)
    validate_plugin_optional_feature_dependency_rows_at_label(
        dependencies,
        label,
        primary_target,
        {},
        diagnostics,
        known_fields_validator=plugin_validate_feature_dependency_known_fields,
        validate_capability_targets=False,
    )
    if len(diagnostics) != diagnostic_count or not isinstance(dependencies, list):
        return []
    parsed: list[dict[str, object]] = []
    for dependency in dependencies:
        if not isinstance(dependency, dict):
            continue
        parsed.append(
            {
                "plugin_id": dependency["plugin_id"],
                "capability": dependency["capability"],
                "primary": dependency["primary"] is True,
            }
        )
    return parsed


def plugin_validate_feature_dependency_known_fields(
    dependency: dict[str, Any],
    dependency_label: str,
    diagnostics: list[str],
) -> None:
    for field_name in sorted(dependency):
        if field_name not in PLUGIN_VALIDATE_FEATURE_PROVIDER_DEPENDENCY_FIELDS:
            diagnostics.append(
                f"{dependency_label}.{field_name} "
                f"{PLUGIN_VALIDATE_FEATURE_PROVIDER_DEPENDENCY_UNKNOWN_FIELD_MESSAGE}"
            )
