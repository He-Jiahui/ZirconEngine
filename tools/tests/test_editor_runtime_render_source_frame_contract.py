from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
PROJECTION_CACHE = ROOT / (
    "zircon_editor/src/ui/layouts/views/view_projection/projection_cache.rs"
)
NODE_PROJECTION = ROOT / (
    "zircon_editor/src/ui/layouts/views/view_projection/node_projection.rs"
)
PROJECTION_BUILD = ROOT / (
    "zircon_editor/src/ui/layouts/views/view_projection/build.rs"
)
ASSETS_ACTIVITY = ROOT / "zircon_editor/src/ui/layouts/views/assets_activity.rs"
ASSET_BROWSER = ROOT / "zircon_editor/src/ui/layouts/views/asset_browser.rs"
SCENE_PROJECTION = ROOT / (
    "zircon_editor/src/ui/layouts/windows/workbench_host_window/scene_projection.rs"
)
VIEW_HOST_DATA = ROOT / (
    "zircon_editor/src/ui/layouts/windows/workbench_host_window/host_data.rs"
)
HOST_PANE_DATA = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/data/panes/basic.rs"
)
PANE_CONVERSION = ROOT / (
    "zircon_editor/src/ui/retained_host/ui/pane_data_conversion/"
    "native_template_node_panes.rs"
)
PAINT_ROOT = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer"
)
PANE_TEMPLATE_PAINT = PAINT_ROOT / "docks/pane/template_nodes.rs"


class EditorRuntimeRenderSourceFrameContractTests(unittest.TestCase):
    def test_projection_cache_publishes_the_runtime_frame_after_rebuild(self) -> None:
        source = PROJECTION_CACHE.read_text(encoding="utf-8")
        ready = source.split("ProjectionCacheUpdate::Ready(CachedProjection {", 1)[1]
        ready = ready.split("}))", 1)[0]

        self.assertIn("source_frame: Arc<UiSurfaceFrame>", source)
        self.assertIn("source_frame: entry.surface.surface_frame()", ready)

    def test_projection_carries_the_source_frame_without_rebuilding_it(self) -> None:
        projection = NODE_PROJECTION.read_text(encoding="utf-8")
        build = PROJECTION_BUILD.read_text(encoding="utf-8")

        self.assertIn("source_frame: Option<Arc<UiSurfaceFrame>>", projection)
        self.assertIn(
            "pub(crate) fn source_frame(&self) -> Option<Arc<UiSurfaceFrame>>",
            projection,
        )
        self.assertIn("self.source_frame.as_ref().map(Arc::clone)", projection)
        self.assertIn("source_frame: Some(projection.source_frame)", build)

    def test_assets_activity_preserves_the_frame_through_the_host_bridge(self) -> None:
        activity = ASSETS_ACTIVITY.read_text(encoding="utf-8")
        view_data = VIEW_HOST_DATA.read_text(encoding="utf-8")
        host_data = HOST_PANE_DATA.read_text(encoding="utf-8")
        conversion = PANE_CONVERSION.read_text(encoding="utf-8")

        self.assertIn(
            "pub render_source_frame: Option<Arc<UiSurfaceFrame>>",
            view_data,
        )
        self.assertIn(
            "pub render_source_frame: Option<Arc<UiSurfaceFrame>>",
            host_data,
        )
        self.assertIn("let render_source_frame = projection.source_frame();", activity)
        self.assertIn("render_source_frame,", activity)
        self.assertIn("render_source_frame: data.render_source_frame", conversion)

    def test_paint_hot_path_does_not_lazy_publish_surface_frames(self) -> None:
        paint_sources = "\n".join(
            path.read_text(encoding="utf-8")
            for path in PAINT_ROOT.rglob("*.rs")
            if path.name != "tests.rs"
        )

        self.assertNotIn(".surface_frame()", paint_sources)

    def test_asset_browser_cache_keeps_nodes_and_source_frame_atomic(self) -> None:
        source = ASSET_BROWSER.read_text(encoding="utf-8")

        self.assertIn("fn asset_browser_pane_data(", source)
        self.assertIn("render_source_frame: Option<Arc<UiSurfaceFrame>>", source)
        self.assertIn("render_source_frame: cached.render_source_frame.clone()", source)
        self.assertIn("let render_source_frame = projection.source_frame();", source)
        self.assertIn("render_source_frame: render_source_frame.clone()", source)
        self.assertIn("asset_browser_pane_data(snapshot, size).nodes", source)
        self.assertIn("Arc::ptr_eq", source)

    def test_asset_browser_source_frame_reaches_the_recording_scope(self) -> None:
        scene = SCENE_PROJECTION.read_text(encoding="utf-8")
        view_data = VIEW_HOST_DATA.read_text(encoding="utf-8")
        host_data = HOST_PANE_DATA.read_text(encoding="utf-8")
        conversion = PANE_CONVERSION.read_text(encoding="utf-8")
        paint = PANE_TEMPLATE_PAINT.read_text(encoding="utf-8")

        self.assertIn("asset_browser_pane_data(asset_browser, size)", scene)
        self.assertIn(
            "pub render_source_frame: Option<Arc<UiSurfaceFrame>>",
            view_data.split("struct AssetBrowserPaneViewData", 1)[1],
        )
        self.assertIn(
            "pub render_source_frame: Option<Arc<UiSurfaceFrame>>",
            host_data.split("struct AssetBrowserPaneData", 1)[1],
        )
        browser_conversion = conversion.split(
            "fn to_host_contract_asset_browser_pane", 1
        )[1]
        self.assertIn("render_source_frame: data.render_source_frame", browser_conversion)
        self.assertIn("pane.asset_browser.render_source_frame.as_ref()", paint)


if __name__ == "__main__":
    unittest.main()
