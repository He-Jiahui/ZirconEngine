"""Source and cost contract for direct Editor export tool cache keys."""

import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_editor/src/core/export/inventory.rs"
LOWER = ROOT / "zircon_editor/src/core/export/inventory/tests/tool_cache_key_tests.rs"


class Editor896ToolIdentityCacheKeyDirectTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.owner = cls.source.split("fn tool_cache_key(", 1)[1].split("\n}\n", 1)[0]

    def test_cache_key_streams_debug_program_and_version_args(self) -> None:
        self.assertIn("String::with_capacity(capacity)", self.owner)
        self.assertIn('write!(&mut cache_key, "{key}\\0{program:?}\\0")', self.owner)
        self.assertIn("for (index, argument) in version_args.iter().enumerate()", self.owner)
        self.assertIn("cache_key.push_str(argument)", self.owner)

    def test_cache_key_keeps_positional_empty_arg_delimiters_without_join(self) -> None:
        self.assertIn("if index > 0", self.owner)
        self.assertIn("cache_key.push('\\0')", self.owner)
        self.assertNotIn(".join(", self.owner)
        self.assertNotIn("collect::<", self.owner)

    def test_lower_parity_and_release_marker_are_wired(self) -> None:
        self.assertIn("mod tool_cache_key_tests;", self.source)
        lower = LOWER.read_text(encoding="utf-8")
        self.assertIn("editor896_tool_identity_cache_key_direct_preserves_exact_bytes", lower)
        self.assertIn("EDITOR896_TOOL_IDENTITY_CACHE_KEY_DIRECT_BENCH_V1", lower)
        self.assertIn("const SAMPLE_PAIRS: usize = 101;", lower)
        self.assertIn("legacy_p50_ns", lower)
        self.assertIn("optimized_p99_ns", lower)

    def test_version_arg_intermediate_join_model(self) -> None:
        keys = 4_096
        self.assertEqual(keys, 4_096)
        optimized_joins = keys if ".join(" in self.owner else 0
        self.assertEqual(optimized_joins, 0)


if __name__ == "__main__":
    unittest.main()
