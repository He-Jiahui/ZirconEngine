from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
COLLECT_PENDING_RS = ROOT / (
    "zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/prepare_frame/"
    "collect_pending_updates.rs"
)


def source() -> str:
    return COLLECT_PENDING_RS.read_text(encoding="utf-8")


def compact(text: str) -> str:
    return re.sub(r"\s+", "", text)


def collect_body() -> str:
    text = source().split("pub(super) fn collect_pending_updates", 1)[1]
    return text.split("fn has_pending_ancestor_update", 1)[0]


class Plugins19CachedPendingSortKeysContract(unittest.TestCase):
    def test_pending_updates_cache_the_expensive_sort_key_once_per_item(self) -> None:
        body = compact(collect_body())

        self.assertIn("pending_updates.sort_by_cached_key(|update|", body)

    def test_legacy_repeated_sort_key_comparison_is_removed(self) -> None:
        body = compact(collect_body())

        self.assertNotIn("pending_updates.sort_by_key(|update|", body)

    def test_cached_key_preserves_every_priority_dimension(self) -> None:
        body = compact(collect_body()).split("sort_by_cached_key", 1)[1]

        self.assertIn("lineage_trace_support_sort_key(runtime,update.probe_id())", body)
        self.assertIn("resident_descendant_count(runtime,update.probe_id())", body)
        self.assertIn("descendant_count(runtime,update.probe_id())", body)
        self.assertIn("probe_depth(runtime,update.probe_id())", body)
        self.assertIn("update.generation()", body)
        self.assertIn("update.probe_id()", body)

    def test_priority_order_has_a_direct_rust_contract(self) -> None:
        self.assertIn("cached_pending_update_sort_preserves_priority_order", source())


if __name__ == "__main__":
    unittest.main()
