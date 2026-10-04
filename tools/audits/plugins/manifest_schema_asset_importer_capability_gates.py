from __future__ import annotations

from typing import Any, Iterable


ManifestEntry = tuple[str, dict[str, Any]]
REQUIRED_CAPABILITY_DECLARED_GATE_DIAGNOSTIC = (
    "should reference a declared static package/feature capability "
    "or an explicitly host-owned capability"
)


def collect_asset_importer_required_capability_gate_violations(
    manifests: Iterable[ManifestEntry],
    violations: list[str],
) -> None:
    manifest_entries = list(manifests)
    declared_capabilities = collect_declared_static_capabilities(manifest_entries)
    for display_path, manifest in manifest_entries:
        collect_manifest_asset_importer_required_capability_gate_violations(
            display_path,
            manifest,
            declared_capabilities,
            violations,
        )


def collect_declared_static_capabilities(
    manifests: Iterable[ManifestEntry],
) -> set[str]:
    capabilities: set[str] = set()
    for _display_path, manifest in manifests:
        collect_manifest_declared_capabilities(manifest, capabilities)
    return capabilities


def collect_manifest_declared_capabilities(
    manifest: dict[str, Any],
    capabilities: set[str],
) -> None:
    collect_declared_capability_values(manifest, capabilities)
    for feature in table_rows(manifest.get("optional_features")):
        collect_declared_capability_values(feature, capabilities)
    for extension in table_rows(manifest.get("feature_extensions")):
        collect_declared_capability_values(extension, capabilities)


def collect_declared_capability_values(
    table: dict[str, Any],
    capabilities: set[str],
) -> None:
    values = table.get("capabilities")
    if not isinstance(values, list):
        return
    for value in values:
        if is_non_empty_trimmed_string(value):
            capabilities.add(value)


def collect_manifest_asset_importer_required_capability_gate_violations(
    display_path: str,
    manifest: dict[str, Any],
    declared_capabilities: set[str],
    violations: list[str],
) -> None:
    asset_importers = manifest.get("asset_importers")
    if not isinstance(asset_importers, list):
        return
    for importer_index, importer in enumerate(asset_importers):
        if not isinstance(importer, dict):
            continue
        collect_single_asset_importer_required_capability_gate_violations(
            display_path,
            f"asset_importers[{importer_index}]",
            importer,
            declared_capabilities,
            violations,
        )


def collect_single_asset_importer_required_capability_gate_violations(
    display_path: str,
    importer_label: str,
    importer: dict[str, Any],
    declared_capabilities: set[str],
    violations: list[str],
) -> None:
    values = importer.get("required_capabilities")
    if not isinstance(values, list):
        return
    label = f"{importer_label}.required_capabilities"
    for value_index, value in enumerate(values):
        if not is_non_empty_trimmed_string(value):
            continue
        if value in declared_capabilities:
            continue
        if required_capability_is_host_owned(value):
            continue
        violations.append(
            f"{display_path}: {label}[{value_index}] {value} "
            f"{REQUIRED_CAPABILITY_DECLARED_GATE_DIAGNOSTIC}"
        )


def required_capability_is_host_owned(capability: str) -> bool:
    return (
        capability.startswith("runtime.capability.")
        or capability == "runtime.asset.importer.native"
    )


def table_rows(value: object) -> list[dict[str, Any]]:
    if not isinstance(value, list):
        return []
    return [row for row in value if isinstance(row, dict)]


def is_non_empty_trimmed_string(value: object) -> bool:
    return isinstance(value, str) and bool(value.strip()) and value.strip() == value
