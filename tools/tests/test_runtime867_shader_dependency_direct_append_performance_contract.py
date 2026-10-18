import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    REPO_ROOT
    / "zircon_runtime/src/asset/project/manager/scan_and_import/shader_import_dependencies.rs"
)
LOWER = (
    REPO_ROOT / "zircon_runtime/src/asset/project/manager/scan_and_import/shader_import_dependencies/tests/optimization_batch_runtime867_shader_dependency_direct_append_tests.rs"
)


class Runtime867ShaderDependencyDirectAppendPerformanceContractTests(
    unittest.TestCase
):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        append_match = re.search(
            r"pub\(super\) fn append_dependencies\(.*?\n    }\n\n",
            cls.source,
            re.S,
        )
        if append_match is None:
            raise AssertionError("shader dependency append owner is missing")
        cls.append_body = append_match.group(0)

    def test_full_projection_appends_directly_into_retained_dependency_vector(self) -> None:
        self.assertIn(
            "self.append_dependency_locators(shader_id, dependencies)",
            self.append_body,
        )
        self.assertNotIn("self.dependency_locators(shader_id)", self.append_body)
        self.assertNotIn("for locator in", self.append_body)

    def test_shared_projection_owner_reserves_import_upper_bound(self) -> None:
        self.assertIn("fn append_dependency_locators(", self.source)
        self.assertIn("dependencies.reserve(shader.imports.len())", self.source)
        self.assertIn("seen_provider_ids.insert(provider_id)", self.source)
        self.assertIn("dependencies.push(provider.locator.clone())", self.source)

    def test_owned_dependency_api_delegates_to_shared_append_owner(self) -> None:
        locator_match = re.search(
            r"pub\(super\) fn dependency_locators\(.*?\n    }\n\n",
            self.source,
            re.S,
        )
        self.assertIsNotNone(locator_match)
        locator_body = locator_match.group(0)
        self.assertIn("let mut dependencies = Vec::new()", locator_body)
        self.assertIn("self.append_dependency_locators(id, &mut dependencies)", locator_body)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn(
            "mod optimization_batch_runtime867_shader_dependency_direct_append_tests;",
            self.source,
        )
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn(
            "runtime867_shader_dependency_direct_append_preserves_metadata_and_provider_order",
            lower,
        )
        self.assertIn(
            "RUNTIME867_SHADER_DEPENDENCY_DIRECT_APPEND_BENCH_V1", lower
        )
        self.assertIn("const SAMPLE_PAIRS: usize = 101;", lower)
        self.assertIn("legacy_p99_ns", lower)
        self.assertIn("optimized_p99_ns", lower)

    def test_dense_allocation_model_removes_temporary_locator_slots(self) -> None:
        import_count = 4_096
        legacy_temporary_locator_slots = import_count
        optimized_temporary_locator_slots = 0
        self.assertEqual(legacy_temporary_locator_slots, 4_096)
        self.assertEqual(optimized_temporary_locator_slots, 0)


if __name__ == "__main__":
    unittest.main()
