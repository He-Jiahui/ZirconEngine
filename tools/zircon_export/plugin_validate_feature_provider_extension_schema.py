"""Generated feature-provider extension row schema validation."""

from __future__ import annotations

from typing import Any

from .plugin_validate_feature_extensions import (
    validate_plugin_feature_extension_id,
    validate_plugin_feature_extension_owner_package_token,
)
from .plugin_validate_common import plugin_validate_trimmed_string


def validate_plugin_feature_provider_extension_schema(
    generated_feature: dict[str, Any],
    package_id: str,
    diagnostics: list[str],
) -> None:
    label = f"plugin {package_id} generated feature_extensions[0]"
    owner_plugin_id = plugin_validate_trimmed_string(
        generated_feature,
        "owner_plugin_id",
        f"{label}.owner_plugin_id",
        diagnostics,
    )
    if owner_plugin_id is not None:
        validate_plugin_feature_extension_owner_package_token(
            owner_plugin_id,
            f"{label}.owner_plugin_id",
            diagnostics,
        )
    validate_plugin_feature_extension_id(
        generated_feature,
        f"{label}.id",
        owner_plugin_id,
        diagnostics,
    )
