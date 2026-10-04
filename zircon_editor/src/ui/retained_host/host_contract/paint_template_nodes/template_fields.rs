//! secondary 链的文本/搜索/步进字段绘制边界；变换类编辑器字段在归属判定中让出。

mod commands;
mod geometry;
mod identity;
mod layers;
mod metrics;
mod search;
mod style;
mod surface;
mod text;

#[cfg(test)]
#[path = "template_fields_tests/tests/mod.rs"]
mod tests;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_field_commands;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use metrics::workbench_field_metrics;
