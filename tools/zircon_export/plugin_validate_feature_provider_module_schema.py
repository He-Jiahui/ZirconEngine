"""Generated feature-provider module schema checks."""

from __future__ import annotations

from typing import Any

from .plugin_validate_common import plugin_validate_allowed_string_values, plugin_validate_string_array
from .plugin_validate_distribution_module_target_modes import PLUGIN_VALIDATE_TARGET_MODES
from .plugin_validate_modules import (
    validate_plugin_module_capabilities,
    validate_plugin_module_crate_name,
    validate_plugin_module_kind,
    validate_plugin_module_name,
    validate_plugin_module_name_kind,
    validate_plugin_module_target_modes,
)


def validate_plugin_feature_provider_module_schema(
    *,
    generated_manifest: dict[str, Any] | None,
    generated_feature: dict[str, Any],
    generated_module: dict[str, Any],
    package_id: str,
    diagnostics: list[str],
) -> None:
    row_label = f"plugin {package_id} generated feature_extensions[0].modules[0]"
    namespace_id = _generated_feature_namespace(generated_feature) or package_id
    supported_targets = _generated_supported_targets(generated_manifest)
    module_name = validate_plugin_module_name(
        generated_module,
        f"{row_label}.name",
        namespace_id,
        diagnostics,
    )
    module_kind = validate_plugin_module_kind(
        generated_module, f"{row_label}.kind", diagnostics
    )
    validate_plugin_module_crate_name(
        generated_module, f"{row_label}.crate_name", diagnostics
    )
    target_modes = plugin_validate_string_array(
        generated_module, "target_modes", f"{row_label}.target_modes", diagnostics
    )
    capabilities = plugin_validate_string_array(
        generated_module, "capabilities", f"{row_label}.capabilities", diagnostics
    )
    if module_name is not None and module_kind is not None:
        validate_plugin_module_name_kind(module_name, module_kind, row_label, diagnostics)
    if target_modes is not None:
        plugin_validate_allowed_string_values(
            target_modes,
            f"{row_label}.target_modes",
            PLUGIN_VALIDATE_TARGET_MODES,
            diagnostics,
        )
        validate_plugin_module_target_modes(
            target_modes, module_kind, row_label, supported_targets, diagnostics
        )
    if module_kind is not None and capabilities is not None:
        validate_plugin_module_capabilities(
            capabilities, module_kind, f"{row_label}.capabilities", diagnostics
        )


def _generated_feature_namespace(generated_feature: dict[str, Any]) -> str | None:
    feature_id = generated_feature.get("id")
    if not isinstance(feature_id, str) or not feature_id.strip():
        return None
    if feature_id.strip() != feature_id:
        return None
    return feature_id


def _generated_supported_targets(generated_manifest: dict[str, Any] | None) -> set[str]:
    if generated_manifest is None:
        return set()
    supported_targets = generated_manifest.get("supported_targets")
    if not isinstance(supported_targets, list):
        return set()
    return {
        target
        for target in supported_targets
        if isinstance(target, str) and target.strip() and target.strip() == target
    }
