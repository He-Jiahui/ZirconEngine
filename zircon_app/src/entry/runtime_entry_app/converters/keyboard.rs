//! Winit 物理键输入进入 Runtime ABI 的编号适配边界；当前回退不满足全部 Runtime/UI 命令键消费合同。
//! 回退编号沿用现有 Debug 名哈希，不应作为跨依赖版本的持久化键标识。

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

// BUG: [CR-APP-ENTRY-0018] 编辑键、方向键及 C/V/X、Enter/Tab 走 Debug 哈希，动态会话只识别有限旧编号并把其余键转为无语义名称；UI 消费语义键或旧键码，导致这些编辑/剪贴板命令不被识别；证据：input_events::keyboard_logical_key、keyboard_ime 与 text_keyboard 消费链。
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

// 保持已有回退键码，与行为测试中的历史值一致；直接格式化到 hasher 避免输入热路径分配。
fn stable_key_code(code: &KeyCode) -> u32 {
    let mut hasher = StableKeyCodeHasher::new();
    let result = fmt::write(&mut hasher, format_args!("{code:?}"));
    debug_assert!(result.is_ok());
    hasher.finish()
}

#[cfg(test)]
#[path = "tests/keyboard.rs"]
mod tests;
