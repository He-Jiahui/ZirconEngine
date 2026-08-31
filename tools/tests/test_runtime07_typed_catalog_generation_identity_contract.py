from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
CATALOG_ROOT = (
    ROOT / "zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog"
)
CATALOG = ROOT / "zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog.rs"
GENERATION = CATALOG_ROOT / "generation.rs"
SNAPSHOT = CATALOG_ROOT / "snapshot.rs"
PUBLICATION = CATALOG_ROOT / "publication.rs"
REPORTS = CATALOG_ROOT / "registration/reports.rs"
UPDATE = CATALOG_ROOT / "registration/update.rs"
OUTCOME = CATALOG_ROOT / "registration/update/outcome.rs"
PROJECT = CATALOG_ROOT / "project.rs"
PROJECT_CACHE = CATALOG_ROOT / "project/cache.rs"
COMPOSITION = ROOT / "zircon_runtime/src/builtin/runtime_modules/composition/identity.rs"
RECEIPT = ROOT / "zircon_runtime/src/dynamic_api/session/composition_receipt.rs"
BRIDGE_LIFECYCLE = (
    CATALOG_ROOT / "bridge_lifecycle_state.rs"
)
LINKED_PLUGINS = ROOT / "zircon_runtime/src/dynamic_api/session/linked_plugins.rs"
SESSION_CONSTRUCTION = ROOT / "zircon_runtime/src/dynamic_api/session/construction.rs"
SESSION_STATE = ROOT / "zircon_runtime/src/dynamic_api/session/state.rs"


class TypedCatalogGenerationIdentityContractTests(unittest.TestCase):
    def test_generation_is_an_opaque_non_zero_value(self) -> None:
        source = GENERATION.read_text(encoding="utf-8")
        self.assertIn("pub struct PluginCatalogGeneration(NonZeroU64);", source)
        self.assertIn("pub const fn get(self) -> u64", source)
        self.assertIn("pub(super) fn checked_next(self)", source)
        self.assertIn("plugin_catalog_generation_keeps_one_word_layout", source)

    def test_catalog_has_one_typed_generation_owner_without_clone_branching(self) -> None:
        source = CATALOG.read_text(encoding="utf-8")
        self.assertIn("mod generation;", source)
        self.assertIn("pub use generation::PluginCatalogGeneration;", source)
        self.assertIn("catalog_generation: PluginCatalogGeneration", source)
        self.assertNotIn("impl Clone for RuntimePluginCatalog", source)

        snapshot = SNAPSHOT.read_text(encoding="utf-8")
        self.assertIn("use std::sync::Arc;", snapshot)
        self.assertIn("pub struct RuntimePluginCatalogSnapshot", snapshot)
        self.assertIn("catalog: RuntimePluginCatalog", snapshot)
        self.assertNotIn("clone_catalog", snapshot)
        self.assertNotIn("into_catalog", snapshot)

        lifecycle = BRIDGE_LIFECYCLE.read_text(encoding="utf-8")
        self.assertIn("snapshot: Arc<RuntimePluginCatalogSnapshot>", lifecycle)
        self.assertIn(
            "pub fn from_snapshot(snapshot: Arc<RuntimePluginCatalogSnapshot>)",
            lifecycle,
        )
        self.assertIn(
            "pub fn from_snapshot_and_extension_report(",
            lifecycle,
        )
        self.assertIn(
            "pub fn snapshot(&self) -> &Arc<RuntimePluginCatalogSnapshot>",
            lifecycle,
        )
        self.assertIn("Arc::ptr_eq(&snapshot, state.snapshot())", lifecycle)
        self.assertIn("Arc::ptr_eq(state.snapshot(), cloned.snapshot())", lifecycle)

    def test_public_generation_flow_stays_typed_until_the_abi_boundary(self) -> None:
        for path in [OUTCOME, PROJECT, PROJECT_CACHE, COMPOSITION]:
            source = path.read_text(encoding="utf-8")
            self.assertNotIn("catalog_generation: u64", source, path.as_posix())
        self.assertIn("catalog_generation.get()", RECEIPT.read_text(encoding="utf-8"))

    def test_publication_never_saturates_or_reuses_the_terminal_generation(self) -> None:
        for path in [REPORTS, UPDATE]:
            source = path.read_text(encoding="utf-8")
            self.assertNotIn("catalog_generation.saturating_add(1)", source)
        update = UPDATE.read_text(encoding="utf-8")
        self.assertIn("checked_next()", update)
        self.assertIn("generation_exhausted", update)

    def test_snapshot_publication_has_one_lock_free_compare_exchange_owner(self) -> None:
        snapshot = SNAPSHOT.read_text(encoding="utf-8")
        self.assertIn(
            "pub fn stage_update(self: &Arc<Self>) -> RuntimePluginCatalogCandidate",
            snapshot,
        )
        self.assertNotIn("pub fn stage_update(&self) -> RuntimePluginCatalog {", snapshot)

        candidate = (CATALOG_ROOT / "candidate.rs").read_text(encoding="utf-8")
        self.assertIn("pub struct RuntimePluginCatalogCandidate", candidate)
        self.assertIn("registrations: CandidateRows<", candidate)
        self.assertIn("pub fn prepare(", candidate)
        self.assertIn("published_generations: 0", candidate)

        publication = PUBLICATION.read_text(encoding="utf-8")
        self.assertIn("current: ArcSwap<RuntimePluginCatalogSnapshot>", publication)
        self.assertIn("pub struct RuntimePluginCatalogAuthority", publication)
        self.assertIn("pub fn snapshot(&self) -> Arc<RuntimePluginCatalogSnapshot>", publication)
        self.assertNotIn("pub fn from_snapshot(", publication)
        self.assertIn("prepared.into_publication_parts()", publication)
        self.assertIn(".compare_and_swap(&expected, Arc::clone(&candidate))", publication)
        self.assertNotIn("expected: &Arc<RuntimePluginCatalogSnapshot>", publication)
        self.assertIn("RuntimePluginCatalogPublicationError::Conflict", publication)
        self.assertIn("stale_publisher_cannot_replace_the_current_snapshot", publication)

        linked = LINKED_PLUGINS.read_text(encoding="utf-8")
        self.assertIn("runtime_plugin_catalog_snapshot: Arc<RuntimePluginCatalogSnapshot>", linked)
        self.assertIn("RuntimePluginCatalogSnapshot::from_catalog(catalog)", linked)
        self.assertIn("compiled_project_plugin_plan.catalog_generation()", linked)
        construction = SESSION_CONSTRUCTION.read_text(encoding="utf-8")
        self.assertIn("_runtime_plugin_catalog_snapshot: runtime_plugin_catalog_snapshot", construction)
        state = SESSION_STATE.read_text(encoding="utf-8")
        self.assertIn(
            "_runtime_plugin_catalog_snapshot: Arc<RuntimePluginCatalogSnapshot>",
            state,
        )


if __name__ == "__main__":
    unittest.main()
