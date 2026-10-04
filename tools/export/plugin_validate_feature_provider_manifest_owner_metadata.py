"""Feature-provider generated manifest owner metadata projection validation."""

from __future__ import annotations

from typing import Any

PLUGIN_VALIDATE_FEATURE_PROVIDER_OWNER_STRING_FIELDS = (
    ("version", "0.1.0"),
    ("sdk_api_version", "0.1.0"),
    ("category", "runtime"),
    ("maturity", "beta"),
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_OWNER_SUPPORTED_PLATFORMS = (
    "windows",
    "linux",
    "macos",
)


def validate_plugin_feature_provider_manifest_owner_metadata(
    *,
    owner_manifest: dict[str, Any],
    generated_manifest: dict[str, Any],
    package_id: str,
    diagnostics: list[str],
) -> None:
    for field, fallback in PLUGIN_VALIDATE_FEATURE_PROVIDER_OWNER_STRING_FIELDS:
        _compare_owner_string_field(
            owner_manifest, generated_manifest, field, fallback, package_id, diagnostics
        )
    _compare_supported_platforms(
        owner_manifest, generated_manifest, package_id, diagnostics
    )


def _compare_owner_string_field(
    owner_manifest: dict[str, Any],
    generated_manifest: dict[str, Any],
    field: str,
    fallback: str,
    package_id: str,
    diagnostics: list[str],
) -> None:
    owner_value = _valid_trimmed_string(owner_manifest.get(field)) or fallback
    generated_value = _valid_trimmed_string(generated_manifest.get(field))
    if generated_value is None or generated_value == owner_value:
        return
    diagnostics.append(
        f"plugin {package_id} generated manifest.{field} "
        f"must equal owner manifest.{field}"
    )


def _compare_supported_platforms(
    owner_manifest: dict[str, Any],
    generated_manifest: dict[str, Any],
    package_id: str,
    diagnostics: list[str],
) -> None:
    owner_value = _valid_string_array(owner_manifest.get("supported_platforms")) or list(
        PLUGIN_VALIDATE_FEATURE_PROVIDER_OWNER_SUPPORTED_PLATFORMS
    )
    generated_value = _valid_string_array(generated_manifest.get("supported_platforms"))
    if generated_value is None or generated_value == owner_value:
        return
    diagnostics.append(
        f"plugin {package_id} generated manifest.supported_platforms "
        "must match owner manifest.supported_platforms"
    )


def _valid_trimmed_string(value: object) -> str | None:
    if not isinstance(value, str) or not value.strip() or value.strip() != value:
        return None
    return value


def _valid_string_array(value: object) -> list[str] | None:
    if not isinstance(value, list) or not value:
        return None
    for item in value:
        if not isinstance(item, str) or not item.strip() or item.strip() != item:
            return None
    return value
