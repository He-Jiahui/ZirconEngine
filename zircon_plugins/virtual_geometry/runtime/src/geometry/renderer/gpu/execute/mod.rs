#[path = "../../../execute/collect_inputs.rs"]
mod collect_inputs;
#[path = "../../../create_bind_group.rs"]
mod create_bind_group;
#[path = "../../../execute/create_buffers.rs"]
mod create_buffers;
#[path = "../../../gpu/execute/dispatch.rs"]
mod dispatch;
#[path = "../../../gpu/execute/execute.rs"]
mod execute;
#[path = "../../../gpu/execute/queue_params.rs"]
mod queue_params;
#[path = "../../../../virtual_geometry_prepare.rs"]
mod virtual_geometry_prepare_execution_buffers;
#[path = "../../../../geometry_prepare_execution_inputs.rs"]
mod virtual_geometry_prepare_execution_inputs;

#[cfg(test)]
#[path = "../../../gpu/execute/tests/cases.rs"]
mod tests;
