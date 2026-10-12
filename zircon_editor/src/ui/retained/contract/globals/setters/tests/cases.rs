use std::cell::RefCell;
use std::rc::Rc;

use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::globals::{HostContractGlobal, HostContractState};
use crate::ui::retained_host::primitives::PhysicalSize;

use super::{HostAssetSurfaceInteractionState, PaneSurfaceHostContext};

#[test]
fn asset_content_interaction_setters_store_activity_and_browser_state() {
    let state = Rc::new(RefCell::new(HostContractState::new(PhysicalSize::new(
        640, 420,
    ))));
    let context = PaneSurfaceHostContext::from_state(Rc::clone(&state));

    context.set_activity_asset_content_scroll_px(72.5);
    context.set_activity_asset_content_hovered_index(4);
    context.set_browser_asset_content_scroll_px(33.0);
    context.set_browser_asset_content_hovered_index(2);

    let state = state.borrow();
    assert_eq!(
        state
            .pane_interaction_state
            .activity_asset_content_scroll_px,
        72.5
    );
    assert_eq!(
        state
            .pane_interaction_state
            .activity_asset_content_hovered_index,
        4
    );
    assert_eq!(
        state.pane_interaction_state.browser_asset_content_scroll_px,
        33.0
    );
    assert_eq!(
        state
            .pane_interaction_state
            .browser_asset_content_hovered_index,
        2
    );
}

#[test]
fn asset_content_interaction_setters_clamp_scroll_and_preserve_no_hover() {
    let state = Rc::new(RefCell::new(HostContractState::new(PhysicalSize::new(
        640, 420,
    ))));
    let context = PaneSurfaceHostContext::from_state(Rc::clone(&state));

    context.set_activity_asset_content_scroll_px(-8.0);
    context.set_activity_asset_content_hovered_index(-1);
    context.set_browser_asset_content_scroll_px(-4.0);
    context.set_browser_asset_content_hovered_index(-1);

    let state = state.borrow();
    assert_eq!(
        state
            .pane_interaction_state
            .activity_asset_content_scroll_px,
        0.0
    );
    assert_eq!(
        state
            .pane_interaction_state
            .activity_asset_content_hovered_index,
        -1
    );
    assert_eq!(
        state.pane_interaction_state.browser_asset_content_scroll_px,
        0.0
    );
    assert_eq!(
        state
            .pane_interaction_state
            .browser_asset_content_hovered_index,
        -1
    );
}

#[test]
fn browser_reference_interaction_setters_store_both_list_states() {
    let state = Rc::new(RefCell::new(HostContractState::new(PhysicalSize::new(
        640, 420,
    ))));
    let context = PaneSurfaceHostContext::from_state(Rc::clone(&state));

    context.set_browser_asset_references_scroll_px(48.0);
    context.set_browser_asset_references_hovered_index(3);
    context.set_browser_asset_used_by_scroll_px(-12.0);
    context.set_browser_asset_used_by_hovered_index(1);
    context.set_browser_asset_reference_hover_frame(FrameRect {
        x: 20.0,
        y: 30.0,
        width: 120.0,
        height: 80.0,
    });

    let state = state.borrow();
    assert_eq!(
        state
            .pane_interaction_state
            .browser_asset_references_scroll_px,
        48.0
    );
    assert_eq!(
        state
            .pane_interaction_state
            .browser_asset_references_hovered_index,
        3
    );
    assert_eq!(
        state.pane_interaction_state.browser_asset_used_by_scroll_px,
        0.0
    );
    assert_eq!(
        state
            .pane_interaction_state
            .browser_asset_used_by_hovered_index,
        1
    );
    assert_eq!(
        state
            .pane_interaction_state
            .browser_asset_reference_hover_frame,
        FrameRect {
            x: 20.0,
            y: 30.0,
            width: 120.0,
            height: 80.0,
        }
    );
}

#[test]
fn activity_reference_interaction_setters_store_both_list_states() {
    let state = Rc::new(RefCell::new(HostContractState::new(PhysicalSize::new(
        640, 420,
    ))));
    let context = PaneSurfaceHostContext::from_state(Rc::clone(&state));

    context.set_activity_asset_references_scroll_px(48.0);
    context.set_activity_asset_references_hovered_index(3);
    context.set_activity_asset_used_by_scroll_px(-12.0);
    context.set_activity_asset_used_by_hovered_index(1);
    context.set_activity_asset_reference_hover_frame(FrameRect {
        x: 20.0,
        y: 30.0,
        width: 120.0,
        height: 80.0,
    });

    let state = state.borrow();
    assert_eq!(
        state
            .pane_interaction_state
            .activity_asset_references_scroll_px,
        48.0
    );
    assert_eq!(
        state
            .pane_interaction_state
            .activity_asset_references_hovered_index,
        3
    );
    assert_eq!(
        state
            .pane_interaction_state
            .activity_asset_used_by_scroll_px,
        0.0
    );
    assert_eq!(
        state
            .pane_interaction_state
            .activity_asset_used_by_hovered_index,
        1
    );
    assert_eq!(
        state
            .pane_interaction_state
            .activity_asset_reference_hover_frame,
        FrameRect {
            x: 20.0,
            y: 30.0,
            width: 120.0,
            height: 80.0,
        }
    );
}

#[test]
fn console_scroll_setter_stores_and_clamps_the_runtime_offset() {
    let state = Rc::new(RefCell::new(HostContractState::new(PhysicalSize::new(
        640, 420,
    ))));
    let context = PaneSurfaceHostContext::from_state(Rc::clone(&state));

    context.set_console_scroll_px(54.0);
    assert_eq!(
        state.borrow().pane_interaction_state.console_scroll_px,
        54.0
    );

    context.set_console_scroll_px(-9.0);
    assert_eq!(state.borrow().pane_interaction_state.console_scroll_px, 0.0);
}

#[test]
fn asset_surface_transaction_advances_generation_once_and_skips_stable_writeback() {
    let state = Rc::new(RefCell::new(HostContractState::new(PhysicalSize::new(
        640, 420,
    ))));
    let context = PaneSurfaceHostContext::from_state(Rc::clone(&state));
    let before = state.borrow().interaction_generation();

    let interaction = HostAssetSurfaceInteractionState {
        tree_hovered_index: 1,
        tree_scroll_px: 12.0,
        content_hovered_index: 2,
        content_scroll_px: 24.0,
        references_hovered_index: 3,
        references_scroll_px: 36.0,
        used_by_hovered_index: 4,
        used_by_scroll_px: 48.0,
    };
    assert!(context.set_asset_surface_interaction("activity", interaction));
    let after_change = state.borrow().interaction_generation();
    assert_eq!(after_change, before + 1);

    assert!(!context.set_asset_surface_interaction("activity", interaction));
    assert_eq!(state.borrow().interaction_generation(), after_change);
}
