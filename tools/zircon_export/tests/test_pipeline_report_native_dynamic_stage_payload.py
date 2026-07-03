from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.native_dynamic_stage_report_test_support import (
    write_native_dynamic_reports,
)


class PipelineReportNativeDynamicStagePayloadTests(unittest.TestCase):
    def test_report_stage_rejects_native_dynamic_schema_before_payload_semantics(
        self,
    ) -> None:
        cases = (
            (
                "materialized_package_id",
                lambda native_report: native_report["materialized_packages"][
                    0
                ].update({"package_id": " animation "}),
                (
                    "native_dynamic report materialized_packages[0].package_id "
                    "must be a non-empty trimmed string"
                ),
                (
                    "native_dynamic report materialized package ids",
                    "native_dynamic loader_manifest plugin ids",
                    "native_dynamic report package_exports package ids",
                    "native_dynamic report native_build_plan package ids",
                ),
            ),
            (
                "file_manifest_path",
                lambda native_report: native_report["file_manifest"][0].update(
                    {"path": f' {native_report["file_manifest"][0]["path"]} '}
                ),
                (
                    "native_dynamic report file_manifest[0].path "
                    "must be a non-empty trimmed string"
                ),
                (
                    "native_dynamic report file_manifest does not match "
                    "current NativeDynamic plugins directory",
                    "native_dynamic report loadable_artifacts are not present "
                    "in current NativeDynamic plugins directory",
                ),
            ),
            (
                "native_plugin_root",
                lambda native_report: native_report.update(
                    {
                        "native_plugin_root": (
                            f' {native_report["native_plugin_root"]} '
                        )
                    }
                ),
                (
                    "native_dynamic report native_plugin_root "
                    "must be a non-empty trimmed string"
                ),
                (
                    "could not be resolved",
                    "is outside native_plugin_root",
                ),
            ),
            (
                "package_count",
                lambda native_report: native_report.update({"package_count": -1}),
                "native_dynamic report package_count must be non-negative",
                (
                    "native_dynamic report package_count -1 does not match "
                    "materialized_packages",
                ),
            ),
            (
                "selected_package_id",
                lambda native_report: native_report.update(
                    {"native_dynamic_packages": [" animation "]}
                ),
                (
                    "native_dynamic report native_dynamic_packages[0] "
                    "must be a non-empty trimmed string"
                ),
                (
                    "native_dynamic report native_dynamic_packages "
                    "[' animation '] does not match",
                ),
            ),
            (
                "build_execution_package_count",
                lambda native_report: native_report[
                    "native_build_execution"
                ].update({"package_count": -1}),
                (
                    "native_dynamic report native_build_execution.package_count "
                    "must be non-negative"
                ),
                (
                    "native_dynamic report native_build_execution.package_count "
                    "-1 does not match",
                    "native_dynamic report native_build_execution package ids",
                ),
            ),
            (
                "operation_audit_artifact_count",
                lambda native_report: native_report["native_signing"].update(
                    {
                        "enabled": True,
                        "package_count": 1,
                        "packages": [
                            {
                                "package_id": "animation",
                                "artifact_count": -1,
                                "artifacts": [],
                            }
                        ],
                    }
                ),
                (
                    "native_dynamic report native_signing "
                    "packages[0].artifact_count must be non-negative"
                ),
                (
                    "native_dynamic report native_signing package animation "
                    "artifact_count -1 does not match",
                    "native_dynamic report native_signing package animation "
                    "package_relative_artifacts [] do not match",
                ),
            ),
            (
                "operation_audit_package_id",
                lambda native_report: native_report["native_signing"].update(
                    {
                        "enabled": True,
                        "package_count": 1,
                        "packages": [
                            {
                                "package_id": " animation ",
                                "artifact_count": 0,
                                "artifacts": [],
                            }
                        ],
                    }
                ),
                (
                    "native_dynamic report native_signing "
                    "packages[0].package_id must be a non-empty trimmed string"
                ),
                (
                    "native_dynamic report native_signing package ids "
                    "[' animation '] do not match",
                ),
            ),
            (
                "operation_audit_artifact_path",
                lambda native_report: native_report["native_signing"].update(
                    {
                        "enabled": True,
                        "package_count": 1,
                        "packages": [
                            {
                                "package_id": "animation",
                                "artifact_count": 1,
                                "artifacts": [
                                    {
                                        "artifact": (
                                            " plugins/animation/native/"
                                            "zircon_plugin_animation.dll "
                                        ),
                                        "package_relative_artifact": (
                                            "native/zircon_plugin_animation.dll"
                                        ),
                                        "command": ["sign"],
                                        "exit_code": 0,
                                        "stdout": "",
                                        "stderr": "",
                                        "before_sha256": "0" * 64,
                                        "after_sha256": "1" * 64,
                                    }
                                ],
                            }
                        ],
                    }
                ),
                (
                    "native_dynamic report native_signing "
                    "packages[0] artifacts[0].artifact "
                    "must be a non-empty trimmed string"
                ),
                (
                    "native_dynamic report native_signing package animation "
                    "artifacts[0].artifact",
                ),
            ),
            (
                "operation_audit_package_relative_artifact",
                lambda native_report: native_report["native_signing"].update(
                    {
                        "enabled": True,
                        "package_count": 1,
                        "packages": [
                            {
                                "package_id": "animation",
                                "artifact_count": 1,
                                "artifacts": [
                                    {
                                        "artifact": (
                                            "plugins/animation/native/"
                                            "zircon_plugin_animation.dll"
                                        ),
                                        "package_relative_artifact": (
                                            " native/zircon_plugin_animation.dll "
                                        ),
                                        "command": ["sign"],
                                        "exit_code": 0,
                                        "stdout": "",
                                        "stderr": "",
                                        "before_sha256": "0" * 64,
                                        "after_sha256": "1" * 64,
                                    }
                                ],
                            }
                        ],
                    }
                ),
                (
                    "native_dynamic report native_signing "
                    "packages[0] artifacts[0].package_relative_artifact "
                    "must be a non-empty trimmed string"
                ),
                (
                    "native_dynamic report native_signing package animation "
                    "artifacts[0].artifact",
                    "native_dynamic report native_signing package animation "
                    "package_relative_artifacts",
                ),
            ),
        )
        for label, mutate, expected, unexpected in cases:
            with self.subTest(label=label):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    native_report_path = write_native_dynamic_reports(out)
                    native_report = json.loads(
                        native_report_path.read_text(encoding="utf-8")
                    )
                    mutate(native_report)
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
                            expected in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )
                    for unexpected_diagnostic in unexpected:
                        self.assertFalse(
                            any(
                                unexpected_diagnostic in diagnostic
                                for diagnostic in report["diagnostics"]
                            ),
                            report["diagnostics"],
                        )

    def test_report_stage_rejects_native_dynamic_build_execution_copied_artifact_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = write_native_dynamic_reports(out)
            native_report = json.loads(
                native_report_path.read_text(encoding="utf-8")
            )
            native_report["native_build_execution"] = {
                "enabled": True,
                "fatal": False,
                "diagnostics": [],
                "package_count": 1,
                "packages": [
                    {
                        "package_id": "animation",
                        "crate_name": "zircon_plugin_animation_native",
                        "command": ["cargo", "build"],
                        "exit_code": 0,
                        "stdout": "",
                        "stderr": "",
                        "expected_loadable_artifact": (
                            "target/native_dynamic/debug/"
                            "zircon_plugin_animation_native.dll"
                        ),
                        "copied_loadable_artifact": (
                            "plugins/animation/native/forged.dll"
                        ),
                        "copied_sidecars": [],
                    }
                ],
            }
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
                    "native_dynamic report native_build_execution package "
                    "animation copied_loadable_artifact "
                    "plugins/animation/native/forged.dll does not match "
                    "materialized loadable artifacts "
                    "['plugins/animation/native/zircon_plugin_animation.dll']"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_build_execution_copied_sidecar_missing(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = write_native_dynamic_reports(out)
            native_report = json.loads(
                native_report_path.read_text(encoding="utf-8")
            )
            native_report["native_build_execution"] = {
                "enabled": True,
                "fatal": False,
                "diagnostics": [],
                "package_count": 1,
                "packages": [
                    {
                        "package_id": "animation",
                        "crate_name": "zircon_plugin_animation_native",
                        "command": ["cargo", "build"],
                        "exit_code": 0,
                        "stdout": "",
                        "stderr": "",
                        "expected_loadable_artifact": (
                            "target/native_dynamic/debug/"
                            "zircon_plugin_animation_native.dll"
                        ),
                        "copied_loadable_artifact": (
                            "plugins/animation/native/"
                            "zircon_plugin_animation.dll"
                        ),
                        "copied_sidecars": [
                            "plugins/animation/native/forged.pdb"
                        ],
                    }
                ],
            }
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
                    "native_dynamic report native_build_execution package "
                    "animation copied_sidecars[0] "
                    "plugins/animation/native/forged.pdb is not present "
                    "in current NativeDynamic plugins file_manifest"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_build_execution_copied_sidecar_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = write_native_dynamic_reports(out)
            native_report = json.loads(
                native_report_path.read_text(encoding="utf-8")
            )
            native_report["native_build_execution"] = {
                "enabled": True,
                "fatal": False,
                "diagnostics": [],
                "package_count": 1,
                "packages": [
                    {
                        "package_id": "animation",
                        "crate_name": "zircon_plugin_animation_native",
                        "command": ["cargo", "build"],
                        "exit_code": 0,
                        "stdout": "",
                        "stderr": "",
                        "expected_loadable_artifact": (
                            "target/native_dynamic/debug/"
                            "zircon_plugin_animation_native.dll"
                        ),
                        "copied_loadable_artifact": (
                            "plugins/animation/native/"
                            "zircon_plugin_animation.dll"
                        ),
                        "copied_sidecars": [
                            "plugins/animation/native/forged.pdb"
                        ],
                    }
                ],
            }
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
                    "native_dynamic report native_build_execution package "
                    "animation copied_sidecars[0] "
                    "plugins/animation/native/forged.pdb is not present "
                    "in current NativeDynamic plugins file_manifest"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_build_execution_command_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = write_native_dynamic_reports(out)
            native_report = json.loads(
                native_report_path.read_text(encoding="utf-8")
            )
            native_report["native_build_execution"] = {
                "enabled": True,
                "fatal": False,
                "diagnostics": [],
                "package_count": 1,
                "packages": [
                    {
                        "package_id": "animation",
                        "crate_name": "zircon_plugin_animation_native",
                        "command": [
                            "cargo",
                            "build",
                            "--manifest-path",
                            "forged/Cargo.toml",
                        ],
                        "exit_code": 0,
                        "stdout": "",
                        "stderr": "",
                        "expected_loadable_artifact": (
                            "target/native_dynamic/debug/"
                            "zircon_plugin_animation_native.dll"
                        ),
                        "copied_loadable_artifact": (
                            "plugins/animation/native/"
                            "zircon_plugin_animation.dll"
                        ),
                        "copied_sidecars": [],
                    }
                ],
            }
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
                    "native_dynamic report native_build_execution package "
                    "animation command "
                    "['cargo', 'build', '--manifest-path', "
                    "'forged/Cargo.toml'] does not match "
                    "native_build_plan package command "
                    "['cargo', 'build', '--manifest-path', "
                    "'zircon_plugins/Cargo.toml', '-p', "
                    "'zircon_plugin_animation_native', '--target-dir', "
                    "'target/native_dynamic']"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
