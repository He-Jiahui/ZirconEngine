"""Feature-provider extension metadata projection validation."""

from __future__ import annotations

from typing import Any

from .plugin_validate_common import (
    plugin_validate_string_array,
    plugin_validate_trimmed_string,
)
from .plugin_validate_feature_provider_extension_metadata_schema import validate_plugin_feature_provider_extension_metadata_schema


def validate_plugin_feature_provider_extension_metadata(
    *,
    selected_feature: dict[str, Any],
    generated_distribution: dict[str, Any] | None,
    generated_feature: dict[str, Any],
    package_id: str,
    diagnostics: list[str],
) -> None:
    validate_plugin_feature_provider_extension_display_name(
        selected_feature=selected_feature,
        generated_feature=generated_feature,
        package_id=package_id,
        diagnostics=diagnostics,
    )
    validate_plugin_feature_provider_enabled_by_default(
        selected_feature=selected_feature,
        generated_feature=generated_feature,
        package_id=package_id,
        diagnostics=diagnostics,
    )
    validate_plugin_feature_provider_extension_default_packaging(
        generated_distribution=generated_distribution,
        generated_feature=generated_feature,
        package_id=package_id,
        diagnostics=diagnostics,
    )


def validate_plugin_feature_provider_extension_display_name(
    *,
    selected_feature: dict[str, Any],
    generated_feature: dict[str, Any],
    package_id: str,
    diagnostics: list[str],
) -> None:
    diagnostic_count = len(diagnostics)
    owner_display_name = plugin_validate_trimmed_string(
        selected_feature,
        "display_name",
        f"plugin {package_id} optional feature display_name",
        diagnostics,
    )
    generated_display_name = plugin_validate_trimmed_string(
        generated_feature,
        "display_name",
        f"plugin {package_id} generated feature_extensions[0].display_name",
        diagnostics,
    )
    if len(diagnostics) != diagnostic_count:
        return
    if generated_display_name != owner_display_name:
        diagnostics.append(
            f"plugin {package_id} generated "
            "feature_extensions[0].display_name must match "
            "owner optional feature display_name"
        )


def validate_plugin_feature_provider_enabled_by_default(
    *,
    selected_feature: dict[str, Any],
    generated_feature: dict[str, Any],
    package_id: str,
    diagnostics: list[str],
) -> None:
    generated_enabled = generated_feature.get("enabled_by_default")
    label = f"plugin {package_id} generated feature_extensions[0].enabled_by_default"
    if type(generated_enabled) is not bool:
        diagnostics.append(f"{label} must be a bool")
    elif generated_enabled != selected_feature.get("enabled_by_default"):
        diagnostics.append(
            f"{label} must match owner optional feature enabled_by_default"
        )


def validate_plugin_feature_provider_extension_default_packaging(
    *,
    generated_distribution: dict[str, Any] | None,
    generated_feature: dict[str, Any],
    package_id: str,
    diagnostics: list[str],
) -> None:
    if generated_distribution is None:
        return
    diagnostic_count = len(diagnostics)
    validate_plugin_feature_provider_extension_metadata_schema(generated_feature, package_id, diagnostics)
    generated_values = plugin_validate_string_array(
        generated_feature,
        "default_packaging",
        f"plugin {package_id} generated feature_extensions[0].default_packaging",
        diagnostics,
    )
    distribution_values = plugin_validate_string_array(
        generated_distribution,
        "default_packaging",
        f"plugin {package_id} generated distribution.default_packaging",
        diagnostics,
    )
    if len(diagnostics) != diagnostic_count:
        return
    if generated_values != distribution_values:
        diagnostics.append(
            f"plugin {package_id} generated "
            "feature_extensions[0].default_packaging must match "
            "generated distribution.default_packaging"
        )
