"""Component and UI component validation for standalone plugin manifests."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .native_build_workspace import read_toml
from .plugin_validate_common import plugin_other_manifest_paths, plugin_validate_trimmed_string

Diagnostics = list[str]
Manifest = dict[str, Any]
IdentityOwners = dict[str, str]

PLUGIN_VALIDATE_COMPONENT_FIELDS = frozenset({"type_id", "plugin_id", "display_name", "properties"})
PLUGIN_VALIDATE_COMPONENT_PROPERTY_FIELDS = frozenset({"name", "value_type", "editable"})
PLUGIN_VALIDATE_UI_COMPONENT_FIELDS = frozenset({"component_id", "plugin_id", "ui_document"})


def validate_plugin_components(
    *,
    plugin_manifest_path: Path | None,
    plugin_root: Path | None,
    package_id: str,
    diagnostics: Diagnostics,
) -> None:
    if plugin_manifest_path is None:
        return
    manifest = read_toml(plugin_manifest_path, diagnostics)
    if manifest is None:
        return
    component_owners, ui_component_owners = plugin_validate_component_identity_index(
        plugin_root, plugin_manifest_path, diagnostics
    )
    validate_plugin_component_rows(
        manifest.get("components"),
        f"plugin {package_id} components",
        package_id,
        component_owners,
        diagnostics,
    )
    validate_plugin_ui_component_rows(
        manifest.get("ui_components"),
        f"plugin {package_id} ui_components",
        package_id,
        ui_component_owners,
        diagnostics,
    )


def validate_plugin_component_rows(
    components: Any,
    label: str,
    package_id: str,
    component_owners: IdentityOwners,
    diagnostics: Diagnostics,
) -> None:
    if components is None:
        return
    if not isinstance(components, list):
        diagnostics.append(f"{label} must be an array")
        return
    if not components:
        diagnostics.append(f"{label} must not be empty when declared")
        return
    seen: dict[str, int] = {}
    for index, component in enumerate(components):
        row_label = f"{label}[{index}]"
        if not isinstance(component, dict):
            diagnostics.append(f"{row_label} must be a table")
            continue
        validate_plugin_component_known_fields(
            component, PLUGIN_VALIDATE_COMPONENT_FIELDS, row_label, "component",
            diagnostics,
        )
        type_id = validate_plugin_component_id(
            component, "type_id", f"{row_label}.type_id", package_id, diagnostics
        )
        validate_plugin_component_plugin_id(
            component, f"{row_label}.plugin_id", package_id, diagnostics
        )
        plugin_validate_trimmed_string(
            component, "display_name", f"{row_label}.display_name", diagnostics
        )
        validate_plugin_component_properties(
            component.get("properties"), f"{row_label}.properties", diagnostics
        )
        if type_id is not None:
            validate_plugin_component_identity_uniqueness(
                type_id, row_label, "component type_id", seen, component_owners, diagnostics
            )


def validate_plugin_ui_component_rows(
    components: Any,
    label: str,
    package_id: str,
    ui_component_owners: IdentityOwners,
    diagnostics: Diagnostics,
) -> None:
    if components is None:
        return
    if not isinstance(components, list):
        diagnostics.append(f"{label} must be an array")
        return
    if not components:
        diagnostics.append(f"{label} must not be empty when declared")
        return
    seen: dict[str, int] = {}
    for index, component in enumerate(components):
        row_label = f"{label}[{index}]"
        if not isinstance(component, dict):
            diagnostics.append(f"{row_label} must be a table")
            continue
        validate_plugin_component_known_fields(
            component, PLUGIN_VALIDATE_UI_COMPONENT_FIELDS, row_label, "ui_component",
            diagnostics,
        )
        component_id = validate_plugin_component_id(
            component,
            "component_id",
            f"{row_label}.component_id",
            package_id,
            diagnostics,
        )
        validate_plugin_component_plugin_id(
            component, f"{row_label}.plugin_id", package_id, diagnostics
        )
        validate_plugin_ui_document(
            component, f"{row_label}.ui_document", diagnostics
        )
        if component_id is not None:
            validate_plugin_component_identity_uniqueness(
                component_id,
                row_label,
                "ui component_id",
                seen,
                ui_component_owners,
                diagnostics,
            )


def validate_plugin_component_properties(
    properties: Any,
    label: str,
    diagnostics: Diagnostics,
) -> None:
    if properties is None:
        return
    if not isinstance(properties, list):
        diagnostics.append(f"{label} must be an array")
        return
    if not properties:
        diagnostics.append(f"{label} must not be empty when declared")
        return
    seen: dict[str, int] = {}
    for index, property_row in enumerate(properties):
        row_label = f"{label}[{index}]"
        if not isinstance(property_row, dict):
            diagnostics.append(f"{row_label} must be a table")
            continue
        validate_plugin_component_known_fields(
            property_row, PLUGIN_VALIDATE_COMPONENT_PROPERTY_FIELDS, row_label,
            "component property",
            diagnostics,
        )
        name = plugin_validate_trimmed_string(
            property_row, "name", f"{row_label}.name", diagnostics
        )
        plugin_validate_trimmed_string(
            property_row, "value_type", f"{row_label}.value_type", diagnostics
        )
        if type(property_row.get("editable")) is not bool:
            diagnostics.append(f"{row_label}.editable must be a bool")
        if name is not None:
            previous_index = seen.get(name)
            if previous_index is not None:
                diagnostics.append(
                    f"{row_label}.name duplicates property row {previous_index}"
                )
            else:
                seen[name] = index


def validate_plugin_component_known_fields(
    table: Manifest,
    known_fields: frozenset[str],
    row_label: str,
    field_label: str,
    diagnostics: Diagnostics,
) -> None:
    for field in sorted(table):
        if field not in known_fields:
            diagnostics.append(
                f"{row_label}.{field} is not a known {field_label} field"
            )


def validate_plugin_component_id(
    table: Manifest,
    field: str,
    label: str,
    package_id: str,
    diagnostics: Diagnostics,
) -> str | None:
    value = plugin_validate_trimmed_string(table, field, label, diagnostics)
    if value is None:
        return None
    plugin_validate_component_dot_namespace(value, label, diagnostics)
    expected_prefix = f"{package_id}."
    if not value.startswith(expected_prefix):
        diagnostics.append(
            f"{label} {value} should stay under package namespace {expected_prefix}"
        )
    return value


def validate_plugin_component_plugin_id(
    table: Manifest,
    label: str,
    package_id: str,
    diagnostics: Diagnostics,
) -> None:
    plugin_id = plugin_validate_trimmed_string(table, "plugin_id", label, diagnostics)
    if plugin_id is not None and plugin_id != package_id:
        diagnostics.append(f"{label} {plugin_id} should match package id {package_id}")


def validate_plugin_ui_document(
    table: Manifest,
    label: str,
    diagnostics: Diagnostics,
) -> None:
    ui_document = plugin_validate_trimmed_string(
        table, "ui_document", label, diagnostics
    )
    if ui_document is None:
        return
    if not ui_document.endswith(".zui"):
        diagnostics.append(f"{label} {ui_document} should reference a .zui component asset")
    if (
        ui_document.startswith("/")
        or "\\" in ui_document
        or any(segment in {"", ".", ".."} for segment in ui_document.split("/"))
    ):
        diagnostics.append(
            f"{label} {ui_document} should be a relative forward-slash package path"
        )


def validate_plugin_component_identity_uniqueness(
    identity: str,
    row_label: str,
    identity_name: str,
    seen: dict[str, int],
    owners: IdentityOwners,
    diagnostics: Diagnostics,
) -> None:
    previous_index = seen.get(identity)
    if previous_index is not None:
        diagnostics.append(
            f"{row_label}.{identity_name.split()[-1]} {identity} "
            f"duplicates {identity_name} row {previous_index}"
        )
        return
    seen[identity] = int(row_label.rsplit("[", 1)[-1].rstrip("]"))
    owner = owners.get(identity)
    if owner is not None:
        diagnostics.append(
            f"{row_label}.{identity_name.split()[-1]} {identity} "
            f"duplicates {identity_name} declared by {owner}"
        )


def plugin_validate_component_dot_namespace(
    value: str,
    label: str,
    diagnostics: Diagnostics,
) -> None:
    segments = value.split(".")
    if len(segments) < 2:
        diagnostics.append(
            f"{label} {value} should use lowercase dot namespace form"
        )
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


def plugin_validate_component_identity_index(
    plugin_root: Path | None,
    current_manifest_path: Path,
    diagnostics: Diagnostics,
) -> tuple[IdentityOwners, IdentityOwners]:
    component_owners: IdentityOwners = {}
    ui_component_owners: IdentityOwners = {}
    if plugin_root is None or not plugin_root.exists():
        return component_owners, ui_component_owners
    for manifest_path in plugin_other_manifest_paths(plugin_root, current_manifest_path):
        manifest = read_toml(manifest_path, diagnostics)
        if manifest is None:
            continue
        owner = manifest.get("id")
        if not isinstance(owner, str) or not owner.strip() or owner.strip() != owner:
            continue
        plugin_validate_collect_component_identities(
            manifest.get("components"), "type_id", owner, component_owners
        )
        plugin_validate_collect_component_identities(
            manifest.get("ui_components"), "component_id", owner, ui_component_owners
        )
    return component_owners, ui_component_owners


def plugin_validate_collect_component_identities(
    rows: Any,
    field: str,
    owner: str,
    owners: IdentityOwners,
) -> None:
    if not isinstance(rows, list):
        return
    for row in rows:
        if not isinstance(row, dict):
            continue
        value = row.get(field)
        if isinstance(value, str) and value.strip() == value:
            owners.setdefault(value, owner)
