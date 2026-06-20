use std::error::Error;

use super::super::super::super::*;
use super::bundle::StartupManagers;
use super::events::subscribe_startup_change_events;
use crate::ui::host::editor_asset_manager::resolve_editor_asset_manager;
use zircon_runtime::asset::pipeline::manager::resolve_asset_manager;

pub(in crate::ui::retained_host::app::host_lifecycle::startup) fn resolve_startup_managers(
    core: &CoreHandle,
) -> Result<StartupManagers, Box<dyn Error>> {
    let resolver = ManagerResolver::new(core.clone());
    let asset_manager = {
        zircon_runtime::profile_scope!("editor", "retained_host", "new_resolve_asset_manager");
        resolve_asset_manager(resolver.core())?
    };
    let editor_asset_manager = {
        zircon_runtime::profile_scope!(
            "editor",
            "retained_host",
            "new_resolve_editor_asset_manager"
        );
        resolve_editor_asset_manager(resolver.core())?
    };
    let resource_manager = {
        zircon_runtime::profile_scope!("editor", "retained_host", "new_resolve_resource_manager");
        resolver.resource()?
    };
    let editor_manager = {
        zircon_runtime::profile_scope!("editor", "retained_host", "new_resolve_editor_manager");
        core.resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)?
    };
    let events =
        subscribe_startup_change_events(&asset_manager, &editor_asset_manager, &resource_manager);

    Ok(StartupManagers {
        asset_manager,
        editor_asset_manager,
        resource_manager,
        editor_manager,
        asset_change_events: events.asset_change_events,
        editor_asset_change_events: events.editor_asset_change_events,
        resource_change_events: events.resource_change_events,
    })
}
