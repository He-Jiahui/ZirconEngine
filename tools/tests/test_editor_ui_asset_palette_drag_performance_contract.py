from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]


def function_body(source: str, start: str, end: str) -> str:
    return source[source.index(start) : source.index(end, source.index(start))]


class EditorUiAssetPaletteDragPerformanceContractTests(unittest.TestCase):
    def test_pointer_move_reuses_the_selected_palette_entry(self) -> None:
        source = (
            ROOT
            / "zircon_editor/src/ui/asset_editor/session/palette_state.rs"
        ).read_text(encoding="utf-8")
        body = function_body(
            source,
            "    fn resolve_palette_drag_target(",
            "    pub(super) fn selected_palette_drag_target(",
        )

        self.assertIn("self.selected_palette_entry.as_ref()?", body)
        self.assertNotIn("build_palette_entries(", body)

    def test_palette_drop_reuses_the_selected_palette_entry(self) -> None:
        source = (
            ROOT
            / "zircon_editor/src/ui/asset_editor/session/palette_state.rs"
        ).read_text(encoding="utf-8")
        body = function_body(
            source,
            "    fn insert_selected_palette_item_with_plan(",
            "    fn move_selected_node(",
        )

        self.assertIn("self.selected_palette_entry.clone()", body)
        self.assertNotIn("build_palette_entries(", body)

    def test_document_revalidation_refreshes_the_cached_entry(self) -> None:
        session = (
            ROOT
            / "zircon_editor/src/ui/asset_editor/session/ui_asset_editor_session.rs"
        ).read_text(encoding="utf-8")
        lifecycle = (
            ROOT / "zircon_editor/src/ui/asset_editor/session/lifecycle.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("selected_palette_entry: Option<UiAssetPaletteEntry>", session)
        self.assertIn("self.selected_palette_entry = self", lifecycle)
        self.assertIn(".selected_palette_index", lifecycle)
        self.assertIn(".and_then(|index| palette_entries.get(index).cloned())", lifecycle)

    def test_pointer_move_validates_one_plan_per_drag_resolution(self) -> None:
        source = (
            ROOT
            / "zircon_editor/src/ui/asset_editor/tree/palette_drop/resolution.rs"
        ).read_text(encoding="utf-8")
        component = function_body(
            source,
            "fn build_component_palette_drag_resolution(",
            "fn build_native_palette_drag_resolution(",
        )
        native = function_body(
            source,
            "fn build_native_palette_drag_resolution(",
            "pub(crate) fn build_palette_drag_slot_target_overlays(",
        )

        for body in (component, native):
            self.assertNotIn("finalize_palette_insert_plan(", body)
            self.assertEqual(body.count("palette_insert_plan_is_valid("), 1)

    def test_component_drag_reuses_resolved_slot_targets(self) -> None:
        source = (
            ROOT
            / "zircon_editor/src/ui/asset_editor/tree/palette_drop/resolution.rs"
        ).read_text(encoding="utf-8")
        component = function_body(
            source,
            "fn build_component_palette_drag_resolution(",
            "fn build_native_palette_drag_resolution(",
        )

        self.assertIn("point_within_overlay(", component)
        self.assertNotIn("component_mount_for_node(", component)

    def test_component_slot_lookup_borrows_mount_names_and_reads_import_once(self) -> None:
        source = (
            ROOT
            / "zircon_editor/src/ui/asset_editor/tree/palette_drop/resolution.rs"
        ).read_text(encoding="utf-8")
        definition = function_body(
            source,
            "fn component_definition_for_node<'a>(",
            "fn available_component_slots(",
        )
        available = function_body(
            source,
            "fn available_component_slots(",
            "fn contains_slot_semantics(",
        )

        self.assertEqual(definition.count(".get(reference)"), 1)
        self.assertIn("BTreeMap::<&str, usize>", available)
        self.assertIn("child.mount.as_deref().unwrap_or_default()", available)
        self.assertNotIn("child.mount.clone()", available)

    def test_pointer_move_reconciles_palette_chooser_by_ownership(self) -> None:
        session = (
            ROOT
            / "zircon_editor/src/ui/asset_editor/session/palette_state.rs"
        ).read_text(encoding="utf-8")
        update = function_body(
            session,
            "    pub fn update_palette_drag_target(",
            "    pub fn clear_palette_drag_target(",
        )
        chooser = (
            ROOT
            / "zircon_editor/src/ui/asset_editor/palette_target_chooser.rs"
        ).read_text(encoding="utf-8")

        self.assertIn("self.palette_target_chooser.take()", update)
        self.assertNotIn("self.palette_target_chooser == next", update)
        self.assertNotIn("previous.clone()", chooser)


if __name__ == "__main__":
    unittest.main()
