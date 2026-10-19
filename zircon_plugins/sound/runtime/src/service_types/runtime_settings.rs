//! 全局增益与默认空间比例在控制线程同步配置、源投影和现有 Kira 句柄。
use zircon_runtime::core::framework::sound::{SoundError, SoundMixBlock};

use crate::automation::values::ensure_finite_value;
use crate::poison_recovery::lock_recover;

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
        #[cfg(test)]
        crate::service_types::wait_for_owner_gain_writer_admission_attempt();
        let mut config = lock_recover(&self.config);
        #[cfg(test)]
        crate::service_types::signal_owner_gain_writer_admitted();
        let mut state = lock_recover(&self.state);
        state.kira.ensure_provider_not_retiring()?;
        state.kira.set_global_volume(gain)?;
        config.master_gain = gain;
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
        let mut config = lock_recover(&self.config);
        let mut state = lock_recover(&self.state);
        state.kira.ensure_provider_not_retiring()?;
        let previous_scale = state.default_spatial_scale;
        state.default_spatial_scale = scale;
        if let Err(error) = state.refresh_source_environments() {
            state.default_spatial_scale = previous_scale;
            return Err(error);
        }
        config.default_spatial_scale = scale;
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn source_environment_snapshot_for_test(
        &self,
        source: zircon_runtime::core::framework::sound::SoundSourceId,
    ) -> (f32, Option<crate::engine::SourceVoice>) {
        let state = lock_recover(&self.state);
        (
            state.default_spatial_scale,
            state.sources.get(&source).cloned(),
        )
    }

    pub(super) fn render_mix_impl(&self, _frames: usize) -> Result<SoundMixBlock, SoundError> {
        Err(SoundError::UnsupportedAdvancedFeature(
            "manual mix rendering was retired; Kira owns the mix graph".to_string(),
        ))
    }
}
