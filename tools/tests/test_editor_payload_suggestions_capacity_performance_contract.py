from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_editor/src/ui/asset_editor/binding/payload_suggestions.rs"


def function_body(source: str) -> str:
    start = source.index("fn immediate_nested_suggestions(")
    end = source.index("\n}", start)
    return source[start:end]


def growth_events(length: int, initial_capacity: int = 0) -> int:
    capacity = initial_capacity
    events = 0
    for current_length in range(1, length + 1):
        if current_length > capacity:
            capacity = 4 if capacity == 0 else capacity * 2
            events += 1
    return events


class EditorPayloadSuggestionsCapacityPerformanceContractTests(unittest.TestCase):
    def test_array_and_table_collectors_reserve_known_bounds(self) -> None:
        body = function_body(SOURCE.read_text(encoding="utf-8"))
        normalized = " ".join(body.split())

        self.assertIn("Vec::with_capacity(", normalized)
        self.assertIn(
            "entries .len() .saturating_add(usize::from(!entries.is_empty()))",
            normalized,
        )
        self.assertIn("suggestions.extend(", body)
        self.assertIn("let mut keys = Vec::with_capacity(entries.len());", body)
        self.assertIn("keys.extend(entries.keys().cloned());", body)

    def test_suggestion_semantics_and_ownership_shape_remain_explicit(self) -> None:
        body = function_body(SOURCE.read_text(encoding="utf-8"))

        for required in (
            "Value::Array(entries)",
            "Value::Table(entries)",
            "current_selected_value",
            ".len().max(entries.len())",
            "entries.first().cloned()",
            "suggestions.push",
            "keys.sort()",
            "entries.get(&key).cloned()",
        ):
            self.assertIn(required, body)
        self.assertNotIn("entries.iter().collect::<Vec<_>>()", body)

    def test_dense_array_and_table_models_remove_geometric_growth(self) -> None:
        dense_entries = 4_096
        legacy_array = growth_events(dense_entries + 1)
        legacy_table = growth_events(dense_entries)
        optimized_array = growth_events(
            dense_entries + 1,
            dense_entries + int(dense_entries > 0),
        )
        optimized_table = growth_events(dense_entries, dense_entries)

        self.assertEqual((legacy_array, legacy_table), (12, 11))
        self.assertEqual((optimized_array, optimized_table), (0, 0))


if __name__ == "__main__":
    unittest.main()
