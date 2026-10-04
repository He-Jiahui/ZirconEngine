//! 区分一帧使用的资源格式种类与按绘制顺序切换资源的段数。
//! 原生渲染复用单个 CPU 帧计划；交错的颜色和覆盖率字形不能因为资源格式分组而重排。

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

/// 仅当一帧所有字形的采样语义一致时允许使用单格式提交路径。
/// 空集不声明格式，混合格式应沿保持绘制顺序的多资源路径处理。
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
