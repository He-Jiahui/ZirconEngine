mod instance;
mod instance_buffer;
mod pipeline;
mod renderer;
mod resources;
mod state;

pub(super) use renderer::GlyphAtlasBitmapRenderer;
pub(super) use state::GlyphAtlasBitmapRendererPrepareReport;

#[cfg(test)]
mod product_framebuffer;
#[cfg(test)]
mod tests;
