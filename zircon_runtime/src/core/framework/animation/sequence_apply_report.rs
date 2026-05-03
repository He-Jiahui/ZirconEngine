use super::AnimationTrackPath;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AnimationSequenceApplyReport {
    pub applied_tracks: Vec<AnimationTrackPath>,
    pub missing_tracks: Vec<AnimationTrackPath>,
}
