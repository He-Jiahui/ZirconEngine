use std::sync::Arc;

use crate::ui::retained_host::primitives::ModelRc;
use crate::ui::retained_host::{
    FloatingWindowData, HostWindowPresentationData, PaneData, SceneViewportChromeData,
};
use zircon_runtime_interface::ui::surface::UiSurfaceFrame;

use super::viewport_toolbar_surface_frame_for_surface;

#[test]
fn native_floating_scene_toolbar_frame_is_resolved_for_its_window_surface() {
    let expected = Arc::new(UiSurfaceFrame::default());
    let window = FloatingWindowData {
        window_id: "window:scene".to_string(),
        active_pane: PaneData {
            viewport: SceneViewportChromeData {
                toolbar_surface_frame: Some(Arc::clone(&expected)),
                ..SceneViewportChromeData::default()
            },
            ..PaneData::default()
        },
        ..FloatingWindowData::default()
    };
    let mut presentation = HostWindowPresentationData::default();
    presentation.native_floating_surface_data.floating_windows =
        ModelRc::with_metadata(vec![window], "native viewport test");

    let resolved = viewport_toolbar_surface_frame_for_surface(&presentation, "window:scene")
        .expect("native floating scene pane should retain its toolbar frame");

    assert!(Arc::ptr_eq(&expected, &resolved));
    assert!(
        viewport_toolbar_surface_frame_for_surface(&presentation, "window:retired").is_none(),
        "a stale native surface key must not reuse another window's frame"
    );
}
