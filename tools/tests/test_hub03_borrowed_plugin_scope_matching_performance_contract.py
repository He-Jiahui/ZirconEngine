import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CATALOG = REPO_ROOT / "zircon_hub" / "src" / "plugins" / "catalog.rs"


def function_body(source: str, function_name: str) -> str:
    match = re.search(rf"\bfn\s+{re.escape(function_name)}\s*\(", source)
    if match is None:
        raise AssertionError(f"missing function {function_name}")
    opening = source.find("{", match.end())
    if opening < 0:
        raise AssertionError(f"missing body for {function_name}")
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated body for {function_name}")


class HubPluginScopeMatchingPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = CATALOG.read_text(encoding="utf-8")

    def test_editor_target_matching_borrows_the_trimmed_value(self) -> None:
        body = function_body(self.source, "is_editor_target")
        self.assertNotIn("to_ascii_lowercase", body)
        self.assertIn(".trim()", body)
        self.assertGreaterEqual(body.count("eq_ignore_ascii_case"), 2)

    def test_editor_capability_matching_borrows_the_ascii_prefix(self) -> None:
        body = function_body(self.source, "is_editor_capability")
        self.assertNotIn("to_ascii_lowercase", body)
        self.assertIn(".as_bytes()", body)
        self.assertIn(".get(..", body)
        self.assertIn("eq_ignore_ascii_case", body)

    def test_manifest_normalization_reuses_owned_strings_and_lazy_fallbacks(self) -> None:
        helper = function_body(self.source, "non_empty_or_else")
        self.assertIn("trimmed.len() == value.len()", helper)
        self.assertIn("fallback()", helper)
        self.assertIn("trimmed.to_owned()", helper)
        manifest = function_body(self.source, "read_plugin_manifest")
        self.assertNotIn("let fallback_id =", manifest)
        self.assertIn("non_empty_or_else(manifest.id, ||", manifest)
        self.assertIn("non_empty_or_else(manifest.display_name, || id.clone())", manifest)
        self.assertIn(
            "editor_scope_classifiers_reuse_canonical_manifest_strings",
            self.source,
        )

    def test_release_evidence_exercises_real_scope_and_normalization_helpers(self) -> None:
        self.assertIn(
            "hub03_plugin_scope_matching_release_benchmark_evidence",
            self.source,
        )
        self.assertIn("HUB03_PLUGIN_SCOPE_MATCHING_BENCH_V1", self.source)
        self.assertIn("const MANIFESTS: usize = 32_768", self.source)
        self.assertIn("is_editor_target(value)", self.source)
        self.assertIn("is_editor_capability(value)", self.source)
        self.assertIn("non_empty_or_else(Some(input.id)", self.source)
        self.assertIn(".div_ceil(100)", self.source)
        self.assertIn("legacy_raw_ns={}", self.source)

    def test_release_evidence_keeps_allocation_and_latency_gates(self) -> None:
        self.assertIn("assert_eq!(legacy_allocations, MANIFESTS * 10)", self.source)
        self.assertIn("assert_eq!(optimized_allocations, 0)", self.source)
        self.assertIn(
            "optimized_p50_ns.saturating_mul(100)"
            " <= legacy_p50_ns.saturating_mul(25)",
            self.source,
        )
        self.assertIn(
            "optimized_p95_ns.saturating_mul(100)"
            " <= legacy_p95_ns.saturating_mul(25)",
            self.source,
        )


if __name__ == "__main__":
    unittest.main()
