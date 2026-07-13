use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum JobCategory {
    Import,
    Compile,
    Thumbnail,
    Export,
    Index,
    Play,
    Misc,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobPriority {
    Interactive,
    #[default]
    Normal,
    Background,
}

impl JobPriority {
    pub(super) const fn admission_rank(self) -> u8 {
        match self {
            Self::Interactive => 0,
            Self::Normal => 1,
            Self::Background => 2,
        }
    }
}
