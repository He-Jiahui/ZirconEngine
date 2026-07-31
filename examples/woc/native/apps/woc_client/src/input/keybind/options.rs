use super::{
    bindings::{Keybinds, BINDING_SLOTS},
    combo::{is_modifier_code, is_reserved_combo, key_label, make_key_combo, KeyModifiers},
    registry::{keybind_action, KEYBIND_ACTIONS, KEYBIND_CATEGORIES},
    storage::StoredKeybinds,
};
use crate::preferences::PreferenceStorage;

pub trait KeybindOptionsBindings {
    fn code_at(&self, id: &str, slot: usize) -> Option<&str>;
    fn bind(&mut self, id: &str, slot: usize, combo: &str) -> bool;
    fn reset(&mut self);
}

impl KeybindOptionsBindings for Keybinds {
    fn code_at(&self, id: &str, slot: usize) -> Option<&str> {
        Keybinds::code_at(self, id, slot)
    }

    fn bind(&mut self, id: &str, slot: usize, combo: &str) -> bool {
        Keybinds::bind(self, id, slot, combo)
    }

    fn reset(&mut self) {
        Keybinds::reset(self);
    }
}

impl<S> KeybindOptionsBindings for StoredKeybinds<S>
where
    S: PreferenceStorage,
{
    fn code_at(&self, id: &str, slot: usize) -> Option<&str> {
        Keybinds::code_at(self, id, slot)
    }

    fn bind(&mut self, id: &str, slot: usize, combo: &str) -> bool {
        StoredKeybinds::bind(self, id, slot, combo)
    }

    fn reset(&mut self) {
        StoredKeybinds::reset(self);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeybindOptionsCategory {
    pub id: &'static str,
    pub rows: Vec<KeybindOptionsRow>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeybindOptionsRow {
    pub action_id: &'static str,
    pub label: &'static str,
    pub primary_hint: Option<String>,
    pub slots: [KeybindOptionsSlot; BINDING_SLOTS],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeybindOptionsSlot {
    pub label: Option<String>,
    pub capturing: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeybindOptionsNote {
    Help,
    Capturing {
        action_id: &'static str,
    },
    Cancelled,
    Bound {
        action_id: &'static str,
        key_label: String,
    },
    Reserved {
        key_label: String,
    },
    Reset,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeybindCaptureOutcome {
    NotCapturing,
    RepeatIgnored,
    ModifierIgnored,
    Cancelled,
    Bound {
        action_id: &'static str,
        slot: usize,
        stored_combo: String,
    },
    Rejected {
        action_id: &'static str,
        slot: usize,
        combo: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CaptureTarget {
    action_id: &'static str,
    slot: usize,
}

pub struct KeybindOptionsModel {
    attack_move_enabled: bool,
    capture: Option<CaptureTarget>,
    note: KeybindOptionsNote,
}

impl Default for KeybindOptionsModel {
    fn default() -> Self {
        Self {
            attack_move_enabled: false,
            capture: None,
            note: KeybindOptionsNote::Help,
        }
    }
}

impl KeybindOptionsModel {
    pub fn set_attack_move_enabled(&mut self, enabled: bool) {
        self.attack_move_enabled = enabled;
    }

    pub fn categories<B>(&self, bindings: &B) -> Vec<KeybindOptionsCategory>
    where
        B: KeybindOptionsBindings,
    {
        KEYBIND_CATEGORIES
            .iter()
            .filter_map(|category| {
                let rows = KEYBIND_ACTIONS
                    .iter()
                    .filter(|action| {
                        action.category == *category
                            && (action.id != "attackMove" || self.attack_move_enabled)
                    })
                    .map(|action| KeybindOptionsRow {
                        action_id: action.id,
                        label: action.label,
                        primary_hint: binding_label(bindings, action.id, 0),
                        slots: std::array::from_fn(|slot| KeybindOptionsSlot {
                            label: binding_label(bindings, action.id, slot),
                            capturing: self.capture
                                == Some(CaptureTarget {
                                    action_id: action.id,
                                    slot,
                                }),
                        }),
                    })
                    .collect::<Vec<_>>();
                (!rows.is_empty()).then_some(KeybindOptionsCategory {
                    id: *category,
                    rows,
                })
            })
            .collect()
    }

    pub fn begin_capture(&mut self, action_id: &str, slot: usize) -> bool {
        let Some(action) = keybind_action(action_id) else {
            return false;
        };
        if slot >= BINDING_SLOTS || (action.id == "attackMove" && !self.attack_move_enabled) {
            return false;
        }
        self.capture = Some(CaptureTarget {
            action_id: action.id,
            slot,
        });
        self.note = KeybindOptionsNote::Capturing {
            action_id: action.id,
        };
        true
    }

    pub fn handle_key_down<B>(
        &mut self,
        bindings: &mut B,
        code: &str,
        modifiers: KeyModifiers,
        repeat: bool,
    ) -> KeybindCaptureOutcome
    where
        B: KeybindOptionsBindings,
    {
        if repeat {
            return KeybindCaptureOutcome::RepeatIgnored;
        }
        let Some(capture) = self.capture else {
            return KeybindCaptureOutcome::NotCapturing;
        };
        if code == "Escape" {
            self.capture = None;
            self.note = KeybindOptionsNote::Cancelled;
            return KeybindCaptureOutcome::Cancelled;
        }
        if is_modifier_code(code) {
            return KeybindCaptureOutcome::ModifierIgnored;
        }

        self.capture = None;
        let combo = make_key_combo(code, modifiers);
        if bindings.bind(capture.action_id, capture.slot, &combo) {
            let stored_combo = bindings
                .code_at(capture.action_id, capture.slot)
                .unwrap_or_default()
                .to_string();
            self.note = KeybindOptionsNote::Bound {
                action_id: capture.action_id,
                key_label: key_label(Some(&stored_combo)),
            };
            return KeybindCaptureOutcome::Bound {
                action_id: capture.action_id,
                slot: capture.slot,
                stored_combo,
            };
        }

        self.note = if is_reserved_combo(&combo) {
            KeybindOptionsNote::Reserved {
                key_label: key_label(Some(&combo)),
            }
        } else {
            KeybindOptionsNote::Capturing {
                action_id: capture.action_id,
            }
        };
        KeybindCaptureOutcome::Rejected {
            action_id: capture.action_id,
            slot: capture.slot,
            combo,
        }
    }

    pub fn reset<B>(&mut self, bindings: &mut B)
    where
        B: KeybindOptionsBindings,
    {
        bindings.reset();
        self.capture = None;
        self.note = KeybindOptionsNote::Reset;
    }

    pub fn leave_panel(&mut self) {
        self.capture = None;
        self.note = KeybindOptionsNote::Help;
    }

    pub fn capture_target(&self) -> Option<(&'static str, usize)> {
        self.capture
            .map(|capture| (capture.action_id, capture.slot))
    }

    pub fn note(&self) -> &KeybindOptionsNote {
        &self.note
    }
}

fn binding_label<B>(bindings: &B, action_id: &str, slot: usize) -> Option<String>
where
    B: KeybindOptionsBindings,
{
    let label = key_label(bindings.code_at(action_id, slot));
    (!label.is_empty()).then_some(label)
}
