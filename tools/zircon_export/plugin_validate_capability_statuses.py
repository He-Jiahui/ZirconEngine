"""Capability status validation for standalone plugin manifests."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .native_build_workspace import read_toml
from .plugin_validate_capability_status_references import validate_plugin_capability_status_references
from .plugin_validate_capability_status_targets import plugin_validate_capability_status_supported_targets, validate_plugin_capability_status_targets
from .plugin_validate_common import plugin_validate_optional_trimmed_string, plugin_validate_trimmed_string

Diagnostics = list[str]
Manifest = dict[str, Any]

PLUGIN_VALIDATE_CAPABILITY_STATUS_VALUES = ("complete", "partial", "stub", "externalized", "unsupported")
PLUGIN_VALIDATE_CAPABILITY_STATUS_FIELDS = frozenset(("capability", "status", "note", "target_modes", "bevy_references"))
PLUGIN_VALIDATE_CAPABILITY_STATUS_OWNERSHIP_MESSAGE = "must reference a package or optional feature capability declared by the same package"


def validate_plugin_capability_statuses(
    *,
    plugin_manifest_path: Path | None,
    package_id: str,
    diagnostics: Diagnostics,
) -> None:
    if plugin_manifest_path is None:
        return
    manifest = read_toml(plugin_manifest_path, diagnostics)
    if manifest is None:
        return
    validate_plugin_capability_status_rows(
        manifest.get("capability_statuses"),
        f"plugin {package_id} capability_statuses",
        plugin_validate_capability_status_owned_capabilities(manifest),
        plugin_validate_capability_status_supported_targets(manifest),
        diagnostics,
    )


def plugin_validate_capability_status_owned_capabilities(manifest: Manifest) -> set[str]:
    owned_capabilities: set[str] = set()
    plugin_validate_capability_status_extend_owned(
        owned_capabilities, manifest.get("capabilities")
    )
    optional_features = manifest.get("optional_features")
    if isinstance(optional_features, list):
        for feature in optional_features:
            if isinstance(feature, dict):
                plugin_validate_capability_status_extend_owned(
                    owned_capabilities, feature.get("capabilities")
                )
    return owned_capabilities


def plugin_validate_capability_status_extend_owned(owned_capabilities: set[str], capabilities: Any) -> None:
    if not isinstance(capabilities, list):
        return
    for capability in capabilities:
        if isinstance(capability, str) and capability.strip() and capability.strip() == capability:
            owned_capabilities.add(capability)


def validate_plugin_capability_status_rows(
    statuses: Any,
    label: str,
    owned_capabilities: set[str],
    supported_targets: set[str],
    diagnostics: Diagnostics,
) -> None:
    if statuses is None:
        return
    if not isinstance(statuses, list):
        diagnostics.append(f"{label} must be an array")
        return
    if not statuses:
        diagnostics.append(f"{label} must not be empty when declared")
        return
    seen_capabilities: dict[str, int] = {}
    for index, status in enumerate(statuses):
        row_label = f"{label}[{index}]"
        if not isinstance(status, dict):
            diagnostics.append(f"{row_label} must be a table")
            continue
        validate_plugin_capability_status_known_fields(status, row_label, diagnostics)
        capability = validate_plugin_capability_status_row(
            status, row_label, owned_capabilities, supported_targets, diagnostics
        )
        if capability is None:
            continue
        previous_index = seen_capabilities.get(capability)
        if previous_index is not None:
            diagnostics.append(
                f"{row_label}.capability {capability} duplicates capability_status "
                f"capability_statuses[{previous_index}]"
            )
        else:
            seen_capabilities[capability] = index


def validate_plugin_capability_status_known_fields(
    status: Manifest, row_label: str, diagnostics: Diagnostics,
) -> None:
    for field in sorted(status):
        if field not in PLUGIN_VALIDATE_CAPABILITY_STATUS_FIELDS:
            diagnostics.append(f"{row_label}.{field} is not a known capability_status field")


def validate_plugin_capability_status_row(
    status: Manifest,
    row_label: str,
    owned_capabilities: set[str],
    supported_targets: set[str],
    diagnostics: Diagnostics,
) -> str | None:
    capability = plugin_validate_trimmed_string(
        status, "capability", f"{row_label}.capability", diagnostics
    )
    if capability is not None:
        validate_plugin_capability_status_namespace(
            capability, f"{row_label}.capability", diagnostics
        )
        if capability not in owned_capabilities:
            diagnostics.append(
                f"{row_label}.capability {capability} "
                f"{PLUGIN_VALIDATE_CAPABILITY_STATUS_OWNERSHIP_MESSAGE}"
            )
    validate_plugin_capability_status_value(
        status, "status", f"{row_label}.status", diagnostics
    )
    plugin_validate_optional_trimmed_string(
        status, "note", f"{row_label}.note", diagnostics
    )
    validate_plugin_capability_status_targets(
        status, row_label, supported_targets, diagnostics
    )
    validate_plugin_capability_status_references(status, row_label, diagnostics)
    return capability


def validate_plugin_capability_status_value(
    status: Manifest,
    field: str,
    label: str,
    diagnostics: Diagnostics,
) -> str | None:
    status_value = plugin_validate_trimmed_string(status, field, label, diagnostics)
    if status_value is None:
        return None
    if status_value not in PLUGIN_VALIDATE_CAPABILITY_STATUS_VALUES:
        diagnostics.append(
            f"{label} {status_value} should be one of "
            + ", ".join(PLUGIN_VALIDATE_CAPABILITY_STATUS_VALUES)
        )
        return None
    return status_value


def validate_plugin_capability_status_namespace(
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
