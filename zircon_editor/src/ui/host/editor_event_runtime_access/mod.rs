#[path = "asset_access.rs"]
mod asset_access;
#[path = "component_dispatch.rs"]
mod component_dispatch;
#[path = "event_dispatch.rs"]
mod event_dispatch;
#[path = "extension_access.rs"]
mod extension_access;
#[path = "input_dispatch.rs"]
mod input_dispatch;
#[path = "settings_mutation.rs"]
mod settings_mutation;
#[path = "settings_projection.rs"]
mod settings_projection;
#[path = "snapshot.rs"]
mod snapshot;
#[path = "status.rs"]
mod status;
#[path = "workbench_projection.rs"]
mod workbench_projection;

pub use asset_access::EditorAssetOperationInvokeError;

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
