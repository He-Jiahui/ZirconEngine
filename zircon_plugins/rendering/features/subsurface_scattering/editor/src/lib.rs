mod capability;
mod plugin;

pub use capability::{CAPABILITY, EDITOR_CAPABILITIES, FEATURE_ID};
pub use plugin::{
    editor_capabilities, editor_feature, feature_manifest,
    RenderingSubsurfaceScatteringEditorFeature,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_feature_exports_manifest_capability() {
        assert_eq!(feature_manifest().id, FEATURE_ID);
        assert_eq!(editor_capabilities(), vec![CAPABILITY.to_string()]);
    }
}
