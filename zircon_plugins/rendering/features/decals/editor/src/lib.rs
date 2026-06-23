mod capability;
mod plugin;

pub use capability::{CAPABILITY, EDITOR_CAPABILITIES, FEATURE_ID};
pub use plugin::{
    editor_capabilities, editor_feature, feature_manifest, RenderingDecalsEditorFeature,
};
pub const DECAL_PROJECTOR_DRAWER_ID: &str = "rendering.Component.DecalProjector.drawer";
