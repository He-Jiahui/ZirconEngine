use std::error::Error;

use super::super::super::super::*;
use super::bundle::StartupManagers;
use super::events::subscribe_startup_change_events;
use crate::ui::host::editor_asset_manager::editor_asset_manager_handle;
use zircon_runtime::asset::asset_manager_handle;

pub(in crate::ui::retained_host::app::host_lifecycle::startup) fn resolve_startup_managers(
    core: &CoreHandle,
) -> Result<StartupManagers, Box<dyn Error>> {
    let resolver = ManagerResolver::new(core.clone());
    let asset_manager = {
        zircon_runtime::profile_scope!("editor", "retained_host", "new_asset_manager_handle");
        asset_manager_handle(core)?
    };
    let editor_asset_manager = {
        zircon_runtime::profile_scope!(
            "editor",
            "retained_host",
            "new_editor_asset_manager_handle"
        );
        editor_asset_manager_handle(core)?
    };
    let resource_manager = {
        zircon_runtime::profile_scope!("editor", "retained_host", "new_resolve_resource_manager");
        resolver.resource_handle()?
    };
    let editor_manager = {
        zircon_runtime::profile_scope!("editor", "retained_host", "new_resolve_editor_manager");
        core.resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)?
    };
    let resolved_asset_manager = resolver.resolve(asset_manager.clone())?;
    let resolved_editor_asset_manager = resolver.resolve(editor_asset_manager.clone())?;
    let resolved_resource_manager = resolver.resolve(resource_manager.clone())?;
    let events = subscribe_startup_change_events(
        resolved_asset_manager.as_ref(),
        resolved_editor_asset_manager.as_ref(),
        resolved_resource_manager.as_ref(),
    );

    Ok(StartupManagers {
        asset_manager,
        editor_asset_manager,
        resource_manager_resolver: resolver,
        resource_manager,
        editor_manager,
        asset_change_events: events.asset_change_events,
        editor_asset_change_events: events.editor_asset_change_events,
        resource_change_events: events.resource_change_events,
    })
}
