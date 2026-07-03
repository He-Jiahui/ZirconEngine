"""Feature-provider module projection validation."""

from __future__ import annotations

from typing import Any

from .plugin_validate_common import (
    plugin_validate_string_array,
    plugin_validate_trimmed_string,
)
from .plugin_validate_feature_provider_module_schema import (
    validate_plugin_feature_provider_module_schema,
)

PLUGIN_VALIDATE_FEATURE_PROVIDER_MODULE_FIELDS = frozenset(
    "capabilities crate_name kind name target_modes".split()
)
PLUGIN_VALIDATE_FEATURE_PROVIDER_MODULE_KIND = "runtime"
PLUGIN_VALIDATE_FEATURE_PROVIDER_MODULE_FALLBACK_TARGET_MODES = (
    "client_runtime",
    "editor_host",
)


def validate_plugin_feature_provider_modules(
    *,
    selected_feature: dict[str, Any],
    generated_manifest: dict[str, Any] | None,
    generated_distribution: dict[str, Any] | None,
    generated_feature: dict[str, Any],
    package_id: str,
    diagnostics: list[str],
) -> None:
    modules_label = f"plugin {package_id} generated feature_extensions[0].modules"
    generated_module = plugin_validate_feature_provider_single_module(
        generated_feature.get("modules"), modules_label, diagnostics
    )
    if generated_module is None:
        return
    validate_plugin_feature_provider_module_schema(
        generated_manifest=generated_manifest,
        generated_feature=generated_feature,
        generated_module=generated_module,
        package_id=package_id,
        diagnostics=diagnostics,
    )
    expected = plugin_validate_feature_provider_expected_runtime_module(
        selected_feature, generated_distribution
    )
    row_label = f"{modules_label}[0]"
    name = plugin_validate_trimmed_string(
        generated_module, "name", f"{row_label}.name", diagnostics
    )
    kind = plugin_validate_trimmed_string(
        generated_module, "kind", f"{row_label}.kind", diagnostics
    )
    crate_name = plugin_validate_trimmed_string(
        generated_module, "crate_name", f"{row_label}.crate_name", diagnostics
    )
    target_modes = plugin_validate_string_array(
        generated_module, "target_modes", f"{row_label}.target_modes", diagnostics
    )
    capabilities = plugin_validate_string_array(
        generated_module, "capabilities", f"{row_label}.capabilities", diagnostics
    )
    if name is not None and name != expected["name"]:
        diagnostics.append(f"{row_label}.name must match owner optional feature runtime module.name")
    if kind is not None and kind != PLUGIN_VALIDATE_FEATURE_PROVIDER_MODULE_KIND:
        diagnostics.append(f"{row_label}.kind must equal runtime")
    if crate_name is not None and crate_name != expected["crate_name"]:
        diagnostics.append(f"{row_label}.crate_name must equal generated distribution.dist_crate")
    if target_modes is not None and target_modes != expected["target_modes"]:
        diagnostics.append(f"{row_label}.target_modes must match owner optional feature runtime module.target_modes")
    if capabilities is not None and capabilities != expected["capabilities"]:
        diagnostics.append(f"{row_label}.capabilities must match owner optional feature runtime module.capabilities")


def plugin_validate_feature_provider_single_module(
    modules: Any, label: str, diagnostics: list[str]
) -> dict[str, Any] | None:
    if not isinstance(modules, list) or len(modules) != 1:
        diagnostics.append(f"{label} must contain exactly one runtime module table")
        return None
    module = modules[0]
    if not isinstance(module, dict):
        diagnostics.append(f"{label}[0] must be a table")
        return None
    for field_name in sorted(module):
        if field_name not in PLUGIN_VALIDATE_FEATURE_PROVIDER_MODULE_FIELDS:
            diagnostics.append(
                f"{label}[0].{field_name} is not a known feature provider module field"
            )
    return module


def plugin_validate_feature_provider_expected_runtime_module(
    selected_feature: dict[str, Any],
    generated_distribution: dict[str, Any] | None,
) -> dict[str, Any]:
    feature_id = _trimmed_string(selected_feature.get("id")) or "feature"
    feature_capabilities = _trimmed_string_array(selected_feature.get("capabilities"))
    source_module = _first_runtime_module(selected_feature)
    dist_crate = (
        _trimmed_string(generated_distribution.get("dist_crate"))
        if isinstance(generated_distribution, dict)
        else None
    ) or ""
    name = _trimmed_string(source_module.get("name")) if source_module else None
    target_modes = (
        _trimmed_string_array(source_module.get("target_modes")) if source_module else []
    ) or list(PLUGIN_VALIDATE_FEATURE_PROVIDER_MODULE_FALLBACK_TARGET_MODES)
    capabilities = (
        _trimmed_string_array(source_module.get("capabilities"))
        if source_module
        else []
    ) or feature_capabilities
    return {
        "name": name or f"{feature_id}.runtime",
        "crate_name": dist_crate,
        "target_modes": target_modes,
        "capabilities": capabilities,
    }


def _first_runtime_module(selected_feature: dict[str, Any]) -> dict[str, Any] | None:
    modules = selected_feature.get("modules")
    if not isinstance(modules, list):
        return None
    for module in modules:
        if isinstance(module, dict) and module.get("kind") == "runtime":
            return module
    return None


def _trimmed_string(value: object) -> str | None:
    return value if isinstance(value, str) and value.strip() == value and value else None


def _trimmed_string_array(value: object) -> list[str]:
    if not isinstance(value, list):
        return []
    return [
        item for item in value if isinstance(item, str) and item.strip() == item and item
    ]
