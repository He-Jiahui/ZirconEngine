//! 分段主体、边界和选中强调的绘制组合边界；各子模块使用共享几何。

mod body;
mod divider;
mod selected;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use body::push_segmented_control;
