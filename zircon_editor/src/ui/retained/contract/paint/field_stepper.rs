//! 数值输入域的步进器资产和密度门面；仅由 fields 命令入口在足够空间时调用。

#[path = "field_stepper/command.rs"]
mod command;
#[path = "field_stepper/metrics.rs"]
mod metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use command::push_field_stepper;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use metrics::workbench_field_stepper_metrics;
