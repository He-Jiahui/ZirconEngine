//! 抽屉尺寸命令须符合布局管理器的最小尺寸许可。
use crate::ui::workbench::layout::{
    ActivityDrawerSlot, LayoutCommand, LayoutManager, WorkbenchLayout,
};

#[test]
/// 在默认规则下约束过小请求的结果；默认阈值来自管理器配置，不能推断全部主题宽度。
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

    assert_eq!(
        layout.active_activity_window_drawers()[&ActivityDrawerSlot::LeftTop].extent,
        120.0
    );
}
