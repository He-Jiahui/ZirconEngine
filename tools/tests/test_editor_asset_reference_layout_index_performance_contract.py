from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
REFERENCE_ROWS = ROOT / "zircon_editor/src/ui/layouts/views/asset_reference_rows.rs"


def function_body(source: str, signature: str) -> str:
    start = source.index(signature)
    end = source.index("\nfn ", start + len(signature))
    return source[start:end]


class EditorAssetReferenceLayoutIndexPerformanceContractTests(unittest.TestCase):
    def test_kind_width_index_is_contiguous_and_linear(self) -> None:
        source = REFERENCE_ROWS.read_text(encoding="utf-8")
        body = function_body(source, "fn reference_kind_slot_widths(")

        self.assertIn("Vec<Option<f32>>", body)
        self.assertIn("if index >= nodes.len()", body)
        self.assertIn("widths.resize(index + 1, None)", body)
        self.assertIn("widths[index] = Some", body)
        self.assertNotIn("BTreeMap", source)
        self.assertNotIn("widths.insert(", body)

    def test_layout_uses_constant_time_index_lookup_and_one_default_width(self) -> None:
        source = REFERENCE_ROWS.read_text(encoding="utf-8")
        body = function_body(source, "fn apply_asset_reference_list_layout(")

        self.assertIn("let mut default_kind_width = None", body)
        self.assertIn(".and_then(|width| *width)", body)
        self.assertIn("default_kind_width", body)
        self.assertIn(".get_or_insert_with(|| reference_kind_slot_width", body)
        self.assertNotIn("unwrap_or_else(|| reference_kind_slot_width", body)


if __name__ == "__main__":
    unittest.main()
