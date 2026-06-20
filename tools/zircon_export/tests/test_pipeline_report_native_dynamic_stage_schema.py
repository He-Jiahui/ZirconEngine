from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.native_dynamic_contract import (
    NATIVE_DYNAMIC_ABI_STRING_FIELDS,
    NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS,
)
from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.native_dynamic_stage_schema_test_support import (
    NativeDynamicStageSchemaReportAssertions,
)


class PipelineReportNativeDynamicStageSchemaTests(
    NativeDynamicStageSchemaReportAssertions,
    unittest.TestCase,
):
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

    def test_report_stage_rejects_native_dynamic_string_array_fields_blank_entry(
        self,
    ) -> None:
        cases = (
            ("artifact_extensions", [""], "artifact_extensions"),
            ("artifact_extensions", ["   "], "artifact_extensions"),
            (
                "artifact_extensions",
                [".dll", ""],
                "artifact_extensions",
            ),
            ("native_dynamic_packages", [""], "native_dynamic_packages"),
            ("native_dynamic_packages", ["   "], "native_dynamic_packages"),
            (
                "native_dynamic_packages",
                ["animation", ""],
                "native_dynamic_packages",
            ),
        )
        for field, value, expected_field in cases:
            with self.subTest(field=field, value=value):
                self._assert_native_dynamic_report_field_diagnostic(
                    field,
                    value,
                    f"native_dynamic report {expected_field} "
                    "must not contain blank entries",
                )

    def test_report_stage_rejects_native_dynamic_count_fields_non_integer(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "package_count",
            "1",
            "native_dynamic report package_count must be an integer",
        )

    def test_report_stage_rejects_native_dynamic_negative_count_fields(
        self,
    ) -> None:
        self._assert_native_dynamic_report_field_diagnostic(
            "package_count",
            -1,
            "native_dynamic report package_count must be non-negative",
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

    def test_report_stage_rejects_native_dynamic_empty_required_string_release_evidence_field(
        self,
    ) -> None:
        for field in (
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
                    "",
                    f"native_dynamic report {field} must be a non-empty string",
                )

    def test_report_stage_rejects_native_dynamic_blank_required_string_release_evidence_field(
        self,
    ) -> None:
        for field in (
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
                    "   ",
                    f"native_dynamic report {field} must be a non-empty string",
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

    def test_report_stage_rejects_native_dynamic_operation_audit_blank_allowed_platform_entry(
        self,
    ) -> None:
        for allowed_platforms in ([""], ["   "], ["windows", ""]):
            with self.subTest(allowed_platforms=allowed_platforms):
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_signing",
                    _native_operation_audit(
                        allowed_platforms=allowed_platforms
                    ),
                    "native_dynamic report native_signing.allowed_platforms "
                    "must not contain blank entries",
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

    def test_report_stage_rejects_native_dynamic_operation_audit_artifact_empty_command(
        self,
    ) -> None:
        for command in ([], ["signtool", ""], ["signtool", "   "]):
            with self.subTest(command=command):
                artifact = _native_operation_audit_artifact(command=command)
                package = _native_operation_audit_package(
                    artifacts=[artifact]
                )
                audit = _native_operation_audit(packages=[package])
                self._assert_native_dynamic_report_field_diagnostic(
                    "native_signing",
                    audit,
                    "native_dynamic report native_signing packages[0] "
                    "artifacts[0].command must be a non-empty string array",
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

    def test_report_stage_rejects_native_dynamic_package_export_abi_shape_before_contract(
        self,
    ) -> None:
        cases = (
            (
                "abi_version",
                "3",
                "native_dynamic report package_exports[0].abi.abi_version "
                "must be an integer",
                (
                    "native_dynamic report package_exports[0].abi.abi_version "
                    "must be 3",
                    "native_dynamic loader_manifest plugin animation "
                    "abi.abi_version must be a string",
                ),
            ),
            *(
                (
                    field,
                    "   ",
                    f"native_dynamic report package_exports[0].abi.{field} "
                    "must be a non-empty trimmed string",
                    (
                        f"native_dynamic report package_exports[0].abi.{field} "
                        f"must be {NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS[field]}",
                        "native_dynamic loader_manifest plugin animation "
                        f"abi.{field}",
                    ),
                )
                for field in NATIVE_DYNAMIC_ABI_STRING_FIELDS
            ),
        )
        for field, value, expected_diagnostic, unexpected_diagnostics in cases:
            with self.subTest(field=field):
                abi = {
                    "abi_version": 3,
                    **NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS,
                    field: value,
                }
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    native_report_path = self._write_native_dynamic_reports(out)
                    native_report = json.loads(
                        native_report_path.read_text(encoding="utf-8")
                    )
                    native_report["package_exports"] = [
                        {
                            "package_id": "animation",
                            "directory": "animation",
                            "path": "plugins/animation",
                            "manifest": "plugins/animation/plugin.toml",
                            "package_report": (
                                "plugins/animation/native_dynamic_package.toml"
                            ),
                            "abi": abi,
                        }
                    ]
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
                    for unexpected_diagnostic in unexpected_diagnostics:
                        self.assertFalse(
                            any(
                                unexpected_diagnostic in diagnostic
                                for diagnostic in report["diagnostics"]
                            ),
                            report["diagnostics"],
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
        "before_sha256": "0000000000000000000000000000000000000000000000000000000000000000",
        "after_sha256": "1111111111111111111111111111111111111111111111111111111111111111",
    }
    artifact.update(overrides)
    return artifact


if __name__ == "__main__":
    unittest.main()
