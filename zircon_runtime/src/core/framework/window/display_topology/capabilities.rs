/// A backend observation, intentionally distinct from compile-time feature
/// selection. `Unknown` is never promoted to a usable output capability.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DisplayFeatureState {
    #[default]
    Unknown,
    Unavailable,
    Available,
}

/// The effective output color space when the backend can observe it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DisplayColorSpace {
    #[default]
    Unknown,
    Srgb,
    DisplayP3,
    Rec2020,
}

/// Per-output facts observed from the active platform backend.
///
/// These facts describe an output only. Surface negotiation records the
/// requested and effective render format separately.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DisplayOutputCapabilities {
    pub hdr: DisplayFeatureState,
    pub variable_refresh_rate: DisplayFeatureState,
    pub wide_color_gamut: DisplayFeatureState,
    pub color_space: DisplayColorSpace,
}
