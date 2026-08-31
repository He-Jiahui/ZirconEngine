from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
ENGINE = ROOT / "zircon_runtime_interface/src/ui/layout/engine.rs"
PERSISTENT_SEQUENCE = (
    ROOT / "zircon_runtime_interface/src/ui/surface/persistent_sequence.rs"
)


class UiLayoutReportAggregationContractTests(unittest.TestCase):
    def test_recompute_reuses_sorted_reason_vec_without_allocating_a_map(self):
        source = ENGINE.read_text(encoding="utf-8")
        body = source.split("pub fn recompute_counts(&mut self)", 1)[1].split(
            "    /// Replaces one stable node route", 1
        )[0]

        self.assertNotIn("BTreeMap", body)
        helper = source.split("fn increment_fallback_reason_count", 1)[1].split(
            "    /// Replaces one stable node route", 1
        )[0]
        self.assertIn("fallback_reason_counts", helper)
        self.assertIn("binary_search_by_key", helper)
        self.assertNotIn("&mut self", helper)
        self.assertIn("fallback_reason_counts: &mut Vec", helper)
        self.assertNotIn("let mut fallback_reason_counts", body)

    def test_reason_aggregation_preserves_incremental_sorted_update_helper(self):
        source = ENGINE.read_text(encoding="utf-8")
        self.assertIn("fn increment_fallback_reason_count", source)
        self.assertIn("Self::increment_fallback_reason_count(", source)
        self.assertIn("selection.fallback_reason", source)
        self.assertIn("replacement.fallback_reason", source)

    def test_incremental_replacement_does_not_clone_the_full_selection_payload(self):
        source = ENGINE.read_text(encoding="utf-8")
        body = source.split("pub fn replace_selection_at", 1)[1].split(
            "\n    }\n}\n\nfn unsupported_reason", 1
        )[0]

        self.assertNotIn("get(index).cloned()", body)
        self.assertIn("previous.selected_backend", body)
        self.assertIn("previous.support", body)
        self.assertIn("get_mut_with_stats(index)", body)

    def test_report_replacement_exposes_the_actual_persistent_cow_work(self):
        source = ENGINE.read_text(encoding="utf-8")

        self.assertIn("replace_selection_at_with_cow_stats", source)
        self.assertIn("UiPersistentSequenceCowStats", source)
        self.assertIn("get_mut_with_stats(index)", source)

    def test_selection_routes_use_persistent_segmented_storage(self):
        source = ENGINE.read_text(encoding="utf-8")

        self.assertIn(
            "use crate::ui::surface::{UiPersistentSequence, UiPersistentSequenceCowStats};",
            source,
        )
        self.assertIn(
            "pub selections: UiPersistentSequence<UiLayoutEngineSelection>",
            source,
        )
        self.assertNotIn(
            "pub selections: Vec<UiLayoutEngineSelection>",
            source,
        )
        self.assertIn("impl Eq for UiLayoutEngineSelectionReport {}", source)

    def test_vec_construction_and_json_wire_format_convert_at_the_boundary(self):
        source = ENGINE.read_text(encoding="utf-8")
        deserialize = source.split("impl<'de> Deserialize<'de>", 1)[1].split(
            "impl UiLayoutEngineSelectionReport", 1
        )[0]
        constructor = source.split("pub fn from_selections", 1)[1].split(
            "pub fn recompute_counts", 1
        )[0]

        self.assertIn("selections: wire.selections.into()", deserialize)
        self.assertIn("selections: selections.into()", constructor)

    def test_source_compatibility_contract_is_the_supported_borrowed_subset(self):
        source = PERSISTENT_SEQUENCE.read_text(encoding="utf-8")

        self.assertIn("impl<T> Index<usize> for UiPersistentSequence<T>", source)
        self.assertIn("impl<T: Clone> IndexMut<usize>", source)
        self.assertIn("impl<'a, T> IntoIterator for &'a UiPersistentSequence<T>", source)


if __name__ == "__main__":
    unittest.main()
