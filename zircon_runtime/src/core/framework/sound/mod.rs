//! Sound framework contracts for runtime audio graphs, scene components, and mixing.

mod automation;
mod components;
mod effects;
mod error;
mod graph;
mod ids;
mod manager;
mod mix;
mod options;
mod playback;
mod status;

pub use automation::{
    SoundAutomationBinding, SoundAutomationTarget, SoundDynamicEventCatalog,
    SoundDynamicEventDescriptor,
};
pub use components::{
    SoundAttenuationMode, SoundListenerDescriptor, SoundSourceDescriptor, SoundSourceInput,
    SoundSourceParameterBinding, SoundSourceSend, SoundSpatialSourceSettings,
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
pub use graph::{
    SoundMixerGraph, SoundMixerSnapshot, SoundRayTracingConvolutionStatus, SoundTrackControls,
    SoundTrackDescriptor, SoundTrackMeter, SoundTrackSend,
};
pub use ids::{
    ExternalAudioSourceHandle, SoundAutomationBindingId, SoundClipId, SoundEffectId,
    SoundImpulseResponseId, SoundListenerId, SoundNodeId, SoundParameterId, SoundPlaybackId,
    SoundSourceId, SoundTrackId, SoundVolumeId,
};
pub use manager::SoundManager;
pub use mix::SoundMixBlock;
pub use options::{SoundConvolutionBudget, SoundPluginOptions, SoundRayTracingQuality};
pub use playback::{SoundClipInfo, SoundPlaybackSettings};
pub use status::{SoundBackendState, SoundBackendStatus};
