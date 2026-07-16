use super::{GlyphAtlasPageKey, GlyphAtlasRect};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasDirtyPage {
    page_key: GlyphAtlasPageKey,
    merged_rect: Option<GlyphAtlasRect>,
}

impl GlyphAtlasDirtyPage {
    pub(crate) fn new(page_key: GlyphAtlasPageKey) -> Self {
        Self {
            page_key,
            merged_rect: None,
        }
    }

    pub(crate) fn mark_dirty(&mut self, page_key: GlyphAtlasPageKey, rect: GlyphAtlasRect) {
        if page_key != self.page_key || rect.width == 0 || rect.height == 0 {
            return;
        }
        self.merged_rect = Some(match self.merged_rect {
            Some(existing) => existing.union(rect),
            None => rect,
        });
    }

    pub(crate) fn page_key(&self) -> GlyphAtlasPageKey {
        self.page_key
    }

    pub(crate) fn merged_rect(&self) -> Option<GlyphAtlasRect> {
        self.merged_rect
    }
}

#[cfg(test)]
mod tests;
