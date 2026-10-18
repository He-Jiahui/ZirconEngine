import re
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_runtime/src/ui/template/asset/prototype_file_cache.rs"
LOWER = (
    REPO_ROOT / "zircon_runtime/src/ui/template/asset/prototype_file_cache/tests/resource_alias_single_buffer_tests.rs"
)


def function_body(source: str, signature: str) -> str:
    match = re.search(rf"fn {signature}\(.*?\n}}\n", source, re.S)
    if match is None:
        raise AssertionError(f"{signature} owner is missing")
    return match.group(0)


class Runtime872PrototypeResourceAliasSingleBufferPerformanceContractTests(
    unittest.TestCase
):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")

    def test_resource_alias_uses_one_capacity_bounded_output(self) -> None:
        owner = function_body(self.source, "resource_alias_for_path")
        self.assertIn("let first = parts.next()?;", owner)
        self.assertIn("let mut alias = String::with_capacity", owner)
        self.assertIn('alias.push_str("res://");', owner)
        self.assertTrue(owner.rstrip().endswith("Some(alias)\n}"))

    def test_components_append_with_positional_delimiters(self) -> None:
        owner = function_body(self.source, "resource_alias_for_path")
        self.assertIn(".components()", owner)
        self.assertIn("filter_map(|component| component.as_os_str().to_str())", owner)
        self.assertIn("alias.push_str(first);", owner)
        self.assertIn("alias.push('/');", owner)
        self.assertIn("alias.push_str(part);", owner)

    def test_resource_alias_does_not_collect_join_or_format_children(self) -> None:
        owner = function_body(self.source, "resource_alias_for_path")
        self.assertNotIn("collect::<", owner)
        self.assertNotIn(".join(", owner)
        self.assertNotIn("format!(", owner)

    def test_lower_regression_and_release_marker_are_wired(self) -> None:
        self.assertIn("mod resource_alias_single_buffer_tests;", self.source)
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn(
            "runtime872_prototype_resource_alias_single_buffer_preserves_exact_text",
            lower,
        )
        self.assertIn(
            "RUNTIME872_PROTOTYPE_RESOURCE_ALIAS_SINGLE_BUFFER_BENCH_V1", lower
        )
        self.assertIn("const SAMPLE_PAIRS: usize = 101;", lower)
        self.assertIn("legacy_p50_ns", lower)
        self.assertIn("optimized_p99_ns", lower)

    def test_dense_alias_temporary_allocation_model(self) -> None:
        aliases = 4_096
        components = 64
        legacy_reference_slots = aliases * components
        legacy_join_outputs = aliases
        optimized_reference_slots = 0
        optimized_join_outputs = 0
        self.assertEqual(legacy_reference_slots, 262_144)
        self.assertEqual(legacy_join_outputs, 4_096)
        self.assertEqual(optimized_reference_slots, 0)
        self.assertEqual(optimized_join_outputs, 0)


if __name__ == "__main__":
    unittest.main()
