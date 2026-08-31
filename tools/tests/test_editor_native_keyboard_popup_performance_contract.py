from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
DISCOVERY = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/"
    "native_keyboard/target/discovery.rs"
)
SEARCH = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/"
    "native_keyboard/target/search.rs"
)
HIT_INDEX = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/"
    "surface_hit_test/template_node/index.rs"
)


class EditorNativeKeyboardPopupPerformanceContractTests(unittest.TestCase):
    def test_discovery_reuses_generation_popup_candidates(self) -> None:
        source = DISCOVERY.read_text(encoding="utf-8")

        self.assertIn(
            "let popup_rows = generation.workbench_hit_index().popup_rows();", source
        )
        self.assertIn("popup_rows: &[usize]", source)
        self.assertIn("for row in popup_rows.iter().rev().copied()", source)
        self.assertNotIn("for row in (0..nodes.row_count()).rev()", source)

    def test_discovery_borrows_wide_template_nodes(self) -> None:
        source = DISCOVERY.read_text(encoding="utf-8")

        self.assertIn("let Some(node) = nodes.get(row)", source)
        self.assertNotIn("nodes.row_data(row)", source)
        self.assertIn("pub(crate) fn popup_rows(&self) -> &[usize]", HIT_INDEX.read_text(encoding="utf-8"))

    def test_candidate_prefix_matching_does_not_allocate_lowercase_strings(self) -> None:
        source = SEARCH.read_text(encoding="utf-8")

        self.assertIn(".flat_map(char::to_lowercase)", source)
        self.assertNotIn("value.as_str().to_lowercase()", source)


if __name__ == "__main__":
    unittest.main()
