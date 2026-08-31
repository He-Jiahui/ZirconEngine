from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
RESOLVE = ROOT / "zircon_runtime/src/ui/template/asset/localization/resolve.rs"
PERFORMANCE = ROOT / "zircon_runtime/src/ui/template/asset/localization/resolve/performance_tests.rs"


def function_region(source: str, start: str, end: str) -> str:
    offset = source.index(start)
    return source[offset : source.index(end, offset)]


class Runtime83LocalizationSingleMessageAllocationPerformanceContractTests(unittest.TestCase):
    def test_missing_key_formats_only_the_final_message_string(self) -> None:
        source = RESOLVE.read_text(encoding="utf-8")
        dependency = function_region(
            source,
            "fn validate_dependency",
            "fn missing_ref_severity(",
        )

        self.assertNotIn("let source = table", dependency)
        self.assertNotIn('format!(" in {source_uri}")', dependency)
        self.assertIn("let message = match table.source_uri.as_deref()", dependency)
        self.assertIn("message,", dependency)

    def test_release_evidence_declares_unique_missing_key_allocation_counts(self) -> None:
        performance = PERFORMANCE.read_text(encoding="utf-8")

        self.assertIn("legacy_missing_key_message_allocations={}", performance)
        self.assertIn("optimized_missing_key_message_allocations={}", performance)
        self.assertIn("DEPENDENCY_COUNT.saturating_mul(2)", performance)


if __name__ == "__main__":
    unittest.main()
