from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/graphics/visibility/static_index/mod.rs"
RECORD = (
    ROOT
    / "docs/plans/optimize/zircon_runtime/09b/2026-08-27-dirty-proportional-static-index-update.md"
)


def production_source() -> str:
    return SOURCE.read_text(encoding="utf-8").split("#[cfg(test)]", 1)[0]


def incremental_update_body() -> str:
    source = production_source()
    start = source.index("    pub(crate) fn apply_update_plan(")
    end = source.index("    pub(crate) fn query_bounds(", start)
    return source[start:end]


class Runtime09BDirtyProportionalStaticIndexContract(unittest.TestCase):
    def test_changed_key_index_capacity_scales_with_delta(self) -> None:
        body = incremental_update_body()
        self.assertIn("let changed_key_count = plan", body)
        self.assertIn("with_capacity(changed_key_count)", body)
        self.assertIn("HashMap::<u64, Option<&VisibilityBvhInstance>>", body)

    def test_incremental_update_does_not_collect_the_full_scene_index(self) -> None:
        body = incremental_update_body()
        self.assertNotIn("collect::<BTreeMap", body)
        self.assertNotIn("instances_by_stable_instance_key", body)
        self.assertEqual(body.count("for instance in instances"), 1)

    def test_projection_preserves_last_duplicate_instance_semantics(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        self.assertIn(
            "visibility_static_index_incremental_projection_keeps_last_duplicate_instance",
            source,
        )
        self.assertIn("*changed_instance = Some(instance);", source)

    def test_performance_model_enforces_memory_and_p95_targets(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        record = RECORD.read_text(encoding="utf-8")
        self.assertIn("RUNTIME09B_DIRTY_PROPORTIONAL_STATIC_INDEX_BENCH_V1", source)
        self.assertIn("optimized_p95 * 100 <= legacy_p95 * 60", source)
        self.assertIn("99.87%", record)
        self.assertIn("76.2%", record)


if __name__ == "__main__":
    unittest.main()
