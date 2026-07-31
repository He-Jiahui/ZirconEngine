from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
PALETTE_ROOT = ROOT / "zircon_editor/src/ui/asset_editor/palette"


def function_body(source: str, start: str, end: str) -> str:
    offset = source.index(start)
    return source[offset : source.index(end, offset)]


class EditorUiAssetPalettePerformanceContractTests(unittest.TestCase):
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
        available = function_body(native_slots, "fn available_slots(", "}")

        for function in (validate, available):
            self.assertIn("BTreeMap::<&str, usize>", function)
            self.assertIn("child.mount.as_deref().unwrap_or_default()", function)
            self.assertNotIn("child.mount.clone()", function)


if __name__ == "__main__":
    unittest.main()
