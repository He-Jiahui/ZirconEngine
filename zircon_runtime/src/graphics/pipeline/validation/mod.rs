//! 管线资产校验把阶段命名、阶段 pass 收集和 renderer 唯一性检查集中在编译前。
mod stage_name;
mod stage_pass_descriptors;
mod validate_renderer_asset;

pub(in crate::graphics::pipeline) use stage_pass_descriptors::stage_pass_descriptors;
pub(in crate::graphics::pipeline) use validate_renderer_asset::validate_renderer_asset;
