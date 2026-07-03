from __future__ import annotations

from typing import Any, Iterable


ManifestEntry = tuple[str, dict[str, Any]]


def collect_global_manifest_identity_violations(
    manifests: Iterable[ManifestEntry],
    violations: list[str],
) -> None:
    seen_asset_importer_ids: dict[str, str] = {}
    seen_option_keys: dict[str, str] = {}
    for display_path, manifest in manifests:
        collect_global_asset_importer_id_violations(
            display_path,
            manifest,
            seen_asset_importer_ids,
            violations,
        )
        collect_global_option_key_violations(
            display_path,
            manifest,
            seen_option_keys,
            violations,
        )


def collect_global_asset_importer_id_violations(
    display_path: str,
    manifest: dict[str, Any],
    seen: dict[str, str],
    violations: list[str],
) -> None:
    asset_importers = manifest.get("asset_importers")
    if not isinstance(asset_importers, list):
        return
    package_label = global_package_label(manifest) or display_path
    for index, importer in enumerate(asset_importers):
        if not isinstance(importer, dict):
            continue
        importer_id = importer.get("id")
        if not is_non_empty_trimmed_string(importer_id):
            continue
        label = f"{package_label} asset_importers[{index}].id"
        previous = seen.get(importer_id)
        if previous is not None:
            violations.append(
                f"plugin validate asset_importers id {importer_id} "
                f"is duplicated by {previous} and {label}"
            )
            continue
        seen[importer_id] = label


def collect_global_option_key_violations(
    display_path: str,
    manifest: dict[str, Any],
    seen: dict[str, str],
    violations: list[str],
) -> None:
    options = manifest.get("options")
    if not isinstance(options, list):
        return
    package_label = global_package_label(manifest) or display_path
    for index, option in enumerate(options):
        if not isinstance(option, dict):
            continue
        key = option.get("key")
        if not is_non_empty_trimmed_string(key):
            continue
        label = f"{package_label} options[{index}].key"
        previous = seen.get(key)
        if previous is not None:
            violations.append(
                f"plugin validate options key {key} "
                f"is duplicated by {previous} and {label}"
            )
            continue
        seen[key] = label


def global_package_label(manifest: dict[str, Any]) -> str | None:
    package_id = manifest.get("id")
    if not is_non_empty_trimmed_string(package_id):
        return None
    return f"plugin {package_id}"


def is_non_empty_trimmed_string(value: object) -> bool:
    return isinstance(value, str) and bool(value.strip()) and value.strip() == value
