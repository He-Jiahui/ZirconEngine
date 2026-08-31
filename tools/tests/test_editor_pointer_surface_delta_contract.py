from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
TOOLBAR_REBUILD = ROOT / (
    "zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/rebuild_surface.rs"
)
OVERLAY_REBUILD = ROOT / (
    "zircon_editor/src/scene/viewport/pointer/overlay_router/rebuild_surface.rs"
)
OVERLAY_SYNC = ROOT / (
    "zircon_editor/src/scene/viewport/pointer/overlay_router/"
    "viewport_overlay_pointer_router_sync.rs"
)


class EditorPointerSurfaceDeltaContractTests(unittest.TestCase):
    def test_toolbar_uses_typed_delta_without_product_tree_audit(self) -> None:
        source = TOOLBAR_REBUILD.read_text(encoding="utf-8")

        self.assertIn("ViewportToolbarSurfaceDelta", source)
        self.assertIn("apply_surface_delta", source)
        self.assertNotIn("retained_surface_topology_matches", source)
        self.assertNotIn("control_node_path_matches", source)

    def test_overlay_uses_staged_delta_without_tree_or_map_key_preflight(self) -> None:
        source = OVERLAY_REBUILD.read_text(encoding="utf-8")

        self.assertIn("ViewportOverlaySurfaceDelta", source)
        self.assertIn("classify_surface_delta", source)
        self.assertNotIn("retained_surface_topology_matches", source)
        self.assertNotIn(".keys()\n            .copied()\n            .eq(", source)

    def test_overlay_preserves_unchanged_extract_early_return(self) -> None:
        source = OVERLAY_SYNC.read_text(encoding="utf-8")
        early_return = source.index("return false;")
        candidate_rebuild = source.index("renderable_candidates(")

        self.assertLess(early_return, candidate_rebuild)
        self.assertIn("Arc::ptr_eq(current, &interaction_extract)", source)
        self.assertIn("self.scene_world_generation == Some(world_generation)", source)


if __name__ == "__main__":
    unittest.main()
