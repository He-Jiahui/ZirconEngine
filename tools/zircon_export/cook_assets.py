"""CookAssets stage handoff for the Zircon export pipeline."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import tomllib
from pathlib import Path
from typing import Any

from .export_template import is_safe_relative_path, normalize_relative_path
from .report_io import write_report_targets
from .stage_handoff import (
    validate_report_asset_filter,
    validate_report_asset_filter_diagnostic,
    validate_report_requires_bundle_strategy_diagnostics,
)

REPORT_FILE_NAME = "report.json"
COOKED_ASSET_MANIFEST_NAME = "assets.json"
RES_ASSET_REFERENCE_RE = re.compile(r"res://[A-Za-z0-9_./\\-]+(?:#[A-Za-z0-9_./\\-]+)?")
ASSET_MANIFEST_FIELDS = ("asset_filter", "assets", "roots")
ASSET_MANIFEST_ASSET_FIELDS = ("dependencies", "labels", "path", "source")


def run_cook_assets(args: argparse.Namespace) -> int:
    out_root = resolve_user_path(args.out)
    stage_dir = out_root / "stages" / "cook_assets"
    report_path = stage_dir / REPORT_FILE_NAME
    cooked_manifest = default_cooked_asset_manifest(out_root)
    diagnostics: list[str] = []
    source_manifest = resolve_cook_assets_optional_path(
        getattr(args, "asset_manifest", None),
        "asset_manifest",
        diagnostics,
    )
    project_manifest = resolve_cook_assets_optional_path(
        args.project,
        "project_manifest",
        diagnostics,
    )
    default_asset_filter = getattr(args, "asset_filter", None)
    validate_asset_filter_diagnostic = getattr(
        args,
        "validate_asset_filter_diagnostic",
        None,
    )
    validate_strategy_diagnostics = validate_report_requires_bundle_strategy_diagnostics(
        out_root,
        args.profile,
        "CookAssets",
    )
    if (
        default_asset_filter is None
        and validate_asset_filter_diagnostic is None
        and not validate_strategy_diagnostics
    ):
        validate_asset_filter_diagnostic = validate_report_asset_filter_diagnostic(
            out_root,
            args.profile,
        )
        if validate_asset_filter_diagnostic is None:
            default_asset_filter = validate_report_asset_filter(out_root, args.profile)
    asset_filter_diagnostic = asset_filter_argument_diagnostic(default_asset_filter)

    print(f"zircon_export stage=CookAssets profile={args.profile}")
    print(f"source_asset_manifest={source_manifest if source_manifest else '<invalid>'}")
    print(f"project_manifest={project_manifest if project_manifest else '<invalid>'}")
    print(f"cooked_asset_manifest={cooked_manifest}")
    if default_asset_filter:
        print(f"default_asset_filter={default_asset_filter}")
    print(f"report={report_path}")
    if args.dry_run:
        for diagnostic in diagnostics:
            print(f"diagnostic={diagnostic}")
        if diagnostics:
            return 2
        if validate_strategy_diagnostics:
            for diagnostic in validate_strategy_diagnostics:
                print(f"diagnostic={diagnostic}")
            return 2
        if validate_asset_filter_diagnostic:
            print(f"diagnostic={validate_asset_filter_diagnostic}")
            return 2
        if asset_filter_diagnostic:
            print(f"diagnostic={asset_filter_diagnostic}")
            return 2
        return 0 if source_manifest is not None or project_manifest.exists() else 2

    diagnostics.extend(validate_strategy_diagnostics)
    if validate_asset_filter_diagnostic:
        diagnostics.append(validate_asset_filter_diagnostic)
    if asset_filter_diagnostic:
        diagnostics.append(asset_filter_diagnostic)
    try:
        stage_dir.mkdir(parents=True, exist_ok=True)
    except OSError as error:
        diagnostics.append(
            f"CookAssets stage directory {stage_dir} could not be created: {error}"
        )
        report = {
            "stage": "CookAssets",
            "profile": args.profile,
            "fatal": True,
            "diagnostics": diagnostics,
            "source_asset_manifest": str(source_manifest) if source_manifest else None,
            "project_manifest": str(project_manifest) if project_manifest else None,
            "generated_from_project": False,
            "project_default_scene": None,
            "cooked_asset_manifest": str(cooked_manifest),
            "asset_count": 0,
            "root_count": 0,
            "asset_filter": None,
        }
        print(json.dumps(report, indent=2))
        return 2
    manifest: dict[str, Any] | None = None
    generated_from_project = False
    project_default_scene: str | None = None
    project_source_manifest: Path | None = None
    if diagnostics:
        manifest = None
    elif source_manifest is None and project_manifest is not None:
        (
            manifest,
            project_default_scene,
            project_source_manifest,
        ) = project_default_scene_manifest(
            project_manifest,
            default_asset_filter,
            diagnostics,
        )
        generated_from_project = manifest is not None
    else:
        manifest = load_cooked_asset_manifest(source_manifest, diagnostics)
        if manifest is not None and not diagnostics:
            manifest = normalized_cooked_asset_manifest(
                manifest,
                source_manifest.parent,
                diagnostics,
            )
            manifest = manifest_with_default_asset_filter(manifest, default_asset_filter)
    if manifest is not None and not diagnostics:
        validate_asset_sources_exist(manifest, diagnostics)

    fatal = bool(diagnostics)
    cooked_asset_manifest_sha256: str | None = None
    if manifest is not None and not fatal:
        try:
            manifest_contents = json.dumps(manifest, indent=2, sort_keys=True)
            cooked_asset_manifest_sha256 = hashlib.sha256(
                manifest_contents.encode("utf-8")
            ).hexdigest()
            cooked_manifest.write_text(
                manifest_contents,
                encoding="utf-8",
                newline="\n",
            )
        except OSError as error:
            diagnostics.append(
                f"cooked asset manifest {cooked_manifest} could not be written: {error}"
            )
            fatal = True
            cooked_asset_manifest_sha256 = None

    report = {
        "stage": "CookAssets",
        "profile": args.profile,
        "fatal": fatal,
        "diagnostics": diagnostics,
        "source_asset_manifest": (
            str(source_manifest or project_source_manifest)
            if source_manifest or project_source_manifest
            else None
        ),
        "project_manifest": str(project_manifest) if project_manifest else None,
        "generated_from_project": generated_from_project,
        "project_default_scene": project_default_scene,
        "cooked_asset_manifest": str(cooked_manifest),
        "cooked_asset_manifest_sha256": cooked_asset_manifest_sha256,
        "asset_count": len(manifest.get("assets", [])) if manifest else 0,
        "root_count": len(manifest.get("roots", [])) if manifest else 0,
        "asset_filter": manifest.get("asset_filter") if manifest else None,
    }
    report_written = write_report_targets([("CookAssets report", report_path)], report)
    print(json.dumps(report, indent=2))
    return 2 if fatal or not report_written else 0


def default_cooked_asset_manifest(out_root: Path) -> Path:
    return out_root / "stages" / "cook_assets" / COOKED_ASSET_MANIFEST_NAME


def resolve_cook_assets_optional_path(
    value: object,
    label: str,
    diagnostics: list[str],
) -> Path | None:
    if value is None:
        return None
    if not isinstance(value, (str, Path)) or not str(value).strip():
        diagnostics.append(f"CookAssets {label} argument must be a non-empty path")
        return None
    try:
        return resolve_user_path(value)
    except OSError as error:
        diagnostics.append(f"CookAssets {label} {value} could not be resolved: {error}")
        return None


def asset_filter_argument_diagnostic(asset_filter: Any) -> str | None:
    if asset_filter is None:
        return None
    if not isinstance(asset_filter, str) or not asset_filter.strip():
        return "asset_filter argument must be a non-empty string"
    return None


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


def sorted_cooked_asset_manifest_entries(
    assets: list[dict[str, Any]],
) -> list[dict[str, Any]]:
    return sorted(assets, key=lambda asset: str(asset.get("path", "")))


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


def resolve_user_path(path: str | Path) -> Path:
    return Path(path).expanduser().resolve()


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
