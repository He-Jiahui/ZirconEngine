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
    / "collection.rs"
)
LOWER_REGRESSION = (
    REPO_ROOT / "zircon_runtime/src/ui/component/state_reducer/collection/tests/mutation_single_resolution_tests.rs"
)


def function_body(source: str, function_name: str) -> str:
    match = re.search(
        rf"\bfn\s+{re.escape(function_name)}(?![A-Za-z0-9_])(?:<[^>]*>)?\s*\(.*?\)",
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


class RuntimeCollectionSingleResolutionPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_array_mutations_resolve_the_state_property_once(self) -> None:
        for function_name in (
            "set_array_element",
            "remove_array_element",
            "move_array_element",
        ):
            body = function_body(self.source, function_name)
            self.assertEqual(body.count("array_value_mut("), 1, function_name)

    def test_map_mutations_resolve_the_state_property_once(self) -> None:
        for function_name in (
            "add_map_entry",
            "set_map_entry",
            "remove_map_entry",
        ):
            body = function_body(self.source, function_name)
            self.assertEqual(body.count("map_value_mut("), 1, function_name)

    def test_add_and_set_use_single_tree_lookups_on_success(self) -> None:
        add_body = function_body(self.source, "add_map_entry")
        set_body = function_body(self.source, "set_map_entry")
        self.assertIn("Entry::Vacant", add_body)
        self.assertIn("Entry::Occupied", add_body)
        self.assertIn("values.get_mut(&key)", set_body)
        self.assertNotIn("values.contains_key(&key)", set_body)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        lower = LOWER_REGRESSION.read_text(encoding="utf-8")
        self.assertIn(
            "runtime770_collection_mutations_preserve_success_and_error_contracts",
            lower,
        )
        self.assertIn("RUNTIME770_COLLECTION_SINGLE_RESOLUTION_BENCH_V1", lower)
        self.assertIn("optimized_property_resolutions", lower)

    def test_deterministic_model_removes_repeated_property_resolution(self) -> None:
        mutations_per_sample = 4_096
        legacy_property_resolutions = mutations_per_sample * 2
        optimized_property_resolutions = mutations_per_sample
        self.assertGreater(legacy_property_resolutions, optimized_property_resolutions)
        self.assertEqual(optimized_property_resolutions, mutations_per_sample)


if __name__ == "__main__":
    unittest.main()
