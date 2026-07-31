import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
VIEWPORT = ROOT / "zircon_editor/src/scene/viewport"


class ViewportInteractionExtractContractTests(unittest.TestCase):
    def source(self, relative: str) -> str:
        return (VIEWPORT / relative).read_text(encoding="utf-8")

    def test_controller_owns_one_generation_cache_for_render_and_pointer(self) -> None:
        controller = self.source("controller/scene_viewport_controller.rs")
        render = self.source(
            "controller/scene_viewport_controller_build_render_snapshot.rs"
        )
        pointer = self.source("controller/scene_viewport_controller_pointer_route.rs")

        self.assertIn("ViewportInteractionExtractCache", controller)
        self.assertIn("interaction_extract", render)
        self.assertIn("interaction_extract", pointer)
        self.assertIn("resolve_from_render_packet", render)
        self.assertIn("resolve_for_pointer", pointer)

    def test_pointer_layout_borrows_shared_extract_slices(self) -> None:
        layout = self.source("pointer/viewport_pointer_layout.rs")
        sync = self.source(
            "pointer/overlay_router/viewport_overlay_pointer_router_sync.rs"
        )

        self.assertIn("Arc<[HandleOverlayExtract]>", layout)
        self.assertIn("Arc<[SceneGizmoOverlayExtract]>", layout)
        self.assertIn("Arc<[ViewportRenderablePickCandidate]>", layout)
        self.assertIn("Arc<ViewportInteractionExtract>", sync)
        self.assertNotIn("scene_gizmo_candidates(", sync)
        self.assertNotIn("renderable_candidates(scene", sync)

    def test_renderable_candidates_come_from_runtime_render_extract(self) -> None:
        candidates = self.source("pointer/candidates/renderable_candidates.rs")

        self.assertIn("RenderMeshSnapshot", candidates)
        self.assertNotIn("scene.nodes()", candidates)
        self.assertNotIn("active_in_hierarchy", candidates)

    def test_scene_gizmo_scan_filters_kind_before_hierarchy_and_reuses_node(self) -> None:
        render_packet = self.source("render_packet.rs")
        gizmo_scan = render_packet.split("pub(in crate::scene::viewport) fn build_scene_gizmos", 1)[
            1
        ].split("fn build_selection_highlights", 1)[0]

        self.assertIn("for node in scene.nodes()", gizmo_scan)
        self.assertIn("matches!(node.kind", gizmo_scan)
        self.assertLess(
            gizmo_scan.index("matches!(node.kind"),
            gizmo_scan.index("active_in_hierarchy"),
        )
        self.assertNotIn("scene.find_node", gizmo_scan)

    def test_legacy_duplicate_scene_key_and_gizmo_builder_are_deleted(self) -> None:
        self.assertFalse(
            (VIEWPORT / "pointer/overlay_router/viewport_pointer_scene_key.rs").exists()
        )
        self.assertFalse(
            (VIEWPORT / "pointer/candidates/scene_gizmo_candidates.rs").exists()
        )


if __name__ == "__main__":
    unittest.main()
