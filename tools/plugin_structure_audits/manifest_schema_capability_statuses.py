from __future__ import annotations

from typing import Any


CAPABILITY_STATUS_VALUES = (
    "complete",
    "partial",
    "stub",
    "externalized",
    "unsupported",
)
CAPABILITY_STATUS_FIELDS = frozenset(
    ("capability", "status", "note", "target_modes", "bevy_references")
)
CAPABILITY_STATUS_TARGET_MODES = (
    "client_runtime",
    "server_runtime",
    "editor_host",
)
CAPABILITY_STATUS_OWNERSHIP_MESSAGE = (
    "must reference a package or optional feature capability declared by the same package"
)
BEVY_REFERENCE_PREFIX = "dev/bevy/"


def collect_capability_statuses_schema_violations(
    display_path: str,
    manifest: dict[str, Any],
    capability_statuses: object,
    violations: list[str],
) -> None:
    if not isinstance(capability_statuses, list):
        violations.append(f"{display_path}: capability_statuses must be an array")
        return
    if not capability_statuses:
        violations.append(
            f"{display_path}: capability_statuses must not be empty when declared"
        )
        return

    owned_capabilities = capability_status_owned_capabilities(manifest)
    supported_targets = capability_status_supported_targets(manifest)
    seen_capabilities: dict[str, int] = {}
    for status_index, status in enumerate(capability_statuses):
        row_label = f"capability_statuses[{status_index}]"
        if not isinstance(status, dict):
            violations.append(f"{display_path}: {row_label} must be a table")
            continue
        capability = collect_capability_status_schema_violations(
            display_path,
            row_label,
            status,
            owned_capabilities,
            supported_targets,
            violations,
        )
        if capability is None:
            continue
        previous_index = seen_capabilities.get(capability)
        if previous_index is not None:
            violations.append(
                f"{display_path}: {row_label}.capability {capability} "
                f"duplicates capability_status capability_statuses[{previous_index}]"
            )
            continue
        seen_capabilities[capability] = status_index


def collect_capability_status_schema_violations(
    display_path: str,
    row_label: str,
    status: dict[str, Any],
    owned_capabilities: set[str],
    supported_targets: set[str],
    violations: list[str],
) -> str | None:
    collect_capability_status_known_field_violations(
        display_path,
        row_label,
        status,
        violations,
    )
    capability = collect_required_trimmed_string_violation(
        display_path,
        f"{row_label}.capability",
        status,
        "capability",
        violations,
    )
    if capability is not None:
        collect_capability_status_namespace_violations(
            display_path,
            f"{row_label}.capability",
            capability,
            violations,
        )
        if capability not in owned_capabilities:
            violations.append(
                f"{display_path}: {row_label}.capability {capability} "
                f"{CAPABILITY_STATUS_OWNERSHIP_MESSAGE}"
            )
    collect_capability_status_value_violations(
        display_path,
        f"{row_label}.status",
        status,
        violations,
    )
    collect_optional_trimmed_string_violation(
        display_path,
        f"{row_label}.note",
        status,
        "note",
        violations,
    )
    collect_capability_status_target_mode_violations(
        display_path,
        row_label,
        status,
        supported_targets,
        violations,
    )
    collect_capability_status_bevy_reference_violations(
        display_path,
        row_label,
        status,
        violations,
    )
    return capability


def collect_capability_status_known_field_violations(
    display_path: str,
    row_label: str,
    status: dict[str, Any],
    violations: list[str],
) -> None:
    for field in sorted(status):
        if field not in CAPABILITY_STATUS_FIELDS:
            violations.append(
                f"{display_path}: {row_label}.{field} "
                "is not a known capability_status field"
            )


def collect_required_trimmed_string_violation(
    display_path: str,
    field_label: str,
    row: dict[str, Any],
    field_name: str,
    violations: list[str],
) -> str | None:
    if field_name not in row:
        violations.append(f"{display_path}: {field_label} is required")
        return None
    value = row[field_name]
    if not isinstance(value, str) or not value.strip() or value.strip() != value:
        violations.append(
            f"{display_path}: {field_label} must be a non-empty trimmed string"
        )
        return None
    return value


def collect_optional_trimmed_string_violation(
    display_path: str,
    field_label: str,
    row: dict[str, Any],
    field_name: str,
    violations: list[str],
) -> None:
    if field_name not in row:
        return
    value = row[field_name]
    if not isinstance(value, str) or not value.strip() or value.strip() != value:
        violations.append(
            f"{display_path}: {field_label} must be a non-empty trimmed string"
        )


def collect_capability_status_value_violations(
    display_path: str,
    field_label: str,
    status: dict[str, Any],
    violations: list[str],
) -> None:
    status_value = collect_required_trimmed_string_violation(
        display_path,
        field_label,
        status,
        "status",
        violations,
    )
    if status_value is None:
        return
    if status_value not in CAPABILITY_STATUS_VALUES:
        violations.append(
            f"{display_path}: {field_label} {status_value} should be one of "
            + ", ".join(CAPABILITY_STATUS_VALUES)
        )


def collect_capability_status_namespace_violations(
    display_path: str,
    label: str,
    value: str,
    violations: list[str],
) -> None:
    segments = value.split(".")
    if len(segments) < 2:
        violations.append(
            f"{display_path}: {label} {value} "
            "should use package.module dot namespace form"
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


def collect_capability_status_target_mode_violations(
    display_path: str,
    row_label: str,
    status: dict[str, Any],
    supported_targets: set[str],
    violations: list[str],
) -> None:
    target_modes = capability_status_string_array(
        display_path,
        f"{row_label}.target_modes",
        status,
        "target_modes",
        violations,
    )
    if target_modes is None:
        return
    seen: dict[str, int] = {}
    allowed_targets = set(CAPABILITY_STATUS_TARGET_MODES)
    expected = ", ".join(CAPABILITY_STATUS_TARGET_MODES)
    for target_index, target_mode in enumerate(target_modes):
        item_label = f"{row_label}.target_modes[{target_index}]"
        if target_mode not in allowed_targets:
            violations.append(
                f'{display_path}: {item_label} "{target_mode}" '
                f"is unsupported; expected one of {expected}"
            )
            continue
        previous_index = seen.get(target_mode)
        if previous_index is not None:
            violations.append(
                f"{display_path}: {item_label} {target_mode} "
                "duplicates capability_status target_modes "
                f"target_modes[{previous_index}]"
            )
        else:
            seen[target_mode] = target_index
        if supported_targets and target_mode not in supported_targets:
            violations.append(
                f"{display_path}: {item_label} {target_mode} "
                "should be covered by package supported_targets"
            )


def collect_capability_status_bevy_reference_violations(
    display_path: str,
    row_label: str,
    status: dict[str, Any],
    violations: list[str],
) -> None:
    references = capability_status_string_array(
        display_path,
        f"{row_label}.bevy_references",
        status,
        "bevy_references",
        violations,
    )
    if references is None:
        return
    seen: dict[str, int] = {}
    for reference_index, reference in enumerate(references):
        item_label = f"{row_label}.bevy_references[{reference_index}]"
        collect_capability_status_bevy_reference_path_violations(
            display_path,
            item_label,
            reference,
            violations,
        )
        previous_index = seen.get(reference)
        if previous_index is not None:
            violations.append(
                f"{display_path}: {item_label} {reference} "
                "duplicates capability_status bevy_references "
                f"bevy_references[{previous_index}]"
            )
            continue
        seen[reference] = reference_index


def collect_capability_status_bevy_reference_path_violations(
    display_path: str,
    item_label: str,
    reference: str,
    violations: list[str],
) -> None:
    if "\\" in reference or ":" in reference:
        violations.append(
            f"{display_path}: {item_label} {reference} "
            "should use repository-relative forward-slash paths"
        )
        return
    if not reference.startswith(BEVY_REFERENCE_PREFIX):
        violations.append(
            f"{display_path}: {item_label} {reference} should start with dev/bevy/"
        )
    if any(segment in {"", ".", ".."} for segment in reference.split("/")):
        violations.append(
            f"{display_path}: {item_label} {reference} "
            "should not contain empty, current, or parent path segments"
        )


def capability_status_string_array(
    display_path: str,
    field_label: str,
    row: dict[str, Any],
    field_name: str,
    violations: list[str],
) -> list[str] | None:
    if field_name not in row:
        return None
    value = row[field_name]
    if not isinstance(value, list):
        violations.append(f"{display_path}: {field_label} must be an array")
        return None
    if not value:
        violations.append(
            f"{display_path}: {field_label} must not be empty when declared"
        )
        return None
    values: list[str] = []
    for item_index, item in enumerate(value):
        if not isinstance(item, str) or not item.strip() or item.strip() != item:
            violations.append(
                f"{display_path}: {field_label}[{item_index}] "
                "must be a non-empty trimmed string"
            )
            continue
        values.append(item)
    return values


def capability_status_owned_capabilities(manifest: dict[str, Any]) -> set[str]:
    owned_capabilities: set[str] = set()
    capability_status_extend_owned(owned_capabilities, manifest.get("capabilities"))
    optional_features = manifest.get("optional_features")
    if isinstance(optional_features, list):
        for feature in optional_features:
            if isinstance(feature, dict):
                capability_status_extend_owned(
                    owned_capabilities,
                    feature.get("capabilities"),
                )
    return owned_capabilities


def capability_status_extend_owned(
    owned_capabilities: set[str],
    capabilities: object,
) -> None:
    if not isinstance(capabilities, list):
        return
    for capability in capabilities:
        if (
            isinstance(capability, str)
            and capability.strip()
            and capability.strip() == capability
        ):
            owned_capabilities.add(capability)


def capability_status_supported_targets(manifest: dict[str, Any]) -> set[str]:
    supported_targets = manifest.get("supported_targets")
    if not isinstance(supported_targets, list):
        return set()
    return {
        target
        for target in supported_targets
        if isinstance(target, str) and target.strip() and target.strip() == target
    }
