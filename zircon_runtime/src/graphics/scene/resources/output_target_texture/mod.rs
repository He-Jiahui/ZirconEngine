#[path = "../../../output_target_texture/output_target_frame_plan.rs"]
mod output_target_frame_plan;
#[path = "resource.rs"]
mod output_target_texture_resource;
#[path = "../../output_target_texture/writeback_converter.rs"]
mod output_target_writeback_converter;

pub(in crate::graphics::scene) use output_target_frame_plan::OutputTargetFramePlan;
pub(in crate::graphics::scene) use output_target_texture_resource::OutputTargetTextureResource;
pub(in crate::graphics::scene::resources) use output_target_writeback_converter::OutputTargetWritebackConverter;
