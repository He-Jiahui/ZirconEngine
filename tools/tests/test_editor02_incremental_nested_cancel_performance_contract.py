from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
LIFECYCLE_RS = ROOT / (
    "zircon_editor/src/core/editing/engine/transaction/lifecycle.rs"
)
SCOPE_TESTS_RS = ROOT / "zircon_editor/src/tests/editing/transaction_engine/scope.rs"
RECOVERY_TESTS_RS = ROOT / (
    "zircon_editor/src/tests/editing/transaction_engine/recovery.rs"
)


def function_region(source: str, start: str, end: str) -> str:
    offset = source.index(start)
    return source[offset : source.index(end, offset)]


class Editor02IncrementalNestedCancelPerformanceContractTests(unittest.TestCase):
    def test_nested_cancel_pops_frames_without_materializing_a_tail_vec(self) -> None:
        source = LIFECYCLE_RS.read_text(encoding="utf-8")
        cancel = function_region(
            source,
            "    pub(super) fn cancel(",
            "    fn cancel_frame(",
        )

        self.assertNotIn(".drain(", cancel)
        self.assertNotIn("collect::<Vec", cancel)
        self.assertIn("let Some(frame) = state.active.pop()", cancel)
        self.assertIn("let reached_scope = frame.id == scope;", cancel)
        self.assertIn("if reached_scope", cancel)

    def test_cancel_failure_restores_only_the_current_frame(self) -> None:
        source = LIFECYCLE_RS.read_text(encoding="utf-8")
        cancel = function_region(
            source,
            "    pub(super) fn cancel(",
            "    fn cancel_frame(",
        )

        self.assertIn("state.active.push(frame);", cancel)
        self.assertNotIn("for retained in frames", cancel)
        self.assertIn("state.context = Some(context);", cancel)
        self.assertIn("state.faulted = true;", cancel)

    def test_existing_nested_and_failure_behavior_oracles_remain_present(self) -> None:
        scope_tests = SCOPE_TESTS_RS.read_text(encoding="utf-8")
        recovery_tests = RECOVERY_TESTS_RS.read_text(encoding="utf-8")

        self.assertIn(
            "fn out_of_order_scope_consumption_cancels_descendants_without_residue()",
            scope_tests,
        )
        self.assertIn(
            "fn revert_error_retains_commands_and_faults_engine_without_finalizing()",
            recovery_tests,
        )


if __name__ == "__main__":
    unittest.main()
