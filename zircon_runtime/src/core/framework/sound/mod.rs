//! Sound framework contracts for runtime audio graphs, scene components, and mixing.

#[path = "acoustics.rs"]
mod acoustics;
#[path = "automation.rs"]
mod automation;
#[path = "components.rs"]
mod components;
#[path = "effects.rs"]
mod effects;
#[path = "emission.rs"]
mod emission;
#[path = "error.rs"]
mod error;
#[path = "events.rs"]
mod events;
#[path = "graph.rs"]
mod graph;
#[path = "ids.rs"]
mod ids;
#[path = "manager.rs"]
mod manager;
#[path = "mix.rs"]
mod mix;
#[path = "options.rs"]
mod options;
#[path = "output.rs"]
mod output;
#[path = "playback.rs"]
mod playback;
#[path = "preset.rs"]
mod preset;
#[path = "status.rs"]
mod status;

pub use acoustics::{
    SoundHrtfProfileDescriptor, SoundRayTracedImpulseResponseDescriptor,
    SoundRayTracingConvolutionStatus,
};
pub use automation::{
    SoundAutomationBinding, SoundAutomationCurve, SoundAutomationInterpolation,
    SoundAutomationKeyframe, SoundAutomationTarget, SoundTimelineAutomationSample,
    SoundTimelineAutomationTrack, SoundTimelineSequence, SoundTimelineSequenceAdvance,
};
pub use components::{
    SoundAttenuationMode, SoundExternalSourceBlock, SoundListenerDescriptor, SoundSourceDescriptor,
    SoundSourceInput, SoundSourceParameterBinding, SoundSourceSend, SoundSpatialSourceSettings,
    SoundVolumeDescriptor, SoundVolumeShape, AUDIO_LISTENER_COMPONENT_TYPE,
    AUDIO_SOURCE_COMPONENT_TYPE, AUDIO_VOLUME_COMPONENT_TYPE,
};
pub use effects::{
    SoundChorusEffect, SoundCompressorEffect, SoundConvolutionReverbEffect, SoundDelayEffect,
    SoundEffectDescriptor, SoundEffectKind, SoundFilterEffect, SoundFilterMode, SoundFlangerEffect,
    SoundGainEffect, SoundLimiterEffect, SoundPanStereoEffect, SoundPhaserEffect,
    SoundReverbEffect, SoundSidechainInput, SoundWaveShaperEffect,
};
pub use emission::{
    SoundGameplayEmission, SoundGameplayEmissionRead, SoundGameplayEmitter,
    SOUND_GAMEPLAY_EMISSION_CAPACITY,
};
pub use error::SoundError;
pub use events::{
    SoundDynamicEventCatalog, SoundDynamicEventDelivery, SoundDynamicEventDescriptor,
    SoundDynamicEventExecutionReport, SoundDynamicEventExecutionStatus,
    SoundDynamicEventHandlerDescriptor, SoundDynamicEventHandlerExecution,
    SoundDynamicEventInvocation,
};
pub use graph::{
    SoundMixerGraph, SoundMixerSnapshot, SoundTrackControls, SoundTrackDescriptor, SoundTrackMeter,
    SoundTrackSend,
};
pub use ids::{
    ExternalAudioSourceHandle, SoundAutomationBindingId, SoundClipId, SoundEffectId,
    SoundImpulseResponseId, SoundListenerId, SoundNodeId, SoundOutputDeviceId, SoundParameterId,
    SoundPlaybackId, SoundSourceId, SoundTimelineSequenceId, SoundTrackId, SoundVolumeId,
};
pub use manager::{
    SoundAcousticsManager, SoundAutomationTimelineManager, SoundBackendManager,
    SoundDynamicEventManager, SoundManager, SoundMixRenderManager, SoundMixerGraphManager,
    SoundOutputDeviceManager, SoundPlaybackManager, SoundRuntimeSettingsManager,
    SoundSourceManager,
};
pub use mix::SoundMixBlock;
pub use options::{SoundConvolutionBudget, SoundPluginOptions, SoundRayTracingQuality};
pub use output::{
    SoundBackendCallbackBlock, SoundBackendCallbackReport, SoundBackendCapability,
    SoundOutputDeviceDescriptor, SoundOutputDeviceInfo, SoundOutputDeviceState,
    SoundOutputDeviceStatus, SoundOutputLatencyStatus, DEFAULT_SOUND_OUTPUT_LATENCY_BLOCKS,
};
pub use playback::{
    SoundClipInfo, SoundPlaybackCompletionAction, SoundPlaybackFinishReason, SoundPlaybackFinished,
    SoundPlaybackSettings, SoundPlaybackStatus, SoundSourceFinishReason, SoundSourceFinished,
    SoundSourceStatus,
};
pub use preset::SoundMixerPresetDescriptor;
pub use status::{SoundBackendState, SoundBackendStatus};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
