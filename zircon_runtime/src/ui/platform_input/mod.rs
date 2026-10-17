//! Platform UI input adapters owned by the runtime UI subsystem.
//!
//! 将平台载荷编码为共享窗口泵事件；表面状态和事件顺序由宿主及后续分发器持有。
//! 本模块不保存输入会话，只在 `ui` 与 `platform-winit` 同时启用时公开。

#[path = "keyboard_map.rs"]
mod keyboard_map;
#[path = "winit_translation.rs"]
mod winit_translation;

pub use winit_translation::{translate_winit_modifiers, translate_winit_window_event};
