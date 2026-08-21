mod generation;

pub(crate) use generation::{
    TimelineStripGeneration, TimelineStripGenerationInput, TimelineStripKey,
    TimelineStripStaticContent, TimelineStripTick,
};

#[cfg(test)]
pub(crate) use generation::{static_content_cache_entry_count, STATIC_CONTENT_CACHE_CAPACITY};

#[cfg(test)]
mod tests;
