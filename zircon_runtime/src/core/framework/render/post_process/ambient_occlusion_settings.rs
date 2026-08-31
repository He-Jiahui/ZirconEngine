use crate::core::math::Real;

pub const AO_SOURCE_SETTINGS_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum AoQualityTier {
    Low = 0,
    Medium = 1,
    #[default]
    High = 2,
    Ultra = 3,
}

impl AoQualityTier {
    pub const fn from_stable_id(id: u32) -> Option<Self> {
        match id {
            0 => Some(Self::Low),
            1 => Some(Self::Medium),
            2 => Some(Self::High),
            3 => Some(Self::Ultra),
            _ => None,
        }
    }

    pub const fn stable_id(self) -> u32 {
        self as u32
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AoSourceSettings {
    /// Strength applied only by an indirect-light AO consumer.
    pub intensity: Real,
    /// World-space occlusion search radius in meters.
    pub radius_meters: Real,
    /// Maximum accepted occluder separation in meters.
    pub thickness_meters: Real,
    /// World-space depth bias in meters used to reject self-occlusion.
    pub depth_bias_meters: Real,
    /// Distance inside the search radius where radial falloff starts.
    pub falloff_start_meters: Real,
    pub quality: AoQualityTier,
    pub half_resolution: bool,
    /// Authoring request only. Compilation remains fail-closed until motion-qualified history exists.
    pub temporal: bool,
}

impl AoSourceSettings {
    pub const DEFAULT: Self = Self {
        intensity: 1.0,
        radius_meters: 1.0,
        thickness_meters: 0.15,
        depth_bias_meters: 0.02,
        falloff_start_meters: 0.5,
        quality: AoQualityTier::High,
        half_resolution: false,
        temporal: false,
    };
}

impl Default for AoSourceSettings {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AoSourceSettingsKey {
    version: u32,
    intensity_bits: u32,
    radius_meters_bits: u32,
    thickness_meters_bits: u32,
    depth_bias_meters_bits: u32,
    falloff_start_meters_bits: u32,
    quality: u32,
    half_resolution: bool,
    temporal: bool,
}

impl AoSourceSettingsKey {
    pub const fn version(self) -> u32 {
        self.version
    }

    pub const fn intensity(self) -> Real {
        Real::from_bits(self.intensity_bits)
    }

    pub const fn radius_meters(self) -> Real {
        Real::from_bits(self.radius_meters_bits)
    }

    pub const fn thickness_meters(self) -> Real {
        Real::from_bits(self.thickness_meters_bits)
    }

    pub const fn depth_bias_meters(self) -> Real {
        Real::from_bits(self.depth_bias_meters_bits)
    }

    pub const fn falloff_start_meters(self) -> Real {
        Real::from_bits(self.falloff_start_meters_bits)
    }

    pub const fn quality(self) -> AoQualityTier {
        match AoQualityTier::from_stable_id(self.quality) {
            Some(quality) => quality,
            None => AoQualityTier::High,
        }
    }

    pub const fn half_resolution(self) -> bool {
        self.half_resolution
    }

    pub const fn temporal(self) -> bool {
        self.temporal
    }
}

impl From<AoSourceSettings> for AoSourceSettingsKey {
    fn from(value: AoSourceSettings) -> Self {
        Self {
            version: AO_SOURCE_SETTINGS_VERSION,
            intensity_bits: value.intensity.to_bits(),
            radius_meters_bits: value.radius_meters.to_bits(),
            thickness_meters_bits: value.thickness_meters.to_bits(),
            depth_bias_meters_bits: value.depth_bias_meters.to_bits(),
            falloff_start_meters_bits: value.falloff_start_meters.to_bits(),
            quality: value.quality.stable_id(),
            half_resolution: value.half_resolution,
            temporal: value.temporal,
        }
    }
}

impl From<AoSourceSettingsKey> for AoSourceSettings {
    fn from(value: AoSourceSettingsKey) -> Self {
        Self {
            intensity: value.intensity(),
            radius_meters: value.radius_meters(),
            thickness_meters: value.thickness_meters(),
            depth_bias_meters: value.depth_bias_meters(),
            falloff_start_meters: value.falloff_start_meters(),
            quality: value.quality(),
            half_resolution: value.half_resolution(),
            temporal: value.temporal(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AoQualityTier, AoSourceSettings, AoSourceSettingsKey, AO_SOURCE_SETTINGS_VERSION};

    #[test]
    fn ao_source_settings_key_preserves_physical_units_and_discrete_modes() {
        let settings = AoSourceSettings {
            intensity: 0.75,
            radius_meters: 2.5,
            thickness_meters: 0.25,
            depth_bias_meters: 0.03,
            falloff_start_meters: 1.25,
            quality: AoQualityTier::Ultra,
            half_resolution: false,
            temporal: true,
        };

        let key = AoSourceSettingsKey::from(settings);

        assert_eq!(key.version(), AO_SOURCE_SETTINGS_VERSION);
        assert_eq!(key.intensity(), settings.intensity);
        assert_eq!(key.radius_meters(), settings.radius_meters);
        assert_eq!(key.thickness_meters(), settings.thickness_meters);
        assert_eq!(key.depth_bias_meters(), settings.depth_bias_meters);
        assert_eq!(key.falloff_start_meters(), settings.falloff_start_meters);
        assert_eq!(key.quality(), settings.quality);
        assert!(!key.half_resolution());
        assert!(key.temporal());
        assert_eq!(AoSourceSettings::from(key), settings);
    }

    #[test]
    fn ao_source_settings_default_does_not_request_unqualified_history() {
        assert!(!AoSourceSettings::default().temporal);
    }
}
