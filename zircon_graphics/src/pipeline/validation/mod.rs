mod stage_name;
mod stage_pass_descriptors;
mod validate_renderer_asset;

pub(in crate::pipeline) use stage_pass_descriptors::stage_pass_descriptors;
pub(in crate::pipeline) use validate_renderer_asset::validate_renderer_asset;
