from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
NODE_POOL = ROOT / "zircon_runtime/src/ui/surface/node_pool.rs"


class RuntimeUiNodePoolLookupOwnershipPerformanceContractTests(unittest.TestCase):
    def test_reuse_path_uses_owned_key_lookup_without_cloning_desired_metadata(self) -> None:
        source = NODE_POOL.read_text(encoding="utf-8")
        start = source.index("pub(crate) fn insert_or_reuse_pooled_child(")
        end = source.index("fn collect_subtree_node_ids(", start)
        insertion = source[start:end]

        self.assertIn("let (desired, pooled) = pool.take_owned(desired);", insertion)
        self.assertNotIn("UiSurfaceNodePoolKey::from_node(&desired)", insertion)
        self.assertNotIn("desired.clone()", insertion)

    def test_owned_take_roundtrips_key_fields_and_reuses_matching_bucket(self) -> None:
        source = NODE_POOL.read_text(encoding="utf-8")
        start = source.index("fn take_owned(")
        end = source.index("    pub fn take(", start)
        take = source[start:end]

        self.assertIn("std::mem::take(&mut desired.node_path.0)", take)
        self.assertIn("std::mem::take(&mut metadata.component)", take)
        self.assertIn("metadata.control_id.take()", take)
        self.assertIn("self.buckets.get_mut(&key)", take)
        self.assertIn("desired.template_metadata = Some(metadata)", take)
        self.assertNotIn("UiSurfaceNodePoolKey::from_node", take)

    def test_node_pool_lower_regressions_cover_owned_take_boundary(self) -> None:
        source = NODE_POOL.read_text(encoding="utf-8")

        self.assertIn("owned_take_roundtrips_desired_identity", source)
        self.assertIn("owned_take_reuses_matching_bucket", source)

    def test_residency_report_uses_one_combined_bucket_walk(self) -> None:
        source = NODE_POOL.read_text(encoding="utf-8")
        start = source.index("impl UiSurfaceNodePoolReport {")
        end = source.index("impl UiSurfaceNodePoolKey {", start)
        report = source[start:end]

        self.assertIn(
            "let (resident_node_count, resident_bucket_count) = pool.residency_counts();",
            report,
        )
        self.assertNotIn("pool.resident_node_count()", report)
        self.assertNotIn("pool.resident_bucket_count()", report)
        self.assertIn("fn residency_counts(&self) -> (usize, usize)", source)

        trim_start = source.index("fn trim(&mut self)")
        trim_end = source.index("    /// Looks up a pooled node", trim_start)
        trim = source[trim_start:trim_end]
        self.assertIn("let (trimmed_node_count, trimmed_bucket_count) = self.residency_counts();", trim)
        self.assertNotIn("self.resident_node_count()", trim)
        self.assertNotIn("self.resident_bucket_count()", trim)


if __name__ == "__main__":
    unittest.main()
