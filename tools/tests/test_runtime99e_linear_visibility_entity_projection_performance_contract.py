from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/scene/world/render_visibility.rs"
RECORD = (
    ROOT
    / "docs/plans/optimize/zircon_runtime/99e/2026-08-27-linear-visibility-entity-projection.md"
)


def production_source() -> str:
    return SOURCE.read_text(encoding="utf-8").split("#[cfg(test)]", 1)[0]


def projection_body() -> str:
    source = production_source()
    start = source.index("fn project_visibility_entity_sets(")
    end = source.index("fn sort_and_dedup_entities(", start)
    return source[start:end]


class Runtime99ELinearVisibilityEntityProjectionContract(unittest.TestCase):
    def test_projection_removes_ordered_tree_materialization(self) -> None:
        source = production_source()
        self.assertNotIn("BTreeSet", source)
        self.assertNotIn("collect::<BTreeSet", source)

    def test_projection_visits_renderables_once(self) -> None:
        body = projection_body()
        self.assertEqual(body.count("for entry in renderables"), 1)
        self.assertEqual(body.count("Vec::with_capacity(renderables.len())"), 3)

    def test_projection_sorts_and_deduplicates_all_entity_sets(self) -> None:
        body = projection_body()
        self.assertEqual(body.count("sort_and_dedup_entities("), 3)
        source = production_source()
        self.assertIn("entities.sort_unstable();", source)
        self.assertIn("entities.dedup();", source)

    def test_behavior_and_performance_evidence_are_recorded(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        record = RECORD.read_text(encoding="utf-8")
        self.assertIn(
            "linear_entity_projection_preserves_sorted_unique_mobility_sets", source
        )
        self.assertIn("98.60%", record)
        self.assertIn("56.99%", record)
        self.assertIn("64.63%", record)


if __name__ == "__main__":
    unittest.main()
