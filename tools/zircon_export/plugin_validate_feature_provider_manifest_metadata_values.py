"""Generated feature-provider manifest metadata value schema validation."""

from __future__ import annotations

from typing import Any

from .plugin_validate_capabilities import validate_plugin_capability_values
from .plugin_validate_common import plugin_validate_string_array
from .plugin_validate_default_packaging import validate_plugin_default_packaging_values


def plugin_validate_feature_provider_manifest_metadata_values(
    manifest: dict[str, Any],
    package_id: str,
    diagnostics: list[str],
) -> None:
    label = f"plugin {package_id} generated manifest"
    if "capabilities" in manifest:
        capabilities = plugin_validate_string_array(
            manifest, "capabilities", f"{label}.capabilities", diagnostics
        )
        if capabilities is not None:
            validate_plugin_capability_values(
                capabilities, f"{label}.capabilities", diagnostics
            )
    if "default_packaging" in manifest:
        validate_plugin_default_packaging_values(
            manifest,
            "default_packaging",
            f"{label}.default_packaging",
            diagnostics,
        )
