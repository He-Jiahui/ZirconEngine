use super::{
    screen_probe_state::HybridGiScreenProbeDescriptor,
    surface_cache_state::HybridGiSurfaceCacheState, voxel_scene_state::HybridGiVoxelSceneState,
};

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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(in crate::hybrid_gi::scene_representation) struct HybridGiRadianceCacheState {
    entries: Vec<HybridGiRadianceCacheEntry>,
}

impl HybridGiRadianceCacheState {
    pub(in crate::hybrid_gi::scene_representation) fn synchronize(
        &mut self,
        probes: &[HybridGiScreenProbeDescriptor],
        surface_cache: &HybridGiSurfaceCacheState,
        voxel_scene: &HybridGiVoxelSceneState,
    ) {
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
