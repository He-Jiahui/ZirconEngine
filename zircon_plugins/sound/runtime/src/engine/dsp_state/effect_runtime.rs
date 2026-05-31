use super::{SoundDelayLineState, SoundHistoryState};
use crate::engine::filter::SoundBiquadFilterState;

#[derive(Clone, Debug)]
pub(crate) struct SoundEffectRuntimeState {
    pub(crate) delay_line: SoundDelayLineState,
    pub(crate) reverb_history: SoundHistoryState,
    pub(crate) convolution_history: SoundHistoryState,
    pub(crate) modulation_history: SoundHistoryState,
    pub(crate) filter_state: SoundBiquadFilterState,
    pub(crate) modulated_delay_phase: f32,
    pub(crate) phaser_phase: f32,
    pub(crate) compressor_gain: f32,
}

impl Default for SoundEffectRuntimeState {
    fn default() -> Self {
        Self {
            delay_line: SoundDelayLineState::default(),
            reverb_history: SoundHistoryState::default(),
            convolution_history: SoundHistoryState::default(),
            modulation_history: SoundHistoryState::default(),
            filter_state: SoundBiquadFilterState::default(),
            modulated_delay_phase: 0.0,
            phaser_phase: 0.0,
            compressor_gain: 1.0,
        }
    }
}
