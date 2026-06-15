"""CookAssets stage handoff for the Zircon export pipeline."""

from __future__ import annotations

import argparse
import json
import tomllib
from pathlib import Path
from typing import Any

REPORT_FILE_NAME = "report.json"
COOKED_ASSET_MANIFEST_NAME = "assets.json"


def run_cook_assets(args: argparse.Namespace) -> int:
    out_root = resolve_user_path(args.out)
    stage_dir = out_root / "stages" / "cook_assets"
    report_path = stage_dir / REPORT_FILE_NAME
    cooked_manifest = default_cooked_asset_manifest(out_root)
    source_manifest = resolve_user_path(args.asset_manifest) if args.asset_manifest else None
    project_manifest = resolve_user_path(args.project)
    default_asset_filter = getattr(args, "asset_filter", None)

    print(f"zircon_export stage=CookAssets profile={args.profile}")
    print(f"source_asset_manifest={source_manifest}")
    print(f"project_manifest={project_manifest}")
    print(f"cooked_asset_manifest={cooked_manifest}")
    if default_asset_filter:
        print(f"default_asset_filter={default_asset_filter}")
    print(f"report={report_path}")
    if args.dry_run:
        return 0 if source_manifest is not None or project_manifest.exists() else 2

    stage_dir.mkdir(parents=True, exist_ok=True)
    diagnostics: list[str] = []
    manifest: dict[str, Any] | None = None
    generated_from_project = False
    project_default_scene: str | None = None
    if source_manifest is None:
        manifest, project_default_scene = project_default_scene_manifest(
            project_manifest,
            default_asset_filter,
            diagnostics,
        )
        generated_from_project = manifest is not None
    else:
        manifest = load_cooked_asset_manifest(source_manifest, diagnostics)
        if manifest is not None and not diagnostics:
            manifest = normalized_cooked_asset_manifest(manifest, source_manifest.parent)
            manifest = manifest_with_default_asset_filter(manifest, default_asset_filter)
    if manifest is not None and not diagnostics:
        validate_asset_sources_exist(manifest, diagnostics)

    fatal = bool(diagnostics)
    if manifest is not None and not fatal:
        cooked_manifest.write_text(
            json.dumps(manifest, indent=2, sort_keys=True),
            encoding="utf-8",
        )

    report = {
        "stage": "CookAssets",
        "profile": args.profile,
        "fatal": fatal,
        "diagnostics": diagnostics,
        "source_asset_manifest": str(source_manifest) if source_manifest else None,
        "project_manifest": str(project_manifest),
        "generated_from_project": generated_from_project,
        "project_default_scene": project_default_scene,
        "cooked_asset_manifest": str(cooked_manifest),
        "asset_count": len(manifest.get("assets", [])) if manifest else 0,
        "root_count": len(manifest.get("roots", [])) if manifest else 0,
        "asset_filter": manifest.get("asset_filter") if manifest else None,
    }
    report_path.write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(json.dumps(report, indent=2))
    return 2 if fatal else 0


def default_cooked_asset_manifest(out_root: Path) -> Path:
    return out_root / "stages" / "cook_assets" / COOKED_ASSET_MANIFEST_NAME


def load_cooked_asset_manifest(
    source_manifest: Path,
    diagnostics: list[str],
) -> dict[str, Any] | None:
    if not source_manifest.exists():
        diagnostics.append(f"asset manifest {source_manifest} does not exist")
        return None
    try:
        manifest = json.loads(source_manifest.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        diagnostics.append(f"asset manifest {source_manifest} is not valid JSON: {error}")
        return None

    if not isinstance(manifest, dict):
        diagnostics.append("asset manifest root must be a JSON object")
        return None

    validate_asset_manifest_shape(manifest, diagnostics)
    return manifest


def project_default_scene_manifest(
    project_manifest: Path,
    default_asset_filter: str | None,
    diagnostics: list[str],
) -> tuple[dict[str, Any] | None, str | None]:
    document = load_project_manifest(project_manifest, diagnostics)
    if document is None:
        return None, None

    default_scene = document.get("default_scene")
    if not isinstance(default_scene, str) or not default_scene:
        diagnostics.append(
            f"project manifest {project_manifest} needs a non-empty default_scene"
        )
        return None, None

    package_path = project_asset_package_path(default_scene, diagnostics)
    if package_path is None:
        return None, default_scene

    source_path = project_manifest.parent / "assets" / Path(*package_path.split("/"))
    asset: dict[str, Any] = {
        "path": package_path,
        "source": str(source_path.resolve()),
        "dependencies": [],
        "labels": [],
    }
    manifest: dict[str, Any] = {
        "roots": [package_path],
        "assets": [asset],
    }
    if default_asset_filter:
        asset["labels"] = [default_asset_filter]
        manifest["asset_filter"] = default_asset_filter
    return manifest, default_scene


def load_project_manifest(
    project_manifest: Path,
    diagnostics: list[str],
) -> dict[str, Any] | None:
    if not project_manifest.exists():
        diagnostics.append(
            f"project manifest {project_manifest} does not exist and --asset-manifest was not supplied"
        )
        return None
    try:
        document = tomllib.loads(project_manifest.read_text(encoding="utf-8"))
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
) -> str | None:
    if not asset_uri.startswith("res://"):
        diagnostics.append(
            f"project default_scene {asset_uri} must use a res:// asset URI for CookAssets fallback"
        )
        return None
    package_path = asset_uri[len("res://") :].replace("\\", "/").lstrip("/")
    parts = [part for part in package_path.split("/") if part]
    if not parts or any(part in (".", "..") for part in parts):
        diagnostics.append(
            f"project default_scene {asset_uri} does not resolve to a safe asset path"
        )
        return None
    return "/".join(parts)


def validate_asset_manifest_shape(
    manifest: dict[str, Any],
    diagnostics: list[str],
) -> None:
    roots = manifest.get("roots", [])
    if not isinstance(roots, list) or any(not isinstance(root, str) for root in roots):
        diagnostics.append("asset manifest field roots must be a string array")

    asset_filter = manifest.get("asset_filter")
    if asset_filter is not None and not isinstance(asset_filter, str):
        diagnostics.append("asset manifest field asset_filter must be a string when present")

    assets = manifest.get("assets")
    if not isinstance(assets, list):
        diagnostics.append("asset manifest field assets must be an array")
        return

    seen_paths: set[str] = set()
    for index, asset in enumerate(assets):
        if not isinstance(asset, dict):
            diagnostics.append(f"asset manifest entry {index} must be an object")
            continue
        path = asset.get("path")
        if not isinstance(path, str) or not path:
            diagnostics.append(f"asset manifest entry {index} needs a non-empty path")
        elif path in seen_paths:
            diagnostics.append(f"asset manifest path {path} is declared more than once")
        else:
            seen_paths.add(path)

        validate_optional_string(asset, "source", index, diagnostics)
        validate_optional_string_array(asset, "dependencies", index, diagnostics)
        validate_optional_string_array(asset, "labels", index, diagnostics)


def normalized_cooked_asset_manifest(
    manifest: dict[str, Any],
    source_manifest_dir: Path,
) -> dict[str, Any]:
    normalized = dict(manifest)
    normalized_assets: list[dict[str, Any]] = []
    for asset in manifest.get("assets", []):
        normalized_asset = dict(asset)
        source = normalized_asset.get("source")
        if isinstance(source, str) and source:
            source_path = Path(source)
            if not source_path.is_absolute():
                normalized_asset["source"] = str((source_manifest_dir / source_path).resolve())
        normalized_assets.append(normalized_asset)
    normalized["assets"] = normalized_assets
    return normalized


def manifest_with_default_asset_filter(
    manifest: dict[str, Any],
    default_asset_filter: str | None,
) -> dict[str, Any]:
    if not default_asset_filter or manifest.get("asset_filter") is not None:
        return manifest
    with_filter = dict(manifest)
    with_filter["asset_filter"] = default_asset_filter
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
        if source_path.exists():
            continue
        asset_path = asset.get("path")
        if not isinstance(asset_path, str) or not asset_path:
            asset_path = f"entry {index}"
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


def validate_optional_string_array(
    asset: dict[str, Any],
    field_name: str,
    index: int,
    diagnostics: list[str],
) -> None:
    value = asset.get(field_name, [])
    if not isinstance(value, list) or any(not isinstance(item, str) for item in value):
        diagnostics.append(f"asset manifest entry {index} field {field_name} must be a string array")


def resolve_user_path(path: str | Path) -> Path:
    return Path(path).expanduser().resolve()
