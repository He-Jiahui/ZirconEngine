from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
CORE = ROOT / "zircon_editor" / "src" / "core" / "plugin"
HOST = ROOT / "zircon_editor" / "src" / "ui" / "host"
BUILD = ROOT / "zircon_editor" / "build.rs"


class EditorPluginCatalogStoreContractTests(unittest.TestCase):
    def source(self, root: Path, relative: str) -> str:
        return (root / relative).read_text(encoding="utf-8")

    def test_catalog_snapshot_exposes_borrowed_generation_indexes(self) -> None:
        snapshot = self.source(CORE, "catalog_snapshot.rs")

        self.assertIn("pub struct EditorPluginCatalogSnapshot", snapshot)
        self.assertIn("generation: u64", snapshot)
        self.assertIn("package_index", snapshot)
        self.assertIn("capabilities_by_package", snapshot)
        self.assertIn("packages_by_capability", snapshot)
        self.assertIn("pub fn package_manifests(&self) -> &[PluginPackageManifest]", snapshot)
        self.assertIn("pub fn capabilities_for_package(&self, package_id: &str) -> &[String]", snapshot)

    def test_catalog_snapshot_indexes_capabilities_back_to_sorted_packages(self) -> None:
        snapshot = self.source(CORE, "catalog_snapshot.rs")

        self.assertIn("packages_by_capability", snapshot)
        self.assertIn("pub fn packages_for_capability(&self, capability: &str) -> &[String]", snapshot)
        self.assertIn("package_ids.sort()", snapshot)
        self.assertIn("package_ids.dedup()", snapshot)

    def test_projection_is_owned_by_the_catalog_snapshot_generation(self) -> None:
        snapshot = self.source(CORE, "catalog_snapshot.rs")
        projection = self.source(CORE, "projection.rs")
        descriptor = self.source(CORE, "descriptor.rs")

        self.assertIn("projection: Arc<EditorPluginCatalogProjection>", snapshot)
        self.assertIn(
            "EditorPluginCatalogProjection::from_registrations", snapshot
        )
        self.assertIn("pub fn projection(&self) -> &Arc<EditorPluginCatalogProjection>", snapshot)
        self.assertIn("pub(super) fn from_registrations", projection)
        self.assertIn("registration.capabilities", projection)
        self.assertNotIn("OnceLock", projection)
        self.assertNotIn("builtin_catalog_projection", descriptor)

    def test_extension_catalog_is_materialized_only_for_the_active_manager_generation(self) -> None:
        snapshot = self.source(CORE, "catalog_snapshot.rs")
        manager = self.source(CORE, "manager.rs")
        manager_snapshot = self.source(CORE, "manager/snapshot.rs")

        self.assertNotIn("EditorExtensionCatalogReport", snapshot)
        self.assertNotIn("editor_extensions", snapshot)
        self.assertIn(
            "active_extensions: Arc<EditorExtensionCatalogReport>", manager_snapshot
        )
        self.assertIn("fn build_active_extensions", manager)
        self.assertIn("EditorPluginState::Active", manager)

    def test_raw_registration_extensions_are_not_a_public_catalog_read_surface(self) -> None:
        registration = self.source(CORE, "registration.rs")

        self.assertIn("pub(crate) extensions: EditorExtensionRegistry", registration)
        self.assertNotIn("pub extensions: EditorExtensionRegistry", registration)

    def test_catalog_store_publishes_one_arc_snapshot_per_generation(self) -> None:
        store = self.source(CORE, "catalog_store.rs")

        self.assertIn("pub(crate) struct EditorPluginCatalogStore", store)
        self.assertIn("RwLock<Arc<EditorPluginCatalogSnapshot>>", store)
        self.assertIn("pub(super) fn snapshot(&self) -> Arc<EditorPluginCatalogSnapshot>", store)
        self.assertIn(
            "EditorPluginCatalogSnapshot::from_catalog(\n                1, catalog,", store
        )
        self.assertNotIn("catalog.generation()", store)
        self.assertNotIn("fn publish(&self, catalog", store)
        self.assertIn("pub(super) fn publish_prepared(", store)

    def test_publish_reads_and_replaces_generation_under_one_write_lock(self) -> None:
        store = self.source(CORE, "catalog_store.rs")
        publish = store.split("pub(super) fn publish_prepared(", 1)[1]

        self.assertIn("let mut snapshot_slot = self", publish)
        self.assertIn(".snapshot", publish)
        self.assertIn(".write()", publish)
        self.assertIn("snapshot.generation(),", publish)
        self.assertIn("snapshot_slot.generation().saturating_add(1),", publish)
        self.assertIn("*snapshot_slot = Arc::clone(&snapshot);", publish)
        self.assertLess(
            publish.index("let mut snapshot_slot = self"),
            publish.index("snapshot.generation(),"),
        )
        self.assertLess(
            publish.index("snapshot.generation(),"),
            publish.index("*snapshot_slot = Arc::clone(&snapshot);"),
        )
        self.assertNotIn("let generation = self.snapshot()", publish)

    def test_core_manager_owns_store_and_ui_delegates_snapshot_reads(self) -> None:
        core_manager = self.source(CORE, "manager.rs")
        manager = self.source(HOST, "editor_manager.rs")
        exports = self.source(HOST, "editor_manager_plugins_export/mod.rs")
        status = self.source(HOST, "editor_manager_plugins_export/status/builtin.rs")

        self.assertIn("pub struct EditorPluginManager", core_manager)
        self.assertIn("catalog_store: EditorPluginCatalogStore", core_manager)
        self.assertIn("plugin_manager: EditorPluginManager", manager)
        self.assertIn("EditorPluginManager::builtin(", manager)
        self.assertNotIn("EditorPluginManager::builtin_shared()", manager)
        self.assertNotIn("plugin_catalog: EditorPluginCatalogStore", manager)
        self.assertIn("Arc<EditorPluginCatalogSnapshot>", exports)
        self.assertIn("self.plugin_manager.catalog_snapshot()", exports)
        self.assertNotIn("EditorPluginCatalog::builtin", exports)
        self.assertIn("editor_catalog.package_manifests()", status)

    def test_catalog_store_is_internal_to_the_manager_publication_boundary(self) -> None:
        plugin_module = self.source(CORE, "mod.rs")
        store = self.source(CORE, "catalog_store.rs")

        self.assertNotIn("pub use catalog_store::EditorPluginCatalogStore;", plugin_module)
        self.assertIn("pub(crate) struct EditorPluginCatalogStore", store)
        self.assertIn("pub(super) fn publish_prepared(", store)

    def test_ui_host_does_not_rebuild_the_builtin_editor_catalog(self) -> None:
        rebuild_paths = [
            source.relative_to(HOST).as_posix()
            for source in HOST.rglob("*.rs")
            if "EditorPluginCatalog::builtin" in source.read_text(encoding="utf-8")
        ]

        self.assertEqual(rebuild_paths, [])

    def test_generated_catalog_artifact_uses_the_plugin_owner_name(self) -> None:
        generated_adapter = self.source(CORE, "catalog_gen.rs")
        build_script = BUILD.read_text(encoding="utf-8")

        self.assertIn("plugin_catalog_generated.rs", generated_adapter)
        self.assertIn("plugin_catalog_generated.rs", build_script)
        self.assertNotIn("editor_plugin_catalog_gen.rs", generated_adapter)
        self.assertNotIn("editor_plugin_catalog_gen.rs", build_script)


if __name__ == "__main__":
    unittest.main()
