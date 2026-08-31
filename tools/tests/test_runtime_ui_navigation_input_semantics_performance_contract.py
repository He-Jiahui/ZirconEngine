import pathlib
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
REBUILD = ROOT / "zircon_runtime/src/ui/surface/surface/rebuild/incremental.rs"
NAVIGATION = ROOT / "zircon_runtime/src/ui/surface/navigation_index.rs"
NAVIGATION_SEMANTICS = ROOT / "zircon_runtime/src/ui/surface/navigation_index/semantics.rs"


class RuntimeUiNavigationInputSemanticsPerformanceContractTests(unittest.TestCase):
    def setUp(self) -> None:
        self.rebuild = REBUILD.read_text(encoding="utf-8")
        self.navigation = NAVIGATION.read_text(encoding="utf-8") + NAVIGATION_SEMANTICS.read_text(
            encoding="utf-8"
        )

    def test_all_local_semantic_domains_use_one_retained_gate(self) -> None:
        self.assertIn("navigation_index_needs_semantics_rebuild", self.rebuild)
        self.assertNotIn(
            "let navigation_semantics_changed =\n            dirty.hit_test || dirty.input",
            self.rebuild,
        )
        self.assertNotIn(
            "let navigation_semantics_changed = dirty.style || dirty.text || dirty.visible_range;",
            self.rebuild,
        )
        self.assertIn("ui.navigation_index.input_semantics_skip_count", self.rebuild)
        self.assertIn(
            "ui.navigation_index.style_text_visible_range_semantics_skip_count",
            self.rebuild,
        )

    def test_gate_checks_the_complete_retained_navigation_signature(self) -> None:
        self.assertIn("needs_semantics_rebuild", self.navigation)
        self.assertIn("resolved_navigation_context", self.navigation)
        self.assertIn("previous.focus_candidate != node.is_focus_candidate()", self.navigation)
        self.assertIn("previous.tab_order != tab_index.order", self.navigation)
        self.assertIn("previous.directional.as_ref() != node.navigation.directional.as_ref()", self.navigation)
        self.assertIn("subtree_navigation_authority", self.navigation)
        self.assertIn("is_active_mui_modal_focus_scope(node)", self.navigation)

    def test_pointer_only_changes_are_documented_as_navigation_independent(self) -> None:
        self.assertIn(
            "Local input, style, text, and visible-range changes can preserve the retained index",
            self.navigation,
        )


if __name__ == "__main__":
    unittest.main()
