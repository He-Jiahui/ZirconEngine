use crate::graphics::text::atlas::{
    glyph_atlas_upload_command, GlyphAtlasFormat, GlyphAtlasPageKey, GlyphAtlasPageSpec,
    GlyphAtlasUploadCommand, GlyphAtlasUploadMode,
};

use super::sdf_atlas::{
    sdf_atlas_layer_count, SdfAtlasCacheReport, SdfAtlasDirtyPageReport, SdfAtlasPlan, SdfAtlasRect,
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
    upload: SdfAtlasUploadReport,
    source_byte_len: usize,
) -> Vec<SdfAtlasUploadCommand> {
    let mode = match upload.mode {
        SdfAtlasUploadMode::None => return Vec::new(),
        SdfAtlasUploadMode::FullTexture => GlyphAtlasUploadMode::FullPage,
        SdfAtlasUploadMode::DirtySlots => GlyphAtlasUploadMode::PartialRect,
    };
    upload
        .dirty_pages
        .into_iter()
        .filter_map(|dirty_page| {
            let page = sdf_atlas_page_spec_for_key(atlas_plan, dirty_page.page_key);
            let page_source_byte_len = sdf_page_source_byte_len(&page)?;
            let mut command = glyph_atlas_upload_command(
                &page,
                mode,
                Some(dirty_page.dirty_rect.into()),
                page_source_byte_len,
            )?;
            offset_upload_command_for_source_layer(
                &mut command,
                page_source_byte_len,
                source_byte_len,
            )?;
            Some(command)
        })
        .collect()
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

fn offset_upload_command_for_source_layer(
    command: &mut SdfAtlasUploadCommand,
    page_source_byte_len: usize,
    source_byte_len: usize,
) -> Option<()> {
    let layer_offset =
        u64::from(command.page_key.page_index).checked_mul(page_source_byte_len as u64)?;
    let page_end = layer_offset.checked_add(page_source_byte_len as u64)?;
    if page_end > source_byte_len as u64 {
        return None;
    }
    command.source_offset = command.source_offset.checked_add(layer_offset)?;
    Some(())
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
        return (0..sdf_atlas_layer_count(atlas_plan))
            .map(|page_index| {
                let page_key = GlyphAtlasPageKey::new(GlyphAtlasFormat::Sdf, page_index);
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
    SdfAtlasUploadPageReport {
        page_key: page.page_key,
        dirty_rect: page.dirty_rect,
        byte_len: page.dirty_rect.width as usize * page.dirty_rect.height as usize,
    }
}

#[cfg(test)]
mod tests;
