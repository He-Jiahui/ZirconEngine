mod capability;
mod plugin;

pub use capability::{CAPABILITY, EDITOR_CAPABILITIES, FEATURE_ID};
pub use plugin::{
    editor_capabilities, editor_feature, feature_manifest, RenderingVfxGraphEditorFeature,
};
pub const VFX_GRAPH_ASSET_VIEW_ID: &str = "rendering.vfx_graph.asset";
