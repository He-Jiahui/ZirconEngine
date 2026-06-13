use serde::{Deserialize, Serialize};

use super::{InputAction, InputBinding};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputActionMap {
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

    pub fn with_binding(mut self, binding: InputBinding) -> Self {
        self.bind(binding);
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

    pub fn bindings_for_action<'a>(
        &'a self,
        action: &'a str,
    ) -> impl Iterator<Item = &'a InputBinding> + 'a {
        self.bindings
            .iter()
            .filter(move |binding| binding.action == action)
    }
}
