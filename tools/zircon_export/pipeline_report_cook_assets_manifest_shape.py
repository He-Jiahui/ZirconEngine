"""CookAssets staged manifest shape diagnostics for final pipeline reports."""

from __future__ import annotations

from typing import Any

from .export_template_manifest import is_safe_relative_path, normalize_relative_path

ASSET_MANIFEST_FIELDS = ("asset_filter", "assets", "roots")
ASSET_MANIFEST_ASSET_FIELDS = ("dependencies", "labels", "path", "source")


def cook_assets_manifest_shape_field_diagnostics(
    manifest: dict[str, Any],
) -> list[str]:
    diagnostics: list[str] = []
    diagnostics.extend(
        table_unknown_field_diagnostics(
            "cook_assets report cooked_asset_manifest",
            manifest,
            ASSET_MANIFEST_FIELDS,
        )
    )
    roots = manifest.get("roots", [])
    root_array_diagnostics = cook_assets_manifest_roots_array_schema_diagnostics(roots)
    diagnostics.extend(root_array_diagnostics)
    if not root_array_diagnostics:
        for index, root in enumerate(roots):
            if not root.strip():
                diagnostics.append(
                    "cook_assets report cooked_asset_manifest "
                    f"roots[{index}] must be a non-empty string"
                )
            elif root != root.strip():
                diagnostics.append(
                    "cook_assets report cooked_asset_manifest "
                    f"roots[{index}] must be a non-empty trimmed string"
                )
            elif not is_safe_asset_package_path(root):
                diagnostics.append(
                    "cook_assets report cooked_asset_manifest "
                    f"roots[{index}] must be a safe relative asset path"
                )
            elif root != normalized_asset_package_path(root):
                diagnostics.append(
                    "cook_assets report cooked_asset_manifest "
                    f"roots[{index}] must use a normalized relative asset path"
                )

    asset_filter = manifest.get("asset_filter")
    if asset_filter is not None and not isinstance(asset_filter, str):
        diagnostics.append(
            "cook_assets report cooked_asset_manifest asset_filter "
            "must be a string when present"
        )
    elif isinstance(asset_filter, str) and not asset_filter.strip():
        diagnostics.append(
            "cook_assets report cooked_asset_manifest asset_filter "
            "must be a non-empty string when present"
        )
    elif isinstance(asset_filter, str) and asset_filter != asset_filter.strip():
        diagnostics.append(
            "cook_assets report cooked_asset_manifest asset_filter "
            "must be a non-empty trimmed string when present"
        )

    assets = manifest.get("assets")
    if not isinstance(assets, list):
        diagnostics.append(
            "cook_assets report cooked_asset_manifest assets must be an array"
        )
        return diagnostics

    seen_paths: set[str] = set()
    for index, asset in enumerate(assets):
        if not isinstance(asset, dict):
            diagnostics.append(
                f"cook_assets report cooked_asset_manifest assets[{index}] "
                "must be an object"
            )
            continue
        diagnostics.extend(
            table_unknown_field_diagnostics(
                f"cook_assets report cooked_asset_manifest assets[{index}]",
                asset,
                ASSET_MANIFEST_ASSET_FIELDS,
            )
        )
        path = asset.get("path")
        if not isinstance(path, str) or not path.strip():
            diagnostics.append(
                f"cook_assets report cooked_asset_manifest assets[{index}].path "
                "must be a non-empty string"
            )
        elif path != path.strip():
            diagnostics.append(
                f"cook_assets report cooked_asset_manifest assets[{index}].path "
                "must be a non-empty trimmed string"
            )
        elif not is_safe_asset_package_path(path):
            diagnostics.append(
                f"cook_assets report cooked_asset_manifest assets[{index}].path "
                "must be a safe relative asset path"
            )
        else:
            normalized_path = normalized_asset_package_path(path)
            if path != normalized_path:
                diagnostics.append(
                    f"cook_assets report cooked_asset_manifest assets[{index}].path "
                    "must use a normalized relative asset path"
                )
            if normalized_path in seen_paths:
                diagnostics.append(
                    "cook_assets report cooked_asset_manifest asset path "
                    f"{normalized_path} is declared more than once"
                )
            else:
                seen_paths.add(normalized_path)
        diagnostics.extend(
            cook_assets_manifest_optional_string_diagnostics(
                asset,
                index,
                "source",
            )
        )
        diagnostics.extend(
            cook_assets_manifest_optional_string_array_diagnostics(
                asset,
                index,
                "dependencies",
            )
        )
        diagnostics.extend(
            cook_assets_manifest_optional_string_array_diagnostics(
                asset,
                index,
                "labels",
            )
        )
    diagnostics.extend(cook_assets_manifest_reference_closure_diagnostics(manifest))
    return diagnostics


def cook_assets_manifest_roots_array_schema_diagnostics(value: Any) -> list[str]:
    if not isinstance(value, list):
        return [
            "cook_assets report cooked_asset_manifest roots "
            "must be a string array"
        ]
    return [
        "cook_assets report cooked_asset_manifest "
        f"roots[{index}] must be a string"
        for index, root in enumerate(value)
        if not isinstance(root, str)
    ]


def cook_assets_manifest_optional_string_diagnostics(
    asset: dict[str, Any],
    index: int,
    field_name: str,
) -> list[str]:
    value = asset.get(field_name)
    if value is None:
        return []
    if isinstance(value, str):
        if not value.strip():
            return [
                f"cook_assets report cooked_asset_manifest assets[{index}].{field_name} "
                "must be a non-empty string when present"
            ]
        if value != value.strip():
            return [
                f"cook_assets report cooked_asset_manifest assets[{index}].{field_name} "
                "must be a non-empty trimmed string when present"
            ]
        return []
    return [
        f"cook_assets report cooked_asset_manifest assets[{index}].{field_name} "
        "must be a string"
    ]


def cook_assets_manifest_optional_string_array_diagnostics(
    asset: dict[str, Any],
    index: int,
    field_name: str,
) -> list[str]:
    value = asset.get(field_name, [])
    if not isinstance(value, list):
        return [
            f"cook_assets report cooked_asset_manifest assets[{index}].{field_name} "
            "must be a string array"
        ]
    diagnostics = [
        "cook_assets report cooked_asset_manifest "
        f"assets[{index}].{field_name}[{entry_index}] "
        "must be a string"
        for entry_index, item in enumerate(value)
        if not isinstance(item, str)
    ]
    if diagnostics:
        return diagnostics
    for entry_index, item in enumerate(value):
        if not item.strip():
            diagnostics.append(
                "cook_assets report cooked_asset_manifest "
                f"assets[{index}].{field_name}[{entry_index}] "
                "must be a non-empty string"
            )
        elif field_name == "dependencies" and item != item.strip():
            diagnostics.append(
                "cook_assets report cooked_asset_manifest "
                f"assets[{index}].{field_name}[{entry_index}] "
                "must be a non-empty trimmed string"
            )
        elif field_name == "dependencies" and not is_safe_asset_package_path(item):
            diagnostics.append(
                "cook_assets report cooked_asset_manifest "
                f"assets[{index}].{field_name}[{entry_index}] "
                "must be a safe relative asset path"
            )
        elif field_name == "dependencies" and item != normalized_asset_package_path(item):
            diagnostics.append(
                "cook_assets report cooked_asset_manifest "
                f"assets[{index}].{field_name}[{entry_index}] "
                "must use a normalized relative asset path"
            )
        elif field_name == "labels" and item != item.strip():
            diagnostics.append(
                "cook_assets report cooked_asset_manifest "
                f"assets[{index}].{field_name}[{entry_index}] "
                "must be a non-empty trimmed string"
            )
    return diagnostics


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


def cook_assets_manifest_roots_are_schema_clean(manifest: dict[str, Any]) -> bool:
    roots = manifest.get("roots", [])
    if not isinstance(roots, list):
        return False
    return all(safe_normalized_manifest_path(root) is not None for root in roots)


def cook_assets_manifest_assets_are_schema_clean(manifest: dict[str, Any]) -> bool:
    assets = manifest.get("assets")
    if not isinstance(assets, list):
        return False
    seen_paths: set[str] = set()
    for asset in assets:
        if not isinstance(asset, dict):
            return False
        if table_unknown_field_diagnostics(
            "cook_assets report cooked_asset_manifest asset",
            asset,
            ASSET_MANIFEST_ASSET_FIELDS,
        ):
            return False
        path = safe_normalized_manifest_path(asset.get("path"))
        if path is None:
            return False
        if path in seen_paths:
            return False
        seen_paths.add(path)
        if not cook_assets_manifest_optional_string_is_schema_clean(
            asset.get("source")
        ):
            return False
        if not cook_assets_manifest_string_array_is_schema_clean(
            asset.get("dependencies", []),
            require_asset_path=True,
        ):
            return False
        if not cook_assets_manifest_string_array_is_schema_clean(
            asset.get("labels", []),
            require_asset_path=False,
        ):
            return False
    return True


def cook_assets_manifest_optional_string_is_schema_clean(value: object) -> bool:
    return value is None or (
        isinstance(value, str)
        and bool(value.strip())
        and value == value.strip()
    )


def cook_assets_manifest_string_array_is_schema_clean(
    value: object,
    *,
    require_asset_path: bool,
) -> bool:
    if not isinstance(value, list):
        return False
    for item in value:
        if not isinstance(item, str) or not item.strip() or item != item.strip():
            return False
        if require_asset_path and safe_normalized_manifest_path(item) is None:
            return False
    return True


def cook_assets_manifest_reference_closure_diagnostics(
    manifest: dict[str, Any],
) -> list[str]:
    diagnostics: list[str] = []
    assets = manifest.get("assets")
    roots = manifest.get("roots", [])
    if not isinstance(assets, list) or not isinstance(roots, list):
        return diagnostics
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
            "cook_assets report cooked_asset_manifest root "
            f"{normalized_root} is not declared in assets"
        )
    for index, asset in enumerate(assets):
        if not isinstance(asset, dict):
            continue
        dependencies = asset.get("dependencies", [])
        if not isinstance(dependencies, list):
            continue
        for entry_index, dependency in enumerate(dependencies):
            normalized_dependency = safe_normalized_manifest_path(dependency)
            if normalized_dependency is None or normalized_dependency in asset_paths:
                continue
            diagnostics.append(
                "cook_assets report cooked_asset_manifest "
                f"assets[{index}].dependencies[{entry_index}] "
                f"{normalized_dependency} is not declared in assets"
            )
    return diagnostics


def safe_normalized_manifest_path(value: object) -> str | None:
    if not isinstance(value, str) or not value.strip() or value != value.strip():
        return None
    if not is_safe_asset_package_path(value):
        return None
    return normalized_asset_package_path(value)
