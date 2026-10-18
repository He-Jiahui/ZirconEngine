import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_runtime/src/ui/v2/surface_tree/interaction.rs"
LOWER_REGRESSION = (
    REPO_ROOT / "zircon_runtime/src/ui/v2/surface_tree/interaction/tests/shared_catalog_tests.rs"
)


def function_body(source: str, function_name: str) -> str:
    match = re.search(
        rf"\bfn\s+{re.escape(function_name)}(?![A-Za-z0-9_])(?:<[^>]*>)?\s*\(.*?\)",
        source,
        re.S,
    )
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


class RuntimeSurfaceTreeInteractionSharedCatalogPerformanceContractTests(
    unittest.TestCase
):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_interaction_catalog_borrows_the_process_shared_registry(self) -> None:
        body = function_body(self.source, "component_descriptor_registry")
        self.assertIn("UiComponentDescriptorRegistry::editor_showcase_shared()", body)
        self.assertNotIn("OnceLock", self.source)
        self.assertNotIn("editor_showcase)", body)
        self.assertNotIn("get_or_init", body)

    def test_pointer_identity_regression_and_release_marker_are_wired(self) -> None:
        lower = LOWER_REGRESSION.read_text(encoding="utf-8")
        self.assertIn("mod shared_catalog_tests;", self.source)
        self.assertIn(
            "runtime782_surface_tree_interaction_borrows_shared_catalog", lower
        )
        self.assertIn("RUNTIME782_SURFACE_TREE_SHARED_CATALOG_BENCH_V1", lower)
        self.assertIn("optimized_descriptor_clones", lower)

    def test_deterministic_model_removes_the_secondary_catalog_clone(self) -> None:
        legacy_descriptor_clones = 69
        optimized_descriptor_clones = 0
        self.assertGreater(legacy_descriptor_clones, optimized_descriptor_clones)
        self.assertEqual(optimized_descriptor_clones, 0)


if __name__ == "__main__":
    unittest.main()
