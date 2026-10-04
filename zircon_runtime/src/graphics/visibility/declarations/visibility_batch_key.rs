use crate::core::framework::render::RenderLayerSet;
use crate::core::framework::scene::Mobility;
use crate::core::resource::ResourceId;

/// 按渲染层、材质、模型与移动性分组的批次身份；不能只按实体或模型合并。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VisibilityBatchKey {
    pub render_layer_mask: RenderLayerSet,
    pub material_id: ResourceId,
    pub model_id: ResourceId,
    pub mobility: Mobility,
}
