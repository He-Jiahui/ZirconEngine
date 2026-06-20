from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.export_test_support import (
    _native_dynamic_content_hash,
    _native_dynamic_package_export,
    _native_dynamic_plugins_file_manifest,
    _write_native_dynamic_package_report,
)
from tools.zircon_export.tests.native_dynamic_stage_report_test_support import (
    refresh_native_dynamic_report_payload,
    write_native_dynamic_reports,
    write_package_report_for_directory,
)


class PipelineReportNativeDynamicStageMaterializedPackageTests(unittest.TestCase):
    def test_report_stage_rejects_native_dynamic_package_report_id_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = write_native_dynamic_reports(out)
            package_report = (
                native_report_path.parent
                / "plugins"
                / "animation"
                / "native_dynamic_package.toml"
            )
            package_report.write_text(
                "\n".join(
                    [
                        "format_version = 1",
                        'package_id = "physics"',
                        'directory = "animation"',
                        'path = "plugins/animation"',
                        'manifest = "plugins/animation/plugin.toml"',
                    ]
                )
                + "\n",
                encoding="utf-8",
            )
            refresh_native_dynamic_report_payload(native_report_path)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("NativeDynamic", report["fatal_stages"])
            self.assertTrue(
                any(
                    "native_dynamic report materialized_packages[0] "
                    "package_report package_id physics does not match "
                    "materialized package animation"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_missing_materialized_package_report(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = write_native_dynamic_reports(out)
            native_report = json.loads(native_report_path.read_text(encoding="utf-8"))
            materialized_packages = native_report["materialized_packages"]
            self.assertIsInstance(materialized_packages, list)
            package = materialized_packages[0]
            self.assertIsInstance(package, dict)
            package.pop("package_report")
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
                    "native_dynamic report materialized_packages[0] "
                    "package_report is required for NativeDynamic stage "
                    "materialized packages"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_missing_materialized_package_source(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = write_native_dynamic_reports(out)
            native_report = json.loads(native_report_path.read_text(encoding="utf-8"))
            materialized_packages = native_report["materialized_packages"]
            self.assertIsInstance(materialized_packages, list)
            package = materialized_packages[0]
            self.assertIsInstance(package, dict)
            package.pop("source")
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
                    "native_dynamic report materialized_packages[0] "
                    "source is required for NativeDynamic stage "
                    "materialized packages"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_empty_materialized_package_source(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = write_native_dynamic_reports(out)
            native_report = json.loads(native_report_path.read_text(encoding="utf-8"))
            materialized_packages = native_report["materialized_packages"]
            self.assertIsInstance(materialized_packages, list)
            package = materialized_packages[0]
            self.assertIsInstance(package, dict)
            package["source"] = ""
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
                    "native_dynamic report materialized_packages[0] "
                    "source must be a non-empty string for NativeDynamic "
                    "stage materialized packages"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_package_source_outside_plugin_root(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            native_report_path = write_native_dynamic_reports(out)
            external_source = root / "external" / "animation"
            external_source.mkdir(parents=True)
            external_source.joinpath("plugin.toml").write_text(
                'id = "animation"\n',
                encoding="utf-8",
            )
            native_report = json.loads(native_report_path.read_text(encoding="utf-8"))
            materialized_packages = native_report["materialized_packages"]
            self.assertIsInstance(materialized_packages, list)
            package = materialized_packages[0]
            self.assertIsInstance(package, dict)
            package["source"] = str(external_source)
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
                    "native_dynamic report materialized_packages[0] "
                    f"source {external_source} is outside native_plugin_root "
                    f"{out / 'zircon_plugins'}"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_package_source_missing_manifest(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = write_native_dynamic_reports(out)
            source_manifest = out / "zircon_plugins" / "animation" / "plugin.toml"
            source_manifest.unlink()

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("NativeDynamic", report["fatal_stages"])
            self.assertTrue(
                any(
                    "native_dynamic report materialized_packages[0] "
                    f"source manifest {source_manifest} does not exist"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_package_source_manifest_id_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = write_native_dynamic_reports(out)
            source_manifest = out / "zircon_plugins" / "animation" / "plugin.toml"
            source_manifest.write_text(
                'id = "physics"\n',
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("NativeDynamic", report["fatal_stages"])
            self.assertTrue(
                any(
                    "native_dynamic report materialized_packages[0] "
                    "source manifest id physics does not match "
                    "materialized package animation"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_package_source_manifest_parse_error(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = write_native_dynamic_reports(out)
            source_manifest = out / "zircon_plugins" / "animation" / "plugin.toml"
            source_manifest.write_text(
                "id = \n",
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("NativeDynamic", report["fatal_stages"])
            self.assertTrue(
                any(
                    "native_dynamic report materialized_packages[0] "
                    f"source manifest {source_manifest} could not be parsed"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_package_source_manifest_missing_id(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = write_native_dynamic_reports(out)
            source_manifest = out / "zircon_plugins" / "animation" / "plugin.toml"
            source_manifest.write_text(
                'name = "animation"\n',
                encoding="utf-8",
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("NativeDynamic", report["fatal_stages"])
            self.assertTrue(
                any(
                    "native_dynamic report materialized_packages[0] "
                    "source manifest id must be a non-empty string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_empty_materialized_package_report(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = write_native_dynamic_reports(out)
            native_report = json.loads(native_report_path.read_text(encoding="utf-8"))
            materialized_packages = native_report["materialized_packages"]
            self.assertIsInstance(materialized_packages, list)
            package = materialized_packages[0]
            self.assertIsInstance(package, dict)
            package["package_report"] = ""
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
                    "native_dynamic report materialized_packages[0] "
                    "package_report must be a non-empty string for "
                    "NativeDynamic stage materialized packages"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_package_report_payload_hash_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = write_native_dynamic_reports(out)
            package_report = (
                native_report_path.parent
                / "plugins"
                / "animation"
                / "native_dynamic_package.toml"
            )
            package_report.write_text(
                "\n".join(
                    [
                        "format_version = 1",
                        'package_id = "animation"',
                        'directory = "animation"',
                        'path = "plugins/animation"',
                        'manifest = "plugins/animation/plugin.toml"',
                        "",
                        "[payload]",
                        "file_count = 1",
                        f'content_hash = "{"0" * 64}"',
                    ]
                )
                + "\n",
                encoding="utf-8",
            )
            refresh_native_dynamic_report_payload(native_report_path)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("NativeDynamic", report["fatal_stages"])
            self.assertTrue(
                any(
                    "native_dynamic report materialized_packages[0] "
                    "package_report payload content_hash "
                    in diagnostic
                    and "does not match current package payload" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_unreported_loadable_artifact(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = write_native_dynamic_reports(out)
            package_dir = native_report_path.parent / "plugins" / "animation"
            package_dir.joinpath(
                "native",
                "zircon_plugin_animation_extra.dll",
            ).write_text(
                "extra loadable",
                encoding="utf-8",
            )
            _write_native_dynamic_package_report(package_dir)
            refresh_native_dynamic_report_payload(native_report_path)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("NativeDynamic", report["fatal_stages"])
            self.assertTrue(
                any(
                    "NativeDynamic payload materialized_packages[0] "
                    "loadable_artifacts do not include current loadable "
                    "artifact "
                    "plugins/animation/native/"
                    "zircon_plugin_animation_extra.dll"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_stage_rejects_native_dynamic_package_export_path_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            native_report_path = write_native_dynamic_reports(out)
            plugins_dir = native_report_path.parent / "plugins"
            package_dir = plugins_dir / "animation"
            forged_package_dir = plugins_dir / "forged-animation"
            package_dir.rename(forged_package_dir)
            write_package_report_for_directory(
                forged_package_dir,
                directory="forged-animation",
            )
            native_report = json.loads(
                native_report_path.read_text(encoding="utf-8")
            )
            materialized_package = native_report["materialized_packages"][0]
            materialized_package["destination"] = str(forged_package_dir)
            materialized_package["package_report"] = str(
                forged_package_dir / "native_dynamic_package.toml"
            )
            materialized_package["loadable_artifacts"] = [
                "plugins/forged-animation/native/zircon_plugin_animation.dll"
            ]
            materialized_package["loadable_artifact_count"] = 1
            native_report["package_exports"] = [_native_dynamic_package_export()]
            file_manifest = _native_dynamic_plugins_file_manifest(plugins_dir)
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
                    "native_dynamic report package_exports package animation "
                    "path plugins/animation does not match materialized "
                    "package path plugins/forged-animation"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
