use std::collections::hash_map::Entry;
use std::collections::HashMap;

#[cfg(test)]
use zircon_runtime::ui::template::UiTemplateInstance;
use zircon_runtime_interface::ui::template::UiBindingRef;

use crate::ui::binding::EditorUiBinding;
use crate::ui::template::EditorTemplateError;

#[cfg(test)]
#[path = "adapter/hash_index_tests.rs"]
mod hash_index_tests;

#[derive(Default)]
pub struct EditorTemplateAdapter {
    bindings: HashMap<String, EditorUiBinding>,
}

impl EditorTemplateAdapter {
    pub fn register_binding(
        &mut self,
        binding_id: impl Into<String>,
        binding: EditorUiBinding,
    ) -> Result<(), EditorTemplateError> {
        let binding_id = binding_id.into();
        match self.bindings.entry(binding_id) {
            Entry::Vacant(entry) => {
                entry.insert(binding);
                Ok(())
            }
            Entry::Occupied(entry) => Err(EditorTemplateError::DuplicateBinding {
                binding_id: entry.key().clone(),
            }),
        }
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

    #[cfg(test)]
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
