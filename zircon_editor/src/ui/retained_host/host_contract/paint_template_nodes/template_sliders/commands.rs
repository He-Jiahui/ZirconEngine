//! 滑块命令的上下文和输出序列门面；外部只消费布尔归属协议。

mod context;
mod entry;
mod sequence;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use entry::push_slider_commands;
