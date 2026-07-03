"""Plugin asset root discovery for zircon_build."""

from __future__ import annotations

from pathlib import Path
from typing import Any, Iterable

try:
    from .zircon_export.plugin_validate_distribution_assets import (
        plugin_validate_distribution_assets,
    )
except ImportError:  # pragma: no cover - exercised when zircon_build.py is run directly.
    from zircon_export.plugin_validate_distribution_assets import (
        plugin_validate_distribution_assets,
    )


def collect_plugin_asset_roots(
    manifest_path: Path,
    data: dict[str, Any],
    distribution: dict[str, Any],
    plugin_id: str,
) -> tuple[Path, ...]:
    validate_plugin_distribution_assets_for_build(
        manifest_path,
        distribution,
        plugin_id,
    )
    roots: list[Path] = []
    if "asset_roots" in data:
        asset_root_values = data["asset_roots"]
        if isinstance(asset_root_values, list) and not asset_root_values:
            asset_root_values = ["assets"]
    else:
        asset_root_values = ["assets"]
    append_plugin_asset_roots_from_field(
        roots,
        manifest_path,
        asset_root_values,
        "asset_roots",
    )
    append_plugin_asset_roots_from_distribution_assets(
        roots,
        manifest_path,
        distribution.get("assets", []),
    )
    existing_roots = [root for root in roots if root.exists() and root.is_dir()]
    return tuple(unique_asset_roots(existing_roots))


def validate_plugin_distribution_assets_for_build(
    manifest_path: Path,
    distribution: dict[str, Any],
    plugin_id: str,
) -> None:
    diagnostics: list[str] = []
    plugin_validate_distribution_assets(
        distribution,
        plugin_id,
        diagnostics,
        plugin_manifest_path=manifest_path,
    )
    if diagnostics:
        raise SystemExit("; ".join(diagnostics))


def append_plugin_asset_roots_from_field(
    roots: list[Path], manifest_path: Path, values: object, field: str
) -> None:
    if values is None:
        return
    if not isinstance(values, list):
        raise SystemExit(f"{manifest_path}: {field} must be a list.")
    if not values:
        return
    for index, value in enumerate(values, start=1):
        root = normalized_plugin_asset_root(manifest_path, value, f"{field}[{index}]")
        roots.append(root)


def append_plugin_asset_roots_from_distribution_assets(
    roots: list[Path], manifest_path: Path, values: object
) -> None:
    if values is None:
        return
    if not isinstance(values, list):
        raise SystemExit(f"{manifest_path}: distribution.assets must be a list.")
    if not values:
        return
    for index, value in enumerate(values, start=1):
        root_text = distribution_asset_root_text(value)
        if root_text is None:
            continue
        roots.append(
            normalized_plugin_asset_root(
                manifest_path,
                root_text,
                f"distribution.assets[{index}]",
            )
        )


def distribution_asset_root_text(value: object) -> str | None:
    text = str(value).strip().replace("\\", "/")
    if not text:
        return None
    wildcard_index = min(
        (index for index in (text.find("*"), text.find("?")) if index >= 0),
        default=-1,
    )
    if wildcard_index >= 0:
        text = text[:wildcard_index]
    if not text:
        return None
    if text.endswith("/"):
        text = text.rstrip("/")
    else:
        text = str(Path(text).parent).replace("\\", "/")
        if text == ".":
            return None
    return text or None


def normalized_plugin_asset_root(
    manifest_path: Path, value: object, field: str
) -> Path:
    text = str(value).strip()
    if not text:
        raise SystemExit(f"{manifest_path}: {field} must not be empty.")
    relative = Path(text)
    if relative.is_absolute():
        raise SystemExit(f"{manifest_path}: {field} must be relative to the package root.")
    if any(part in ("", ".", "..") for part in relative.parts):
        raise SystemExit(
            f"{manifest_path}: {field} must not contain empty, current, or parent segments."
        )
    return manifest_path.parent / relative


def unique_asset_roots(paths: Iterable[Path]) -> list[Path]:
    seen: set[str] = set()
    result: list[Path] = []
    for path in paths:
        key = str(path)
        if key in seen:
            continue
        seen.add(key)
        result.append(path)
    return result
