use std::error::Error;
use std::fmt;

use zircon_runtime::core::framework::animation::AnimationTargetId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TargetTableError {
    ConflictingBinding { target_id: AnimationTargetId },
    CapacityExceeded,
}

impl fmt::Display for TargetTableError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConflictingBinding { target_id } => write!(
                formatter,
                "animation target {target_id} is already bound to another runtime target"
            ),
            Self::CapacityExceeded => {
                formatter.write_str("animation target table exceeded the u32 slot capacity")
            }
        }
    }
}

impl Error for TargetTableError {}
