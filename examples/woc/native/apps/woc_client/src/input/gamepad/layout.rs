pub const STANDARD_GAMEPAD_BUTTON_COUNT: usize = 17;
pub const GAMEPAD_TRIGGER_THRESHOLD: f64 = 0.5;
pub const GAMEPAD_NONE_ACTION: &str = "none";

pub mod gamepad_button {
    pub const A: usize = 0;
    pub const B: usize = 1;
    pub const X: usize = 2;
    pub const Y: usize = 3;
    pub const LB: usize = 4;
    pub const RB: usize = 5;
    pub const LT: usize = 6;
    pub const RT: usize = 7;
    pub const BACK: usize = 8;
    pub const START: usize = 9;
    pub const L3: usize = 10;
    pub const R3: usize = 11;
    pub const DPAD_UP: usize = 12;
    pub const DPAD_DOWN: usize = 13;
    pub const DPAD_LEFT: usize = 14;
    pub const DPAD_RIGHT: usize = 15;
    pub const GUIDE: usize = 16;
}

pub const BINDABLE_GAMEPAD_BUTTONS: [usize; 16] = [
    gamepad_button::A,
    gamepad_button::B,
    gamepad_button::X,
    gamepad_button::Y,
    gamepad_button::LB,
    gamepad_button::RB,
    gamepad_button::LT,
    gamepad_button::RT,
    gamepad_button::BACK,
    gamepad_button::START,
    gamepad_button::L3,
    gamepad_button::R3,
    gamepad_button::DPAD_UP,
    gamepad_button::DPAD_DOWN,
    gamepad_button::DPAD_LEFT,
    gamepad_button::DPAD_RIGHT,
];

pub const DEFAULT_GAMEPAD_BINDINGS: [(usize, &str); 16] = [
    (gamepad_button::A, "jump"),
    (gamepad_button::B, "interact"),
    (gamepad_button::X, "slot0"),
    (gamepad_button::Y, "target"),
    (gamepad_button::RB, "slot1"),
    (gamepad_button::LB, "slot2"),
    (gamepad_button::RT, "slot3"),
    (gamepad_button::LT, "slot4"),
    (gamepad_button::DPAD_UP, "slot5"),
    (gamepad_button::DPAD_RIGHT, "slot6"),
    (gamepad_button::DPAD_DOWN, "slot7"),
    (gamepad_button::DPAD_LEFT, "slot8"),
    (gamepad_button::BACK, "map"),
    (gamepad_button::START, "escape"),
    (gamepad_button::L3, "autorun"),
    (gamepad_button::R3, "targetFriendly"),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GamepadKind {
    Xbox,
    PlayStation,
    Nintendo,
    Generic,
}

impl GamepadKind {
    pub const ALL: [Self; 4] = [Self::Generic, Self::Xbox, Self::PlayStation, Self::Nintendo];
}

pub fn default_gamepad_action(button: usize) -> Option<&'static str> {
    DEFAULT_GAMEPAD_BINDINGS
        .iter()
        .find_map(|(candidate, action)| (*candidate == button).then_some(*action))
}

pub fn detect_gamepad_kind(id: &str) -> GamepadKind {
    let lowercase = id.to_ascii_lowercase();
    if ["dualsense", "dualshock", "playstation"]
        .iter()
        .any(|name| lowercase.contains(name))
    {
        return GamepadKind::PlayStation;
    }
    if ["xbox", "x-box", "xinput"]
        .iter()
        .any(|name| lowercase.contains(name))
    {
        return GamepadKind::Xbox;
    }
    if ["switch", "joy-con", "joycon", "pro controller"]
        .iter()
        .any(|name| lowercase.contains(name))
    {
        return GamepadKind::Nintendo;
    }

    match gamepad_vendor_id(&lowercase) {
        Some("054c") => GamepadKind::PlayStation,
        Some("045e") => GamepadKind::Xbox,
        Some("057e") => GamepadKind::Nintendo,
        _ => GamepadKind::Generic,
    }
}

pub fn gamepad_button_label(button: usize, kind: GamepadKind) -> String {
    brand_button_label(button, kind)
        .or_else(|| brand_button_label(button, GamepadKind::Generic))
        .map_or_else(|| format!("#{button}"), str::to_string)
}

fn gamepad_vendor_id(id: &str) -> Option<&str> {
    if let Some((_, tail)) = id.split_once("vendor:") {
        let candidate = tail.trim_start().get(..4)?;
        if candidate.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Some(candidate);
        }
    }

    let bytes = id.as_bytes();
    if bytes.len() >= 10
        && bytes[4] == b'-'
        && bytes[9] == b'-'
        && bytes[..4].iter().all(u8::is_ascii_hexdigit)
        && bytes[5..9].iter().all(u8::is_ascii_hexdigit)
    {
        id.get(..4)
    } else {
        None
    }
}

fn brand_button_label(button: usize, kind: GamepadKind) -> Option<&'static str> {
    if let Some(label) = dpad_label(button) {
        return Some(label);
    }
    match kind {
        GamepadKind::Generic => match button {
            gamepad_button::A => Some("A / Cross"),
            gamepad_button::B => Some("B / Circle"),
            gamepad_button::X => Some("X / Square"),
            gamepad_button::Y => Some("Y / Triangle"),
            gamepad_button::LB => Some("LB / L1"),
            gamepad_button::RB => Some("RB / R1"),
            gamepad_button::LT => Some("LT / L2"),
            gamepad_button::RT => Some("RT / R2"),
            gamepad_button::BACK => Some("Back / Share"),
            gamepad_button::START => Some("Start / Options"),
            gamepad_button::L3 => Some("L3"),
            gamepad_button::R3 => Some("R3"),
            _ => None,
        },
        GamepadKind::Xbox => match button {
            gamepad_button::A => Some("A"),
            gamepad_button::B => Some("B"),
            gamepad_button::X => Some("X"),
            gamepad_button::Y => Some("Y"),
            gamepad_button::LB => Some("LB"),
            gamepad_button::RB => Some("RB"),
            gamepad_button::LT => Some("LT"),
            gamepad_button::RT => Some("RT"),
            gamepad_button::BACK => Some("View"),
            gamepad_button::START => Some("Menu"),
            gamepad_button::L3 => Some("L3"),
            gamepad_button::R3 => Some("R3"),
            _ => None,
        },
        GamepadKind::PlayStation => match button {
            gamepad_button::A => Some("Cross"),
            gamepad_button::B => Some("Circle"),
            gamepad_button::X => Some("Square"),
            gamepad_button::Y => Some("Triangle"),
            gamepad_button::LB => Some("L1"),
            gamepad_button::RB => Some("R1"),
            gamepad_button::LT => Some("L2"),
            gamepad_button::RT => Some("R2"),
            gamepad_button::BACK => Some("Share / Create"),
            gamepad_button::START => Some("Options"),
            gamepad_button::L3 => Some("L3"),
            gamepad_button::R3 => Some("R3"),
            _ => None,
        },
        GamepadKind::Nintendo => match button {
            gamepad_button::A => Some("B"),
            gamepad_button::B => Some("A"),
            gamepad_button::X => Some("Y"),
            gamepad_button::Y => Some("X"),
            gamepad_button::LB => Some("L"),
            gamepad_button::RB => Some("R"),
            gamepad_button::LT => Some("ZL"),
            gamepad_button::RT => Some("ZR"),
            gamepad_button::BACK => Some("Minus"),
            gamepad_button::START => Some("Plus"),
            gamepad_button::L3 => Some("L Stick"),
            gamepad_button::R3 => Some("R Stick"),
            _ => None,
        },
    }
}

fn dpad_label(button: usize) -> Option<&'static str> {
    match button {
        gamepad_button::DPAD_UP => Some("D-pad ↑"),
        gamepad_button::DPAD_DOWN => Some("D-pad ↓"),
        gamepad_button::DPAD_LEFT => Some("D-pad ←"),
        gamepad_button::DPAD_RIGHT => Some("D-pad →"),
        _ => None,
    }
}
