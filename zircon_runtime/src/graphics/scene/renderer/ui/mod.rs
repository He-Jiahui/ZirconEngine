//! 屏幕空间 UI 的场景绘制适配层：消费布局与文字产物，组织几何、字形图集及帧上传事务。
//! 文本塑形归 text 子系统，UI 语义与命令契约归 runtime_interface；本层只负责图形资源和回放。
#[path = "atlas_renderer/mod.rs"]
mod atlas_renderer;
#[path = "atlas_texture_upload.rs"]
mod atlas_texture_upload;
#[path = "construct.rs"]
mod construct;
#[cfg(test)]
#[path = "tests/font_asset.rs"]
mod font_asset;
#[path = "image.rs"]
mod image;
#[path = "render.rs"]
mod render;
#[path = "resource_upload.rs"]
mod resource_upload;
#[path = "screen_space_ui_renderer.rs"]
mod screen_space_ui_renderer;
#[path = "sdf_advances.rs"]
mod sdf_advances;
#[path = "sdf_atlas.rs"]
mod sdf_atlas;
#[path = "sdf_render.rs"]
mod sdf_render;
#[path = "sdf_upload.rs"]
mod sdf_upload;
#[path = "text.rs"]
mod text;
#[path = "text_pixel_snap.rs"]
mod text_pixel_snap;

pub(in crate::graphics::scene::scene_renderer) use resource_upload::ScreenSpaceUiPreparedUpload;
pub(crate) use screen_space_ui_renderer::ScreenSpaceUiRenderer;
pub(crate) use text::ScreenSpaceUiTextPrepareReport;
