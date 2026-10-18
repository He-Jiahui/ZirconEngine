"""Cargo invalidation contract: visual assets are loaded and watched at runtime."""

from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
BUILD = ROOT / "zircon_editor/build.rs"
EMBEDDED_OWNERS = (
    "core/commands/keymap.rs",
    "core/i18n/catalog.rs",
    "ui/runtime/builtin/component_descriptors.rs",
    "ui/operator/roles.rs",
    "ui/retained/dispatch/templates/workbench/main_menu_items.rs",
)


class EditorBuildInputsTests(unittest.TestCase):
    def test_visual_edits_do_not_invalidate_the_build_script(self):
        source = BUILD.read_text(encoding="utf-8")
        self.assertNotIn('emit_rerun_if_changed_recursive("assets")', source)
        self.assertNotIn("visit_rerun_if_changed", source)
        paths = re.findall(r'"(assets/[^"\n]+)"', source)
        self.assertGreater(len(paths), 0)
        for relative in paths:
            if "{" not in relative:
                self.assertTrue((BUILD.parent / relative).is_file(), relative)
                self.assertNotEqual(Path(relative).suffix, ".zui")

    def test_embedded_production_resources_remain_explicit_inputs(self):
        source = BUILD.read_text(encoding="utf-8")
        tracked = set(re.findall(r'"(assets/[^"\n]+)"', source))
        required = set()
        for owner in EMBEDDED_OWNERS:
            path = ROOT / "zircon_editor/src" / owner
            text = path.read_text(encoding="utf-8")
            for include in re.findall(r'include_(?:str|bytes)!\(\s*"([^"]+)"', text):
                asset = (path.parent / include).resolve()
                required.add(asset.relative_to(BUILD.parent).as_posix())
            for asset in re.findall(r'"/(assets/[^"\n]+)"', text):
                required.add(asset)
        self.assertGreaterEqual(len(required), 6)
        self.assertTrue(required <= tracked, f"Untracked embedded inputs: {required - tracked}")

    def test_catalog_tracks_membership_and_manifests_instead_of_plugin_sources(self):
        source = BUILD.read_text(encoding="utf-8")
        self.assertNotIn('println!("cargo:rerun-if-changed={}", plugins_root.display())', source)
        self.assertIn('plugins_root.join("Cargo.toml")', source)
        self.assertIn('println!("cargo:rerun-if-changed={}", plugin_manifest.display())', source)
        self.assertIn('assets/viewport_gizmos/{filename}', source)


if __name__ == "__main__":
    unittest.main()
