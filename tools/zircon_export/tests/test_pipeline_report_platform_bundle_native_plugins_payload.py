from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.platform_bundle_report_test_support import (
    _read_stage_report,
    _remove_tree,
    _write_bundle_manifest_from_platform_report,
    _write_native_plugins,
    _write_platform_bundle_fixture,
    _write_stage_report,
    _write_text,
    _write_validate_report_with_strategies,
)


class PlatformBundleNativePluginsPayloadReportTests(unittest.TestCase):
    def test_report_rejects_stale_native_plugins_payload_hash(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            fixture = _write_platform_bundle_fixture(out)
            plugin_artifact = fixture["native_plugins"] / "animation" / "native" / "zircon_plugin_animation.dll"
            plugin_artifact.write_text("mutated plugin", encoding="utf-8")

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any("native_plugins_payload content_hash" in diagnostic for diagnostic in report["diagnostics"]),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_missing_native_plugins_payload_directory(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            fixture = _write_platform_bundle_fixture(out)
            _remove_tree(fixture["native_plugins"])

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any("native_plugins" in diagnostic and "does not exist" in diagnostic for diagnostic in report["diagnostics"]),
                report["diagnostics"],
            )

    def test_report_rejects_empty_platform_host_output(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            fixture = _write_platform_bundle_fixture(out)
            manual_host_source = root / "manual" / "zircon_runtime.exe"
            manual_host_source.parent.mkdir(parents=True, exist_ok=True)
            manual_host_source.write_bytes(b"")
            fixture["platform_host"].write_bytes(b"")
            platform_report = _read_stage_report(out, "platform_bundle")
            platform_report["host_source"] = str(manual_host_source)
            platform_report["host_source_origin"] = "argument"
            _write_stage_report(out, "platform_bundle", platform_report)
            _write_bundle_manifest_from_platform_report(
                fixture["bundle_manifest"],
                platform_report,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "PlatformBundle report host_executable" in diagnostic
                    and "is empty" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )

    def test_report_rejects_native_plugins_payload_package_count_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_platform_bundle_fixture(
                out,
                payload_overrides={"package_count": 2},
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any("package_count does not match" in diagnostic for diagnostic in report["diagnostics"]),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_bundle_path_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            external_plugins = root / "external" / "plugins"
            _write_platform_bundle_fixture(
                out,
                payload_overrides={"bundle_path": str(external_plugins)},
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "native_plugins_payload bundle_path"
                    in diagnostic
                    and "does not match native_plugins"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_stage_report_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            external_stage_report = root / "external" / "native_dynamic" / "report.json"
            _write_text(external_stage_report, "{}")
            _write_platform_bundle_fixture(
                out,
                payload_overrides={"stage_report": str(external_stage_report)},
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "native_plugins_payload stage_report"
                    in diagnostic
                    and "does not match NativeDynamic report"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_missing_stage_report_for_pipeline_payload(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            platform_report = _read_stage_report(out, "platform_bundle")
            payload = platform_report["native_plugins_payload"]
            self.assertIsInstance(payload, dict)
            payload["stage_report"] = None
            _write_stage_report(out, "platform_bundle", platform_report)
            _write_bundle_manifest_from_platform_report(
                fixture["bundle_manifest"],
                platform_report,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "native_plugins_payload stage_report is required"
                    in diagnostic
                    and "NativeDynamic stage report is present"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_does_not_use_fatal_native_dynamic_stage_report_for_payload(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_platform_bundle_fixture(out)
            native_report = _read_stage_report(out, "native_dynamic")
            packages = native_report["materialized_packages"]
            self.assertIsInstance(packages, list)
            package = packages[0]
            self.assertIsInstance(package, dict)
            package["loadable_artifact_count"] = -1
            _write_stage_report(out, "native_dynamic", native_report)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("NativeDynamic", report["fatal_stages"])
            self.assertTrue(
                any(
                    "native_dynamic report materialized_packages[0].loadable_artifact_count "
                    "must be non-negative"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(
                any(
                    "NativeDynamic report materialized_packages are malformed"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(
                any(
                    "native_plugins_payload" in diagnostic
                    and "not backed by the current NativeDynamic report" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_does_not_use_fatal_pack_stage_report_for_pack_source(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_platform_bundle_fixture(out, with_delta=True)
            pack_report = _read_stage_report(out, "pack")
            pack_report["pack"] = []
            _write_stage_report(out, "pack", pack_report)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertIn("Pack", report["fatal_stages"])
            self.assertTrue(
                any(
                    "pack report pack must be a string" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(
                any(
                    "PlatformBundle report pack is present but "
                    "Pack report does not contain pack evidence" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(
                any(
                    "PlatformBundle report delta_pack is present but "
                    "Pack report does not contain verified delta_pack evidence"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_source_mismatch_for_stage_payload(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            external_source = root / "external" / "plugins"
            _write_platform_bundle_fixture(
                out,
                payload_overrides={
                    "stage_report": str(
                        out / "stages" / "native_dynamic" / "report.json"
                    ),
                    "source": str(external_source),
                },
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "native_plugins_payload source"
                    in diagnostic
                    and "does not match NativeDynamic plugins"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_package_report_outside_package(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            external_report = root / "external" / "native_dynamic_package.toml"
            _write_text(external_report, 'package_id = "animation"\n')
            fixture = _write_platform_bundle_fixture(out)
            platform_report = _read_stage_report(out, "platform_bundle")
            payload = platform_report["native_plugins_payload"]
            self.assertIsInstance(payload, dict)
            packages = payload["materialized_packages"]
            self.assertIsInstance(packages, list)
            package = packages[0]
            self.assertIsInstance(package, dict)
            package["package_report"] = str(external_report)
            _write_stage_report(out, "platform_bundle", platform_report)
            _write_bundle_manifest_from_platform_report(
                fixture["bundle_manifest"],
                platform_report,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "native_plugins_payload materialized_packages[0] package_report"
                    in diagnostic
                    and "is outside package destination"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_package_destination_outside_plugins(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            external_package = root / "external" / "animation"
            _write_platform_bundle_fixture(out)
            platform_report = _read_stage_report(out, "platform_bundle")
            payload = platform_report["native_plugins_payload"]
            self.assertIsInstance(payload, dict)
            packages = payload["materialized_packages"]
            self.assertIsInstance(packages, list)
            package = packages[0]
            self.assertIsInstance(package, dict)
            package["destination"] = str(external_package)
            _write_stage_report(out, "platform_bundle", platform_report)
            bundle_manifest = Path(str(platform_report["bundle_manifest"]))
            _write_bundle_manifest_from_platform_report(
                bundle_manifest,
                platform_report,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "native_plugins_payload materialized_packages[0] destination"
                    in diagnostic
                    and "is outside native_plugins"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_missing_native_plugins_payload_package_report(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_platform_bundle_fixture(out)
            platform_report = _read_stage_report(out, "platform_bundle")
            payload = platform_report["native_plugins_payload"]
            self.assertIsInstance(payload, dict)
            packages = payload["materialized_packages"]
            self.assertIsInstance(packages, list)
            package = packages[0]
            self.assertIsInstance(package, dict)
            package_report = Path(str(package["package_report"]))
            package_report.unlink()

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "native_plugins_payload materialized_packages[0] package_report"
                    in diagnostic
                    and "does not exist"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_accepts_current_native_plugins_payload(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_platform_bundle_fixture(out)

            report = build_pipeline_report(out, "windows-release")

            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])

    def test_report_accepts_current_native_plugins_payload_for_legacy_validate_report(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_platform_bundle_fixture(out)
            validate_report = _read_stage_report(out, "validate")
            validate_report["profile_summary"].pop("strategies")
            _write_stage_report(out, "validate", validate_report)

            report = build_pipeline_report(out, "windows-release")

            self.assertFalse(report["fatal"], report["diagnostics"])
            self.assertEqual(report["missing_stages"], [])

    def test_report_rejects_legacy_native_plugins_payload_wrong_profile_stage_report(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_platform_bundle_fixture(out)
            validate_report = _read_stage_report(out, "validate")
            validate_report["profile_summary"].pop("strategies")
            _write_stage_report(out, "validate", validate_report)
            native_report = _read_stage_report(out, "native_dynamic")
            native_report["profile"] = "other-profile"
            _write_stage_report(out, "native_dynamic", native_report)

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "NativeDynamic report profile other-profile does not match requested profile windows-release"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_legacy_native_plugins_payload_stage_report_directory(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_platform_bundle_fixture(out)
            validate_report = _read_stage_report(out, "validate")
            validate_report["profile_summary"].pop("strategies")
            _write_stage_report(out, "validate", validate_report)
            native_report_path = out / "stages" / "native_dynamic" / "report.json"
            native_report_path.unlink()
            native_report_path.mkdir()

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "NativeDynamic report" in diagnostic
                    and "is not a file" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_external_native_plugins_payload_stage_report_for_legacy_validate_report(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            fixture = _write_platform_bundle_fixture(out)
            validate_report = _read_stage_report(out, "validate")
            validate_report["profile_summary"].pop("strategies")
            _write_stage_report(out, "validate", validate_report)
            external_plugins = root / "external" / "native_dynamic" / "plugins"
            _write_native_plugins(external_plugins)
            external_stage_report = external_plugins.parent / "report.json"
            _write_text(external_stage_report, "{}")
            platform_report = _read_stage_report(out, "platform_bundle")
            payload = platform_report["native_plugins_payload"]
            self.assertIsInstance(payload, dict)
            payload["stage_report"] = str(external_stage_report)
            payload["source"] = str(external_plugins)
            _write_stage_report(out, "platform_bundle", platform_report)
            _write_bundle_manifest_from_platform_report(
                fixture["bundle_manifest"],
                platform_report,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "native_plugins_payload stage_report"
                    in diagnostic
                    and "does not match NativeDynamic report"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_for_library_embed_strategy(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            _write_platform_bundle_fixture(out)
            _write_validate_report_with_strategies(out, ["library_embed"])

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "native_plugins_payload"
                    in diagnostic
                    and "native_dynamic strategy"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_manual_native_plugins_payload_without_stage_handoff_noise(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            _write_validate_report_with_strategies(out, ["library_embed"])
            platform_report = _read_stage_report(out, "platform_bundle")
            payload = platform_report["native_plugins_payload"]
            self.assertIsInstance(payload, dict)
            payload["stage_report"] = None
            payload["source"] = platform_report["native_plugins"]
            _write_stage_report(out, "platform_bundle", platform_report)
            _write_bundle_manifest_from_platform_report(
                fixture["bundle_manifest"],
                platform_report,
            )

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertTrue(
                any(
                    "native_plugins_payload" in diagnostic
                    and "native_dynamic strategy" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertFalse(
                any(
                    "stage_report is required" in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )


if __name__ == "__main__":
    unittest.main()