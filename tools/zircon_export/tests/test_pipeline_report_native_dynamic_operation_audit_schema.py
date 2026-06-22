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


class PipelineReportNativeDynamicOperationAuditSchemaTests(unittest.TestCase):
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

    def test_report_stage_rejects_native_dynamic_operation_audit_artifact_missing_execution_evidence_field(
        self,
    ) -> None:
        cases = (
            ("exit_code", "must be an integer"),
            ("before_sha256", "must be a string"),
            ("after_sha256", "must be a string"),
        )
        for field, expected_type in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    native_report_path = self._write_native_dynamic_reports(out)
                    native_report = json.loads(
                        native_report_path.read_text(encoding="utf-8")
                    )
                    native_report["native_signing"] = _native_operation_audit(
                        packages=[
                            _native_operation_audit_package(
                                artifacts=[
                                    _native_operation_audit_artifact_without(field)
                                ]
                            )
                        ]
                    )
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
                            "native_dynamic report native_signing packages[0] "
                            f"artifacts[0].{field} {expected_type}" in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )
                    self.assertFalse(
                        any(
                            "NativeDynamic report native_signing is malformed"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_native_dynamic_operation_audit_artifact_nonzero_exit_code(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = self._write_native_dynamic_reports(out)
            native_report = json.loads(
                native_report_path.read_text(encoding="utf-8")
            )
            native_report["native_signing"] = _native_operation_audit(
                packages=[
                    _native_operation_audit_package(
                        artifacts=[
                            _native_operation_audit_artifact(exit_code=1)
                        ]
                    )
                ]
            )
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
                    "native_dynamic report native_signing packages[0] "
                    "artifacts[0].exit_code must be 0 for non-fatal "
                    "operation audit"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_operation_audit_blank_diagnostic_entry(
        self,
    ) -> None:
        cases = (
            ("native_signing", [""]),
            ("native_notarization", ["   "]),
        )
        for operation, diagnostics in cases:
            with self.subTest(operation=operation, diagnostics=diagnostics):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    native_report_path = self._write_native_dynamic_reports(out)
                    native_report = json.loads(
                        native_report_path.read_text(encoding="utf-8")
                    )
                    native_report[operation] = _native_operation_audit(
                        diagnostics=diagnostics
                    )
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
                            "native_dynamic report "
                            f"{operation}.diagnostics must not contain blank entries"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )
                    self.assertFalse(
                        any(
                            f"NativeDynamic report {operation} is malformed"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_native_dynamic_operation_audit_padded_diagnostic_entry(
        self,
    ) -> None:
        cases = ("native_signing", "native_notarization")
        for operation in cases:
            with self.subTest(operation=operation):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    native_report_path = self._write_native_dynamic_reports(out)
                    native_report = json.loads(
                        native_report_path.read_text(encoding="utf-8")
                    )
                    native_report[operation] = _native_operation_audit(
                        diagnostics=[" signing warning "]
                    )
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
                            "native_dynamic report "
                            f"{operation}.diagnostics[0] "
                            "must be a non-empty trimmed string"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )
                    self.assertFalse(
                        any(
                            f"NativeDynamic report {operation} is malformed"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_native_dynamic_operation_audit_non_string_diagnostic_entry_before_array_shape(
        self,
    ) -> None:
        cases = ("native_signing", "native_notarization")
        for operation in cases:
            with self.subTest(operation=operation):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    native_report_path = self._write_native_dynamic_reports(out)
                    native_report = json.loads(
                        native_report_path.read_text(encoding="utf-8")
                    )
                    native_report[operation] = _native_operation_audit(
                        diagnostics=["signing warning", 42]
                    )
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
                            "native_dynamic report "
                            f"{operation}.diagnostics[1] "
                            "must be a string"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )
                    self.assertFalse(
                        any(
                            "native_dynamic report "
                            f"{operation}.diagnostics "
                            "must be a string array"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_native_dynamic_operation_audit_command_non_string_entry_before_array_shape(
        self,
    ) -> None:
        cases = ("native_signing", "native_notarization")
        for operation in cases:
            with self.subTest(operation=operation):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    native_report_path = self._write_native_dynamic_reports(out)
                    native_report = json.loads(
                        native_report_path.read_text(encoding="utf-8")
                    )
                    native_report[operation] = _native_operation_audit(
                        packages=[
                            _native_operation_audit_package(
                                artifacts=[
                                    _native_operation_audit_artifact(
                                        command=["signtool", 42],
                                    )
                                ]
                            )
                        ]
                    )
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
                            "native_dynamic report "
                            f"{operation} packages[0] "
                            "artifacts[0].command[1] must be a string"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )
                    self.assertFalse(
                        any(
                            "native_dynamic report "
                            f"{operation} packages[0] "
                            "artifacts[0].command must be a string array"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )
                    self.assertFalse(
                        any(
                            f"NativeDynamic report {operation} is malformed"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_native_dynamic_operation_audit_negative_counts(
        self,
    ) -> None:
        cases = (
            (
                "package_count",
                lambda audit: audit.__setitem__("package_count", -1),
                "native_dynamic report native_signing.package_count must be non-negative",
            ),
            (
                "artifact_count",
                lambda audit: audit["packages"][0].__setitem__(
                    "artifact_count",
                    -1,
                ),
                "native_dynamic report native_signing packages[0].artifact_count "
                "must be non-negative",
            ),
        )
        for field, mutate, expected_diagnostic in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    native_report_path = self._write_native_dynamic_reports(out)
                    native_report = json.loads(
                        native_report_path.read_text(encoding="utf-8")
                    )
                    audit = _native_operation_audit()
                    mutate(audit)
                    native_report["native_signing"] = audit
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
                    if field == "package_relative_artifact":
                        self.assertFalse(
                            any(
                                "native_dynamic report native_signing packages[0] "
                                "artifacts[0].package_relative_artifact "
                                "must be a safe relative path"
                                in diagnostic
                                for diagnostic in report["diagnostics"]
                            ),
                            report["diagnostics"],
                        )

    def test_report_stage_rejects_native_dynamic_operation_audit_unsafe_relative_artifact(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = self._write_native_dynamic_reports(out)
            native_report = json.loads(
                native_report_path.read_text(encoding="utf-8")
            )
            native_report["native_signing"] = _native_operation_audit(
                packages=[
                    _native_operation_audit_package(
                        artifacts=[
                            _native_operation_audit_artifact(
                                package_relative_artifact=(
                                    "../zircon_plugin_animation.dll"
                                )
                            )
                        ]
                    )
                ]
            )
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
                    "native_dynamic report native_signing packages[0] "
                    "artifacts[0].package_relative_artifact "
                    "must be a safe relative path"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_operation_audit_artifact_path_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = self._write_native_dynamic_reports(out)
            native_report = json.loads(
                native_report_path.read_text(encoding="utf-8")
            )
            native_report["native_signing"] = _native_operation_audit(
                packages=[
                    _native_operation_audit_package(
                        artifacts=[
                            _native_operation_audit_artifact(
                                artifact="plugins/animation/native/forged.dll"
                            )
                        ]
                    )
                ]
            )
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
                    "native_dynamic report native_signing package animation "
                    "artifacts[0].artifact "
                    "plugins/animation/native/forged.dll does not match "
                    "package_relative_artifact "
                    "plugins/animation/native/zircon_plugin_animation.dll"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_operation_audit_duplicate_package_relative_artifact(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = self._write_native_dynamic_reports(out)
            native_report = json.loads(
                native_report_path.read_text(encoding="utf-8")
            )
            native_report["native_signing"] = _native_operation_audit(
                packages=[
                    _native_operation_audit_package(
                        artifact_count=2,
                        artifacts=[
                            _native_operation_audit_artifact(),
                            _native_operation_audit_artifact(),
                        ],
                    )
                ]
            )
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
                    "native_dynamic report native_signing packages[0] "
                    "artifacts.package_relative_artifact "
                    "must not contain duplicate entries"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_operation_audit_padded_duplicate_package_relative_artifact_before_uniqueness(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = self._write_native_dynamic_reports(out)
            native_report = json.loads(
                native_report_path.read_text(encoding="utf-8")
            )
            native_report["native_signing"] = _native_operation_audit(
                packages=[
                    _native_operation_audit_package(
                        artifact_count=2,
                        artifacts=[
                            _native_operation_audit_artifact(
                                package_relative_artifact=(
                                    " native/zircon_plugin_animation.dll "
                                )
                            ),
                            _native_operation_audit_artifact(
                                package_relative_artifact=(
                                    " native/zircon_plugin_animation.dll "
                                )
                            ),
                        ],
                    )
                ]
            )
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
                    "native_dynamic report native_signing packages[0] "
                    "artifacts[0].package_relative_artifact "
                    "must be a non-empty trimmed string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(
                any(
                    "native_dynamic report native_signing packages[0] "
                    "artifacts.package_relative_artifact "
                    "must not contain duplicate entries"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_operation_audit_empty_required_identity_string(
        self,
    ) -> None:
        cases = (
            (
                "package_id",
                lambda: _native_operation_audit(
                    packages=[
                        _native_operation_audit_package(package_id=""),
                    ]
                ),
                "native_dynamic report native_signing packages[0].package_id "
                "must be a non-empty string",
            ),
            (
                "artifact",
                lambda: _native_operation_audit(
                    packages=[
                        _native_operation_audit_package(
                            artifacts=[
                                _native_operation_audit_artifact(artifact="   ")
                            ]
                        )
                    ]
                ),
                "native_dynamic report native_signing packages[0] "
                "artifacts[0].artifact must be a non-empty string",
            ),
            (
                "package_relative_artifact",
                lambda: _native_operation_audit(
                    packages=[
                        _native_operation_audit_package(
                            artifacts=[
                                _native_operation_audit_artifact(
                                    package_relative_artifact=""
                                )
                            ]
                        )
                    ]
                ),
                "native_dynamic report native_signing packages[0] "
                "artifacts[0].package_relative_artifact "
                "must be a non-empty string",
            ),
            (
                "before_sha256",
                lambda: _native_operation_audit(
                    packages=[
                        _native_operation_audit_package(
                            artifacts=[
                                _native_operation_audit_artifact(
                                    before_sha256="   "
                                )
                            ]
                        )
                    ]
                ),
                "native_dynamic report native_signing packages[0] "
                "artifacts[0].before_sha256 must be a non-empty string",
            ),
            (
                "after_sha256",
                lambda: _native_operation_audit(
                    packages=[
                        _native_operation_audit_package(
                            artifacts=[
                                _native_operation_audit_artifact(after_sha256="")
                            ]
                        )
                    ]
                ),
                "native_dynamic report native_signing packages[0] "
                "artifacts[0].after_sha256 must be a non-empty string",
            ),
        )
        for field, make_audit, expected_diagnostic in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    native_report_path = self._write_native_dynamic_reports(out)
                    native_report = json.loads(
                        native_report_path.read_text(encoding="utf-8")
                    )
                    native_report["native_signing"] = make_audit()
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

    def test_report_stage_rejects_native_dynamic_operation_audit_padded_required_identity_string(
        self,
    ) -> None:
        cases = (
            (
                "package_id",
                lambda: _native_operation_audit(
                    packages=[
                        _native_operation_audit_package(
                            package_id=" animation ",
                        ),
                    ]
                ),
                "native_dynamic report native_signing packages[0].package_id "
                "must be a non-empty trimmed string",
            ),
            (
                "artifact",
                lambda: _native_operation_audit(
                    packages=[
                        _native_operation_audit_package(
                            artifacts=[
                                _native_operation_audit_artifact(
                                    artifact=(
                                        " plugins/animation/native/"
                                        "zircon_plugin_animation.dll "
                                    )
                                )
                            ]
                        )
                    ]
                ),
                "native_dynamic report native_signing packages[0] "
                "artifacts[0].artifact must be a non-empty trimmed string",
            ),
            (
                "package_relative_artifact",
                lambda: _native_operation_audit(
                    packages=[
                        _native_operation_audit_package(
                            artifacts=[
                                _native_operation_audit_artifact(
                                    package_relative_artifact=(
                                        " native/zircon_plugin_animation.dll "
                                    )
                                )
                            ]
                        )
                    ]
                ),
                "native_dynamic report native_signing packages[0] "
                "artifacts[0].package_relative_artifact "
                "must be a non-empty trimmed string",
            ),
            (
                "before_sha256",
                lambda: _native_operation_audit(
                    packages=[
                        _native_operation_audit_package(
                            artifacts=[
                                _native_operation_audit_artifact(
                                    before_sha256=(
                                        " 0000000000000000000000000000000000000000"
                                        "000000000000000000000000 "
                                    )
                                )
                            ]
                        )
                    ]
                ),
                "native_dynamic report native_signing packages[0] "
                "artifacts[0].before_sha256 "
                "must be a non-empty trimmed string",
            ),
            (
                "after_sha256",
                lambda: _native_operation_audit(
                    packages=[
                        _native_operation_audit_package(
                            artifacts=[
                                _native_operation_audit_artifact(
                                    after_sha256=(
                                        " 1111111111111111111111111111111111111111"
                                        "111111111111111111111111 "
                                    )
                                )
                            ]
                        )
                    ]
                ),
                "native_dynamic report native_signing packages[0] "
                "artifacts[0].after_sha256 "
                "must be a non-empty trimmed string",
            ),
        )
        for field, make_audit, expected_diagnostic in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    native_report_path = self._write_native_dynamic_reports(out)
                    native_report = json.loads(
                        native_report_path.read_text(encoding="utf-8")
                    )
                    native_report["native_signing"] = make_audit()
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

    def test_report_stage_rejects_native_dynamic_operation_audit_invalid_hash_string(
        self,
    ) -> None:
        cases = (
            (
                "before_sha256",
                _native_operation_audit_artifact(before_sha256="not-a-sha256"),
                "native_dynamic report native_signing packages[0] "
                "artifacts[0].before_sha256 must be a SHA-256 hex digest",
            ),
            (
                "after_sha256",
                _native_operation_audit_artifact(after_sha256="not-a-sha256"),
                "native_dynamic report native_signing packages[0] "
                "artifacts[0].after_sha256 must be a SHA-256 hex digest",
            ),
        )
        for field, artifact, expected_diagnostic in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    native_report_path = self._write_native_dynamic_reports(out)
                    native_report = json.loads(
                        native_report_path.read_text(encoding="utf-8")
                    )
                    native_report["native_signing"] = _native_operation_audit(
                        packages=[
                            _native_operation_audit_package(artifacts=[artifact])
                        ]
                    )
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

    def test_report_stage_rejects_native_dynamic_operation_audit_blank_target_platform(
        self,
    ) -> None:
        cases = (
            ("native_signing", ""),
            ("native_notarization", "   "),
        )
        for operation, target_platform in cases:
            with self.subTest(operation=operation, target_platform=target_platform):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    native_report_path = self._write_native_dynamic_reports(out)
                    native_report = json.loads(
                        native_report_path.read_text(encoding="utf-8")
                    )
                    native_report[operation] = _native_operation_audit(
                        target_platform=target_platform
                    )
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
                            "native_dynamic report "
                            f"{operation}.target_platform "
                            "must be a non-empty string"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_native_dynamic_operation_audit_blank_profile(
        self,
    ) -> None:
        cases = (
            ("native_signing", ""),
            ("native_notarization", "   "),
        )
        for operation, profile in cases:
            with self.subTest(operation=operation, profile=profile):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    native_report_path = self._write_native_dynamic_reports(out)
                    native_report = json.loads(
                        native_report_path.read_text(encoding="utf-8")
                    )
                    native_report[operation] = _native_operation_audit(
                        profile=profile
                    )
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
                            "native_dynamic report "
                            f"{operation}.profile "
                            "must be a non-empty string"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_native_dynamic_operation_audit_padded_summary_string(
        self,
    ) -> None:
        cases = (
            (
                "target_platform",
                "native_signing",
                {"target_platform": " windows-x86_64 "},
                "native_dynamic report native_signing.target_platform "
                "must be a non-empty trimmed string",
            ),
            (
                "profile",
                "native_notarization",
                {"profile": " windows-store "},
                "native_dynamic report native_notarization.profile "
                "must be a non-empty trimmed string",
            ),
        )
        for field, operation, overrides, expected_diagnostic in cases:
            with self.subTest(field=field, operation=operation):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    native_report_path = self._write_native_dynamic_reports(out)
                    native_report = json.loads(
                        native_report_path.read_text(encoding="utf-8")
                    )
                    native_report[operation] = _native_operation_audit(
                        **overrides
                    )
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
                    if field == "target_platform":
                        self.assertFalse(
                            any(
                                "native_dynamic report "
                                f"{operation}.platform_allowed does not match "
                                "target_platform and allowed_platforms"
                                in diagnostic
                                for diagnostic in report["diagnostics"]
                            ),
                            report["diagnostics"],
                        )

    def test_report_stage_rejects_native_dynamic_operation_audit_duplicate_allowed_platform(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = self._write_native_dynamic_reports(out)
            native_report = json.loads(
                native_report_path.read_text(encoding="utf-8")
            )
            native_report["native_signing"] = _native_operation_audit(
                allowed_platforms=["windows", "windows"]
            )
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
                    "native_dynamic report native_signing.allowed_platforms "
                    "must not contain duplicate entries"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_operation_audit_padded_duplicate_allowed_platform_before_uniqueness(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = self._write_native_dynamic_reports(out)
            native_report = json.loads(
                native_report_path.read_text(encoding="utf-8")
            )
            native_report["native_signing"] = _native_operation_audit(
                allowed_platforms=[" windows ", " windows "]
            )
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
                    "native_dynamic report native_signing.allowed_platforms[0] "
                    "must be a non-empty trimmed string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(
                any(
                    "native_dynamic report native_signing.allowed_platforms "
                    "must not contain duplicate entries"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_operation_audit_padded_allowed_platform_entry(
        self,
    ) -> None:
        cases = ("native_signing", "native_notarization")
        for operation in cases:
            with self.subTest(operation=operation):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    native_report_path = self._write_native_dynamic_reports(out)
                    native_report = json.loads(
                        native_report_path.read_text(encoding="utf-8")
                    )
                    native_report[operation] = _native_operation_audit(
                        allowed_platforms=[" windows "],
                    )
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
                            "native_dynamic report "
                            f"{operation}.allowed_platforms[0] "
                            "must be a non-empty trimmed string"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )
                    self.assertFalse(
                        any(
                            "native_dynamic report "
                            f"{operation}.platform_allowed does not match "
                            "target_platform and allowed_platforms"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_native_dynamic_operation_audit_non_string_allowed_platform_entry_before_array_shape(
        self,
    ) -> None:
        cases = ("native_signing", "native_notarization")
        for operation in cases:
            with self.subTest(operation=operation):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    native_report_path = self._write_native_dynamic_reports(out)
                    native_report = json.loads(
                        native_report_path.read_text(encoding="utf-8")
                    )
                    native_report[operation] = _native_operation_audit(
                        allowed_platforms=["windows", 42],
                    )
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
                            "native_dynamic report "
                            f"{operation}.allowed_platforms[1] "
                            "must be a string"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )
                    self.assertFalse(
                        any(
                            "native_dynamic report "
                            f"{operation}.allowed_platforms "
                            "must be a string array"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )
                    self.assertFalse(
                        any(
                            "native_dynamic report "
                            f"{operation}.platform_allowed does not match "
                            "target_platform and allowed_platforms"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )

    def test_report_stage_rejects_native_dynamic_operation_audit_platform_allowed_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = self._write_native_dynamic_reports(out)
            native_report = json.loads(
                native_report_path.read_text(encoding="utf-8")
            )
            native_report["native_signing"] = _native_operation_audit(
                allowed_platforms=["macos"],
                platform_allowed=True,
            )
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
                    "native_dynamic report native_signing.platform_allowed "
                    "does not match target_platform and allowed_platforms"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
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
        "artifact": "plugins/animation/native/zircon_plugin_animation.dll",
        "package_relative_artifact": "native/zircon_plugin_animation.dll",
        "command": ["signtool", "sign", "native/zircon_plugin_animation.dll"],
        "exit_code": 0,
        "stdout": "",
        "stderr": "",
        "before_sha256": "0000000000000000000000000000000000000000000000000000000000000000",
        "after_sha256": "1111111111111111111111111111111111111111111111111111111111111111",
    }
    artifact.update(overrides)
    return artifact


def _native_operation_audit_artifact_without(field: str) -> dict[str, object]:
    artifact = _native_operation_audit_artifact()
    artifact.pop(field, None)
    return artifact


if __name__ == "__main__":
    unittest.main()
