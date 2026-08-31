from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
STYLE_ROOT = ROOT / "zircon_editor/src/ui/asset_editor/style"


class EditorUiAssetStylePerformanceContractTests(unittest.TestCase):
    def test_semantic_reads_walk_borrowed_path_segments_without_allocating(self) -> None:
        source = (STYLE_ROOT / "inspector_semantics.rs").read_text(encoding="utf-8")
        segments_start = source.index("fn path_segments")
        segments_end = source.index("fn value_map_literal", segments_start)
        segments = source[segments_start:segments_end]
        value_start = source.index("fn value_map_value")
        value_end = source.index("fn set_path_value", value_start)
        value_lookup = source[value_start:value_end]
        kind_start = source.index("fn semantic_node_kind")
        kind_end = source.index("fn set_value_in_map", kind_start)
        kind_lookup = source[kind_start:kind_end]

        self.assertIn("impl Iterator<Item = &str>", segments)
        self.assertIn("path.split('.')", segments)
        self.assertNotIn("collect", segments)
        self.assertNotIn("String", segments)
        self.assertIn("let mut segments = path_segments(path)", value_lookup)
        self.assertNotIn("split_path(path)", value_lookup)
        self.assertNotIn("Vec<String>", value_lookup)
        self.assertIn("Option<&str>", kind_lookup)
        self.assertNotIn(".map(str::to_string)", kind_lookup)
        self.assertNotIn("node.widget_type.clone()", kind_lookup)

    def test_theme_helper_refactor_presence_does_not_build_labels_or_rescan(self) -> None:
        source = (STYLE_ROOT / "theme_authoring.rs").read_text(encoding="utf-8")
        can_start = source.index("pub(crate) fn can_prune_duplicate_local_theme_overrides")
        can_end = source.index("pub(crate) fn prune_duplicate_local_theme_overrides", can_start)
        can_prune = source[can_start:can_end]
        actions_start = source.index("pub(crate) fn theme_rule_helper_actions")
        actions_end = source.index("pub(crate) fn adopt_imported_theme_token", actions_start)
        actions = source[actions_start:actions_end]

        self.assertIn("theme_refactor_actions(document, imported_styles)", can_prune)
        self.assertNotIn("build_theme_refactor_items", can_prune)
        self.assertEqual(actions.count("theme_refactor_actions(document, imported_styles)"), 1)
        self.assertNotIn("can_prune_duplicate_local_theme_overrides", actions)

    def test_theme_rule_actions_index_local_rules_once(self) -> None:
        source = (
            STYLE_ROOT / "theme_authoring" / "action_projection.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("fn local_rule_index", source)
        for start_marker, end_marker in (
            (
                "pub(super) fn build_adopt_imported_theme_rule_actions",
                "pub(super) fn local_rule_blocks",
            ),
            (
                "pub(super) fn active_cascade_rules",
                "pub(super) fn resolve_local_clone_token_renames",
            ),
        ):
            start = source.index(start_marker)
            end = source.index(end_marker, start)
            function = source[start:end]
            self.assertIn("local_rule_index(document)", function)
            self.assertRegex(function, r"local_rules\s*\.get")


if __name__ == "__main__":
    unittest.main()
