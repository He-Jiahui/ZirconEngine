"""Static contract tests for Editor12 plugin template source roots."""

from __future__ import annotations

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
EXTENSIONS = ROOT / "zircon_editor" / "src" / "core" / "editor_extension.rs"
TEMPLATE_CONTRIBUTIONS = EXTENSIONS.parent / "editor_extension" / "template_contributions.rs"
NATIVE_MANAGER = (
    ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "host"
    / "editor_manager_plugins_export"
    / "native_registration"
    / "manager.rs"
)


class PluginTemplateSourceRootContractTests(unittest.TestCase):
    def test_template_descriptor_keeps_host_local_root_out_of_the_serialized_abi(
        self,
    ) -> None:
        extension = EXTENSIONS.read_text(encoding="utf-8")
        source = TEMPLATE_CONTRIBUTIONS.read_text(encoding="utf-8")

        self.assertIn("mod template_contributions;", extension)
        self.assertIn("pub use template_contributions::EditorUiTemplateDescriptor;", extension)
        self.assertIn("plugin_root: Option<PathBuf>", source)
        self.assertIn("#[serde(skip)]", source)
        self.assertIn("bind_ui_template_root", source)
        self.assertIn("plugin_root(&self)", source)
        self.assertIn("plugin_template_roots_are_host_local_and_not_serialized", source)

    def test_native_projection_derives_template_roots_from_discovered_manifest_paths(
        self,
    ) -> None:
        source = NATIVE_MANAGER.read_text(encoding="utf-8")

        self.assertRegex(source, r"native_report\s*\.discovered")
        self.assertRegex(source, r"manifest_path\s*\.parent\(\)")
        self.assertIn("bind_ui_template_root", source)
        self.assertNotIn("plugins://navigation", source)


if __name__ == "__main__":
    unittest.main()
