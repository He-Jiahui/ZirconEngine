use zircon_runtime::core::framework::sound::{
    SoundAutomationBinding, SoundAutomationBindingId, SoundAutomationCurve,
    SoundAutomationTimelineManager, SoundError, SoundParameterId, SoundTimelineSequence,
    SoundTimelineSequenceAdvance, SoundTimelineSequenceId,
};

use super::super::DefaultSoundManager;

impl SoundAutomationTimelineManager for DefaultSoundManager {
    fn set_parameter(&self, parameter: SoundParameterId, value: f32) -> Result<(), SoundError> {
        self.set_parameter_impl(parameter, value)
    }

    fn parameter_value(&self, parameter: &SoundParameterId) -> Result<f32, SoundError> {
        self.parameter_value_impl(parameter)
    }

    fn bind_automation(&self, binding: SoundAutomationBinding) -> Result<(), SoundError> {
        self.bind_automation_impl(binding)
    }

    fn apply_automation_value(
        &self,
        binding: SoundAutomationBindingId,
        value: f32,
    ) -> Result<(), SoundError> {
        self.apply_automation_value_impl(binding, value)
    }

    fn apply_automation_curve_sample(
        &self,
        binding: SoundAutomationBindingId,
        curve: &SoundAutomationCurve,
        time_seconds: f32,
    ) -> Result<f32, SoundError> {
        self.apply_automation_curve_sample_impl(binding, curve, time_seconds)
    }

    fn unbind_automation(&self, binding: SoundAutomationBindingId) -> Result<(), SoundError> {
        self.unbind_automation_impl(binding)
    }

    fn schedule_timeline_sequence(
        &self,
        sequence: SoundTimelineSequence,
    ) -> Result<(), SoundError> {
        self.schedule_timeline_sequence_impl(sequence)
    }

    fn remove_timeline_sequence(
        &self,
        sequence: &SoundTimelineSequenceId,
    ) -> Result<(), SoundError> {
        self.remove_timeline_sequence_impl(sequence)
    }

    fn timeline_sequences(&self) -> Result<Vec<SoundTimelineSequence>, SoundError> {
        self.timeline_sequences_impl()
    }

    fn advance_timeline_sequences(
        &self,
        delta_seconds: f32,
    ) -> Result<Vec<SoundTimelineSequenceAdvance>, SoundError> {
        self.advance_timeline_sequences_impl(delta_seconds)
    }
}
