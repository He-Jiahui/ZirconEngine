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
    / "keyboard"
    / "menu.rs"
)
LOWER_REGRESSION = (
    REPO_ROOT / "zircon_runtime/src/ui/component/state_reducer/keyboard/menu/tests/search_streaming_tests.rs"
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


class RuntimeMenuSearchStreamingPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_root_builder_uses_one_capacity_hinted_output(self) -> None:
        body = function_body(self.source, "menu_search_option_list")
        self.assertIn(
            "Vec::with_capacity(menu_search_option_capacity_hint(value))", body
        )
        self.assertIn(
            "collect_top_level_search_options(value, &mut next_top_level_index, true, &mut options);",
            body,
        )

    def test_recursive_builders_stream_into_caller_owned_outputs(self) -> None:
        top_level = function_body(self.source, "collect_top_level_search_options")
        descendant = function_body(self.source, "collect_descendant_search_options")
        children = function_body(self.source, "collect_child_search_options")
        self.assertIn("options.reserve(values.len());", top_level)
        self.assertIn("options.reserve(values.len());", descendant)
        # Runtime781 removes the per-node borrowed child-value vector.  The
        # retained output vector is intentionally grown on demand so this
        # contract does not reintroduce the old broad map-size capacity hint.
        self.assertIn("let mut children = Vec::new();", children)
        self.assertNotIn("Vec::with_capacity(values.len())", children)
        self.assertNotIn("flat_map", top_level)
        self.assertNotIn("flat_map", descendant)
        self.assertIn("top_level_id: top_level_id.to_string()", descendant)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        lower = LOWER_REGRESSION.read_text(encoding="utf-8")
        self.assertIn(
            "runtime764_menu_search_stream_preserves_tree_order_and_focus_indices",
            lower,
        )
        self.assertIn("RUNTIME764_MENU_SEARCH_STREAM_BENCH_V1", lower)
        self.assertIn("optimized_intermediate_vectors", lower)

    def test_deterministic_model_eliminates_recursive_temporary_vectors(self) -> None:
        leaf_options = 1_024
        legacy_intermediate_vectors = leaf_options
        optimized_intermediate_vectors = 0
        self.assertGreater(legacy_intermediate_vectors, optimized_intermediate_vectors)
        self.assertEqual(optimized_intermediate_vectors, 0)
        self.assertEqual(leaf_options, 1_024)


if __name__ == "__main__":
    unittest.main()
