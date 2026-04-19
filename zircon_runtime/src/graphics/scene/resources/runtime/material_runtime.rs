use crate::core::math::Vec4;
use crate::core::resource::ResourceId;

use super::super::PipelineKey;

#[derive(Clone, Debug)]
pub(crate) struct MaterialRuntime {
    pub(crate) base_color: Vec4,
    pub(crate) base_color_texture: Option<ResourceId>,
    pub(crate) pipeline_key: PipelineKey,
}
