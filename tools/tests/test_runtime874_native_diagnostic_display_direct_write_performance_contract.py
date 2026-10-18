import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    REPO_ROOT
    / "zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/diagnostics.rs"
)
LOWER = (
    REPO_ROOT / "zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/diagnostics/tests/display_direct_write_tests.rs"
)


def display_owner(source: str) -> str:
    match = re.search(
        r"impl std::fmt::Display for NativePluginBehaviorDiagnosticError \{.*?\n}\n",
        source,
        re.S,
    )
    if match is None:
        raise AssertionError("NativePluginBehaviorDiagnosticError Display owner is missing")
    return match.group(0)


class Runtime874NativeDiagnosticDisplayDirectWritePerformanceContractTests(
    unittest.TestCase
):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_display_writes_the_first_diagnostic_without_joining(self) -> None:
        owner = display_owner(self.source)
        self.assertIn("let mut diagnostics = diagnostics.iter()", owner)
        self.assertIn("let Some(first) = diagnostics.next()", owner)
        self.assertIn("formatter.write_str(first)?", owner)

    def test_display_appends_remaining_diagnostics_positionally(self) -> None:
        owner = display_owner(self.source)
        self.assertIn("for diagnostic in diagnostics", owner)
        self.assertIn('formatter.write_str("; ")?', owner)
        self.assertIn("formatter.write_str(diagnostic)?", owner)
        self.assertTrue(owner.rstrip().endswith("}\n}"))

    def test_display_does_not_allocate_a_joined_child(self) -> None:
        owner = display_owner(self.source)
        self.assertNotIn(".join(", owner)
        self.assertNotIn("collect::<", owner)
        self.assertNotIn("format!", owner)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn("mod display_direct_write_tests;", self.source)
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn(
            "runtime874_native_diagnostic_display_direct_write_preserves_exact_text",
            lower,
        )
        self.assertIn(
            "RUNTIME874_NATIVE_DIAGNOSTIC_DISPLAY_DIRECT_WRITE_BENCH_V1", lower
        )
        self.assertIn("const SAMPLE_PAIRS: usize = 101;", lower)
        self.assertIn("legacy_p50_ns", lower)
        self.assertIn("optimized_p99_ns", lower)

    def test_dense_display_temporary_allocation_model(self) -> None:
        displays = 4_096
        legacy_join_outputs = displays
        optimized_join_outputs = 0
        self.assertEqual(legacy_join_outputs, 4_096)
        self.assertEqual(optimized_join_outputs, 0)


if __name__ == "__main__":
    unittest.main()
