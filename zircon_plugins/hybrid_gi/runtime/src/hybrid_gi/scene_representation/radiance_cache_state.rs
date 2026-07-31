use std::collections::BTreeSet;

use zircon_runtime::core::math::Vec3;

use super::{
    screen_probe_state::HybridGiScreenProbeDescriptor,
    surface_cache_state::HybridGiSurfaceCacheState, voxel_scene_state::HybridGiVoxelSceneState,
};

const RADIANCE_CACHE_CLIPMAP_LEVEL_COUNT: u32 = 4;
const RADIANCE_CACHE_CLIPMAP_RESOLUTION: u32 = 48;
const RADIANCE_CACHE_BASE_CELL_SIZE: f32 = 1.0;
const RADIANCE_CACHE_CLIPMAP_LEVEL_SCALE: f32 = 2.0;
const RADIANCE_CACHE_PROBE_CENTER_OFFSET: f32 = 0.5;
const SURFACE_CACHE_CAPTURE_CONFIDENCE_Q8: u8 = 255;
const SURFACE_CACHE_ATLAS_CONFIDENCE_Q8: u8 = 220;
const VOXEL_FALLBACK_CONFIDENCE_Q8: u8 = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HybridGiRadianceCacheSource {
    SurfaceCache,
    VoxelFallback,
    Missing,
}

impl HybridGiRadianceCacheSource {
    #[cfg(test)]
    fn label(self) -> &'static str {
        match self {
            Self::SurfaceCache => "surface-cache",
            Self::VoxelFallback => "voxel-fallback",
            Self::Missing => "missing",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(not(test), allow(dead_code))]
struct HybridGiRadianceCacheEntry {
    probe_id: u32,
    card_id: u32,
    surface_page_id: Option<u32>,
    radiance_rgb: [u8; 3],
    confidence_q8: u8,
    source: HybridGiRadianceCacheSource,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct HybridGiRadianceCacheClipmapDescriptor {
    level: u32,
    anchor: Vec3,
    cell_size: f32,
    resolution: u32,
}

impl HybridGiRadianceCacheClipmapDescriptor {
    fn interpolation_cell(self, world_position: Vec3) -> Option<[i32; 3]> {
        if !world_position.is_finite()
            || !self.anchor.is_finite()
            || !self.cell_size.is_finite()
            || self.cell_size <= 0.0
        {
            return None;
        }

        let relative_position = world_position - self.anchor;
        if !relative_position.is_finite() {
            return None;
        }
        let probe_coord = relative_position / self.cell_size
            + Vec3::splat(self.resolution as f32 * RADIANCE_CACHE_PROBE_CENTER_OFFSET);
        if !probe_coord.is_finite() {
            return None;
        }

        let lower_bound = RADIANCE_CACHE_PROBE_CENTER_OFFSET;
        let upper_bound = self.resolution as f32 - RADIANCE_CACHE_PROBE_CENTER_OFFSET;
        if ![probe_coord.x, probe_coord.y, probe_coord.z]
            .into_iter()
            .all(|component| component > lower_bound && component < upper_bound)
        {
            return None;
        }

        Some([
            (probe_coord.x - RADIANCE_CACHE_PROBE_CENTER_OFFSET).floor() as i32,
            (probe_coord.y - RADIANCE_CACHE_PROBE_CENTER_OFFSET).floor() as i32,
            (probe_coord.z - RADIANCE_CACHE_PROBE_CENTER_OFFSET).floor() as i32,
        ])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(not(test), allow(dead_code))]
struct HybridGiRadianceProbeDemand {
    clipmap_level: u32,
    probe_coord: [i32; 3],
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(in crate::hybrid_gi::scene_representation) struct HybridGiRadianceCacheState {
    clipmaps: Vec<HybridGiRadianceCacheClipmapDescriptor>,
    #[cfg_attr(not(test), allow(dead_code))]
    probe_demands: Vec<HybridGiRadianceProbeDemand>,
    entries: Vec<HybridGiRadianceCacheEntry>,
}

impl HybridGiRadianceCacheState {
    pub(in crate::hybrid_gi::scene_representation) fn synchronize(
        &mut self,
        probes: &[HybridGiScreenProbeDescriptor],
        surface_cache: &HybridGiSurfaceCacheState,
        voxel_scene: &HybridGiVoxelSceneState,
    ) {
        self.clipmaps = build_radiance_cache_clipmaps(probes);
        self.probe_demands = mark_radiance_probe_demands(probes, &self.clipmaps);

        let surface_cache_page_contents = surface_cache.page_contents_snapshot();
        let voxel_cells = voxel_scene.voxel_cells_snapshot();

        self.entries = probes
            .iter()
            .map(|probe| {
                let surface_cache_radiance = probe.surface_page_id().and_then(|surface_page_id| {
                    surface_cache_radiance(surface_page_id, &surface_cache_page_contents)
                });
                let voxel_radiance =
                    voxel_fallback_radiance(probe.card_id(), &voxel_cells).map(|radiance_rgb| {
                        (
                            radiance_rgb,
                            VOXEL_FALLBACK_CONFIDENCE_Q8,
                            HybridGiRadianceCacheSource::VoxelFallback,
                        )
                    });
                let (radiance_rgb, confidence_q8, source) = surface_cache_radiance
                    .or(voxel_radiance)
                    .unwrap_or(([0, 0, 0], 0, HybridGiRadianceCacheSource::Missing));

                HybridGiRadianceCacheEntry {
                    probe_id: probe.probe_id(),
                    card_id: probe.card_id(),
                    surface_page_id: probe.surface_page_id(),
                    radiance_rgb,
                    confidence_q8,
                    source,
                }
            })
            .collect();
    }

    pub(in crate::hybrid_gi::scene_representation) fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub(in crate::hybrid_gi::scene_representation) fn radiance_rgb(
        &self,
        probe_id: u32,
    ) -> Option<[u8; 3]> {
        self.entries
            .iter()
            .find_map(|entry| (entry.probe_id == probe_id).then_some(entry.radiance_rgb))
    }

    #[cfg(test)]
    pub(in crate::hybrid_gi::scene_representation) fn clipmap_topology(
        &self,
    ) -> Vec<(u32, u32, f32)> {
        self.clipmaps
            .iter()
            .map(|clipmap| (clipmap.level, clipmap.resolution, clipmap.cell_size))
            .collect()
    }

    #[cfg(test)]
    pub(in crate::hybrid_gi::scene_representation) fn probe_demands(&self) -> Vec<(u32, [i32; 3])> {
        self.probe_demands
            .iter()
            .map(|demand| (demand.clipmap_level, demand.probe_coord))
            .collect()
    }

    #[cfg(test)]
    pub(in crate::hybrid_gi::scene_representation) fn entries(
        &self,
    ) -> Vec<(u32, u32, Option<u32>, [u8; 3], u8, &'static str)> {
        self.entries
            .iter()
            .map(|entry| {
                (
                    entry.probe_id,
                    entry.card_id,
                    entry.surface_page_id,
                    entry.radiance_rgb,
                    entry.confidence_q8,
                    entry.source.label(),
                )
            })
            .collect()
    }
}

fn build_radiance_cache_clipmaps(
    probes: &[HybridGiScreenProbeDescriptor],
) -> Vec<HybridGiRadianceCacheClipmapDescriptor> {
    let Some(anchor) = probes
        .iter()
        .map(HybridGiScreenProbeDescriptor::bounds_center)
        .find(|center| center.is_finite())
    else {
        return Vec::new();
    };

    (0..RADIANCE_CACHE_CLIPMAP_LEVEL_COUNT)
        .map(|level| HybridGiRadianceCacheClipmapDescriptor {
            level,
            anchor,
            cell_size: RADIANCE_CACHE_BASE_CELL_SIZE
                * RADIANCE_CACHE_CLIPMAP_LEVEL_SCALE.powi(level as i32),
            resolution: RADIANCE_CACHE_CLIPMAP_RESOLUTION,
        })
        .collect()
}

fn mark_radiance_probe_demands(
    probes: &[HybridGiScreenProbeDescriptor],
    clipmaps: &[HybridGiRadianceCacheClipmapDescriptor],
) -> Vec<HybridGiRadianceProbeDemand> {
    let mut demands = BTreeSet::new();

    for probe in probes {
        let world_position = probe.bounds_center();
        let Some((clipmap, bottom)) = clipmaps.iter().copied().find_map(|clipmap| {
            clipmap
                .interpolation_cell(world_position)
                .map(|bottom| (clipmap, bottom))
        }) else {
            continue;
        };
        for x_offset in 0..=1 {
            for y_offset in 0..=1 {
                for z_offset in 0..=1 {
                    demands.insert(HybridGiRadianceProbeDemand {
                        clipmap_level: clipmap.level,
                        probe_coord: [
                            bottom[0] + x_offset,
                            bottom[1] + y_offset,
                            bottom[2] + z_offset,
                        ],
                    });
                }
            }
        }
    }

    demands.into_iter().collect()
}

fn surface_cache_radiance(
    surface_page_id: u32,
    surface_cache_page_contents: &[(u32, u32, u32, u32, [u8; 4], [u8; 4])],
) -> Option<([u8; 3], u8, HybridGiRadianceCacheSource)> {
    surface_cache_page_contents
        .iter()
        .find(|(page_id, _, _, _, _, _)| *page_id == surface_page_id)
        .and_then(|(_, _, _, _, atlas_sample_rgba, capture_sample_rgba)| {
            if capture_sample_rgba[3] > 0 {
                return Some((
                    [
                        capture_sample_rgba[0],
                        capture_sample_rgba[1],
                        capture_sample_rgba[2],
                    ],
                    SURFACE_CACHE_CAPTURE_CONFIDENCE_Q8,
                    HybridGiRadianceCacheSource::SurfaceCache,
                ));
            }
            if atlas_sample_rgba[3] > 0 {
                return Some((
                    [
                        atlas_sample_rgba[0],
                        atlas_sample_rgba[1],
                        atlas_sample_rgba[2],
                    ],
                    SURFACE_CACHE_ATLAS_CONFIDENCE_Q8,
                    HybridGiRadianceCacheSource::SurfaceCache,
                ));
            }
            None
        })
}

fn voxel_fallback_radiance(
    card_id: u32,
    voxel_cells: &[crate::hybrid_gi::HybridGiPrepareVoxelCell],
) -> Option<[u8; 3]> {
    let mut best_radiance = None;
    let mut best_strength = 0_u32;

    for cell in voxel_cells {
        if cell.dominant_card_id != card_id || !cell.radiance_present {
            continue;
        }

        let strength = radiance_strength(cell.radiance_rgb);
        if best_radiance.is_none() || strength > best_strength {
            best_radiance = Some(cell.radiance_rgb);
            best_strength = strength;
        }
    }

    best_radiance
}

fn radiance_strength(radiance_rgb: [u8; 3]) -> u32 {
    radiance_rgb[0] as u32 + radiance_rgb[1] as u32 + radiance_rgb[2] as u32
}
