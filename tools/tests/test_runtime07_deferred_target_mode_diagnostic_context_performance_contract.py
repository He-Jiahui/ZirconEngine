from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
TARGET_MODES = (
    ROOT
    / "zircon_runtime/src/plugin/export_build_plan/project_manifest_validation/target_modes.rs"
)
ALLOCATION_TESTS = (
    ROOT
    / "zircon_runtime/src/plugin/export_build_plan/project_manifest_validation/target_modes/allocation_tests.rs"
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


class DeferredTargetModeDiagnosticContextPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = TARGET_MODES.read_text(encoding="utf-8")
        cls.allocation_tests = ALLOCATION_TESTS.read_text(encoding="utf-8")

    def test_target_mode_validator_borrows_format_arguments(self) -> None:
        self.assertIn("use std::fmt;", self.source)
        validator = function_body(self.source, "fn validate_project_target_modes(")
        self.assertIn("context: fmt::Arguments<'_>", self.source)
        self.assertNotIn("context: &str", validator)

    def test_selection_and_feature_calls_defer_context_formatting(self) -> None:
        diagnostics = function_body(self.source, "fn project_target_mode_diagnostics(")
        self.assertEqual(2, diagnostics.count("format_args!"))
        self.assertNotIn("&format!", diagnostics)

    def test_rust_regression_preserves_target_mode_diagnostic(self) -> None:
        self.assertIn(
            "deferred_target_mode_diagnostic_context_preserves_contract",
            self.allocation_tests,
        )
        self.assertIn(
            "project plugin feature rendering.shadow target_modes must not repeat target mode ClientRuntime",
            self.allocation_tests,
        )


if __name__ == "__main__":
    unittest.main()
