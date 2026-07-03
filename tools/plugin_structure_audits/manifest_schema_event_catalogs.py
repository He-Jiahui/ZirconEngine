from __future__ import annotations

from typing import Any


EVENT_CATALOG_FIELDS = frozenset(("namespace", "version", "events"))
EVENT_FIELDS = frozenset(("id", "display_name", "payload_schema"))
MAX_U32 = 0xFFFF_FFFF


def collect_event_catalogs_schema_violations(
    display_path: str,
    manifest: dict[str, Any],
    violations: list[str],
) -> None:
    event_catalogs = manifest.get("event_catalogs")
    if event_catalogs is None:
        return
    if not isinstance(event_catalogs, list):
        violations.append(f"{display_path}: event_catalogs must be an array")
        return
    if not event_catalogs:
        violations.append(
            f"{display_path}: event_catalogs must not be empty when declared"
        )
        return

    package_id = manifest.get("id")
    seen_namespaces: dict[str, int] = {}
    for catalog_index, catalog in enumerate(event_catalogs):
        row_label = f"event_catalogs[{catalog_index}]"
        if not isinstance(catalog, dict):
            violations.append(f"{display_path}: {row_label} must be a table")
            continue
        collect_event_catalog_known_field_violations(
            display_path,
            row_label,
            catalog,
            EVENT_CATALOG_FIELDS,
            "event catalog",
            violations,
        )
        namespace = collect_event_catalog_row_schema_violations(
            display_path,
            row_label,
            catalog,
            package_id,
            violations,
        )
        if namespace is None:
            continue
        previous_index = seen_namespaces.get(namespace)
        if previous_index is not None:
            violations.append(
                f"{display_path}: {row_label}.namespace {namespace} "
                f"duplicates event catalog namespace row {previous_index}"
            )
            continue
        seen_namespaces[namespace] = catalog_index


def collect_event_catalog_row_schema_violations(
    display_path: str,
    row_label: str,
    catalog: dict[str, Any],
    package_id: object,
    violations: list[str],
) -> str | None:
    namespace = collect_event_trimmed_string_violation(
        display_path,
        f"{row_label}.namespace",
        catalog,
        "namespace",
        violations,
    )
    if namespace is not None:
        collect_event_dot_namespace_violations(
            display_path,
            f"{row_label}.namespace",
            namespace,
            violations,
        )
        collect_event_package_namespace_violation(
            display_path,
            f"{row_label}.namespace",
            namespace,
            package_id,
            violations,
        )
    collect_event_catalog_version_violations(
        display_path,
        f"{row_label}.version",
        catalog.get("version"),
        violations,
    )
    collect_event_rows_schema_violations(
        display_path,
        f"{row_label}.events",
        catalog.get("events"),
        package_id,
        namespace,
        violations,
    )
    return namespace


def collect_event_rows_schema_violations(
    display_path: str,
    events_label: str,
    events: object,
    package_id: object,
    namespace: str | None,
    violations: list[str],
) -> None:
    if events is None:
        violations.append(f"{display_path}: {events_label} is required")
        return
    if not isinstance(events, list):
        violations.append(f"{display_path}: {events_label} must be an array")
        return
    if not events:
        violations.append(f"{display_path}: {events_label} must not be empty when declared")
        return

    seen_events: dict[str, int] = {}
    for event_index, event in enumerate(events):
        event_label = f"{events_label}[{event_index}]"
        if not isinstance(event, dict):
            violations.append(f"{display_path}: {event_label} must be a table")
            continue
        collect_event_catalog_known_field_violations(
            display_path,
            event_label,
            event,
            EVENT_FIELDS,
            "event",
            violations,
        )
        event_id = collect_event_trimmed_string_violation(
            display_path,
            f"{event_label}.id",
            event,
            "id",
            violations,
        )
        if event_id is not None:
            collect_event_dot_namespace_violations(
                display_path,
                f"{event_label}.id",
                event_id,
                violations,
            )
            if namespace is not None and not event_id.startswith(f"{namespace}."):
                violations.append(
                    f"{display_path}: {event_label}.id {event_id} "
                    f"should stay under namespace {namespace}."
                )
            previous_index = seen_events.get(event_id)
            if previous_index is not None:
                violations.append(
                    f"{display_path}: {event_label}.id duplicates event row "
                    f"{previous_index}"
                )
            else:
                seen_events[event_id] = event_index
        collect_event_trimmed_string_violation(
            display_path,
            f"{event_label}.display_name",
            event,
            "display_name",
            violations,
        )
        collect_event_payload_schema_violations(
            display_path,
            f"{event_label}.payload_schema",
            event,
            package_id,
            violations,
        )


def collect_event_catalog_known_field_violations(
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


def collect_event_catalog_version_violations(
    display_path: str,
    field_label: str,
    value: object,
    violations: list[str],
) -> None:
    if type(value) is not int:
        violations.append(f"{display_path}: {field_label} must be an integer")
        return
    if value <= 0 or value > MAX_U32:
        violations.append(
            f"{display_path}: {field_label} {value} should be a positive u32"
        )


def collect_event_payload_schema_violations(
    display_path: str,
    field_label: str,
    event: dict[str, Any],
    package_id: object,
    violations: list[str],
) -> None:
    if "payload_schema" not in event:
        return
    payload_schema = collect_event_trimmed_string_violation(
        display_path,
        field_label,
        event,
        "payload_schema",
        violations,
    )
    if payload_schema is None:
        return
    collect_event_dot_namespace_violations(
        display_path,
        field_label,
        payload_schema,
        violations,
    )
    collect_event_package_namespace_violation(
        display_path,
        field_label,
        payload_schema,
        package_id,
        violations,
    )
    collect_event_payload_schema_version_violations(
        display_path,
        field_label,
        payload_schema,
        violations,
    )


def collect_event_dot_namespace_violations(
    display_path: str,
    field_label: str,
    value: str,
    violations: list[str],
) -> None:
    segments = value.split(".")
    if len(segments) < 2:
        violations.append(
            f"{display_path}: {field_label} {value} should use at least two "
            "dot-separated namespace segments"
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


def collect_event_package_namespace_violation(
    display_path: str,
    field_label: str,
    value: str,
    package_id: object,
    violations: list[str],
) -> None:
    if not (
        isinstance(package_id, str)
        and package_id.strip()
        and package_id.strip() == package_id
    ):
        return
    expected_prefix = f"{package_id}."
    if not value.startswith(expected_prefix):
        violations.append(
            f"{display_path}: {field_label} {value} should stay under package "
            f"namespace {expected_prefix}"
        )


def collect_event_payload_schema_version_violations(
    display_path: str,
    field_label: str,
    payload_schema: str,
    violations: list[str],
) -> None:
    version_segment = payload_schema.rsplit(".", 1)[-1]
    if not version_segment.startswith("v"):
        violations.append(
            f"{display_path}: {field_label} {payload_schema} should end with "
            "a version segment like v1"
        )
        return
    version_number = version_segment[1:]
    if not version_number:
        violations.append(
            f"{display_path}: {field_label} {payload_schema} version segment "
            "should include digits"
        )
        return
    if not version_number.isdigit():
        violations.append(
            f"{display_path}: {field_label} {payload_schema} version segment "
            "should contain only digits after v"
        )
        return
    if version_number.startswith("0"):
        violations.append(
            f"{display_path}: {field_label} {payload_schema} version segment "
            "should be a positive integer without leading zeroes"
        )


def collect_event_trimmed_string_violation(
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
