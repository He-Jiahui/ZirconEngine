use crate::settings::HubLanguage;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildMessageId {
    BuildOutputPath,
}

impl BuildMessageId {
    pub const ALL: &'static [Self] = &[Self::BuildOutputPath];

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::BuildOutputPath => "build.output-path",
        }
    }

    pub(super) fn param_count(self) -> usize {
        match self {
            Self::BuildOutputPath => 1,
        }
    }

    pub(super) fn template(self, language: HubLanguage) -> &'static str {
        match (language, self) {
            (HubLanguage::English, Self::BuildOutputPath) => "{0}",
            (HubLanguage::Chinese, Self::BuildOutputPath) => "{0}",
        }
    }
}
