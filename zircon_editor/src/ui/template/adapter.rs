use std::collections::BTreeMap;

use zircon_runtime::ui::template::{UiBindingRef, UiTemplateInstance};

use crate::ui::binding::EditorUiBinding;
use crate::ui::template::EditorTemplateError;

#[derive(Default)]
pub struct EditorTemplateAdapter {
    bindings: BTreeMap<String, EditorUiBinding>,
}

impl EditorTemplateAdapter {
    pub fn register_binding(
        &mut self,
        binding_id: impl Into<String>,
        binding: EditorUiBinding,
    ) -> Result<(), EditorTemplateError> {
        let binding_id = binding_id.into();
        if self.bindings.contains_key(&binding_id) {
            return Err(EditorTemplateError::DuplicateBinding { binding_id });
        }
        self.bindings.insert(binding_id, binding);
        Ok(())
    }

    pub fn resolve_binding(
        &self,
        binding_ref: &UiBindingRef,
    ) -> Result<EditorUiBinding, EditorTemplateError> {
        let binding = self.bindings.get(&binding_ref.id).cloned().ok_or_else(|| {
            EditorTemplateError::MissingBinding {
                binding_id: binding_ref.id.clone(),
            }
        })?;
        let actual = binding.path().event_kind;
        if actual != binding_ref.event {
            return Err(EditorTemplateError::BindingEventMismatch {
                binding_id: binding_ref.id.clone(),
                expected: binding_ref.event,
                actual,
            });
        }
        Ok(binding)
    }

    pub fn resolve_instance_bindings(
        &self,
        instance: &UiTemplateInstance,
    ) -> Result<Vec<EditorUiBinding>, EditorTemplateError> {
        instance
            .binding_refs()
            .into_iter()
            .map(|binding_ref| self.resolve_binding(binding_ref))
            .collect()
    }
}
