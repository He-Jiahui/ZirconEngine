use crate::ui::retained_host::host_page_pointer::{HostPagePointerBridge, HostPagePointerRoute};
use crate::ui::workbench::page_tabs::main_page_tab_close_frame;
use zircon_runtime_interface::ui::layout::{UiFrame, UiPoint};

use super::support::{sample_host_page_layout, sample_overflow_host_page_layout};

#[test]
fn shared_host_page_pointer_bridge_routes_tabs_from_shared_hit_test() {
    let mut bridge = HostPagePointerBridge::new();
    let layout = sample_host_page_layout();
    let tab = layout.tabs[1].clone();
    assert!(bridge.sync(layout));

    let route = bridge
        .handle_click(
            tab.page_index,
            tab.frame.x,
            tab.frame.width,
            UiPoint::new(4.0, 12.0),
        )
        .unwrap();
    assert_eq!(
        route.route,
        Some(HostPagePointerRoute::Tab {
            item_index: 1,
            page_id: "inspector".to_string(),
        })
    );
}

#[test]
fn shared_host_page_pointer_bridge_routes_close_from_the_page_tab_close_hit_target() {
    let mut bridge = HostPagePointerBridge::new();
    let layout = sample_host_page_layout();
    let tab = layout.tabs[1].clone();
    let close = tab.close_frame.expect("closeable page close frame");
    assert!(bridge.sync(layout));

    let route = bridge
        .handle_click(
            tab.page_index,
            tab.frame.x,
            tab.frame.width,
            UiPoint::new(
                close.x + close.width * 0.5 - tab.frame.x,
                close.y + close.height * 0.5 - tab.frame.y,
            ),
        )
        .unwrap();

    assert_eq!(
        route.route,
        Some(HostPagePointerRoute::Close {
            item_index: 1,
            instance_id: "editor.prefab#1".to_string(),
        })
    );
}

#[test]
fn shared_host_page_pointer_bridge_uses_the_authored_local_frame_for_close_hit_testing() {
    let mut bridge = HostPagePointerBridge::new();
    let mut layout = sample_host_page_layout();
    let strip_x = 40.0;
    let authored_tab_x = 120.0;
    layout.strip_frame.x = strip_x;
    let tab = &mut layout.tabs[1];
    tab.frame.x = strip_x + authored_tab_x;
    tab.frame.width = 180.0;
    tab.close_frame = Some(main_page_tab_close_frame(tab.frame));
    let tab = tab.clone();
    assert!(bridge.sync(layout));

    let authored_frame = UiFrame::new(
        strip_x + authored_tab_x,
        tab.frame.y,
        108.0,
        tab.frame.height,
    );
    let authored_close = main_page_tab_close_frame(authored_frame);
    let route = bridge
        .handle_click(
            tab.page_index,
            authored_tab_x,
            authored_frame.width,
            UiPoint::new(
                authored_close.center().x - authored_frame.x,
                authored_close.center().y - authored_frame.y,
            ),
        )
        .unwrap();

    assert_eq!(
        route.route,
        Some(HostPagePointerRoute::Close {
            item_index: 1,
            instance_id: "editor.prefab#1".to_string(),
        })
    );
}

#[test]
fn shared_host_page_pointer_bridge_rejects_a_stale_adjacent_close_hit() {
    let mut bridge = HostPagePointerBridge::new();
    let mut layout = sample_host_page_layout();
    layout.items[0].close_instance_id = Some("editor.project#1".to_string());
    layout.tabs[0].frame.width = 180.0;
    layout.tabs[0].close_frame = Some(main_page_tab_close_frame(layout.tabs[0].frame));
    layout.tabs[1].frame = UiFrame::new(194.0, layout.tabs[1].frame.y, 180.0, 30.0);
    layout.tabs[1].close_frame = Some(main_page_tab_close_frame(layout.tabs[1].frame));
    let stale_adjacent_close = layout.tabs[0]
        .close_frame
        .expect("previous closeable tab should have a retained close frame");
    assert!(bridge.sync(layout));

    let authored_frame = UiFrame::new(122.0, 25.0, 108.0, 30.0);
    let route = bridge
        .handle_click(
            1,
            authored_frame.x,
            authored_frame.width,
            UiPoint::new(
                stale_adjacent_close.center().x - authored_frame.x,
                stale_adjacent_close.center().y - authored_frame.y,
            ),
        )
        .unwrap();

    assert_eq!(
        route.route,
        Some(HostPagePointerRoute::Tab {
            item_index: 1,
            page_id: "inspector".to_string(),
        })
    );
}

#[test]
fn shared_host_page_pointer_bridge_skips_rebuild_for_unchanged_layout() {
    let mut bridge = HostPagePointerBridge::new();
    let layout = sample_host_page_layout();

    assert!(bridge.sync(layout.clone()));
    assert!(!bridge.sync(layout));
}

#[test]
fn shared_host_page_pointer_bridge_routes_overflow_button_from_shared_hit_test() {
    let mut bridge = HostPagePointerBridge::new();
    let layout = sample_overflow_host_page_layout();
    assert!(bridge.sync(layout));

    let route = bridge
        .handle_overflow_click(UiPoint::new(5.0, 12.0))
        .expect("overflow hit should dispatch through the shared surface");

    assert_eq!(
        route.route,
        Some(HostPagePointerRoute::Overflow {
            hidden_page_indices: vec![2, 3],
        })
    );
}
