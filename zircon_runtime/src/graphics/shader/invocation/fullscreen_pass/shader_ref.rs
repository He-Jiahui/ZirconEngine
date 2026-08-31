use serde::{Deserialize, Serialize};
use zircon_runtime_interface::resource::{AssetReference, ResourceLocator, ResourceLocatorError};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FullscreenShaderRef {
    pub shader: AssetReference,
    pub fragment_entry: String,
}

impl FullscreenShaderRef {
    pub fn new(shader: AssetReference, fragment_entry: impl Into<String>) -> Self {
        Self {
            shader,
            fragment_entry: fragment_entry.into(),
        }
    }

    pub fn from_locator_str(
        shader: &str,
        fragment_entry: impl Into<String>,
    ) -> Result<Self, ResourceLocatorError> {
        Ok(Self::new(
            AssetReference::from_locator(ResourceLocator::parse(shader)?),
            fragment_entry,
        ))
    }
}
