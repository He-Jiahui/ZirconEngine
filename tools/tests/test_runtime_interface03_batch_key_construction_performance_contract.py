
from tools.tests.rust_test_files import read_rust_test_file
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
KEY_SOURCE = ROOT / "zircon_runtime_interface/src/ui/surface/render/batch/key.rs"
PLAN_SOURCE = ROOT / "zircon_runtime_interface/src/ui/surface/render/batch/plan.rs"
PLAN_TEST_SOURCE = (
    ROOT / "zircon_runtime_interface/src/ui/surface/render/batch/plan/tests/performance_tests.rs"
)


def function_body(source: str, signature: str) -> str:
    start = source.index(signature)
    brace = source.index("{", start)
    depth = 0
    for index in range(brace, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[brace : index + 1]
    raise AssertionError(f"unterminated function body for {signature}")


class RuntimeInterface03BatchKeyConstructionPerformanceContract(unittest.TestCase):
    def test_standalone_key_construction_bypasses_single_entry_clip_interner(self) -> None:
        source = KEY_SOURCE.read_text(encoding="utf-8")
        standalone = function_body(source, "pub fn from_paint_element(")
        shared = function_body(
            source,
            "pub(super) fn from_paint_element_with_clip_states(",
        )

        self.assertIn(
            "Self::from_paint_element_with_clip(element, element.clip.clone())",
            standalone,
        )
        self.assertNotIn("UiBatchClipStates::default()", standalone)
        self.assertIn("clip_states.intern(clip)", shared)
        self.assertIn(
            "RUNTIME_INTERFACE03_STANDALONE_BATCH_KEY_CLIP_BENCH_V1",
            read_rust_test_file("zircon_runtime_interface/src/ui/surface/render/batch/tests/key.rs"),
        )

    def test_batch_plan_moves_the_previous_active_key_at_rollover(self) -> None:
        source = PLAN_SOURCE.read_text(encoding="utf-8")
        body = function_body(source, "pub fn from_paint_elements(")

        self.assertIn("let previous_key = active_key", body)
        self.assertIn(".replace(key)", body)
        self.assertIn("key: previous_key", body)
        self.assertNotIn("key: current_key.clone()", body)
        self.assertIn("mod performance_tests;", source)
        benchmark = PLAN_TEST_SOURCE.read_text(encoding="utf-8")
        self.assertIn(
            "RUNTIME_INTERFACE03_MOVED_BATCH_KEY_ROLLOVER_BENCH_V1",
            benchmark,
        )


if __name__ == "__main__":
    unittest.main()
