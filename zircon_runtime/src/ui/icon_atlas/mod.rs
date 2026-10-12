#[path = "atlas.rs"]
mod atlas;
#[path = "builtins.rs"]
mod builtins;
#[path = "svg.rs"]
mod svg;

pub use atlas::{
    UiIconAtlasBuildError, UiIconAtlasBuilder, UiIconAtlasPlan, UiIconAtlasRect, UiIconAtlasSlot,
    UiIconAtlasUvRect, UiIconRasterRequest,
};
pub(crate) use builtins::UiBuiltinIconReference;
pub use builtins::{builtin_icon_supported, UiBuiltinIcon, UiIconSize};
pub(crate) use svg::parse_ui_svg_icon_cached;
pub use svg::{
    parse_ui_svg_icon, UiSvgIconDocument, UiSvgIconElement, UiSvgIconParseError, UiSvgIconViewport,
};
