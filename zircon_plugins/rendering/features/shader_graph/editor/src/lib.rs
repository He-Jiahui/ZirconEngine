mod capability;
mod plugin;

pub use capability::{CAPABILITY, EDITOR_CAPABILITIES, FEATURE_ID};
pub use plugin::{
    editor_capabilities, editor_feature, feature_manifest, RenderingShaderGraphEditorFeature,
};
pub const SHADER_GRAPH_ASSET_VIEW_ID: &str = "rendering.shader_graph.asset";
