//! 自动化绑定保存目标身份，应用值时才解析为当前状态；采样路径与直接赋值共用目标校验。
use zircon_runtime::core::framework::sound::{
    SoundAutomationBinding, SoundAutomationBindingId, SoundAutomationCurve, SoundAutomationTarget,
    SoundError, SoundParameterId,
};

use crate::automation::binding::normalized_automation_binding;
use crate::automation::curve::sample_automation_curve;
use crate::automation::target::apply_automation_target;
use crate::engine::SoundEngineState;

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
        let (target, parameter) = owned_automation_target(&state, binding)?;
        apply_automation_target(&mut state, target, &parameter, value)
    }

    pub(super) fn apply_automation_curve_sample_impl(
        &self,
        binding: SoundAutomationBindingId,
        curve: &SoundAutomationCurve,
        time_seconds: f32,
    ) -> Result<f32, SoundError> {
        let value = sample_automation_curve(curve, time_seconds)?;
        let mut state = crate::poison_recovery::lock_recover(&self.state);
        let (target, parameter) = owned_automation_target(&state, binding)?;
        apply_automation_target(&mut state, target, &parameter, value)?;
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

fn owned_automation_target(
    state: &SoundEngineState,
    binding: SoundAutomationBindingId,
) -> Result<(SoundAutomationTarget, SoundParameterId), SoundError> {
    state
        .automation_bindings
        .get(&binding)
        .map(|descriptor| (descriptor.target.clone(), descriptor.parameter.clone()))
        .ok_or(SoundError::UnknownAutomationBinding { binding })
}

#[cfg(test)]
#[path = "tests/automation_timeline.rs"]
mod tests;
