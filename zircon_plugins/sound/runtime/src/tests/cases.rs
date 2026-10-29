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

#[path = "automation_binding.rs"]
mod automation_binding;
#[path = "automation_curve.rs"]
mod automation_curve;
#[path = "convolution.rs"]
mod convolution;
#[path = "dynamic_events/mod.rs"]
mod dynamic_events;
#[path = "graph_config.rs"]
mod graph_config;
#[path = "kira_bridge.rs"]
mod kira_bridge;
#[path = "kira_graph_sync.rs"]
mod kira_graph_sync;
#[path = "manifest.rs"]
mod manifest;
#[path = "mixer_graph.rs"]
mod mixer_graph;
#[path = "optional_feature_manifest.rs"]
mod optional_feature_manifest;
#[path = "output_device.rs"]
mod output_device;
#[path = "playback.rs"]
mod playback;
#[path = "poison_recovery.rs"]
mod poison_recovery;
#[path = "presets.rs"]
mod presets;
#[path = "ray_tracing.rs"]
mod ray_tracing;
#[path = "runtime_core.rs"]
mod runtime_core;
#[path = "scene_support.rs"]
mod scene_support;
#[path = "scene_system.rs"]
mod scene_system;
#[path = "source_inputs.rs"]
mod source_inputs;
#[path = "spatial.rs"]
mod spatial;
#[path = "support.rs"]
mod support;

use support::*;
