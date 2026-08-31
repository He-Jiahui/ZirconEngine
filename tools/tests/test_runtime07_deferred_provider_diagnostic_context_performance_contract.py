from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
IDENTITY = (
    ROOT
    / "zircon_runtime/src/plugin/export_build_plan/project_manifest_validation/identity.rs"
)
PROVIDER = (
    ROOT
    / "zircon_runtime/src/plugin/export_build_plan/project_manifest_validation/provider.rs"
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


class DeferredProviderDiagnosticContextPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.identity = IDENTITY.read_text(encoding="utf-8")
        cls.provider = PROVIDER.read_text(encoding="utf-8")

    def test_package_id_validator_borrows_format_arguments(self) -> None:
        self.assertIn("use std::fmt;", self.identity)
        self.assertIn("context: fmt::Arguments<'_>", self.identity)
        self.assertNotIn("context: &str", self.identity)

    def test_selection_and_provider_calls_defer_context_formatting(self) -> None:
        selection = function_body(self.identity, "fn project_plugin_package_id_diagnostics(")
        provider = function_body(self.provider, "fn project_feature_provider_package_id_diagnostics(")
        self.assertIn("format_args!", selection)
        self.assertIn("format_args!", provider)
        self.assertNotIn("&format!", provider)

    def test_rust_regression_preserves_package_id_diagnostics(self) -> None:
        self.assertIn("deferred_provider_diagnostic_context_preserves_contract", self.identity)
        self.assertIn("project plugin selection id", self.identity)
        self.assertIn("project plugin feature streaming provider_package_id", self.identity)


if __name__ == "__main__":
    unittest.main()
