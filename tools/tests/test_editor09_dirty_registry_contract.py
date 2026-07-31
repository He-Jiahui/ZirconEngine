import pathlib
import re
import unittest


REPO_ROOT = pathlib.Path(__file__).resolve().parents[2]
DIRTY_ROOT = REPO_ROOT / "zircon_editor/src/core/asset/dirty"


def read(relative: str) -> str:
    return (REPO_ROOT / relative).read_text(encoding="utf-8")


def public_reexports(source: str, module: str) -> set[str]:
    match = re.search(
        rf"pub use {module}::(?P<body>\{{.*?\}}|[A-Za-z0-9_]+);",
        source,
        re.DOTALL,
    )
    if match is None:
        return set()
    body = match.group("body").strip("{}")
    return {token.strip() for token in body.split(",") if token.strip()}


class Editor09DirtyRegistryContractTests(unittest.TestCase):
    def test_asset_facade_mounts_and_reexports_dirty_registry(self) -> None:
        facade = read("zircon_editor/src/core/asset/mod.rs")

        self.assertIn("mod dirty;", facade)
        self.assertNotIn("pub mod dirty;", facade)
        self.assertEqual(
            public_reexports(facade, "dirty"),
            {
                "DirtyDocumentSnapshot",
                "DirtyRegistryCursor",
                "DirtyRegistryDelta",
                "DirtyExternalEffectId",
                "DirtyExternalEffectIdError",
                "DirtyExternalEffectRevision",
                "DirtyRegistry",
                "DirtyRegistryError",
            },
        )

        owner = read("zircon_editor/src/core/asset/dirty/mod.rs")
        self.assertEqual(
            public_reexports(owner, "error"), {"DirtyRegistryError"}
        )
        self.assertEqual(
            public_reexports(owner, "external_effect_id"),
            {"DirtyExternalEffectId", "DirtyExternalEffectIdError"},
        )
        self.assertEqual(
            public_reexports(owner, "registry"),
            {
                "DirtyDocumentSnapshot",
                "DirtyExternalEffectRevision",
                "DirtyRegistry",
                "DirtyRegistryCursor",
                "DirtyRegistryDelta",
            },
        )

        consumer = read("zircon_editor/tests/editor_asset_facade.rs")
        self.assertIn("use zircon_editor::core::asset::{", consumer)
        self.assertIn("DirtyDocumentSnapshot", consumer)
        self.assertIn("DirtyRegistry", consumer)

    def test_owner_is_folder_backed_and_responsibility_split(self) -> None:
        for name in ("mod.rs", "error.rs", "external_effect_id.rs", "registry.rs", "tests.rs"):
            self.assertTrue((DIRTY_ROOT / name).is_file(), name)
        facade = read("zircon_editor/src/core/asset/dirty/mod.rs")
        self.assertIn("mod error;", facade)
        self.assertIn("mod external_effect_id;", facade)
        self.assertIn("mod registry;", facade)
        self.assertLess(len(facade.splitlines()), 80)

    def test_transaction_dirty_is_editor03_delta_projection_not_cached_boolean(self) -> None:
        registry = read("zircon_editor/src/core/asset/dirty/registry.rs")
        self.assertIn("HistoryContextId::Document", registry)
        self.assertIn("dirty_states_since", registry)
        self.assertIn("HistoryDirtyCursor", registry)
        self.assertIn("HistoryDirtyBatchKind", registry)
        self.assertNotIn("transaction_dirty: BTreeMap", registry)
        self.assertNotIn("transaction_dirty: HashMap", registry)
        self.assertNotIn("mark_saved(", registry)

    def test_multi_document_projection_is_cursor_delta_not_full_clone_retry(self) -> None:
        registry = read("zircon_editor/src/core/asset/dirty/registry.rs")

        self.assertIn("pub fn changes_since", registry)
        self.assertIn("DirtyRegistryCursor", registry)
        self.assertIn("DirtyRegistryDelta", registry)
        self.assertIn("VecDeque", registry)
        self.assertIn("change_start_after", registry)
        self.assertIn("changed_documents_after", registry)
        self.assertIn(".range(start..)", registry)
        self.assertNotIn("journal_visits += visits", registry)
        self.assertIn("journal_visits", registry)
        registry_owner = registry.split("impl DirtyRegistry {", 1)[1]
        self.assertNotIn("pub fn snapshots(&self) -> Result<Vec", registry_owner)
        self.assertNotIn("pub fn dirty_snapshots(", registry_owner)
        self.assertNotIn("SnapshotSetUnstable", registry)
        self.assertNotIn("collect::<Vec<_>>()", registry)

    def test_external_effect_owner_is_typed_revisioned_and_deterministic(self) -> None:
        effect = read("zircon_editor/src/core/asset/dirty/external_effect_id.rs")
        registry = read("zircon_editor/src/core/asset/dirty/registry.rs")
        self.assertIn("DirtyExternalEffectId", effect)
        self.assertIn("DirtyExternalEffectIdError", effect)
        self.assertIn("DirtyExternalEffectRevision", registry)
        self.assertIn("BTreeMap<DocumentId", registry)
        self.assertIn("BTreeMap<DirtyExternalEffectId", registry)

    def test_registry_never_writes_asset_sidecar_or_creates_save_baseline(self) -> None:
        combined = "\n".join(
            read(f"zircon_editor/src/core/asset/dirty/{name}")
            for name in ("mod.rs", "error.rs", "external_effect_id.rs", "registry.rs")
        )
        for forbidden in (
            "std::fs::",
            ".zmeta",
            "saved_top:",
            "source_digest",
            "AssetRegistryIndex",
            "DocumentToolkit",
        ):
            self.assertNotIn(forbidden, combined)

    def test_errors_are_typed_and_unknown_documents_do_not_auto_register(self) -> None:
        error = read("zircon_editor/src/core/asset/dirty/error.rs")
        registry = read("zircon_editor/src/core/asset/dirty/registry.rs")
        self.assertIn("DirtyRegistryError", error)
        self.assertIn("DocumentNotRegistered", error)
        self.assertIn("EditCommandError", error)
        self.assertIn("require_document", registry)

    def test_rust_contract_covers_saved_top_external_effects_and_lifecycle(self) -> None:
        tests = read("zircon_editor/src/core/asset/dirty/tests.rs")
        for test_name in (
            "saved_top_is_queried_live_without_registry_dirty_mutations",
            "external_effects_are_typed_sorted_and_independently_clearable",
            "remarking_an_effect_advances_its_revision",
            "unknown_documents_are_rejected_without_implicit_registration",
            "unregister_discards_only_that_documents_external_effects",
            "initial_dirty_delta_is_sorted_and_combines_both_sources",
            "stable_cursor_returns_no_snapshots_or_removals",
            "dirty_delta_updates_only_changed_documents",
            "unregister_is_published_as_a_typed_removal",
            "ten_thousand_document_stable_and_single_change_work_is_delta_bounded",
            "cursor_from_another_registry_is_rejected",
            "snapshot_retries_external_generation_change_instead_of_returning_false_clean",
            "dirty_delta_retries_only_changed_documents_when_external_generation_moves",
        ):
            self.assertIn(f"fn {test_name}", tests)


if __name__ == "__main__":
    unittest.main()
