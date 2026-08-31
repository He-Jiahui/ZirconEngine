from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
INSTANTIATE = ROOT / "zircon_editor/src/ui/asset_editor/palette/instantiate.rs"


def function_region(source: str, start: str, end: str) -> str:
    offset = source.index(start)
    return source[offset : source.index(end, offset)]


class EditorUiAssetPaletteBaseNodeIdPerformanceContractTests(unittest.TestCase):
    def test_base_node_id_normalizes_in_one_preallocated_buffer(self) -> None:
        source = INSTANTIATE.read_text(encoding="utf-8")
        normalization = function_region(
            source,
            "fn base_node_id(",
            "fn new_child_mount_with_placement(",
        )

        self.assertIn("String::with_capacity(label.len())", normalization)
        self.assertIn("for ch in label.chars()", normalization)
        self.assertIn("ch.to_ascii_lowercase()", normalization)
        self.assertIn("normalized.truncate(trimmed_len);", normalization)
        self.assertNotIn("collect::<String>()", normalization)
        self.assertNotIn("trim_matches", normalization)
        self.assertNotIn("to_ascii_lowercase();", normalization)

        benchmark = (ROOT / "zircon_editor/src/ui/asset_editor/palette/instantiate/base_node_id_tests.rs").read_text(encoding="utf-8")
        self.assertIn("RUNTIME75_PALETTE_BASE_NODE_ID_BENCH_V1", benchmark)
        self.assertIn("legacy_allocations_per_id=2", benchmark)
        self.assertIn("optimized_allocations_per_id=1", benchmark)
        self.assertIn("legacy_p95_ns.saturating_mul(80)", benchmark)


if __name__ == "__main__":
    unittest.main()
