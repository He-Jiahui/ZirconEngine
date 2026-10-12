#[path = "bindings/gameplay_animation.rs"]
mod gameplay_animation;
#[path = "bindings/gameplay_state.rs"]
mod gameplay_state;
#[path = "bindings/install.rs"]
mod install;
#[path = "bindings/online_sessions.rs"]
mod online_sessions;
#[path = "bindings/render_asset_vfx.rs"]
mod render_asset_vfx;
#[path = "bindings/runtime_state.rs"]
mod runtime_state;
#[path = "bindings/simulation_physics.rs"]
mod simulation_physics;
#[path = "bindings/types.rs"]
mod types;
#[path = "bindings/ui_diagnostics.rs"]
mod ui_diagnostics;
#[path = "bindings/world_building.rs"]
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
