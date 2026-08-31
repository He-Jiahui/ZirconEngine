use std::collections::BTreeMap;

/// Editor-side description of the object injected into a preview runtime session.
///
/// Asset locators remain opaque at this boundary. Resolving them, loading meshes, and evaluating
/// animation remain responsibilities of the runtime-backed preview session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreviewSubject {
    pub primary_asset: String,
    pub additional_assets: Vec<String>,
    pub animation_asset: Option<String>,
    pub parameter_overrides: BTreeMap<String, String>,
}

impl PreviewSubject {
    pub fn new(primary_asset: impl Into<String>) -> Self {
        Self {
            primary_asset: primary_asset.into(),
            additional_assets: Vec::new(),
            animation_asset: None,
            parameter_overrides: BTreeMap::new(),
        }
    }

    pub fn with_additional_asset(mut self, asset: impl Into<String>) -> Self {
        let asset = asset.into();
        if !self
            .additional_assets
            .iter()
            .any(|existing| existing == &asset)
        {
            self.additional_assets.push(asset);
        }
        self
    }

    pub fn with_animation_asset(mut self, asset: impl Into<String>) -> Self {
        self.animation_asset = Some(asset.into());
        self
    }

    pub fn with_parameter_override(
        mut self,
        parameter: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        self.parameter_overrides
            .insert(parameter.into(), value.into());
        self
    }
}
