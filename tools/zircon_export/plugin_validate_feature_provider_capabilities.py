"""Feature-provider capability projection validation."""

from __future__ import annotations

from typing import Any

from .plugin_validate_capabilities import validate_plugin_capability_values
from .plugin_validate_common import plugin_validate_string_array


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
    validate_plugin_capability_values(
        expected_capabilities,
        f"plugin {package_id} optional feature capabilities",
        diagnostics,
    )
    if len(diagnostics) != diagnostic_count:
        return
    generated_capabilities = plugin_validate_string_array(
        generated_feature,
        "capabilities",
        f"plugin {package_id} generated feature_extensions[0].capabilities",
        diagnostics,
    )
    if len(diagnostics) != diagnostic_count or generated_capabilities is None:
        return
    validate_plugin_capability_values(
        generated_capabilities,
        f"plugin {package_id} generated feature_extensions[0].capabilities",
        diagnostics,
    )
    if len(diagnostics) != diagnostic_count:
        return
    if generated_capabilities != expected_capabilities:
        diagnostics.append(
            f"plugin {package_id} generated feature_extensions[0].capabilities "
            "must match owner optional feature capabilities"
        )
