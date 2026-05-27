use zircon_runtime::core::framework::sound::{
    SoundAutomationBinding, SoundAutomationBindingId, SoundAutomationCurve, SoundError,
    SoundParameterId, SoundTimelineSequence, SoundTimelineSequenceAdvance, SoundTimelineSequenceId,
};

use crate::automation::{
    apply_automation_target, ensure_finite_value, sample_automation_curve,
    validate_automation_binding,
};
use crate::timeline::{
    advance_timeline_sequences, remove_timeline_sequence, schedule_timeline_sequence,
};

use super::DefaultSoundManager;

impl DefaultSoundManager {
    pub(super) fn set_parameter_impl(
        &self,
        parameter: SoundParameterId,
        value: f32,
    ) -> Result<(), SoundError> {
        if !value.is_finite() {
            return Err(SoundError::InvalidParameter(format!(
                "parameter {} must be finite",
                parameter.as_str()
            )));
        }
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .parameters
            .insert(parameter, value);
        Ok(())
    }

    pub(super) fn parameter_value_impl(
        &self,
        parameter: &SoundParameterId,
    ) -> Result<f32, SoundError> {
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .parameters
            .get(parameter)
            .copied()
            .ok_or_else(|| SoundError::UnknownParameter {
                parameter: parameter.clone(),
            })
    }

    pub(super) fn bind_automation_impl(
        &self,
        binding: SoundAutomationBinding,
    ) -> Result<(), SoundError> {
        validate_automation_binding(&binding)?;
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .automation_bindings
            .insert(binding.id, binding);
        Ok(())
    }

    pub(super) fn apply_automation_value_impl(
        &self,
        binding: SoundAutomationBindingId,
        value: f32,
    ) -> Result<(), SoundError> {
        ensure_finite_value("automation value", value)?;
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let binding_descriptor = state
            .automation_bindings
            .get(&binding)
            .cloned()
            .ok_or(SoundError::UnknownAutomationBinding { binding })?;
        apply_automation_target(
            &mut state,
            binding_descriptor.target,
            &binding_descriptor.parameter,
            value,
        )
    }

    pub(super) fn apply_automation_curve_sample_impl(
        &self,
        binding: SoundAutomationBindingId,
        curve: &SoundAutomationCurve,
        time_seconds: f32,
    ) -> Result<f32, SoundError> {
        let value = sample_automation_curve(curve, time_seconds)?;
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        let binding_descriptor = state
            .automation_bindings
            .get(&binding)
            .cloned()
            .ok_or(SoundError::UnknownAutomationBinding { binding })?;
        apply_automation_target(
            &mut state,
            binding_descriptor.target,
            &binding_descriptor.parameter,
            value,
        )?;
        Ok(value)
    }

    pub(super) fn unbind_automation_impl(
        &self,
        binding: SoundAutomationBindingId,
    ) -> Result<(), SoundError> {
        self.state
            .lock()
            .expect("sound state mutex poisoned")
            .automation_bindings
            .remove(&binding)
            .map(|_| ())
            .ok_or(SoundError::UnknownAutomationBinding { binding })
    }

    pub(super) fn schedule_timeline_sequence_impl(
        &self,
        sequence: SoundTimelineSequence,
    ) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        schedule_timeline_sequence(&mut state, sequence)
    }

    pub(super) fn remove_timeline_sequence_impl(
        &self,
        sequence: &SoundTimelineSequenceId,
    ) -> Result<(), SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        remove_timeline_sequence(&mut state, sequence)
    }

    pub(super) fn timeline_sequences_impl(&self) -> Result<Vec<SoundTimelineSequence>, SoundError> {
        Ok(self
            .state
            .lock()
            .expect("sound state mutex poisoned")
            .timeline_sequences
            .iter()
            .map(|playback| playback.sequence.clone())
            .collect())
    }

    pub(super) fn advance_timeline_sequences_impl(
        &self,
        delta_seconds: f32,
    ) -> Result<Vec<SoundTimelineSequenceAdvance>, SoundError> {
        let mut state = self.state.lock().expect("sound state mutex poisoned");
        advance_timeline_sequences(&mut state, delta_seconds)
    }
}
