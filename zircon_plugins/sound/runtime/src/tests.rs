use zircon_runtime::core::framework::audio::{AudioChannelLayout, AudioSpeakerChannel};
use zircon_runtime::core::framework::sound::{
    ExternalAudioSourceHandle, SoundAcousticsManager, SoundAttenuationMode, SoundAutomationBinding,
    SoundAutomationBindingId, SoundAutomationCurve, SoundAutomationKeyframe, SoundAutomationTarget,
    SoundAutomationTimelineManager, SoundBackendManager, SoundBackendState, SoundChorusEffect,
    SoundClipId, SoundCompressorEffect, SoundConvolutionBudget, SoundConvolutionReverbEffect,
    SoundDelayEffect, SoundDynamicEventDescriptor, SoundDynamicEventExecutionStatus,
    SoundDynamicEventHandlerDescriptor, SoundDynamicEventInvocation, SoundDynamicEventManager,
    SoundEffectDescriptor, SoundEffectId, SoundEffectKind, SoundError, SoundExternalSourceBlock,
    SoundFilterEffect, SoundFilterMode, SoundFlangerEffect, SoundGainEffect,
    SoundHrtfProfileDescriptor, SoundImpulseResponseId, SoundLimiterEffect, SoundListenerId,
    SoundMixRenderManager, SoundMixerGraph, SoundMixerGraphManager, SoundOutputDeviceDescriptor,
    SoundOutputDeviceId, SoundOutputDeviceManager, SoundOutputDeviceState, SoundPanStereoEffect,
    SoundParameterId, SoundPhaserEffect, SoundPlaybackCompletionAction, SoundPlaybackFinishReason,
    SoundPlaybackManager, SoundPlaybackSettings, SoundPluginOptions,
    SoundRayTracedImpulseResponseDescriptor, SoundRayTracingConvolutionStatus,
    SoundRayTracingQuality, SoundReverbEffect, SoundRuntimeSettingsManager, SoundSidechainInput,
    SoundSourceDescriptor, SoundSourceFinishReason, SoundSourceId, SoundSourceInput,
    SoundSourceManager, SoundSourceParameterBinding, SoundSourceSend, SoundSpatialSourceSettings,
    SoundTimelineAutomationTrack, SoundTimelineSequence, SoundTimelineSequenceId,
    SoundTrackDescriptor, SoundTrackId, SoundTrackSend, SoundVolumeDescriptor, SoundVolumeId,
    SoundVolumeShape, SoundWaveShaperEffect, AUDIO_LISTENER_COMPONENT_TYPE,
    AUDIO_SOURCE_COMPONENT_TYPE, AUDIO_VOLUME_COMPONENT_TYPE,
};
use zircon_runtime::plugin::RuntimePluginRegistrationReport;

use super::{
    package_manifest, runtime_plugin, DefaultSoundManager, SoundConfig, RUNTIME_CAPABILITIES,
    SOUND_DIST_CRATE_NAME, SOUND_DIST_RUNTIME_ENTRY, SOUND_DYNAMIC_EVENT_NAMESPACE,
    SOUND_MODULE_NAME,
};

mod automation_binding;
mod automation_curve;
mod convolution;
mod dynamic_events;
mod graph_config;
mod kira_bridge;
mod kira_graph_sync;
mod manifest;
mod mixer_graph;
mod optional_feature_manifest;
mod output_device;
mod playback;
mod poison_recovery;
mod presets;
mod ray_tracing;
mod runtime_core;
mod source_inputs;
mod spatial;
mod support;

use support::*;
