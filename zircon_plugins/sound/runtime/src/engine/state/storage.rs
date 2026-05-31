use std::collections::HashMap;

use zircon_runtime::core::framework::sound::{
    ExternalAudioSourceHandle, SoundAutomationBinding, SoundAutomationBindingId, SoundClipId,
    SoundDynamicEventCatalog, SoundDynamicEventHandlerDescriptor, SoundDynamicEventInvocation,
    SoundExternalSourceBlock, SoundHrtfProfileDescriptor, SoundImpulseResponseId,
    SoundListenerDescriptor, SoundListenerId, SoundMixerGraph, SoundParameterId,
    SoundPlaybackFinished, SoundPlaybackId, SoundRayTracedImpulseResponseDescriptor,
    SoundRayTracingConvolutionStatus, SoundSourceFinished, SoundSourceId, SoundTrackId,
    SoundTrackMeter, SoundVolumeDescriptor, SoundVolumeId,
};

use crate::output::SoundOutputDeviceRuntimeState;
use crate::timeline::playback::SoundTimelineSequencePlayback;
use crate::SoundConfig;

use super::super::{
    SoundEffectRuntimeState, SoundEffectStateKey, SoundHrtfRenderState, SoundHrtfRenderStateKey,
    SoundTrackRuntimeState,
};
use super::{
    ActivePlayback, LoadedClip, SoundDynamicEventExecutor, SoundDynamicEventExecutorKey,
    SourceVoice,
};

#[derive(Debug)]
pub(crate) struct SoundEngineState {
    pub(crate) next_clip_id: u64,
    pub(crate) next_playback_id: u64,
    pub(crate) next_source_id: u64,
    pub(crate) clip_ids_by_locator: HashMap<String, SoundClipId>,
    pub(crate) clips: HashMap<SoundClipId, LoadedClip>,
    pub(crate) external_sources: HashMap<ExternalAudioSourceHandle, SoundExternalSourceBlock>,
    pub(crate) playbacks: HashMap<SoundPlaybackId, ActivePlayback>,
    pub(crate) finished_playbacks: Vec<SoundPlaybackFinished>,
    pub(crate) sources: HashMap<SoundSourceId, SourceVoice>,
    pub(crate) finished_sources: Vec<SoundSourceFinished>,
    pub(crate) listeners: HashMap<SoundListenerId, SoundListenerDescriptor>,
    pub(crate) volumes: HashMap<SoundVolumeId, SoundVolumeDescriptor>,
    pub(crate) automation_bindings: HashMap<SoundAutomationBindingId, SoundAutomationBinding>,
    pub(crate) timeline_sequences: Vec<SoundTimelineSequencePlayback>,
    pub(crate) parameters: HashMap<SoundParameterId, f32>,
    pub(crate) impulse_responses: HashMap<SoundImpulseResponseId, Vec<f32>>,
    pub(crate) hrtf_profiles: HashMap<String, SoundHrtfProfileDescriptor>,
    pub(crate) hrtf_states: HashMap<SoundHrtfRenderStateKey, SoundHrtfRenderState>,
    pub(crate) ray_traced_impulse_responses:
        HashMap<SoundImpulseResponseId, SoundRayTracedImpulseResponseDescriptor>,
    pub(crate) dynamic_events: SoundDynamicEventCatalog,
    pub(crate) dynamic_event_handlers: Vec<SoundDynamicEventHandlerDescriptor>,
    pub(crate) dynamic_event_executors:
        HashMap<SoundDynamicEventExecutorKey, SoundDynamicEventExecutor>,
    pub(crate) pending_dynamic_events: Vec<SoundDynamicEventInvocation>,
    pub(crate) graph: SoundMixerGraph,
    pub(crate) effect_states: HashMap<SoundEffectStateKey, SoundEffectRuntimeState>,
    pub(crate) track_states: HashMap<SoundTrackId, SoundTrackRuntimeState>,
    pub(crate) output_device: SoundOutputDeviceRuntimeState,
    pub(crate) meters: Vec<SoundTrackMeter>,
    pub(crate) latency_frames: usize,
    pub(crate) ray_tracing: SoundRayTracingConvolutionStatus,
}

impl SoundEngineState {
    pub(crate) fn new(config: &SoundConfig) -> Self {
        let mut graph = SoundMixerGraph::default_stereo(config.sample_rate_hz);
        graph.channel_count = config.channel_count.max(1);
        Self {
            next_clip_id: 0,
            next_playback_id: 0,
            next_source_id: 0,
            clip_ids_by_locator: HashMap::new(),
            clips: HashMap::new(),
            external_sources: HashMap::new(),
            playbacks: HashMap::new(),
            finished_playbacks: Vec::new(),
            sources: HashMap::new(),
            finished_sources: Vec::new(),
            listeners: HashMap::new(),
            volumes: HashMap::new(),
            automation_bindings: HashMap::new(),
            timeline_sequences: Vec::new(),
            parameters: HashMap::new(),
            impulse_responses: HashMap::new(),
            hrtf_profiles: HashMap::new(),
            hrtf_states: HashMap::new(),
            ray_traced_impulse_responses: HashMap::new(),
            dynamic_events: SoundDynamicEventCatalog::empty(),
            dynamic_event_handlers: Vec::new(),
            dynamic_event_executors: HashMap::new(),
            pending_dynamic_events: Vec::new(),
            graph,
            effect_states: HashMap::new(),
            track_states: HashMap::new(),
            output_device: SoundOutputDeviceRuntimeState::new(config),
            meters: vec![SoundTrackMeter::silent(SoundTrackId::master())],
            latency_frames: 0,
            ray_tracing: SoundRayTracingConvolutionStatus::Disabled,
        }
    }

    pub(crate) fn next_source_id(&mut self) -> SoundSourceId {
        self.next_source_id += 1;
        SoundSourceId::new(self.next_source_id)
    }
}
