use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use crate::ui::retained_host::host_contract::data::HostWindowPresentationData;
use crate::ui::retained_host::host_contract::globals::{HostContractGlobal, HostContractState};
use crate::ui::retained_host::primitives::PhysicalSize;

use super::*;

#[test]
fn native_play_pending_capture_keeps_the_last_completed_game_and_simulate_images() {
    let state = Rc::new(RefCell::new(HostContractState::new(PhysicalSize::new(
        640, 420,
    ))));
    let context = PaneSurfaceHostContext::from_state(Rc::clone(&state));
    let capture = |generation, rgba| {
        PlayPreviewFrame::for_test(
            crate::core::play::PlayInstanceId::for_test(7),
            zircon_runtime_interface::GatewaySessionIdentity::new(
                11,
                zircon_runtime_interface::ZrRuntimeSessionHandle::new(13),
                17,
                None,
            )
            .with_gateway_generation(19)
            .with_play_instance(Some(7)),
            1,
            1,
            generation,
            rgba,
        )
    };
    assert!(!context.set_game_viewport_frame(capture(0, vec![0; 4])));
    assert!(state.borrow().viewport_images.game().is_none());
    let color = vec![19, 197, 83, 255];
    assert!(context.set_game_viewport_frame(capture(23, color.clone())));
    assert!(context.set_simulate_viewport_frame(capture(23, color.clone()), None));
    assert!(!context.set_game_viewport_frame(capture(0, vec![0; 4])));
    assert!(!context.set_simulate_viewport_frame(capture(0, vec![0; 4]), None));
    {
        let state = state.borrow();
        for image in [
            state.viewport_images.game(),
            state.viewport_images.simulate(),
        ] {
            let image = image.expect("pending capture must retain the completed image");
            assert_eq!(image.play_frame_identity().unwrap().generation(), 23);
            assert_eq!(image.rgba().unwrap().as_ref(), color.as_slice());
        }
    }
    assert!(context.clear_game_viewport_image());
    assert!(context.clear_simulate_viewport_image());
    assert!(state.borrow().viewport_images.game().is_none());
    assert!(state.borrow().viewport_images.simulate().is_none());
}

#[test]
fn duplicate_viewport_capture_does_not_report_an_update() {
    let state = Rc::new(RefCell::new(HostContractState::new(PhysicalSize::new(
        640, 420,
    ))));
    let context = PaneSurfaceHostContext::from_state(state);

    assert!(context.set_scene_viewport_capture(
        RenderViewportHandle::new(3),
        CapturedFrame::new(1, 1, vec![255, 0, 0, 255], 7),
    ));
    assert!(!context.set_scene_viewport_capture(
        RenderViewportHandle::new(3),
        CapturedFrame::new(1, 1, vec![255, 0, 0, 255], 7),
    ));
}

#[test]
fn scene_viewport_capture_publishes_to_the_selected_document_leaf() {
    let state = Rc::new(RefCell::new(HostContractState::new(PhysicalSize::new(
        640, 420,
    ))));
    let context = PaneSurfaceHostContext::from_state(Rc::clone(&state));

    assert!(context.set_scene_viewport_capture_for_surface(
        "document:left",
        RenderViewportHandle::new(3),
        CapturedFrame::new(1, 1, vec![255, 0, 0, 255], 7),
    ));
    assert!(context.set_scene_viewport_capture_for_surface(
        "document:right",
        RenderViewportHandle::new(3),
        CapturedFrame::new(1, 1, vec![0, 0, 255, 255], 8),
    ));

    let state = state.borrow();
    assert_eq!(
        state
            .viewport_images
            .for_surface("document:left", "Scene")
            .expect("left leaf image")
            .resource_key,
        "viewport:3:7"
    );
    assert_eq!(
        state
            .viewport_images
            .for_surface("document:right", "Scene")
            .expect("right leaf image")
            .resource_key,
        "viewport:3:8"
    );
    assert_eq!(context.scene_viewport_surface_key(), None);
}

#[test]
fn game_viewport_visibility_uses_the_active_pane_instead_of_tab_existence() {
    let state = Rc::new(RefCell::new(HostContractState::new(PhysicalSize::new(
        640, 420,
    ))));
    let context = PaneSurfaceHostContext::from_state(Rc::clone(&state));
    assert!(!context.game_viewport_visible());

    let mut presentation = HostWindowPresentationData::default();
    presentation.host_scene_data.document_dock.pane.kind = "Game".into();
    presentation
        .host_scene_data
        .document_dock
        .content_frame
        .width = 640.0;
    presentation
        .host_scene_data
        .document_dock
        .content_frame
        .height = 360.0;
    state.borrow_mut().host_presentation = Arc::new(presentation);

    assert!(context.game_viewport_visible());
}
