"""Feature-provider generated manifest required metadata validation."""

from __future__ import annotations

from typing import Any

PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_STRING_FIELDS = (
    "version", "display_name", "description", "sdk_api_version", "category", "maturity",
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_ARRAY_FIELDS = (
    "supported_targets", "supported_platforms", "capabilities", "default_packaging",
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_REQUIRED_FIELDS = (
    PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_STRING_FIELDS
    + PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_ARRAY_FIELDS
)


def validate_plugin_feature_provider_manifest_required_metadata(
    manifest: dict[str, Any],
    package_id: str,
    diagnostics: list[str],
) -> None:
    label = f"plugin {package_id} generated manifest"
    for field_name in PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_REQUIRED_FIELDS:
        if field_name not in manifest:
            diagnostics.append(f"{label}.{field_name} is required")
