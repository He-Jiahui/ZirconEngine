use super::sdf_atlas::{SdfAtlasCacheReport, SdfAtlasPlan};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum SdfAtlasUploadMode {
    #[default]
    None,
    FullTexture,
    DirtySlots,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct SdfAtlasUploadReport {
    pub(super) mode: SdfAtlasUploadMode,
    pub(super) byte_len: usize,
    pub(super) full_texture: bool,
    // Dirty slot fields model the future partial-upload boundary while the current GPU path
    // still uploads the full texture for correctness.
    pub(super) dirty_slot_count: usize,
    pub(super) dirty_byte_len: usize,
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
    let slot_byte_len = atlas_plan
        .slots
        .first()
        .map(|slot| slot.rect.width as usize * slot.rect.height as usize)
        .unwrap_or(0);
    let dirty_byte_len = if atlas_resized {
        atlas_upload_byte_len
    } else {
        dirty_slot_count
            .saturating_mul(slot_byte_len)
            .min(atlas_upload_byte_len)
    };

    SdfAtlasUploadReport {
        mode: if atlas_upload_byte_len == 0 {
            SdfAtlasUploadMode::None
        } else if atlas_upload_full_texture {
            SdfAtlasUploadMode::FullTexture
        } else {
            SdfAtlasUploadMode::DirtySlots
        },
        byte_len: atlas_upload_byte_len,
        full_texture: atlas_upload_full_texture,
        dirty_slot_count,
        dirty_byte_len,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::math::UVec2;
    use crate::graphics::scene::scene_renderer::ui::sdf_atlas::{
        SdfAtlasGlyphKey, SdfAtlasRect, SdfAtlasSlot,
    };

    #[test]
    fn sdf_upload_report_uses_full_texture_when_atlas_resizes() {
        let report = sdf_atlas_upload_report(
            &atlas_plan(2),
            SdfAtlasCacheReport {
                previous_slot_count: 0,
                current_slot_count: 2,
                retained_slot_count: 0,
                stable_slot_count: 0,
                relocated_slot_count: 0,
                added_slot_count: 2,
                evicted_slot_count: 0,
                atlas_resized: true,
            },
            true,
            512 * 512,
            true,
        );

        assert_eq!(report.mode, SdfAtlasUploadMode::FullTexture);
        assert!(report.full_texture);
        assert_eq!(report.byte_len, 512 * 512);
        assert_eq!(report.dirty_slot_count, 2);
        assert_eq!(report.dirty_byte_len, 512 * 512);
    }

    #[test]
    fn sdf_upload_report_tracks_future_partial_dirty_slots() {
        let stable = sdf_atlas_upload_report(
            &atlas_plan(2),
            SdfAtlasCacheReport {
                previous_slot_count: 2,
                current_slot_count: 2,
                retained_slot_count: 2,
                stable_slot_count: 2,
                relocated_slot_count: 0,
                added_slot_count: 0,
                evicted_slot_count: 0,
                atlas_resized: false,
            },
            false,
            512 * 512,
            true,
        );

        assert_eq!(stable.mode, SdfAtlasUploadMode::FullTexture);
        assert_eq!(stable.dirty_slot_count, 0);
        assert_eq!(stable.dirty_byte_len, 0);

        let relocated = sdf_atlas_upload_report(
            &atlas_plan(2),
            SdfAtlasCacheReport {
                previous_slot_count: 3,
                current_slot_count: 3,
                retained_slot_count: 2,
                stable_slot_count: 1,
                relocated_slot_count: 1,
                added_slot_count: 1,
                evicted_slot_count: 1,
                atlas_resized: false,
            },
            false,
            512 * 512,
            true,
        );

        assert_eq!(relocated.dirty_slot_count, 2);
        assert_eq!(relocated.dirty_byte_len, 2 * 64 * 64);
    }

    fn atlas_plan(slot_count: usize) -> SdfAtlasPlan {
        let slots = (0..slot_count)
            .map(|index| SdfAtlasSlot {
                key: SdfAtlasGlyphKey {
                    glyph: char::from_u32('A' as u32 + index as u32).unwrap_or('A'),
                    font: Some("res://fonts/default.font.toml".to_string()),
                    font_family: Some("Zircon Sans".to_string()),
                    font_size_milli: 16_000,
                },
                rect: SdfAtlasRect {
                    x: index as u32 * 64,
                    y: 0,
                    width: 64,
                    height: 64,
                },
            })
            .collect();
        SdfAtlasPlan {
            atlas_size: UVec2::splat(512),
            slots,
            runs: Vec::new(),
        }
    }
}
