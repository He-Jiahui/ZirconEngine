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
            _ => stable_key_code(format!("{code:?}").as_bytes()),
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

fn stable_key_code(bytes: &[u8]) -> u32 {
    const FNV_OFFSET: u32 = 2_166_136_261;
    const FNV_PRIME: u32 = 16_777_619;

    let mut hash = FNV_OFFSET;
    for byte in bytes {
        hash ^= u32::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash.max(1)
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
        assert_eq!(
            physical_key_code(&PhysicalKey::Unidentified(NativeKeyCode::Xkb(77))),
            77
        );
    }
}
