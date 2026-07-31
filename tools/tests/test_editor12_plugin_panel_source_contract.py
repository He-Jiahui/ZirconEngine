"""Static ownership checks for the Editor12 plugin panel read model."""

from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
PLUGIN = ROOT / "zircon_editor" / "src" / "core" / "plugin"


class Editor12PluginPanelSourceContractTests(unittest.TestCase):
    def source(self, relative: str) -> str:
        return (PLUGIN / relative).read_text(encoding="utf-8")

    def test_panel_source_is_exported_from_the_core_plugin_owner(self) -> None:
        plugin_module = self.source("mod.rs")
        panel_source = self.source("panel_source.rs")

        self.assertIn("mod panel_source;", plugin_module)
        self.assertIn("EditorPluginPanelRow", plugin_module)
        self.assertIn("EditorPluginPanelSource", plugin_module)
        self.assertIn("pub struct EditorPluginPanelSource", panel_source)
        self.assertIn("pub struct EditorPluginPanelRow", panel_source)

    def test_panel_source_keeps_one_manager_snapshot_per_read_operation(self) -> None:
        panel_source = self.source("panel_source.rs")

        self.assertIn("Arc<EditorPluginManagerSnapshot>", panel_source)
        self.assertIn(
            "pub fn from_manager(manager: &EditorPluginManager) -> Self", panel_source
        )
        self.assertIn(
            "pub fn from_snapshot(snapshot: Arc<EditorPluginManagerSnapshot>) -> Self",
            panel_source,
        )
        self.assertIn("pub fn generation(&self) -> u64", panel_source)
        self.assertIn("pub fn row(&self, package_id: &str)", panel_source)
        self.assertIn("pub fn rows(&self)", panel_source)

    def test_panel_rows_borrow_the_canonical_catalog_without_read_time_io(self) -> None:
        panel_source = self.source("panel_source.rs")

        self.assertIn("EditorPluginCatalogEntry", panel_source)
        self.assertIn("EditorPluginManagerEntry", panel_source)
        self.assertIn("capabilities_for_package", panel_source)
        self.assertNotIn("NativePluginLoader", panel_source)
        self.assertNotIn("ProjectManifest::load", panel_source)
        self.assertNotIn("load_discovered_all", panel_source)
        self.assertNotIn("Vec<EditorPluginPanelRow>", panel_source)

    def test_panel_rows_zip_one_generation_without_per_row_binary_search(self) -> None:
        panel_source = self.source("panel_source.rs")
        rows = panel_source.split("pub fn rows(&self)", 1)[1].split("}\n}\n", 1)[0]

        self.assertIn(".zip(projections.iter())", rows)
        self.assertIn("entries.len()", rows)
        self.assertIn("projections.len()", rows)
        self.assertNotIn("self.row(entry.package_id())", rows)

    def test_selected_panel_details_borrow_the_generation_registration_report(self) -> None:
        snapshot = self.source("catalog_snapshot.rs")
        panel_source = self.source("panel_source.rs")

        self.assertIn("registration_index", snapshot)
        self.assertIn("pub fn registration(&self, package_id: &str)", snapshot)
        self.assertIn("EditorPluginRegistrationReport", snapshot)
        self.assertIn("pub fn registration(&self, package_id: &str)", panel_source)
        self.assertIn("pub fn diagnostics(&self) -> &[String]", panel_source)
        self.assertNotIn("EditorPluginRegistrationReport::clone", panel_source)


if __name__ == "__main__":
    unittest.main()
