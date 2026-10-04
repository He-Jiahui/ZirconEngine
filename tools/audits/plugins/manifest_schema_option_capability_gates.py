from __future__ import annotations

from typing import Any, Iterable

from .manifest_schema_asset_importer_capability_gates import (
    REQUIRED_CAPABILITY_DECLARED_GATE_DIAGNOSTIC,
    collect_declared_static_capabilities,
    is_non_empty_trimmed_string,
    required_capability_is_host_owned,
)


ManifestEntry = tuple[str, dict[str, Any]]


def collect_option_required_capability_gate_violations(
    manifests: Iterable[ManifestEntry],
    violations: list[str],
) -> None:
    manifest_entries = list(manifests)
    declared_capabilities = collect_declared_static_capabilities(manifest_entries)
    for display_path, manifest in manifest_entries:
        collect_manifest_option_required_capability_gate_violations(
            display_path,
            manifest,
            declared_capabilities,
            violations,
        )


def collect_manifest_option_required_capability_gate_violations(
    display_path: str,
    manifest: dict[str, Any],
    declared_capabilities: set[str],
    violations: list[str],
) -> None:
    options = manifest.get("options")
    if not isinstance(options, list):
        return
    for option_index, option in enumerate(options):
        if not isinstance(option, dict):
            continue
        collect_single_option_required_capability_gate_violations(
            display_path,
            f"options[{option_index}]",
            option,
            declared_capabilities,
            violations,
        )


def collect_single_option_required_capability_gate_violations(
    display_path: str,
    option_label: str,
    option: dict[str, Any],
    declared_capabilities: set[str],
    violations: list[str],
) -> None:
    if "required_capability" not in option:
        return
    value = option["required_capability"]
    if not is_non_empty_trimmed_string(value):
        return
    if value in declared_capabilities:
        return
    if required_capability_is_host_owned(value):
        return
    violations.append(
        f"{display_path}: {option_label}.required_capability {value} "
        f"{REQUIRED_CAPABILITY_DECLARED_GATE_DIAGNOSTIC}"
    )
