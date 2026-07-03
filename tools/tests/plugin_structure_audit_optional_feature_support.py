from __future__ import annotations

from tools.plugin_structure_audits.manifest_schema import (
    collect_manifest_schema_violations,
)


def plugin_manifest(
    *,
    optional_features: list[dict[str, object]] | None = None,
) -> dict[str, object]:
    manifest: dict[str, object] = {
        "id": "sound",
        "version": "0.1.0",
        "sdk_api_version": "0.1.0",
        "display_name": "Sound",
        "category": "runtime",
        "description": "Sound plugin.",
        "supported_targets": ["client_runtime"],
        "supported_platforms": ["windows"],
        "capabilities": ["runtime.plugin.sound"],
        "default_packaging": ["source_template", "library_embed", "native_dynamic"],
        "maturity": "experimental",
        "modules": [
            {
                "name": "sound.runtime",
                "kind": "runtime",
                "crate_name": "zircon_plugin_sound_runtime",
                "target_modes": ["client_runtime"],
                "capabilities": ["runtime.plugin.sound"],
            }
        ],
    }
    if optional_features is not None:
        manifest["optional_features"] = optional_features
    return manifest


def plugin_feature(
    *,
    feature_id: object = "sound.preview",
    owner_plugin_id: object = "sound",
    capabilities: list[str] | None = None,
    optional_modules: list[dict[str, object]] | None = None,
    dependencies: list[object] | None = None,
    distribution: object | None = None,
    provider_package_id: object | None = None,
    default_packaging: list[str] | None = None,
    enabled_by_default: object = False,
) -> dict[str, object]:
    feature: dict[str, object] = {
        "id": feature_id,
        "display_name": "Sound Preview",
        "owner_plugin_id": owner_plugin_id,
        "capabilities": capabilities or ["runtime.feature.sound.preview"],
        "default_packaging": default_packaging or ["source_template"],
        "enabled_by_default": enabled_by_default,
        "dependencies": [feature_dependency()] if dependencies is None else dependencies,
    }
    if optional_modules is not None:
        feature["modules"] = optional_modules
    if distribution is not None:
        feature["distribution"] = distribution
    if provider_package_id is not None:
        feature["provider_package_id"] = provider_package_id
    return feature


def feature_module(
    *,
    kind: str = "runtime",
    target_modes: list[str] | None = None,
) -> dict[str, object]:
    return {
        "name": "sound.preview.runtime",
        "kind": kind,
        "crate_name": "zircon_plugin_sound_preview_runtime",
        "target_modes": target_modes or ["client_runtime"],
        "capabilities": ["runtime.feature.sound.preview"],
    }


def feature_dependency(
    *,
    plugin_id: object = "sound",
    capability: object = "runtime.plugin.sound",
    primary: object = True,
) -> dict[str, object]:
    return {
        "plugin_id": plugin_id,
        "capability": capability,
        "primary": primary,
    }


def feature_distribution(
    *,
    forms: list[str] | None = None,
    default_packaging: list[str] | None = None,
) -> dict[str, object]:
    return {
        "forms": forms or ["dist"],
        "default_packaging": default_packaging or ["native_dynamic"],
        "abi_version": 3,
        "engine_compat": ">=0.1, <0.2",
        "dist_crate": "zircon_plugin_sound_preview_dist",
        "descriptor_symbol": "zircon_native_plugin_descriptor_v3",
        "runtime_entry": "zircon_plugin_sound_preview_runtime_entry_v3",
    }
