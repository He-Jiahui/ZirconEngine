use serde::{Deserialize, Serialize};

use crate::core::framework::audio::AudioChannelLayout;

use super::{SoundSourceInput, SoundTrackId};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundClipInfo {
    pub locator: String,
    pub sample_rate_hz: u32,
    pub channel_count: u16,
    pub channel_layout: AudioChannelLayout,
    pub frame_count: usize,
    pub duration_seconds: f32,
}

/// 一次性 clip 播放的请求设置；时间范围和输出轨道在运行时接纳时验证。
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundPlaybackSettings {
    pub gain: f32,
    pub speed: f32,
    pub looped: bool,
    pub completion_action: SoundPlaybackCompletionAction,
    pub paused: bool,
    pub muted: bool,
    pub start_seconds: Option<f32>,
    pub duration_seconds: Option<f32>,
    pub output_track: SoundTrackId,
    pub pan: f32,
}

impl Default for SoundPlaybackSettings {
    fn default() -> Self {
        Self::ONCE
    }
}

impl SoundPlaybackSettings {
    pub const ONCE: Self = Self {
        gain: 1.0,
        speed: 1.0,
        looped: false,
        completion_action: SoundPlaybackCompletionAction::None,
        paused: false,
        muted: false,
        start_seconds: None,
        duration_seconds: None,
        output_track: SoundTrackId::MASTER,
        pan: 0.0,
    };

    pub const LOOP: Self = Self {
        looped: true,
        ..Self::ONCE
    };

    pub const DESPAWN: Self = Self {
        completion_action: SoundPlaybackCompletionAction::DespawnEntity,
        ..Self::ONCE
    };

    pub const REMOVE: Self = Self {
        completion_action: SoundPlaybackCompletionAction::RemoveAudioComponents,
        ..Self::ONCE
    };

    pub const fn paused(mut self) -> Self {
        self.paused = true;
        self
    }

    pub const fn muted(mut self) -> Self {
        self.muted = true;
        self
    }

    pub const fn with_gain(mut self, gain: f32) -> Self {
        self.gain = gain;
        self
    }

    pub const fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    pub const fn with_output_track(mut self, output_track: SoundTrackId) -> Self {
        self.output_track = output_track;
        self
    }

    pub const fn with_pan(mut self, pan: f32) -> Self {
        self.pan = pan;
        self
    }

    pub const fn with_start_seconds(mut self, start_seconds: f32) -> Self {
        self.start_seconds = Some(start_seconds);
        self
    }

    pub const fn with_duration_seconds(mut self, duration_seconds: f32) -> Self {
        self.duration_seconds = Some(duration_seconds);
        self
    }

    pub const fn with_completion_action(
        mut self,
        completion_action: SoundPlaybackCompletionAction,
    ) -> Self {
        self.completion_action = completion_action;
        self
    }

    pub const fn with_looped(mut self, looped: bool) -> Self {
        self.looped = looped;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundPlaybackStatus {
    pub playback: super::SoundPlaybackId,
    pub clip: super::SoundClipId,
    pub paused: bool,
    pub muted: bool,
    pub looped: bool,
    pub completion_action: SoundPlaybackCompletionAction,
    pub gain: f32,
    pub speed: f32,
    pub range_start_frame: usize,
    pub range_end_frame: Option<usize>,
    pub cursor_frame: usize,
    pub cursor_seconds: f32,
    pub output_track: SoundTrackId,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundSourceStatus {
    pub source: super::SoundSourceId,
    pub input: SoundSourceInput,
    pub playing: bool,
    pub muted: bool,
    pub looped: bool,
    pub completion_action: SoundPlaybackCompletionAction,
    pub gain: f32,
    pub speed: f32,
    pub range_start_frame: usize,
    pub range_end_frame: Option<usize>,
    pub cursor_frame: usize,
    pub cursor_seconds: f32,
    pub output_track: SoundTrackId,
}

// TODO: [CR-FRAMEWORK-SOUND-0001] 明确谁消费完成动作并执行 ECS 变更；当前声音插件仅把动作写入 finished 事件。
/// 完成事件携带的后续实体处理意图；声音插件只报告该值，不直接变更场景实体。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoundPlaybackCompletionAction {
    None,
    DespawnEntity,
    RemoveAudioComponents,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoundPlaybackFinishReason {
    Completed,
    Stopped,
    MissingClip,
}

/// 一次性播放的终止报告；上层通过 drain_finished_playbacks 取走，按 reason 与 completion_action 收尾。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoundPlaybackFinished {
    pub playback: super::SoundPlaybackId,
    pub clip: super::SoundClipId,
    pub reason: SoundPlaybackFinishReason,
    pub completion_action: SoundPlaybackCompletionAction,
    pub output_track: SoundTrackId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SoundSourceFinishReason {
    Completed,
    MissingClip,
    Stopped,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundSourceFinished {
    pub source: super::SoundSourceId,
    pub input: SoundSourceInput,
    pub clip: Option<super::SoundClipId>,
    pub reason: SoundSourceFinishReason,
    pub completion_action: SoundPlaybackCompletionAction,
    pub output_track: SoundTrackId,
}
