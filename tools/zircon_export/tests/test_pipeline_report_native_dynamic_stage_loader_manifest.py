from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.native_dynamic_contract import (
    NATIVE_DYNAMIC_ABI_STRING_FIELDS,
    NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS,
)
from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.native_dynamic_stage_report_test_support import (
    loader_manifest_with_overrides,
    rewrite_loader_manifest,
    write_native_dynamic_reports,
)


class PipelineReportNativeDynamicStageLoaderManifestTests(unittest.TestCase):
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

    def test_report_stage_rejects_native_dynamic_loader_manifest_abi_padded_strings_before_mismatch(
        self,
    ) -> None:
        for field in NATIVE_DYNAMIC_ABI_STRING_FIELDS:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    native_report_path = write_native_dynamic_reports(out)
                    rewrite_loader_manifest(
                        native_report_path,
                        loader_manifest_with_overrides(
                            abi_overrides={
                                field: (
                                    f" {NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS[field]} "
                                )
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
                            f"abi.{field} must be a non-empty trimmed string"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )
                    self.assertFalse(
                        any(
                            (
                                "native_dynamic loader_manifest plugin "
                                f"animation abi.{field} "
                            )
                            in diagnostic
                            and (
                                "does not match native_dynamic report "
                                "package_exports"
                            )
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

    def test_report_stage_rejects_native_dynamic_loader_manifest_padded_string_fields_before_row_semantics(
        self,
    ) -> None:
        cases = (
            (
                "id",
                loader_manifest_with_overrides().replace(
                    'id = "animation"',
                    'id = " animation "',
                ),
                "native_dynamic loader_manifest plugins[0].id "
                "must be a non-empty trimmed string",
                "native_dynamic loader_manifest plugin ids",
            ),
            (
                "path",
                loader_manifest_with_overrides(path=" plugins/animation "),
                "native_dynamic loader_manifest plugins[0].path "
                "must be a non-empty trimmed string",
                "native_dynamic loader_manifest plugin animation path",
            ),
            (
                "manifest",
                loader_manifest_with_overrides(
                    manifest=" plugins/animation/plugin.toml "
                ),
                "native_dynamic loader_manifest plugins[0].manifest "
                "must be a non-empty trimmed string",
                "native_dynamic loader_manifest plugin animation manifest",
            ),
            (
                "package_report",
                loader_manifest_with_overrides(
                    package_report=(
                        " plugins/animation/native_dynamic_package.toml "
                    )
                ),
                "native_dynamic loader_manifest plugins[0].package_report "
                "must be a non-empty trimmed string",
                "native_dynamic loader_manifest plugin animation package_report",
            ),
        )
        for field, loader_manifest, expected_diagnostic, unexpected_diagnostic in cases:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    native_report_path = write_native_dynamic_reports(out)
                    rewrite_loader_manifest(native_report_path, loader_manifest)

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
                    self.assertFalse(
                        any(
                            unexpected_diagnostic in diagnostic
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


if __name__ == "__main__":
    unittest.main()