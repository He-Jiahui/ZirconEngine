"""Row-level module validation for standalone plugin manifests."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .plugin_validate_common import (
    plugin_validate_optional_trimmed_string,
    plugin_validate_string_array,
    plugin_validate_trimmed_string,
)
from .plugin_validate_module_crates import validate_plugin_module_workspace_crate
from .plugin_validate_module_systems import validate_plugin_module_system_contracts

Diagnostics = list[str]
Manifest = dict[str, Any]
ModuleRowContext = tuple[Path | None, dict[str, dict[str, Any]], set[str], dict[str, str], Diagnostics]

PLUGIN_VALIDATE_MODULE_KINDS = ("runtime", "editor", "native", "vm")
PLUGIN_VALIDATE_MODULE_FIELDS = frozenset(
    (
        "name",
        "description",
        "kind",
        "crate_name",
        "target_modes",
        "capabilities",
        "system_sets",
        "system_anchors",
        "event_consumers",
    )
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
    plugin_validate_optional_trimmed_string(
        module, "description", f"{row_label}.description", diagnostics
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
    validate_plugin_module_event_consumers(
        module, module_kind, row_label, namespace_id, diagnostics
    )
    if crate_name is not None:
        validate_plugin_module_workspace_crate(
            crate_name,
            f"{row_label}.crate_name",
            package_root,
            plugin_root,
            workspace_crate_index,
            diagnostics,
        )


def validate_plugin_module_event_consumers(
    module: Manifest,
    module_kind: str | None,
    row_label: str,
    namespace_id: str,
    diagnostics: Diagnostics,
) -> None:
    consumers = module.get("event_consumers")
    if consumers is None:
        return
    if module_kind != "editor":
        diagnostics.append(
            f"{row_label}.event_consumers may only be declared by editor modules"
        )
    if not isinstance(consumers, list) or not consumers:
        diagnostics.append(f"{row_label}.event_consumers must be a non-empty array")
        return
    known_fields = {"consumer_id", "event_id", "payload_schema", "required_capability"}
    seen_ids: set[str] = set()
    for index, consumer in enumerate(consumers):
        label = f"{row_label}.event_consumers[{index}]"
        if not isinstance(consumer, dict):
            diagnostics.append(f"{label} must be a table")
            continue
        for field in sorted(consumer):
            if field not in known_fields:
                diagnostics.append(f"{label}.{field} is not a known event consumer field")
        values: dict[str, str] = {}
        for field in ("consumer_id", "event_id", "payload_schema", "required_capability"):
            value = consumer.get(field)
            if not isinstance(value, str) or not value.strip() or value.strip() != value:
                diagnostics.append(f"{label}.{field} must be a non-empty trimmed string")
            else:
                values[field] = value
        consumer_id = values.get("consumer_id")
        if consumer_id is not None:
            if consumer_id in seen_ids:
                diagnostics.append(f"{label}.consumer_id {consumer_id} is duplicated")
            seen_ids.add(consumer_id)
            if not consumer_id.startswith(f"{namespace_id}."):
                diagnostics.append(
                    f"{label}.consumer_id {consumer_id} should stay under namespace {namespace_id}."
                )
        event_id = values.get("event_id")
        if event_id is not None and not event_id.startswith(f"{namespace_id}."):
            diagnostics.append(
                f"{label}.event_id {event_id} should stay under namespace {namespace_id}."
            )
        capability = values.get("required_capability")
        if capability is not None and not capability.startswith("editor."):
            diagnostics.append(
                f"{label}.required_capability {capability} should start with editor."
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
