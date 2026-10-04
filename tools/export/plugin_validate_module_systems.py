"""System field validation for standalone plugin module rows."""

from __future__ import annotations

from typing import Any

Diagnostics = list[str]
Manifest = dict[str, Any]

PLUGIN_VALIDATE_MODULE_SYSTEM_FIELDS = ("system_sets", "system_anchors")


def validate_plugin_module_system_contracts(
    module: Manifest,
    module_kind: str | None,
    row_label: str,
    namespace_id: str,
    diagnostics: Diagnostics,
) -> None:
    for field in PLUGIN_VALIDATE_MODULE_SYSTEM_FIELDS:
        validate_plugin_module_system_names(
            module, field, module_kind, row_label, namespace_id, diagnostics
        )


def validate_plugin_module_system_names(
    module: Manifest,
    field: str,
    module_kind: str | None,
    row_label: str,
    namespace_id: str,
    diagnostics: Diagnostics,
) -> None:
    if field not in module:
        return
    if module_kind is not None and module_kind != "runtime":
        diagnostics.append(f"{row_label}.{field} may only be declared by runtime modules")
    value = module[field]
    label = f"{row_label}.{field}"
    if not isinstance(value, list) or not value:
        diagnostics.append(f"{label} must be a non-empty string array when declared")
        return
    seen: dict[str, int] = {}
    for index, item in enumerate(value):
        item_label = f"{label}[{index}]"
        if not isinstance(item, str) or not item.strip() or item.strip() != item:
            diagnostics.append(f"{item_label} must be a non-empty trimmed string")
            continue
        validate_plugin_module_system_namespace(
            item, item_label, namespace_id, diagnostics
        )
        previous = seen.get(item)
        if previous is not None:
            diagnostics.append(f"{item_label} {item} duplicates {field}[{previous}]")
        else:
            seen[item] = index


def validate_plugin_module_system_namespace(
    value: str,
    label: str,
    namespace_id: str,
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
    expected_prefix = f"{namespace_id}."
    if not value.startswith(expected_prefix):
        diagnostics.append(
            f"{label} {value} should stay under namespace {expected_prefix}"
        )
