from __future__ import annotations

import hashlib
import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.export_test_support import (
    _write_compile_host_report,
    _write_pack_report,
    _write_platform_bundle_report_with_native_plugins_payload,
    _write_validate_report_with_strategies,
    json_dumps,
    json_loads,
)
from tools.zircon_export.tests.pack_test_support import asset_entry, manifest_for_assets


class PipelineReportCookAssetsPackHandoffTests(unittest.TestCase):
    def test_report_stage_rejects_pack_asset_manifest_drift_from_cook_assets_source(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            source = root / "source" / "main.scene"
            source.parent.mkdir(parents=True)
            source.write_bytes(b"scene-bytes")
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            write_cook_assets_report(
                out,
                {
                    "roots": ["scenes/main.zscene"],
                    "assets": [
                        {
                            "path": "scenes/main.zscene",
                            "source": str(source),
                            "dependencies": [],
                            "labels": [],
                        },
                    ],
                },
            )
            _write_pack_report(out, out / "stages" / "pack" / "assets.zrpack")
            rewrite_pack_report_for_trim_evidence(
                out,
                included_assets=["scenes/main.zscene"],
                trimmed_assets=[],
            )
            _write_platform_bundle_report_with_native_plugins_payload(out, {})

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "pack report manifest.assets[0].size 8 does not match "
                    "CookAssets source byte length 11 for included asset "
                    "scenes/main.zscene"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertTrue(
                any(
                    "pack report manifest.assets[0].chunk_hash does not match "
                    "CookAssets source content hash for included asset "
                    "scenes/main.zscene"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_pack_included_asset_missing_cook_assets_source(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            write_cook_assets_report(
                out,
                {
                    "roots": ["scenes/main.zscene"],
                    "assets": [
                        {
                            "path": "scenes/main.zscene",
                            "dependencies": [],
                            "labels": [],
                        },
                    ],
                },
            )
            _write_pack_report(out, out / "stages" / "pack" / "assets.zrpack")
            rewrite_pack_report_for_trim_evidence(
                out,
                included_assets=["scenes/main.zscene"],
                trimmed_assets=[],
            )
            _write_platform_bundle_report_with_native_plugins_payload(out, {})

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "pack report trim_report.included_assets contains "
                    "scenes/main.zscene but CookAssets manifest asset is "
                    "missing source"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_pack_trim_included_assets_outside_cook_assets_closure(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            write_cook_assets_report(
                out,
                {
                    "roots": ["scenes/main.zscene"],
                    "asset_filter": "shipping",
                    "assets": [
                        {
                            "path": "scenes/main.zscene",
                            "dependencies": ["textures/hero.png"],
                            "labels": ["shipping"],
                        },
                        {
                            "path": "textures/hero.png",
                            "dependencies": [],
                            "labels": ["shipping"],
                        },
                        {
                            "path": "textures/unused.png",
                            "dependencies": [],
                            "labels": ["shipping"],
                        },
                    ],
                },
            )
            _write_pack_report(out, out / "stages" / "pack" / "assets.zrpack")
            rewrite_pack_report_for_included_assets(
                out,
                ["scenes/main.zscene"],
            )
            _write_platform_bundle_report_with_native_plugins_payload(out, {})

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "pack report trim_report.included_assets does not match "
                    "CookAssets dependency closure"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_pack_trimmed_assets_outside_cook_assets_closure(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            write_cook_assets_report(
                out,
                {
                    "roots": ["scenes/main.zscene"],
                    "asset_filter": "shipping",
                    "assets": [
                        {
                            "path": "scenes/main.zscene",
                            "dependencies": ["textures/editor.png"],
                            "labels": ["shipping"],
                        },
                        {
                            "path": "textures/editor.png",
                            "dependencies": [],
                            "labels": ["editor"],
                        },
                        {
                            "path": "textures/unused.png",
                            "dependencies": [],
                            "labels": ["shipping"],
                        },
                    ],
                },
            )
            _write_pack_report(out, out / "stages" / "pack" / "assets.zrpack")
            rewrite_pack_report_for_trim_evidence(
                out,
                included_assets=["scenes/main.zscene"],
                trimmed_assets=[
                    {
                        "path": "textures/editor.png",
                        "reason": "Unreferenced",
                    },
                    {
                        "path": "textures/unused.png",
                        "reason": "Unreferenced",
                    },
                ],
            )
            _write_platform_bundle_report_with_native_plugins_payload(out, {})

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "pack report trim_report.trimmed_assets does not match "
                    "CookAssets dependency closure"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_pack_success_with_cook_assets_missing_dependency(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            write_cook_assets_report(
                out,
                {
                    "roots": ["scenes/main.zscene"],
                    "assets": [
                        {
                            "path": "scenes/main.zscene",
                            "dependencies": ["textures/missing.png"],
                            "labels": [],
                        },
                    ],
                },
            )
            _write_pack_report(out, out / "stages" / "pack" / "assets.zrpack")
            rewrite_pack_report_for_trim_evidence(
                out,
                included_assets=["scenes/main.zscene"],
                trimmed_assets=[],
            )
            _write_platform_bundle_report_with_native_plugins_payload(out, {})

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "pack report trim_report.missing_dependencies does not match "
                    "CookAssets dependency closure"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_pack_success_with_cook_assets_duplicate_asset(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            write_cook_assets_report(
                out,
                {
                    "roots": ["scenes/main.zscene"],
                    "assets": [
                        {
                            "path": "scenes/main.zscene",
                            "dependencies": [],
                            "labels": [],
                        },
                        {
                            "path": "scenes/main.zscene",
                            "dependencies": [],
                            "labels": [],
                        },
                    ],
                },
            )
            _write_pack_report(out, out / "stages" / "pack" / "assets.zrpack")
            rewrite_pack_report_for_trim_evidence(
                out,
                included_assets=["scenes/main.zscene"],
                trimmed_assets=[],
            )
            _write_platform_bundle_report_with_native_plugins_payload(out, {})

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "pack report trim_report.duplicate_assets does not match "
                    "CookAssets dependency closure"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_pack_trim_diagnostics_outside_cook_assets_closure(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            write_cook_assets_report(
                out,
                {
                    "roots": ["scenes/main.zscene"],
                    "assets": [
                        {
                            "path": "scenes/main.zscene",
                            "dependencies": [],
                            "labels": [],
                        },
                        {
                            "path": "textures/unused.png",
                            "dependencies": [],
                            "labels": [],
                        },
                    ],
                },
            )
            _write_pack_report(out, out / "stages" / "pack" / "assets.zrpack")
            rewrite_pack_report_for_trim_evidence(
                out,
                included_assets=["scenes/main.zscene"],
                trimmed_assets=[
                    {
                        "path": "textures/unused.png",
                        "reason": "Unreferenced",
                    },
                ],
                diagnostics=[],
            )
            _write_platform_bundle_report_with_native_plugins_payload(out, {})

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "pack report trim_report.diagnostics does not match "
                    "CookAssets dependency closure"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )


def write_cook_assets_report(out: Path, manifest: dict[str, object]) -> None:
    report_dir = out / "stages" / "cook_assets"
    manifest_path = report_dir / "assets.json"
    report_dir.mkdir(parents=True, exist_ok=True)
    manifest_contents = json.dumps(manifest, indent=2, sort_keys=True)
    manifest_path.write_text(manifest_contents, encoding="utf-8", newline="\n")
    assets = manifest.get("assets", [])
    roots = manifest.get("roots", [])
    report = {
        "stage": "CookAssets",
        "profile": "windows-release",
        "fatal": False,
        "diagnostics": [],
        "source_asset_manifest": None,
        "project_manifest": None,
        "generated_from_project": False,
        "project_default_scene": None,
        "cooked_asset_manifest": str(manifest_path),
        "cooked_asset_manifest_sha256": hashlib.sha256(
            manifest_contents.encode("utf-8")
        ).hexdigest(),
        "asset_count": len(assets) if isinstance(assets, list) else 0,
        "root_count": len(roots) if isinstance(roots, list) else 0,
        "asset_filter": manifest.get("asset_filter"),
    }
    report_dir.joinpath("report.json").write_text(
        json_dumps(report),
        encoding="utf-8",
    )


def rewrite_pack_report_for_included_assets(
    out: Path,
    included_assets: list[str],
) -> None:
    rewrite_pack_report_for_trim_evidence(
        out,
        included_assets=included_assets,
        trimmed_assets=[],
    )


def rewrite_pack_report_for_trim_evidence(
    out: Path,
    *,
    included_assets: list[str],
    trimmed_assets: list[dict[str, object]],
    diagnostics: list[str] | None = None,
) -> None:
    report_path = out / "stages" / "pack" / "report.json"
    report = json_loads(report_path.read_text(encoding="utf-8"))
    assert isinstance(report, dict)
    report["manifest"] = manifest_for_assets(
        [
            asset_entry(hash_value=index + 1, path=path)
            for index, path in enumerate(included_assets)
        ],
        hash_values=list(range(1, len(included_assets) + 1)),
    )
    report["asset_count"] = len(included_assets)
    report["chunk_count"] = len(included_assets)
    report["trim_report"] = {
        "included_assets": included_assets,
        "trimmed_assets": trimmed_assets,
        "missing_dependencies": [],
        "duplicate_assets": [],
        "diagnostics": diagnostics or [],
    }
    report_path.write_text(json_dumps(report), encoding="utf-8")


if __name__ == "__main__":
    unittest.main()
