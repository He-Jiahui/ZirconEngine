//! Sound framework contracts for runtime audio graphs, scene components, and mixing.

mod acoustics;
mod automation;
mod components;
mod effects;
mod error;
mod events;
mod graph;
mod ids;
mod manager;
mod mix;
mod options;
mod output;
mod playback;
mod preset;
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
pub use error::SoundError;
pub use events::{
    SoundDynamicEventCatalog, SoundDynamicEventDelivery, SoundDynamicEventDescriptor,
    SoundDynamicEventHandlerDescriptor, SoundDynamicEventInvocation,
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
pub use manager::SoundManager;
pub use mix::SoundMixBlock;
pub use options::{SoundConvolutionBudget, SoundPluginOptions, SoundRayTracingQuality};
pub use output::{
    SoundBackendCallbackBlock, SoundBackendCallbackReport, SoundBackendCapability,
    SoundOutputDeviceDescriptor, SoundOutputDeviceState, SoundOutputDeviceStatus,
};
pub use playback::{SoundClipInfo, SoundPlaybackSettings};
pub use preset::SoundMixerPresetDescriptor;
pub use status::{SoundBackendState, SoundBackendStatus};
