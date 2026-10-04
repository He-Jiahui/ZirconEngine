//! 上下band有/无底部区域时的纯几何契约；核对共享间隔与状态栏末端，不涉及宿主呈现或性能。
use super::{resolve_vertical_flex_bands, VerticalFlexBandRequest};
use crate::ui::workbench::autolayout::{
    AxisConstraint, ShellSizePx, StretchMode, WorkbenchChromeMetrics,
};

/// 给两项几何用例准备可伸展轴约束，隔离中心与底区的分配参数；不是主题或宿主尺寸基线。
fn stretch_band(min: f32, preferred: f32, weight: f32) -> AxisConstraint {
    AxisConstraint {
        min,
        max: -1.0,
        preferred,
        priority: 50,
        weight,
        stretch_mode: StretchMode::Stretch,
    }
}

#[test]
/// 底区存在时，中心顶部累计固定chrome高度，中心与底区通过共享间隔衔接，状态栏末端对齐壳高度。
fn workbench_shell_geometry_vertical_flex_bands_fill_the_shell_with_token_gaps() {
    let metrics = WorkbenchChromeMetrics::default();
    let shell = ShellSizePx::new(900.0, 620.0);
    let bands = resolve_vertical_flex_bands(
        shell,
        VerticalFlexBandRequest::new(
            stretch_band(280.0, 420.0, 3.0),
            Some(stretch_band(120.0, 148.0, 1.0)),
            metrics,
        ),
    );

    assert_eq!(
        bands.center_band_frame.y,
        metrics.top_bar_height + metrics.host_bar_height + metrics.separator_thickness * 2.0
    );
    assert_eq!(
        bands.bottom_frame.y,
        bands.center_band_frame.y + bands.center_band_frame.height + metrics.separator_thickness
    );
    assert_eq!(
        bands.status_bar_frame.y + bands.status_bar_frame.height,
        shell.height
    );
}

#[test]
/// 没有底区请求时底区高度为零，中心仍遵守顶部间隔，状态栏保持壳底边；不专门覆盖紧凑宽度。
fn workbench_shell_geometry_vertical_flex_without_bottom_keeps_status_at_shell_edge() {
    let metrics = WorkbenchChromeMetrics::default();
    let shell = ShellSizePx::new(640.0, 420.0);
    let bands = resolve_vertical_flex_bands(
        shell,
        VerticalFlexBandRequest::new(stretch_band(180.0, 240.0, 1.0), None, metrics),
    );

    assert_eq!(
        bands.center_band_frame.y,
        metrics.top_bar_height + metrics.host_bar_height + metrics.separator_thickness * 2.0
    );
    assert_eq!(bands.bottom_frame.height, 0.0);
    assert_eq!(
        bands.status_bar_frame.y + bands.status_bar_frame.height,
        shell.height
    );
}
