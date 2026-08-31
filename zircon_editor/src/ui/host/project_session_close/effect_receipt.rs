use crate::core::recovery::{ProjectSessionEffect, ProjectSessionEffectDisposition};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProjectCloseEffectReceipt {
    pub(super) effect: ProjectSessionEffect,
    pub(super) disposition: ProjectSessionEffectDisposition,
}

impl ProjectCloseEffectReceipt {
    pub(crate) const fn effect(&self) -> ProjectSessionEffect {
        self.effect
    }

    pub(crate) const fn disposition(&self) -> ProjectSessionEffectDisposition {
        self.disposition
    }
}
