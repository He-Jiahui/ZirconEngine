use std::error::Error;
use std::fmt;

use zircon_runtime::core::framework::animation::AnimationTargetId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnimationClipCompileError {
    EmptyBoneName {
        bone_index: usize,
    },
    NonCanonicalBoneName {
        bone_index: usize,
        name: String,
    },
    InvalidParentIndex {
        bone_index: usize,
        parent_index: usize,
    },
    ParentCycle {
        bone_index: usize,
    },
    DuplicateTarget {
        target_id: AnimationTargetId,
    },
    TargetCapacityExceeded,
    NonCanonicalTrackTarget {
        track_index: usize,
        target: String,
    },
    UnresolvedTrack {
        track_index: usize,
        target: String,
    },
    AmbiguousTrack {
        track_index: usize,
        target: String,
    },
    DuplicateTrackTarget {
        first_track_index: usize,
        duplicate_track_index: usize,
        target_id: AnimationTargetId,
    },
    MissingResolvedTarget {
        track_index: usize,
    },
}

impl fmt::Display for AnimationClipCompileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyBoneName { bone_index } => {
                write!(
                    formatter,
                    "animation skeleton bone {bone_index} has an empty name"
                )
            }
            Self::NonCanonicalBoneName { bone_index, name } => write!(
                formatter,
                "animation skeleton bone {bone_index} has non-canonical name `{name}`"
            ),
            Self::InvalidParentIndex {
                bone_index,
                parent_index,
            } => write!(
                formatter,
                "animation skeleton bone {bone_index} references missing parent {parent_index}"
            ),
            Self::ParentCycle { bone_index } => write!(
                formatter,
                "animation skeleton parent chain for bone {bone_index} contains a cycle"
            ),
            Self::DuplicateTarget { target_id } => write!(
                formatter,
                "animation skeleton contains duplicate stable target {target_id}"
            ),
            Self::TargetCapacityExceeded => formatter
                .write_str("animation skeleton target table exceeded the u32 slot capacity"),
            Self::NonCanonicalTrackTarget {
                track_index,
                target,
            } => write!(
                formatter,
                "animation clip track {track_index} has non-canonical target `{target}`"
            ),
            Self::UnresolvedTrack {
                track_index,
                target,
            } => write!(
                formatter,
                "animation clip track {track_index} cannot resolve target `{target}`"
            ),
            Self::AmbiguousTrack {
                track_index,
                target,
            } => write!(
                formatter,
                "animation clip track {track_index} has ambiguous target `{target}`"
            ),
            Self::DuplicateTrackTarget {
                first_track_index,
                duplicate_track_index,
                target_id,
            } => write!(
                formatter,
                "animation clip tracks {first_track_index} and {duplicate_track_index} both resolve to target {target_id}"
            ),
            Self::MissingResolvedTarget { track_index } => write!(
                formatter,
                "animation clip track {track_index} resolved to a row without a stable target"
            ),
        }
    }
}

impl Error for AnimationClipCompileError {}
