//! 分段主体、边界和选中强调的绘制组合边界；各子模块使用共享几何。

#[path = "segments/body.rs"]
mod body;
#[path = "../segments/divider.rs"]
mod divider;
#[path = "../segments/selected.rs"]
mod selected;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use body::push_segmented_control;
