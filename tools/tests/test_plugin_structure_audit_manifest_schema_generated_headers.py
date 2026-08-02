import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from tools.plugin_structure_audits.manifest_schema import (
    PLUGIN_DECLARATION_GENERATED_MANIFEST_HEADER,
    audit_plugin_manifest_schema,
)


class PluginStructureAuditManifestSchemaGeneratedHeadersTests(unittest.TestCase):
    def test_plugin_declaration_header_counts_as_generated_manifest(self):
        audit = audit_single_native_manifest(
            PLUGIN_DECLARATION_GENERATED_MANIFEST_HEADER
        )

        self.assertEqual([], audit.generated_manifest_header_violation_paths)
        self.assertEqual(1, audit.to_json()["generated_manifest_count"])
        self.assertEqual(0, audit.to_json()["hand_written_native_manifest_count"])

    def test_legacy_native_sdk_header_is_rejected(self):
        audit = audit_single_native_manifest(
            "# @generated from zircon_plugin_sdk::native_plugin_manifest_v3!; do not edit by hand."
        )

        self.assertEqual(
            ["zircon_plugins/native_dynamic_fixture/plugin.toml"],
            audit.generated_manifest_header_violation_paths,
        )

    def test_legacy_descriptor_header_is_rejected(self):
        audit = audit_single_native_manifest(
            "# @generated from Rust descriptor package_manifest(); do not edit by hand."
        )

        self.assertEqual(
            ["zircon_plugins/native_dynamic_fixture/plugin.toml"],
            audit.generated_manifest_header_violation_paths,
        )

    def test_native_manifest_without_generated_header_is_rejected(self):
        audit = audit_single_native_manifest("")

        self.assertEqual(
            ["zircon_plugins/native_dynamic_fixture/plugin.toml"],
            audit.generated_manifest_header_violation_paths,
        )


def audit_single_native_manifest(header: str):
    with tempfile.TemporaryDirectory() as temporary_directory:
        repo_root = Path(temporary_directory)
        manifest_path = (
            repo_root
            / "zircon_plugins"
            / "native_dynamic_fixture"
            / "plugin.toml"
        )
        manifest_path.parent.mkdir(parents=True)
        manifest_path.write_text(
            "\n".join(
                [
                    header,
                    'id = "native_dynamic_fixture"',
                    'version = "0.1.0"',
                    'sdk_api_version = "0.1.0"',
                    'display_name = "Native Dynamic Fixture"',
                    'category = "sdk"',
                    'description = "Fixture"',
                    'supported_targets = ["client_runtime"]',
                    'supported_platforms = ["windows"]',
                    'capabilities = ["runtime.plugin.native_dynamic_fixture"]',
                    'maturity = "experimental"',
                    'default_packaging = ["native_dynamic"]',
                    "",
                ]
            ),
            encoding="utf-8",
        )
        with patch(
            "tools.plugin_structure_audits.manifest_schema.expected_plugin_manifest_roots",
            return_value=["native_dynamic_fixture"],
        ):
            return audit_plugin_manifest_schema(repo_root)


if __name__ == "__main__":
    unittest.main()
