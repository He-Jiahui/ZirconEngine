//! 管线资产校验把阶段命名、阶段 pass 收集和 renderer 唯一性检查集中在编译前。
#[path = "stage_name.rs"]
mod stage_name;
#[path = "stage_pass_descriptors.rs"]
mod stage_pass_descriptors;
#[path = "validate_renderer_asset.rs"]
mod validate_renderer_asset;

pub(in crate::graphics::pipeline) use stage_pass_descriptors::stage_pass_descriptors;
pub(in crate::graphics::pipeline) use validate_renderer_asset::validate_renderer_asset;
