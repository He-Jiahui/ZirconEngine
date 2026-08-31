from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
EXTENSION_INPUTS = ROOT / (
    "zircon_runtime/src/builtin/runtime_modules/assembly/extension_inputs.rs"
)


def function_region(source: str, start: str, end: str) -> str:
    offset = source.index(start)
    return source[offset : source.index(end, offset)]


class Runtime39BorrowedShaderSourceDedupPerformanceContractTests(unittest.TestCase):
    def test_shader_sources_are_borrowed_until_after_deduplication(self) -> None:
        source = EXTENSION_INPUTS.read_text(encoding="utf-8")
        collection = function_region(
            source,
            "fn collect_shader_module_sources(",
            "#[cfg(all(test, feature = \"graphics\"))]",
        )

        self.assertIn(
            ".flat_map(|registry| registry.shader_module_sources().iter())",
            collection,
        )
        dedup = collection.index(".filter(|source|")
        clone = collection.index(".cloned()", dedup)
        collect = collection.index(".collect()", clone)
        self.assertLess(dedup, clone)
        self.assertLess(clone, collect)
        self.assertNotIn("shader_module_sources().iter().cloned()", collection)

    def test_deduplication_key_borrows_all_string_fields(self) -> None:
        source = EXTENSION_INPUTS.read_text(encoding="utf-8")
        collection = function_region(
            source,
            "fn collect_shader_module_sources(",
            "#[cfg(all(test, feature = \"graphics\"))]",
        )

        for field in ("owner_id", "import_path", "content_hash"):
            self.assertIn(f"source.{field}.as_str()", collection)
            self.assertNotIn(f"source.{field}.clone()", collection)

    def test_first_seen_deduplication_behavior_remains_covered_by_rust(self) -> None:
        source = EXTENSION_INPUTS.read_text(encoding="utf-8")

        self.assertIn(
            "fn identical_feature_extension_shader_modules_are_collected_once()",
            source,
        )
        self.assertIn("collect_shader_module_sources(&[&first, &second])", source)


if __name__ == "__main__":
    unittest.main()
