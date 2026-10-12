use serde::{Deserialize, Serialize};

use crate::scene::World;

use crate::scene::EntityId;

use super::{DynamicScene, DynamicSceneError, EntityRemap};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScenePatchPreviewEntityRemap {
    pub source_entity: EntityId,
    pub target_entity: EntityId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScenePatchPreviewComponentType {
    pub type_id: String,
    pub plugin_id: String,
    pub display_name: String,
    pub already_registered: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScenePatchPreviewResource {
    pub type_path: String,
    pub already_present: bool,
    pub can_create_on_apply: bool,
    pub field_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// 对指定目标世界编译出的应用预览；实体映射和资源状态只在该目标的当前代际成立。
pub struct ScenePatchPreviewReport {
    pub component_type_count: usize,
    pub existing_component_type_count: usize,
    pub new_component_type_count: usize,
    pub component_instance_count: usize,
    pub entity_count: usize,
    pub resource_count: usize,
    pub target_entity_count: usize,
    pub preserved_entity_count: usize,
    pub remapped_entity_count: usize,
    pub component_types: Vec<ScenePatchPreviewComponentType>,
    pub resources: Vec<ScenePatchPreviewResource>,
    pub entity_remaps: Vec<ScenePatchPreviewEntityRemap>,
}

impl ScenePatchPreviewReport {
    pub fn has_entity_remaps(&self) -> bool {
        self.remapped_entity_count > 0
    }

    pub fn has_new_component_types(&self) -> bool {
        self.new_component_type_count > 0
    }

    pub fn new_component_types(&self) -> impl Iterator<Item = &ScenePatchPreviewComponentType> {
        self.component_types
            .iter()
            .filter(|component_type| !component_type.already_registered)
    }

    pub fn resources_requiring_creation(&self) -> impl Iterator<Item = &ScenePatchPreviewResource> {
        self.resources
            .iter()
            .filter(|resource| !resource.already_present && resource.can_create_on_apply)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
/// 用于编辑/导入调用链的场景补丁外壳；构造本身不校验载荷，预览或应用时才对目标世界编译。
pub struct ScenePatch {
    pub scene: DynamicScene,
}

impl ScenePatch {
    pub fn from_scene(scene: DynamicScene) -> Self {
        Self { scene }
    }

    pub fn from_world(world: &World) -> Result<Self, DynamicSceneError> {
        DynamicScene::from_world(world).map(Self::from_scene)
    }

    /// 在不写入目标世界的前提下解析映射与模式；正式应用仍需执行适配器预检，预览成功并非提交保证。
    pub fn preview_apply(
        &self,
        world: &World,
    ) -> Result<ScenePatchPreviewReport, DynamicSceneError> {
        self.scene.preview_spawn_into(world)
    }

    /// 将快照事务性写入当前目标世界，并返回所有源实体到已发布实体的映射。
    pub fn apply(&self, world: &mut World) -> Result<EntityRemap, DynamicSceneError> {
        self.scene.spawn_into(world)
    }
}
