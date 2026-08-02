use std::cell::RefCell;
use std::rc::Rc;

use crate::ui::retained_host::host_contract::globals::{HostContractGlobal, HostContractState};
use crate::ui::retained_host::primitives::PhysicalSize;

use super::PaneSurfaceHostContext;

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
