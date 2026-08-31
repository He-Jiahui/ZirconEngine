use serde::{Deserialize, Serialize};

use crate::core::editor_message::PlayStateKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PlayModePredicate {
    Edit,
    Building,
    Playing,
    CleanupFailed,
}

impl PlayModePredicate {
    pub(crate) fn matches(self, state: PlayStateKind) -> bool {
        matches!(
            (self, state),
            (Self::Edit, PlayStateKind::Edit)
                | (Self::Building, PlayStateKind::Building)
                | (Self::Playing, PlayStateKind::Playing)
                | (Self::CleanupFailed, PlayStateKind::CleanupFailed)
        )
    }
}
