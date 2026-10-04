"""Feature-provider generated manifest description projection validation."""

from __future__ import annotations

from typing import Any


def plugin_validate_feature_provider_manifest_description_projection(
    manifest: dict[str, Any],
    feature_extension: dict[str, Any],
    package_id: str,
    diagnostics: list[str],
) -> None:
    manifest_description = _valid_trimmed_string(manifest.get("description"))
    feature_id = _valid_trimmed_string(feature_extension.get("id"))
    if manifest_description is None or feature_id is None:
        return
    expected_description = (
        f"Native dynamic provider for optional feature {feature_id}."
    )
    if manifest_description != expected_description:
        diagnostics.append(
            f"plugin {package_id} generated manifest.description must equal "
            "Native dynamic provider for optional feature "
            "feature_extensions[0].id"
        )


def _valid_trimmed_string(value: object) -> str | None:
    if not isinstance(value, str) or not value.strip() or value.strip() != value:
        return None
    return value
