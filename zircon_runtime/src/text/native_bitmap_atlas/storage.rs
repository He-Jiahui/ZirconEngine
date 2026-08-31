use crate::text::atlas::{GlyphAtlasFormat, GlyphAtlasStorageFormat};

/// Counts atlas resources independently of painter-order draw segments.
///
/// A frame owns one submission plan. Rendering may switch resources between
/// commands, but that order must never trigger a second CPU-side frame plan.
pub(crate) fn native_bitmap_atlas_storage_resource_count<I>(formats: I) -> usize
where
    I: IntoIterator<Item = GlyphAtlasFormat>,
{
    let mut resources = Vec::new();
    for format in formats {
        if !resources.contains(&format) {
            resources.push(format);
        }
    }
    resources.len()
}

pub(crate) fn single_native_bitmap_atlas_format<I>(formats: I) -> Option<GlyphAtlasFormat>
where
    I: IntoIterator<Item = GlyphAtlasFormat>,
{
    let mut formats = formats.into_iter();
    let first = formats.next()?;
    formats.all(|format| format == first).then_some(first)
}

pub(crate) fn single_native_bitmap_atlas_storage_format<I>(
    formats: I,
) -> Option<GlyphAtlasStorageFormat>
where
    I: IntoIterator<Item = GlyphAtlasStorageFormat>,
{
    let mut formats = formats.into_iter();
    let first = formats.next()?;
    formats.all(|format| format == first).then_some(first)
}

pub(crate) fn native_bitmap_atlas_has_mixed_storage_formats<I>(formats: I) -> bool
where
    I: IntoIterator<Item = GlyphAtlasStorageFormat>,
{
    let mut formats = formats.into_iter();
    let Some(first) = formats.next() else {
        return false;
    };
    formats.any(|format| format != first)
}
