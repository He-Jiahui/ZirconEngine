"""Feature-provider package projection validation."""

from __future__ import annotations

from pathlib import Path
from typing import Any

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - Python <3.11 fallback.
    import tomli as tomllib  # type: ignore[no-redef]

from .plugin_validate_common import PLUGIN_VALIDATE_FEATURE_SOURCE
from .plugin_validate_feature_provider_extension import validate_plugin_feature_extension_projection
from .plugin_validate_feature_provider_manifest_schema import plugin_validate_feature_provider_manifest_metadata_schema, plugin_validate_feature_provider_manifest_projection_consistency

PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_FIELDS = frozenset(
    "capabilities category default_packaging description display_name distribution "
    "feature_extensions id maturity package_kind sdk_api_version supported_platforms supported_targets version".split()
)


def plugin_validate_feature_provider_manifest_known_fields(manifest: dict[str, Any], package_id: str, diagnostics: list[str]) -> None:
    label = f"plugin {package_id} generated manifest"
    for field_name in manifest:
        if field_name in PLUGIN_VALIDATE_FEATURE_PROVIDER_MANIFEST_FIELDS:
            continue
        diagnostics.append(
            f"{label}.{field_name} is not a known feature provider manifest field"
        )


def validate_plugin_feature_provider_package_projection(
    *,
    plugin_manifest_path: Path | None,
    package_manifest_text: str | None,
    requested_plugin_id: str,
    package_id: str,
    diagnostics: list[str],
) -> None:
    if package_manifest_text is None:
        return
    generated_manifest = plugin_validate_generated_package_manifest(package_manifest_text, package_id, diagnostics)
    if generated_manifest is None:
        return
    plugin_validate_feature_provider_manifest_known_fields(generated_manifest, package_id, diagnostics)
    plugin_validate_feature_provider_manifest_metadata_schema(generated_manifest, package_id, diagnostics)
    plugin_validate_feature_provider_manifest_projection_consistency(generated_manifest, package_id, diagnostics)
    if generated_manifest.get("id") != package_id:
        diagnostics.append(f"plugin {package_id} generated id must equal {package_id}")
    if generated_manifest.get("package_kind") != PLUGIN_VALIDATE_FEATURE_SOURCE:
        diagnostics.append(f"plugin {package_id} generated package_kind must equal {PLUGIN_VALIDATE_FEATURE_SOURCE}")
    generated_distribution = generated_manifest.get("distribution")
    if not isinstance(generated_distribution, dict):
        diagnostics.append(f"plugin {package_id} generated distribution must be a table")
        generated_distribution = None

    feature_extensions = generated_manifest.get("feature_extensions")
    if not isinstance(feature_extensions, list) or len(feature_extensions) != 1:
        diagnostics.append(f"plugin {package_id} generated feature_extensions must contain exactly one table")
        return
    feature_extension = feature_extensions[0]
    if not isinstance(feature_extension, dict):
        diagnostics.append(
            f"plugin {package_id} generated feature_extensions[0] must be a table"
        )
        return
    validate_plugin_feature_extension_projection(
        plugin_manifest_path=plugin_manifest_path,
        generated_distribution=generated_distribution,
        generated_feature=feature_extension,
        generated_manifest=generated_manifest,
        requested_plugin_id=requested_plugin_id,
        package_id=package_id,
        diagnostics=diagnostics,
    )

def plugin_validate_generated_package_manifest(
    package_manifest_text: str,
    package_id: str,
    diagnostics: list[str],
) -> dict[str, Any] | None:
    try:
        manifest = tomllib.loads(package_manifest_text)
    except tomllib.TOMLDecodeError as error:
        diagnostics.append(
            f"plugin {package_id} generated package manifest is invalid TOML: {error}"
        )
        return None
    if not isinstance(manifest, dict):
        diagnostics.append(f"plugin {package_id} generated package manifest must be a table")
        return None
    return manifest
