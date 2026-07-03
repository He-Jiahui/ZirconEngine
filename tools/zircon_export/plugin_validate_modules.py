"""Module row validation for standalone plugin manifests."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .native_build_workspace import read_toml
from .plugin_validate_common import plugin_validate_string_array, plugin_validate_trimmed_string
from .plugin_validate_module_crates import plugin_validate_optional_feature_root, validate_plugin_module_workspace_crate
from .plugin_validate_module_systems import validate_plugin_module_system_contracts

Diagnostics = list[str]
Manifest = dict[str, Any]
ModuleRowContext = tuple[Path | None, dict[str, dict[str, Any]], set[str], dict[str, str], Diagnostics]

PLUGIN_VALIDATE_MODULE_KINDS = ("runtime", "editor", "native", "vm")
PLUGIN_VALIDATE_MODULE_FIELDS = frozenset(("name", "kind", "crate_name", "target_modes", "capabilities", "system_sets", "system_anchors"))


def validate_plugin_modules(
    *, plugin_manifest_path: Path | None, plugin_root: Path | None, package_id: str,
    workspace_crate_index: dict[str, dict[str, Any]], diagnostics: Diagnostics,
) -> None:
    if plugin_manifest_path is None:
        return
    manifest = read_toml(plugin_manifest_path, diagnostics)
    if manifest is None:
        return
    supported_targets = plugin_validate_root_supported_targets(manifest)
    seen_names: dict[str, str] = {}
    row_context: ModuleRowContext = (plugin_root, workspace_crate_index, supported_targets, seen_names, diagnostics)
    validate_plugin_module_rows(
        manifest.get("modules"), f"plugin {package_id} modules", package_id,
        plugin_manifest_path.parent, *row_context,
    )
    optional_features = manifest.get("optional_features")
    if isinstance(optional_features, list):
        for feature_index, feature in enumerate(optional_features):
            if not isinstance(feature, dict):
                continue
            feature_id = feature.get("id")
            if not isinstance(feature_id, str) or not feature_id.strip():
                continue
            if feature_id.strip() != feature_id:
                continue
            feature_root = plugin_validate_optional_feature_root(
                plugin_manifest_path.parent, package_id, feature_id
            )
            validate_plugin_module_rows(
                feature.get("modules"),
                f"plugin {package_id} optional_features[{feature_index}].modules",
                feature_id, feature_root, *row_context,
            )
    validate_plugin_feature_extension_modules(
        manifest.get("feature_extensions"), plugin_manifest_path.parent, package_id, row_context,
    )


def validate_plugin_feature_extension_modules(
    feature_extensions: Any, package_root: Path, package_id: str, row_context: ModuleRowContext,
) -> None:
    if not isinstance(feature_extensions, list):
        return
    for feature_index, feature in enumerate(feature_extensions):
        if not isinstance(feature, dict):
            continue
        feature_id = feature.get("id")
        if not isinstance(feature_id, str) or not feature_id.strip():
            continue
        if feature_id.strip() != feature_id:
            continue
        validate_plugin_module_rows(
            feature.get("modules"),
            f"plugin {package_id} feature_extensions[{feature_index}].modules",
            feature_id, package_root, *row_context,
        )


def validate_plugin_module_rows(
    modules: Any, label: str, namespace_id: str, package_root: Path, plugin_root: Path | None,
    workspace_crate_index: dict[str, dict[str, Any]],
    supported_targets: set[str], seen_names: dict[str, str], diagnostics: Diagnostics,
) -> None:
    if modules is None:
        return
    if not isinstance(modules, list):
        diagnostics.append(f"{label} must be an array")
        return
    if not modules:
        diagnostics.append(f"{label} must not be empty when declared")
        return
    for index, module in enumerate(modules):
        row_label = f"{label}[{index}]"
        if not isinstance(module, dict):
            diagnostics.append(f"{row_label} must be a table")
            continue
        validate_plugin_module_known_fields(module, row_label, diagnostics)
        validate_plugin_module_row(
            module, row_label, namespace_id, package_root, plugin_root, workspace_crate_index,
            supported_targets, seen_names, f"row {index}", diagnostics,
        )


def validate_plugin_module_known_fields(module: Manifest, row_label: str, diagnostics: Diagnostics) -> None:
    for field in sorted(module):
        if field not in PLUGIN_VALIDATE_MODULE_FIELDS:
            diagnostics.append(f"{row_label}.{field} is not a known module field")


def validate_plugin_module_row(
    module: Manifest, row_label: str, namespace_id: str, package_root: Path, plugin_root: Path | None,
    workspace_crate_index: dict[str, dict[str, Any]],
    supported_targets: set[str], seen_names: dict[str, str], row_identity: str, diagnostics: Diagnostics,
) -> None:
    module_name = validate_plugin_module_name(
        module, f"{row_label}.name", namespace_id, diagnostics
    )
    module_kind = validate_plugin_module_kind(module, f"{row_label}.kind", diagnostics)
    crate_name = validate_plugin_module_crate_name(
        module, f"{row_label}.crate_name", diagnostics
    )
    target_modes = plugin_validate_string_array(
        module, "target_modes", f"{row_label}.target_modes", diagnostics
    )
    capabilities = plugin_validate_string_array(
        module, "capabilities", f"{row_label}.capabilities", diagnostics
    )
    if module_name is not None:
        previous = seen_names.get(module_name)
        if previous is not None:
            diagnostics.append(
                f"{row_label}.name {module_name} duplicates module name {previous}"
            )
        else:
            seen_names[module_name] = row_identity
    if module_name is not None and module_kind is not None:
        validate_plugin_module_name_kind(module_name, module_kind, row_label, diagnostics)
    if target_modes is not None:
        validate_plugin_module_target_modes(
            target_modes, module_kind, row_label, supported_targets, diagnostics
        )
    if module_kind is not None and capabilities is not None:
        validate_plugin_module_capabilities(
            capabilities, module_kind, f"{row_label}.capabilities", diagnostics
        )
    validate_plugin_module_system_contracts(module, module_kind, row_label, namespace_id, diagnostics)
    if crate_name is not None:
        validate_plugin_module_workspace_crate(
            crate_name,
            f"{row_label}.crate_name",
            package_root,
            plugin_root,
            workspace_crate_index,
            diagnostics,
        )


def validate_plugin_module_name(
    module: Manifest,
    label: str,
    namespace_id: str,
    diagnostics: Diagnostics,
) -> str | None:
    module_name = plugin_validate_trimmed_string(module, "name", label, diagnostics)
    if module_name is None:
        return None
    plugin_validate_module_dot_namespace(module_name, label, diagnostics)
    expected_prefix = f"{namespace_id}."
    if not module_name.startswith(expected_prefix):
        diagnostics.append(
            f"{label} {module_name} should stay under namespace {expected_prefix}"
        )
    return module_name


def validate_plugin_module_kind(
    module: Manifest,
    label: str,
    diagnostics: Diagnostics,
) -> str | None:
    module_kind = plugin_validate_trimmed_string(module, "kind", label, diagnostics)
    if module_kind is None:
        return None
    if module_kind not in PLUGIN_VALIDATE_MODULE_KINDS:
        diagnostics.append(
            f"{label} {module_kind} should be one of "
            + ", ".join(PLUGIN_VALIDATE_MODULE_KINDS)
        )
        return None
    return module_kind


def validate_plugin_module_crate_name(
    module: Manifest,
    label: str,
    diagnostics: Diagnostics,
) -> str | None:
    crate_name = plugin_validate_trimmed_string(module, "crate_name", label, diagnostics)
    if crate_name is None:
        return None
    if not crate_name.startswith("zircon_plugin_"):
        diagnostics.append(f"{label} {crate_name} should use the zircon_plugin_ prefix")
    if not all(char.isascii() and (char.islower() or char.isdigit() or char == "_") for char in crate_name):
        diagnostics.append(
            f"{label} {crate_name} should use lowercase ASCII letters, digits, or underscores"
        )
    if crate_name.endswith("_") or "__" in crate_name:
        diagnostics.append(
            f"{label} {crate_name} should not end with an underscore or contain repeated underscores"
        )
    return crate_name


def validate_plugin_module_name_kind(
    module_name: str,
    module_kind: str,
    row_label: str,
    diagnostics: Diagnostics,
) -> None:
    expected_suffix = {"runtime": ".runtime", "editor": ".editor"}.get(module_kind)
    if expected_suffix is not None and not module_name.endswith(expected_suffix):
        diagnostics.append(
            f"{row_label}.name {module_name} with kind {module_kind} "
            f"should end with {expected_suffix}"
        )


def validate_plugin_module_target_modes(
    target_modes: list[str],
    module_kind: str | None,
    row_label: str,
    supported_targets: set[str],
    diagnostics: Diagnostics,
) -> None:
    for index, target_mode in enumerate(target_modes):
        if supported_targets and target_mode not in supported_targets:
            diagnostics.append(
                f"{row_label}.target_modes[{index}] {target_mode} "
                "should be covered by package supported_targets"
            )
        if module_kind == "editor" and target_mode != "editor_host":
            diagnostics.append(
                f"{row_label} is an editor module and should only target editor_host, "
                f"got {target_mode}"
            )


def validate_plugin_module_capabilities(
    capabilities: list[str],
    module_kind: str,
    label: str,
    diagnostics: Diagnostics,
) -> None:
    expected_prefix = {"runtime": "runtime.", "editor": "editor."}.get(module_kind)
    if expected_prefix is None:
        return
    for index, capability in enumerate(capabilities):
        if not capability.startswith(expected_prefix):
            diagnostics.append(
                f"{label}[{index}] {capability} should start with {expected_prefix}"
            )


def plugin_validate_module_dot_namespace(
    value: str,
    label: str,
    diagnostics: Diagnostics,
) -> None:
    segments = value.split(".")
    if len(segments) < 2:
        diagnostics.append(f"{label} {value} should use package.module dot namespace form")
    if any(not segment for segment in segments):
        diagnostics.append(f"{label} {value} should not contain empty namespace segments")
    if not all(
        char.isascii() and (char.islower() or char.isdigit() or char in {"_", "."})
        for char in value
    ):
        diagnostics.append(
            f"{label} {value} should contain only lowercase ASCII letters, "
            "digits, underscores, and dots"
        )


def plugin_validate_root_supported_targets(manifest: Manifest) -> set[str]:
    supported_targets = manifest.get("supported_targets")
    if not isinstance(supported_targets, list):
        return set()
    return {
        target
        for target in supported_targets
        if isinstance(target, str) and target.strip() and target.strip() == target
    }
