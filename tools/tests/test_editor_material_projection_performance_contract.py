from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


class EditorMaterialProjectionPerformanceContractTests(unittest.TestCase):
    def test_diagnostic_count_index_borrows_feature_names(self) -> None:
        source = (
            ROOT
            / "zircon_editor/src/ui/material_editor/renderer_data_projection.rs"
        ).read_text(encoding="utf-8")
        start = source.index("fn diagnostic_counts_by_feature(")
        end = source.index("fn diagnostic_row(", start)
        body = source[start:end]

        self.assertIn("HashMap<&str, usize>", body)
        self.assertIn("diagnostic.feature.as_str()", body)
        self.assertNotIn("diagnostic.feature.clone()", body)


if __name__ == "__main__":
    unittest.main()
