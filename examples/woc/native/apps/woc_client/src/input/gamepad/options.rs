use super::{
    bindings::GamepadBindings,
    layout::{gamepad_button_label, GamepadKind, BINDABLE_GAMEPAD_BUTTONS, GAMEPAD_NONE_ACTION},
    storage::StoredGamepadBindings,
};
use crate::{
    input::keybind::{KeyBindingKind, KEYBIND_ACTIONS},
    preferences::PreferenceStorage,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GamepadActionOption {
    pub action_id: &'static str,
    pub fallback_label: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GamepadControllerRow {
    pub button: usize,
    pub button_label: String,
    pub action: String,
}

pub trait GamepadOptionsBindings {
    fn action_for(&self, button: usize) -> &str;
    fn bind(&mut self, button: usize, action: &str);
    fn reset(&mut self);
}

impl GamepadOptionsBindings for GamepadBindings {
    fn action_for(&self, button: usize) -> &str {
        GamepadBindings::action_for(self, button)
    }

    fn bind(&mut self, button: usize, action: &str) {
        GamepadBindings::bind(self, button, action);
    }

    fn reset(&mut self) {
        GamepadBindings::reset(self);
    }
}

impl<S> GamepadOptionsBindings for StoredGamepadBindings<S>
where
    S: PreferenceStorage,
{
    fn action_for(&self, button: usize) -> &str {
        GamepadBindings::action_for(self, button)
    }

    fn bind(&mut self, button: usize, action: &str) {
        StoredGamepadBindings::bind(self, button, action);
    }

    fn reset(&mut self) {
        StoredGamepadBindings::reset(self);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GamepadControllerModel {
    kind: GamepadKind,
}

impl GamepadControllerModel {
    pub const fn new(kind: GamepadKind) -> Self {
        Self { kind }
    }

    pub const fn kind(self) -> GamepadKind {
        self.kind
    }

    pub fn rows<B>(&self, bindings: &B) -> Vec<GamepadControllerRow>
    where
        B: GamepadOptionsBindings,
    {
        BINDABLE_GAMEPAD_BUTTONS
            .into_iter()
            .map(|button| GamepadControllerRow {
                button,
                button_label: gamepad_button_label(button, self.kind),
                action: bindings.action_for(button).to_string(),
            })
            .collect()
    }

    pub fn bind<B>(&self, bindings: &mut B, button: usize, action: &str)
    where
        B: GamepadOptionsBindings,
    {
        bindings.bind(button, action);
    }

    pub fn reset<B>(&self, bindings: &mut B)
    where
        B: GamepadOptionsBindings,
    {
        bindings.reset();
    }
}

pub fn gamepad_action_options() -> Vec<GamepadActionOption> {
    let mut options = Vec::with_capacity(55);
    options.push(GamepadActionOption {
        action_id: GAMEPAD_NONE_ACTION,
        fallback_label: "Unbound",
    });
    options.push(GamepadActionOption {
        action_id: "escape",
        fallback_label: "Game Menu",
    });
    options.extend(
        KEYBIND_ACTIONS
            .iter()
            .filter(|action| {
                action.id != "attackMove"
                    && (action.kind == KeyBindingKind::Edge || action.id == "jump")
            })
            .map(|action| GamepadActionOption {
                action_id: action.id,
                fallback_label: action.label,
            }),
    );
    options
}
