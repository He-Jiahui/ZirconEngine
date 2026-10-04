"""Generated feature-provider extension metadata schema validation."""

from __future__ import annotations

from typing import Any

from .plugin_validate_default_packaging import validate_plugin_default_packaging_values


def validate_plugin_feature_provider_extension_metadata_schema(
    generated_feature: dict[str, Any],
    package_id: str,
    diagnostics: list[str],
) -> None:
    validate_plugin_default_packaging_values(
        generated_feature,
        "default_packaging",
        f"plugin {package_id} generated feature_extensions[0].default_packaging",
        diagnostics,
    )
