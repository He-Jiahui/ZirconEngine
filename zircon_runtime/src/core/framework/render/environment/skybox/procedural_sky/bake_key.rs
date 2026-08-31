use crate::core::math::Vec4;

use super::super::{IblBakeKey, SkyboxMode};
use super::{ProceduralSkyParams, ResolvedProceduralSun};

impl ProceduralSkyParams {
    pub fn ibl_bake_key(&self) -> IblBakeKey {
        let sun = self.resolved_sun();
        IblBakeKey {
            source_kind: SkyboxMode::ProceduralGradient.source_kind(),
            source_revision: self.source_revision,
            horizon_color: vec4_bits(self.horizon_color),
            zenith_color: vec4_bits(self.zenith_color),
            ground_color: vec4_bits(self.ground_color),
            source_hash: procedural_sun_hash(self, sun),
        }
    }
}

fn vec4_bits(value: Vec4) -> [u32; 4] {
    [
        value.x.to_bits(),
        value.y.to_bits(),
        value.z.to_bits(),
        value.w.to_bits(),
    ]
}

fn procedural_sun_hash(params: &ProceduralSkyParams, sun: ResolvedProceduralSun) -> [u32; 4] {
    if sun.direction.w < 0.5 {
        return [0; 4];
    }

    let mut hasher = blake3::Hasher::new();
    for value in [
        sun.direction.x,
        sun.direction.y,
        sun.direction.z,
        params.sun_color.x,
        params.sun_color.y,
        params.sun_color.z,
        sun.intensity_and_cosines.x,
        sun.intensity_and_cosines.y,
        sun.intensity_and_cosines.z,
    ] {
        hasher.update(&value.to_bits().to_le_bytes());
    }
    let hash = hasher.finalize();
    let bytes = hash.as_bytes();
    [
        u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
        u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
        u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]),
    ]
}
