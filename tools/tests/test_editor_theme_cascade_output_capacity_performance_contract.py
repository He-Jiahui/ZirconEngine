from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_editor/src/ui/asset_editor/style/theme_cascade_inspection.rs"
LOWER_TEST = (
    ROOT
    / "zircon_editor/src/ui/asset_editor/style/theme_cascade_inspection/"
    / "optimization_batch_jm_editor652_tests.rs"
)


def function_body(source: str, signature: str) -> str:
    start = source.index(signature)
    opening = source.index("{", start)
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated Rust function: {signature}")


class EditorThemeCascadeOutputCapacityPerformanceContractTests(unittest.TestCase):
    def test_layer_projection_reserves_and_extends_once(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        body = function_body(source, "fn cascade_layer_items(")
        self.assertIn("Vec::with_capacity(layers.len())", body)
        self.assertIn("items.extend(", body)
        self.assertNotIn("let mut items = Vec::new();", body)

    def test_token_and_rule_projections_use_known_bounds(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        token_body = function_body(source, "fn cascade_token_items<'a>(")
        rule_body = function_body(source, "fn cascade_rule_items<'a>(")

        self.assertIn("Vec::with_capacity(total_token_definitions)", token_body)
        self.assertIn("let total_rule_items = layers", rule_body)
        self.assertIn("saturating_mul(2)", rule_body)
        self.assertIn("Vec::with_capacity(total_rule_items)", rule_body)
        self.assertNotIn("let mut items = Vec::new();", token_body)
        self.assertNotIn("let mut items = Vec::new();", rule_body)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        source = SOURCE.read_text(encoding="utf-8")
        self.assertIn(
            '#[path = "theme_cascade_inspection/optimization_batch_jm_editor652_tests.rs"]',
            source,
        )
        self.assertIn("mod optimization_batch_jm_editor652_tests;", source)

        lower = LOWER_TEST.read_text(encoding="utf-8")
        self.assertIn("optimization_batch_jm_editor652_cascade_outputs_reserve_input_bounds", lower)
        self.assertIn("EDITOR652_CASCADE_OUTPUT_CAPACITY_BENCH_V1", lower)


if __name__ == "__main__":
    unittest.main()
