//! 将位图字形计划映射到按格式驻留的 GPU 资源；上传只排队，shadow 状态由文字系统在帧提交成功后确认。
#[path = "instance.rs"]
mod instance;
#[path = "instance_buffer.rs"]
mod instance_buffer;
#[path = "pipeline.rs"]
mod pipeline;
#[path = "renderer.rs"]
mod renderer;
#[path = "resources.rs"]
mod resources;
#[path = "state.rs"]
mod state;

pub(super) use renderer::GlyphAtlasBitmapRenderer;
pub(super) use state::GlyphAtlasBitmapRendererPrepareReport;

#[cfg(test)]
#[path = "../tests/product_framebuffer.rs"]
mod product_framebuffer;
#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
