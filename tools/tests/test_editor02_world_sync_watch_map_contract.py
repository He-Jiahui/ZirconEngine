from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


class EditorWorldSyncWatchMapContractTests(unittest.TestCase):
    def read(self, relative: str) -> str:
        return (ROOT / relative).read_text(encoding="utf-8")

    def test_editor_core_exports_one_world_sync_watch_map_authority(self) -> None:
        core = self.read("zircon_editor/src/core/mod.rs")
        sync = self.read("zircon_editor/src/core/sync/mod.rs")
        self.assertIn("pub mod sync;", core)
        self.assertIn("pub use watch_map::", sync)
        self.assertIn("WorldWatchMap", sync)

    def test_watch_map_owns_token_and_view_reverse_indexes(self) -> None:
        source = self.read("zircon_editor/src/core/sync/watch_map.rs")
        for contract in (
            "by_token",
            "by_view",
            "pub fn bind",
            "pub fn unbind_token",
            "pub fn unbind_view",
            "pub fn drain_tokens",
        ):
            self.assertIn(contract, source)

    def test_batch_projection_is_deterministic_and_diagnostic(self) -> None:
        source = self.read("zircon_editor/src/core/sync/watch_map.rs")
        for contract in (
            "InvalidationBatch",
            "ViewDirtySet",
            "pub fn project",
            "duplicate_tokens",
            "unknown_tokens",
            "generation",
        ):
            self.assertIn(contract, source)

    def test_regressions_cover_replace_cleanup_coalesce_and_unknown_tokens(self) -> None:
        source = self.read("zircon_editor/src/core/sync/watch_map/tests.rs")
        for test_name in (
            "binding_a_token_replaces_both_sides_of_the_old_relation",
            "unbinding_a_view_returns_sorted_runtime_tokens_and_clears_reverse_state",
            "project_coalesces_masks_per_view_and_reports_duplicate_and_unknown_tokens",
            "draining_tokens_clears_the_session_owned_map",
            "empty_masks_are_rejected_without_mutating_indexes",
            "invalid_rebind_preserves_the_existing_relation",
            "unbind_token_cleans_reverse_state_and_unknown_token_is_a_no_op",
        ):
            self.assertIn(test_name, source)

    def test_public_watch_map_surface_documents_replacement_and_cleanup_semantics(self) -> None:
        source = self.read("zircon_editor/src/core/sync/watch_map.rs")
        for phrase in (
            "Returns the replaced binding",
            "sorted token order",
            "Discards duplicate and unknown-token diagnostics",
        ):
            self.assertIn(phrase, source)

    def test_live_runtime_token_collision_preserves_existing_editor_binding(self) -> None:
        pump = self.read("zircon_editor/src/core/sync/pump.rs")
        regressions = self.read("zircon_editor/src/core/sync/pump/tests.rs")
        watch_view = pump.split("pub fn watch_view", 1)[1].split(
            "if let Err(error) = self.watches.bind", 1
        )[0]
        returned_token_path = watch_view.split(
            "let token = gateway.watch_world(registration.clone())?;", 1
        )[1]

        self.assertIn("reject_live_watch_token", pump)
        self.assertIn("self.reject_live_watch_token(token)?;", watch_view)
        self.assertLess(
            returned_token_path.index("self.synchronize_gateway_generation(gateway);"),
            returned_token_path.index("self.reject_live_watch_token(token)?;"),
        )
        self.assertNotIn("unwatch_world", watch_view)
        self.assertNotIn("TokenCollisionCleanup", pump)
        self.assertIn(
            "live_token_collision_preserves_the_existing_editor_binding", regressions
        )

    def test_public_integration_gate_avoids_unrelated_lib_test_modules(self) -> None:
        source = self.read("zircon_editor/tests/editor_world_sync_watch_map.rs")
        self.assertIn(
            "public_watch_map_projects_runtime_tokens_into_view_dirty_state", source
        )
        self.assertIn(
            "public_watch_map_view_and_session_cleanup_return_sorted_tokens", source
        )

if __name__ == "__main__":
    unittest.main()
