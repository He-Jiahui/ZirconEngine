use zircon_runtime::core::framework::sound::{
    SoundAutomationBinding, SoundAutomationBindingId, SoundAutomationCurve, SoundError,
};

use crate::automation::binding::normalized_automation_binding;
use crate::automation::curve::sample_automation_curve;
use crate::automation::target::apply_automation_target;

use super::DefaultSoundManager;

impl DefaultSoundManager {
    pub(super) fn bind_automation_impl(
        &self,
        binding: SoundAutomationBinding,
    ) -> Result<(), SoundError> {
        let binding = normalized_automation_binding(binding)?;
        crate::poison_recovery::lock_recover(&self.state)
            .automation_bindings
            .insert(binding.id, binding);
        Ok(())
    }

    pub(super) fn apply_automation_value_impl(
        &self,
        binding: SoundAutomationBindingId,
        value: f32,
    ) -> Result<(), SoundError> {
        let mut state = crate::poison_recovery::lock_recover(&self.state);
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
        let mut state = crate::poison_recovery::lock_recover(&self.state);
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
        crate::poison_recovery::lock_recover(&self.state)
            .automation_bindings
            .remove(&binding)
            .map(|_| ())
            .ok_or(SoundError::UnknownAutomationBinding { binding })
    }
}
