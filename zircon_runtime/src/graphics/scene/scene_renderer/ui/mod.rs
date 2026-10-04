//! 屏幕空间 UI 的场景绘制适配层：消费布局与文字产物，组织几何、字形图集及帧上传事务。
//! 文本塑形归 text 子系统，UI 语义与命令契约归 runtime_interface；本层只负责图形资源和回放。
mod atlas_renderer;
mod atlas_texture_upload;
mod construct;
#[cfg(test)]
#[path = "tests/font_asset.rs"]
mod font_asset;
mod image;
mod render;
mod resource_upload;
mod screen_space_ui_renderer;
mod sdf_advances;
mod sdf_atlas;
mod sdf_render;
mod sdf_upload;
mod text;
mod text_pixel_snap;

pub(in crate::graphics::scene::scene_renderer) use resource_upload::ScreenSpaceUiPreparedUpload;
pub(crate) use screen_space_ui_renderer::ScreenSpaceUiRenderer;
pub(crate) use text::ScreenSpaceUiTextPrepareReport;
