from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
PALETTE_ROOT = ROOT / "zircon_editor/src/ui/asset_editor/palette"
SESSION_ROOT = ROOT / "zircon_editor/src/ui/asset_editor/session"


def function_body(source: str, start: str, end: str) -> str:
    offset = source.index(start)
    return source[offset : source.index(end, offset)]


class EditorUiAssetPalettePerformanceContractTests(unittest.TestCase):
    def test_palette_catalog_is_published_per_document_or_import_generation(self) -> None:
        catalog = (PALETTE_ROOT / "catalog.rs").read_text(encoding="utf-8")
        palette_module = (PALETTE_ROOT / "mod.rs").read_text(encoding="utf-8")
        session = (SESSION_ROOT / "ui_asset_editor_session.rs").read_text(
            encoding="utf-8"
        )
        lifecycle = (SESSION_ROOT / "lifecycle.rs").read_text(encoding="utf-8")
        lifecycle_palette_catalog = (
            SESSION_ROOT / "lifecycle/palette_catalog.rs"
        ).read_text(encoding="utf-8")
        palette_state = (SESSION_ROOT / "palette_state.rs").read_text(
            encoding="utf-8"
        )
        presentation = "\n".join(
            path.read_text(encoding="utf-8")
            for path in sorted((SESSION_ROOT / "presentation").glob("*.rs"))
        )
        promotion = (SESSION_ROOT / "promotion_state.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("struct UiAssetPaletteCatalog", catalog)
        self.assertIn("Arc<[UiAssetPaletteEntry]>", catalog)
        self.assertIn("reference_imports: Arc<BTreeMap<String, UiAssetDocument>>", catalog)
        self.assertIn("fn reference_imports(&self)", catalog)
        self.assertFalse((PALETTE_ROOT / "build.rs").exists())
        self.assertNotIn("build_palette_entries", palette_module)
        self.assertIn("palette_catalog: UiAssetPaletteCatalog", session)
        self.assertIn("fn refresh_palette_catalog(", lifecycle_palette_catalog)
        self.assertIn("palette_catalog::refresh_palette_catalog(self)?;", lifecycle)
        self.assertNotIn("self.compiler_imports.widgets", palette_state)
        self.assertIn("build_v2(&document, &v2_compiler_imports.widgets)", lifecycle)
        self.assertIn("v2_palette_reference_imports", lifecycle_palette_catalog)
        self.assertNotIn("build_palette_entries", lifecycle)
        for source in (palette_state, presentation, promotion):
            self.assertIn("self.palette_catalog", source)
            self.assertIn("self.palette_catalog.reference_imports()", source)
            self.assertNotIn("build_palette_entries", source)

    def test_reference_param_validation_uses_component_map_without_key_set_clone(self) -> None:
        source = (PALETTE_ROOT / "instantiate.rs").read_text(encoding="utf-8")
        function = function_body(
            source,
            "fn build_reference_params(",
            "fn validate_child_mounts_for_component(",
        )

        self.assertIn("component.params.contains_key(key)", function)
        self.assertNotIn("collect::<BTreeSet", function)
        self.assertNotIn("let allowed", function)

    def test_slot_occupancy_maps_borrow_mount_names(self) -> None:
        instantiate = (PALETTE_ROOT / "instantiate.rs").read_text(encoding="utf-8")
        validate = function_body(
            instantiate,
            "fn validate_child_mounts_for_component(",
            "fn child_index_in_parent(",
        )
        native_slots = (PALETTE_ROOT / "native_slots.rs").read_text(encoding="utf-8")
        native_available = function_body(
            native_slots,
            "fn native_slot_is_available(",
            "}",
        )

        self.assertIn("BTreeMap::<&str, usize>", validate)
        self.assertIn("child.mount.as_deref().unwrap_or_default()", validate)
        self.assertNotIn("child.mount.clone()", validate)

        self.assertIn(".any(|child|", native_available)
        self.assertIn(
            "child.mount.as_deref().unwrap_or_default() == slot.name.as_str()",
            native_available,
        )
        self.assertNotIn("BTreeMap", native_available)
        self.assertNotIn("child.mount.clone()", native_available)


if __name__ == "__main__":
    unittest.main()
