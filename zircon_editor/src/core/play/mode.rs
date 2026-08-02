use super::PlayStartRequest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PlayModeKind {
    #[default]
    Edit,
    Building,
    Playing,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayKind {
    #[default]
    Play,
    Simulate,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum PlayMode {
    #[default]
    Edit,
    Building {
        request: PlayStartRequest,
        play_after_build: bool,
    },
    Playing {
        kind: PlayKind,
    },
}

impl PlayMode {
    pub const fn kind(&self) -> PlayModeKind {
        match self {
            Self::Edit => PlayModeKind::Edit,
            Self::Building { .. } => PlayModeKind::Building,
            Self::Playing { .. } => PlayModeKind::Playing,
        }
    }
}
