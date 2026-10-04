use std::error::Error;

use super::super::super::super::runtime_lease::RetainedHostStartupRuntimeAccess;
use super::super::super::super::*;
use super::bundle::StartupManagers;
use super::events::subscribe_startup_change_events;

// 从 Runtime 租约解析共享服务后立即建立变更订阅；装配宿主持有这些接收器直到关闭。
pub(in crate::ui::retained_host::app::host_lifecycle::startup) fn resolve_startup_managers(
    runtime_access: &RetainedHostStartupRuntimeAccess,
    background_event_wake: zircon_runtime::core::framework::channel::ChannelWakeCallback,
) -> Result<StartupManagers, Box<dyn Error + Send + Sync>> {
    let asset_runtime_access = {
        zircon_runtime::profile_scope!("editor", "retained_host", "new_asset_manager_handle");
        runtime_access.asset_runtime_access()?
    };
    let editor_manager = {
        zircon_runtime::profile_scope!("editor", "retained_host", "new_resolve_editor_manager");
        runtime_access.editor_manager()?
    };
    let resolved_asset_manager = asset_runtime_access.asset_manager()?;
    let resolved_editor_asset_manager = asset_runtime_access.editor_asset_manager()?;
    let resolved_resource_manager = asset_runtime_access.resource_manager()?;
    let events = subscribe_startup_change_events(
        resolved_asset_manager.as_ref(),
        resolved_editor_asset_manager.as_ref(),
        resolved_resource_manager.as_ref(),
        background_event_wake,
    );

    Ok(StartupManagers {
        asset_runtime_access,
        editor_manager,
        asset_change_events: events.asset_change_events,
        editor_asset_change_events: events.editor_asset_change_events,
        resource_change_events: events.resource_change_events,
    })
}
