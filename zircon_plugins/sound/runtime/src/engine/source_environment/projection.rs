//! 控制线程上的源级空间增益和声像投影；不改PCM/HRTF处理和用户保存的增益。
use std::collections::HashMap;

use zircon_runtime::core::framework::sound::{
    SoundError, SoundImpulseResponseId, SoundListenerDescriptor,
    SoundRayTracedImpulseResponseDescriptor, SoundSourceId,
};

use super::{active_listener_for, spatial};
use crate::engine::{SoundEngineState, SourceVoice};

impl SourceVoice {
    /// Cache only the inexpensive spatial controls; PCM/HRTF processing remains separate.
    pub(crate) fn project_environment(
        &mut self,
        default_scale: f32,
        listeners: &[SoundListenerDescriptor],
        ray_traced: &HashMap<SoundImpulseResponseId, SoundRayTracedImpulseResponseDescriptor>,
    ) -> Result<(), SoundError> {
        let (gain, pan) =
            if let Some(listener) = active_listener_for(listeners, self.descriptor.output_track) {
                let scale = self
                    .descriptor
                    .spatial
                    .spatial_scale
                    .unwrap_or(default_scale)
                    .max(0.0);
                let profile = spatial::spatial_profile(
                    self.descriptor.id.unwrap_or(SoundSourceId::new(0)),
                    &self.descriptor,
                    listener,
                    scale,
                    None,
                    ray_traced,
                );
                // A coincident source (or zero scale) has no lateral direction. Keep the
                // shared gain but do not expose normalize3's fallback direction as pan.
                let pan = if scale == 0.0 || self.descriptor.position == listener.position {
                    0.0
                } else {
                    profile.pan
                };
                (profile.gain, pan)
            } else {
                (1.0, 0.0)
            };
        crate::automation::values::ensure_finite_value("source environment gain", gain)?;
        crate::automation::values::ensure_finite_value("source environment pan", pan)?;
        self.environment_gain = gain;
        self.environment_pan = pan;
        Ok(())
    }
}

impl SoundEngineState {
    pub(crate) fn refresh_source_environments(&mut self) -> Result<(), SoundError> {
        self.kira.ensure_provider_not_retiring()?;
        let listeners = self.listeners.values().cloned().collect::<Vec<_>>();
        // Prepare every projection before publishing any cache or backend command.
        let mut projected = self.sources.clone();
        for voice in projected.values_mut() {
            voice.project_environment(
                self.default_spatial_scale,
                &listeners,
                &self.ray_traced_impulse_responses,
            )?;
            voice.environment_output_gain(voice.descriptor.gain, voice.descriptor.muted)?;
            if voice
                .kira_playback
                .is_some_and(|id| self.kira.contains_playback(id))
            {
                self.kira.ensure_control_available()?;
            }
        }
        // The state lock keeps the prechecked handle registry/control state unchanged.
        // Kira's native volume/pan command writers themselves have no fallible result.
        for voice in projected.values() {
            voice.apply_environment(&mut self.kira)?;
        }
        self.sources = projected;
        Ok(())
    }
}
