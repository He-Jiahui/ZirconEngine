use std::collections::HashMap;

use zircon_runtime::core::framework::sound::{
    SoundAutomationBinding, SoundAutomationBindingId, SoundError, SoundMixerGraph,
};

use crate::automation::binding::normalized_automation_binding;

pub(crate) fn configured_automation_bindings(
    graph: &SoundMixerGraph,
) -> Result<HashMap<SoundAutomationBindingId, SoundAutomationBinding>, SoundError> {
    let mut automation_bindings = HashMap::new();
    for binding in graph.automation_bindings.iter().cloned() {
        let binding = normalized_automation_binding(binding)?;
        if automation_bindings.insert(binding.id, binding).is_some() {
            return Err(SoundError::InvalidParameter(
                "configured mixer graph contains duplicate automation binding ids".to_string(),
            ));
        }
    }
    Ok(automation_bindings)
}
