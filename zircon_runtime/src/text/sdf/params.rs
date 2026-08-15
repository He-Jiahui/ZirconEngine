use super::SdfMode;

const DEFAULT_SDF_BAKE_EM_PX: u32 = 48;
const DEFAULT_SDF_SPREAD_PX_MILLI: u32 = 8_000;
const MIN_SCREEN_PX_RANGE: f32 = 1.0;
const PX_MILLI_SCALE: f32 = 1_000.0;

/// Stable bake identity shared by runtime generation, atlas keys, and offline artifacts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct SdfBakeParams {
    pub(crate) mode: SdfMode,
    pub(crate) bake_em_px: u32,
    pub(crate) spread_px_milli: u32,
}

impl Default for SdfBakeParams {
    fn default() -> Self {
        Self {
            mode: SdfMode::Sdf,
            bake_em_px: DEFAULT_SDF_BAKE_EM_PX,
            spread_px_milli: DEFAULT_SDF_SPREAD_PX_MILLI,
        }
    }
}

impl SdfBakeParams {
    pub(crate) fn for_mode(mode: SdfMode) -> Self {
        Self {
            mode,
            ..Self::default()
        }
    }

    pub(crate) fn normalized(self) -> Self {
        Self {
            mode: self.mode,
            bake_em_px: self.bake_em_px.max(1),
            spread_px_milli: self.spread_px_milli.max(1),
        }
    }

    pub(crate) fn bake_em_px_f32(self) -> f32 {
        self.normalized().bake_em_px as f32
    }

    pub(crate) fn spread_px_f32(self) -> f32 {
        self.normalized().spread_px_milli as f32 / PX_MILLI_SCALE
    }

    pub(crate) fn screen_px_range(self, display_px: f32) -> f32 {
        if !display_px.is_finite() || display_px <= 0.0 {
            return MIN_SCREEN_PX_RANGE;
        }
        let params = self.normalized();
        ((display_px / params.bake_em_px_f32()) * params.spread_px_f32()).max(MIN_SCREEN_PX_RANGE)
    }
}
