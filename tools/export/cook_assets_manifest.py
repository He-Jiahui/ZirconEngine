"""CookAssets asset manifest loading, schema, and normalization."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from .export_template_manifest import is_safe_relative_path, normalize_relative_path


ASSET_MANIFEST_FIELDS = ("asset_filter", "assets", "roots")
ASSET_MANIFEST_ASSET_FIELDS = ("dependencies", "labels", "path", "source")


def load_cooked_asset_manifest(
    source_manifest: Path,
    diagnostics: list[str],
) -> dict[str, Any] | None:
    if not source_manifest.exists():
        diagnostics.append(f"asset manifest {source_manifest} does not exist")
        return None
    if not source_manifest.is_file():
        diagnostics.append(f"asset manifest {source_manifest} is not a file")
        return None
    try:
        manifest = json.loads(source_manifest.read_text(encoding="utf-8"))
    except OSError as error:
        diagnostics.append(f"asset manifest {source_manifest} could not be read: {error}")
        return None
    except json.JSONDecodeError as error:
        diagnostics.append(f"asset manifest {source_manifest} is not valid JSON: {error}")
        return None
    if not isinstance(manifest, dict):
        diagnostics.append("asset manifest root must be a JSON object")
        return None

    validate_asset_manifest_shape(manifest, diagnostics)
    return manifest


def validate_asset_manifest_shape(
    manifest: dict[str, Any],
    diagnostics: list[str],
) -> None:
    diagnostics.extend(
        table_unknown_field_diagnostics(
            "asset manifest",
            manifest,
            ASSET_MANIFEST_FIELDS,
        )
    )
    roots = manifest.get("roots", [])
    if not isinstance(roots, list):
        diagnostics.append("asset manifest field roots must be a string array")
    else:
        for index, root in enumerate(roots):
            if not isinstance(root, str):
                diagnostics.append(
                    f"asset manifest field roots entry {index} must be a string"
                )
            elif not root.strip():
                diagnostics.append(
                    f"asset manifest field roots entry {index} must be a non-empty string"
                )
            elif not is_safe_asset_package_path(root):
                diagnostics.append(
                    f"asset manifest field roots entry {index} "
                    "must be a safe relative asset path"
                )

    asset_filter = manifest.get("asset_filter")
    if asset_filter is not None and not isinstance(asset_filter, str):
        diagnostics.append("asset manifest field asset_filter must be a string when present")
    elif isinstance(asset_filter, str) and not asset_filter.strip():
        diagnostics.append(
            "asset manifest field asset_filter must be a non-empty string when present"
        )

    assets = manifest.get("assets")
    if not isinstance(assets, list):
        diagnostics.append("asset manifest field assets must be an array")
        return

    seen_paths: set[str] = set()
    for index, asset in enumerate(assets):
        if not isinstance(asset, dict):
            diagnostics.append(f"asset manifest entry {index} must be an object")
            continue
        diagnostics.extend(
            table_unknown_field_diagnostics(
                f"asset manifest entry {index}",
                asset,
                ASSET_MANIFEST_ASSET_FIELDS,
            )
        )
        path = asset.get("path")
        if not isinstance(path, str) or not path.strip():
            diagnostics.append(f"asset manifest entry {index} needs a non-empty path")
        else:
            normalized_path = normalized_asset_package_path(path)
            if not is_safe_asset_package_path(path):
                diagnostics.append(
                    f"asset manifest entry {index} path must be a safe relative asset path"
                )
            elif normalized_path in seen_paths:
                diagnostics.append(
                    f"asset manifest path {normalized_path} is declared more than once"
                )
            else:
                seen_paths.add(normalized_path)

        validate_optional_string(asset, "source", index, diagnostics)
        validate_optional_string_array(asset, "dependencies", index, diagnostics)
        validate_optional_string_array(asset, "labels", index, diagnostics)

    validate_asset_manifest_reference_closure(manifest, diagnostics)


def normalized_cooked_asset_manifest(
    manifest: dict[str, Any],
    source_manifest_dir: Path,
    diagnostics: list[str],
) -> dict[str, Any]:
    normalized = dict(manifest)
    asset_filter = normalized.get("asset_filter")
    if isinstance(asset_filter, str):
        normalized["asset_filter"] = asset_filter.strip()
    roots = normalized.get("roots")
    if isinstance(roots, list):
        normalized["roots"] = sorted(
            set(normalized_asset_package_path(root) for root in roots)
        )
    normalized_assets: list[dict[str, Any]] = []
    for index, asset in enumerate(manifest.get("assets", [])):
        normalized_asset = dict(asset)
        asset_path = normalized_asset.get("path")
        if isinstance(asset_path, str):
            normalized_asset["path"] = normalized_asset_package_path(asset_path)
        dependencies = normalized_asset.get("dependencies")
        if isinstance(dependencies, list):
            normalized_asset["dependencies"] = sorted(
                set(normalized_asset_package_path(dependency) for dependency in dependencies)
            )
        labels = normalized_asset.get("labels")
        if isinstance(labels, list):
            normalized_asset["labels"] = sorted(set(label.strip() for label in labels))
        source = normalized_asset.get("source")
        if isinstance(source, str) and source:
            source = source.strip()
            normalized_asset["source"] = source
            source_path = Path(source)
            if not source_path.is_absolute():
                normalized_asset_path = normalized_asset.get("path")
                if not isinstance(normalized_asset_path, str) or not normalized_asset_path:
                    normalized_asset_path = f"entry {index}"
                resolved_source_path = resolve_asset_source_path(
                    normalized_asset_path,
                    source_manifest_dir / source_path,
                    diagnostics,
                )
                if resolved_source_path:
                    normalized_asset["source"] = str(resolved_source_path)
        normalized_assets.append(normalized_asset)
    normalized["assets"] = sorted_cooked_asset_manifest_entries(normalized_assets)
    return normalized


def sorted_cooked_asset_manifest_entries(
    assets: list[dict[str, Any]],
) -> list[dict[str, Any]]:
    return sorted(assets, key=lambda asset: str(asset.get("path", "")))


def resolve_asset_source_path(
    asset_path: str,
    source_path: Path,
    diagnostics: list[str],
) -> Path | None:
    try:
        return source_path.resolve()
    except OSError as error:
        diagnostics.append(
            f"asset source for {asset_path} could not be resolved: {error}"
        )
        return None


def manifest_with_default_asset_filter(
    manifest: dict[str, Any],
    default_asset_filter: str | None,
) -> dict[str, Any]:
    if not default_asset_filter or manifest.get("asset_filter") is not None:
        return manifest
    with_filter = dict(manifest)
    with_filter["asset_filter"] = default_asset_filter.strip()
    return with_filter


def validate_asset_sources_exist(
    manifest: dict[str, Any],
    diagnostics: list[str],
) -> None:
    for index, asset in enumerate(manifest.get("assets", [])):
        if not isinstance(asset, dict):
            continue
        source = asset.get("source")
        if not isinstance(source, str) or not source:
            continue
        source_path = Path(source)
        if source_path.is_file():
            continue
        asset_path = asset.get("path")
        if not isinstance(asset_path, str) or not asset_path:
            asset_path = f"entry {index}"
        if source_path.exists():
            diagnostics.append(
                f"asset source for {asset_path} is not a file: {source_path}"
            )
            continue
        diagnostics.append(f"asset source for {asset_path} does not exist: {source_path}")


def validate_optional_string(
    asset: dict[str, Any],
    field_name: str,
    index: int,
    diagnostics: list[str],
) -> None:
    value = asset.get(field_name)
    if value is not None and not isinstance(value, str):
        diagnostics.append(f"asset manifest entry {index} field {field_name} must be a string")
    elif isinstance(value, str) and not value.strip():
        diagnostics.append(
            f"asset manifest entry {index} field {field_name} "
            "must be a non-empty string when present"
        )


def validate_optional_string_array(
    asset: dict[str, Any],
    field_name: str,
    index: int,
    diagnostics: list[str],
) -> None:
    value = asset.get(field_name, [])
    if not isinstance(value, list):
        diagnostics.append(f"asset manifest entry {index} field {field_name} must be a string array")
        return
    type_diagnostics = [
        f"asset manifest entry {index} field {field_name} entry {entry_index} "
        "must be a string"
        for entry_index, item in enumerate(value)
        if not isinstance(item, str)
    ]
    if type_diagnostics:
        diagnostics.extend(type_diagnostics)
        return
    for entry_index, item in enumerate(value):
        if not item.strip():
            diagnostics.append(
                f"asset manifest entry {index} field {field_name} entry {entry_index} "
                "must be a non-empty string"
            )
        elif field_name == "dependencies" and not is_safe_asset_package_path(item):
            diagnostics.append(
                f"asset manifest entry {index} field {field_name} entry {entry_index} "
                "must be a safe relative asset path"
            )


def is_safe_asset_package_path(value: str) -> bool:
    normalized = normalize_relative_path(value)
    return bool(normalized) and is_safe_relative_path(normalized)


def normalized_asset_package_path(value: str) -> str:
    return normalize_relative_path(value)


def table_unknown_field_diagnostics(
    label: str,
    table: dict[str, Any],
    known_fields: tuple[str, ...],
) -> list[str]:
    known_field_set = set(known_fields)
    return [
        f"{label} unknown field {field}"
        for field in sorted(table)
        if field not in known_field_set
    ]


def validate_asset_manifest_reference_closure(
    manifest: dict[str, Any],
    diagnostics: list[str],
) -> None:
    assets = manifest.get("assets")
    roots = manifest.get("roots", [])
    if not isinstance(assets, list) or not isinstance(roots, list):
        return
    if any(not isinstance(root, str) for root in roots):
        return
    asset_paths = {
        normalized_path
        for asset in assets
        if isinstance(asset, dict)
        for normalized_path in [safe_normalized_manifest_path(asset.get("path"))]
        if normalized_path is not None
    }
    for root in roots:
        normalized_root = safe_normalized_manifest_path(root)
        if normalized_root is None or normalized_root in asset_paths:
            continue
        diagnostics.append(
            f"asset manifest root {normalized_root} is not declared in assets"
        )
    for index, asset in enumerate(assets):
        if not isinstance(asset, dict):
            continue
        dependencies = asset.get("dependencies", [])
        if not isinstance(dependencies, list):
            continue
        for dependency in dependencies:
            normalized_dependency = safe_normalized_manifest_path(dependency)
            if normalized_dependency is None or normalized_dependency in asset_paths:
                continue
            diagnostics.append(
                f"asset manifest entry {index} dependency "
                f"{normalized_dependency} is not declared in assets"
            )


def safe_normalized_manifest_path(value: object) -> str | None:
    if not isinstance(value, str) or not value.strip():
        return None
    if not is_safe_asset_package_path(value):
        return None
    return normalized_asset_package_path(value)
