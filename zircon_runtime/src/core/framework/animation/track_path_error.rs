use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnimationTrackPathError;

impl fmt::Display for AnimationTrackPathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid animation track path")
    }
}

impl std::error::Error for AnimationTrackPathError {}
