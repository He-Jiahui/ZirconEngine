use crate::asset::assets::PrefabInstanceAsset;
use crate::scene::world::World;

use super::{SceneProjectError, PREFAB_INSTANCE_COMPONENT};

// 预制体元数据保存在动态组件中；这里只解码保留记录，不负责创建或变更实体。
pub(super) fn prefab_instance_for_record(
    world: &World,
    entity: u64,
) -> Result<Option<PrefabInstanceAsset>, SceneProjectError> {
    let Some(components) = world.dynamic_components.get(&entity) else {
        return Ok(None);
    };
    let Some(value) = components.get(PREFAB_INSTANCE_COMPONENT) else {
        return Ok(None);
    };
    serde_json::from_value(value.clone())
        .map(Some)
        .map_err(|error| {
            SceneProjectError::SceneAsset(format!(
                "failed to decode retained prefab instance for entity {entity}: {error}"
            ))
        })
}
