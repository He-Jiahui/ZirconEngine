use std::collections::HashMap;

use zircon_runtime::core::framework::sound::{
    ExternalAudioSourceHandle, SoundClipId, SoundExternalSourceBlock, SoundHrtfProfileDescriptor,
    SoundImpulseResponseId, SoundListenerDescriptor, SoundParameterId,
    SoundRayTracedImpulseResponseDescriptor, SoundRayTracingConvolutionStatus,
    SoundSourceDescriptor, SoundTrackId, SoundVolumeDescriptor,
};

use crate::engine::source_environment::active_listener_for;
use crate::engine::state::{LoadedClip, SoundEngineState};

pub(super) struct SourceRenderSnapshot {
    pub(super) clips: HashMap<SoundClipId, LoadedClip>,
    pub(super) external_sources: HashMap<ExternalAudioSourceHandle, SoundExternalSourceBlock>,
    pub(super) parameters: HashMap<SoundParameterId, f32>,
    pub(super) listeners: Vec<SoundListenerDescriptor>,
    pub(super) volumes: Vec<SoundVolumeDescriptor>,
    pub(super) impulse_responses: HashMap<SoundImpulseResponseId, Vec<f32>>,
    pub(super) ray_traced_impulse_responses:
        HashMap<SoundImpulseResponseId, SoundRayTracedImpulseResponseDescriptor>,
    pub(super) hrtf_profiles: HashMap<String, SoundHrtfProfileDescriptor>,
    pub(super) ray_tracing: SoundRayTracingConvolutionStatus,
}

impl SourceRenderSnapshot {
    pub(super) fn new(state: &SoundEngineState) -> Self {
        Self {
            clips: state.clips.clone(),
            external_sources: state.external_sources.clone(),
            parameters: state.parameters.clone(),
            listeners: state.listeners.values().cloned().collect(),
            volumes: state.volumes.values().cloned().collect(),
            impulse_responses: state.impulse_responses.clone(),
            ray_traced_impulse_responses: state.ray_traced_impulse_responses.clone(),
            hrtf_profiles: state.hrtf_profiles.clone(),
            ray_tracing: state.ray_tracing.clone(),
        }
    }

    pub(super) fn active_listener(
        &self,
        output_track: SoundTrackId,
    ) -> Option<&SoundListenerDescriptor> {
        active_listener_for(&self.listeners, output_track)
    }

    pub(super) fn output_track_for(
        &self,
        descriptor: &SoundSourceDescriptor,
        track_buffers: &HashMap<SoundTrackId, Vec<f32>>,
    ) -> SoundTrackId {
        if track_buffers.contains_key(&descriptor.output_track) {
            descriptor.output_track
        } else {
            SoundTrackId::master()
        }
    }
}
