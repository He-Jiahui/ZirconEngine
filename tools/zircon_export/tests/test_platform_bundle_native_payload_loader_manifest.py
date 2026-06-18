from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.export_test_support import (
    _platform_bundle_args,
    _run_platform_bundle_quiet,
    _write_native_dynamic_stage_plugins,
    _write_validate_report_with_strategies,
    json_loads,
)
from tools.zircon_export.tests.test_pipeline_report_platform_bundle import (
    _native_plugins_content_hash,
    _native_plugins_file_manifest,
    _read_stage_report,
    _write_bundle_manifest_from_platform_report,
    _write_platform_bundle_fixture,
    _write_stage_report,
)


class PlatformBundleNativePayloadLoaderManifestTests(unittest.TestCase):
    def test_platform_bundle_records_loader_manifest_in_native_plugins_payload(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            out = root / "out"
            pack = root / "assets.zrpack"
            pack.write_text("pack placeholder", encoding="utf-8")
            host = root / "zircon_runtime.exe"
            host.write_text("host placeholder", encoding="utf-8")
            native_plugins = _write_native_dynamic_stage_plugins(
                root / "manual-native"
            )
            _write_validate_report_with_strategies(out, ["native_dynamic"])
            args = _platform_bundle_args(
                out=out,
                profile="windows-release",
                template_dir=None,
                pack_file=pack,
                target_platform="windows-x86_64",
            )
            args.host_executable = str(host)
            args.native_plugins_dir = str(native_plugins)

            exit_code = _run_platform_bundle_quiet(args)

            report = json_loads(
                (out / "stages" / "platform_bundle" / "report.json").read_text(
                    encoding="utf-8"
                )
            )
            bundle_manifest = json_loads(
                (out / "bundle" / "windows-release" / "bundle.json").read_text(
                    encoding="utf-8"
                )
            )
            bundled_loader_manifest = (
                out / "bundle" / "windows-release" / "plugins" / "native_plugins.toml"
            )
            self.assertEqual(exit_code, 0, report["diagnostics"])
            self.assertTrue(bundled_loader_manifest.is_file())
            self.assertEqual(
                report["native_plugins_payload"]["loader_manifest"],
                str(bundled_loader_manifest),
            )
            self.assertEqual(
                bundle_manifest["native_plugins_payload"]["loader_manifest"],
                str(bundled_loader_manifest),
            )

    def test_report_rejects_native_plugins_payload_missing_loader_manifest(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            platform_report = _read_stage_report(out, "platform_bundle")
            payload = platform_report["native_plugins_payload"]
            self.assertIsInstance(payload, dict)
            payload.pop("loader_manifest", None)
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
                    "PlatformBundle report native_plugins_payload.loader_manifest "
                    "must be a string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_loader_manifest_package_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            loader_manifest = (
                fixture["native_plugins"] / "native_plugins.toml"
            )
            loader_manifest.write_text(
                '[[plugins]]\nid = "physics"\n',
                encoding="utf-8",
            )
            _refresh_platform_native_plugins_payload(out, fixture["native_plugins"])

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "PlatformBundle report native_plugins_payload "
                    "loader_manifest plugin ids ['physics'] do not match "
                    "materialized package ids ['animation']"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_malformed_loader_manifest(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            loader_manifest = (
                fixture["native_plugins"] / "native_plugins.toml"
            )
            loader_manifest.write_text(
                "[[plugins]\nid = \"animation\"\n",
                encoding="utf-8",
            )
            _refresh_platform_native_plugins_payload(out, fixture["native_plugins"])

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "PlatformBundle report native_plugins_payload "
                    f"loader_manifest {loader_manifest} could not be parsed"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_loader_manifest_missing_plugins_table(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            loader_manifest = (
                fixture["native_plugins"] / "native_plugins.toml"
            )
            loader_manifest.write_text(
                '[metadata]\nname = "animation"\n',
                encoding="utf-8",
            )
            _refresh_platform_native_plugins_payload(out, fixture["native_plugins"])

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "PlatformBundle report native_plugins_payload "
                    "loader_manifest plugins must be an array"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_loader_manifest_bad_plugin_id(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            loader_manifest = (
                fixture["native_plugins"] / "native_plugins.toml"
            )
            loader_manifest.write_text(
                "[[plugins]]\nid = 42\n",
                encoding="utf-8",
            )
            _refresh_platform_native_plugins_payload(out, fixture["native_plugins"])

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "PlatformBundle report native_plugins_payload "
                    "loader_manifest plugins[0].id must be a non-empty string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_loader_manifest_path_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            loader_manifest = (
                fixture["native_plugins"] / "native_plugins.toml"
            )
            loader_manifest.write_text(
                "\n".join(
                    [
                        "[[plugins]]",
                        'id = "animation"',
                        'path = "plugins/forged"',
                        'manifest = "plugins/animation/plugin.toml"',
                        'package_report = "plugins/animation/native_dynamic_package.toml"',
                    ]
                )
                + "\n",
                encoding="utf-8",
            )
            _refresh_platform_native_plugins_payload(out, fixture["native_plugins"])

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "PlatformBundle report native_plugins_payload "
                    "loader_manifest plugin animation path plugins/forged "
                    "does not match materialized package path plugins/animation"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_loader_manifest_manifest_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            loader_manifest = (
                fixture["native_plugins"] / "native_plugins.toml"
            )
            loader_manifest.write_text(
                "\n".join(
                    [
                        "[[plugins]]",
                        'id = "animation"',
                        'path = "plugins/animation"',
                        'manifest = "plugins/forged/plugin.toml"',
                        'package_report = "plugins/animation/native_dynamic_package.toml"',
                    ]
                )
                + "\n",
                encoding="utf-8",
            )
            _refresh_platform_native_plugins_payload(out, fixture["native_plugins"])

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "PlatformBundle report native_plugins_payload "
                    "loader_manifest plugin animation manifest "
                    "plugins/forged/plugin.toml does not match materialized "
                    "package manifest plugins/animation/plugin.toml"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_loader_manifest_package_report_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            loader_manifest = (
                fixture["native_plugins"] / "native_plugins.toml"
            )
            loader_manifest.write_text(
                "\n".join(
                    [
                        "[[plugins]]",
                        'id = "animation"',
                        'path = "plugins/animation"',
                        'manifest = "plugins/animation/plugin.toml"',
                        'package_report = "plugins/forged/native_dynamic_package.toml"',
                    ]
                )
                + "\n",
                encoding="utf-8",
            )
            _refresh_platform_native_plugins_payload(out, fixture["native_plugins"])

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "PlatformBundle report native_plugins_payload "
                    "loader_manifest plugin animation package_report "
                    "plugins/forged/native_dynamic_package.toml does not "
                    "match materialized package package_report "
                    "plugins/animation/native_dynamic_package.toml"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_loader_manifest_abi_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            loader_manifest = (
                fixture["native_plugins"] / "native_plugins.toml"
            )
            loader_manifest.write_text(
                "\n".join(
                    [
                        "[[plugins]]",
                        'id = "animation"',
                        'path = "plugins/animation"',
                        'manifest = "plugins/animation/plugin.toml"',
                        'package_report = "plugins/animation/native_dynamic_package.toml"',
                        "",
                        "[plugins.abi]",
                        'descriptor_symbol = "zircon_native_plugin_descriptor_legacy"',
                    ]
                )
                + "\n",
                encoding="utf-8",
            )
            _refresh_platform_native_plugins_payload(out, fixture["native_plugins"])

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "PlatformBundle report native_plugins_payload "
                    "loader_manifest plugin animation abi.descriptor_symbol "
                    "zircon_native_plugin_descriptor_legacy does not match "
                    "materialized package abi.descriptor_symbol "
                    "zircon_native_plugin_descriptor_v3"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_loader_manifest_mismatch(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            platform_report = _read_stage_report(out, "platform_bundle")
            payload = platform_report["native_plugins_payload"]
            self.assertIsInstance(payload, dict)
            bundle_path = payload["bundle_path"]
            self.assertIsInstance(bundle_path, str)
            forged_manifest = Path(bundle_path) / "forged_native_plugins.toml"
            forged_manifest.write_text(
                "[[plugins]]\nid = \"animation\"\n",
                encoding="utf-8",
            )
            payload["loader_manifest"] = str(forged_manifest)
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
                    "PlatformBundle report native_plugins_payload loader_manifest"
                    in diagnostic
                    and "does not match current bundle loader manifest"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)


def _refresh_platform_native_plugins_payload(
    out: Path,
    native_plugins: Path,
) -> None:
    platform_report = _read_stage_report(out, "platform_bundle")
    payload = platform_report["native_plugins_payload"]
    assert isinstance(payload, dict)
    file_manifest = _native_plugins_file_manifest(native_plugins)
    payload["file_manifest"] = file_manifest
    payload["file_count"] = len(file_manifest)
    payload["content_hash"] = _native_plugins_content_hash(file_manifest)
    _write_stage_report(out, "platform_bundle", platform_report)
    _write_bundle_manifest_from_platform_report(
        out / "bundle" / "windows-release" / "bundle.json",
        platform_report,
    )
