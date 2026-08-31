from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_plugins/plugin_sdk/src/manifest/feature_bundle_builder.rs"


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
    raise AssertionError(f"unterminated function: {signature}")


class SizeHintFeatureCapabilityBatchPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.body = function_body(cls.source, "pub fn with_capabilities<I, S>(")

    def test_batch_append_uses_iterator_size_hint_through_extend(self) -> None:
        self.assertIn("self.feature", self.body)
        self.assertIn(".capabilities", self.body)
        self.assertIn(
            ".extend(capabilities.into_iter().map(Into::into))",
            self.body,
        )

    def test_batch_append_does_not_reenter_single_item_builder(self) -> None:
        self.assertNotIn("for capability in capabilities", self.body)
        self.assertNotIn("self = self.with_capability(capability)", self.body)

    def test_rust_regression_preserves_existing_and_batch_order(self) -> None:
        self.assertIn(
            "capability_batch_appends_after_existing_in_input_order",
            self.source,
        )
        self.assertIn('"runtime.existing"', self.source)
        self.assertIn('"runtime.batch.second"', self.source)


if __name__ == "__main__":
    unittest.main()
