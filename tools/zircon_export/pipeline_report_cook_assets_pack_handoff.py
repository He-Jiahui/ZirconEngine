"""CookAssets-to-Pack handoff diagnostics for final pipeline reports."""

from __future__ import annotations

from typing import Any

from .pipeline_report_cook_assets_manifest_io import (
    cook_assets_manifest_path,
    resolve_cook_assets_path_or_diagnostic,
)


def cook_assets_pack_manifest_handoff_diagnostics(
    stage_reports: list[dict[str, Any]],
) -> list[str]:
    diagnostics: list[str] = []
    cook_assets_manifest = cook_assets_manifest_path(stage_reports, diagnostics)
    if cook_assets_manifest is None:
        return diagnostics
    for stage_report in stage_reports:
        if stage_report.get("stage_key") != "pack":
            continue
        if stage_report.get("fatal") is True:
            continue
        report = stage_report.get("report")
        if not isinstance(report, dict):
            continue
        pack_manifest = report.get("asset_manifest")
        if not isinstance(pack_manifest, str) or not pack_manifest:
            continue
        pack_manifest_path = resolve_cook_assets_path_or_diagnostic(
            pack_manifest,
            diagnostics,
            "pack report asset_manifest",
        )
        if pack_manifest_path is None:
            continue
        if pack_manifest_path != cook_assets_manifest:
            diagnostics.append(
                f"pack report asset_manifest {pack_manifest_path} does not match "
                f"cook_assets report cooked_asset_manifest {cook_assets_manifest}"
            )
    return diagnostics
