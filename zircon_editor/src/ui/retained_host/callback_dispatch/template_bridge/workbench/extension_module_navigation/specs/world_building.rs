use super::types::{action, spec, ActionControl, ExtensionNavigationSpec};

mod level_tools;
mod prefab_and_scatter;
mod terrain_and_foliage;
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
