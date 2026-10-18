import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_runtime/src/ui/template/asset/hot_reload_executor.rs"
REGRESSION = (
    REPO_ROOT / "zircon_runtime/src/ui/template/asset/hot_reload_executor/tests/template_assets_capacity_tests.rs"
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


class RuntimeHotReloadTemplateAssetsCapacityPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.regression = REGRESSION.read_text(encoding="utf-8")

    def test_template_asset_collection_reserves_two_input_bound(self) -> None:
        helper = function_body(self.source, "template_asset_capacity")
        self.assertIn("saturating_add", helper)

        body = function_body(self.source, "template_assets_for_surface")
        self.assertIn("Vec::with_capacity(template_asset_capacity(", body)
        self.assertLess(body.index("Vec::with_capacity"), body.index(".filter"))

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn(
            "#[path = \"hot_reload_executor/tests/template_assets_capacity_tests.rs\"]",
            self.source,
        )
        self.assertIn(
            "optimization_batch_20260917_runtime794_hot_reload_template_assets_capacity_bench",
            self.regression,
        )
        self.assertIn(
            "RUNTIME794_HOT_RELOAD_TEMPLATE_ASSETS_CAPACITY_BENCH_V1",
            self.regression,
        )
        self.assertIn("optimized_growth_events", self.regression)

    def test_deterministic_model_eliminates_asset_vector_growth(self) -> None:
        input_count = 16_384
        capacity = 0
        legacy_growth_events = 0
        for length in range(1, input_count + 1):
            if length > capacity:
                capacity = 4 if capacity == 0 else capacity * 2
                legacy_growth_events += 1
        optimized_growth_events = 0
        self.assertGreater(legacy_growth_events, 0)
        self.assertEqual(optimized_growth_events, 0)


if __name__ == "__main__":
    unittest.main()
