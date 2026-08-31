from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_runtime/src/plugin/runtime_plugin/runtime_plugin_catalog/bridge_lifecycle.rs"
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
    raise AssertionError(f"unterminated function: {signature}")


class StreamingBridgeLifecycleReportPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_affected_slots_preallocate_the_known_total(self) -> None:
        body = function_body(self.source, "pub fn affected_slots(&self)")
        self.assertIn("Vec::with_capacity(self.affected_slot_count())", body)
        self.assertIn("extend_from_slice", body)
        self.assertNotIn(".flat_map(", body)

    def test_blocked_diagnostic_streams_into_the_final_buffer(self) -> None:
        block_impl = self.source.index("impl RuntimePluginBridgeLifecycleBlock")
        body = function_body(self.source[block_impl:], "pub fn diagnostic(&self)")
        self.assertIn("write!", body)
        self.assertIn("blocker.write_diagnostic(&mut diagnostic)", body)
        self.assertNotIn("collect::<Vec<_>>()", body)
        self.assertNotIn('.join("; ")', body)

    def test_rust_regression_preserves_blocked_diagnostic(self) -> None:
        self.assertIn(
            "streaming_bridge_lifecycle_block_diagnostic_preserves_contract",
            self.source,
        )
        self.assertIn(
            "bridge.provider_lifecycle_blocked: provider plugin `rendering` Disable blocked by 2 strong dependent(s)",
            self.source,
        )


if __name__ == "__main__":
    unittest.main()
