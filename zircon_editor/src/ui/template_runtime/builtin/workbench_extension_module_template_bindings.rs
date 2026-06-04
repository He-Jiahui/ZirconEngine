mod gameplay_animation;
mod gameplay_state;
mod install;
mod online_sessions;
mod render_asset_vfx;
mod runtime_state;
mod simulation_physics;
mod types;
mod ui_diagnostics;
mod world_building;

use std::collections::BTreeMap;

use crate::ui::binding::EditorUiBinding;
use gameplay_animation::GAMEPLAY_ANIMATION_BINDINGS;
use gameplay_state::GAMEPLAY_STATE_BINDINGS;
use install::insert_workbench_extension_bindings;
use online_sessions::ONLINE_SESSIONS_BINDINGS;
use render_asset_vfx::RENDER_ASSET_VFX_BINDINGS;
use runtime_state::RUNTIME_STATE_BINDINGS;
use simulation_physics::SIMULATION_PHYSICS_BINDINGS;
use ui_diagnostics::{DIAGNOSTICS_OBSERVABILITY_BINDINGS, UI_AUTHORING_BINDINGS};
use world_building::WORLD_BUILDING_BINDINGS;

pub(super) fn insert_workbench_extension_module_bindings(
    bindings: &mut BTreeMap<String, EditorUiBinding>,
) {
    insert_workbench_extension_bindings(bindings, GAMEPLAY_ANIMATION_BINDINGS);
    insert_workbench_extension_bindings(bindings, GAMEPLAY_STATE_BINDINGS);
    insert_workbench_extension_bindings(bindings, RENDER_ASSET_VFX_BINDINGS);
    insert_workbench_extension_bindings(bindings, SIMULATION_PHYSICS_BINDINGS);
    insert_workbench_extension_bindings(bindings, ONLINE_SESSIONS_BINDINGS);
    insert_workbench_extension_bindings(bindings, RUNTIME_STATE_BINDINGS);
    insert_workbench_extension_bindings(bindings, UI_AUTHORING_BINDINGS);
    insert_workbench_extension_bindings(bindings, DIAGNOSTICS_OBSERVABILITY_BINDINGS);
    insert_workbench_extension_bindings(bindings, WORLD_BUILDING_BINDINGS);
}
