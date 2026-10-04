//! 将位图字形计划映射到按格式驻留的 GPU 资源；上传只排队，shadow 状态由文字系统在帧提交成功后确认。
mod instance;
mod instance_buffer;
mod pipeline;
mod renderer;
mod resources;
mod state;

pub(super) use renderer::GlyphAtlasBitmapRenderer;
pub(super) use state::GlyphAtlasBitmapRendererPrepareReport;

#[cfg(test)]
#[path = "tests/product_framebuffer.rs"]
mod product_framebuffer;
#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
