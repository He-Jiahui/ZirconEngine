from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.export_test_support import (
    _compile_host_plan,
    _write_compile_host_report,
    _write_pack_report,
    _write_stage_report,
    _write_validate_report_with_strategies,
)


class PipelineReportValidateSelectionSchemaTests(unittest.TestCase):
    def test_report_stage_rejects_validate_selected_plugins_non_string_array(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, out / "pack-output" / "assets.zrpack")
            _write_stage_report(out, "platform_bundle", fatal=False)
            validate_report_path = out / "stages" / "validate" / "report.json"
            validate_report = json.loads(
                validate_report_path.read_text(encoding="utf-8")
            )
            validate_report["profile_summary"]["selected_plugins"] = [
                "rendering",
                42,
            ]
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
                    "validate report profile_summary.selected_plugins[1] must be a string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_validate_selected_plugin_ids_invalid(
        self,
    ) -> None:
        cases = (
            (
                "",
                (
                    "validate report profile_summary.selected_plugins[0] must be "
                    "a non-empty trimmed project plugin id"
                ),
            ),
            (
                "rendering ",
                (
                    "validate report profile_summary.selected_plugins[0] must be "
                    "a non-empty trimmed project plugin id"
                ),
            ),
            (
                "Rendering",
                (
                    "validate report profile_summary.selected_plugins[0] must start "
                    "with a lowercase ASCII letter"
                ),
            ),
            (
                "rendering-plugin",
                (
                    "validate report profile_summary.selected_plugins[0] must contain "
                    "only lowercase ASCII letters, digits, and underscores"
                ),
            ),
            (
                "rendering__",
                (
                    "validate report profile_summary.selected_plugins[0] must not "
                    "end with an underscore or contain repeated underscores"
                ),
            ),
        )
        for plugin_id, expected_diagnostic in cases:
            with self.subTest(plugin_id=plugin_id):
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
                    validate_report["profile_summary"]["selected_plugins"] = [
                        plugin_id
                    ]
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

    def test_report_stage_rejects_validate_selected_plugins_duplicate(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, out / "pack-output" / "assets.zrpack")
            _write_stage_report(out, "platform_bundle", fatal=False)
            validate_report_path = out / "stages" / "validate" / "report.json"
            validate_report = json.loads(
                validate_report_path.read_text(encoding="utf-8")
            )
            validate_report["profile_summary"]["selected_plugins"] = [
                "rendering",
                "rendering",
            ]
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
                    "validate report profile_summary.selected_plugins[1] "
                    "duplicates entry 0"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_validate_enabled_runtime_plugin_ids_invalid(
        self,
    ) -> None:
        cases = (
            (
                "",
                (
                    "validate report plan_summary.enabled_runtime_plugins[0] must be "
                    "a non-empty trimmed project plugin id"
                ),
            ),
            (
                "rendering ",
                (
                    "validate report plan_summary.enabled_runtime_plugins[0] must be "
                    "a non-empty trimmed project plugin id"
                ),
            ),
            (
                "Rendering",
                (
                    "validate report plan_summary.enabled_runtime_plugins[0] must start "
                    "with a lowercase ASCII letter"
                ),
            ),
            (
                "rendering-plugin",
                (
                    "validate report plan_summary.enabled_runtime_plugins[0] must contain "
                    "only lowercase ASCII letters, digits, and underscores"
                ),
            ),
            (
                "rendering__",
                (
                    "validate report plan_summary.enabled_runtime_plugins[0] must not "
                    "end with an underscore or contain repeated underscores"
                ),
            ),
        )
        for plugin_id, expected_diagnostic in cases:
            with self.subTest(plugin_id=plugin_id):
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
                        "enabled_runtime_plugins": [plugin_id]
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

    def test_report_stage_rejects_validate_compile_host_expected_plugin_ids_invalid(
        self,
    ) -> None:
        cases = (
            (
                "",
                (
                    "validate report plan_summary.library_embed_compile_host."
                    "expected_runtime_plugins[0] must be a non-empty trimmed "
                    "project plugin id"
                ),
            ),
            (
                "rendering ",
                (
                    "validate report plan_summary.library_embed_compile_host."
                    "expected_runtime_plugins[0] must be a non-empty trimmed "
                    "project plugin id"
                ),
            ),
            (
                "Rendering",
                (
                    "validate report plan_summary.library_embed_compile_host."
                    "expected_runtime_plugins[0] must start with a lowercase "
                    "ASCII letter"
                ),
            ),
            (
                "rendering-plugin",
                (
                    "validate report plan_summary.library_embed_compile_host."
                    "expected_runtime_plugins[0] must contain only lowercase "
                    "ASCII letters, digits, and underscores"
                ),
            ),
            (
                "rendering__",
                (
                    "validate report plan_summary.library_embed_compile_host."
                    "expected_runtime_plugins[0] must not end with an underscore "
                    "or contain repeated underscores"
                ),
            ),
        )
        for plugin_id, expected_diagnostic in cases:
            with self.subTest(plugin_id=plugin_id):
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
                    compile_host_plan = _compile_host_plan()
                    compile_host_plan["expected_runtime_plugins"] = [plugin_id]
                    validate_report["plan_summary"] = {
                        "library_embed_compile_host": compile_host_plan
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
