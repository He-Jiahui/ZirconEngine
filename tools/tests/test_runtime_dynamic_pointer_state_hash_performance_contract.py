import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


class RuntimeDynamicPointerStateHashPerformanceContractTests(unittest.TestCase):
    def test_surface_set_uses_exact_key_maps_for_pointer_state(self):
        source = (
            ROOT / "zircon_runtime/src/dynamic_api/session/runtime_ui.rs"
        ).read_text(encoding="utf-8")
        surface = source.split("pub(super) struct RuntimeUiSurfaceSet", 1)[1].split(
            "\nstruct RuntimeUiSurface", 1
        )[0]
        self.assertIn("use std::collections::{BTreeSet, HashMap};", source)
        self.assertIn(
            "pointer_capture_surfaces: HashMap<Option<u64>, usize>", surface
        )
        self.assertIn("pointer_positions: HashMap<Option<u64>, UiPoint>", surface)
        self.assertNotIn("pointer_capture_surfaces: BTreeMap", surface)
        self.assertNotIn("pointer_positions: BTreeMap", surface)

    def test_capture_helpers_keep_hash_map_signatures_and_no_order_dependency(self):
        source = (
            ROOT
            / "zircon_runtime/src/dynamic_api/session/runtime_ui/input_routing.rs"
        ).read_text(encoding="utf-8")
        capture = source.split("pub(super) fn capture_surface_for_event(", 1)[1].split(
            "\npub(super) fn split_global_node_id", 1
        )[0]
        # The helper order in the source is capture -> update; include both
        # function bodies without depending on unrelated routing code.
        capture += source.split("pub(super) fn update_capture_surface(", 1)[1].split(
            "\n}", 1
        )[0]
        self.assertIn("use std::collections::{BTreeSet, HashMap};", source)
        self.assertIn("&HashMap<Option<u64>, usize>", capture)
        self.assertIn("&mut HashMap<Option<u64>, usize>", capture)
        self.assertNotIn("BTreeMap<Option<u64>, usize>", capture)

    def test_behavior_tests_use_the_same_hash_key_contract(self):
        source = (
            ROOT / "zircon_runtime/src/dynamic_api/session/runtime_ui/tests/cases.rs"
        ).read_text(encoding="utf-8")
        self.assertIn("use std::collections::{BTreeSet, HashMap};", source)
        self.assertNotIn("let mut captures = BTreeMap::new();", source)


if __name__ == "__main__":
    unittest.main()
