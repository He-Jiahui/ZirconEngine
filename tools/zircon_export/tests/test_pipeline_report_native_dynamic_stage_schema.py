from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.export_test_support import (
    _write_compile_host_report,
    _write_native_dynamic_report,
    _write_native_dynamic_stage_plugins,
    _write_pack_report,
    _write_stage_report,
    _write_validate_report_with_native_dynamic_exports,
)


class PipelineReportNativeDynamicStageSchemaTests(unittest.TestCase):
    def _write_native_dynamic_reports(self, out: Path) -> Path:
        _write_validate_report_with_native_dynamic_exports(out)
        native_plugins = _write_native_dynamic_stage_plugins(
            out / "stages" / "native_dynamic"
        )
        _write_native_dynamic_report(out, native_plugins)
        _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
        _write_stage_report(out, "cook_assets", fatal=False)
        _write_pack_report(out, out / "pack-output" / "assets.zrpack")
        _write_stage_report(out, "platform_bundle", fatal=False)
        return out / "stages" / "native_dynamic" / "report.json"

    def _assert_native_dynamic_report_field_diagnostic(
        self,
        field: str,
        value: object,
        expected_diagnostic: str,
        unexpected_diagnostic: str | None = None,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = self._write_native_dynamic_reports(out)
            native_report = json.loads(
                native_report_path.read_text(encoding="utf-8")
            )
            native_report[field] = value
            native_report_path.write_text(
                json.dumps(native_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("NativeDynamic", report["fatal_stages"])
            self.assertTrue(
                any(
                    expected_diagnostic in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            if unexpected_diagnostic is not None:
                self.assertFalse(
                    any(
                        unexpected_diagnostic in diagnostic
                        for diagnostic in report["diagnostics"]
                    ),
                    report["diagnostics"],
                )

    def test_report_stage_rejects_native_dynamic_unknown_top_level_field(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = self._write_native_dynamic_reports(out)
            native_report = json.loads(
                native_report_path.read_text(encoding="utf-8")
            )
            native_report["unsigned_sidecar"] = "sidecar.bin"
            native_report_path.write_text(
                json.dumps(native_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("NativeDynamic", report["fatal_stages"])
            self.assertTrue(
                any(
                    "native_dynamic report unknown field unsigned_sidecar"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_string_fields_non_string(
        self,
    ) -> None:
        for field in (
            "cleanup_reason",
            "content_hash",
            "loader_manifest",
            "native_plugin_root",
            "plugins_dir",
            "stage_output",
            "target_platform",
            "validate_report",
        ):
            with self.subTest(field=field):
                self._assert_native_dynamic_report_field_diagnostic(
                    field,
                    42,
                    f"native_dynamic report {field} must be a string",
                )

    def test_report_stage_rejects_native_dynamic_string_array_fields_non_string_array(
        self,
    ) -> None:
        for field in ("artifact_extensions", "native_dynamic_packages"):
            with self.subTest(field=field):
                self._assert_native_dynamic_report_field_diagnostic(
                    field,
                    ["zircon_plugin_animation.dll", 42],
                    f"native_dynamic report {field} must be a string array",
                )

    def test_report_stage_rejects_native_dynamic_count_fields_non_integer(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "package_count",
            "1",
            "native_dynamic report package_count must be an integer",
        )

    def test_report_stage_rejects_native_dynamic_bool_fields_non_bool(self) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "payload_cleaned",
            "true",
            "native_dynamic report payload_cleaned must be a boolean",
        )

    def test_report_stage_rejects_native_dynamic_object_fields_non_object(
        self,
    ) -> None:
        for field in (
            "native_build_execution",
            "native_build_plan",
            "native_notarization",
            "native_signing",
        ):
            with self.subTest(field=field):
                self._assert_native_dynamic_report_field_diagnostic(
                    field,
                    "not-an-object",
                    f"native_dynamic report {field} must be an object",
                )

    def test_report_stage_rejects_native_dynamic_missing_release_evidence_field(
        self,
    ) -> None:
        cases = (
            (
                "artifact_extensions",
                "native_dynamic report artifact_extensions must be a string array",
            ),
            ("plugins_dir", "native_dynamic report plugins_dir must be a string"),
            ("stage_output", "native_dynamic report stage_output must be a string"),
            (
                "target_platform",
                "native_dynamic report target_platform must be a string",
            ),
            (
                "validate_report",
                "native_dynamic report validate_report must be a string",
            ),
            (
                "loader_manifest",
                "native_dynamic report loader_manifest must be a string",
            ),
            (
                "file_manifest",
                "native_dynamic report file_manifest must be an object array",
            ),
            ("content_hash", "native_dynamic report content_hash must be a string"),
            (
                "materialized_packages",
                "native_dynamic report materialized_packages must be an object array",
            ),
            (
                "package_exports",
                "native_dynamic report package_exports must be an object array",
            ),
            (
                "native_dynamic_packages",
                "native_dynamic report native_dynamic_packages must be a string array",
            ),
            (
                "native_plugin_root",
                "native_dynamic report native_plugin_root must be a string",
            ),
            ("package_count", "native_dynamic report package_count must be an integer"),
            (
                "native_build_plan",
                "native_dynamic report native_build_plan must be an object",
            ),
            (
                "native_build_execution",
                "native_dynamic report native_build_execution must be an object",
            ),
            (
                "native_signing",
                "native_dynamic report native_signing must be an object",
            ),
            (
                "native_notarization",
                "native_dynamic report native_notarization must be an object",
            ),
        )
        for field, expected_diagnostic in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    native_report_path = self._write_native_dynamic_reports(out)
                    native_report = json.loads(
                        native_report_path.read_text(encoding="utf-8")
                    )
                    native_report.pop(field, None)
                    native_report_path.write_text(
                        json.dumps(native_report, indent=2),
                        encoding="utf-8",
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertIn("NativeDynamic", report["fatal_stages"])
                    self.assertTrue(
                        any(
                            expected_diagnostic in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_native_dynamic_build_plan_unknown_field(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_plan",
            _native_build_plan(unsigned_sidecar="target/sidecar.bin"),
            "native_dynamic report native_build_plan unknown field unsigned_sidecar",
        )

    def test_report_stage_rejects_native_dynamic_build_plan_missing_release_evidence_field(
        self,
    ) -> None:
        cases = (
            (
                "workspace_manifest",
                "native_dynamic report native_build_plan.workspace_manifest must be a string",
            ),
            (
                "target_dir",
                "native_dynamic report native_build_plan.target_dir must be a string",
            ),
            (
                "cargo_profile",
                "native_dynamic report native_build_plan.cargo_profile must be a string",
            ),
            (
                "release",
                "native_dynamic report native_build_plan.release must be a boolean",
            ),
            (
                "build_features",
                "native_dynamic report native_build_plan.build_features must be a string array",
            ),
            (
                "package_count",
                "native_dynamic report native_build_plan.package_count must be an integer",
            ),
            (
                "diagnostics",
                "native_dynamic report native_build_plan.diagnostics must be a string array",
            ),
            (
                "packages",
                "native_dynamic report native_build_plan.packages must be an object array",
            ),
            (
                "fatal",
                "native_dynamic report native_build_plan.fatal must be a boolean",
            ),
        )
        for field, expected_diagnostic in cases:
            with self.subTest(field=field):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_build_plan",
                    _native_build_plan_without(field),
                    expected_diagnostic,
                )

    def test_report_stage_rejects_native_dynamic_build_plan_field_types(
        self,
    ) -> None:
        cases = (
            ("fatal", "false", "must be a boolean"),
            ("diagnostics", [42], "must be a string array"),
            ("workspace_manifest", 42, "must be a string"),
            ("target_dir", 42, "must be a string"),
            ("cargo_profile", 42, "must be a string"),
            ("release", "true", "must be a boolean"),
            ("build_features", [42], "must be a string array"),
            ("package_count", "1", "must be an integer"),
            ("packages", "not-an-array", "must be an object array"),
            ("packages[0]", [42], "must be an object"),
        )
        for expected_field, value, expected_type in cases:
            field = expected_field.split("[", maxsplit=1)[0]
            with self.subTest(field=expected_field, value=value):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_build_plan",
                    _native_build_plan(**{field: value}),
                    "native_dynamic report "
                    f"native_build_plan.{expected_field} {expected_type}",
                )

    def test_report_stage_rejects_native_dynamic_build_plan_package_unknown_field(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_plan",
            _native_build_plan(
                packages=[
                    _native_build_plan_package(
                        unsigned_sidecar="target/sidecar.bin"
                    )
                ]
            ),
            "native_dynamic report native_build_plan.packages[0] "
            "unknown field unsigned_sidecar",
        )

    def test_report_stage_rejects_native_dynamic_build_plan_package_field_types(
        self,
    ) -> None:
        cases = (
            ("package_id", 42, "must be a string"),
            ("crate_name", 42, "must be a string"),
            ("manifest_path", 42, "must be a string"),
            ("workspace_manifest", 42, "must be a string"),
            ("target_dir", 42, "must be a string"),
            ("cargo_profile", 42, "must be a string"),
            ("expected_loadable_artifact", 42, "must be a string"),
            ("release", "true", "must be a boolean"),
            ("features", [42], "must be a string array"),
            ("command", [42], "must be a string array"),
        )
        for field, value, expected_type in cases:
            with self.subTest(field=field, value=value):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_build_plan",
                    _native_build_plan(
                        packages=[_native_build_plan_package(**{field: value})]
                    ),
                    "native_dynamic report native_build_plan."
                    f"packages[0].{field} {expected_type}",
                )

    def test_report_stage_rejects_native_dynamic_build_execution_unknown_field(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_execution",
            _native_build_execution(unsigned_sidecar="target/sidecar.bin"),
            "native_dynamic report native_build_execution unknown field unsigned_sidecar",
        )

    def test_report_stage_rejects_native_dynamic_build_execution_missing_release_evidence_field(
        self,
    ) -> None:
        cases = (
            (
                "enabled",
                "native_dynamic report native_build_execution.enabled must be a boolean",
            ),
            (
                "fatal",
                "native_dynamic report native_build_execution.fatal must be a boolean",
            ),
            (
                "diagnostics",
                "native_dynamic report native_build_execution.diagnostics must be a string array",
            ),
            (
                "package_count",
                "native_dynamic report native_build_execution.package_count must be an integer",
            ),
            (
                "packages",
                "native_dynamic report native_build_execution.packages must be an object array",
            ),
        )
        for field, expected_diagnostic in cases:
            with self.subTest(field=field):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_build_execution",
                    _native_build_execution_without(field),
                    expected_diagnostic,
                )

    def test_report_stage_rejects_native_dynamic_build_execution_field_types(
        self,
    ) -> None:
        cases = (
            ("enabled", "true", "must be a boolean"),
            ("fatal", "false", "must be a boolean"),
            ("skipped", "true", "must be a boolean"),
            ("skip_reason", 42, "must be a string"),
            ("diagnostics", [42], "must be a string array"),
            ("package_count", "1", "must be an integer"),
            ("packages", "not-an-array", "must be an object array"),
            ("packages[0]", [42], "must be an object"),
        )
        for expected_field, value, expected_type in cases:
            field = expected_field.split("[", maxsplit=1)[0]
            with self.subTest(field=expected_field, value=value):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_build_execution",
                    _native_build_execution(**{field: value}),
                    "native_dynamic report "
                    f"native_build_execution.{expected_field} {expected_type}",
                )

    def test_report_stage_rejects_native_dynamic_build_execution_package_unknown_field(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_build_execution",
            _native_build_execution(
                packages=[
                    _native_build_execution_package(
                        unsigned_sidecar="target/sidecar.bin"
                    )
                ]
            ),
            "native_dynamic report native_build_execution.packages[0] "
            "unknown field unsigned_sidecar",
        )

    def test_report_stage_rejects_native_dynamic_build_execution_package_field_types(
        self,
    ) -> None:
        cases = (
            ("package_id", 42, "must be a string"),
            ("crate_name", 42, "must be a string"),
            ("command", [42], "must be a string array"),
            ("exit_code", "0", "must be an integer"),
            ("stdout", 42, "must be a string"),
            ("stderr", 42, "must be a string"),
            ("expected_loadable_artifact", 42, "must be a string"),
            ("copied_loadable_artifact", 42, "must be a string"),
            ("copied_sidecars", [42], "must be a string array"),
        )
        for field, value, expected_type in cases:
            with self.subTest(field=field, value=value):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_build_execution",
                    _native_build_execution(
                        packages=[
                            _native_build_execution_package(**{field: value})
                        ]
                    ),
                    "native_dynamic report native_build_execution."
                    f"packages[0].{field} {expected_type}",
                )

    def test_report_stage_rejects_native_dynamic_operation_audit_unknown_field(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "native_signing",
            {
                "enabled": False,
                "profile": None,
                "target_platform": "windows-x86_64",
                "allowed_platforms": [],
                "platform_allowed": True,
                "fatal": False,
                "package_count": 0,
                "unsigned_sidecar": "plugins/animation/sidecar.bin",
            },
            "native_dynamic report native_signing unknown field unsigned_sidecar",
        )

    def test_report_stage_rejects_native_dynamic_operation_audit_missing_stage_evidence_field(
        self,
    ) -> None:
        cases = (
            (
                "diagnostics",
                "native_dynamic report native_signing.diagnostics must be a string array",
            ),
            (
                "packages",
                "native_dynamic report native_signing packages must be an object array",
            ),
        )
        for field, expected_diagnostic in cases:
            with self.subTest(field=field):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_signing",
                    _disabled_native_operation_audit_without(field),
                    expected_diagnostic,
                    "NativeDynamic report native_signing is malformed",
                )

    def test_report_stage_rejects_native_dynamic_operation_audit_field_types(
        self,
    ) -> None:
        cases = (
            ("enabled", "true", "must be a boolean"),
            ("profile", 42, "must be a string"),
            ("target_platform", 42, "must be a string"),
            ("allowed_platforms", "windows-x86_64", "must be a string array"),
            ("allowed_platforms", [42], "must be a string array"),
            ("platform_allowed", "true", "must be a boolean"),
            ("fatal", "false", "must be a boolean"),
            ("package_count", "1", "must be an integer"),
        )
        for field, value, expected_type in cases:
            with self.subTest(field=field, value=value):
                audit = {
                    "enabled": False,
                    "profile": None,
                    "target_platform": "windows-x86_64",
                    "allowed_platforms": [],
                    "platform_allowed": True,
                    "fatal": False,
                    "package_count": 0,
                    field: value,
                }
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_signing",
                    audit,
                    f"native_dynamic report native_signing.{field} {expected_type}",
                    "NativeDynamic report native_signing is malformed",
                )

    def test_report_stage_rejects_native_dynamic_operation_audit_package_missing_required_field(
        self,
    ) -> None:
        cases = (
            ("package_id", "must be a string"),
            ("artifact_count", "must be an integer"),
            ("artifacts", "must be an object array"),
        )
        for field, expected_type in cases:
            with self.subTest(field=field):
                package = _native_operation_audit_package()
                package.pop(field)
                audit = _native_operation_audit(packages=[package])
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_signing",
                    audit,
                    "native_dynamic report native_signing "
                    f"packages[0].{field} {expected_type}",
                    "NativeDynamic report native_signing is malformed",
                )

    def test_report_stage_rejects_native_dynamic_operation_audit_artifact_missing_required_field(
        self,
    ) -> None:
        cases = (
            ("artifact", "must be a string"),
            ("package_relative_artifact", "must be a string"),
            ("stdout", "must be a string"),
            ("stderr", "must be a string"),
            ("command", "must be a string array"),
        )
        for field, expected_type in cases:
            with self.subTest(field=field):
                artifact = _native_operation_audit_artifact()
                artifact.pop(field)
                package = _native_operation_audit_package(artifacts=[artifact])
                audit = _native_operation_audit(packages=[package])
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_signing",
                    audit,
                    "native_dynamic report native_signing packages[0] "
                    f"artifacts[0].{field} {expected_type}",
                    "NativeDynamic report native_signing is malformed",
                )

    def test_report_stage_rejects_native_dynamic_object_array_fields_non_object_array(
        self,
    ) -> None:
        for field in ("file_manifest", "materialized_packages", "package_exports"):
            with self.subTest(field=field):
                self._assert_native_dynamic_report_field_diagnostic(
                    field,
                    "not-an-object-array",
                    f"native_dynamic report {field} must be an object array",
                )

    def test_report_stage_rejects_native_dynamic_package_export_unknown_field(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "package_exports",
            [
                {
                    "package_id": "animation",
                    "directory": "animation",
                    "path": "plugins/animation",
                    "manifest": "plugins/animation/plugin.toml",
                    "package_report": "plugins/animation/native_dynamic_package.toml",
                    "unsigned_sidecar": "plugins/animation/sidecar.bin",
                }
            ],
            "native_dynamic report package_exports[0] unknown field unsigned_sidecar",
        )

    def test_report_stage_rejects_native_dynamic_package_export_field_types(
        self,
    ) -> None:
        cases = (
            ("package_id", 42, "must be a string"),
            ("directory", 42, "must be a string"),
            ("path", 42, "must be a string"),
            ("manifest", 42, "must be a string"),
            ("package_report", 42, "must be a string"),
        )
        for field, value, expected_type in cases:
            with self.subTest(field=field):
                self._assert_native_dynamic_report_field_diagnostic(
                    "package_exports",
                    [
                        {
                            "package_id": "animation",
                            "directory": "animation",
                            "path": "plugins/animation",
                            "manifest": "plugins/animation/plugin.toml",
                            "package_report": "plugins/animation/native_dynamic_package.toml",
                            field: value,
                        }
                    ],
                    f"native_dynamic report package_exports[0].{field} {expected_type}",
                )

    def test_report_stage_rejects_native_dynamic_package_export_missing_required_field(
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
                package_export = {
                    "package_id": "animation",
                    "directory": "animation",
                    "path": "plugins/animation",
                    "manifest": "plugins/animation/plugin.toml",
                    "package_report": "plugins/animation/native_dynamic_package.toml",
                    "abi": {"abi_version": 3},
                }
                package_export.pop(field)
                self._assert_native_dynamic_report_field_diagnostic(
                    "package_exports",
                    [package_export],
                    "native_dynamic report package_exports[0]."
                    f"{field} {expected_type}",
                )

    def test_report_stage_rejects_native_dynamic_package_export_abi_field_types(
        self,
    ) -> None:
        cases = (
            ("abi_version", "3", "must be an integer"),
            ("behavior_contract", 42, "must be a string"),
            ("bridge_method_table", 42, "must be a string"),
            ("descriptor_contract", 42, "must be a string"),
            ("descriptor_symbol", 42, "must be a string"),
            ("editor_entry_source", 42, "must be a string"),
            ("entry_report_contract", 42, "must be a string"),
            ("host_function_table", 42, "must be a string"),
            ("runtime_entry_source", 42, "must be a string"),
            ("state_snapshot_contract", 42, "must be a string"),
        )
        for field, value, expected_type in cases:
            with self.subTest(field=field):
                self._assert_native_dynamic_report_field_diagnostic(
                    "package_exports",
                    [
                        {
                            "package_id": "animation",
                            "directory": "animation",
                            "path": "plugins/animation",
                            "manifest": "plugins/animation/plugin.toml",
                            "package_report": "plugins/animation/native_dynamic_package.toml",
                            "abi": {
                                "abi_version": 3,
                                "descriptor_symbol": "zircon_plugin_descriptor_v3",
                                field: value,
                            },
                        }
                    ],
                    "native_dynamic report "
                    f"package_exports[0].abi.{field} {expected_type}",
                )

    def test_report_stage_rejects_native_dynamic_package_export_abi_missing_required_field(
        self,
    ) -> None:
        fields = (
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
        for field, expected_type in fields:
            with self.subTest(field=field):
                abi = {
                    "abi_version": 3,
                    "behavior_contract": "NativePluginBehaviorV3",
                    "bridge_method_table": "NativePluginBridgeMethodTableV3",
                    "descriptor_contract": "NativePluginAbiV3",
                    "descriptor_symbol": "zircon_native_plugin_descriptor_v3",
                    "editor_entry_source": "NativePluginAbiV3.editor_entry_name",
                    "entry_report_contract": "NativePluginEntryReportV3",
                    "host_function_table": "NativePluginHostFunctionTableV3",
                    "runtime_entry_source": "NativePluginAbiV3.runtime_entry_name",
                    "state_snapshot_contract": (
                        "NativePluginBehaviorV3.save_state/restore_state"
                    ),
                }
                abi.pop(field)
                self._assert_native_dynamic_report_field_diagnostic(
                    "package_exports",
                    [
                        {
                            "package_id": "animation",
                            "directory": "animation",
                            "path": "plugins/animation",
                            "manifest": "plugins/animation/plugin.toml",
                            "package_report": "plugins/animation/native_dynamic_package.toml",
                            "abi": abi,
                        }
                    ],
                    "native_dynamic report "
                    f"package_exports[0].abi.{field} {expected_type}",
                )

    def test_report_stage_rejects_native_dynamic_package_export_abi_unknown_field(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "package_exports",
            [
                {
                    "package_id": "animation",
                    "directory": "animation",
                    "path": "plugins/animation",
                    "manifest": "plugins/animation/plugin.toml",
                    "package_report": "plugins/animation/native_dynamic_package.toml",
                    "abi": {
                        "abi_version": 3,
                        "descriptor_symbol": "zircon_plugin_descriptor_v3",
                        "unsigned_sidecar": "plugins/animation/sidecar.bin",
                    },
                }
            ],
            "native_dynamic report package_exports[0].abi unknown field unsigned_sidecar",
        )

    def test_report_stage_rejects_native_dynamic_file_manifest_field_types(
        self,
    ) -> None:
        cases = (
            ("path", 42, "must be a string"),
            ("bytes", "1", "must be an integer"),
            ("sha256", 42, "must be a string"),
        )
        for field, value, expected_type in cases:
            with self.subTest(field=field, value=value):
                self._assert_native_dynamic_report_field_diagnostic(
                    "file_manifest",
                    [
                        {
                            "path": "plugins/animation/plugin.toml",
                            "bytes": 1,
                            "sha256": "hash",
                            field: value,
                        }
                    ],
                    f"native_dynamic report file_manifest[0].{field} {expected_type}",
                )

    def test_report_stage_rejects_native_dynamic_materialized_package_field_types(
        self,
    ) -> None:
        cases = (
            ("package_id", 42, "must be a string"),
            ("destination", 42, "must be a string"),
            ("package_report", 42, "must be a string"),
            ("source", 42, "must be a string"),
            ("loadable_artifact_count", "1", "must be an integer"),
            ("loadable_artifacts", [42], "must be a string array"),
        )
        for field, value, expected_type in cases:
            with self.subTest(field=field, value=value):
                self._assert_native_dynamic_report_field_diagnostic(
                    "materialized_packages",
                    [
                        {
                            "package_id": "animation",
                            "destination": "plugins/animation",
                            "package_report": "plugins/animation/native_dynamic_package.toml",
                            "source": "fixtures/native_dynamic/animation",
                            "loadable_artifact_count": 1,
                            "loadable_artifacts": [
                                "plugins/animation/native/zircon_plugin_animation.dll"
                            ],
                            field: value,
                        }
                    ],
                    "native_dynamic report "
                    f"materialized_packages[0].{field} {expected_type}",
                )

def _native_build_plan(**overrides: object) -> dict[str, object]:
    plan = {
        "fatal": False,
        "diagnostics": [],
        "workspace_manifest": "zircon_plugins/Cargo.toml",
        "target_dir": "target/native_dynamic",
        "cargo_profile": "release",
        "release": True,
        "build_features": ["v3_fixture_diagnostics"],
        "package_count": 1,
        "packages": [_native_build_plan_package()],
    }
    plan.update(overrides)
    return plan


def _native_build_plan_without(field: str) -> dict[str, object]:
    plan = _native_build_plan()
    plan.pop(field, None)
    return plan


def _native_build_plan_package(**overrides: object) -> dict[str, object]:
    package = {
        "package_id": "animation",
        "crate_name": "zircon_plugin_animation_native",
        "manifest_path": "zircon_plugins/animation/native/Cargo.toml",
        "workspace_manifest": "zircon_plugins/Cargo.toml",
        "target_dir": "target/native_dynamic",
        "cargo_profile": "release",
        "release": True,
        "features": ["v3_fixture_diagnostics"],
        "command": [
            "cargo",
            "build",
            "--manifest-path",
            "zircon_plugins/Cargo.toml",
        ],
        "expected_loadable_artifact": (
            "target/native_dynamic/release/zircon_plugin_animation_native.dll"
        ),
    }
    package.update(overrides)
    return package


def _native_operation_audit(**overrides: object) -> dict[str, object]:
    audit = {
        "enabled": True,
        "profile": "windows-store",
        "target_platform": "windows-x86_64",
        "allowed_platforms": ["windows"],
        "platform_allowed": True,
        "fatal": False,
        "package_count": 1,
        "diagnostics": [],
        "packages": [_native_operation_audit_package()],
    }
    audit.update(overrides)
    return audit


def _native_operation_audit_without(field: str) -> dict[str, object]:
    audit = _native_operation_audit()
    audit.pop(field, None)
    return audit


def _disabled_native_operation_audit_without(field: str) -> dict[str, object]:
    audit = _native_operation_audit(
        enabled=False,
        profile=None,
        allowed_platforms=[],
        package_count=0,
        packages=[],
    )
    audit.pop(field, None)
    return audit


def _native_operation_audit_package(**overrides: object) -> dict[str, object]:
    package = {
        "package_id": "animation",
        "artifact_count": 1,
        "artifacts": [_native_operation_audit_artifact()],
    }
    package.update(overrides)
    return package


def _native_operation_audit_artifact(**overrides: object) -> dict[str, object]:
    artifact = {
        "artifact": "E:/tmp/out/stages/native_dynamic/plugins/animation/native/plugin.dll",
        "package_relative_artifact": "native/plugin.dll",
        "command": ["signtool", "sign", "native/plugin.dll"],
        "exit_code": 0,
        "stdout": "",
        "stderr": "",
        "before_sha256": "before-hash",
        "after_sha256": "after-hash",
    }
    artifact.update(overrides)
    return artifact


if __name__ == "__main__":
    unittest.main()


def _native_build_execution(**overrides: object) -> dict[str, object]:
    execution = {
        "enabled": True,
        "fatal": False,
        "diagnostics": [],
        "package_count": 1,
        "packages": [_native_build_execution_package()],
    }
    execution.update(overrides)
    return execution


def _native_build_execution_without(field: str) -> dict[str, object]:
    execution = _native_build_execution()
    execution.pop(field, None)
    return execution


def _native_build_execution_package(**overrides: object) -> dict[str, object]:
    package = {
        "package_id": "animation",
        "crate_name": "zircon_plugin_animation_native",
        "command": [
            "cargo",
            "build",
            "--manifest-path",
            "zircon_plugins/Cargo.toml",
        ],
        "exit_code": 0,
        "stdout": "",
        "stderr": "",
        "expected_loadable_artifact": (
            "target/native_dynamic/release/zircon_plugin_animation_native.dll"
        ),
        "copied_loadable_artifact": "plugins/animation/native/plugin.dll",
        "copied_sidecars": ["plugins/animation/native/plugin.pdb"],
    }
    package.update(overrides)
    return package
