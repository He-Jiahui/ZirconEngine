from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
READINESS = (
    ROOT
    / "zircon_runtime/crates/zr_resource/src/manager/readiness_projection.rs"
)
READINESS_TESTS = (
    ROOT
    / "zircon_runtime/crates/zr_resource/src/manager/readiness_projection/tests.rs"
)
READINESS_BEHAVIOR_TESTS = (
    ROOT
    / "zircon_runtime/crates/zr_resource/src/manager/readiness_projection/tests/behavior_red.rs"
)


def function_region(source: str, start: str, end: str) -> str:
    offset = source.index(start)
    return source[offset : source.index(end, offset)]


class Runtime64ReadinessNoopPerformanceContractTests(unittest.TestCase):
    def test_noop_update_preflights_before_allocating_the_source_arc(self) -> None:
        source = READINESS.read_text(encoding="utf-8")
        apply_updates = function_region(
            source,
            "    pub(super) fn apply_updates(",
            "    fn reverse_closure(",
        )

        preflight = apply_updates.index(
            "source_matches_update(self.sources.get(&update.id), &update)"
        )
        source_arc = apply_updates.index("record: Arc::new(record)")
        self.assertLess(preflight, source_arc)
        self.assertNotIn("self.sources.get(&update.id) == next.as_ref()", apply_updates)

    def test_preflight_borrows_record_fields_and_changed_path_moves_the_arc(self) -> None:
        source = READINESS.read_text(encoding="utf-8")
        preflight = function_region(
            source,
            "fn source_matches_update(",
            "fn source_load_state(",
        )
        apply_updates = function_region(
            source,
            "    pub(super) fn apply_updates(",
            "    fn reverse_closure(",
        )

        self.assertIn("current.record.as_ref() == record", preflight)
        self.assertIn("current.runtime_state != update.runtime_state", preflight)
        self.assertIn("current.payload_type_id != update.payload_type_id", preflight)
        exact_match = preflight.index("current.record.as_ref() == record")
        canonical_clone = preflight.index("let mut canonical = record.clone();")
        self.assertLess(exact_match, canonical_clone)
        self.assertIn("(None, None) => true", preflight)
        self.assertIn("self.sources.insert(update.id, next);", apply_updates)
        self.assertNotIn("self.sources.insert(update.id, next.clone());", apply_updates)

    def test_noop_identity_behavior_is_covered_by_rust(self) -> None:
        source = READINESS_TESTS.read_text(encoding="utf-8")

        self.assertIn(
            "fn identical_updates_preserve_source_and_generation_identity()",
            source,
        )
        self.assertGreaterEqual(source.count("Arc::ptr_eq("), 3)

    def test_canonical_dependency_slow_path_is_an_active_regression(self) -> None:
        source = READINESS_BEHAVIOR_TESTS.read_text(encoding="utf-8")
        test = function_region(
            source,
            "fn duplicate_and_reordered_dependency_sets_preserve_generation_identity()",
            "fn dependency_arrival_replacement_and_removal_update_the_exact_reverse_closure()",
        )

        self.assertNotIn("#[ignore", test)
        self.assertGreaterEqual(test.count("Arc::ptr_eq("), 2)


if __name__ == "__main__":
    unittest.main()
