from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_runtime/src/plugin/export_build_plan/project_manifest_validation/crates.rs"
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


class DeferredCrateDiagnosticContextPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_crate_diagnostic_walks_defer_context_formatting(self) -> None:
        runtime = function_body(self.source, "fn project_runtime_crate_diagnostics(")
        editor = function_body(self.source, "fn project_editor_crate_diagnostics(")
        self.assertIn("format_args!", runtime)
        self.assertIn("format_args!", editor)
        self.assertNotIn("&format!", runtime)
        self.assertNotIn("&format!", editor)

    def test_crate_validators_borrow_format_arguments(self) -> None:
        self.assertIn("use std::fmt;", self.source)
        self.assertEqual(self.source.count("context: fmt::Arguments<'_>"), 2)
        self.assertNotIn("context: &str", self.source)

    def test_rust_regression_preserves_diagnostic_text(self) -> None:
        self.assertIn("deferred_crate_diagnostic_context_preserves_contract", self.source)
        self.assertIn("project plugin audio runtime_crate", self.source)
        self.assertIn("project plugin audio editor_crate", self.source)


if __name__ == "__main__":
    unittest.main()
