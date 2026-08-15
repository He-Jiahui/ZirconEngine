from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE_SYNC = (
    ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "asset_editor"
    / "source"
    / "source_sync.rs"
)
SOURCE_BUFFER = (
    ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "asset_editor"
    / "source"
    / "source_buffer.rs"
)
NAVIGATION_STATE = (
    ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "asset_editor"
    / "session"
    / "navigation_state.rs"
)
PRESENTATION_DIR = (
    ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "asset_editor"
    / "session"
    / "presentation"
)
PRESENTATION_PANE = PRESENTATION_DIR / "pane.rs"
PRESENTATION_SOURCE = PRESENTATION_DIR / "source.rs"
LIFECYCLE_OUTLINE_CACHE = (
    ROOT
    / "zircon_editor"
    / "src"
    / "ui"
    / "asset_editor"
    / "session"
    / "lifecycle"
    / "source_outline_cache.rs"
)


def function_body(source: str, signature: str) -> str:
    function_start = source.index(signature)
    body_start = source.index("{", function_start)
    depth = 0
    for offset, character in enumerate(source[body_start:], start=body_start):
        if character == "{":
            depth += 1
        elif character == "}":
            depth -= 1
            if depth == 0:
                return source[body_start : offset + 1]
    raise AssertionError(f"unterminated function: {signature}")


class UiAssetSourceOutlineIndexContractTests(unittest.TestCase):
    def test_outline_index_is_the_single_source_scan_owner(self) -> None:
        source = SOURCE_SYNC.read_text(encoding="utf-8")

        self.assertIn("struct UiAssetSourceOutlineIndex", source)
        self.assertIn("line_segments: Vec<UiAssetSourceOutlineLineSegment>", source)
        self.assertIn("fn build_source_outline_index", source)
        self.assertIn("fn build_line_segments", source)
        self.assertIn("struct UiAssetSourceOutlineCache", source)
        self.assertIn("index: Arc<UiAssetSourceOutlineIndex>", source)
        self.assertIn("fn shares_index_with(&self, other: &Self)", source)
        self.assertIn("fn is_current(&self, source_revision: u64)", source)
        self.assertIn("fn header_path_contains", source)
        self.assertIn("if entry.end_line < entry.line", source)
        self.assertIn(
            "outline_index_skips_empty_tree_ranges_instead_of_clamping_them_to_a_node_line",
            source,
        )
        self.assertIn(
            "outline_index_uses_the_complete_parent_path_for_tree_wrappers",
            source,
        )
        self.assertIn('for segment in node_path.split(\'.\')', source)
        self.assertIn('if segment == "node"', source)
        self.assertNotIn('match_indices(".node")', source)
        self.assertNotIn("active_headers.retain", source)
        self.assertNotIn('format!("{}.", active_header.path)', source)

    def test_node_and_line_queries_consume_the_immutable_index(self) -> None:
        source = SOURCE_SYNC.read_text(encoding="utf-8")

        self.assertIn("fn entry_for_node(&self, node_id: &str)", source)
        self.assertIn("fn entry_for_line(&self, line: usize)", source)
        self.assertIn("fn node_id_for_line(&self, line: usize)", source)
        self.assertIn(".line_segments\n            .partition_point", source)
        self.assertNotIn(".filter(|entry| line >= entry.line", source)
        self.assertNotIn("fn build_source_outline(", source)
        self.assertNotIn("fn source_outline_node_id_for_line(", source)

    def test_presentation_uses_the_precompiled_node_entry_index(self) -> None:
        source = SOURCE_SYNC.read_text(encoding="utf-8")
        presentation = PRESENTATION_SOURCE.read_text(encoding="utf-8")

        self.assertIn("fn index_for_node(&self, node_id: &str)", source)
        self.assertIn("source_outline.index_for_node(node_id)", presentation)
        self.assertNotIn(
            ".entries()\n                    .iter()\n                    .position(|entry| entry.node_id.as_str() == node_id)",
            presentation,
        )

    def test_navigation_and_presentation_use_one_index_per_operation(self) -> None:
        navigation = NAVIGATION_STATE.read_text(encoding="utf-8")
        presentation = PRESENTATION_SOURCE.read_text(encoding="utf-8")
        pane = PRESENTATION_PANE.read_text(encoding="utf-8")
        source_buffer = SOURCE_BUFFER.read_text(encoding="utf-8")
        lifecycle_cache = LIFECYCLE_OUTLINE_CACHE.read_text(encoding="utf-8")

        self.assertIn("build_source_outline_index", navigation)
        self.assertNotIn("source_outline_entry_for_node", navigation)
        self.assertNotIn("source_outline_node_id_for_line", navigation)
        self.assertIn("fn source_outline_index(&self)", navigation)
        self.assertIn("fn roundtrip_source_outline_index(&self)", navigation)
        self.assertIn("source_outline_caches_share_index", navigation)
        self.assertIn("source_outline_cache", navigation)
        self.assertIn("last_valid_source_outline_cache", navigation)
        self.assertIn("fn set_source_cursor_for_selected_node_line(", navigation)
        for selection_method in (
            "pub fn select_source_outline_index",
            "pub fn select_source_line",
            "pub fn select_source_byte_offset",
        ):
            self.assertEqual(
                function_body(navigation, selection_method).count(
                    "build_source_outline_index("
                ),
                0,
            )
        self.assertEqual(
            function_body(navigation, "pub(super) fn source_outline_index").count(
                "build_source_outline_index("
            ),
            1,
        )
        self.assertIn("let source_outline = self.roundtrip_source_outline_index();", presentation)
        self.assertNotIn("build_source_outline_index", presentation)
        self.assertRegex(
            presentation,
            re.compile(r"build_source_selection_summary\(\s*&source_outline,"),
        )
        self.assertIn("source_outline_items: source.outline_items", pane)
        self.assertIn(
            ".entries()\n                .iter()\n                .map(|entry| format!(\"line {}",
            presentation,
        )
        self.assertIn("revision: u64", source_buffer)
        self.assertIn("if self.text == text", source_buffer)
        self.assertIn("self.revision = self.revision.wrapping_add(1);", source_buffer)
        self.assertIn(
            "source_generation_reuses_the_outline_across_presentation_and_navigation",
            navigation,
        )
        self.assertIn("initial_source_outline_state", lifecycle_cache)
        self.assertIn("fn refresh_valid_source_outline_caches", lifecycle_cache)
        self.assertEqual(
            function_body(
                lifecycle_cache, "pub(super) fn refresh_valid_source_outline_caches"
            ).count("build_source_outline_index("),
            1,
        )
        self.assertIn(
            "let outline = Arc::new(build_source_outline_index(", lifecycle_cache
        )
        self.assertIn(".replace_shared_built(source_revision, Arc::clone(&outline));", lifecycle_cache)
        self.assertIn(
            ".replace_shared(session.last_valid_source_generation, outline);",
            lifecycle_cache,
        )
        self.assertIn('entry_for_node("root")', navigation)
        self.assertIn('"invalid source draft"', navigation)


if __name__ == "__main__":
    unittest.main()
