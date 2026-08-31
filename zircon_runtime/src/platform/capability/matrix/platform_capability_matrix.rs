use crate::platform::PlatformFeatureSelection;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlatformCapabilityMatrix {
    pub features: PlatformFeatureSelection,
}

impl PlatformCapabilityMatrix {
    pub const fn new(features: PlatformFeatureSelection) -> Self {
        Self { features }
    }

    pub fn compiled() -> Self {
        Self::new(PlatformFeatureSelection::from_compiled_features())
    }
}
