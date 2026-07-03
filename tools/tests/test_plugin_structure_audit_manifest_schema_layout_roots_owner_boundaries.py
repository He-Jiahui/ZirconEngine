import unittest
from pathlib import Path

from tools.plugin_structure_audits.manifest_schema import (
    collect_manifest_schema_violations,
)


class PluginStructureAuditManifestSchemaLayoutRootsOwnerBoundaryTests(
    unittest.TestCase
):
    def test_manifest_schema_rejects_layout_root_drive_separator(self):
        violations: list[str] = []
        manifest = plugin_manifest()
        manifest["asset_roots"] = ["C:/assets"]
        manifest["content_roots"] = ["D:/content"]

        collect_manifest_schema_violations(
            "zircon_plugins/sound/plugin.toml",
            manifest,
            violations,
        )

        self.assertEqual(
            [
                "zircon_plugins/sound/plugin.toml: asset_roots[0] C:/assets must not contain a drive separator",
                "zircon_plugins/sound/plugin.toml: content_roots[0] D:/content must not contain a drive separator",
            ],
            violations,
        )

    def test_layout_roots_schema_lives_in_layout_roots_owner(self):
        repo_root = Path(__file__).resolve().parents[2]
        manifest_schema_path = (
            repo_root / "tools/plugin_structure_audits/manifest_schema.py"
        )
        owner_path = (
            repo_root / "tools/plugin_structure_audits/manifest_schema_layout_roots.py"
        )

        self.assertTrue(owner_path.exists())
        manifest_schema_text = manifest_schema_path.read_text(encoding="utf-8")
        owner_text = owner_path.read_text(encoding="utf-8")

        self.assertIn(
            "from .manifest_schema_layout_roots import",
            manifest_schema_text,
        )
        self.assertIn(
            "collect_layout_root_schema_violations",
            manifest_schema_text,
        )
        self.assertNotIn("LAYOUT_ROOT_FIELDS =", manifest_schema_text)
        self.assertNotIn(
            "def collect_layout_root_schema_violations",
            manifest_schema_text,
        )
        self.assertNotIn(
            "def collect_layout_root_path_violations",
            manifest_schema_text,
        )
        self.assertNotIn("def layout_root_has_drive_separator", manifest_schema_text)

        self.assertIn("LAYOUT_ROOT_FIELDS =", owner_text)
        self.assertIn("def collect_layout_root_schema_violations", owner_text)
        self.assertIn("def collect_layout_root_path_violations", owner_text)
        self.assertIn("def layout_root_has_drive_separator", owner_text)


def plugin_manifest() -> dict[str, object]:
    return {
        "id": "sound",
        "version": "0.1.0",
        "sdk_api_version": "0.1.0",
        "display_name": "Sound",
        "category": "runtime",
        "description": "Sound plugin.",
        "supported_targets": ["client_runtime"],
        "supported_platforms": ["windows"],
        "capabilities": ["runtime.plugin.sound"],
        "maturity": "experimental",
        "default_packaging": ["source_template", "library_embed", "native_dynamic"],
        "modules": [
            {
                "name": "sound.runtime",
                "kind": "runtime",
                "crate_name": "zircon_plugin_sound_runtime",
                "target_modes": ["client_runtime"],
                "capabilities": ["runtime.plugin.sound"],
            }
        ],
    }


if __name__ == "__main__":
    unittest.main()
