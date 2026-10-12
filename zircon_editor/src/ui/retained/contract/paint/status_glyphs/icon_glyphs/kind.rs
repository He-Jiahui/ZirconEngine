/// 状态栏专用动作图标的稳定语义；图像名字由 glyph 绘制器映射。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) enum StatusIconKind {
    Snap,
    World,
    Target,
}
