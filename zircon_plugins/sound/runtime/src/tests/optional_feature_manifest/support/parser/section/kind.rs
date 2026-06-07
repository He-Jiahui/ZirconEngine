#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in super::super) enum OptionalFeatureSection {
    None,
    Feature,
    Dependency,
    Module,
}

impl Default for OptionalFeatureSection {
    fn default() -> Self {
        Self::None
    }
}
