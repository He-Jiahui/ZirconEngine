from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.export_test_support import (
    _write_compile_host_report,
    _write_pack_report,
    _write_stage_report,
    _write_validate_report_with_strategies,
)


class PipelineReportValidateProfileSummarySchemaTests(unittest.TestCase):
    def test_report_stage_rejects_validate_profile_summary_missing_required_field(
        self,
    ) -> None:
        cases = (
            (
                "build_mode",
                "validate report profile_summary.build_mode must be a string",
            ),
            (
                "features",
                "validate report profile_summary.features must be an object",
            ),
            (
                "name",
                "validate report profile_summary.name must be a string",
            ),
            (
                "selected_plugins",
                "validate report profile_summary.selected_plugins must be a string array",
            ),
            (
                "target_mode",
                "validate report profile_summary.target_mode must be a string",
            ),
            (
                "target_platform",
                "validate report profile_summary.target_platform must be a string",
            ),
        )
        for field, expected_diagnostic in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_validate_report_with_strategies(out, ["library_embed"])
                    _write_compile_host_report(
                        out, out / "compile" / "zircon_runtime.exe"
                    )
                    _write_stage_report(out, "cook_assets", fatal=False)
                    _write_pack_report(out, out / "pack-output" / "assets.zrpack")
                    _write_stage_report(out, "platform_bundle", fatal=False)
                    validate_report_path = (
                        out / "stages" / "validate" / "report.json"
                    )
                    validate_report = json.loads(
                        validate_report_path.read_text(encoding="utf-8")
                    )
                    validate_report["profile_summary"].pop(field)
                    validate_report_path.write_text(
                        json.dumps(validate_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertEqual(report["fatal_stages"], ["Validate"])
                    self.assertTrue(
                        any(
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_validate_profile_summary_name_not_trimmed(
        self,
    ) -> None:
        cases = (
            "",
            " windows-release",
            "windows-release ",
        )
        for profile_name in cases:
            with self.subTest(profile_name=profile_name):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_validate_report_with_strategies(out, ["library_embed"])
                    _write_compile_host_report(
                        out, out / "compile" / "zircon_runtime.exe"
                    )
                    _write_stage_report(out, "cook_assets", fatal=False)
                    _write_pack_report(out, out / "pack-output" / "assets.zrpack")
                    _write_stage_report(out, "platform_bundle", fatal=False)
                    validate_report_path = (
                        out / "stages" / "validate" / "report.json"
                    )
                    validate_report = json.loads(
                        validate_report_path.read_text(encoding="utf-8")
                    )
                    validate_report["profile_summary"]["name"] = profile_name
                    validate_report_path.write_text(
                        json.dumps(validate_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertEqual(report["fatal_stages"], ["Validate"])
                    self.assertTrue(
                        any(
                            "validate report profile_summary.name "
                            "must be a non-empty trimmed profile name"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_validate_profile_summary_asset_filter_not_trimmed(
        self,
    ) -> None:
        cases = (
            "",
            " shipping",
            "shipping ",
        )
        for asset_filter in cases:
            with self.subTest(asset_filter=asset_filter):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_validate_report_with_strategies(out, ["library_embed"])
                    _write_compile_host_report(
                        out, out / "compile" / "zircon_runtime.exe"
                    )
                    _write_stage_report(out, "cook_assets", fatal=False)
                    _write_pack_report(out, out / "pack-output" / "assets.zrpack")
                    _write_stage_report(out, "platform_bundle", fatal=False)
                    validate_report_path = (
                        out / "stages" / "validate" / "report.json"
                    )
                    validate_report = json.loads(
                        validate_report_path.read_text(encoding="utf-8")
                    )
                    validate_report["profile_summary"][
                        "asset_filter"
                    ] = asset_filter
                    validate_report_path.write_text(
                        json.dumps(validate_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertEqual(report["fatal_stages"], ["Validate"])
                    self.assertTrue(
                        any(
                            "validate report profile_summary.asset_filter "
                            "must be a non-empty trimmed asset filter"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_validate_profile_feature_ids_invalid(
        self,
    ) -> None:
        cases = (
            (
                {"rendering": ["hdr"]},
                "validate report profile_summary.features.rendering feature id "
                "must use owner.feature dot namespace form",
            ),
            (
                {"rendering": ["rendering..hdr"]},
                "validate report profile_summary.features.rendering feature id "
                "must not contain empty namespace segments",
            ),
            (
                {"rendering": ["rendering.HDR"]},
                "validate report profile_summary.features.rendering feature id "
                "must contain only lowercase ASCII letters, digits, "
                "underscores, and dots",
            ),
            (
                {"rendering": ["physics.hdr"]},
                "validate report profile_summary.features.rendering feature id "
                "must be prefixed by project plugin rendering",
            ),
        )
        for features, expected_diagnostic in cases:
            with self.subTest(features=features):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_validate_report_with_strategies(out, ["library_embed"])
                    _write_compile_host_report(
                        out, out / "compile" / "zircon_runtime.exe"
                    )
                    _write_stage_report(out, "cook_assets", fatal=False)
                    _write_pack_report(out, out / "pack-output" / "assets.zrpack")
                    _write_stage_report(out, "platform_bundle", fatal=False)
                    validate_report_path = (
                        out / "stages" / "validate" / "report.json"
                    )
                    validate_report = json.loads(
                        validate_report_path.read_text(encoding="utf-8")
                    )
                    validate_report["profile_summary"]["features"] = features
                    validate_report_path.write_text(
                        json.dumps(validate_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertEqual(report["fatal_stages"], ["Validate"])
                    self.assertTrue(
                        any(
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_validate_profile_strategies_empty_as_schema(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, [])

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertEqual(report["fatal_stages"], ["Validate"])
            self.assertTrue(
                any(
                    "validate report profile_summary.strategies must include "
                    "at least one supported export strategy"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_validate_profile_strategies_unknown_as_schema(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["future_export_path"])

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertEqual(report["fatal_stages"], ["Validate"])
            self.assertTrue(
                any(
                    "unsupported export strategy future_export_path"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_validate_profile_strategies_not_trimmed(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, [" library_embed"])
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, out / "pack-output" / "assets.zrpack")
            _write_stage_report(out, "platform_bundle", fatal=False)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertEqual(report["fatal_stages"], ["Validate"])
            self.assertTrue(
                any(
                    "validate report profile_summary.strategies[0] must be "
                    "a non-empty trimmed export strategy"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_validate_profile_strategies_duplicate(
        self,
    ) -> None:
        cases = (
            (
                ["library_embed", "library_embed"],
                "validate report profile_summary.strategies[1] duplicates entry 0",
            ),
            (
                ["library_embed", "LibraryEmbed"],
                "validate report profile_summary.strategies[1] duplicates entry 0",
            ),
        )
        for strategies, expected_diagnostic in cases:
            with self.subTest(strategies=strategies):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_validate_report_with_strategies(out, strategies)
                    _write_compile_host_report(
                        out, out / "compile" / "zircon_runtime.exe"
                    )
                    _write_stage_report(out, "cook_assets", fatal=False)
                    _write_pack_report(out, out / "pack-output" / "assets.zrpack")
                    _write_stage_report(out, "platform_bundle", fatal=False)

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertEqual(report["fatal_stages"], ["Validate"])
                    self.assertTrue(
                        any(
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )


if __name__ == "__main__":
    unittest.main()
