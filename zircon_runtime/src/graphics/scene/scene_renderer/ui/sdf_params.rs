const DEFAULT_SDF_BAKE_EM_PX: u32 = 32;
const DEFAULT_SDF_SPREAD_PX_MILLI: u32 = 8_000;
const MIN_SCREEN_PX_RANGE: f32 = 1.0;
const PX_MILLI_SCALE: f32 = 1000.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) enum SdfBakeMode {
    Sdf,
}

impl Default for SdfBakeMode {
    fn default() -> Self {
        Self::Sdf
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct SdfBakeParams {
    pub(super) mode: SdfBakeMode,
    pub(super) bake_em_px: u32,
    pub(super) spread_px_milli: u32,
}

impl Default for SdfBakeParams {
    fn default() -> Self {
        Self {
            mode: SdfBakeMode::Sdf,
            bake_em_px: DEFAULT_SDF_BAKE_EM_PX,
            spread_px_milli: DEFAULT_SDF_SPREAD_PX_MILLI,
        }
    }
}

impl SdfBakeParams {
    pub(super) fn normalized(self) -> Self {
        Self {
            mode: self.mode,
            bake_em_px: self.bake_em_px.max(1),
            spread_px_milli: self.spread_px_milli.max(1),
        }
    }

    pub(super) fn bake_em_px_f32(self) -> f32 {
        self.normalized().bake_em_px as f32
    }

    pub(super) fn spread_px_f32(self) -> f32 {
        self.normalized().spread_px_milli as f32 / PX_MILLI_SCALE
    }

    pub(super) fn screen_px_range(self, display_px: f32) -> f32 {
        let params = self.normalized();
        let display_px = display_px.max(1.0);
        ((display_px / params.bake_em_px_f32()) * params.spread_px_f32()).max(MIN_SCREEN_PX_RANGE)
    }
}
