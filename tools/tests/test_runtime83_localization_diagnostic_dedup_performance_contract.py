from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
RESOLVE = ROOT / "zircon_runtime/src/ui/template/asset/localization/resolve.rs"


def function_region(source: str, start: str, end: str) -> str:
    offset = source.index(start)
    return source[offset : source.index(end, offset)]


class Runtime83LocalizationDiagnosticDedupPerformanceContractTests(unittest.TestCase):
    def test_missing_diagnostics_deduplicate_borrowed_identity_before_allocation(self) -> None:
        source = RESOLVE.read_text(encoding="utf-8")
        entry = function_region(
            source,
            "pub fn validate_localization_report_against_catalog(",
            "pub fn localization_table_keys_from_toml_str(",
        )
        dependency = function_region(
            source,
            "fn validate_dependency",
            "fn missing_ref_severity(",
        )

        self.assertIn("let mut emitted_diagnostics = HashSet::new();", entry)
        self.assertIn("&mut emitted_diagnostics", entry)
        self.assertIn("HashSet<(&'dependency str, &'dependency str, &'dependency str, bool)>", dependency)
        self.assertEqual(dependency.count("emitted_diagnostics.insert(identity)"), 2)
        first_allocation = min(
            dependency.index("dependency.path.clone()"),
            dependency.index("format!("),
        )
        self.assertLess(
            dependency.index("emitted_diagnostics.insert(identity)"),
            first_allocation,
        )

    def test_valid_keys_bypass_the_diagnostic_identity_set(self) -> None:
        source = RESOLVE.read_text(encoding="utf-8")
        dependency = function_region(
            source,
            "fn validate_dependency",
            "fn missing_ref_severity(",
        )

        valid_return = dependency.index("if table.keys.contains")
        second_insert = dependency.rindex("emitted_diagnostics.insert(identity)")
        self.assertLess(valid_return, second_insert)
        self.assertIn("diagnostics.sort();", source)
        performance = (ROOT / "zircon_runtime/src/ui/template/asset/localization/resolve/performance_tests.rs").read_text(encoding="utf-8")
        self.assertIn("legacy_diagnostic_constructions={}", performance)
        self.assertIn("optimized_diagnostic_constructions=1", performance)


if __name__ == "__main__":
    unittest.main()
