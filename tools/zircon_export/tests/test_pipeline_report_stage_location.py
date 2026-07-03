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
    _native_dynamic_content_hash,
    _native_dynamic_package_export,
    _native_dynamic_plugins_file_manifest,
    _write_native_dynamic_report,
    _write_native_dynamic_stage_plugins,
)
from tools.zircon_export.tests.pack_schema_test_support import write_library_embed_reports


class PipelineReportStageLocationTests(unittest.TestCase):
    def _write_native_dynamic_reports(self, out: Path) -> Path:
        _write_validate_report_with_native_dynamic_exports(out)
        native_plugins = _write_native_dynamic_stage_plugins(
            out / "stages" / "native_dynamic"
        )
        _write_native_dynamic_report(out, native_plugins)
        _write_compile_host_report(out, out / "compile" / "zircon_runtime.exe")
        _write_stage_report(out, "cook_assets", fatal=False)
        _write_pack_report(out, out / "stages" / "pack" / "assets.zrpack")
        _write_stage_report(out, "platform_bundle", fatal=False)
        return out / "stages" / "native_dynamic" / "report.json"

    def test_report_stage_rejects_validate_stage_output_outside_current_stage(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            write_library_embed_reports(out)
            validate_report_path = out / "stages" / "validate" / "report.json"
            validate_report = json.loads(
                validate_report_path.read_text(encoding="utf-8")
            )
            validate_report["stage_output"] = str(root / "external" / "validate")
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
                    "validate report stage_output "
                    f"{root / 'external' / 'validate'} does not match current "
                    f"Validate stage directory {validate_report_path.parent}"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_stage_output_outside_current_stage(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            native_report_path = self._write_native_dynamic_reports(out)
            native_report = json.loads(native_report_path.read_text(encoding="utf-8"))
            native_report["stage_output"] = str(root / "external" / "native_dynamic")
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
                    "native_dynamic report stage_output "
                    f"{root / 'external' / 'native_dynamic'} does not match "
                    f"current NativeDynamic stage directory "
                    f"{native_report_path.parent}"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_plugins_dir_outside_current_stage(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            native_report_path = self._write_native_dynamic_reports(out)
            current_stage = native_report_path.parent
            external_plugins = _write_native_dynamic_stage_plugins(
                root / "external" / "native_dynamic"
            )
            _write_native_dynamic_report(out, external_plugins)
            native_report = json.loads(native_report_path.read_text(encoding="utf-8"))
            native_report["stage_output"] = str(current_stage)
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
                    "native_dynamic report plugins_dir "
                    f"{external_plugins} does not match current "
                    f"NativeDynamic plugins directory {current_stage / 'plugins'}"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_loader_manifest_outside_current_stage(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            native_report_path = self._write_native_dynamic_reports(out)
            current_stage = native_report_path.parent
            external_plugins = _write_native_dynamic_stage_plugins(
                root / "external" / "native_dynamic"
            )
            native_report = json.loads(native_report_path.read_text(encoding="utf-8"))
            native_report["loader_manifest"] = str(
                external_plugins / "native_plugins.toml"
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
                    "native_dynamic report loader_manifest "
                    f"{external_plugins / 'native_plugins.toml'} does not match "
                    "current NativeDynamic loader manifest "
                    f"{current_stage / 'plugins' / 'native_plugins.toml'}"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_stale_file_manifest(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            native_report_path = self._write_native_dynamic_reports(out)
            current_plugins = native_report_path.parent / "plugins"
            current_artifact = (
                current_plugins
                / "animation"
                / "native"
                / "zircon_plugin_animation.dll"
            )
            current_artifact.write_text("changed native payload", encoding="utf-8")
            native_report = json.loads(native_report_path.read_text(encoding="utf-8"))
            stale_content_hash = native_report["content_hash"]

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("NativeDynamic", report["fatal_stages"])
            self.assertTrue(
                any(
                    "native_dynamic report content_hash "
                    f"{stale_content_hash} does not match current NativeDynamic "
                    f"plugins directory {current_plugins} content_hash "
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_loader_manifest_package_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            native_report_path = self._write_native_dynamic_reports(out)
            current_plugins = native_report_path.parent / "plugins"
            loader_manifest = current_plugins / "native_plugins.toml"
            loader_manifest.write_text(
                '[[plugins]]\nid = "physics"\n',
                encoding="utf-8",
            )
            native_report = json.loads(native_report_path.read_text(encoding="utf-8"))
            file_manifest = _native_dynamic_plugins_file_manifest(current_plugins)
            native_report["file_manifest"] = file_manifest
            native_report["content_hash"] = _native_dynamic_content_hash(
                file_manifest
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
                    "native_dynamic loader_manifest plugin ids ['physics'] "
                    "do not match materialized package ids ['animation']"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_malformed_loader_manifest(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            native_report_path = self._write_native_dynamic_reports(out)
            current_plugins = native_report_path.parent / "plugins"
            loader_manifest = current_plugins / "native_plugins.toml"
            loader_manifest.write_text(
                "[[plugins]\nid = \"animation\"\n",
                encoding="utf-8",
            )
            native_report = json.loads(native_report_path.read_text(encoding="utf-8"))
            file_manifest = _native_dynamic_plugins_file_manifest(current_plugins)
            native_report["file_manifest"] = file_manifest
            native_report["content_hash"] = _native_dynamic_content_hash(
                file_manifest
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
                    f"native_dynamic loader_manifest {loader_manifest} "
                    "could not be parsed"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_package_destination_outside_plugins_dir(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            native_report_path = self._write_native_dynamic_reports(out)
            current_plugins = native_report_path.parent / "plugins"
            external_package = root / "external" / "native_dynamic" / "animation"
            external_package.mkdir(parents=True)
            native_report = json.loads(native_report_path.read_text(encoding="utf-8"))
            packages = native_report["materialized_packages"]
            self.assertIsInstance(packages, list)
            package = packages[0]
            self.assertIsInstance(package, dict)
            package["destination"] = str(external_package)
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
                    "NativeDynamic payload materialized_packages[0] "
                    f"destination {external_package} is outside plugins_dir "
                    f"{current_plugins}"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_package_count_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            native_report_path = self._write_native_dynamic_reports(out)
            native_report = json.loads(native_report_path.read_text(encoding="utf-8"))
            native_report["package_count"] = 2
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
                    "native_dynamic report package_count 2 does not match "
                    "materialized_packages 1"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_package_selection_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            native_report_path = self._write_native_dynamic_reports(out)
            native_report = json.loads(native_report_path.read_text(encoding="utf-8"))
            native_report["native_dynamic_packages"] = ["physics"]
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
                    "native_dynamic report native_dynamic_packages ['physics'] "
                    "does not match materialized package ids ['animation']"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_package_selection_validate_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            native_report_path = self._write_native_dynamic_reports(out)
            _write_validate_report_with_native_dynamic_exports(
                out,
                package_export_overrides={
                    "package_id": "physics",
                    "directory": "physics",
                    "path": "plugins/physics",
                    "manifest": "plugins/physics/plugin.toml",
                    "package_report": (
                        "plugins/physics/native_dynamic_package.toml"
                    ),
                },
                native_dynamic_packages=["physics"],
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("NativeDynamic", report["fatal_stages"])
            self.assertTrue(
                any(
                    "native_dynamic report materialized package ids "
                    "['animation'] do not match validate report "
                    "plan_summary.native_dynamic_packages ['physics']"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertTrue(native_report_path.exists())

    def test_report_stage_rejects_native_dynamic_package_export_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            native_report_path = self._write_native_dynamic_reports(out)
            native_report = json.loads(native_report_path.read_text(encoding="utf-8"))
            native_report["package_exports"] = [
                _native_dynamic_package_export(
                    {
                        "package_id": "physics",
                        "directory": "physics",
                        "path": "plugins/physics",
                        "manifest": "plugins/physics/plugin.toml",
                        "package_report": (
                            "plugins/physics/native_dynamic_package.toml"
                        ),
                    }
                )
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
                    "native_dynamic report package_exports package ids "
                    "['physics'] do not match materialized package ids "
                    "['animation']"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_package_export_validate_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            self._write_native_dynamic_reports(out)
            validate_report_path = out / "stages" / "validate" / "report.json"
            validate_report = json.loads(
                validate_report_path.read_text(encoding="utf-8")
            )
            package_export = validate_report["plan_summary"][
                "native_dynamic_package_exports"
            ][0]
            package_export.update(
                {
                    "package_id": "physics",
                    "directory": "physics",
                    "path": "plugins/physics",
                    "manifest": "plugins/physics/plugin.toml",
                    "package_report": (
                        "plugins/physics/native_dynamic_package.toml"
                    ),
                }
            )
            validate_report_path.write_text(
                json.dumps(validate_report, indent=2),
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("NativeDynamic", report["fatal_stages"])
            self.assertTrue(
                any(
                    "native_dynamic report materialized package ids "
                    "['animation'] do not match validate report "
                    "plan_summary.native_dynamic_package_exports package ids "
                    "['physics']"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_build_plan_package_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            native_report_path = self._write_native_dynamic_reports(out)
            native_report = json.loads(native_report_path.read_text(encoding="utf-8"))
            native_report["native_build_plan"] = {
                "fatal": False,
                "workspace_manifest": "zircon_plugins/Cargo.toml",
                "target_dir": "target/native_dynamic",
                "cargo_profile": "release",
                "release": True,
                "build_features": [],
                "package_count": 1,
                "diagnostics": [],
                "packages": [
                    {
                        "package_id": "physics",
                        "crate_name": "zircon_plugin_physics_native",
                        "manifest_path": "zircon_plugins/physics/native/Cargo.toml",
                        "workspace_manifest": "zircon_plugins/Cargo.toml",
                        "target_dir": "target/native_dynamic",
                        "cargo_profile": "release",
                        "release": True,
                        "features": [],
                        "command": ["cargo", "build"],
                        "expected_loadable_artifact": (
                            "target/native_dynamic/release/"
                            "zircon_plugin_physics_native.dll"
                        ),
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
                    "native_dynamic report native_build_plan package ids "
                    "['physics'] do not match materialized package ids "
                    "['animation']"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_build_execution_package_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            native_report_path = self._write_native_dynamic_reports(out)
            native_report = json.loads(native_report_path.read_text(encoding="utf-8"))
            native_report["native_build_execution"] = {
                "enabled": True,
                "fatal": False,
                "diagnostics": [],
                "package_count": 1,
                "packages": [
                    {
                        "package_id": "physics",
                        "crate_name": "zircon_plugin_physics_native",
                        "command": ["cargo", "build"],
                        "exit_code": 0,
                        "stdout": "",
                        "stderr": "",
                        "expected_loadable_artifact": (
                            "target/native_dynamic/release/"
                            "zircon_plugin_physics_native.dll"
                        ),
                        "copied_loadable_artifact": (
                            "plugins/physics/native/"
                            "zircon_plugin_physics_native.dll"
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
                    "native_dynamic report native_build_execution package ids "
                    "['physics'] do not match materialized package ids "
                    "['animation']"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_signing_package_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            native_report_path = self._write_native_dynamic_reports(out)
            native_report = json.loads(native_report_path.read_text(encoding="utf-8"))
            native_report["native_signing"] = {
                "enabled": True,
                "profile": "windows-release",
                "target_platform": "windows-x86_64",
                "allowed_platforms": ["windows"],
                "platform_allowed": True,
                "fatal": False,
                "package_count": 1,
                "diagnostics": [],
                "packages": [
                    {
                        "package_id": "physics",
                        "artifact_count": 0,
                        "artifacts": [],
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
                    "native_dynamic report native_signing package ids "
                    "['physics'] do not match materialized package ids "
                    "['animation']"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_signing_artifact_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            native_report_path = self._write_native_dynamic_reports(out)
            current_plugins = native_report_path.parent / "plugins"
            forged_artifact = (
                current_plugins / "animation" / "native" / "forged.dll"
            )
            native_report = json.loads(native_report_path.read_text(encoding="utf-8"))
            native_report["native_signing"] = {
                "enabled": True,
                "profile": "windows-store",
                "target_platform": "windows-x86_64",
                "allowed_platforms": ["windows"],
                "platform_allowed": True,
                "fatal": False,
                "package_count": 1,
                "diagnostics": [],
                "packages": [
                    {
                        "package_id": "animation",
                        "artifact_count": 1,
                        "artifacts": [
                            {
                                "artifact": str(forged_artifact),
                                "package_relative_artifact": "native/forged.dll",
                                "command": ["signtool", "sign"],
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
                    "package_relative_artifacts ['native/forged.dll'] do not "
                    "match materialized loadable artifacts "
                    "['native/zircon_plugin_animation.dll']"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_signing_artifact_count_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            native_report_path = self._write_native_dynamic_reports(out)
            current_plugins = native_report_path.parent / "plugins"
            current_artifact = (
                current_plugins
                / "animation"
                / "native"
                / "zircon_plugin_animation.dll"
            )
            native_report = json.loads(native_report_path.read_text(encoding="utf-8"))
            native_report["native_signing"] = {
                "enabled": True,
                "profile": "windows-store",
                "target_platform": "windows-x86_64",
                "allowed_platforms": ["windows"],
                "platform_allowed": True,
                "fatal": False,
                "package_count": 1,
                "diagnostics": [],
                "packages": [
                    {
                        "package_id": "animation",
                        "artifact_count": 2,
                        "artifacts": [
                            {
                                "artifact": str(current_artifact),
                                "package_relative_artifact": (
                                    "native/zircon_plugin_animation.dll"
                                ),
                                "command": ["signtool", "sign"],
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
                    "artifact_count 2 does not match artifacts 1"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_notarization_package_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            native_report_path = self._write_native_dynamic_reports(out)
            native_report = json.loads(native_report_path.read_text(encoding="utf-8"))
            native_report["native_notarization"] = {
                "enabled": True,
                "profile": "windows-release",
                "target_platform": "windows-x86_64",
                "allowed_platforms": ["windows"],
                "platform_allowed": True,
                "fatal": False,
                "package_count": 1,
                "diagnostics": [],
                "packages": [
                    {
                        "package_id": "physics",
                        "artifact_count": 0,
                        "artifacts": [],
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
                    "native_dynamic report native_notarization package ids "
                    "['physics'] do not match materialized package ids "
                    "['animation']"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_notarization_artifact_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            native_report_path = self._write_native_dynamic_reports(out)
            current_plugins = native_report_path.parent / "plugins"
            forged_artifact = (
                current_plugins / "animation" / "native" / "forged.dll"
            )
            native_report = json.loads(native_report_path.read_text(encoding="utf-8"))
            native_report["native_notarization"] = {
                "enabled": True,
                "profile": "windows-attestation",
                "target_platform": "windows-x86_64",
                "allowed_platforms": ["windows"],
                "platform_allowed": True,
                "fatal": False,
                "package_count": 1,
                "diagnostics": [],
                "packages": [
                    {
                        "package_id": "animation",
                        "artifact_count": 1,
                        "artifacts": [
                            {
                                "artifact": str(forged_artifact),
                                "package_relative_artifact": "native/forged.dll",
                                "command": ["notarytool", "submit"],
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
                    "native_dynamic report native_notarization package animation "
                    "package_relative_artifacts ['native/forged.dll'] do not "
                    "match materialized loadable artifacts "
                    "['native/zircon_plugin_animation.dll']"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()
