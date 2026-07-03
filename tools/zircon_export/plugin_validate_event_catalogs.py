"""Event catalog validation for standalone plugin manifests."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .native_build_workspace import read_toml
from .plugin_validate_common import plugin_validate_trimmed_string

Diagnostics = list[str]
Manifest = dict[str, Any]
NamespaceOwners = dict[str, str]

PLUGIN_VALIDATE_EVENT_CATALOG_FIELDS = frozenset({"namespace", "version", "events"})
PLUGIN_VALIDATE_EVENT_FIELDS = frozenset({"id", "display_name", "payload_schema"})


def validate_plugin_event_catalogs(
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
    catalogs = manifest.get("event_catalogs")
    if catalogs is None:
        return
    label = f"plugin {package_id} event_catalogs"
    if not isinstance(catalogs, list):
        diagnostics.append(f"{label} must be an array")
        return
    if not catalogs:
        diagnostics.append(f"{label} must not be empty when declared")
        return

    namespace_owners = plugin_validate_event_catalog_namespace_index(
        plugin_root, plugin_manifest_path, diagnostics
    )
    seen_namespaces: dict[str, int] = {}
    for index, catalog in enumerate(catalogs):
        catalog_label = f"{label}[{index}]"
        if not isinstance(catalog, dict):
            diagnostics.append(f"{catalog_label} must be a table")
            continue
        validate_plugin_event_known_fields(
            catalog, PLUGIN_VALIDATE_EVENT_CATALOG_FIELDS, catalog_label, "event catalog", diagnostics
        )
        namespace = validate_plugin_event_catalog_row(
            catalog,
            catalog_label,
            package_id,
            namespace_owners,
            diagnostics,
        )
        if namespace is None:
            continue
        previous_index = seen_namespaces.get(namespace)
        if previous_index is not None:
            diagnostics.append(
                f"{catalog_label}.namespace {namespace} "
                f"duplicates event catalog namespace row {previous_index}"
            )
            continue
        seen_namespaces[namespace] = index


def validate_plugin_event_catalog_row(
    catalog: Manifest,
    catalog_label: str,
    package_id: str,
    namespace_owners: NamespaceOwners,
    diagnostics: Diagnostics,
) -> str | None:
    namespace = plugin_validate_trimmed_string(
        catalog, "namespace", f"{catalog_label}.namespace", diagnostics
    )
    if namespace is not None:
        plugin_validate_dot_namespace(
            namespace, f"{catalog_label}.namespace", diagnostics
        )
        expected_prefix = f"{package_id}."
        if not namespace.startswith(expected_prefix):
            diagnostics.append(
                f"{catalog_label}.namespace {namespace} "
                f"should stay under package namespace {expected_prefix}"
            )
        owner = namespace_owners.get(namespace)
        if owner is not None:
            diagnostics.append(
                f"{catalog_label}.namespace {namespace} duplicates event catalog "
                f"namespace declared by {owner}"
            )

    version = catalog.get("version")
    if type(version) is not int:
        diagnostics.append(f"{catalog_label}.version must be an integer")
    elif version <= 0 or version > 0xFFFF_FFFF:
        diagnostics.append(
            f"{catalog_label}.version {version} should be a positive u32"
        )

    validate_plugin_event_rows(
        catalog.get("events"),
        f"{catalog_label}.events",
        package_id,
        namespace,
        diagnostics,
    )
    return namespace


def validate_plugin_event_rows(
    events: Any,
    events_label: str,
    package_id: str,
    namespace: str | None,
    diagnostics: Diagnostics,
) -> None:
    if events is None:
        diagnostics.append(f"{events_label} is required")
        return
    if not isinstance(events, list):
        diagnostics.append(f"{events_label} must be an array")
        return
    if not events:
        diagnostics.append(f"{events_label} must not be empty when declared")
        return

    seen_events: dict[str, int] = {}
    for index, event in enumerate(events):
        event_label = f"{events_label}[{index}]"
        if not isinstance(event, dict):
            diagnostics.append(f"{event_label} must be a table")
            continue
        validate_plugin_event_known_fields(event, PLUGIN_VALIDATE_EVENT_FIELDS, event_label, "event", diagnostics)
        event_id = plugin_validate_trimmed_string(
            event, "id", f"{event_label}.id", diagnostics
        )
        if event_id is not None:
            plugin_validate_dot_namespace(event_id, f"{event_label}.id", diagnostics)
            if namespace is not None and not event_id.startswith(f"{namespace}."):
                diagnostics.append(
                    f"{event_label}.id {event_id} should stay under namespace "
                    f"{namespace}."
                )
            previous_index = seen_events.get(event_id)
            if previous_index is not None:
                diagnostics.append(
                    f"{event_label}.id duplicates event row {previous_index}"
                )
            else:
                seen_events[event_id] = index
        plugin_validate_trimmed_string(
            event, "display_name", f"{event_label}.display_name", diagnostics
        )
        validate_plugin_event_payload_schema(
            event, f"{event_label}.payload_schema", package_id, diagnostics
        )


def validate_plugin_event_known_fields(
    table: Manifest,
    known_fields: frozenset[str],
    row_label: str,
    field_label: str,
    diagnostics: Diagnostics,
) -> None:
    for field in sorted(table):
        if field not in known_fields:
            diagnostics.append(f"{row_label}.{field} is not a known {field_label} field")


def validate_plugin_event_payload_schema(
    event: Manifest,
    label: str,
    package_id: str,
    diagnostics: Diagnostics,
) -> None:
    if "payload_schema" not in event:
        return
    payload_schema = plugin_validate_trimmed_string(
        event, "payload_schema", label, diagnostics
    )
    if payload_schema is None:
        return
    plugin_validate_dot_namespace(payload_schema, label, diagnostics)
    expected_prefix = f"{package_id}."
    if not payload_schema.startswith(expected_prefix):
        diagnostics.append(
            f"{label} {payload_schema} should stay under package namespace "
            f"{expected_prefix}"
        )
    plugin_validate_payload_schema_version(payload_schema, label, diagnostics)


def plugin_validate_dot_namespace(
    value: str,
    label: str,
    diagnostics: Diagnostics,
) -> None:
    segments = value.split(".")
    if len(segments) < 2:
        diagnostics.append(
            f"{label} {value} should use at least two dot-separated namespace segments"
        )
    if any(not segment for segment in segments):
        diagnostics.append(f"{label} {value} should not contain empty namespace segments")
    if not all(
        byte.isascii() and (byte.islower() or byte.isdigit() or byte in {"_", "."})
        for byte in value
    ):
        diagnostics.append(
            f"{label} {value} should contain only lowercase ASCII letters, "
            "digits, underscores, and dots"
        )


def plugin_validate_payload_schema_version(
    payload_schema: str,
    label: str,
    diagnostics: Diagnostics,
) -> None:
    version_segment = payload_schema.rsplit(".", 1)[-1]
    if not version_segment.startswith("v"):
        diagnostics.append(
            f"{label} {payload_schema} should end with a version segment like v1"
        )
        return
    version_number = version_segment[1:]
    if not version_number:
        diagnostics.append(
            f"{label} {payload_schema} version segment should include digits"
        )
        return
    if not version_number.isdigit():
        diagnostics.append(
            f"{label} {payload_schema} version segment should contain only digits after v"
        )
        return
    if version_number.startswith("0"):
        diagnostics.append(
            f"{label} {payload_schema} version segment should be a positive integer "
            "without leading zeroes"
        )


def plugin_validate_event_catalog_namespace_index(
    plugin_root: Path | None,
    current_manifest_path: Path,
    diagnostics: Diagnostics,
) -> NamespaceOwners:
    owners: NamespaceOwners = {}
    if plugin_root is None or not plugin_root.exists():
        return owners
    current = current_manifest_path.resolve()
    for manifest_path in sorted(plugin_root.rglob("plugin.toml")):
        if manifest_path.resolve() == current:
            continue
        manifest = read_toml(manifest_path, diagnostics)
        if manifest is None:
            continue
        owner = manifest.get("id")
        if not isinstance(owner, str) or not owner.strip() or owner.strip() != owner:
            continue
        catalogs = manifest.get("event_catalogs")
        if not isinstance(catalogs, list):
            continue
        for catalog in catalogs:
            if not isinstance(catalog, dict):
                continue
            namespace = catalog.get("namespace")
            if isinstance(namespace, str) and namespace.strip() == namespace:
                owners.setdefault(namespace, owner)
    return owners
