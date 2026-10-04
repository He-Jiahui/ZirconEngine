//! Versioned AppSession V2 configuration negotiated before native IME events.
//!
//! The descriptor owns the public candidate rectangle topology.  It is passed through a
//! standalone symbol so the frozen V8 function table and the V3 session creation ABI remain
//! unchanged.  Every field is bounded and validated before a runtime session is touched.

use core::mem::size_of;

use super::ime_composition_capability_v2::ZrRuntimeImeCompositionNegotiation;
use super::ime_composition_capability_v2::ZR_RUNTIME_IME_COMPOSITION_V2_MIN_API_VERSION;

pub const ZR_RUNTIME_APP_SESSION_CONFIGURATION_ABI_VERSION_V2: u32 = 2;
pub const ZR_RUNTIME_APP_SESSION_CAPABILITY_IME_COMPOSITION_V2: u64 = 1 << 0;
pub const ZR_RUNTIME_APP_SESSION_CAPABILITY_CANDIDATE_RECT_V2: u64 = 1 << 1;
pub const ZR_RUNTIME_APP_SESSION_COMPOSITION_SCHEMA_V2: u16 = 2;
pub const ZR_RUNTIME_APP_SESSION_CANDIDATE_RECT_SCHEMA_V2: u16 = 1;
pub const ZR_RUNTIME_APP_SESSION_COORDINATE_SPACE_WINDOW_V1: u32 = 1;
pub const ZR_RUNTIME_APP_SESSION_MAX_NATIVE_COORDINATE_V2: i64 = i32::MAX as i64;
pub const ZR_RUNTIME_CONFIGURE_APP_SESSION_SYMBOL_V2: &[u8] =
    b"zircon_runtime_configure_app_session_v2\0";

const REQUIRED_CAPABILITIES: u64 = ZR_RUNTIME_APP_SESSION_CAPABILITY_IME_COMPOSITION_V2
    | ZR_RUNTIME_APP_SESSION_CAPABILITY_CANDIDATE_RECT_V2;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrRuntimeImeCandidateRectV2 {
    /// Window-relative left edge of the text bounds in pixels.
    pub origin_x: i32,
    /// Window-relative top edge of the text bounds in pixels.
    pub origin_y: i32,
    /// Text-bound width.  Windows IMM uses a zero-width exclusion area at the caret.
    pub extent_width: u32,
    /// Text-bound height; must be nonzero for a candidate popup anchor.
    pub extent_height: u32,
    /// Explicit coordinate-space tag; screen to window conversion is a host responsibility.
    pub coordinate_space: u32,
    pub reserved: u32,
}

impl ZrRuntimeImeCandidateRectV2 {
    pub const fn window_relative(origin_x: i32, origin_y: i32, width: u32, height: u32) -> Self {
        Self {
            origin_x,
            origin_y,
            extent_width: width,
            extent_height: height,
            coordinate_space: ZR_RUNTIME_APP_SESSION_COORDINATE_SPACE_WINDOW_V1,
            reserved: 0,
        }
    }

    fn validate(&self) -> Result<(), ZrRuntimeAppSessionConfigurationError> {
        if self.coordinate_space != ZR_RUNTIME_APP_SESSION_COORDINATE_SPACE_WINDOW_V1 {
            return Err(ZrRuntimeAppSessionConfigurationError::UnsupportedCoordinateSpace);
        }
        if self.extent_height == 0 {
            return Err(ZrRuntimeAppSessionConfigurationError::InvalidCandidateRect);
        }
        if self.extent_width > ZR_RUNTIME_APP_SESSION_MAX_NATIVE_COORDINATE_V2 as u32
            || self.extent_height > ZR_RUNTIME_APP_SESSION_MAX_NATIVE_COORDINATE_V2 as u32
            || i64::from(self.origin_x) + i64::from(self.extent_width)
                > ZR_RUNTIME_APP_SESSION_MAX_NATIVE_COORDINATE_V2
            || i64::from(self.origin_y) + i64::from(self.extent_height)
                > ZR_RUNTIME_APP_SESSION_MAX_NATIVE_COORDINATE_V2
        {
            return Err(ZrRuntimeAppSessionConfigurationError::UnrepresentableCandidateRect);
        }
        if self.reserved != 0 {
            return Err(ZrRuntimeAppSessionConfigurationError::NonZeroReserved);
        }
        Ok(())
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrRuntimeAppSessionConfigurationV2 {
    pub abi_version: u32,
    pub size_bytes: usize,
    pub capability_bits: u64,
    pub peer_api_version: u32,
    pub composition_schema: u16,
    pub candidate_rect_schema: u16,
    pub reserved: u32,
    pub window_generation: u64,
    pub candidate_rect: ZrRuntimeImeCandidateRectV2,
    pub reserved_tail: [u64; 2],
}

impl ZrRuntimeAppSessionConfigurationV2 {
    pub const SIZE_BYTES: usize = size_of::<Self>();

    pub const fn ime_composition(
        window_generation: u64,
        candidate_rect: ZrRuntimeImeCandidateRectV2,
    ) -> Self {
        Self {
            abi_version: ZR_RUNTIME_APP_SESSION_CONFIGURATION_ABI_VERSION_V2,
            size_bytes: Self::SIZE_BYTES,
            capability_bits: REQUIRED_CAPABILITIES,
            peer_api_version: ZR_RUNTIME_IME_COMPOSITION_V2_MIN_API_VERSION,
            composition_schema: ZR_RUNTIME_APP_SESSION_COMPOSITION_SCHEMA_V2,
            candidate_rect_schema: ZR_RUNTIME_APP_SESSION_CANDIDATE_RECT_SCHEMA_V2,
            reserved: 0,
            window_generation,
            candidate_rect,
            reserved_tail: [0; 2],
        }
    }

    pub fn validate(&self) -> Result<(), ZrRuntimeAppSessionConfigurationError> {
        if self.abi_version != ZR_RUNTIME_APP_SESSION_CONFIGURATION_ABI_VERSION_V2 {
            return Err(ZrRuntimeAppSessionConfigurationError::UnsupportedVersion);
        }
        if self.size_bytes != Self::SIZE_BYTES {
            return Err(ZrRuntimeAppSessionConfigurationError::SizeMismatch);
        }
        if self.capability_bits & !REQUIRED_CAPABILITIES != 0 {
            return Err(ZrRuntimeAppSessionConfigurationError::UnknownCapability);
        }
        if self.capability_bits & REQUIRED_CAPABILITIES != REQUIRED_CAPABILITIES {
            return Err(ZrRuntimeAppSessionConfigurationError::MissingRequiredCapability);
        }
        if self.reserved != 0 || self.reserved_tail != [0; 2] {
            return Err(ZrRuntimeAppSessionConfigurationError::NonZeroReserved);
        }
        if self.window_generation == 0 {
            return Err(ZrRuntimeAppSessionConfigurationError::InvalidWindowGeneration);
        }
        if self.peer_api_version < ZR_RUNTIME_IME_COMPOSITION_V2_MIN_API_VERSION {
            return Err(ZrRuntimeAppSessionConfigurationError::UnsupportedPeerApiVersion);
        }
        if self.composition_schema != ZR_RUNTIME_APP_SESSION_COMPOSITION_SCHEMA_V2 {
            return Err(ZrRuntimeAppSessionConfigurationError::UnsupportedCompositionSchema);
        }
        if self.candidate_rect_schema != ZR_RUNTIME_APP_SESSION_CANDIDATE_RECT_SCHEMA_V2 {
            return Err(ZrRuntimeAppSessionConfigurationError::UnsupportedCandidateRectSchema);
        }
        self.candidate_rect.validate()
    }

    pub fn negotiation(
        &self,
    ) -> Result<ZrRuntimeImeCompositionNegotiation, ZrRuntimeAppSessionConfigurationError> {
        self.validate()?;
        Ok(ZrRuntimeImeCompositionNegotiation::from_raw(
            self.peer_api_version,
            Some(self.composition_schema),
            true,
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZrRuntimeAppSessionConfigurationError {
    UnsupportedVersion,
    SizeMismatch,
    UnknownCapability,
    MissingRequiredCapability,
    NonZeroReserved,
    InvalidWindowGeneration,
    UnsupportedPeerApiVersion,
    UnsupportedCompositionSchema,
    UnsupportedCandidateRectSchema,
    UnsupportedCoordinateSpace,
    InvalidCandidateRect,
    UnrepresentableCandidateRect,
}

impl core::fmt::Display for ZrRuntimeAppSessionConfigurationError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            Self::UnsupportedVersion => "unsupported AppSession V2 configuration ABI version",
            Self::SizeMismatch => "AppSession V2 configuration size does not match",
            Self::UnknownCapability => "AppSession V2 configuration contains an unknown capability",
            Self::MissingRequiredCapability => {
                "AppSession V2 configuration is missing a required capability"
            }
            Self::NonZeroReserved => "AppSession V2 configuration reserved fields must be zero",
            Self::InvalidWindowGeneration => "AppSession V2 window generation is invalid",
            Self::UnsupportedPeerApiVersion => "AppSession V2 peer API version is unsupported",
            Self::UnsupportedCompositionSchema => "AppSession V2 composition schema is unsupported",
            Self::UnsupportedCandidateRectSchema => {
                "AppSession V2 candidate rectangle schema is unsupported"
            }
            Self::UnsupportedCoordinateSpace => {
                "AppSession V2 candidate rectangle coordinate space is unsupported"
            }
            Self::InvalidCandidateRect => "AppSession V2 candidate rectangle height is invalid",
            Self::UnrepresentableCandidateRect => {
                "AppSession V2 candidate rectangle exceeds native coordinate bounds"
            }
        };
        formatter.write_str(message)
    }
}

#[cfg(test)]
#[path = "tests/app_session_configuration_v2.rs"]
mod tests;
