use zircon_math::Vec4;
use zircon_resource::ResourceId;

use super::pipeline_key::PipelineKey;

#[derive(Clone, Debug)]
pub(crate) struct MaterialRuntime {
    pub(crate) base_color: Vec4,
    pub(crate) base_color_texture: Option<ResourceId>,
    pub(crate) pipeline_key: PipelineKey,
}
