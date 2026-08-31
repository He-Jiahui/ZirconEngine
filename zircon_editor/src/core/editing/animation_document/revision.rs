use super::AnimationAuthoringDocumentError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct AnimationDocumentRevision(u64);

impl AnimationDocumentRevision {
    pub(crate) const INITIAL: Self = Self(1);

    pub(crate) const fn value(self) -> u64 {
        self.0
    }

    pub(crate) fn next(self) -> Result<Self, AnimationAuthoringDocumentError> {
        self.0
            .checked_add(1)
            .map(Self)
            .ok_or(AnimationAuthoringDocumentError::RevisionExhausted)
    }
}
