use crate::core::framework::render::{CookieProjection, CookieWrapMode, LightCookieData};
use crate::core::math::Vec2;
use crate::core::resource::ResourceId;

use super::{
    COOKIE_PROJECTION_DIRECTIONAL, COOKIE_PROJECTION_POINT_OCTAHEDRAL, COOKIE_PROJECTION_SPOT,
};

pub(crate) const COOKIE_ATLAS_GRID_SIZE: u32 = 8;
pub(crate) const COOKIE_ATLAS_MAX_ENTRIES: usize =
    (COOKIE_ATLAS_GRID_SIZE * COOKIE_ATLAS_GRID_SIZE) as usize;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct CookieGpuMetadata {
    pub(crate) uv_rect: [f32; 4],
    pub(crate) misc: [u32; 4],
    pub(crate) directional_offset_scale: [f32; 4],
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CookieAtlasEntry {
    pub(crate) slot: u32,
    pub(crate) light_id: u64,
    pub(crate) texture: ResourceId,
    pub(crate) metadata: CookieGpuMetadata,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct CookieFramePlan {
    entries: Vec<CookieAtlasEntry>,
}

impl CookieFramePlan {
    pub(crate) fn entries(&self) -> &[CookieAtlasEntry] {
        &self.entries
    }

    pub(crate) fn metadata_for_light(&self, light_id: u64) -> Option<CookieGpuMetadata> {
        self.entries
            .binary_search_by_key(&light_id, |entry| entry.light_id)
            .ok()
            .map(|index| self.entries[index].metadata)
    }
}

pub(crate) fn build_cookie_frame_plan(cookies: &[LightCookieData]) -> CookieFramePlan {
    let mut indexed = Vec::with_capacity(cookies.len());
    indexed.extend(cookies.iter().enumerate());
    indexed.sort_unstable_by_key(|(input_index, cookie)| (cookie.light_id, *input_index));
    let cell = 1.0 / COOKIE_ATLAS_GRID_SIZE as f32;
    let mut entries = Vec::with_capacity(indexed.len().min(COOKIE_ATLAS_MAX_ENTRIES));
    let mut cursor = 0;
    while cursor < indexed.len() && entries.len() < COOKIE_ATLAS_MAX_ENTRIES {
        let light_id = indexed[cursor].1.light_id;
        let mut group_end = cursor + 1;
        while group_end < indexed.len() && indexed[group_end].1.light_id == light_id {
            group_end += 1;
        }
        let cookie = indexed[group_end - 1].1;
        let slot = entries.len() as u32;
        let x = slot % COOKIE_ATLAS_GRID_SIZE;
        let y = slot / COOKIE_ATLAS_GRID_SIZE;
        let (projection, wrap, offset, scale) = projection_metadata(cookie.projection);
        entries.push(CookieAtlasEntry {
            slot,
            light_id,
            texture: cookie.texture,
            metadata: CookieGpuMetadata {
                uv_rect: [x as f32 * cell, y as f32 * cell, cell, cell],
                misc: [projection, wrap, 0, 0],
                directional_offset_scale: [offset.x, offset.y, scale.x, scale.y],
            },
        });
        cursor = group_end;
    }
    CookieFramePlan { entries }
}

fn projection_metadata(projection: CookieProjection) -> (u32, u32, Vec2, Vec2) {
    match projection {
        CookieProjection::Directional {
            offset,
            scale,
            wrap,
        } => (
            COOKIE_PROJECTION_DIRECTIONAL,
            wrap_mode(wrap),
            offset,
            scale,
        ),
        CookieProjection::Spot => (
            COOKIE_PROJECTION_SPOT,
            wrap_mode(CookieWrapMode::Clamp),
            Vec2::ZERO,
            Vec2::ONE,
        ),
        CookieProjection::PointOctahedral => (
            COOKIE_PROJECTION_POINT_OCTAHEDRAL,
            wrap_mode(CookieWrapMode::Clamp),
            Vec2::ZERO,
            Vec2::ONE,
        ),
    }
}

const fn wrap_mode(mode: CookieWrapMode) -> u32 {
    match mode {
        CookieWrapMode::Clamp => 0,
        CookieWrapMode::Repeat => 1,
    }
}

#[cfg(test)]
mod performance_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cookie_frame_plan_is_sorted_deduplicated_and_uses_fixed_atlas_cells() {
        let texture_a = ResourceId::from_stable_label("runtime://cookie/a");
        let texture_b = ResourceId::from_stable_label("runtime://cookie/b");
        let plan = build_cookie_frame_plan(&[
            LightCookieData {
                light_id: 9,
                texture: texture_a,
                projection: CookieProjection::Spot,
            },
            LightCookieData {
                light_id: 3,
                texture: texture_b,
                projection: CookieProjection::PointOctahedral,
            },
            LightCookieData {
                light_id: 9,
                texture: texture_b,
                projection: CookieProjection::Directional {
                    offset: Vec2::new(0.25, 0.5),
                    scale: Vec2::new(2.0, 3.0),
                    wrap: CookieWrapMode::Repeat,
                },
            },
        ]);

        assert_eq!(plan.entries().len(), 2);
        assert_eq!(plan.entries()[0].light_id, 3);
        assert_eq!(plan.entries()[1].light_id, 9);
        assert_eq!(plan.entries()[0].slot, 0);
        assert_eq!(plan.entries()[1].slot, 1);
        assert_eq!(plan.entries()[0].metadata.uv_rect, [0.0, 0.0, 0.125, 0.125]);
        assert_eq!(
            plan.entries()[1].metadata.uv_rect,
            [0.125, 0.0, 0.125, 0.125]
        );
        assert_eq!(
            plan.entries()[1].metadata.misc,
            [COOKIE_PROJECTION_DIRECTIONAL, 1, 0, 0]
        );
        assert_eq!(
            plan.entries()[1].metadata.directional_offset_scale,
            [0.25, 0.5, 2.0, 3.0]
        );
    }
}
