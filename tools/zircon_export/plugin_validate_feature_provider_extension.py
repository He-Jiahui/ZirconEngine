"""Feature-provider feature-extension projection validation."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .native_build_workspace import read_toml
from .plugin_validate_common import plugin_validate_manifest_target_id, plugin_validate_selected_feature
from .plugin_validate_feature_provider_capabilities import validate_plugin_feature_provider_capabilities
from .plugin_validate_feature_provider_dependencies import validate_plugin_feature_provider_dependencies
from .plugin_validate_feature_provider_distribution import validate_plugin_feature_provider_distribution_projection
from .plugin_validate_feature_provider_extension_metadata import validate_plugin_feature_provider_extension_metadata
from .plugin_validate_feature_provider_extension_schema import validate_plugin_feature_provider_extension_schema
from .plugin_validate_feature_provider_modules import validate_plugin_feature_provider_modules
from .plugin_validate_feature_provider_manifest_owner_metadata import validate_plugin_feature_provider_manifest_owner_metadata
from .plugin_validate_optional_feature_dependencies import plugin_validate_package_capabilities

PLUGIN_VALIDATE_FEATURE_PROVIDER_EXTENSION_FIELDS = frozenset(
    "capabilities default_packaging dependencies display_name enabled_by_default id "
    "modules owner_plugin_id".split()
)


def plugin_validate_feature_provider_extension_known_fields(
    generated_feature: dict[str, Any],
    package_id: str,
    diagnostics: list[str],
) -> None:
    label = f"plugin {package_id} generated feature_extensions[0]"
    for field_name in generated_feature:
        if field_name in PLUGIN_VALIDATE_FEATURE_PROVIDER_EXTENSION_FIELDS:
            continue
        diagnostics.append(
            f"{label}.{field_name} "
            "is not a known feature provider extension field"
        )


def validate_plugin_feature_extension_projection(
    *,
    plugin_manifest_path: Path | None,
    generated_distribution: dict[str, Any] | None,
    generated_feature: dict[str, Any],
    requested_plugin_id: str,
    package_id: str,
    diagnostics: list[str],
    generated_manifest: dict[str, Any] | None = None,
) -> None:
    if plugin_manifest_path is None:
        return
    owner_manifest = read_toml(plugin_manifest_path, diagnostics)
    if owner_manifest is None:
        return
    owner_id = plugin_validate_manifest_target_id(owner_manifest, f"plugin {package_id} owner manifest id", diagnostics)
    owner_package_capabilities = plugin_validate_package_capabilities(owner_manifest)
    if generated_manifest is not None:
        validate_plugin_feature_provider_manifest_owner_metadata(
            owner_manifest=owner_manifest,
            generated_manifest=generated_manifest,
            package_id=package_id,
            diagnostics=diagnostics,
        )
    selected_feature = plugin_validate_selected_feature(owner_manifest, requested_plugin_id, package_id)
    if selected_feature is None:
        diagnostics.append(
            f"plugin {package_id} generated feature_extension was not found "
            "in owner manifest"
        )
        return
    plugin_validate_feature_provider_extension_known_fields(
        generated_feature, package_id, diagnostics
    )
    validate_plugin_feature_provider_extension_schema(generated_feature, package_id, diagnostics)
    feature_id = plugin_validate_manifest_target_id(selected_feature, f"plugin {package_id} selected feature id", diagnostics)
    if feature_id is not None and generated_feature.get("id") != feature_id:
        diagnostics.append(
            f"plugin {package_id} generated feature_extensions[0].id "
            f"must equal {feature_id}"
        )
    if owner_id is not None and generated_feature.get("owner_plugin_id") != owner_id:
        diagnostics.append(
            f"plugin {package_id} generated feature_extensions[0].owner_plugin_id "
            f"must equal {owner_id}"
        )
    validate_plugin_feature_provider_extension_metadata(
        selected_feature=selected_feature,
        generated_distribution=generated_distribution,
        generated_feature=generated_feature,
        package_id=package_id,
        diagnostics=diagnostics,
    )
    validate_plugin_feature_provider_distribution_projection(
        selected_feature=selected_feature,
        generated_distribution=generated_distribution,
        package_id=package_id,
        diagnostics=diagnostics,
    )
    validate_plugin_feature_provider_capabilities(
        selected_feature=selected_feature,
        generated_feature=generated_feature,
        package_id=package_id,
        diagnostics=diagnostics,
    )
    validate_plugin_feature_provider_dependencies(
        selected_feature=selected_feature,
        generated_feature=generated_feature,
        package_id=package_id,
        owner_plugin_id=owner_id,
        owner_package_capabilities=owner_package_capabilities,
        diagnostics=diagnostics,
    )
    validate_plugin_feature_provider_modules(
        selected_feature=selected_feature,
        generated_manifest=generated_manifest,
        generated_distribution=generated_distribution,
        generated_feature=generated_feature,
        package_id=package_id,
        diagnostics=diagnostics,
    )
