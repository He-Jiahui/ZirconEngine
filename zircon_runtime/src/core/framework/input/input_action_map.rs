use serde::{Deserialize, Serialize};

use super::{InputAction, InputActionContext, InputBinding};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputActionMap {
    #[serde(default)]
    pub contexts: Vec<InputActionContext>,
    pub actions: Vec<InputAction>,
    pub bindings: Vec<InputBinding>,
}

impl InputActionMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_action(mut self, action: InputAction) -> Self {
        self.add_action(action);
        self
    }

    pub fn with_context(mut self, context: InputActionContext) -> Self {
        self.add_context(context);
        self
    }

    pub fn with_binding(mut self, binding: InputBinding) -> Self {
        self.bind(binding);
        self
    }

    pub fn add_context(&mut self, context: InputActionContext) -> &mut Self {
        if !self.has_context(&context.id) {
            self.contexts.push(context);
            self.contexts.sort_by(|left, right| {
                right
                    .priority
                    .cmp(&left.priority)
                    .then(left.id.cmp(&right.id))
            });
        }
        self
    }

    pub fn add_action(&mut self, action: InputAction) -> &mut Self {
        if !self.has_action(&action.id) {
            self.actions.push(action);
        }
        self
    }

    pub fn bind(&mut self, binding: InputBinding) -> &mut Self {
        if !binding.is_empty() {
            self.bindings.push(binding);
        }
        self
    }

    pub fn clear_bindings(&mut self, action: impl AsRef<str>) -> &mut Self {
        let action = action.as_ref();
        self.bindings.retain(|binding| binding.action != action);
        self
    }

    pub fn has_action(&self, action: impl AsRef<str>) -> bool {
        let action = action.as_ref();
        self.actions.iter().any(|candidate| candidate.id == action)
    }

    pub fn has_context(&self, context: impl AsRef<str>) -> bool {
        let context = context.as_ref();
        self.contexts
            .iter()
            .any(|candidate| candidate.id == context)
    }

    pub fn context_enabled(&self, context: impl AsRef<str>) -> bool {
        let context = context.as_ref();
        self.contexts
            .iter()
            .find(|candidate| candidate.id == context)
            .map(|candidate| candidate.enabled)
            .unwrap_or(true)
    }

    pub fn bindings_for_action<'a>(
        &'a self,
        action: &'a str,
    ) -> impl Iterator<Item = &'a InputBinding> + 'a {
        self.bindings
            .iter()
            .filter(move |binding| binding.action == action)
    }
}
