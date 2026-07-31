use super::combo::KeyBindingKind;

pub const ACTION_BAR_SLOTS: usize = 23;
pub const KEYBIND_CATEGORIES: [&str; 5] =
    ["Movement", "Targeting", "Interface", "Pet", "Action Bar"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeybindAction {
    pub id: &'static str,
    pub label: &'static str,
    pub category: &'static str,
    pub kind: KeyBindingKind,
    pub defaults: [Option<&'static str>; 2],
    pub allow_shared: bool,
}

const fn action(
    id: &'static str,
    label: &'static str,
    category: &'static str,
    kind: KeyBindingKind,
    primary: &'static str,
) -> KeybindAction {
    KeybindAction {
        id,
        label,
        category,
        kind,
        defaults: [Some(primary), None],
        allow_shared: false,
    }
}

const fn action_with_secondary(
    id: &'static str,
    label: &'static str,
    category: &'static str,
    kind: KeyBindingKind,
    primary: &'static str,
    secondary: &'static str,
) -> KeybindAction {
    KeybindAction {
        id,
        label,
        category,
        kind,
        defaults: [Some(primary), Some(secondary)],
        allow_shared: false,
    }
}

pub const KEYBIND_ACTIONS: [KeybindAction; 61] = [
    action_with_secondary(
        "forward",
        "Move Forward",
        "Movement",
        KeyBindingKind::Held,
        "KeyW",
        "ArrowUp",
    ),
    action_with_secondary(
        "back",
        "Move Backward",
        "Movement",
        KeyBindingKind::Held,
        "KeyS",
        "ArrowDown",
    ),
    action_with_secondary(
        "turnLeft",
        "Turn Left",
        "Movement",
        KeyBindingKind::Held,
        "KeyA",
        "ArrowLeft",
    ),
    action_with_secondary(
        "turnRight",
        "Turn Right",
        "Movement",
        KeyBindingKind::Held,
        "KeyD",
        "ArrowRight",
    ),
    action(
        "strafeLeft",
        "Strafe Left",
        "Movement",
        KeyBindingKind::Held,
        "KeyQ",
    ),
    action(
        "strafeRight",
        "Strafe Right",
        "Movement",
        KeyBindingKind::Held,
        "KeyE",
    ),
    action("jump", "Jump", "Movement", KeyBindingKind::Held, "Space"),
    action(
        "autorun",
        "Toggle Autorun",
        "Movement",
        KeyBindingKind::Edge,
        "KeyR",
    ),
    action(
        "target",
        "Target Nearest Enemy",
        "Targeting",
        KeyBindingKind::Edge,
        "Tab",
    ),
    action(
        "targetFriendly",
        "Target Nearest Friendly",
        "Targeting",
        KeyBindingKind::Edge,
        "KeyH",
    ),
    action(
        "targetFriendlyNext",
        "Cycle Friendly Target",
        "Targeting",
        KeyBindingKind::Edge,
        "KeyJ",
    ),
    action(
        "interact",
        "Interact / Loot",
        "Targeting",
        KeyBindingKind::Edge,
        "KeyF",
    ),
    KeybindAction {
        allow_shared: true,
        ..action(
            "attackMove",
            "Attack Move",
            "Targeting",
            KeyBindingKind::Edge,
            "KeyA",
        )
    },
    action(
        "char",
        "Character",
        "Interface",
        KeyBindingKind::Edge,
        "KeyC",
    ),
    action(
        "spellbook",
        "Spellbook",
        "Interface",
        KeyBindingKind::Edge,
        "KeyP",
    ),
    action(
        "questlog",
        "Quest Log",
        "Interface",
        KeyBindingKind::Edge,
        "KeyL",
    ),
    action(
        "map",
        "World Map",
        "Interface",
        KeyBindingKind::Edge,
        "KeyM",
    ),
    action("bags", "Bags", "Interface", KeyBindingKind::Edge, "KeyB"),
    action(
        "crafting",
        "Crafting",
        "Interface",
        KeyBindingKind::Edge,
        "KeyT",
    ),
    action(
        "nameplates",
        "Toggle Nameplates",
        "Interface",
        KeyBindingKind::Edge,
        "KeyV",
    ),
    action(
        "talents",
        "Talents",
        "Interface",
        KeyBindingKind::Edge,
        "KeyN",
    ),
    action(
        "meters",
        "Damage Meters",
        "Interface",
        KeyBindingKind::Edge,
        "Shift+KeyH",
    ),
    action(
        "social",
        "Friends & Guild",
        "Interface",
        KeyBindingKind::Edge,
        "KeyO",
    ),
    action(
        "arena",
        "Arena (Ashen Coliseum)",
        "Interface",
        KeyBindingKind::Edge,
        "KeyG",
    ),
    action(
        "dungeonFinder",
        "Dungeon Finder",
        "Interface",
        KeyBindingKind::Edge,
        "Shift+KeyI",
    ),
    action(
        "valecup",
        "Vale Cup",
        "Interface",
        KeyBindingKind::Edge,
        "KeyY",
    ),
    action(
        "leaderboard",
        "Leaderboard",
        "Interface",
        KeyBindingKind::Edge,
        "KeyK",
    ),
    action(
        "calendar",
        "Event Calendar",
        "Interface",
        KeyBindingKind::Edge,
        "KeyI",
    ),
    action(
        "discord",
        "Discord",
        "Interface",
        KeyBindingKind::Edge,
        "KeyU",
    ),
    action(
        "deeds",
        "Book of Deeds",
        "Interface",
        KeyBindingKind::Edge,
        "Shift+KeyZ",
    ),
    action_with_secondary(
        "chat",
        "Open Chat",
        "Interface",
        KeyBindingKind::Edge,
        "Enter",
        "NumpadEnter",
    ),
    action(
        "emoteWheel",
        "Emote Wheel",
        "Interface",
        KeyBindingKind::Held,
        "KeyX",
    ),
    action(
        "sheathe",
        "Sheathe/Unsheathe Weapon",
        "Interface",
        KeyBindingKind::Edge,
        "KeyZ",
    ),
    action(
        "petAttack",
        "Pet: Attack",
        "Pet",
        KeyBindingKind::Edge,
        "Ctrl+Digit1",
    ),
    action(
        "petStop",
        "Pet: Stop",
        "Pet",
        KeyBindingKind::Edge,
        "Ctrl+Digit2",
    ),
    action(
        "petTaunt",
        "Pet: Taunt",
        "Pet",
        KeyBindingKind::Edge,
        "Ctrl+Digit3",
    ),
    action(
        "petDefensive",
        "Pet: Defensive",
        "Pet",
        KeyBindingKind::Edge,
        "Ctrl+Digit4",
    ),
    action(
        "petAggressive",
        "Pet: Aggressive",
        "Pet",
        KeyBindingKind::Edge,
        "Ctrl+Digit5",
    ),
    action(
        "slot0",
        "Attack",
        "Action Bar",
        KeyBindingKind::Edge,
        "Digit1",
    ),
    action(
        "slot1",
        "Action Bar 2",
        "Action Bar",
        KeyBindingKind::Edge,
        "Digit2",
    ),
    action(
        "slot2",
        "Action Bar 3",
        "Action Bar",
        KeyBindingKind::Edge,
        "Digit3",
    ),
    action(
        "slot3",
        "Action Bar 4",
        "Action Bar",
        KeyBindingKind::Edge,
        "Digit4",
    ),
    action(
        "slot4",
        "Action Bar 5",
        "Action Bar",
        KeyBindingKind::Edge,
        "Digit5",
    ),
    action(
        "slot5",
        "Action Bar 6",
        "Action Bar",
        KeyBindingKind::Edge,
        "Digit6",
    ),
    action(
        "slot6",
        "Action Bar 7",
        "Action Bar",
        KeyBindingKind::Edge,
        "Digit7",
    ),
    action(
        "slot7",
        "Action Bar 8",
        "Action Bar",
        KeyBindingKind::Edge,
        "Digit8",
    ),
    action(
        "slot8",
        "Action Bar 9",
        "Action Bar",
        KeyBindingKind::Edge,
        "Digit9",
    ),
    action(
        "slot9",
        "Action Bar 10",
        "Action Bar",
        KeyBindingKind::Edge,
        "Digit0",
    ),
    action(
        "slot10",
        "Action Bar 11",
        "Action Bar",
        KeyBindingKind::Edge,
        "Minus",
    ),
    action(
        "slot11",
        "Action Bar 12",
        "Action Bar",
        KeyBindingKind::Edge,
        "Equal",
    ),
    action(
        "slot12",
        "Secondary Bar 1",
        "Action Bar",
        KeyBindingKind::Edge,
        "Numpad1",
    ),
    action(
        "slot13",
        "Secondary Bar 2",
        "Action Bar",
        KeyBindingKind::Edge,
        "Numpad2",
    ),
    action(
        "slot14",
        "Secondary Bar 3",
        "Action Bar",
        KeyBindingKind::Edge,
        "Numpad3",
    ),
    action(
        "slot15",
        "Secondary Bar 4",
        "Action Bar",
        KeyBindingKind::Edge,
        "Numpad4",
    ),
    action(
        "slot16",
        "Secondary Bar 5",
        "Action Bar",
        KeyBindingKind::Edge,
        "Numpad5",
    ),
    action(
        "slot17",
        "Secondary Bar 6",
        "Action Bar",
        KeyBindingKind::Edge,
        "Numpad6",
    ),
    action(
        "slot18",
        "Secondary Bar 7",
        "Action Bar",
        KeyBindingKind::Edge,
        "Numpad7",
    ),
    action(
        "slot19",
        "Secondary Bar 8",
        "Action Bar",
        KeyBindingKind::Edge,
        "Numpad8",
    ),
    action(
        "slot20",
        "Secondary Bar 9",
        "Action Bar",
        KeyBindingKind::Edge,
        "Numpad9",
    ),
    action(
        "slot21",
        "Secondary Bar 10",
        "Action Bar",
        KeyBindingKind::Edge,
        "Numpad0",
    ),
    action(
        "slot22",
        "Secondary Bar 11",
        "Action Bar",
        KeyBindingKind::Edge,
        "NumpadDecimal",
    ),
];

pub fn keybind_action(id: &str) -> Option<&'static KeybindAction> {
    KEYBIND_ACTIONS.iter().find(|action| action.id == id)
}

pub fn action_kind(id: &str) -> Option<KeyBindingKind> {
    keybind_action(id).map(|action| action.kind)
}

pub fn action_allows_shared(id: &str) -> bool {
    keybind_action(id).is_some_and(|action| action.allow_shared)
}
