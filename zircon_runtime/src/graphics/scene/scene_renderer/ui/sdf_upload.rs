use crate::text::atlas::{
    GlyphAtlasFormat, GlyphAtlasPageKey, GlyphAtlasPageSpec, GlyphAtlasUploadCommand,
    GlyphAtlasUploadMode, glyph_atlas_upload_command,
};
use crate::text::sdf::{SdfAtlasBakeDirtyPage, SdfAtlasRect};

use super::sdf_atlas::{
    SdfAtlasCacheReport, SdfAtlasDirtyPageReport, SdfAtlasPlan, distance_field_atlas_page_keys,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum SdfAtlasUploadMode {
    #[default]
    None,
    FullTexture,
    DirtySlots,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct SdfAtlasUploadReport {
    pub(super) mode: SdfAtlasUploadMode,
    pub(super) byte_len: usize,
    pub(super) full_texture: bool,
    pub(super) dirty_slot_count: usize,
    pub(super) dirty_rect: Option<SdfAtlasRect>,
    pub(super) dirty_byte_len: usize,
    pub(super) dirty_pages: Vec<SdfAtlasUploadPageReport>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SdfAtlasUploadPageReport {
    pub(super) page_key: GlyphAtlasPageKey,
    pub(super) dirty_rect: SdfAtlasRect,
    pub(super) byte_len: usize,
}

pub(super) type SdfAtlasUploadCommand = GlyphAtlasUploadCommand;

pub(super) fn merge_sdf_bake_dirty_pages(
    atlas_cache: &mut SdfAtlasCacheReport,
    bake_dirty_pages: &[SdfAtlasBakeDirtyPage],
) {
    // RUNTIME132_SDF_DIRTY_PAGE_BINARY_MERGE_BENCH_V1
    if !atlas_cache
        .dirty_pages
        .windows(2)
        .all(|pair| pair[0].page_key <= pair[1].page_key)
    {
        atlas_cache
            .dirty_pages
            .sort_unstable_by_key(|page| page.page_key);
    }
    for bake_page in bake_dirty_pages {
        match atlas_cache
            .dirty_pages
            .binary_search_by_key(&bake_page.page_key, |page| page.page_key)
        {
            Ok(index) => {
                let cache_page = &mut atlas_cache.dirty_pages[index];
                cache_page.dirty_rect = union_rect(cache_page.dirty_rect, bake_page.dirty_rect);
            }
            Err(index) => atlas_cache.dirty_pages.insert(
                index,
                SdfAtlasDirtyPageReport {
                    page_key: bake_page.page_key,
                    dirty_rect: bake_page.dirty_rect,
                },
            ),
        }
    }
    atlas_cache.dirty_rect = atlas_cache
        .dirty_pages
        .iter()
        .find(|page| page.page_key == GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, 0))
        .map(|page| page.dirty_rect);
}

pub(super) fn sdf_atlas_upload_report(
    atlas_plan: &SdfAtlasPlan,
    atlas_cache: SdfAtlasCacheReport,
    atlas_resized: bool,
    atlas_upload_byte_len: usize,
    atlas_upload_full_texture: bool,
) -> SdfAtlasUploadReport {
    let dirty_slot_count = if atlas_resized {
        atlas_plan.slots.len()
    } else {
        atlas_cache
            .added_slot_count
            .saturating_add(atlas_cache.relocated_slot_count)
    };
    let dirty_pages = sdf_upload_dirty_pages(
        atlas_plan,
        &atlas_cache,
        atlas_resized,
        atlas_upload_byte_len,
    );
    let dirty_rect = dirty_pages
        .iter()
        .find(|page| page.page_key == GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, 0))
        .map(|page| page.dirty_rect);
    let dirty_byte_len = dirty_pages
        .iter()
        .map(|page| page.byte_len)
        .sum::<usize>()
        .min(atlas_upload_byte_len);
    let mode = if atlas_upload_byte_len == 0 {
        SdfAtlasUploadMode::None
    } else if atlas_upload_full_texture {
        SdfAtlasUploadMode::FullTexture
    } else if !dirty_pages.is_empty() {
        SdfAtlasUploadMode::DirtySlots
    } else {
        SdfAtlasUploadMode::None
    };
    let byte_len = match mode {
        SdfAtlasUploadMode::None => 0,
        SdfAtlasUploadMode::FullTexture => atlas_upload_byte_len,
        SdfAtlasUploadMode::DirtySlots => dirty_byte_len,
    };

    SdfAtlasUploadReport {
        mode,
        byte_len,
        full_texture: matches!(mode, SdfAtlasUploadMode::FullTexture),
        dirty_slot_count,
        dirty_rect,
        dirty_byte_len,
        dirty_pages,
    }
}

pub(super) fn sdf_atlas_upload_commands(
    atlas_plan: &SdfAtlasPlan,
    upload: &SdfAtlasUploadReport,
    source_byte_len: usize,
) -> Vec<SdfAtlasUploadCommand> {
    let mode = match upload.mode {
        SdfAtlasUploadMode::None => return Vec::new(),
        SdfAtlasUploadMode::FullTexture => GlyphAtlasUploadMode::FullPage,
        SdfAtlasUploadMode::DirtySlots => GlyphAtlasUploadMode::PartialRect,
    };
    let Some(source_pages) = sdf_atlas_source_pages(atlas_plan, source_byte_len) else {
        return Vec::new();
    };
    if !source_pages
        .windows(2)
        .all(|pair| pair[0].0.key < pair[1].0.key)
        || !upload
            .dirty_pages
            .windows(2)
            .all(|pair| pair[0].page_key < pair[1].page_key)
    {
        return Vec::new();
    }

    let mut commands = Vec::with_capacity(upload.dirty_pages.len());
    let mut source_page_index = 0_usize;
    for dirty_page in &upload.dirty_pages {
        while source_pages
            .get(source_page_index)
            .is_some_and(|(page, _, _)| page.key < dirty_page.page_key)
        {
            source_page_index = source_page_index.saturating_add(1);
        }
        let Some((page, source_offset, page_source_byte_len)) = source_pages.get(source_page_index)
        else {
            return Vec::new();
        };
        if page.key != dirty_page.page_key {
            return Vec::new();
        }
        let Some(mut command) = glyph_atlas_upload_command(
            page,
            mode,
            Some(dirty_page.dirty_rect.into()),
            *page_source_byte_len,
        ) else {
            return Vec::new();
        };
        let Some(source_offset) = u64::try_from(*source_offset).ok() else {
            return Vec::new();
        };
        let Some(command_source_offset) = command.source_offset.checked_add(source_offset) else {
            return Vec::new();
        };
        command.source_offset = command_source_offset;
        commands.push(command);
    }
    commands
}

fn sdf_atlas_source_pages(
    atlas_plan: &SdfAtlasPlan,
    source_byte_len: usize,
) -> Option<Vec<(GlyphAtlasPageSpec, usize, usize)>> {
    let page_keys = distance_field_atlas_page_keys(atlas_plan);
    let mut pages = Vec::with_capacity(page_keys.len());
    let mut source_offset = 0usize;
    for page_key in page_keys {
        let page = sdf_atlas_page_spec_for_key(atlas_plan, page_key);
        let page_byte_len = sdf_page_source_byte_len(&page)?;
        let page_end = source_offset.checked_add(page_byte_len)?;
        if page_end > source_byte_len {
            return None;
        }
        pages.push((page, source_offset, page_byte_len));
        source_offset = page_end;
    }
    Some(pages)
}

fn sdf_atlas_page_spec_for_key(
    atlas_plan: &SdfAtlasPlan,
    page_key: GlyphAtlasPageKey,
) -> GlyphAtlasPageSpec {
    atlas_plan
        .atlas_set
        .page(page_key.format, page_key.page_index)
        .cloned()
        .unwrap_or_else(|| GlyphAtlasPageSpec::new(page_key, atlas_plan.atlas_size))
}

fn sdf_page_source_byte_len(page: &GlyphAtlasPageSpec) -> Option<usize> {
    let byte_len = page
        .size
        .x
        .max(1)
        .saturating_mul(page.size.y.max(1))
        .saturating_mul(page.storage_format.bytes_per_pixel());
    usize::try_from(byte_len).ok()
}

fn sdf_upload_dirty_pages(
    atlas_plan: &SdfAtlasPlan,
    atlas_cache: &SdfAtlasCacheReport,
    atlas_resized: bool,
    atlas_upload_byte_len: usize,
) -> Vec<SdfAtlasUploadPageReport> {
    if atlas_upload_byte_len == 0 {
        return Vec::new();
    }
    if atlas_resized {
        return distance_field_atlas_page_keys(atlas_plan)
            .into_iter()
            .map(|page_key| {
                let page = sdf_atlas_page_spec_for_key(atlas_plan, page_key);
                SdfAtlasUploadPageReport {
                    page_key,
                    dirty_rect: SdfAtlasRect {
                        x: 0,
                        y: 0,
                        width: page.size.x.max(1),
                        height: page.size.y.max(1),
                    },
                    byte_len: sdf_page_source_byte_len(&page).unwrap_or(0),
                }
            })
            .collect();
    }

    atlas_cache
        .dirty_pages
        .iter()
        .map(sdf_upload_page_report)
        .collect()
}

fn sdf_upload_page_report(page: &SdfAtlasDirtyPageReport) -> SdfAtlasUploadPageReport {
    let bytes_per_pixel = page.page_key.format.storage_format().bytes_per_pixel() as usize;
    SdfAtlasUploadPageReport {
        page_key: page.page_key,
        dirty_rect: page.dirty_rect,
        byte_len: page.dirty_rect.width as usize
            * page.dirty_rect.height as usize
            * bytes_per_pixel,
    }
}

fn union_rect(left: SdfAtlasRect, right: SdfAtlasRect) -> SdfAtlasRect {
    let x = left.x.min(right.x);
    let y = left.y.min(right.y);
    let right_edge = left
        .x
        .saturating_add(left.width)
        .max(right.x.saturating_add(right.width));
    let bottom_edge = left
        .y
        .saturating_add(left.height)
        .max(right.y.saturating_add(right.height));
    SdfAtlasRect {
        x,
        y,
        width: right_edge.saturating_sub(x),
        height: bottom_edge.saturating_sub(y),
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "sdf_upload/merge_index_tests.rs"]
mod merge_index_tests;
