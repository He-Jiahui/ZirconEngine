use crate::asset::assets::SceneScriptBindingAsset;
use crate::scene::world::World;

use super::{SceneProjectError, SCRIPT_BINDINGS_COMPONENT};
// 脚本绑定作为动态组件读取，缺少组件表示该实体没有脚本绑定，解码错误则阻止项目记录继续使用。
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
