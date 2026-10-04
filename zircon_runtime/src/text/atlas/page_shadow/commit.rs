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
        let Self {
            patches,
            mut zero_initialized_pages,
            mut failed_zero_initialized_pages,
        } = other;
        append_or_adopt_owned_vec(&mut self.patches, patches);
        self.zero_initialized_pages
            .append(&mut zero_initialized_pages);
        self.failed_zero_initialized_pages
            .append(&mut failed_zero_initialized_pages);
    }
}

fn append_or_adopt_owned_vec<T>(target: &mut Vec<T>, mut source: Vec<T>) {
    // 空目标可直接接管输入缓冲；已有预留容量时继续追加，以复用目标的分配。
    if target.is_empty() && target.capacity() == 0 {
        *target = source;
        return;
    }
    target.append(&mut source);
}

#[cfg(test)]
#[path = "tests/commit.rs"]
mod tests;
