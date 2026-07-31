from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
EDITOR = ROOT / "zircon_editor" / "src" / "core"


class EditorPluginCatalogProjectionContractTests(unittest.TestCase):
    def source(self, relative: str) -> str:
        return (EDITOR / relative).read_text(encoding="utf-8")

    def test_builtin_catalog_projection_has_one_shared_generation_owner(self) -> None:
        snapshot = self.source("plugin/catalog_snapshot.rs")
        manager = self.source("plugin/manager.rs")

        self.assertIn(
            "projection: Arc<EditorPluginCatalogProjection>",
            snapshot,
        )
        self.assertIn(
            "Arc::new(EditorPluginCatalogProjection::from_registrations(",
            snapshot,
        )
        self.assertIn(
            "static BUILTIN_EDITOR_PLUGIN_MANAGER: OnceLock<EditorPluginManager>",
            manager,
        )
        self.assertIn(
            "pub fn builtin_shared() -> Result<&'static Self, EditorPluginDiscoveryError>",
            manager,
        )
        self.assertIn("BUILTIN_EDITOR_PLUGIN_MANAGER_INIT", manager)
        self.assertIn("initialize_once(", manager)
        self.assertIn(
            "Self::builtin(RuntimePluginCatalog::builtin().package_manifests().cloned())",
            manager,
        )

    def test_plugin_core_is_a_hard_cutover_folder_without_legacy_root_modules(self) -> None:
        core_mod = self.source("mod.rs")

        self.assertIn("pub mod plugin;", core_mod)
        self.assertNotIn("pub mod editor_plugin;", core_mod)
        self.assertNotIn("editor_plugin_catalog_gen", core_mod)
        self.assertFalse((EDITOR / "editor_plugin.rs").exists())
        self.assertFalse((EDITOR / "editor_plugin_catalog_gen.rs").exists())
        self.assertFalse((EDITOR / "editor_plugin_sdk").exists())

    def test_plugin_list_retains_the_canonical_projection_arc_and_identity_regression(self) -> None:
        runner = self.source("commandlet/runner.rs")
        tests = self.source("commandlet/tests.rs")

        self.assertIn("crate::core::plugin::{", runner)
        self.assertIn("Option<Arc<EditorPluginCatalogProjection>>", runner)
        self.assertIn("plugin_catalog_projection", runner)
        self.assertIn("impl Serialize for CommandletReport", runner)
        self.assertIn(
            "plugin_list_reuses_the_canonical_catalog_projection_without_rebuild",
            tests,
        )
        self.assertIn("Arc::ptr_eq", tests)


if __name__ == "__main__":
    unittest.main()
