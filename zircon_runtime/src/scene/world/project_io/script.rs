use crate::asset::assets::SceneScriptBindingAsset;
use crate::scene::world::World;

use super::{SCRIPT_BINDINGS_COMPONENT, SceneProjectError};
pub(super) fn script_bindings_for_record(
    world: &World,
    entity: u64,
) -> Result<Vec<SceneScriptBindingAsset>, SceneProjectError> {
    let Some(components) = world.dynamic_components.get(&entity) else {
        return Ok(Vec::new());
    };
    let Some(value) = components.get(SCRIPT_BINDINGS_COMPONENT) else {
        return Ok(Vec::new());
    };
    serde_json::from_value(value.clone()).map_err(|error| {
        SceneProjectError::SceneAsset(format!(
            "failed to decode script bindings for entity {entity}: {error}"
        ))
    })
}
