from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
TARGET_ROWS = (
    ROOT
    / "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/build_export/target_rows"
)
CACHE = (
    ROOT
    / "zircon_editor/src/ui/retained_host/app/build_export_projection/cache.rs"
)
PROJECTION = ROOT / "zircon_editor/src/ui/retained_host/app/build_export_projection.rs"
WIZARD_PANEL = ROOT / (
    "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/"
    "build_export_wizard_panel.rs"
)


class EditorBuildExportProjectionPerformanceContractTests(unittest.TestCase):
    def test_base_projection_cache_publishes_one_shared_generation(self) -> None:
        cache = CACHE.read_text(encoding="utf-8")
        projection = PROJECTION.read_text(encoding="utf-8")

        self.assertIn("projection: Arc<BuildExportBaseProjection>", cache)
        self.assertIn("BuildExportBaseLookup::Hit", projection)
        self.assertIn("Arc::clone(&cached.projection)", cache)
        self.assertNotIn("cached.projection.clone()", cache)
        self.assertNotIn("base.clone()", projection)
        self.assertIn(
            "cached_base_reuses_the_same_projection_allocation",
            cache,
        )
        self.assertIn("Arc::ptr_eq", cache)

    def test_base_cache_hit_reads_a_watcher_epoch_without_filesystem_probes(self) -> None:
        cache = CACHE.read_text(encoding="utf-8")
        lookup = cache.split("fn lookup_base(", 1)[1].split("fn store_base(", 1)[0]
        compact = "".join(cache.split())

        self.assertIn("BuildExportSourceWatch", cache)
        self.assertIn("AtomicU64", cache)
        self.assertIn("source_generation", lookup)
        self.assertNotIn("capture_source_identity", cache)
        self.assertNotIn("file_metadata_identity", cache)
        self.assertNotIn("std::fs::metadata", lookup)
        self.assertNotIn("read_dir", lookup)
        self.assertNotIn("start_with_generation", cache)
        self.assertIn("self.watcher.watch(&self.export_directory", compact)
        self.assertIn("self.watcher.unwatch(&self.export_directory)", compact)

    def test_base_publication_rejects_a_generation_changed_during_build(self) -> None:
        cache = CACHE.read_text(encoding="utf-8")
        projection = PROJECTION.read_text(encoding="utf-8")
        store = cache.split("fn store_base(", 1)[1].split(
            "fn cached_rendered(", 1
        )[0]

        self.assertIn("BuildExportBaseLookup::Miss", projection)
        self.assertIn("BuildExportBaseBuildToken", cache)
        self.assertIn("token.source_generation", store)
        self.assertIn("watch.source_generation()", store)
        self.assertIn(
            "source_generation == token.source_generation", store
        )
        self.assertIn("!source_generation_unchanged", store)
        self.assertIn("preset_write_advances_the_watcher_generation", cache)
        self.assertIn(
            "created_export_directory_is_watched_for_followup_preset_changes",
            cache,
        )

    def test_wizard_projection_borrows_the_published_view_model(self) -> None:
        source = WIZARD_PANEL.read_text(encoding="utf-8")
        selector = source.split(
            "fn build_export_wizard_panel_view_model", 1
        )[1].split("fn build_export_wizard_panel_first_target", 1)[0]

        self.assertIn("Cow<'_, ExportWizardPanelViewModel>", selector)
        self.assertIn("data.wizard_view_model.as_ref()", selector)
        self.assertIn("Cow::Borrowed(view_model)", selector)
        self.assertIn("Cow::Owned", selector)
        self.assertNotIn("data.wizard_view_model.clone()", selector)
        self.assertIn(
            "published_wizard_view_model_is_borrowed_without_a_payload_clone",
            source,
        )
        self.assertIn(
            "missing_wizard_view_model_constructs_an_owned_fallback",
            source,
        )

    def test_wizard_projection_consumes_owned_node_payloads(self) -> None:
        source = WIZARD_PANEL.read_text(encoding="utf-8")
        projection = source.split(
            "fn retained_projection_template_nodes", 1
        )[1].split("fn build_export_wizard_panel_action_id", 1)[0]
        compact = "".join(projection.split())

        self.assertIn("projection.nodes.into_iter()", compact)
        self.assertIn("mut node: RetainedUiHostNodeModel", projection)
        self.assertIn("node.control_id.take()?", projection)
        self.assertIn("node.routes.into_iter().next()", projection)
        self.assertNotIn("node.options.clone()", projection)
        self.assertNotIn("node.collection_items.clone()", projection)
        self.assertNotIn("node.menu_items.clone()", projection)

    def test_platform_identity_is_normalized_once_per_target(self) -> None:
        source = (TARGET_ROWS / "mod.rs").read_text(encoding="utf-8")
        function = source.split("fn build_export_target_row_nodes", 1)[1].split(
            "fn build_export_target_list_frame", 1
        )[0]

        self.assertEqual(
            function.count("build_export_key(target.platform.as_str())"), 1
        )
        self.assertIn("targets_with_platform_id", function)

    def test_four_row_actions_use_fixed_cardinality_storage(self) -> None:
        source = (TARGET_ROWS / "actions.rs").read_text(encoding="utf-8")
        function = source.split("fn build_export_row_actions", 1)[1].split(
            "fn build_export_action_button_nodes", 1
        )[0]

        self.assertIn(") -> [BuildExportRowAction; BUILD_EXPORT_ACTION_COUNT]", function)
        self.assertNotIn("vec![", function)

    def test_target_and_row_node_vectors_reserve_known_node_count(self) -> None:
        module_source = (TARGET_ROWS / "mod.rs").read_text(encoding="utf-8")
        row_source = (TARGET_ROWS / "row.rs").read_text(encoding="utf-8")
        compact_module_source = "".join(module_source.split())

        self.assertIn(
            "targets_with_platform_id.len()"
            ".saturating_mul(BUILD_EXPORT_NODES_PER_TARGET)",
            compact_module_source,
        )
        self.assertIn(
            "Vec::with_capacity(BUILD_EXPORT_NODES_PER_TARGET)", row_source
        )


if __name__ == "__main__":
    unittest.main()
