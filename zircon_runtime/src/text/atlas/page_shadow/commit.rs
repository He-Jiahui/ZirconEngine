use std::collections::BTreeSet;

use super::super::GlyphAtlasPageKey;
use super::GlyphAtlasBitmapPageShadowPatch;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct GlyphAtlasBitmapPageShadowCommit {
    pub(crate) patches: Vec<GlyphAtlasBitmapPageShadowPatch>,
    pub(crate) zero_initialized_pages: BTreeSet<GlyphAtlasPageKey>,
    pub(crate) failed_zero_initialized_pages: BTreeSet<GlyphAtlasPageKey>,
}

impl GlyphAtlasBitmapPageShadowCommit {
    pub(crate) fn extend(&mut self, other: Self) {
        self.patches.extend(other.patches);
        self.zero_initialized_pages
            .extend(other.zero_initialized_pages);
        self.failed_zero_initialized_pages
            .extend(other.failed_zero_initialized_pages);
    }
}
