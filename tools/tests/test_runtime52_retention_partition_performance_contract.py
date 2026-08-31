from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
PLANNING = ROOT / "zircon_runtime/src/scene/dynamic_scene/session/retention/prune/planning.rs"
PARTITION = (
    ROOT
    / "zircon_runtime/src/scene/dynamic_scene/session/retention/prune/planning/partition.rs"
)
MANIFEST = ROOT / "zircon_runtime/src/scene/dynamic_scene/session/manifest/archive.rs"
SORTED_LOOKUP = (
    ROOT
    / "zircon_runtime/src/scene/dynamic_scene/session/manifest/archive/sorted_lookup.rs"
)
ARCHIVE = ROOT / "zircon_runtime/src/scene/dynamic_scene/session/archive.rs"
SECONDARY_INDEX = (
    ROOT / "zircon_runtime/src/scene/dynamic_scene/session/archive/secondary_index.rs"
)


def function_body(source: str, function_name: str) -> str:
    marker = f"fn {function_name}("
    start = source.find(marker)
    if start < 0:
        raise AssertionError(f"missing function {function_name}")
    opening = source.find("{", start)
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated function {function_name}")


class Runtime52RetentionPartitionPerformanceContractTests(unittest.TestCase):
    def test_retention_partition_is_an_isolated_production_helper(self) -> None:
        planning = PLANNING.read_text(encoding="utf-8")
        partition = PARTITION.read_text(encoding="utf-8")

        self.assertIn("mod partition;", planning)
        self.assertIn("use self::partition::partition_pruned_slot_ids;", planning)
        self.assertNotIn("fn partition_pruned_slot_ids(", planning)
        self.assertIn("pub(super) fn partition_pruned_slot_ids(", partition)

    def test_retention_partition_moves_ids_in_one_pass_without_a_removed_set(
        self,
    ) -> None:
        partition = PARTITION.read_text(encoding="utf-8").split("#[cfg(test)]", 1)[0]

        self.assertIn(".partition(", partition)
        self.assertIn(".peekable()", partition)
        self.assertNotIn(".cloned()", partition)
        self.assertNotIn("removed_set", partition)
        self.assertNotIn(".contains(slot_id)", partition)

    def test_retention_partition_keeps_behavior_and_performance_evidence_together(
        self,
    ) -> None:
        partition = PARTITION.read_text(encoding="utf-8")

        self.assertIn("runtime52_retention_partition_preserves_order", partition)
        self.assertIn("runtime52_retention_partition_evidence", partition)
        self.assertIn("RUNTIME52_RETENTION_PARTITION_BENCH_V1", partition)
        self.assertIn("SLOT_COUNT: usize = 100_000", partition)
        self.assertIn("SAMPLE_PAIRS: usize = 21", partition)
        self.assertIn("optimized_p95_ns <= legacy_p95_ns.saturating_mul(80) / 100", partition)

    def test_manifest_slot_lookup_uses_an_isolated_sorted_helper(self) -> None:
        manifest = MANIFEST.read_text(encoding="utf-8")
        sorted_lookup = SORTED_LOOKUP.read_text(encoding="utf-8")

        self.assertIn("mod sorted_lookup;", manifest)
        self.assertIn("use self::sorted_lookup::sorted_index_by_key;", manifest)
        self.assertIn("sorted_index_by_key(&self.slots, slot_id", manifest)
        self.assertIn("pub(super) fn sorted_index_by_key", sorted_lookup)

    def test_manifest_sorted_lookup_retains_unsorted_compatibility_fallback(self) -> None:
        sorted_lookup = SORTED_LOOKUP.read_text(encoding="utf-8")

        self.assertIn(".binary_search_by(", sorted_lookup)
        self.assertIn(".or_else(||", sorted_lookup)
        self.assertIn(".position(", sorted_lookup)
        self.assertIn("runtime52_sorted_lookup_preserves_unsorted_input", sorted_lookup)

    def test_manifest_sorted_lookup_has_a_release_p95_gate(self) -> None:
        sorted_lookup = SORTED_LOOKUP.read_text(encoding="utf-8")

        self.assertIn("RUNTIME52_SORTED_LOOKUP_BENCH_V1", sorted_lookup)
        self.assertIn("SLOT_COUNT: usize = 100_000", sorted_lookup)
        self.assertIn("QUERY_COUNT: usize = 100", sorted_lookup)
        self.assertIn("SAMPLE_PAIRS: usize = 21", sorted_lookup)
        self.assertIn("optimized_p95_ns <= legacy_p95_ns.saturating_mul(20) / 100", sorted_lookup)

    def test_archive_secondary_index_uses_an_isolated_production_helper(self) -> None:
        archive = ARCHIVE.read_text(encoding="utf-8")
        secondary_index = SECONDARY_INDEX.read_text(encoding="utf-8")

        self.assertIn("mod secondary_index;", archive)
        self.assertIn("use self::secondary_index::{", archive)
        self.assertIn("index_secondary_entries", archive)
        self.assertIn("remove_secondary_entries", archive)
        self.assertIn("pub(super) fn index_secondary_entries", secondary_index)
        self.assertIn("pub(super) fn remove_secondary_entries", secondary_index)

    def test_metadata_replace_moves_old_metadata_without_cloning_scene_or_tag_vector(
        self,
    ) -> None:
        archive = ARCHIVE.read_text(encoding="utf-8")
        replace = function_body(archive, "replace_slot_metadata")
        index = function_body(archive, "index_slot_secondary_entries")

        self.assertIn("std::mem::replace", replace)
        self.assertNotIn("self.slots[slot_index].clone()", replace)
        self.assertNotIn("metadata.tags.clone()", index)

    def test_metadata_secondary_index_has_behavior_and_release_p95_evidence(self) -> None:
        secondary_index = SECONDARY_INDEX.read_text(encoding="utf-8")

        self.assertIn("runtime52_secondary_index_preserves_ordered_queries", secondary_index)
        self.assertIn("runtime52_metadata_index_evidence", secondary_index)
        self.assertIn("RUNTIME52_METADATA_INDEX_BENCH_V1", secondary_index)
        self.assertIn("SCENE_PAYLOAD_BYTES: usize = 1024 * 1024", secondary_index)
        self.assertIn("TAG_COUNT: usize = 64", secondary_index)
        self.assertIn("SAMPLE_PAIRS: usize = 21", secondary_index)
        self.assertIn("optimized_p95_ns <= legacy_p95_ns.saturating_mul(50) / 100", secondary_index)


if __name__ == "__main__":
    unittest.main()
