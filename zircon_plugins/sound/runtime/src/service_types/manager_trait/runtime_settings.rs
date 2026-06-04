use zircon_runtime::core::framework::sound::{SoundError, SoundRuntimeSettingsManager};

use super::super::DefaultSoundManager;

impl SoundRuntimeSettingsManager for DefaultSoundManager {
    fn global_volume_gain(&self) -> Result<f32, SoundError> {
        self.global_volume_gain_impl()
    }

    fn set_global_volume_gain(&self, gain: f32) -> Result<(), SoundError> {
        self.set_global_volume_gain_impl(gain)
    }

    fn default_spatial_scale(&self) -> Result<f32, SoundError> {
        self.default_spatial_scale_impl()
    }

    fn set_default_spatial_scale(&self, scale: f32) -> Result<(), SoundError> {
        self.set_default_spatial_scale_impl(scale)
    }
}
