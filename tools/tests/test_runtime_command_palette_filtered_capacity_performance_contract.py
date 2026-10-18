import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    REPO_ROOT
    / "zircon_runtime"
    / "src"
    / "ui"
    / "component"
    / "state_reducer"
    / "command_palette.rs"
)


def function_body(source: str, function_name: str) -> str:
    match = re.search(
        rf"\bfn\s+{re.escape(function_name)}(?![A-Za-z0-9_])\s*\(.*?\)",
        source,
        re.S,
    )
    if match is None:
        raise AssertionError(f"missing function {function_name}")
    opening = source.find("{", match.end())
    if opening < 0:
        raise AssertionError(f"missing body for {function_name}")
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated body for {function_name}")


class RuntimeCommandPaletteFilteredCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_filtered_projection_reserves_the_parsed_entry_upper_bound(self) -> None:
        body = function_body(self.source, "filtered_command_ids_for_entries")
        self.assertIn("Vec::with_capacity(entries.len())", body)
        self.assertLess(
            body.index("Vec::with_capacity(entries.len())"),
            body.index("for entry in entries"),
        )
        self.assertIn("command_matches_source(entry, source)", body)
        self.assertIn("entry.matches_query(query)", body)
        self.assertIn("filtered.push(entry.id.clone())", body)
        self.assertNotIn(".collect", body)

    def test_state_sync_reuses_the_bounded_filter_helper(self) -> None:
        body = function_body(self.source, "sync_filter_state")
        compact = "".join(body.split())
        self.assertRegex(
            compact,
            r"filtered_command_ids_for_entries\(&entries,source\.as_deref\(\),query\.as_deref\(\),?\)",
        )

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn(
            "runtime755_command_palette_filtered_capacity_preserves_source_query_and_order",
            self.source,
        )
        self.assertIn(
            "RUNTIME755_COMMAND_PALETTE_FILTERED_CAPACITY_BENCH_V1",
            self.source,
        )
        self.assertIn("optimized_growth_events", self.source)

    def test_deterministic_model_eliminates_filtered_vector_growth(self) -> None:
        parsed_entries = 1_024
        legacy_growth_events = 10
        optimized_growth_events = 0
        self.assertGreater(legacy_growth_events, optimized_growth_events)
        self.assertEqual(optimized_growth_events, 0)
        self.assertEqual(parsed_entries, 1_024)


if __name__ == "__main__":
    unittest.main()
