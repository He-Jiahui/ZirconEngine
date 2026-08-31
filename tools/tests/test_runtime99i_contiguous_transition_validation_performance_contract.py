from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/scene/ecs/archetype/table/table.rs"
RECORD = (
    ROOT
    / "docs/plans/optimize/zircon_runtime/99i/2026-08-27-contiguous-transition-validation.md"
)


def production_source() -> str:
    return SOURCE.read_text(encoding="utf-8").split("#[cfg(test)]", 1)[0]


def transition_body() -> str:
    source = production_source()
    start = source.index("    pub(crate) fn validate_transition(")
    end = source.index("    pub(crate) fn bind_prevalidated_row(", start)
    return source[start:end]


class Runtime99IContiguousTransitionValidationContract(unittest.TestCase):
    def test_transition_uses_one_contiguous_component_id_buffer(self) -> None:
        body = transition_body()
        self.assertNotIn("BTreeSet", body)
        self.assertIn("collect::<Vec<_>>()", body)
        self.assertIn("final_component_ids.sort_unstable();", body)
        self.assertIn("final_component_ids.dedup();", body)

    def test_transition_reserves_only_real_insertions_and_applies_sorted_delta(self) -> None:
        body = transition_body()
        self.assertIn("inserted_component_count", body)
        self.assertIn("reserve(inserted_component_count)", body)
        self.assertIn("apply_component_membership_updates(", body)
        self.assertIn("binary_search(&component_id)", production_source())

    def test_linear_schema_comparison_preserves_error_precedence(self) -> None:
        body = transition_body()
        unexpected = body.index("first_unexpected_component(")
        missing = body.index("first_missing_component(")
        self.assertLess(unexpected, missing)
        source = SOURCE.read_text(encoding="utf-8")
        self.assertIn(
            "runtime99i_contiguous_transition_validation_preserves_error_precedence",
            source,
        )

    def test_performance_evidence_enforces_allocation_and_p95_targets(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        record = RECORD.read_text(encoding="utf-8")
        self.assertIn("RUNTIME99I_CONTIGUOUS_TRANSITION_VALIDATION_BENCH_V1", source)
        self.assertIn("optimized_p95 * 100 <= legacy_p95 * 60", source)
        self.assertIn("99.74%", record)
        self.assertIn("66.90%", record)


if __name__ == "__main__":
    unittest.main()
