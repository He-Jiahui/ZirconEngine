"""Feature-provider distribution projection validation."""

from __future__ import annotations

from typing import Any

from .plugin_validate_feature_provider_projection_compare import (
    plugin_validate_compare_int_projection,
    plugin_validate_compare_required_string_projection,
    plugin_validate_compare_string_array_projection,
)
from .plugin_validate_feature_provider_distribution_schema import validate_plugin_feature_provider_distribution_schema
from .plugin_validate_feature_provider_projection_optional import (
    plugin_validate_compare_optional_string_array_projection,
    plugin_validate_compare_optional_string_projection,
)

PLUGIN_VALIDATE_FEATURE_PROVIDER_DISTRIBUTION_FIELDS = frozenset(
    "abi_version assets default_packaging descriptor_symbol dist_crate editor_entry "
    "engine_compat forms runtime_entry".split()
)


def plugin_validate_feature_provider_distribution_known_fields(
    distribution: dict[str, Any],
    package_id: str,
    diagnostics: list[str],
) -> None:
    label = f"plugin {package_id} generated distribution"
    for field_name in distribution:
        if field_name in PLUGIN_VALIDATE_FEATURE_PROVIDER_DISTRIBUTION_FIELDS:
            continue
        diagnostics.append(
            f"{label}.{field_name} "
            "is not a known feature provider distribution field"
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
    plugin_validate_feature_provider_distribution_known_fields(
        generated_distribution, package_id, diagnostics
    )
    validate_plugin_feature_provider_distribution_schema(
        generated_distribution, package_id, diagnostics
    )
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
