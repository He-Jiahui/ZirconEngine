from __future__ import annotations

import hashlib
import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.pack_test_support import asset_entry, manifest_for_assets
from tools.zircon_export.tests.pack_schema_test_support import write_library_embed_reports


class PipelineReportCookAssetsManifestEvidenceTests(unittest.TestCase):
    def test_report_stage_rejects_cook_assets_manifest_outside_stage_directory(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            write_library_embed_reports(out)
            cook_report_path = out / "stages" / "cook_assets" / "report.json"
            cook_report = json.loads(cook_report_path.read_text(encoding="utf-8"))
            original_manifest_path = Path(str(cook_report["cooked_asset_manifest"]))
            manifest = json.loads(original_manifest_path.read_text(encoding="utf-8"))
            external_manifest_path = root / "external" / "assets.json"
            external_manifest_path.parent.mkdir(parents=True)
            manifest_contents = json.dumps(manifest, indent=2, sort_keys=True)
            external_manifest_path.write_text(
                manifest_contents,
                encoding="utf-8",
                newline="\n",
            )
            cook_report["cooked_asset_manifest"] = str(external_manifest_path)
            cook_report["cooked_asset_manifest_sha256"] = hashlib.sha256(
                manifest_contents.encode("utf-8")
            ).hexdigest()
            cook_report_path.write_text(
                json.dumps(cook_report, indent=2),
                encoding="utf-8",
            )
            pack_report_path = out / "stages" / "pack" / "report.json"
            pack_report = json.loads(pack_report_path.read_text(encoding="utf-8"))
            pack_report["asset_manifest"] = str(external_manifest_path)
            pack_report_path.write_text(
                json.dumps(pack_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "cook_assets report cooked_asset_manifest "
                    in diagnostic
                    and "does not match current CookAssets stage manifest"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_cook_assets_manifest_bad_source_path(
        self,
    ) -> None:
        cases = (
            (
                "relative source",
                lambda root: "main.scene",
                (
                    "cook_assets report cooked_asset_manifest "
                    "assets[0].source must be an absolute path"
                ),
            ),
            (
                "missing source",
                lambda root: str(root / "source" / "missing.scene"),
                (
                    "cook_assets report cooked_asset_manifest "
                    "assets[0].source does not exist"
                ),
            ),
        )
        for case_name, source_value, expected_diagnostic in cases:
            with self.subTest(case=case_name):
                with tempfile.TemporaryDirectory() as temp_dir:
                    root = Path(temp_dir)
                    out = root / "out"
                    write_library_embed_reports(out)
                    cook_report_path = (
                        out / "stages" / "cook_assets" / "report.json"
                    )
                    cook_report = json.loads(
                        cook_report_path.read_text(encoding="utf-8")
                    )
                    manifest = {
                        "roots": ["scenes/main.zscene"],
                        "assets": [
                            {
                                "path": "scenes/main.zscene",
                                "source": source_value(root),
                                "dependencies": [],
                                "labels": [],
                            }
                        ],
                    }
                    write_cook_assets_manifest(cook_report, manifest)
                    sync_cook_assets_report_counts(cook_report, manifest)
                    cook_report_path.write_text(
                        json.dumps(cook_report, indent=2),
                        encoding="utf-8",
                    )
                    rewrite_pack_report_for_included_assets(
                        out,
                        ["scenes/main.zscene"],
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_cook_assets_manifest_non_deterministic_order(
        self,
    ) -> None:
        cases = (
            (
                {
                    "roots": [
                        "scenes/main.zscene",
                        "scenes/intro.zscene",
                        "scenes/main.zscene",
                    ],
                    "assets": [
                        {
                            "path": "scenes/intro.zscene",
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
                ["scenes/intro.zscene", "scenes/main.zscene"],
                "cook_assets report cooked_asset_manifest roots must be sorted and unique",
            ),
            (
                {
                    "roots": ["scenes/main.zscene"],
                    "assets": [
                        {
                            "path": "textures/hero.png",
                            "dependencies": [],
                            "labels": [],
                        },
                        {
                            "path": "scenes/main.zscene",
                            "dependencies": ["textures/hero.png"],
                            "labels": [],
                        },
                    ],
                },
                ["scenes/main.zscene", "textures/hero.png"],
                "cook_assets report cooked_asset_manifest assets must be sorted by path",
            ),
            (
                {
                    "roots": ["scenes/main.zscene"],
                    "assets": [
                        {
                            "path": "scenes/main.zscene",
                            "dependencies": [
                                "textures/z.png",
                                "textures/a.png",
                                "textures/a.png",
                            ],
                            "labels": [],
                        },
                        {
                            "path": "textures/a.png",
                            "dependencies": [],
                            "labels": [],
                        },
                        {
                            "path": "textures/z.png",
                            "dependencies": [],
                            "labels": [],
                        },
                    ],
                },
                ["scenes/main.zscene", "textures/a.png", "textures/z.png"],
                (
                    "cook_assets report cooked_asset_manifest "
                    "assets[0].dependencies must be sorted and unique"
                ),
            ),
            (
                {
                    "roots": ["scenes/main.zscene"],
                    "asset_filter": "shipping",
                    "assets": [
                        {
                            "path": "scenes/main.zscene",
                            "dependencies": [],
                            "labels": ["shipping", "editor", "shipping"],
                        },
                    ],
                },
                ["scenes/main.zscene"],
                (
                    "cook_assets report cooked_asset_manifest "
                    "assets[0].labels must be sorted and unique"
                ),
            ),
        )
        for manifest, included_assets, expected_diagnostic in cases:
            with self.subTest(expected_diagnostic=expected_diagnostic):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    write_library_embed_reports(out)
                    cook_report_path = (
                        out / "stages" / "cook_assets" / "report.json"
                    )
                    cook_report = json.loads(
                        cook_report_path.read_text(encoding="utf-8")
                    )
                    write_cook_assets_manifest(cook_report, manifest)
                    sync_cook_assets_report_counts(cook_report, manifest)
                    if "asset_filter" in manifest:
                        cook_report["asset_filter"] = manifest["asset_filter"]
                    cook_report_path.write_text(
                        json.dumps(cook_report, indent=2),
                        encoding="utf-8",
                    )
                    rewrite_pack_report_for_included_assets(out, included_assets)

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_cook_assets_manifest_shape_mismatch(
        self,
    ) -> None:
        cases = (
            (
                {"roots": [42], "assets": []},
                "cook_assets report cooked_asset_manifest roots must be a string array",
            ),
            (
                {"roots": [], "assets": [42]},
                "cook_assets report cooked_asset_manifest assets[0] must be an object",
            ),
            (
                {"roots": [], "assets": [], "asset_filter": []},
                (
                    "cook_assets report cooked_asset_manifest asset_filter "
                    "must be a string when present"
                ),
            ),
        )
        for manifest, expected_diagnostic in cases:
            with self.subTest(expected_diagnostic=expected_diagnostic):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    write_library_embed_reports(out)
                    cook_report_path = (
                        out / "stages" / "cook_assets" / "report.json"
                    )
                    cook_report = json.loads(
                        cook_report_path.read_text(encoding="utf-8")
                    )
                    write_cook_assets_manifest(cook_report, manifest)
                    sync_cook_assets_report_counts(cook_report, manifest)
                    cook_report_path.write_text(
                        json.dumps(cook_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_cook_assets_manifest_asset_filter_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            write_library_embed_reports(out)
            cook_report_path = out / "stages" / "cook_assets" / "report.json"
            cook_report = json.loads(cook_report_path.read_text(encoding="utf-8"))
            manifest = json.loads(
                Path(cook_report["cooked_asset_manifest"]).read_text(encoding="utf-8")
            )
            manifest["asset_filter"] = "shipping"
            write_cook_assets_manifest(cook_report, manifest)
            cook_report["asset_filter"] = "editor"
            cook_report_path.write_text(
                json.dumps(cook_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "cook_assets report asset_filter editor does not match "
                    "cooked_asset_manifest asset_filter shipping"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )


def write_cook_assets_manifest(
    cook_report: dict[str, object],
    manifest: dict[str, object],
) -> None:
    manifest_path = Path(str(cook_report["cooked_asset_manifest"]))
    manifest_contents = json.dumps(manifest, indent=2, sort_keys=True)
    manifest_path.write_text(manifest_contents, encoding="utf-8", newline="\n")
    cook_report["cooked_asset_manifest_sha256"] = hashlib.sha256(
        manifest_contents.encode("utf-8")
    ).hexdigest()


def sync_cook_assets_report_counts(
    cook_report: dict[str, object],
    manifest: dict[str, object],
) -> None:
    roots = manifest.get("roots")
    assets = manifest.get("assets")
    cook_report["root_count"] = len(roots) if isinstance(roots, list) else 0
    cook_report["asset_count"] = len(assets) if isinstance(assets, list) else 0


def rewrite_pack_report_for_included_assets(
    out: Path,
    included_assets: list[str],
) -> None:
    report_path = out / "stages" / "pack" / "report.json"
    report = json.loads(report_path.read_text(encoding="utf-8"))
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
        "trimmed_assets": [],
        "missing_dependencies": [],
        "duplicate_assets": [],
        "diagnostics": [],
    }
    report_path.write_text(json.dumps(report, indent=2), encoding="utf-8")


if __name__ == "__main__":
    unittest.main()
