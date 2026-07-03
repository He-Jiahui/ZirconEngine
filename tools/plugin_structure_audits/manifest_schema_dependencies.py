from __future__ import annotations

from typing import Any


DEPENDENCY_FIELDS = frozenset(("id", "required", "capability", "interfaces"))
DependencyIdentity = tuple[str, str]


def collect_dependencies_schema_violations(
    display_path: str,
    manifest: dict[str, Any],
    violations: list[str],
) -> None:
    dependencies = manifest.get("dependencies")
    if dependencies is None:
        return
    if not isinstance(dependencies, list):
        violations.append(f"{display_path}: dependencies must be an array")
        return
    if not dependencies:
        violations.append(
            f"{display_path}: dependencies must not be empty when declared"
        )
        return

    seen: dict[DependencyIdentity, int] = {}
    for dependency_index, dependency in enumerate(dependencies):
        row_label = f"dependencies[{dependency_index}]"
        if not isinstance(dependency, dict):
            violations.append(f"{display_path}: {row_label} must be a table")
            continue
        collect_dependency_known_field_violations(
            display_path,
            row_label,
            dependency,
            violations,
        )
        identity = collect_dependency_row_schema_violations(
            display_path,
            row_label,
            dependency,
            violations,
        )
        if identity is None:
            continue
        previous_index = seen.get(identity)
        if previous_index is not None:
            violations.append(
                f"{display_path}: {row_label} duplicates dependency row "
                f"{previous_index}"
            )
            continue
        seen[identity] = dependency_index


def collect_dependency_row_schema_violations(
    display_path: str,
    row_label: str,
    dependency: dict[str, Any],
    violations: list[str],
) -> DependencyIdentity | None:
    dependency_id = collect_dependency_trimmed_string_violation(
        display_path,
        f"{row_label}.id",
        dependency,
        "id",
        violations,
    )
    if type(dependency.get("required")) is not bool:
        violations.append(f"{display_path}: {row_label}.required must be a bool")

    if "capability" in dependency:
        capability = collect_dependency_trimmed_string_violation(
            display_path,
            f"{row_label}.capability",
            dependency,
            "capability",
            violations,
        )
        if dependency_id is None or capability is None:
            return None
        return (dependency_id, f"capability:{capability}")

    interfaces = collect_dependency_interfaces_violations(
        display_path,
        f"{row_label}.interfaces",
        dependency.get("interfaces"),
        violations,
    )
    if dependency_id is None or interfaces is None:
        return None
    return (dependency_id, "interfaces:" + ",".join(interfaces))


def collect_dependency_known_field_violations(
    display_path: str,
    row_label: str,
    dependency: dict[str, Any],
    violations: list[str],
) -> None:
    for field in sorted(dependency):
        if field not in DEPENDENCY_FIELDS:
            violations.append(
                f"{display_path}: {row_label}.{field} "
                "is not a known dependency field"
            )


def collect_dependency_interfaces_violations(
    display_path: str,
    interfaces_label: str,
    interfaces: object,
    violations: list[str],
) -> list[str] | None:
    if not isinstance(interfaces, list) or not interfaces:
        violations.append(
            f"{display_path}: {interfaces_label} must be a non-empty string array"
        )
        return None

    values: list[str] = []
    seen: dict[str, int] = {}
    valid = True
    for interface_index, interface in enumerate(interfaces):
        item_label = f"{interfaces_label}[{interface_index}]"
        if (
            not isinstance(interface, str)
            or not interface.strip()
            or interface.strip() != interface
        ):
            violations.append(
                f"{display_path}: {item_label} must be a non-empty trimmed string"
            )
            valid = False
            continue
        collect_dependency_interface_namespace_violations(
            display_path,
            item_label,
            interface,
            violations,
        )
        previous_index = seen.get(interface)
        if previous_index is not None:
            violations.append(
                f"{display_path}: {item_label} {interface} "
                f"duplicates dependency interface interfaces[{previous_index}]"
            )
        else:
            seen[interface] = interface_index
        values.append(interface)
    if not valid:
        return None
    return values


def collect_dependency_interface_namespace_violations(
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


def collect_dependency_trimmed_string_violation(
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
