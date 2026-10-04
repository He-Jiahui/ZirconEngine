use super::super::{
    SoundEffectDescriptor, SoundEffectId, SoundError, SoundMixerGraph, SoundMixerPresetDescriptor,
    SoundMixerSnapshot, SoundTrackDescriptor, SoundTrackId, SoundTrackSend,
};

/// 混音拓扑的配置入口；完整图与局部轨道变更由插件验证后发布到活动音频后端。
pub trait SoundMixerGraphManager {
    fn available_mixer_presets(&self) -> Result<Vec<SoundMixerPresetDescriptor>, SoundError>;
    fn apply_mixer_preset(&self, locator: &str) -> Result<(), SoundError>;
    fn configure_mixer(&self, graph: SoundMixerGraph) -> Result<(), SoundError>;
    fn mixer_snapshot(&self) -> Result<SoundMixerSnapshot, SoundError>;
    fn add_or_update_track(&self, track: SoundTrackDescriptor) -> Result<(), SoundError>;
    fn remove_track(&self, track: SoundTrackId) -> Result<(), SoundError>;
    fn add_or_update_track_send(
        &self,
        track: SoundTrackId,
        send: SoundTrackSend,
    ) -> Result<(), SoundError>;
    fn remove_track_send(
        &self,
        track: SoundTrackId,
        target: SoundTrackId,
    ) -> Result<(), SoundError>;
    fn add_or_update_effect(
        &self,
        track: SoundTrackId,
        effect: SoundEffectDescriptor,
    ) -> Result<(), SoundError>;
    fn remove_effect(&self, track: SoundTrackId, effect: SoundEffectId) -> Result<(), SoundError>;
}
