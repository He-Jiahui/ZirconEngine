from __future__ import annotations

from typing import Any


ALLOWED_SCRIPT_HOST_VALUE_KINDS = (
    "null",
    "bool",
    "int",
    "float",
    "string",
    "bytes",
    "host_handle",
)
PROVIDED_INTERFACE_FIELDS = frozenset(("id", "methods"))
INTERFACE_METHOD_FIELDS = frozenset(
    (
        "documentation",
        "method_slot",
        "name",
        "parameters",
        "required_capabilities",
        "return_value_kind",
    )
)
INTERFACE_METHOD_PARAMETER_FIELDS = frozenset(("name", "type_ref", "value_kind"))
INTERFACE_METHOD_TYPE_REF_FIELDS = frozenset(("type_name", "value_kind"))


def collect_provided_interfaces_schema_violations(
    display_path: str,
    manifest: dict[str, Any],
    violations: list[str],
) -> None:
    interfaces = manifest.get("provides_interfaces")
    if interfaces is None:
        return
    if not isinstance(interfaces, list):
        violations.append(f"{display_path}: provides_interfaces must be an array")
        return
    if not interfaces:
        violations.append(
            f"{display_path}: provides_interfaces must not be empty when declared"
        )
        return

    seen_interfaces: dict[str, int] = {}
    for interface_index, interface in enumerate(interfaces):
        row_label = f"provides_interfaces[{interface_index}]"
        if not isinstance(interface, dict):
            violations.append(f"{display_path}: {row_label} must be a table")
            continue
        collect_interface_known_field_violations(
            display_path,
            row_label,
            interface,
            PROVIDED_INTERFACE_FIELDS,
            "provided interface",
            violations,
        )
        collect_interface_methods_schema_violations(
            display_path,
            f"{row_label}.methods",
            interface.get("methods"),
            violations,
        )
        interface_id = collect_interface_trimmed_string_violation(
            display_path,
            f"{row_label}.id",
            interface,
            "id",
            violations,
        )
        if interface_id is None:
            continue
        collect_interface_dot_namespace_violations(
            display_path,
            f"{row_label}.id",
            interface_id,
            violations,
        )
        previous_index = seen_interfaces.get(interface_id)
        if previous_index is not None:
            violations.append(
                f"{display_path}: {row_label}.id {interface_id} "
                f"duplicates provided interface id provides_interfaces[{previous_index}]"
            )
            continue
        seen_interfaces[interface_id] = interface_index


def collect_interface_methods_schema_violations(
    display_path: str,
    methods_label: str,
    methods: object,
    violations: list[str],
) -> None:
    if methods is None:
        return
    if not isinstance(methods, list):
        violations.append(f"{display_path}: {methods_label} must be an array")
        return
    if not methods:
        violations.append(
            f"{display_path}: {methods_label} must not be empty when declared"
        )
        return

    seen_names: dict[str, int] = {}
    seen_slots: dict[int, int] = {}
    for method_index, method in enumerate(methods):
        method_label = f"{methods_label}[{method_index}]"
        if not isinstance(method, dict):
            violations.append(f"{display_path}: {method_label} must be a table")
            continue
        collect_interface_known_field_violations(
            display_path,
            method_label,
            method,
            INTERFACE_METHOD_FIELDS,
            "provided interface method",
            violations,
        )
        method_name = collect_interface_method_token_field_violations(
            display_path,
            method_label,
            method,
            "name",
            violations,
        )
        method_slot = collect_interface_method_slot_violations(
            display_path,
            f"{method_label}.method_slot",
            method.get("method_slot"),
            violations,
        )
        if method_name is not None:
            previous_index = seen_names.get(method_name)
            if previous_index is not None:
                violations.append(
                    f"{display_path}: {method_label}.name {method_name} "
                    f"duplicates provided interface method methods[{previous_index}]"
                )
            else:
                seen_names[method_name] = method_index
        if method_slot is not None:
            previous_index = seen_slots.get(method_slot)
            if previous_index is not None:
                violations.append(
                    f"{display_path}: {method_label}.method_slot {method_slot} "
                    "duplicates provided interface method_slot "
                    f"methods[{previous_index}]"
                )
            else:
                seen_slots[method_slot] = method_index
        collect_interface_method_return_kind_violations(
            display_path,
            method,
            f"{method_label}.return_value_kind",
            violations,
        )
        collect_interface_method_parameters_schema_violations(
            display_path,
            f"{method_label}.parameters",
            method.get("parameters"),
            violations,
        )
        collect_interface_method_required_capabilities_violations(
            display_path,
            f"{method_label}.required_capabilities",
            method.get("required_capabilities"),
            violations,
        )
        collect_interface_optional_trimmed_string_violation(
            display_path,
            f"{method_label}.documentation",
            method,
            "documentation",
            violations,
        )


def collect_interface_method_parameters_schema_violations(
    display_path: str,
    parameters_label: str,
    parameters: object,
    violations: list[str],
) -> None:
    if parameters is None:
        return
    if not isinstance(parameters, list):
        violations.append(f"{display_path}: {parameters_label} must be an array")
        return

    seen_names: dict[str, int] = {}
    for parameter_index, parameter in enumerate(parameters):
        parameter_label = f"{parameters_label}[{parameter_index}]"
        if not isinstance(parameter, dict):
            violations.append(f"{display_path}: {parameter_label} must be a table")
            continue
        collect_interface_method_parameter_signature_violations(
            display_path,
            parameter_label,
            parameter,
            violations,
        )
        parameter_name = collect_interface_method_token_field_violations(
            display_path,
            parameter_label,
            parameter,
            "name",
            violations,
        )
        if parameter_name is None:
            continue
        previous_index = seen_names.get(parameter_name)
        if previous_index is not None:
            violations.append(
                f"{display_path}: {parameter_label}.name {parameter_name} "
                f"duplicates interface method parameter parameters[{previous_index}]"
            )
        else:
            seen_names[parameter_name] = parameter_index


def collect_interface_method_parameter_signature_violations(
    display_path: str,
    parameter_label: str,
    parameter: dict[str, Any],
    violations: list[str],
) -> None:
    collect_interface_known_field_violations(
        display_path,
        parameter_label,
        parameter,
        INTERFACE_METHOD_PARAMETER_FIELDS,
        "interface method parameter",
        violations,
    )
    collect_interface_signature_required_value_kind_violations(
        display_path,
        f"{parameter_label}.value_kind",
        parameter,
        "value_kind",
        violations,
    )
    collect_interface_method_type_ref_violations(
        display_path,
        f"{parameter_label}.type_ref",
        parameter.get("type_ref"),
        violations,
    )


def collect_interface_method_type_ref_violations(
    display_path: str,
    type_ref_label: str,
    type_ref: object,
    violations: list[str],
) -> None:
    if type_ref is None:
        return
    if not isinstance(type_ref, dict):
        violations.append(f"{display_path}: {type_ref_label} must be a table")
        return
    collect_interface_known_field_violations(
        display_path,
        type_ref_label,
        type_ref,
        INTERFACE_METHOD_TYPE_REF_FIELDS,
        "interface method type_ref",
        violations,
    )
    collect_interface_signature_required_value_kind_violations(
        display_path,
        f"{type_ref_label}.value_kind",
        type_ref,
        "value_kind",
        violations,
    )
    collect_interface_trimmed_string_violation(
        display_path,
        f"{type_ref_label}.type_name",
        type_ref,
        "type_name",
        violations,
    )


def collect_interface_known_field_violations(
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


def collect_interface_method_return_kind_violations(
    display_path: str,
    method: dict[str, Any],
    field_label: str,
    violations: list[str],
) -> None:
    if "return_value_kind" not in method:
        return
    collect_interface_signature_value_kind_violations(
        display_path,
        field_label,
        method.get("return_value_kind"),
        violations,
    )


def collect_interface_signature_required_value_kind_violations(
    display_path: str,
    field_label: str,
    table: dict[str, Any],
    field_name: str,
    violations: list[str],
) -> str | None:
    if field_name not in table:
        violations.append(f"{display_path}: {field_label} is required")
        return None
    return collect_interface_signature_value_kind_violations(
        display_path,
        field_label,
        table.get(field_name),
        violations,
    )


def collect_interface_signature_value_kind_violations(
    display_path: str,
    field_label: str,
    value: object,
    violations: list[str],
) -> str | None:
    if not isinstance(value, str) or not value.strip() or value.strip() != value:
        violations.append(
            f"{display_path}: {field_label} must be a non-empty trimmed string"
        )
        return None
    if value not in ALLOWED_SCRIPT_HOST_VALUE_KINDS:
        expected = ", ".join(ALLOWED_SCRIPT_HOST_VALUE_KINDS)
        violations.append(
            f"{display_path}: {field_label} {value} is unsupported; expected "
            f"one of {expected}"
        )
    return value


def collect_interface_method_required_capabilities_violations(
    display_path: str,
    capabilities_label: str,
    capabilities: object,
    violations: list[str],
) -> None:
    if capabilities is None:
        return
    if not isinstance(capabilities, list):
        violations.append(f"{display_path}: {capabilities_label} must be an array")
        return
    if not capabilities:
        violations.append(
            f"{display_path}: {capabilities_label} must not be empty when declared"
        )
        return

    seen: dict[str, int] = {}
    for capability_index, capability in enumerate(capabilities):
        capability_label = f"{capabilities_label}[{capability_index}]"
        if (
            not isinstance(capability, str)
            or not capability.strip()
            or capability.strip() != capability
        ):
            violations.append(
                f"{display_path}: {capability_label} "
                "must be a non-empty trimmed string"
            )
            continue
        collect_interface_dot_namespace_violations(
            display_path,
            capability_label,
            capability,
            violations,
        )
        previous_index = seen.get(capability)
        if previous_index is not None:
            violations.append(
                f"{display_path}: {capability_label} {capability} "
                "duplicates required capability "
                f"required_capabilities[{previous_index}]"
            )
        else:
            seen[capability] = capability_index


def collect_interface_method_token_field_violations(
    display_path: str,
    row_label: str,
    table: dict[str, Any],
    field_name: str,
    violations: list[str],
) -> str | None:
    value = collect_interface_trimmed_string_violation(
        display_path,
        f"{row_label}.{field_name}",
        table,
        field_name,
        violations,
    )
    if value is None:
        return None
    if not all(
        char.isascii() and (char.islower() or char.isdigit() or char == "_")
        for char in value
    ):
        violations.append(
            f"{display_path}: {row_label}.{field_name} {value} should contain "
            "only lowercase ASCII letters, digits, and underscores"
        )
    return value


def collect_interface_method_slot_violations(
    display_path: str,
    field_label: str,
    value: object,
    violations: list[str],
) -> int | None:
    if type(value) is not int or value < 0:
        violations.append(
            f"{display_path}: {field_label} must be a non-negative integer"
        )
        return None
    return value


def collect_interface_dot_namespace_violations(
    display_path: str,
    field_label: str,
    value: str,
    violations: list[str],
) -> None:
    segments = value.split(".")
    if len(segments) < 2:
        violations.append(
            f"{display_path}: {field_label} {value} "
            "should use package.module dot namespace form"
        )
    if any(not segment for segment in segments):
        violations.append(
            f"{display_path}: {field_label} {value} "
            "should not contain empty namespace segments"
        )
    if not all(
        char.isascii() and (char.islower() or char.isdigit() or char in {"_", "."})
        for char in value
    ):
        violations.append(
            f"{display_path}: {field_label} {value} should contain only "
            "lowercase ASCII letters, digits, underscores, and dots"
        )


def collect_interface_optional_trimmed_string_violation(
    display_path: str,
    field_label: str,
    table: dict[str, Any],
    field_name: str,
    violations: list[str],
) -> str | None:
    if field_name not in table:
        return None
    return collect_interface_trimmed_string_violation(
        display_path,
        field_label,
        table,
        field_name,
        violations,
    )


def collect_interface_trimmed_string_violation(
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
