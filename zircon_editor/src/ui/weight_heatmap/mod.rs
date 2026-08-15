mod generation;

pub(crate) use generation::{
    WeightHeatmapGeneration, WeightHeatmapGenerationInput, WeightHeatmapSource,
};

#[cfg(test)]
pub(crate) use generation::{STATIC_FIELD_CACHE_CAPACITY, static_field_cache_entry_count};

#[cfg(test)]
mod tests;
