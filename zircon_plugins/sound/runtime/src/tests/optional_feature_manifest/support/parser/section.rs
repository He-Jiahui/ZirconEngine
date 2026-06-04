#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum OptionalFeatureSection {
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

impl OptionalFeatureSection {
    pub(super) fn from_table_header(line: &str) -> Option<Self> {
        match line {
            "[[optional_features]]" => Some(Self::Feature),
            "[[optional_features.dependencies]]" => Some(Self::Dependency),
            "[[optional_features.modules]]" => Some(Self::Module),
            _ if line.starts_with("[[") => Some(Self::None),
            _ => None,
        }
    }
}
