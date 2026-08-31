from pathlib import Path
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    REPO_ROOT
    / "zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/pending_completion/apply_gpu_cache_entries.rs"
)


def source_text() -> str:
    return SOURCE.read_text(encoding="utf-8")


def compact_source() -> str:
    return "".join(source_text().split())


class Runtime98HashedGpuCacheMembershipPerformanceContract(unittest.TestCase):
    def test_gpu_cache_membership_uses_one_capacity_bounded_hash_index(self) -> None:
        source = source_text()
        compact = compact_source()

        self.assertIn("use std::collections::HashSet;", source)
        self.assertNotIn("BTreeSet", source)
        self.assertIn(
            "letmutgpu_resident_probe_ids=HashSet::with_capacity(cache_entries.len());",
            compact,
        )
        self.assertEqual(source.count("HashSet::with_capacity"), 1)

    def test_live_deduplication_index_is_reused_for_resident_eviction(self) -> None:
        compact = compact_source()

        self.assertIn("!gpu_resident_probe_ids.insert(*probe_id)", compact)
        self.assertIn("!gpu_resident_probe_ids.contains(&probe_id)", compact)
        self.assertNotIn("collect::<HashSet", compact)
        self.assertNotIn("gpu_resident_probes", compact)

    def test_first_slot_and_input_order_remain_owned_by_the_unique_entry_vector(self) -> None:
        compact = compact_source()

        self.assertIn(
            "letmutunique_cache_entries=Vec::with_capacity(cache_entries.len());",
            compact,
        )
        self.assertIn("unique_cache_entries.push((*probe_id,*slot));", compact)
        self.assertIn("for(probe_id,slot)inunique_cache_entries", compact)
        self.assertLess(
            compact.index("unique_cache_entries.push((*probe_id,*slot));"),
            compact.index("for(probe_id,slot)inunique_cache_entries"),
        )
        self.assertNotIn("forprobe_idingpu_resident_probe_ids", compact)

    def test_rust_behavior_covers_filter_dedup_eviction_and_promotion_order(self) -> None:
        compact = compact_source()

        self.assertIn(
            "fnruntime98_gpu_cache_membership_preserves_first_slot_and_evicts_absent_residents()",
            compact,
        )
        self.assertIn(
            "state.apply_gpu_cache_entries(&[(2,7),(2,9),(3,8),(4,8),(99,11)]);",
            compact,
        )
        self.assertIn("assert_eq!(state.probe_slot(1),None);", compact)
        self.assertIn("assert_eq!(state.probe_slot(2),Some(7));", compact)
        self.assertIn("assert_eq!(state.probe_slot(3),None);", compact)
        self.assertIn("assert_eq!(state.probe_slot(4),Some(8));", compact)
        self.assertIn("assert_eq!(state.probe_slot(99),None);", compact)


if __name__ == "__main__":
    unittest.main()
