from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

from tools.zircon_export.native_dynamic_contract import (
    NATIVE_DYNAMIC_ABI_STRING_FIELDS,
    NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS,
)
from tools.zircon_export.pipeline_report import build_pipeline_report
from tools.zircon_export.tests.platform_bundle_native_payload_loader_manifest_test_support import (
    _refresh_platform_native_plugins_payload,
)
from tools.zircon_export.tests.platform_bundle_report_test_support import (
    _write_platform_bundle_fixture,
)


class PlatformBundleNativePayloadLoaderManifestAbiSchemaTests(unittest.TestCase):
    def test_report_rejects_native_plugins_payload_loader_manifest_bad_abi_table(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            loader_manifest = fixture["native_plugins"] / "native_plugins.toml"
            loader_manifest.write_text(
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
                encoding="utf-8",
            )
            _refresh_platform_native_plugins_payload(out, fixture["native_plugins"])

            report = build_pipeline_report(out, "windows-release")

            self.assertTrue(report["fatal"])
            self.assertEqual(report["missing_stages"], [])
            self.assertTrue(
                any(
                    "PlatformBundle report native_plugins_payload "
                    "loader_manifest plugins[0].abi must be a table"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_loader_manifest_unknown_abi_field(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            loader_manifest = fixture["native_plugins"] / "native_plugins.toml"
            loader_manifest.write_text(
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
                        "",
                        "[plugins.abi]",
                        'future_contract = "ignored"',
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
                    "loader_manifest plugin animation abi.future_contract "
                    "is not supported by materialized package"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_loader_manifest_abi_field_types(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            loader_manifest = fixture["native_plugins"] / "native_plugins.toml"
            loader_manifest.write_text(
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
                        "",
                        "[plugins.abi]",
                        'abi_version = "3"',
                        "descriptor_symbol = 42",
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
                    "loader_manifest plugin animation "
                    "abi.abi_version must be an integer"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertTrue(
                any(
                    "PlatformBundle report native_plugins_payload "
                    "loader_manifest plugin animation "
                    "abi.descriptor_symbol must be a string"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_loader_manifest_abi_blank_strings(
        self,
    ) -> None:
        for field in NATIVE_DYNAMIC_ABI_STRING_FIELDS:
            with self.subTest(field=field):
                with tempfile.TemporaryDirectory() as temp_dir:
                    out = Path(temp_dir) / "out"
                    fixture = _write_platform_bundle_fixture(out)
                    loader_manifest = fixture["native_plugins"] / "native_plugins.toml"
                    loader_manifest.write_text(
                        (
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
                            + "\n\n[plugins.abi]\n"
                            + "abi_version = 3\n"
                            + "\n".join(
                                (
                                    f'{abi_field} = "   "'
                                    if abi_field == field
                                    else f'{abi_field} = "{abi_value}"'
                                )
                                for abi_field, abi_value in (
                                    NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS.items()
                                )
                            )
                            + "\n"
                        ),
                        encoding="utf-8",
                    )
                    _refresh_platform_native_plugins_payload(
                        out,
                        fixture["native_plugins"],
                    )

                    report = build_pipeline_report(out, "windows-release")

                    self.assertTrue(report["fatal"])
                    self.assertEqual(report["missing_stages"], [])
                    self.assertTrue(
                        any(
                            "PlatformBundle report native_plugins_payload "
                            "loader_manifest plugin animation "
                            f"abi.{field} must be a non-empty string"
                            in diagnostic
                            for diagnostic in report["diagnostics"]
                        ),
                        report["diagnostics"],
                    )
                    self.assertNotIn("native_plugins_payload", report)

    def test_report_rejects_native_plugins_payload_loader_manifest_abi_missing_required_field(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            out = Path(temp_dir) / "out"
            fixture = _write_platform_bundle_fixture(out)
            loader_manifest = fixture["native_plugins"] / "native_plugins.toml"
            loader_manifest.write_text(
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
                        "",
                        "[plugins.abi]",
                        "abi_version = 3",
                        'descriptor_symbol = "zircon_native_plugin_descriptor_v3"',
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
                    "loader_manifest plugin animation "
                    "abi.descriptor_contract is required when abi is present"
                    in diagnostic
                    for diagnostic in report["diagnostics"]
                ),
                report["diagnostics"],
            )
            self.assertNotIn("native_plugins_payload", report)


if __name__ == "__main__":
    unittest.main()
