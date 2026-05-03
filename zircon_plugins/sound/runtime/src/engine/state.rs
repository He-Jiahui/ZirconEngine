use std::collections::HashMap;

use zircon_runtime::asset::SoundAsset;
use zircon_runtime::core::framework::sound::{
    ExternalAudioSourceHandle, SoundAutomationBinding, SoundAutomationBindingId, SoundClipId,
    SoundDynamicEventCatalog, SoundDynamicEventHandlerDescriptor, SoundDynamicEventInvocation,
    SoundError, SoundExternalSourceBlock, SoundHrtfProfileDescriptor, SoundImpulseResponseId,
    SoundListenerDescriptor, SoundListenerId, SoundMixerGraph, SoundMixerSnapshot,
    SoundParameterId, SoundPlaybackId, SoundRayTracedImpulseResponseDescriptor,
    SoundRayTracingConvolutionStatus, SoundSourceDescriptor, SoundSourceId, SoundTrackDescriptor,
    SoundTrackId, SoundTrackMeter, SoundVolumeDescriptor, SoundVolumeId,
};

use crate::output::SoundOutputDeviceRuntimeState;
use crate::timeline::SoundTimelineSequencePlayback;
use crate::SoundConfig;

use super::{SoundEffectRuntimeState, SoundEffectStateKey, SoundTrackRuntimeState};

#[derive(Debug)]
pub(crate) struct SoundEngineState {
    pub(crate) next_clip_id: u64,
    pub(crate) next_playback_id: u64,
    pub(crate) next_source_id: u64,
    pub(crate) clip_ids_by_locator: HashMap<String, SoundClipId>,
    pub(crate) clips: HashMap<SoundClipId, LoadedClip>,
    pub(crate) external_sources: HashMap<ExternalAudioSourceHandle, SoundExternalSourceBlock>,
    pub(crate) playbacks: HashMap<SoundPlaybackId, ActivePlayback>,
    pub(crate) sources: HashMap<SoundSourceId, SourceVoice>,
    pub(crate) listeners: HashMap<SoundListenerId, SoundListenerDescriptor>,
    pub(crate) volumes: HashMap<SoundVolumeId, SoundVolumeDescriptor>,
    pub(crate) automation_bindings: HashMap<SoundAutomationBindingId, SoundAutomationBinding>,
    pub(crate) timeline_sequences: Vec<SoundTimelineSequencePlayback>,
    pub(crate) parameters: HashMap<SoundParameterId, f32>,
    pub(crate) impulse_responses: HashMap<SoundImpulseResponseId, Vec<f32>>,
    pub(crate) hrtf_profiles: HashMap<String, SoundHrtfProfileDescriptor>,
    pub(crate) ray_traced_impulse_responses:
        HashMap<SoundImpulseResponseId, SoundRayTracedImpulseResponseDescriptor>,
    pub(crate) dynamic_events: SoundDynamicEventCatalog,
    pub(crate) dynamic_event_handlers: Vec<SoundDynamicEventHandlerDescriptor>,
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
            sources: HashMap::new(),
            listeners: HashMap::new(),
            volumes: HashMap::new(),
            automation_bindings: HashMap::new(),
            timeline_sequences: Vec::new(),
            parameters: HashMap::new(),
            impulse_responses: HashMap::new(),
            hrtf_profiles: HashMap::new(),
            ray_traced_impulse_responses: HashMap::new(),
            dynamic_events: SoundDynamicEventCatalog::empty(),
            dynamic_event_handlers: Vec::new(),
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

    pub(crate) fn snapshot(&self) -> SoundMixerSnapshot {
        let mut graph = self.graph.clone();
        graph.sources = self
            .sources
            .values()
            .map(|source| source.descriptor.clone())
            .collect();
        graph.automation_bindings = self.automation_bindings.values().cloned().collect();
        graph.dynamic_events = self.dynamic_events.clone();
        SoundMixerSnapshot {
            graph,
            meters: self.meters.clone(),
            latency_frames: self.latency_frames,
            ray_tracing: self.ray_tracing.clone(),
        }
    }

    pub(crate) fn next_source_id(&mut self) -> SoundSourceId {
        self.next_source_id += 1;
        SoundSourceId::new(self.next_source_id)
    }

    pub(crate) fn add_or_replace_track(
        &mut self,
        track: SoundTrackDescriptor,
    ) -> Result<(), SoundError> {
        if track.id == SoundTrackId::master() && track.parent.is_some() {
            return Err(SoundError::InvalidMixerGraph(
                "master track cannot have a parent track".to_string(),
            ));
        }
        let mut graph = self.graph.clone();
        if let Some(existing) = graph
            .tracks
            .iter_mut()
            .find(|existing| existing.id == track.id)
        {
            *existing = track;
        } else {
            graph.tracks.push(track);
        }
        super::validation::validate_graph(&graph)?;
        self.graph = graph;
        Ok(())
    }

    pub(crate) fn remove_track(&mut self, track: SoundTrackId) -> Result<(), SoundError> {
        if track == SoundTrackId::master() {
            return Err(SoundError::InvalidMixerGraph(
                "master track cannot be removed".to_string(),
            ));
        }
        let mut graph = self.graph.clone();
        let before = graph.tracks.len();
        graph.tracks.retain(|existing| existing.id != track);
        if before == graph.tracks.len() {
            return Err(SoundError::UnknownTrack { track });
        }
        super::validation::validate_graph(&graph)?;
        self.graph = graph;
        for source in self.sources.values_mut() {
            if source.descriptor.output_track == track {
                source.descriptor.output_track = SoundTrackId::master();
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub(crate) struct LoadedClip {
    pub(crate) asset: SoundAsset,
}

#[derive(Clone, Debug)]
pub(crate) struct ActivePlayback {
    pub(crate) clip: SoundClipId,
    pub(crate) cursor_frame: usize,
    pub(crate) cursor_position: f64,
    pub(crate) gain: f32,
    pub(crate) looped: bool,
    pub(crate) output_track: SoundTrackId,
    pub(crate) pan: f32,
}

#[derive(Clone, Debug)]
pub(crate) struct SourceVoice {
    pub(crate) descriptor: SoundSourceDescriptor,
    pub(crate) cursor_frame: usize,
    pub(crate) cursor_position: f64,
}
