from __future__ import annotations

from typing import Any, Iterable


ManifestEntry = tuple[str, dict[str, Any]]


def collect_global_event_catalog_namespace_violations(
    manifests: Iterable[ManifestEntry],
    violations: list[str],
) -> None:
    seen_namespaces: dict[str, str] = {}
    for display_path, manifest in manifests:
        collect_manifest_event_catalog_namespace_violations(
            display_path,
            manifest,
            seen_namespaces,
            violations,
        )


def collect_manifest_event_catalog_namespace_violations(
    display_path: str,
    manifest: dict[str, Any],
    seen_namespaces: dict[str, str],
    violations: list[str],
) -> None:
    event_catalogs = manifest.get("event_catalogs")
    if not isinstance(event_catalogs, list):
        return
    package_label = event_catalog_package_label(manifest) or display_path
    for catalog_index, catalog in enumerate(event_catalogs):
        if not isinstance(catalog, dict):
            continue
        namespace = catalog.get("namespace")
        if not is_non_empty_trimmed_string(namespace):
            continue
        label = f"{package_label} event_catalogs[{catalog_index}].namespace"
        previous = seen_namespaces.get(namespace)
        if previous is not None:
            violations.append(
                f"plugin validate event_catalog namespace {namespace} "
                f"is duplicated by {previous} and {label}"
            )
            continue
        seen_namespaces[namespace] = label


def event_catalog_package_label(manifest: dict[str, Any]) -> str | None:
    package_id = manifest.get("id")
    if not is_non_empty_trimmed_string(package_id):
        return None
    return f"plugin {package_id}"


def is_non_empty_trimmed_string(value: object) -> bool:
    return isinstance(value, str) and bool(value.strip()) and value.strip() == value
