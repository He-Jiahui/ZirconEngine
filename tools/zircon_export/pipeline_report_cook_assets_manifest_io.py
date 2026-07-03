"""CookAssets staged manifest IO helpers for final pipeline reports."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any


def resolve_cook_assets_path_or_diagnostic(
    path: str | Path,
    diagnostics: list[str],
    label: str,
) -> Path | None:
    try:
        return Path(path).expanduser().resolve()
    except OSError as error:
        diagnostics.append(f"{label} {path} could not be resolved: {error}")
        return None


def cook_assets_manifest_path(
    stage_reports: list[dict[str, Any]],
    diagnostics: list[str],
) -> Path | None:
    for stage_report in stage_reports:
        if stage_report.get("stage_key") != "cook_assets":
            continue
        if stage_report.get("fatal") is True:
            continue
        report = stage_report.get("report")
        if not isinstance(report, dict):
            continue
        manifest = report.get("cooked_asset_manifest")
        if not cook_assets_is_non_empty_trimmed_string(manifest):
            return None
        return resolve_cook_assets_path_or_diagnostic(
            manifest,
            diagnostics,
            "cook_assets report cooked_asset_manifest",
        )
    return None


def cook_assets_report_manifest_path(
    report: dict[str, Any],
    diagnostics: list[str],
) -> Path | None:
    manifest = report.get("cooked_asset_manifest")
    if not cook_assets_is_non_empty_trimmed_string(manifest):
        return None
    return resolve_cook_assets_path_or_diagnostic(
        manifest,
        diagnostics,
        "cook_assets report cooked_asset_manifest",
    )


def cook_assets_manifest_json(
    manifest_path: Path,
    diagnostics: list[str],
) -> object:
    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    except OSError as error:
        diagnostics.append(
            f"cook_assets report cooked_asset_manifest {manifest_path} "
            f"could not be read: {error}"
        )
        return None
    except json.JSONDecodeError as error:
        diagnostics.append(
            f"cook_assets report cooked_asset_manifest {manifest_path} "
            f"is not valid JSON: {error}"
        )
        return None
    if not isinstance(manifest, dict):
        diagnostics.append(
            f"cook_assets report cooked_asset_manifest {manifest_path} "
            "must be a JSON object"
        )
    return manifest


def cook_assets_is_non_empty_trimmed_string(value: object) -> bool:
    return isinstance(value, str) and bool(value.strip()) and value.strip() == value
