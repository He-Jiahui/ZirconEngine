use crate::ui::retained_host::drawer_header_pointer::build_host_drawer_header_pointer_layout;
use crate::ui::workbench::fixture::default_preview_fixture;
use crate::ui::workbench::layout::ActivityDrawerSlot;
use crate::ui::workbench::model::WorkbenchViewModel;

#[test]
fn drawer_header_receipt_projection_preserves_region_order_and_typed_targets() {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );

    let layout = build_host_drawer_header_pointer_layout(&model);
    assert!(layout
        .surfaces
        .windows(2)
        .all(|pair| surface_order(pair[0].key) < surface_order(pair[1].key)));
    for surface in &layout.surfaces {
        for item in &surface.items {
            assert!(item.slot.shares_region(region_slot(surface.key)));
            assert!(!item.instance_id.0.is_empty());
        }
    }
}

fn surface_order(key: &str) -> usize {
    match key {
        "left" => 0,
        "right" => 1,
        "bottom" => 2,
        _ => usize::MAX,
    }
}

fn region_slot(key: &str) -> ActivityDrawerSlot {
    match key {
        "left" => ActivityDrawerSlot::LeftTop,
        "right" => ActivityDrawerSlot::RightTop,
        "bottom" => ActivityDrawerSlot::Bottom,
        _ => panic!("unexpected drawer receipt surface {key}"),
    }
}
