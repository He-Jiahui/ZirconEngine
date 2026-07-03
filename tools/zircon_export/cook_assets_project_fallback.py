"""CookAssets project manifest fallback and direct-reference closure helpers."""

from __future__ import annotations

import re
import tomllib
from pathlib import Path
from typing import Any

from .cook_assets_manifest import (
    is_safe_asset_package_path,
    load_cooked_asset_manifest,
    manifest_with_default_asset_filter,
    normalized_asset_package_path,
    normalized_cooked_asset_manifest,
    resolve_asset_source_path,
    sorted_cooked_asset_manifest_entries,
)
from .export_template_manifest import is_safe_relative_path, normalize_relative_path

RES_ASSET_REFERENCE_RE = re.compile(
    r"res://[A-Za-z0-9_./\\-]+(?:#[A-Za-z0-9_./\\-]+)?"
)


def project_default_scene_manifest(
    project_manifest: Path,
    default_asset_filter: str | None,
    diagnostics: list[str],
) -> tuple[dict[str, Any] | None, str | None, Path | None]:
    document = load_project_manifest(project_manifest, diagnostics)
    if document is None:
        return None, None, None

    default_scene = document.get("default_scene")
    if not isinstance(default_scene, str) or not default_scene:
        diagnostics.append(
            f"project manifest {project_manifest} needs a non-empty default_scene"
        )
        return None, None, None

    project_source_manifest = project_asset_manifest_path(
        project_manifest,
        document,
        diagnostics,
    )
    if "asset_manifest" in document:
        if project_source_manifest is None:
            return None, default_scene, None
        manifest = load_cooked_asset_manifest(project_source_manifest, diagnostics)
        if manifest is not None and not diagnostics:
            manifest = normalized_cooked_asset_manifest(
                manifest,
                project_source_manifest.parent,
                diagnostics,
            )
            manifest = manifest_with_default_asset_filter(manifest, default_asset_filter)
        return manifest, default_scene, project_source_manifest

    package_path = project_asset_package_path(default_scene, diagnostics)
    if package_path is None:
        return None, default_scene, None

    source_path = project_manifest.parent / "assets" / Path(*package_path.split("/"))
    resolved_source_path = resolve_asset_source_path(
        package_path,
        source_path,
        diagnostics,
    )
    asset: dict[str, Any] = {
        "path": package_path,
        "source": str(resolved_source_path or source_path),
        "dependencies": [],
        "labels": [],
    }
    direct_dependencies, referenced_assets = project_direct_reference_assets(
        project_manifest.parent,
        package_path,
        resolved_source_path or source_path,
        default_asset_filter,
        diagnostics,
    )
    if direct_dependencies:
        asset["dependencies"] = direct_dependencies
    assets = sorted_cooked_asset_manifest_entries([asset, *referenced_assets])
    manifest: dict[str, Any] = {
        "roots": [package_path],
        "assets": assets,
    }
    if default_asset_filter:
        asset["labels"] = [default_asset_filter]
        manifest["asset_filter"] = default_asset_filter
    return manifest, default_scene, None


def project_asset_manifest_path(
    project_manifest: Path,
    document: dict[str, Any],
    diagnostics: list[str],
) -> Path | None:
    value = document.get("asset_manifest")
    if value is None:
        return None
    if not isinstance(value, str) or not value.strip():
        diagnostics.append(
            f"project manifest {project_manifest} asset_manifest must be a non-empty path"
        )
        return None
    if value != value.strip():
        diagnostics.append(
            f"project manifest {project_manifest} asset_manifest must be a trimmed path"
        )
        return None
    normalized = normalize_relative_path(value)
    if not normalized or not is_safe_relative_path(normalized):
        diagnostics.append(
            f"project manifest {project_manifest} asset_manifest {value} "
            "must be a safe relative path"
        )
        return None
    return resolve_asset_source_path(
        "project asset_manifest",
        project_manifest.parent / Path(*normalized.split("/")),
        diagnostics,
    )


def project_direct_reference_assets(
    project_root: Path,
    root_asset_path: str,
    root_source_path: Path,
    default_asset_filter: str | None,
    diagnostics: list[str],
) -> tuple[list[str], list[dict[str, Any]]]:
    references = project_direct_res_asset_references(root_source_path, diagnostics)
    direct_dependencies = sorted(
        reference_path
        for reference_path in references
        if reference_path != root_asset_path
    )
    assets: list[dict[str, Any]] = []
    queued_references = list(references)
    seen_assets = {root_asset_path}
    while queued_references:
        reference_path = queued_references.pop(0)
        if reference_path in seen_assets:
            continue
        seen_assets.add(reference_path)
        if reference_path == root_asset_path:
            continue
        source_path = project_root / "assets" / Path(*reference_path.split("/"))
        resolved_source_path = resolve_asset_source_path(
            reference_path,
            source_path,
            diagnostics,
        )
        asset: dict[str, Any] = {
            "path": reference_path,
            "source": str(resolved_source_path or source_path),
            "dependencies": [],
            "labels": [],
        }
        if default_asset_filter:
            asset["labels"] = [default_asset_filter]
        child_references = project_direct_res_asset_references(
            resolved_source_path or source_path,
            diagnostics,
        )
        if child_references:
            asset["dependencies"] = sorted(
                child_path
                for child_path in child_references
                if child_path != reference_path
            )
            queued_references.extend(child_references)
        assets.append(asset)
    return direct_dependencies, assets


def project_direct_res_asset_references(
    source_path: Path,
    diagnostics: list[str],
) -> list[str]:
    if not source_path.exists():
        return []
    if not source_path.is_file():
        diagnostics.append(f"asset source {source_path} is not a file")
        return []
    try:
        contents = source_path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        return []
    except OSError as error:
        diagnostics.append(f"asset source {source_path} could not be read: {error}")
        return []
    references: list[str] = []
    seen: set[str] = set()
    for match in RES_ASSET_REFERENCE_RE.finditer(contents):
        asset_path = project_asset_reference_package_path(match.group(0), diagnostics)
        if asset_path is None or asset_path in seen:
            continue
        seen.add(asset_path)
        references.append(asset_path)
    return sorted(references)


def project_asset_reference_package_path(
    asset_uri: str,
    diagnostics: list[str],
) -> str | None:
    asset_without_fragment = asset_uri.split("#", 1)[0]
    return project_asset_package_path(
        asset_without_fragment,
        diagnostics,
        label="project asset reference",
    )


def load_project_manifest(
    project_manifest: Path,
    diagnostics: list[str],
) -> dict[str, Any] | None:
    if not project_manifest.exists():
        diagnostics.append(
            f"project manifest {project_manifest} does not exist and --asset-manifest was not supplied"
        )
        return None
    if not project_manifest.is_file():
        diagnostics.append(f"project manifest {project_manifest} is not a file")
        return None
    try:
        document = tomllib.loads(project_manifest.read_text(encoding="utf-8"))
    except OSError as error:
        diagnostics.append(f"project manifest {project_manifest} could not be read: {error}")
        return None
    except tomllib.TOMLDecodeError as error:
        diagnostics.append(f"project manifest {project_manifest} is not valid TOML: {error}")
        return None
    if not isinstance(document, dict):
        diagnostics.append(f"project manifest {project_manifest} must decode to a table")
        return None
    return document


def project_asset_package_path(
    asset_uri: str,
    diagnostics: list[str],
    *,
    label: str = "project default_scene",
) -> str | None:
    if not asset_uri.startswith("res://"):
        diagnostics.append(
            f"{label} {asset_uri} must use a res:// asset URI for CookAssets fallback"
        )
        return None
    package_path = asset_uri[len("res://") :]
    if not is_safe_asset_package_path(package_path):
        diagnostics.append(
            f"{label} {asset_uri} does not resolve to a safe asset path"
        )
        return None
    return normalized_asset_package_path(package_path)
