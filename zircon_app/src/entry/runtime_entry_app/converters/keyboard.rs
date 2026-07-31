use std::fmt;

use winit::event::ElementState;
use winit::keyboard::{KeyCode, NativeKeyCode, PhysicalKey};
use zircon_runtime_interface::{
    ZR_RUNTIME_KEY_ACTION_PRESSED_V1, ZR_RUNTIME_KEY_ACTION_RELEASED_V1,
};

pub(in crate::entry::runtime_entry_app) fn key_action(state: ElementState) -> Option<u32> {
    match state {
        ElementState::Pressed => Some(ZR_RUNTIME_KEY_ACTION_PRESSED_V1),
        ElementState::Released => Some(ZR_RUNTIME_KEY_ACTION_RELEASED_V1),
    }
}

pub(in crate::entry::runtime_entry_app) fn physical_key_code(key: &PhysicalKey) -> u32 {
    match key {
        PhysicalKey::Code(code) => match code {
            KeyCode::ShiftLeft | KeyCode::ShiftRight => 16,
            KeyCode::ControlLeft | KeyCode::ControlRight => 17,
            KeyCode::AltLeft | KeyCode::AltRight => 18,
            KeyCode::KeyA => u32::from(b'A'),
            KeyCode::KeyD => u32::from(b'D'),
            KeyCode::KeyS => u32::from(b'S'),
            KeyCode::KeyW => u32::from(b'W'),
            _ => stable_key_code(code),
        },
        PhysicalKey::Unidentified(native) => native_key_code(native),
    }
}

fn native_key_code(native: &NativeKeyCode) -> u32 {
    match *native {
        NativeKeyCode::Unidentified => 0,
        NativeKeyCode::Android(code) | NativeKeyCode::Xkb(code) => code,
        NativeKeyCode::MacOS(code) | NativeKeyCode::Windows(code) => code as u32,
    }
}

const FNV_OFFSET: u32 = 2_166_136_261;
const FNV_PRIME: u32 = 16_777_619;

struct StableKeyCodeHasher {
    hash: u32,
}

impl StableKeyCodeHasher {
    fn new() -> Self {
        Self { hash: FNV_OFFSET }
    }

    fn finish(self) -> u32 {
        self.hash.max(1)
    }
}

impl fmt::Write for StableKeyCodeHasher {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        for byte in value.as_bytes() {
            self.hash ^= u32::from(*byte);
            self.hash = self.hash.wrapping_mul(FNV_PRIME);
        }
        Ok(())
    }
}

fn stable_key_code(code: &KeyCode) -> u32 {
    let mut hasher = StableKeyCodeHasher::new();
    let result = fmt::write(&mut hasher, format_args!("{code:?}"));
    debug_assert!(result.is_ok());
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_states_map_to_runtime_constants() {
        assert_eq!(
            key_action(ElementState::Pressed),
            Some(ZR_RUNTIME_KEY_ACTION_PRESSED_V1)
        );
        assert_eq!(
            key_action(ElementState::Released),
            Some(ZR_RUNTIME_KEY_ACTION_RELEASED_V1)
        );
    }

    #[test]
    fn physical_keys_map_to_runtime_values() {
        assert_eq!(
            physical_key_code(&PhysicalKey::Code(KeyCode::ShiftLeft)),
            16
        );
        assert_eq!(
            physical_key_code(&PhysicalKey::Code(KeyCode::ControlRight)),
            17
        );
        assert_eq!(physical_key_code(&PhysicalKey::Code(KeyCode::AltLeft)), 18);
        assert_eq!(physical_key_code(&PhysicalKey::Code(KeyCode::KeyW)), 87);
        assert_eq!(physical_key_code(&PhysicalKey::Code(KeyCode::KeyA)), 65);
        assert_eq!(physical_key_code(&PhysicalKey::Code(KeyCode::KeyS)), 83);
        assert_eq!(physical_key_code(&PhysicalKey::Code(KeyCode::KeyD)), 68);
        assert_eq!(
            physical_key_code(&PhysicalKey::Unidentified(NativeKeyCode::Xkb(77))),
            77
        );
    }

    #[test]
    fn fallback_key_codes_keep_the_previous_debug_fnv_values() {
        for (code, expected) in [
            (KeyCode::Escape, 3_082_514_982),
            (KeyCode::F12, 3_736_956_062),
            (KeyCode::ArrowUp, 154_847_355),
            (KeyCode::Numpad9, 2_061_263_975),
        ] {
            assert_eq!(physical_key_code(&PhysicalKey::Code(code)), expected);
        }
    }

    #[test]
    fn production_key_fallback_formats_into_the_hash_without_allocating() {
        let production = include_str!("keyboard.rs")
            .split_once("\n#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("keyboard production source precedes its test module");

        assert!(!production.contains("format!("));
        assert!(!production.contains("to_string("));
        assert!(production.contains("impl fmt::Write for StableKeyCodeHasher"));
        assert!(production.contains("fmt::write(&mut hasher, format_args!(\"{code:?}\"))"));
    }
}
