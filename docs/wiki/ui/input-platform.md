---
related_code:
  - zircon_runtime/src/ui/platform_input/mod.rs
  - zircon_runtime/src/ui/platform_input/winit_translation.rs
  - zircon_runtime/src/ui/surface/input/window_pump.rs
  - zircon_runtime/src/platform/capability/matrix/platform_capability_matrix.rs
  - zircon_runtime_interface/src/ui/window/pump.rs
implementation_files:
  - zircon_runtime/src/ui/platform_input/keyboard_map.rs
  - zircon_runtime/src/ui/surface/input/text_keyboard.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine UI Wiki
tests:
  - zircon_runtime/src/ui/tests/runtime_window_input_pump.rs
  - zircon_runtime/src/ui/platform_input/winit_translation/keyboard.rs
  - zircon_runtime/src/platform/tests/headless_synthetic_input.rs
doc_type: module-detail
---

# 输入与平台

## 平台转换

启用 `platform-winit` 后，`translate_winit_window_event` 将 winit 窗口、指针、键盘、IME、滚轮和拖放事件转换为接口层 `UiWindowPlatformInputEvent`；`translate_winit_modifiers` 统一 Ctrl/Command、Alt/Option 等修饰键。`keyboard_map` 将物理键映射到引擎键码。

## 输入泵与效果

窗口输入泵按窗口顺序读取 `UiWindowInputPumpEvent`，调用表面路由并应用 `UiSurfaceInputEffectResult`：焦点、指针捕获、滚动、弹窗、工具提示、typeahead、toast、剪贴板和 IME 光标矩形。文本键盘路径区分 committed text、preedit 和 selection mutation。

```rust
use zircon_runtime::ui::platform_input::translate_winit_window_event;
let pump_event = translate_winit_window_event(window_input_context, &winit_event);
// UiRuntimeDriver 的窗口输入泵随后消费 pump_event 并更新目标 UiSurface。
```

## 能力矩阵

`PlatformCapabilityMatrix`/`PlatformRuntimeCapabilityReport` 报告窗口、输入、IME、光标、触摸、手柄、拖放和事件循环能力；宿主通过 `PlatformHostService` 与 `HostCommandBroker` 执行窗口命令。无能力时事件被标记为 unavailable，而不是伪造成功。

## 限制

平台后端受 Cargo feature 和目标系统约束；headless 测试只能使用 synthetic input。IME 事件必须绑定当前焦点文本节点，失焦后旧 preedit 会被丢弃。
