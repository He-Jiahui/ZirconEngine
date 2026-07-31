from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


class EditorWorldSyncSubscriptionTableContractTests(unittest.TestCase):
    def read(self, relative: str) -> str:
        return (ROOT / relative).read_text(encoding="utf-8")

    def test_scene_inspection_exports_one_subscription_table_authority(self) -> None:
        module = self.read("zircon_runtime/src/scene/inspection/mod.rs")
        self.assertIn("mod subscription;", module)
        self.assertIn("pub use subscription::{", module)
        self.assertIn("SubscriptionTableDiagnostics", module)
        self.assertIn("SubscriptionTableLimits", module)

    def test_subscription_table_owns_typed_indexes_without_generic_by_key(self) -> None:
        source = self.read(
            "zircon_runtime/src/scene/inspection/subscription.rs"
        )
        for contract in (
            "by_token",
            "world_tokens",
            "subtree_tokens",
            "component_tokens",
            "asset_tokens",
            "pending_facts",
            "pending_dirty",
            "pub fn watch",
            "pub fn unwatch",
            "pub fn record_fact",
            "pub fn flush",
        ):
            self.assertIn(contract, source)
        self.assertNotIn("by_key:", source)

    def test_mutation_throats_use_direct_routes_and_one_ancestor_chain(self) -> None:
        source = self.read(
            "zircon_runtime/src/scene/inspection/subscription.rs"
        )
        self.assertIn("pub fn invalidate_subtree", source)
        self.assertIn("pub fn invalidate_component_type", source)
        self.assertIn("pub fn invalidate_asset", source)
        self.assertIn("ancestor_chain", source)
        self.assertIn("component_tokens.get(type_name)", source)
        self.assertIn("asset_tokens.values()", source)

    def test_fact_queue_has_limits_coalesce_and_overflow_diagnostics(self) -> None:
        source = self.read(
            "zircon_runtime/src/scene/inspection/subscription.rs"
        )
        for contract in (
            "SubscriptionTableLimits",
            "SubscriptionTableDiagnostics",
            "pending_fact_index",
            "pending_estimated_bytes",
            "coalesced_facts",
            "overflowed_facts",
            "oldest_pending_age_generations",
            "pub fn with_limits",
            "pub fn diagnostics",
        ):
            self.assertIn(contract, source)

    def test_regressions_cover_direct_routing_single_walk_and_bounded_queue(self) -> None:
        source = self.read(
            "zircon_runtime/src/scene/inspection/subscription/tests.rs"
        )
        for test_name in (
            "watch_allocates_distinct_tokens_and_unwatch_revokes_pending_dirty",
            "typed_indexes_route_without_scanning_unrelated_watch_variants",
            "subtree_invalidation_walks_ancestry_once_for_many_watches",
            "fact_queue_coalesces_by_semantic_identity_and_stays_bounded",
            "overflow_marks_world_dirty_and_records_diagnostics",
            "subtree_walk_stops_at_malformed_parent_cycles",
        ):
            self.assertIn(test_name, source)

    def test_public_integration_gate_avoids_unrelated_lib_test_modules(self) -> None:
        source = self.read(
            "zircon_runtime/tests/runtime_world_sync_subscription_table.rs"
        )
        self.assertIn("public_subscription_table_lifecycle_and_flush_contract", source)
        self.assertIn("public_subscription_table_matches_subtree_and_asset_throats", source)
        self.assertIn("public_subscription_table_100k_watch_routing_is_direct_and_bounded", source)


if __name__ == "__main__":
    unittest.main()
