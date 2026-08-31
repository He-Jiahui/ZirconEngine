from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


class Editor09RuntimeAssetIndexProjectionContract(unittest.TestCase):
    def read(self, relative: str) -> str:
        return (ROOT / relative).read_text(encoding="utf-8")

    def test_editor_asset_module_exports_the_runtime_projection(self) -> None:
        source = self.read("zircon_editor/src/core/asset/mod.rs")
        self.assertIn("mod index;", source)
        self.assertIn("pub use index::", source)
        self.assertIn("EditorAssetIndex", source)
        self.assertIn("EditorAssetRow", source)

    def test_projection_retains_shared_authority_instead_of_copying_registry_rows(self) -> None:
        source = self.read("zircon_editor/src/core/asset/index.rs")
        self.assertIn("Arc<AssetRegistryIndex>", source)
        self.assertIn("Arc<AssetMetaDocument>", source)
        self.assertIn("runtime_entry", source)
        self.assertNotIn("HashMap<AssetUri, AssetRegistryEntry>", source)
        self.assertNotIn("HashMap<AssetUuid, AssetRegistryEntry>", source)

    def test_runtime_project_factory_keeps_the_registry_as_one_shared_generation(self) -> None:
        manager = self.read("zircon_runtime/src/asset/project/manager/mod.rs")
        access = self.read("zircon_runtime/src/asset/project/manager/registry_access.rs")
        index = self.read("zircon_editor/src/core/asset/index.rs")

        self.assertIn("asset_registry: Arc<AssetRegistryIndex>", manager)
        self.assertIn("fn asset_registry_shared(&self) -> Arc<AssetRegistryIndex>", access)
        self.assertIn("Arc::clone(&self.asset_registry)", access)
        self.assertIn("project.asset_registry_shared()", index)
        self.assertNotIn("Arc::new(project.asset_registry().clone())", index)

    def test_row_authoritative_fields_delegate_to_the_runtime_entry(self) -> None:
        source = self.read("zircon_editor/src/core/asset/index.rs")
        for accessor in (
            "self.runtime_entry.uuid()",
            "self.runtime_entry.path()",
            "self.runtime_entry.type_marker()",
            "self.runtime_entry.tags()",
            "self.runtime_entry.dependencies()",
            "self.runtime_entry.source_digest()",
        ):
            self.assertIn(accessor, source)

    def test_watch_events_resolve_through_runtime_registry_without_file_io(self) -> None:
        source = self.read("zircon_editor/src/core/asset/index.rs")
        self.assertIn("AssetWatchEvent", source)
        self.assertIn("entry_by_path", source)
        self.assertIn("pending_dirty_paths", source)
        self.assertNotIn("std::fs", source)
        self.assertNotIn("AssetMetaDocument::load", source)

    def test_import_validity_uses_persisted_artifacts_not_thumbnail_state(self) -> None:
        source = self.read("zircon_editor/src/core/asset/index.rs")
        self.assertIn("artifact_locator", source)
        self.assertNotIn("PreviewState", source)

    def test_document_refresh_uses_reverse_membership_without_full_index_scan(self) -> None:
        source = self.read("zircon_editor/src/core/asset/index.rs")
        ingest = source.split("pub fn ingest_meta_document", 1)[1].split(
            "pub fn apply_watch_events", 1
        )[0]
        self.assertIn("document_members", ingest)
        self.assertNotIn("metadata_by_uuid.retain", ingest)

    def test_regressions_cover_authority_atomicity_state_and_snapshot_reconciliation(self) -> None:
        source = self.read("zircon_editor/src/core/asset/index/tests.rs")
        for test_name in (
            "rows_borrow_runtime_registry_authority_and_meta_v7_projection",
            "watch_events_mark_only_touched_runtime_entries_dirty",
            "runtime_snapshot_replacement_resolves_pending_added_paths",
            "metadata_mismatch_is_typed_and_atomic",
            "import_state_precedence_is_deterministic",
            "removed_and_renamed_unknown_paths_cancel_pending_tombstones",
            "reingesting_a_document_removes_deleted_child_projections",
            "multi_entry_validation_failure_rolls_back_the_whole_document",
            "metadata_refresh_does_not_complete_an_active_import",
            "rows_are_path_sorted_for_reverse_registry_input",
        ):
            self.assertIn(test_name, source)

    def test_public_integration_gate_uses_only_exported_contracts(self) -> None:
        source = self.read("zircon_editor/tests/editor_asset_index_projection.rs")
        self.assertIn(
            "public_editor_asset_index_projects_runtime_registry_and_meta_v7", source
        )
        self.assertIn(
            "public_editor_asset_index_reconciles_watch_events_against_replaced_runtime_snapshot",
            source,
        )

if __name__ == "__main__":
    unittest.main()
