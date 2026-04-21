use crate::ui::workbench::layout::{
    ActivityDrawerSlot, LayoutCommand, LayoutManager, WorkbenchLayout,
};

#[test]
fn set_drawer_extent_clamps_to_minimum_size() {
    let manager = LayoutManager::default();
    let mut layout = WorkbenchLayout::default();

    manager
        .apply(
            &mut layout,
            LayoutCommand::SetDrawerExtent {
                slot: ActivityDrawerSlot::LeftTop,
                extent: 48.0,
            },
        )
        .unwrap();

    assert_eq!(layout.drawers[&ActivityDrawerSlot::LeftTop].extent, 120.0);
}
