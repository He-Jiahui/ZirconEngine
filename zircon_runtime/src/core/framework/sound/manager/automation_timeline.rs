use super::super::{
    SoundAutomationBinding, SoundAutomationBindingId, SoundAutomationCurve, SoundError,
    SoundParameterId, SoundTimelineSequence, SoundTimelineSequenceAdvance, SoundTimelineSequenceId,
};

pub trait SoundAutomationTimelineManager {
    fn set_parameter(&self, parameter: SoundParameterId, value: f32) -> Result<(), SoundError>;
    fn parameter_value(&self, parameter: &SoundParameterId) -> Result<f32, SoundError>;
    fn bind_automation(&self, binding: SoundAutomationBinding) -> Result<(), SoundError>;
    fn apply_automation_value(
        &self,
        binding: SoundAutomationBindingId,
        value: f32,
    ) -> Result<(), SoundError>;
    fn apply_automation_curve_sample(
        &self,
        binding: SoundAutomationBindingId,
        curve: &SoundAutomationCurve,
        time_seconds: f32,
    ) -> Result<f32, SoundError>;
    fn unbind_automation(&self, binding: SoundAutomationBindingId) -> Result<(), SoundError>;
    fn schedule_timeline_sequence(&self, sequence: SoundTimelineSequence)
        -> Result<(), SoundError>;
    fn remove_timeline_sequence(
        &self,
        sequence: &SoundTimelineSequenceId,
    ) -> Result<(), SoundError>;
    fn timeline_sequences(&self) -> Result<Vec<SoundTimelineSequence>, SoundError>;
    fn advance_timeline_sequences(
        &self,
        delta_seconds: f32,
    ) -> Result<Vec<SoundTimelineSequenceAdvance>, SoundError>;
}
