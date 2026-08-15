from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
ASSET_EDITOR = ROOT / "zircon_editor" / "src" / "ui" / "asset_editor"
PREVIEW = ASSET_EDITOR / "preview" / "preview_projection.rs"
PALETTE_STATE = ASSET_EDITOR / "session" / "palette_state.rs"
LIFECYCLE = ASSET_EDITOR / "session" / "lifecycle.rs"
PREVIEW_STATE = ASSET_EDITOR / "session" / "preview_state.rs"
SESSION = ASSET_EDITOR / "session" / "ui_asset_editor_session.rs"
RESOLUTION = ASSET_EDITOR / "tree" / "palette_drop" / "resolution.rs"


class UiAssetPreviewHitIndexContractTests(unittest.TestCase):
    def test_drag_hit_testing_uses_a_compact_projection_independent_index(self) -> None:
        preview = PREVIEW.read_text(encoding="utf-8")
        palette_state = PALETTE_STATE.read_text(encoding="utf-8")
        session = SESSION.read_text(encoding="utf-8")
        resolution = RESOLUTION.read_text(encoding="utf-8")

        self.assertIn("pub(crate) struct UiAssetPreviewHitIndex", preview)
        self.assertIn("pub(crate) struct UiAssetPreviewHitNode", preview)
        self.assertIn("pub(crate) fn build_preview_hit_index", preview)
        self.assertIn("node_id: document_node_id\n                .or(control_id)", preview)
        self.assertIn("preview_hit_index: Option<UiAssetPreviewHitIndex>", session)
        self.assertIn("fn ensure_preview_hit_index(&mut self)", palette_state)
        self.assertIn("self.ensure_preview_hit_index();", palette_state)
        self.assertIn("self.preview_hit_index.as_ref()?", palette_state)
        self.assertNotIn("build_preview_projection(", palette_state)
        self.assertIn("hit_index: &UiAssetPreviewHitIndex", resolution)
        hit_node = preview[
            preview.index("pub(crate) struct UiAssetPreviewHitNode") : preview.index(
                "impl Default for UiAssetPreviewProjection"
            )
        ]
        for presentation_field in ("label:", "kind:", "selected:", "depth:", "z_index:"):
            self.assertNotIn(presentation_field, hit_node)
        drag_resolver = palette_state[
            palette_state.index("fn resolve_palette_drag_target(") :
        ]
        self.assertIn("if self.selected_palette_entry.is_none()", drag_resolver)
        self.assertIn("self.ensure_preview_hit_index();", drag_resolver)
        self.assertIn("self.selected_palette_entry.as_ref()?", drag_resolver)
        self.assertNotIn(".clone()", drag_resolver[:900])
        ensure_index = palette_state[
            palette_state.index("fn ensure_preview_hit_index(&mut self)") :
        ]
        self.assertIn("if self.preview_hit_index.is_none()", ensure_index)
        self.assertIn("preview_hit_index_build_count", ensure_index)
        self.assertIn(
            "palette_drag_reuses_the_hit_index_until_preview_or_document_rebuild",
            palette_state,
        )
        self.assertIn("const PREVIEW_HIT_INDEX_LAYOUT", palette_state)
        for assertion in (
            'session.rebuild_preview_snapshot().expect("preview rebuild")',
            "set_preview_preset(UiAssetPreviewPreset::Dialog)",
            ".apply_valid_document(document)",
        ):
            self.assertIn(assertion, palette_state)

    def test_hit_index_is_invalidated_for_every_preview_or_document_rebuild(self) -> None:
        lifecycle = LIFECYCLE.read_text(encoding="utf-8")
        preview_state = PREVIEW_STATE.read_text(encoding="utf-8")

        self.assertIn("fn invalidate_preview_hit_index(&mut self)", preview_state)
        self.assertIn("self.preview_hit_index = None;", preview_state)
        for entry_point in (
            "pub(super) fn rebuild_preview_snapshot(&mut self)",
            "pub(super) fn refresh_preview_for_current_preset(",
            "fn apply_valid_projection_document(",
        ):
            start = lifecycle.index(entry_point)
            self.assertIn("self.invalidate_preview_hit_index();", lifecycle[start : start + 500])


if __name__ == "__main__":
    unittest.main()
