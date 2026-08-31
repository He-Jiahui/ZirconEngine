from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
ASSET_EDITOR_ROOT = ROOT / "zircon_editor/src/ui/asset_editor"


def function_body(source: str, start: str, end: str) -> str:
    offset = source.index(start)
    return source[offset : source.index(end, offset)]


class EditorUiAssetRootPerformanceContractTests(unittest.TestCase):
    def test_document_replay_skips_transactional_clone_without_commands(self) -> None:
        source = (ASSET_EDITOR_ROOT / "undo_stack.rs").read_text(encoding="utf-8")
        body = function_body(
            source,
            "    pub fn apply_to_document(&self, document: &mut UiAssetDocument)",
            "}\n\n#[derive(Clone, Debug, PartialEq, Eq)]",
        )

        fast_path = body.index("self.document_commands.is_empty()")
        clone = body.index("document.clone()")
        self.assertLess(fast_path, clone)
        self.assertIn("replay.apply_to_document(document)", body)

    def test_node_projection_borrows_roots_and_render_text_until_model_creation(self) -> None:
        source = (ASSET_EDITOR_ROOT / "node_projection.rs").read_text(encoding="utf-8")
        dirty = function_body(
            source,
            "fn mark_surface_roots_layout_dirty(",
            "fn project_ui_asset_editor_nodes(",
        )
        projection = function_body(
            source,
            "fn project_ui_asset_editor_nodes(",
            "fn asset_path(",
        )

        self.assertNotIn(".roots.clone()", dirty)
        self.assertIn("text: Option<&'a str>", source)
        self.assertIn("command.text.as_deref()", projection)
        self.assertIn(".and_then(|info| info.text)", projection)
        self.assertNotIn("info.text.clone()", projection)
        self.assertNotIn("text.clone()", projection)


if __name__ == "__main__":
    unittest.main()
