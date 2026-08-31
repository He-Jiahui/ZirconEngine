import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
COMING_SOON = (
    REPO_ROOT
    / "zircon_hub"
    / "src"
    / "tauri_app"
    / "view_model"
    / "coming_soon.rs"
)
CATALOG = (
    REPO_ROOT
    / "zircon_hub"
    / "src"
    / "tauri_app"
    / "view_model"
    / "catalog.rs"
)


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


class HubProjectionAllocationPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.coming_soon = COMING_SOON.read_text(encoding="utf-8")
        cls.catalog = CATALOG.read_text(encoding="utf-8")

    def test_coming_soon_projection_borrows_static_copy(self) -> None:
        self.assertIn("use std::borrow::Cow;", self.coming_soon)
        for field in (
            "id",
            "category",
            "category_label",
            "title",
            "detail",
            "status",
        ):
            self.assertRegex(
                self.coming_soon,
                rf"pub\s+{field}:\s+Cow<'static,\s*str>",
            )
        projection = function_body(self.coming_soon, "coming_soon_entries")
        self.assertIn("id: Cow::Borrowed(id)", projection)
        self.assertIn("category: Cow::Borrowed(category)", projection)
        self.assertIn("title: Cow::Borrowed(title)", projection)
        self.assertIn("detail: Cow::Borrowed(detail)", projection)

    def test_catalog_classifiers_do_not_allocate_lowercase_strings(self) -> None:
        maturity = function_body(self.catalog, "plugin_maturity_tone")
        category = function_body(self.catalog, "catalog_category_key")
        self.assertNotIn("to_ascii_lowercase", maturity)
        self.assertNotIn("to_ascii_lowercase", category)
        self.assertIn("contains_ascii_case_insensitive", maturity)
        self.assertIn("contains_ascii_case_insensitive", category)
        helper = function_body(self.catalog, "contains_ascii_case_insensitive")
        self.assertIn(".windows(needle.len())", helper)
        self.assertIn("eq_ignore_ascii_case", helper)

    def test_release_evidence_tracks_both_allocation_hotpaths(self) -> None:
        self.assertIn(
            "hub03_coming_soon_projection_release_benchmark_evidence",
            self.coming_soon,
        )
        self.assertIn(
            "PERF_RESULT hub03_coming_soon_projection",
            self.coming_soon,
        )
        self.assertIn("legacy_string_allocations=105", self.coming_soon)
        self.assertIn("optimized_string_allocations=15", self.coming_soon)
        self.assertIn("legacy_raw_ns={}", self.coming_soon)
        self.assertIn(
            "optimized_p95.saturating_mul(100)"
            " <= legacy_p95.saturating_mul(80)",
            self.coming_soon,
        )
        self.assertIn(
            "hub03_catalog_classifiers_release_benchmark_evidence",
            self.catalog,
        )
        self.assertIn(
            "PERF_RESULT hub03_catalog_classifiers",
            self.catalog,
        )
        self.assertIn("optimized_heap_allocations_per_call=0", self.catalog)
        self.assertIn("legacy_raw_ns={}", self.catalog)
        self.assertIn(
            "optimized_p95.saturating_mul(100)"
            " <= legacy_p95.saturating_mul(80)",
            self.catalog,
        )


if __name__ == "__main__":
    unittest.main()
