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


class PipelineReportStageMetadataTests(unittest.TestCase):
    def test_report_stage_rejects_stage_report_without_profile(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            for stage in (
                "validate",
                "cook_assets",
                "pack",
                "platform_bundle",
            ):
                _write_stage_report(out, stage, fatal=False)
            _write_stage_report(out, "compile_host", fatal=False, profile=None)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertIn("CompileHost", report["fatal_stages"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "compile_host report profile is missing" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_stage_identity_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, out / "pack-output" / "assets.zrpack")
            _write_stage_report(out, "platform_bundle", fatal=False)
            compile_host_report_path = out / "stages" / "compile_host" / "report.json"
            compile_host_report = json.loads(
                compile_host_report_path.read_text(encoding="utf-8")
            )
            compile_host_report["stage"] = "Pack"
            compile_host_report_path.write_text(
                json.dumps(compile_host_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertIn("CompileHost", report["fatal_stages"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "compile_host report stage Pack does not match expected stage CompileHost"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_stage_report_without_boolean_fatal(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            for stage in (
                "validate",
                "compile_host",
                "cook_assets",
                "pack",
                "platform_bundle",
            ):
                _write_stage_report(out, stage, fatal=False)
            pack_report_path = out / "stages" / "pack" / "report.json"
            pack_report = json.loads(pack_report_path.read_text(encoding="utf-8"))
            del pack_report["fatal"]
            pack_report_path.write_text(
                json.dumps(pack_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertIn("Pack", report["fatal_stages"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "pack report fatal must be a boolean" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_stage_report_without_string_diagnostics(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            for stage in (
                "validate",
                "compile_host",
                "cook_assets",
                "pack",
                "platform_bundle",
            ):
                _write_stage_report(out, stage, fatal=False)
            cook_report_path = out / "stages" / "cook_assets" / "report.json"
            cook_report = json.loads(cook_report_path.read_text(encoding="utf-8"))
            cook_report["diagnostics"] = ["ok", {"not": "string"}]
            cook_report_path.write_text(
                json.dumps(cook_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertIn("CookAssets", report["fatal_stages"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "cook_assets report diagnostics[1] must be a string" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_compile_host_unknown_top_level_field(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, out / "pack-output" / "assets.zrpack")
            _write_stage_report(out, "platform_bundle", fatal=False)
            compile_host_report_path = out / "stages" / "compile_host" / "report.json"
            compile_host_report = json.loads(
                compile_host_report_path.read_text(encoding="utf-8")
            )
            compile_host_report["unsigned_sidecar"] = "sidecar.bin"
            compile_host_report_path.write_text(
                json.dumps(compile_host_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertIn("CompileHost", report["fatal_stages"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "compile_host report unknown field unsigned_sidecar"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_compile_host_command_non_string_array(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, out / "pack-output" / "assets.zrpack")
            _write_stage_report(out, "platform_bundle", fatal=False)
            compile_host_report_path = out / "stages" / "compile_host" / "report.json"
            compile_host_report = json.loads(
                compile_host_report_path.read_text(encoding="utf-8")
            )
            compile_host_report["command"] = ["cargo", 42]
            compile_host_report_path.write_text(
                json.dumps(compile_host_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertIn("CompileHost", report["fatal_stages"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "compile_host report command[1] must be a string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_accepts_compile_host_link_plan(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, out / "pack-output" / "assets.zrpack")
            _write_stage_report(out, "platform_bundle", fatal=False)
            compile_host_report_path = out / "stages" / "compile_host" / "report.json"
            compile_host_report = json.loads(
                compile_host_report_path.read_text(encoding="utf-8")
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])
            self.assertEqual(
                report["stages"][1]["report"]["link_plan"],
                compile_host_report["link_plan"],
            )

    def test_report_stage_rejects_compile_host_link_plan_validate_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, out / "pack-output" / "assets.zrpack")
            _write_stage_report(out, "platform_bundle", fatal=False)
            compile_host_report_path = out / "stages" / "compile_host" / "report.json"
            compile_host_report = json.loads(
                compile_host_report_path.read_text(encoding="utf-8")
            )
            compile_host_report["link_plan"] = {
                "app_features": ["target-client"],
                "runtime_features": ["target-client"],
                "expected_runtime_plugins": ["rendering"],
                "linked_runtime_crates": [
                    {
                        "crate_name": "zircon_plugin_rendering_runtime",
                        "path": "zircon_plugins/rendering/runtime",
                        "provider_package_id": "rendering",
                        "registration_kind": "runtime_plugin",
                    },
                ],
            }
            compile_host_report_path.write_text(
                json.dumps(compile_host_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "compile_host report link_plan.expected_runtime_plugins does not match "
                    "validate report plan_summary.library_embed_compile_host.expected_runtime_plugins"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_compile_host_link_plan_unknown_field(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, out / "pack-output" / "assets.zrpack")
            _write_stage_report(out, "platform_bundle", fatal=False)
            compile_host_report_path = out / "stages" / "compile_host" / "report.json"
            compile_host_report = json.loads(
                compile_host_report_path.read_text(encoding="utf-8")
            )
            compile_host_report["link_plan"] = {
                "app_features": ["target-client"],
                "runtime_features": ["target-client"],
                "expected_runtime_plugins": [],
                "linked_runtime_crates": [],
                "unsigned_sidecar": "sidecar.bin",
            }
            compile_host_report_path.write_text(
                json.dumps(compile_host_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertIn("CompileHost", report["fatal_stages"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "compile_host report link_plan unknown field unsigned_sidecar"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_compile_host_link_plan_invalid_shape(self) -> None:
        cases = (
            (
                "link_plan",
                "not-an-object",
                "compile_host report link_plan must be an object",
            ),
            (
                "app_features",
                ["target-client", 42],
                "compile_host report link_plan.app_features[1] must be a string",
            ),
            (
                "runtime_features",
                "target-client",
                "compile_host report link_plan.runtime_features must be a string array",
            ),
            (
                "expected_runtime_plugins",
                ["rendering", None],
                "compile_host report link_plan.expected_runtime_plugins[1] must be a string",
            ),
            (
                "linked_runtime_crates",
                ["zircon_plugin_rendering_runtime"],
                "compile_host report link_plan.linked_runtime_crates[0] must be an object",
            ),
        )
        for field, value, expected_diagnostic in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_validate_report_with_strategies(out, ["library_embed"])
                    _write_compile_host_report(
                        out,
                        out / "compile" / "zircon_runtime.exe",
                    )
                    _write_stage_report(out, "cook_assets", fatal=False)
                    _write_pack_report(out, out / "pack-output" / "assets.zrpack")
                    _write_stage_report(out, "platform_bundle", fatal=False)
                    compile_host_report_path = (
                        out / "stages" / "compile_host" / "report.json"
                    )
                    compile_host_report = json.loads(
                        compile_host_report_path.read_text(encoding="utf-8")
                    )
                    if field == "link_plan":
                        compile_host_report["link_plan"] = value
                    else:
                        compile_host_report["link_plan"] = {
                            "app_features": ["target-client"],
                            "runtime_features": ["target-client"],
                            "expected_runtime_plugins": [],
                            "linked_runtime_crates": [],
                            field: value,
                        }
                    compile_host_report_path.write_text(
                        json.dumps(compile_host_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertIn("CompileHost", report["fatal_stages"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_compile_host_log_line_non_string_array(self) -> None:
        for field in ("stderr_lines", "stdout_lines"):
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_validate_report_with_strategies(out, ["library_embed"])
                    _write_compile_host_report(
                        out,
                        out / "compile" / "zircon_runtime.exe",
                    )
                    _write_stage_report(out, "cook_assets", fatal=False)
                    _write_pack_report(out, out / "pack-output" / "assets.zrpack")
                    _write_stage_report(out, "platform_bundle", fatal=False)
                    compile_host_report_path = (
                        out / "stages" / "compile_host" / "report.json"
                    )
                    compile_host_report = json.loads(
                        compile_host_report_path.read_text(encoding="utf-8")
                    )
                    compile_host_report[field] = ["line", 42]
                    compile_host_report_path.write_text(
                        json.dumps(compile_host_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertIn("CompileHost", report["fatal_stages"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(
                            f"compile_host report {field}[1] must be a string"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_compile_host_link_plan_missing_evidence_field(
        self,
    ) -> None:
        cases = (
            (
                "app_features",
                "compile_host report link_plan.app_features must be a string array",
            ),
            (
                "runtime_features",
                "compile_host report link_plan.runtime_features must be a string array",
            ),
            (
                "expected_runtime_plugins",
                "compile_host report link_plan.expected_runtime_plugins must be a string array",
            ),
            (
                "linked_runtime_crates",
                "compile_host report link_plan.linked_runtime_crates must be an object array",
            ),
        )
        for field, expected_diagnostic in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_validate_report_with_strategies(out, ["library_embed"])
                    _write_compile_host_report(
                        out,
                        out / "compile" / "zircon_runtime.exe",
                    )
                    _write_stage_report(out, "cook_assets", fatal=False)
                    _write_pack_report(out, out / "pack-output" / "assets.zrpack")
                    _write_stage_report(out, "platform_bundle", fatal=False)
                    compile_host_report_path = (
                        out / "stages" / "compile_host" / "report.json"
                    )
                    compile_host_report = json.loads(
                        compile_host_report_path.read_text(encoding="utf-8")
                    )
                    compile_host_report["link_plan"].pop(field)
                    compile_host_report_path.write_text(
                        json.dumps(compile_host_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertIn("CompileHost", report["fatal_stages"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_compile_host_missing_log_line_array(self) -> None:
        for field in ("stderr_lines", "stdout_lines"):
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_validate_report_with_strategies(out, ["library_embed"])
                    _write_compile_host_report(
                        out,
                        out / "compile" / "zircon_runtime.exe",
                    )
                    _write_stage_report(out, "cook_assets", fatal=False)
                    _write_pack_report(out, out / "pack-output" / "assets.zrpack")
                    _write_stage_report(out, "platform_bundle", fatal=False)
                    compile_host_report_path = (
                        out / "stages" / "compile_host" / "report.json"
                    )
                    compile_host_report = json.loads(
                        compile_host_report_path.read_text(encoding="utf-8")
                    )
                    compile_host_report.pop(field)
                    compile_host_report_path.write_text(
                        json.dumps(compile_host_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertIn("CompileHost", report["fatal_stages"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(
                            f"compile_host report {field} must be a string array"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_compile_host_missing_release_evidence_field(
        self,
    ) -> None:
        cases = (
            ("command", "compile_host report command must be a string array"),
            ("host_executable", "compile_host report host_executable must be a string"),
            ("exit_code", "compile_host report exit_code must be an integer"),
            ("link_plan", "compile_host report link_plan must be an object"),
        )
        for field, expected_diagnostic in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_validate_report_with_strategies(out, ["library_embed"])
                    _write_compile_host_report(
                        out,
                        out / "compile" / "zircon_runtime.exe",
                    )
                    _write_stage_report(out, "cook_assets", fatal=False)
                    _write_pack_report(out, out / "pack-output" / "assets.zrpack")
                    _write_stage_report(out, "platform_bundle", fatal=False)
                    compile_host_report_path = (
                        out / "stages" / "compile_host" / "report.json"
                    )
                    compile_host_report = json.loads(
                        compile_host_report_path.read_text(encoding="utf-8")
                    )
                    compile_host_report.pop(field)
                    compile_host_report_path.write_text(
                        json.dumps(compile_host_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertIn("CompileHost", report["fatal_stages"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_compile_host_host_executable_non_string(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(
                out,
                out / "compile" / "zircon_runtime.exe",
                host_value=42,
            )
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, out / "pack-output" / "assets.zrpack")
            _write_stage_report(out, "platform_bundle", fatal=False)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertIn("CompileHost", report["fatal_stages"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "compile_host report host_executable must be a string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_compile_host_exit_code_non_integer(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_strategies(out, ["library_embed"])
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, out / "pack-output" / "assets.zrpack")
            _write_stage_report(out, "platform_bundle", fatal=False)
            compile_host_report_path = out / "stages" / "compile_host" / "report.json"
            compile_host_report = json.loads(
                compile_host_report_path.read_text(encoding="utf-8")
            )
            compile_host_report["exit_code"] = "0"
            compile_host_report_path.write_text(
                json.dumps(compile_host_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertIn("CompileHost", report["fatal_stages"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "compile_host report exit_code must be an integer"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
