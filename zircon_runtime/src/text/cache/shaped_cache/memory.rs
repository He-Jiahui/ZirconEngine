use std::mem::size_of;

use crate::text::{OpenTypeFeature, ShapedGlyph, ShapedGlyphRun, ShapedTextLine};

use super::{ShapedRunCacheEntry, ShapedRunCacheKey, TextCacheSlot};

const SHAPED_CACHE_ARC_ALLOCATIONS_PER_ENTRY: usize = 3;
const HASH_INDEX_CAPACITY_RESERVE_DIVISOR: usize = 2;

pub(super) fn estimated_entry_bytes(key: &ShapedRunCacheKey, run: &ShapedGlyphRun) -> usize {
    // The cache budget is intentionally conservative: capacities retain memory,
    // and every entry participates in eight hash-backed indexes, including LRU links.
    let line_bytes = run.lines.iter().fold(0_usize, |total, line| {
        total.saturating_add(
            line.glyphs
                .capacity()
                .saturating_mul(size_of::<ShapedGlyph>()),
        )
    });
    let family_capacity = key.font_family.as_ref().map_or(0, String::capacity);
    let language_capacity = key.language.as_ref().map_or(0, String::capacity);

    size_of::<ShapedRunCacheEntry>()
        .saturating_add(size_of::<ShapedGlyphRun>())
        .saturating_add(
            run.lines
                .capacity()
                .saturating_mul(size_of::<ShapedTextLine>()),
        )
        .saturating_add(family_capacity.saturating_mul(2))
        .saturating_add(language_capacity.saturating_mul(2))
        .saturating_add(
            key.features
                .len()
                .saturating_mul(size_of::<OpenTypeFeature>()),
        )
        .saturating_add(run.source_text.len())
        .saturating_add(line_bytes)
        .saturating_add(
            arc_allocation_header_bytes().saturating_mul(SHAPED_CACHE_ARC_ALLOCATIONS_PER_ENTRY),
        )
        .saturating_add(cache_index_resident_bytes())
}

const fn arc_allocation_header_bytes() -> usize {
    // ArcInner carries strong and weak atomic counters before its payload.
    size_of::<usize>() * 2
}

const fn cache_index_resident_bytes() -> usize {
    let indexed_inline_payload = size_of::<TextCacheSlot>()
        + size_of::<ShapedRunCacheKey>()
        + size_of::<Vec<TextCacheSlot>>()
        + size_of::<TextCacheSlot>()
        + size_of::<usize>()
        + size_of::<TextCacheSlot>()
        + size_of::<Option<TextCacheSlot>>() * 2
        + (size_of::<u64>() + size_of::<Vec<TextCacheSlot>>() + size_of::<TextCacheSlot>()) * 2
        + (size_of::<TextCacheSlot>() + size_of::<usize>()) * 2;
    // Reserve 50% beyond inline payload for hash control bytes, alignment, and
    // load-factor slack without depending on the platform allocator internals.
    indexed_inline_payload
        + indexed_inline_payload.saturating_add(size_of::<ShapedRunCacheEntry>())
            / HASH_INDEX_CAPACITY_RESERVE_DIVISOR
}
