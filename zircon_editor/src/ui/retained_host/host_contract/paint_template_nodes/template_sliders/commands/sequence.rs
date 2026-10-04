//! 已准备好的滑块命令顺序门面；上下文确定后按标签、轨道、滑块、值层输出。

mod entry;
mod label;
mod thumbs;
mod track;
mod values;

pub(super) use entry::push_ready_slider_commands;
