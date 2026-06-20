from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.native_dynamic_stage_report_test_support import (
    loader_manifest_with_overrides,
    rewrite_loader_manifest,
    write_native_dynamic_reports,
)


class PipelineReportNativeDynamicStagePayloadTests(unittest.TestCase):
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

    def test_report_stage_rejects_native_dynamic_loader_manifest_path_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = write_native_dynamic_reports(out)
            rewrite_loader_manifest(
                native_report_path,
                loader_manifest_with_overrides(path="plugins/forged"),
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("NativeDynamic", report["fatal_stages"])
            self.assertTrue(
                any(
                    "native_dynamic loader_manifest plugin animation path "
                    "plugins/forged does not match native_dynamic report "
                    "package_exports path plugins/animation"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
    def test_report_stage_rejects_native_dynamic_loader_manifest_missing_row_field(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = write_native_dynamic_reports(out)
            rewrite_loader_manifest(
                native_report_path,
                loader_manifest_with_overrides().replace(
                    'path = "plugins/animation"\n',
                    "",
                ),
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("NativeDynamic", report["fatal_stages"])
            self.assertTrue(
                any(
                    "native_dynamic loader_manifest plugin animation path "
                    "is required by native_dynamic report package_exports"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
    def test_report_stage_rejects_native_dynamic_loader_manifest_missing_abi_table(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = write_native_dynamic_reports(out)
            rewrite_loader_manifest(
                native_report_path,
                "\n".join(
                    [
                        "[[plugins]]",
                        'id = "animation"',
                        'path = "plugins/animation"',
                        'manifest = "plugins/animation/plugin.toml"',
                        (
                            'package_report = "plugins/animation/'
                            'native_dynamic_package.toml"'
                        ),
                    ]
                )
                + "\n",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("NativeDynamic", report["fatal_stages"])
            self.assertTrue(
                any(
                    "native_dynamic loader_manifest plugin animation abi "
                    "is required by native_dynamic report package_exports"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
    def test_report_stage_rejects_native_dynamic_loader_manifest_abi_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = write_native_dynamic_reports(out)
            rewrite_loader_manifest(
                native_report_path,
                loader_manifest_with_overrides(
                    abi_overrides={
                        "descriptor_symbol": "zircon_native_plugin_descriptor_v2"
                    },
                ),
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("NativeDynamic", report["fatal_stages"])
            self.assertTrue(
                any(
                    "native_dynamic loader_manifest plugin animation "
                    "abi.descriptor_symbol zircon_native_plugin_descriptor_v2 "
                    "does not match native_dynamic report package_exports "
                    "abi.descriptor_symbol zircon_native_plugin_descriptor_v3"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_loader_manifest_bad_abi_table(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = write_native_dynamic_reports(out)
            rewrite_loader_manifest(
                native_report_path,
                "\n".join(
                    [
                        "[[plugins]]",
                        'id = "animation"',
                        'path = "plugins/animation"',
                        'manifest = "plugins/animation/plugin.toml"',
                        (
                            'package_report = "plugins/animation/'
                            'native_dynamic_package.toml"'
                        ),
                        'abi = "legacy"',
                    ]
                )
                + "\n",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("NativeDynamic", report["fatal_stages"])
            self.assertTrue(
                any(
                    "native_dynamic loader_manifest plugins[0].abi "
                    "must be a table"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_loader_manifest_unknown_abi_field(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = write_native_dynamic_reports(out)
            rewrite_loader_manifest(
                native_report_path,
                loader_manifest_with_overrides()
                + 'future_contract = "ignored"\n',
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("NativeDynamic", report["fatal_stages"])
            self.assertTrue(
                any(
                    "native_dynamic loader_manifest plugin animation "
                    "abi.future_contract is not supported by "
                    "native_dynamic report package_exports"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_loader_manifest_abi_field_types(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = write_native_dynamic_reports(out)
            rewrite_loader_manifest(
                native_report_path,
                loader_manifest_with_overrides()
                .replace("abi_version = 3", 'abi_version = "3"')
                .replace(
                    'descriptor_symbol = "zircon_native_plugin_descriptor_v3"',
                    "descriptor_symbol = 42",
                ),
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("NativeDynamic", report["fatal_stages"])
            self.assertTrue(
                any(
                    "native_dynamic loader_manifest plugin animation "
                    "abi.abi_version must be an integer"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertTrue(
                any(
                    "native_dynamic loader_manifest plugin animation "
                    "abi.descriptor_symbol must be a string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_loader_manifest_abi_missing_required_field(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = write_native_dynamic_reports(out)
            rewrite_loader_manifest(
                native_report_path,
                loader_manifest_with_overrides().replace(
                    'descriptor_contract = "NativePluginAbiV3"',
                    "",
                ),
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("NativeDynamic", report["fatal_stages"])
            self.assertTrue(
                any(
                    "native_dynamic loader_manifest plugin animation "
                    "abi.descriptor_contract is required when abi is present"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_loader_manifest_unknown_plugin_field(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = write_native_dynamic_reports(out)
            rewrite_loader_manifest(
                native_report_path,
                loader_manifest_with_overrides().replace(
                    "\n[plugins.abi]\n",
                    '\nfuture_field = "ignored"\n\n[plugins.abi]\n',
                ),
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("NativeDynamic", report["fatal_stages"])
            self.assertTrue(
                any(
                    "native_dynamic loader_manifest plugins[0].future_field "
                    "is not supported"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_loader_manifest_unknown_top_level_field(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = write_native_dynamic_reports(out)
            rewrite_loader_manifest(
                native_report_path,
                loader_manifest_with_overrides()
                + "\n[metadata]\nsource = \"sidecar\"\n",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("NativeDynamic", report["fatal_stages"])
            self.assertTrue(
                any(
                    "native_dynamic loader_manifest metadata is not supported"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_loader_manifest_string_field_type(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = write_native_dynamic_reports(out)
            rewrite_loader_manifest(
                native_report_path,
                loader_manifest_with_overrides().replace(
                    'path = "plugins/animation"',
                    "path = 42",
                ),
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("NativeDynamic", report["fatal_stages"])
            self.assertTrue(
                any(
                    "native_dynamic loader_manifest plugins[0].path "
                    "must be a string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_loader_manifest_missing_plugins_table(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = write_native_dynamic_reports(out)
            rewrite_loader_manifest(
                native_report_path,
                'plugins = "animation"\n',
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("NativeDynamic", report["fatal_stages"])
            self.assertTrue(
                any(
                    "native_dynamic loader_manifest plugins must be an array"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_loader_manifest_bad_plugin_id(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = write_native_dynamic_reports(out)
            rewrite_loader_manifest(
                native_report_path,
                "[[plugins]]\nid = 42\n",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("NativeDynamic", report["fatal_stages"])
            self.assertTrue(
                any(
                    "native_dynamic loader_manifest plugins[0].id "
                    "must be a non-empty string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
