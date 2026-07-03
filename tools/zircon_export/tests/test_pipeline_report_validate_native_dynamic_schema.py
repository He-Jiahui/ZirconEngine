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
    _write_validate_report_with_native_dynamic_exports,
)
from tools.zircon_export.tests.native_dynamic_export_test_support import (
    _write_native_dynamic_report,
    _write_native_dynamic_stage_plugins,
)


class PipelineReportValidateNativeDynamicSchemaTests(unittest.TestCase):
    def test_report_stage_rejects_validate_native_dynamic_export_unknown_field(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_native_dynamic_exports(out)
            native_plugins = _write_native_dynamic_stage_plugins(
                out / "stages" / "native_dynamic"
            )
            _write_native_dynamic_report(out, native_plugins)
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, out / "pack-output" / "assets.zrpack")
            _write_stage_report(out, "platform_bundle", fatal=False)
            validate_report_path = out / "stages" / "validate" / "report.json"
            validate_report = json.loads(
                validate_report_path.read_text(encoding="utf-8")
            )
            package_export = validate_report["plan_summary"][
                "native_dynamic_package_exports"
            ][0]
            package_export["unsigned_sidecar"] = "sidecar.bin"
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
                    "validate report plan_summary.native_dynamic_package_exports[0] unknown field unsigned_sidecar"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_validate_native_dynamic_export_non_object(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_native_dynamic_exports(out)
            native_plugins = _write_native_dynamic_stage_plugins(
                out / "stages" / "native_dynamic"
            )
            _write_native_dynamic_report(out, native_plugins)
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, out / "pack-output" / "assets.zrpack")
            _write_stage_report(out, "platform_bundle", fatal=False)
            validate_report_path = out / "stages" / "validate" / "report.json"
            validate_report = json.loads(
                validate_report_path.read_text(encoding="utf-8")
            )
            validate_report["plan_summary"]["native_dynamic_package_exports"] = [
                "animation"
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
                    "validate report plan_summary.native_dynamic_package_exports[0] must be an object"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_validate_native_dynamic_export_string_fields_non_string(
        self,
    ) -> None:
        package_export_string_fields = (
            "directory",
            "manifest",
            "package_id",
            "package_report",
            "path",
        )
        for field in package_export_string_fields:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_validate_report_with_native_dynamic_exports(out)
                    native_plugins = _write_native_dynamic_stage_plugins(
                        out / "stages" / "native_dynamic"
                    )
                    _write_native_dynamic_report(out, native_plugins)
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
                    package_export = validate_report["plan_summary"][
                        "native_dynamic_package_exports"
                    ][0]
                    package_export[field] = 42
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
                            "validate report plan_summary.native_dynamic_package_exports"
                            f"[0].{field} must be a string" in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_validate_native_dynamic_export_missing_required_field(
        self,
    ) -> None:
        cases = (
            ("package_id", "must be a string"),
            ("directory", "must be a string"),
            ("path", "must be a string"),
            ("manifest", "must be a string"),
            ("package_report", "must be a string"),
            ("abi", "must be an object"),
        )
        for field, expected_type in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    validate_report_path = self._write_native_dynamic_validate_fixture(
                        out,
                    )
                    validate_report = json.loads(
                        validate_report_path.read_text(encoding="utf-8")
                    )
                    package_export = validate_report["plan_summary"][
                        "native_dynamic_package_exports"
                    ][0]
                    package_export.pop(field)
                    validate_report_path.write_text(
                        json.dumps(validate_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self._assert_validate_fatal(report)
                    self._assert_diagnostic_contains(
                        report,
                        "validate report plan_summary.native_dynamic_package_exports"
                        f"[0].{field} {expected_type}",
                    )

    def test_report_stage_rejects_validate_native_dynamic_missing_package_exports(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            validate_report_path = self._write_native_dynamic_validate_fixture(out)
            validate_report = json.loads(
                validate_report_path.read_text(encoding="utf-8")
            )
            validate_report["plan_summary"].pop("native_dynamic_package_exports")
            validate_report_path.write_text(
                json.dumps(validate_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self._assert_validate_fatal(report)
            self._assert_diagnostic_contains(
                report,
                "validate report plan_summary.native_dynamic_package_exports "
                "must be a list",
            )

    def test_report_stage_rejects_validate_native_dynamic_export_abi_non_object(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_native_dynamic_exports(out)
            native_plugins = _write_native_dynamic_stage_plugins(
                out / "stages" / "native_dynamic"
            )
            _write_native_dynamic_report(out, native_plugins)
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, out / "pack-output" / "assets.zrpack")
            _write_stage_report(out, "platform_bundle", fatal=False)
            validate_report_path = out / "stages" / "validate" / "report.json"
            validate_report = json.loads(
                validate_report_path.read_text(encoding="utf-8")
            )
            package_export = validate_report["plan_summary"][
                "native_dynamic_package_exports"
            ][0]
            package_export["abi"] = "NativePluginAbiV3"
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
                    "validate report plan_summary.native_dynamic_package_exports[0].abi must be an object"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_validate_native_dynamic_abi_unknown_field(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_native_dynamic_exports(out)
            native_plugins = _write_native_dynamic_stage_plugins(
                out / "stages" / "native_dynamic"
            )
            _write_native_dynamic_report(out, native_plugins)
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, out / "pack-output" / "assets.zrpack")
            _write_stage_report(out, "platform_bundle", fatal=False)
            validate_report_path = out / "stages" / "validate" / "report.json"
            validate_report = json.loads(
                validate_report_path.read_text(encoding="utf-8")
            )
            abi = validate_report["plan_summary"]["native_dynamic_package_exports"][0][
                "abi"
            ]
            abi["unsigned_sidecar"] = "sidecar.bin"
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
                    "validate report plan_summary.native_dynamic_package_exports[0].abi unknown field unsigned_sidecar"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_validate_native_dynamic_abi_version_non_integer(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_validate_report_with_native_dynamic_exports(out)
            native_plugins = _write_native_dynamic_stage_plugins(
                out / "stages" / "native_dynamic"
            )
            _write_native_dynamic_report(out, native_plugins)
            _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
            _write_stage_report(out, "cook_assets", fatal=False)
            _write_pack_report(out, out / "pack-output" / "assets.zrpack")
            _write_stage_report(out, "platform_bundle", fatal=False)
            validate_report_path = out / "stages" / "validate" / "report.json"
            validate_report = json.loads(
                validate_report_path.read_text(encoding="utf-8")
            )
            abi = validate_report["plan_summary"]["native_dynamic_package_exports"][0][
                "abi"
            ]
            abi["abi_version"] = "3"
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
                    "validate report plan_summary.native_dynamic_package_exports[0].abi.abi_version must be an integer"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_validate_native_dynamic_abi_string_fields_non_string(
        self,
    ) -> None:
        abi_string_fields = (
            "behavior_contract",
            "bridge_method_table",
            "descriptor_contract",
            "descriptor_symbol",
            "editor_entry_source",
            "entry_report_contract",
            "host_function_table",
            "runtime_entry_source",
            "state_snapshot_contract",
        )
        for field in abi_string_fields:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    _write_validate_report_with_native_dynamic_exports(out)
                    native_plugins = _write_native_dynamic_stage_plugins(
                        out / "stages" / "native_dynamic"
                    )
                    _write_native_dynamic_report(out, native_plugins)
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
                    abi = validate_report["plan_summary"][
                        "native_dynamic_package_exports"
                    ][0]["abi"]
                    abi[field] = 42
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
                            "validate report plan_summary.native_dynamic_package_exports"
                            f"[0].abi.{field} must be a string" in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_validate_native_dynamic_abi_missing_required_field(
        self,
    ) -> None:
        cases = (
            ("abi_version", "must be an integer"),
            ("behavior_contract", "must be a string"),
            ("bridge_method_table", "must be a string"),
            ("descriptor_contract", "must be a string"),
            ("descriptor_symbol", "must be a string"),
            ("editor_entry_source", "must be a string"),
            ("entry_report_contract", "must be a string"),
            ("host_function_table", "must be a string"),
            ("runtime_entry_source", "must be a string"),
            ("state_snapshot_contract", "must be a string"),
        )
        for field, expected_type in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    validate_report_path = self._write_native_dynamic_validate_fixture(
                        out,
                    )
                    validate_report = json.loads(
                        validate_report_path.read_text(encoding="utf-8")
                    )
                    abi = validate_report["plan_summary"][
                        "native_dynamic_package_exports"
                    ][0]["abi"]
                    abi.pop(field)
                    validate_report_path.write_text(
                        json.dumps(validate_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self._assert_validate_fatal(report)
                    self._assert_diagnostic_contains(
                        report,
                        "validate report plan_summary.native_dynamic_package_exports"
                        f"[0].abi.{field} {expected_type}",
                    )

    def test_report_stage_rejects_validate_native_dynamic_export_path_contract_mismatch(
        self,
    ) -> None:
        cases = (
            (
                {"package_id": ""},
                (
                    "validate report plan_summary.native_dynamic_package_exports"
                    "[0].package_id must be a non-empty trimmed native dynamic "
                    "package id"
                ),
            ),
            (
                {"package_id": "animation.fx", "directory": "animation"},
                (
                    "validate report plan_summary.native_dynamic_package_exports"
                    "[0].directory must be animation_fx for package_id "
                    "animation.fx"
                ),
            ),
            (
                {"directory": " animation"},
                (
                    "validate report plan_summary.native_dynamic_package_exports"
                    "[0].directory must be a non-empty trimmed string"
                ),
            ),
            (
                {"path": "plugins/wrong"},
                (
                    "validate report plan_summary.native_dynamic_package_exports"
                    "[0].path must be plugins/animation for directory animation"
                ),
            ),
            (
                {"manifest": "plugins/animation/wrong.toml"},
                (
                    "validate report plan_summary.native_dynamic_package_exports"
                    "[0].manifest must be plugins/animation/plugin.toml for "
                    "directory animation"
                ),
            ),
            (
                {"package_report": "plugins/animation/wrong.toml"},
                (
                    "validate report plan_summary.native_dynamic_package_exports"
                    "[0].package_report must be "
                    "plugins/animation/native_dynamic_package.toml for "
                    "directory animation"
                ),
            ),
        )
        for package_export_overrides, expected_diagnostic in cases:
            with self.subTest(package_export_overrides=package_export_overrides):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    self._write_native_dynamic_validate_fixture(
                        out,
                        package_export_overrides=package_export_overrides,
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self._assert_validate_fatal(report)
                    self._assert_diagnostic_contains(report, expected_diagnostic)

    def test_report_stage_rejects_validate_native_dynamic_abi_v3_contract_mismatch(
        self,
    ) -> None:
        cases = (
            (
                "abi_version",
                2,
                (
                    "validate report plan_summary.native_dynamic_package_exports"
                    "[0].abi.abi_version must be 3"
                ),
            ),
            (
                "descriptor_symbol",
                "zircon_plugin_descriptor_v3",
                (
                    "validate report plan_summary.native_dynamic_package_exports"
                    "[0].abi.descriptor_symbol must be "
                    "zircon_native_plugin_descriptor_v3"
                ),
            ),
            (
                "host_function_table",
                " NativePluginHostFunctionTableV3",
                (
                    "validate report plan_summary.native_dynamic_package_exports"
                    "[0].abi.host_function_table must be a non-empty trimmed "
                    "string"
                ),
            ),
            (
                "bridge_method_table",
                "",
                (
                    "validate report plan_summary.native_dynamic_package_exports"
                    "[0].abi.bridge_method_table must be a non-empty trimmed "
                    "string"
                ),
            ),
        )
        for field, value, expected_diagnostic in cases:
            with self.subTest(field=field, value=value):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    validate_report_path = self._write_native_dynamic_validate_fixture(
                        out,
                    )
                    validate_report = json.loads(
                        validate_report_path.read_text(encoding="utf-8")
                    )
                    abi = validate_report["plan_summary"][
                        "native_dynamic_package_exports"
                    ][0]["abi"]
                    abi[field] = value
                    validate_report_path.write_text(
                        json.dumps(validate_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self._assert_validate_fatal(report)
                    self._assert_diagnostic_contains(report, expected_diagnostic)

    def _write_native_dynamic_validate_fixture(
        self,
        out: Path,
        *,
        package_export_overrides: dict[str, object] | None = None,
    ) -> Path:
        _write_validate_report_with_native_dynamic_exports(
            out,
            package_export_overrides=package_export_overrides,
        )
        native_plugins = _write_native_dynamic_stage_plugins(
            out / "stages" / "native_dynamic"
        )
        _write_native_dynamic_report(out, native_plugins)
        _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
        _write_stage_report(out, "cook_assets", fatal=False)
        _write_pack_report(out, out / "pack-output" / "assets.zrpack")
        _write_stage_report(out, "platform_bundle", fatal=False)
        return out / "stages" / "validate" / "report.json"

    def _assert_validate_fatal(self, report: dict[str, object]) -> None:
        self.assertTrue(report["fatal"])
        self.assertEqual(report["missing_stages"], [])
        self.assertEqual(report["fatal_stages"], ["Validate"])

    def _assert_diagnostic_contains(
        self,
        report: dict[str, object],
        expected_diagnostic: str,
    ) -> None:
        self.assertTrue(
            any(
                expected_diagnostic in diagnostic
                for diagnostic in report["diagnostics"]
            ),
            report["diagnostics"],
        )


if __name__ == "__main__":
    unittest.main()
