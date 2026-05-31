use zircon_runtime::core::framework::sound::{
    ExternalAudioSourceHandle, SoundAttenuationMode, SoundAutomationBinding,
    SoundAutomationBindingId, SoundAutomationCurve, SoundAutomationKeyframe, SoundAutomationTarget,
    SoundBackendState, SoundChorusEffect, SoundClipId, SoundCompressorEffect,
    SoundConvolutionBudget, SoundConvolutionReverbEffect, SoundDelayEffect,
    SoundDynamicEventDescriptor, SoundDynamicEventExecutionStatus,
    SoundDynamicEventHandlerDescriptor, SoundDynamicEventInvocation, SoundEffectDescriptor,
    SoundEffectId, SoundEffectKind, SoundError, SoundExternalSourceBlock, SoundFilterEffect,
    SoundFilterMode, SoundFlangerEffect, SoundGainEffect, SoundHrtfProfileDescriptor,
    SoundImpulseResponseId, SoundLimiterEffect, SoundListenerId, SoundManager, SoundMixerGraph,
    SoundOutputDeviceDescriptor, SoundOutputDeviceId, SoundOutputDeviceState, SoundPanStereoEffect,
    SoundParameterId, SoundPhaserEffect, SoundPlaybackCompletionAction, SoundPlaybackFinishReason,
    SoundPlaybackSettings, SoundPluginOptions, SoundRayTracedImpulseResponseDescriptor,
    SoundRayTracingConvolutionStatus, SoundRayTracingQuality, SoundReverbEffect,
    SoundSidechainInput, SoundSourceDescriptor, SoundSourceFinishReason, SoundSourceId,
    SoundSourceInput, SoundSourceParameterBinding, SoundSourceSend, SoundSpatialSourceSettings,
    SoundTimelineAutomationTrack, SoundTimelineSequence, SoundTimelineSequenceId,
    SoundTrackDescriptor, SoundTrackId, SoundTrackSend, SoundVolumeDescriptor, SoundVolumeId,
    SoundVolumeShape, SoundWaveShaperEffect, AUDIO_LISTENER_COMPONENT_TYPE,
    AUDIO_SOURCE_COMPONENT_TYPE, AUDIO_VOLUME_COMPONENT_TYPE,
};
use zircon_runtime::plugin::RuntimePluginRegistrationReport;

use super::{
    runtime_plugin, DefaultSoundManager, SoundConfig, SOUND_DYNAMIC_EVENT_NAMESPACE,
    SOUND_MODULE_NAME,
};

mod automation_binding;
mod automation_curve;
mod common;
mod convolution;
mod dsp_state;
mod dynamic_events;
mod graph_config;
mod manifest;
mod mixer_graph;
mod optional_feature_manifest;
mod output_device;
mod playback;
mod presets;
mod ray_tracing;
mod runtime_core;
mod source_inputs;
mod spatial;

use common::*;
