from __future__ import annotations

from tools.plugin_structure_audits.manifest_schema import (
    collect_manifest_schema_violations,
)


def plugin_manifest(
    *,
    feature_extensions: list[object] | None = None,
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
        "package_kind": "feature_extension",
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
    if feature_extensions is not None:
        manifest["feature_extensions"] = feature_extensions
    return manifest


def plugin_feature_extension(
    *,
    feature_id: object = "sound.preview",
    owner_plugin_id: object = "sound",
    capabilities: list[str] | None = None,
    dependencies: list[object] | None = None,
    distribution: object | None = None,
    modules: list[dict[str, object]] | None = None,
) -> dict[str, object]:
    feature: dict[str, object] = {
        "id": feature_id,
        "display_name": "Sound Preview",
        "owner_plugin_id": owner_plugin_id,
        "capabilities": capabilities or ["runtime.feature.sound.preview"],
        "default_packaging": ["source_template"],
        "enabled_by_default": False,
        "dependencies": [feature_dependency()] if dependencies is None else dependencies,
    }
    if distribution is not None:
        feature["distribution"] = distribution
    if modules is not None:
        feature["modules"] = modules
    return feature


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


def feature_distribution() -> dict[str, object]:
    return {
        "forms": ["dist"],
        "default_packaging": ["native_dynamic"],
        "abi_version": 3,
        "engine_compat": ">=0.1, <0.2",
        "dist_crate": "zircon_plugin_sound_preview_dist",
        "descriptor_symbol": "zircon_native_plugin_descriptor_v3",
        "runtime_entry": "zircon_plugin_sound_preview_runtime_entry_v3",
    }
