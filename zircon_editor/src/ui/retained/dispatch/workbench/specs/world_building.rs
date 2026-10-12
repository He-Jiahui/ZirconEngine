use super::types::{action, spec, ActionControl, ExtensionNavigationSpec};

#[path = "../../specs/world_building/level_tools.rs"]
mod level_tools;
#[path = "../../../../dispatch/specs/world_building/prefab_and_scatter.rs"]
mod prefab_and_scatter;
#[path = "../../../../dispatch/specs/world_building/terrain_and_foliage.rs"]
mod terrain_and_foliage;
#[path = "../../../../dispatch/specs/world_building/volume_and_weather.rs"]
mod volume_and_weather;

pub(super) use level_tools::{LEVEL_STREAMING_NAVIGATION_SPEC, LEVEL_VARIANT_NAVIGATION_SPEC};
pub(super) use prefab_and_scatter::{
    PREFAB_EDITOR_NAVIGATION_SPEC, SCATTER_EDITOR_NAVIGATION_SPEC,
};
pub(super) use terrain_and_foliage::{
    FOLIAGE_EDITOR_NAVIGATION_SPEC, TERRAIN_EDITOR_NAVIGATION_SPEC,
};
pub(super) use volume_and_weather::{
    VOLUME_EDITOR_NAVIGATION_SPEC, WEATHER_EDITOR_NAVIGATION_SPEC,
};

#[cfg(test)]
#[path = "tests/world_building.rs"]
mod tests;
