use zircon_resource::ResourceId;

use super::fallback_shader_uri::fallback_shader_uri;
use super::pipeline_key::PipelineKey;

pub(crate) fn default_pipeline_key() -> PipelineKey {
    PipelineKey {
        shader_id: ResourceId::from_locator(&fallback_shader_uri()),
        shader_revision: 1,
        double_sided: false,
        alpha_blend: false,
    }
}
