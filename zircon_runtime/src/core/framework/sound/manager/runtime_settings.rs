use super::super::SoundError;

pub trait SoundRuntimeSettingsManager {
    fn global_volume_gain(&self) -> Result<f32, SoundError>;
    fn set_global_volume_gain(&self, gain: f32) -> Result<(), SoundError>;
    fn default_spatial_scale(&self) -> Result<f32, SoundError>;
    fn set_default_spatial_scale(&self, scale: f32) -> Result<(), SoundError>;
}
