"""CookAssets stage handoff for the Zircon export pipeline."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any

from .cook_assets_manifest import (
    load_cooked_asset_manifest,
    manifest_with_default_asset_filter,
    normalized_cooked_asset_manifest,
    validate_asset_sources_exist,
)
from .cook_assets_project_fallback import project_default_scene_manifest
from .report_io import write_report_targets
from .stage_handoff import (
    validate_report_asset_filter,
    validate_report_asset_filter_diagnostic,
)
from .stage_handoff_strategy import (
    validate_report_requires_bundle_strategy_diagnostics,
)

REPORT_FILE_NAME = "report.json"
COOKED_ASSET_MANIFEST_NAME = "assets.json"


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


def resolve_user_path(path: str | Path) -> Path:
    return Path(path).expanduser().resolve()
