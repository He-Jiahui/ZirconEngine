use zircon_runtime::core::framework::sound::{SoundError, SoundMixBlock};

use crate::automation::values::ensure_finite_value;

use super::DefaultSoundManager;

impl DefaultSoundManager {
    pub(super) fn global_volume_gain_impl(&self) -> Result<f32, SoundError> {
        Ok(self.config().master_gain)
    }

    pub(super) fn set_global_volume_gain_impl(&self, gain: f32) -> Result<(), SoundError> {
        ensure_finite_value("global volume gain", gain)?;
        if gain < 0.0 {
            return Err(SoundError::InvalidParameter(
                "global volume gain must be non-negative".to_string(),
            ));
        }
        self.config
            .lock()
            .expect("sound config mutex poisoned")
            .master_gain = gain;
        Ok(())
    }

    pub(super) fn default_spatial_scale_impl(&self) -> Result<f32, SoundError> {
        Ok(self.config().default_spatial_scale)
    }

    pub(super) fn set_default_spatial_scale_impl(&self, scale: f32) -> Result<(), SoundError> {
        ensure_finite_value("default spatial scale", scale)?;
        if scale < 0.0 {
            return Err(SoundError::InvalidParameter(
                "default spatial scale must be non-negative".to_string(),
            ));
        }
        self.config
            .lock()
            .expect("sound config mutex poisoned")
            .default_spatial_scale = scale;
        Ok(())
    }

    pub(super) fn render_mix_impl(&self, frames: usize) -> Result<SoundMixBlock, SoundError> {
        let config = self.config();
        if !config.enabled {
            return Err(SoundError::BackendUnavailable {
                detail: "sound playback is disabled".to_string(),
            });
        }

        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .render_mix(&config, frames)
    }
}
