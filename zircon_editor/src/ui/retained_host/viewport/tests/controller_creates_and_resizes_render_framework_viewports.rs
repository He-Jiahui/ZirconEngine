use std::collections::BTreeSet;
use std::sync::Arc;

use crate::scene::viewport::{RenderViewportDescriptor, RenderViewportHandle};
use zircon_runtime_interface::math::UVec2;

use super::super::RetainedViewportController;
use super::fake_render_framework::FakeRenderFramework;
use super::test_extract::test_extract;

#[test]
fn controller_creates_and_resizes_render_framework_viewports() {
    let framework = Arc::new(FakeRenderFramework::default());
    let controller = RetainedViewportController::new_with_framework(framework.clone());
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
    assert_eq!(state.quality_profiles.len(), 2);
    for (index, (viewport, profile)) in state.quality_profiles.iter().enumerate() {
        assert_eq!(*viewport, RenderViewportHandle::new(index as u64 + 1));
        assert!(profile.features.hybrid_global_illumination);
        assert!(!profile.features.virtual_geometry);
    }
    assert_eq!(state.submitted_hybrid_gi_settings.len(), 2);
    for settings in &state.submitted_hybrid_gi_settings {
        let settings = settings
            .as_ref()
            .expect("editor viewport should inject default Hybrid GI settings");
        assert!(settings.enabled);
        assert!(settings.trace_budget > 0);
        assert!(settings.card_budget > 0);
        assert!(settings.voxel_budget > 0);
    }
}

#[test]
fn controller_keeps_independent_surface_handles_and_retires_closed_surfaces() {
    let framework = Arc::new(FakeRenderFramework::default());
    let controller = RetainedViewportController::new_with_framework(framework.clone());

    controller
        .submit_extract_with_ui("document:left", test_extract(), None, UVec2::new(320, 240))
        .unwrap();
    controller
        .submit_extract_with_ui("document:right", test_extract(), None, UVec2::new(640, 480))
        .unwrap();
    controller
        .submit_extract_with_ui("document:left", test_extract(), None, UVec2::new(320, 240))
        .unwrap();
    controller
        .retain_viewport_surfaces(&BTreeSet::from(["document:right".to_string()]))
        .unwrap();

    let state = framework.state.lock().unwrap();
    assert_eq!(state.submitted_viewports.len(), 3);
    assert_ne!(state.submitted_viewports[0], state.submitted_viewports[1]);
    assert_eq!(state.submitted_viewports[0], state.submitted_viewports[2]);
    assert_eq!(
        state.destroyed_viewports,
        vec![state.submitted_viewports[0]]
    );
}

#[test]
fn failed_multi_surface_retirement_keeps_all_remaining_handles_owned_for_retry() {
    let framework = Arc::new(FakeRenderFramework::default());
    let controller = RetainedViewportController::new_with_framework(framework.clone());
    for surface in ["document:a", "document:b", "document:c"] {
        controller
            .submit_extract_with_ui(surface, test_extract(), None, UVec2::new(320, 240))
            .unwrap();
    }
    framework.fail_next_destroy();

    let first = controller.retain_viewport_surfaces(&BTreeSet::new());

    assert!(first.is_err());
    assert!(framework
        .state
        .lock()
        .unwrap()
        .destroyed_viewports
        .is_empty());
    assert_eq!(controller.lock_shared().viewports.len(), 3);

    controller
        .retain_viewport_surfaces(&BTreeSet::new())
        .expect("retry should retire every still-owned viewport");
    assert_eq!(framework.state.lock().unwrap().destroyed_viewports.len(), 3);
    assert!(controller.lock_shared().viewports.is_empty());
}
