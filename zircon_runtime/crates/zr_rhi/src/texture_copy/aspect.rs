use serde::{Deserialize, Serialize};

/// Selects one texture plane for a buffer/texture copy operation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextureCopyAspect {
    /// The single color aspect of a color texture.
    #[default]
    All,
    /// The depth aspect of a depth or depth-stencil texture.
    DepthOnly,
    /// The stencil aspect of a depth-stencil texture.
    StencilOnly,
}
