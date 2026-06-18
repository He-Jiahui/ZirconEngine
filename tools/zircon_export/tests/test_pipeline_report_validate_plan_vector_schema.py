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


class PipelineReportValidatePlanVectorSchemaTests(unittest.TestCase):
    def test_report_stage_rejects_validate_native_dynamic_package_ids_not_trimmed(
        self,
    ) -> None:
        cases = (
            (
                "",
                (
                    "validate report plan_summary.native_dynamic_packages[0] "
                    "must be a non-empty trimmed native dynamic package id"
                ),
            ),
            (
                "animation ",
                (
                    "validate report plan_summary.native_dynamic_packages[0] "
                    "must be a non-empty trimmed native dynamic package id"
                ),
            ),
            (
                " animation",
                (
                    "validate report plan_summary.native_dynamic_packages[0] "
                    "must be a non-empty trimmed native dynamic package id"
                ),
            ),
        )
        for package_id, expected_diagnostic in cases:
            with self.subTest(package_id=package_id):
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
                    validate_report["plan_summary"] = {
                        "native_dynamic_packages": [package_id]
                    }
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

    def test_report_stage_rejects_validate_linked_runtime_crate_names_invalid(
        self,
    ) -> None:
        cases = (
            (
                "",
                (
                    "validate report plan_summary.linked_runtime_crates[0] must be "
                    "a non-empty trimmed runtime crate name"
                ),
            ),
            (
                "zircon_plugin_rendering_runtime ",
                (
                    "validate report plan_summary.linked_runtime_crates[0] must be "
                    "a non-empty trimmed runtime crate name"
                ),
            ),
            (
                "Zircon_plugin_rendering_runtime",
                (
                    "validate report plan_summary.linked_runtime_crates[0] must use "
                    "zircon_plugin_ crate prefix or builtin_ runtime-domain prefix "
                    "and contain only lowercase ASCII letters, digits, and underscores"
                ),
            ),
            (
                "rendering_runtime",
                (
                    "validate report plan_summary.linked_runtime_crates[0] must use "
                    "zircon_plugin_ crate prefix or builtin_ runtime-domain prefix "
                    "and contain only lowercase ASCII letters, digits, and underscores"
                ),
            ),
            (
                "zircon-plugin-rendering-runtime",
                (
                    "validate report plan_summary.linked_runtime_crates[0] must use "
                    "zircon_plugin_ crate prefix or builtin_ runtime-domain prefix "
                    "and contain only lowercase ASCII letters, digits, and underscores"
                ),
            ),
            (
                "zircon_plugin_rendering__",
                (
                    "validate report plan_summary.linked_runtime_crates[0] must not "
                    "end with an underscore or contain repeated underscores"
                ),
            ),
        )
        for crate_name, expected_diagnostic in cases:
            with self.subTest(crate_name=crate_name):
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
                    validate_report["plan_summary"] = {
                        "linked_runtime_crates": [crate_name]
                    }
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
