use std::sync::Arc;

use crate::scene::viewport::{RenderViewportDescriptor, RenderViewportHandle};
use zircon_runtime_interface::math::UVec2;

use super::super::SlintViewportController;
use super::fake_render_framework::FakeRenderFramework;
use super::test_extract::test_extract;

#[test]
fn controller_creates_and_resizes_render_framework_viewports() {
    let framework = Arc::new(FakeRenderFramework::default());
    let controller = SlintViewportController::new_with_framework(framework.clone());
    let extract = test_extract();

    controller
        .submit_extract(extract.clone(), UVec2::new(320, 240))
        .unwrap();
    controller
        .submit_extract(extract, UVec2::new(640, 480))
        .unwrap();

    let state = framework.state.lock().unwrap();
    assert_eq!(
        state.created_viewports,
        vec![
            RenderViewportDescriptor::new(UVec2::new(320, 240)).with_label("editor.viewport"),
            RenderViewportDescriptor::new(UVec2::new(640, 480)).with_label("editor.viewport"),
        ]
    );
    assert_eq!(
        state.destroyed_viewports,
        vec![RenderViewportHandle::new(1)]
    );
    assert_eq!(
        state.submitted_viewports,
        vec![RenderViewportHandle::new(1), RenderViewportHandle::new(2)]
    );
}
