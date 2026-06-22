from __future__ import annotations

import hashlib
import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.pack_schema_test_support import write_library_embed_reports


class PipelineReportCookAssetsManifestShapeSchemaTests(unittest.TestCase):
    def test_report_rejects_cook_assets_manifest_non_string_root_entry_before_array_shape(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            write_library_embed_reports(out)
            cook_report_path = out / "stages" / "cook_assets" / "report.json"
            cook_report = json.loads(cook_report_path.read_text(encoding="utf-8"))
            manifest = {
                "roots": ["scenes/main.zscene", 42],
                "assets": [{"path": "scenes/main.zscene"}],
            }
            write_cook_assets_manifest(cook_report, manifest)
            sync_cook_assets_report_counts(cook_report, manifest)
            cook_report_path.write_text(
                json.dumps(cook_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "cook_assets report cooked_asset_manifest roots[1] "
                    "must be a string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(
                any(
                    "cook_assets report cooked_asset_manifest roots "
                    "must be a string array"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_cook_assets_manifest_non_string_asset_string_array_entry_before_array_shape(
        self,
    ) -> None:
        cases = (
            (
                {
                    "roots": ["scenes/main.zscene"],
                    "assets": [
                        {
                            "path": "scenes/main.zscene",
                            "dependencies": ["textures/hero.png", 42],
                        },
                        {"path": "textures/hero.png"},
                    ],
                },
                (
                    "cook_assets report cooked_asset_manifest "
                    "assets[0].dependencies[1] must be a string"
                ),
                (
                    "cook_assets report cooked_asset_manifest "
                    "assets[0].dependencies must be a string array"
                ),
            ),
            (
                {
                    "roots": [],
                    "assets": [
                        {
                            "path": "textures/hero.png",
                            "labels": ["shipping", False],
                        }
                    ],
                },
                (
                    "cook_assets report cooked_asset_manifest "
                    "assets[0].labels[1] must be a string"
                ),
                (
                    "cook_assets report cooked_asset_manifest "
                    "assets[0].labels must be a string array"
                ),
            ),
        )
        for manifest, expected_diagnostic, unexpected_diagnostic in cases:
            with self.subTest(expected_diagnostic=expected_diagnostic):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    write_library_embed_reports(out)
                    cook_report_path = out / "stages" / "cook_assets" / "report.json"
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

                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )
                    self.assertFalse(
                        any(
                            unexpected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_rejects_cook_assets_manifest_padded_package_path_field(
        self,
    ) -> None:
        cases = (
            (
                {"roots": [" scenes/main.zscene "], "assets": []},
                (
                    "cook_assets report cooked_asset_manifest roots[0] "
                    "must be a non-empty trimmed string"
                ),
            ),
            (
                {
                    "roots": [],
                    "assets": [{"path": " scenes/main.zscene "}],
                },
                (
                    "cook_assets report cooked_asset_manifest assets[0].path "
                    "must be a non-empty trimmed string"
                ),
            ),
            (
                {
                    "roots": [],
                    "assets": [
                        {
                            "path": "scenes/main.zscene",
                            "dependencies": [" textures/hero.png "],
                        },
                        {"path": "textures/hero.png"},
                    ],
                },
                (
                    "cook_assets report cooked_asset_manifest "
                    "assets[0].dependencies[0] "
                    "must be a non-empty trimmed string"
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

                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )
                    self.assertFalse(
                        any(
                            "must use a normalized relative asset path" in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )
                    self.assertFalse(
                        any(
                            "pack report trim_report" in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_rejects_cook_assets_manifest_padded_optional_or_label_string(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            source_file = root / "source" / "main.scene"
            source_file.parent.mkdir(parents=True)
            source_file.write_text("scene", encoding="utf-8")
            cases = (
                (
                    {"roots": [], "assets": [], "asset_filter": " shipping "},
                    (
                        "cook_assets report cooked_asset_manifest asset_filter "
                        "must be a non-empty trimmed string when present"
                    ),
                ),
                (
                    {
                        "roots": [],
                        "assets": [
                            {
                                "path": "scenes/main.zscene",
                                "source": f" {source_file} ",
                            }
                        ],
                    },
                    (
                        "cook_assets report cooked_asset_manifest "
                        "assets[0].source must be a non-empty trimmed string "
                        "when present"
                    ),
                ),
                (
                    {
                        "roots": [],
                        "assets": [
                            {
                                "path": "scenes/main.zscene",
                                "labels": [" shipping "],
                            }
                        ],
                    },
                    (
                        "cook_assets report cooked_asset_manifest "
                        "assets[0].labels[0] must be a non-empty trimmed string"
                    ),
                ),
            )
            for index, (manifest, expected_diagnostic) in enumerate(cases):
                with self.subTest(expected_diagnostic=expected_diagnostic):
                    out = root / f"out-{index}"
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

                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )
                    self.assertFalse(
                        any(
                            "must use a trimmed string" in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )
                    self.assertFalse(
                        any(
                            "pack report trim_report" in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_rejects_cook_assets_manifest_schema_before_count_semantics(
        self,
    ) -> None:
        cases = (
            (
                {"roots": [" scenes/main.zscene "], "assets": []},
                "root_count",
                2,
                (
                    "cook_assets report cooked_asset_manifest roots[0] "
                    "must be a non-empty trimmed string"
                ),
                "cook_assets report root_count 2 does not match "
                "cooked_asset_manifest roots length 1",
            ),
            (
                {
                    "roots": [],
                    "assets": [{"path": " scenes/main.zscene "}],
                },
                "asset_count",
                2,
                (
                    "cook_assets report cooked_asset_manifest assets[0].path "
                    "must be a non-empty trimmed string"
                ),
                "cook_assets report asset_count 2 does not match "
                "cooked_asset_manifest assets length 1",
            ),
        )
        for manifest, count_field, count_value, expected, unexpected in cases:
            with self.subTest(count_field=count_field):
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
                    cook_report[count_field] = count_value
                    cook_report_path.write_text(
                        json.dumps(cook_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(
                            expected in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )
                    self.assertFalse(
                        any(
                            unexpected in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_rejects_cook_assets_manifest_schema_before_determinism_semantics(
        self,
    ) -> None:
        cases = (
            (
                {
                    "roots": [" z.zscene ", "a.zscene"],
                    "assets": [{"path": "a.zscene"}],
                },
                (
                    "cook_assets report cooked_asset_manifest roots[0] "
                    "must be a non-empty trimmed string"
                ),
                "cook_assets report cooked_asset_manifest roots must be "
                "sorted and unique",
            ),
            (
                {
                    "roots": [],
                    "assets": [
                        {"path": "z.zscene "},
                        {"path": "a.zscene"},
                    ],
                },
                (
                    "cook_assets report cooked_asset_manifest assets[0].path "
                    "must be a non-empty trimmed string"
                ),
                "cook_assets report cooked_asset_manifest assets must be "
                "sorted by path",
            ),
            (
                {
                    "roots": [],
                    "assets": [
                        {
                            "path": "scenes/main.zscene",
                            "dependencies": [" z.zscene ", "a.zscene"],
                        },
                        {"path": "a.zscene"},
                        {"path": "z.zscene"},
                    ],
                },
                (
                    "cook_assets report cooked_asset_manifest "
                    "assets[0].dependencies[0] "
                    "must be a non-empty trimmed string"
                ),
                "cook_assets report cooked_asset_manifest "
                "assets[0].dependencies must be sorted and unique",
            ),
            (
                {
                    "roots": [],
                    "assets": [
                        {
                            "path": "scenes/main.zscene",
                            "labels": [" z ", "a"],
                        },
                    ],
                },
                (
                    "cook_assets report cooked_asset_manifest "
                    "assets[0].labels[0] must be a non-empty trimmed string"
                ),
                "cook_assets report cooked_asset_manifest "
                "assets[0].labels must be sorted and unique",
            ),
        )
        for manifest, expected, unexpected in cases:
            with self.subTest(unexpected=unexpected):
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

                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(expected in diagnostic for diagnostic in report["diagnostics"]),
                        report["diagnostics"],
                    )
                    self.assertFalse(
                        any(
                            unexpected in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_rejects_cook_assets_manifest_schema_before_trim_closure_semantics(
        self,
    ) -> None:
        cases = (
            (
                {"roots": ["../escape.zscene"], "assets": []},
                (
                    "cook_assets report cooked_asset_manifest roots[0] "
                    "must be a safe relative asset path"
                ),
            ),
            (
                {
                    "roots": [],
                    "assets": [{"path": "scenes\\main.zscene"}],
                },
                (
                    "cook_assets report cooked_asset_manifest assets[0].path "
                    "must use a normalized relative asset path"
                ),
            ),
            (
                {
                    "roots": ["scenes/main.zscene"],
                    "assets": [
                        {
                            "path": "scenes/main.zscene",
                            "dependencies": ["../escape.png"],
                        }
                    ],
                },
                (
                    "cook_assets report cooked_asset_manifest "
                    "assets[0].dependencies[0] "
                    "must be a safe relative asset path"
                ),
            ),
        )
        for manifest, expected in cases:
            with self.subTest(expected=expected):
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

                    self.assertTrue(report["fatal"], report["diagnostics"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(expected in diagnostic for diagnostic in report["diagnostics"]),
                        report["diagnostics"],
                    )
                    self.assertFalse(
                        any(
                            "CookAssets dependency closure" in diagnostic
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


if __name__ == "__main__":
    unittest.main()
