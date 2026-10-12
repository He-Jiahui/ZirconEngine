//! secondary 链的文本/搜索/步进字段绘制边界；变换类编辑器字段在归属判定中让出。

#[path = "fields/commands.rs"]
mod commands;
#[path = "fields/geometry.rs"]
mod geometry;
#[path = "fields/identity.rs"]
mod identity;
#[path = "fields/layers.rs"]
mod layers;
#[path = "fields/metrics.rs"]
mod metrics;
#[path = "fields/search.rs"]
mod search;
#[path = "fields/style.rs"]
mod style;
#[path = "fields/surface.rs"]
mod surface;
#[path = "fields/text.rs"]
mod text;

#[cfg(test)]
#[path = "fields_tests/tests/mod.rs"]
mod tests;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use commands::push_field_commands;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use geometry::{
    field_has_label_lane, is_field_value_text_command,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use metrics::workbench_field_metrics;
