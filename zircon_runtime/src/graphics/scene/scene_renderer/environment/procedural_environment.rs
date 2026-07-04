use crate::core::framework::render::ProceduralSkyParams;
use crate::core::math::{Real, Vec3};

pub(crate) fn procedural_sky_color_at_vertical(
    params: ProceduralSkyParams,
    vertical01: Real,
) -> Vec3 {
    let t = vertical01.clamp(0.0, 1.0);
    params
        .horizon_color
        .truncate()
        .lerp(params.zenith_color.truncate(), t)
        * params.intensity.max(0.0)
}

pub(crate) fn roughness_from_smoothness(smoothness: Real) -> Real {
    (1.0 - smoothness).clamp(0.04, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framework::render::ProceduralSkyParams;

    #[test]
    fn procedural_sky_gradient_matches_contract_endpoints() {
        let params = ProceduralSkyParams::default_gradient();

        assert_eq!(
            procedural_sky_color_at_vertical(params, 0.0),
            params.horizon_color.truncate()
        );
        assert_eq!(
            procedural_sky_color_at_vertical(params, 1.0),
            params.zenith_color.truncate()
        );
    }

    #[test]
    fn roughness_from_smoothness_clamps_for_pbr_sampling() {
        assert_eq!(roughness_from_smoothness(0.0), 1.0);
        assert_eq!(roughness_from_smoothness(0.5), 0.5);
        assert_eq!(roughness_from_smoothness(1.0), 0.04);
        assert_eq!(roughness_from_smoothness(2.0), 0.04);
    }
}
