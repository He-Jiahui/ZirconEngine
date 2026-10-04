from __future__ import annotations

from typing import Any


COMPONENT_FIELDS = frozenset(("type_id", "plugin_id", "display_name", "properties"))
COMPONENT_PROPERTY_FIELDS = frozenset(("name", "value_type", "editable"))
UI_COMPONENT_FIELDS = frozenset(("component_id", "plugin_id", "ui_document"))


def collect_components_schema_violations(
    display_path: str,
    manifest: dict[str, Any],
    violations: list[str],
) -> None:
    package_id = manifest.get("id")
    collect_component_rows_schema_violations(
        display_path,
        "components",
        manifest.get("components"),
        package_id,
        violations,
    )
    collect_ui_component_rows_schema_violations(
        display_path,
        "ui_components",
        manifest.get("ui_components"),
        package_id,
        violations,
    )


def collect_component_rows_schema_violations(
    display_path: str,
    root_label: str,
    components: object,
    package_id: object,
    violations: list[str],
) -> None:
    if components is None:
        return
    if not isinstance(components, list):
        violations.append(f"{display_path}: {root_label} must be an array")
        return
    if not components:
        violations.append(
            f"{display_path}: {root_label} must not be empty when declared"
        )
        return

    seen_type_ids: dict[str, int] = {}
    for component_index, component in enumerate(components):
        row_label = f"{root_label}[{component_index}]"
        if not isinstance(component, dict):
            violations.append(f"{display_path}: {row_label} must be a table")
            continue
        collect_component_known_field_violations(
            display_path,
            row_label,
            component,
            COMPONENT_FIELDS,
            "component",
            violations,
        )
        type_id = collect_component_id_violations(
            display_path,
            f"{row_label}.type_id",
            component,
            "type_id",
            package_id,
            violations,
        )
        collect_component_plugin_id_violations(
            display_path,
            f"{row_label}.plugin_id",
            component,
            package_id,
            violations,
        )
        collect_required_trimmed_string_violation(
            display_path,
            f"{row_label}.display_name",
            component,
            "display_name",
            violations,
        )
        collect_component_properties_schema_violations(
            display_path,
            f"{row_label}.properties",
            component.get("properties"),
            violations,
        )
        if type_id is not None:
            collect_component_identity_duplicate_violations(
                display_path,
                f"{row_label}.type_id",
                type_id,
                "component type_id",
                component_index,
                seen_type_ids,
                violations,
            )


def collect_ui_component_rows_schema_violations(
    display_path: str,
    root_label: str,
    components: object,
    package_id: object,
    violations: list[str],
) -> None:
    if components is None:
        return
    if not isinstance(components, list):
        violations.append(f"{display_path}: {root_label} must be an array")
        return
    if not components:
        violations.append(
            f"{display_path}: {root_label} must not be empty when declared"
        )
        return

    seen_component_ids: dict[str, int] = {}
    for component_index, component in enumerate(components):
        row_label = f"{root_label}[{component_index}]"
        if not isinstance(component, dict):
            violations.append(f"{display_path}: {row_label} must be a table")
            continue
        collect_component_known_field_violations(
            display_path,
            row_label,
            component,
            UI_COMPONENT_FIELDS,
            "ui_component",
            violations,
        )
        component_id = collect_component_id_violations(
            display_path,
            f"{row_label}.component_id",
            component,
            "component_id",
            package_id,
            violations,
        )
        collect_component_plugin_id_violations(
            display_path,
            f"{row_label}.plugin_id",
            component,
            package_id,
            violations,
        )
        collect_ui_document_violations(
            display_path,
            f"{row_label}.ui_document",
            component,
            violations,
        )
        if component_id is not None:
            collect_component_identity_duplicate_violations(
                display_path,
                f"{row_label}.component_id",
                component_id,
                "ui component_id",
                component_index,
                seen_component_ids,
                violations,
            )


def collect_component_properties_schema_violations(
    display_path: str,
    properties_label: str,
    properties: object,
    violations: list[str],
) -> None:
    if properties is None:
        return
    if not isinstance(properties, list):
        violations.append(f"{display_path}: {properties_label} must be an array")
        return
    if not properties:
        violations.append(
            f"{display_path}: {properties_label} must not be empty when declared"
        )
        return

    seen_names: dict[str, int] = {}
    for property_index, property_row in enumerate(properties):
        row_label = f"{properties_label}[{property_index}]"
        if not isinstance(property_row, dict):
            violations.append(f"{display_path}: {row_label} must be a table")
            continue
        collect_component_known_field_violations(
            display_path,
            row_label,
            property_row,
            COMPONENT_PROPERTY_FIELDS,
            "component property",
            violations,
        )
        name = collect_required_trimmed_string_violation(
            display_path,
            f"{row_label}.name",
            property_row,
            "name",
            violations,
        )
        collect_required_trimmed_string_violation(
            display_path,
            f"{row_label}.value_type",
            property_row,
            "value_type",
            violations,
        )
        if type(property_row.get("editable")) is not bool:
            violations.append(f"{display_path}: {row_label}.editable must be a bool")
        if name is not None:
            previous_index = seen_names.get(name)
            if previous_index is not None:
                violations.append(
                    f"{display_path}: {row_label}.name duplicates property row "
                    f"{previous_index}"
                )
                continue
            seen_names[name] = property_index


def collect_component_known_field_violations(
    display_path: str,
    row_label: str,
    table: dict[str, Any],
    known_fields: frozenset[str],
    field_label: str,
    violations: list[str],
) -> None:
    for field in sorted(table):
        if field not in known_fields:
            violations.append(
                f"{display_path}: {row_label}.{field} "
                f"is not a known {field_label} field"
            )


def collect_component_id_violations(
    display_path: str,
    field_label: str,
    table: dict[str, Any],
    field_name: str,
    package_id: object,
    violations: list[str],
) -> str | None:
    value = collect_required_trimmed_string_violation(
        display_path,
        field_label,
        table,
        field_name,
        violations,
    )
    if value is None:
        return None
    collect_component_dot_namespace_violations(display_path, field_label, value, violations)
    if isinstance(package_id, str) and package_id.strip() and package_id.strip() == package_id:
        expected_prefix = f"{package_id}."
        if not value.startswith(expected_prefix):
            violations.append(
                f"{display_path}: {field_label} {value} "
                f"should stay under package namespace {expected_prefix}"
            )
    return value


def collect_component_plugin_id_violations(
    display_path: str,
    field_label: str,
    table: dict[str, Any],
    package_id: object,
    violations: list[str],
) -> None:
    plugin_id = collect_required_trimmed_string_violation(
        display_path,
        field_label,
        table,
        "plugin_id",
        violations,
    )
    if not (
        plugin_id is not None
        and isinstance(package_id, str)
        and package_id.strip()
        and package_id.strip() == package_id
    ):
        return
    if plugin_id != package_id:
        violations.append(
            f"{display_path}: {field_label} {plugin_id} "
            f"should match package id {package_id}"
        )


def collect_ui_document_violations(
    display_path: str,
    field_label: str,
    table: dict[str, Any],
    violations: list[str],
) -> None:
    ui_document = collect_required_trimmed_string_violation(
        display_path,
        field_label,
        table,
        "ui_document",
        violations,
    )
    if ui_document is None:
        return
    if not ui_document.endswith(".zui"):
        violations.append(
            f"{display_path}: {field_label} {ui_document} "
            "should reference a .zui component asset"
        )
    if (
        ui_document.startswith("/")
        or "\\" in ui_document
        or any(segment in {"", ".", ".."} for segment in ui_document.split("/"))
    ):
        violations.append(
            f"{display_path}: {field_label} {ui_document} "
            "should be a relative forward-slash package path"
        )


def collect_component_identity_duplicate_violations(
    display_path: str,
    field_label: str,
    identity: str,
    identity_name: str,
    current_index: int,
    seen: dict[str, int],
    violations: list[str],
) -> None:
    previous_index = seen.get(identity)
    if previous_index is not None:
        violations.append(
            f"{display_path}: {field_label} {identity} "
            f"duplicates {identity_name} row {previous_index}"
        )
        return
    seen[identity] = current_index


def collect_component_dot_namespace_violations(
    display_path: str,
    label: str,
    value: str,
    violations: list[str],
) -> None:
    segments = value.split(".")
    if len(segments) < 2:
        violations.append(
            f"{display_path}: {label} {value} "
            "should use lowercase dot namespace form"
        )
    if any(not segment for segment in segments):
        violations.append(
            f"{display_path}: {label} {value} "
            "should not contain empty namespace segments"
        )
    if not all(
        char.isascii() and (char.islower() or char.isdigit() or char in {"_", "."})
        for char in value
    ):
        violations.append(
            f"{display_path}: {label} {value} should contain only "
            "lowercase ASCII letters, digits, underscores, and dots"
        )


def collect_required_trimmed_string_violation(
    display_path: str,
    field_label: str,
    table: dict[str, Any],
    field_name: str,
    violations: list[str],
) -> str | None:
    if field_name not in table:
        violations.append(f"{display_path}: {field_label} is required")
        return None
    value = table[field_name]
    if not isinstance(value, str) or not value.strip() or value.strip() != value:
        violations.append(
            f"{display_path}: {field_label} must be a non-empty trimmed string"
        )
        return None
    return value
