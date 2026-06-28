---
related_code:
  - zircon_runtime/src/ui/surface/input/editable_text/ime_context.rs
  - zircon_runtime/src/ui/surface/input/editable_text/mutation.rs
  - zircon_runtime/src/ui/surface/input/text_keyboard/edit_actions.rs
  - zircon_runtime/src/ui/surface/input/text_keyboard/clipboard.rs
  - zircon_runtime/src/ui/text/edit_state.rs
  - zircon_runtime/src/ui/text/grapheme.rs
  - zircon_runtime_interface/src/ui/dispatch/input/effect.rs
  - zircon_runtime_interface/src/ui/dispatch/input/event.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs
design_references:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/PlatformTextField.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/GenericPlatformTextField.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/Mac
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/Linux
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/IOS
  - dev/godot/servers/text/text_server.h
plan_sources:
  - docs/plans/zircon_runtime/text/index.md
  - docs/plans/zircon_runtime/text/03-line-breaking-measure-and-layout.md
  - docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md
status: planned
---

# 08 多平台 IME 输入法接口

> 本计划把文本输入(尤其 CJK/复杂文种组合)从"硬编码度量光标 + 候选窗未实机"收敛为**多平台 IME 接口 + 真实度量光标定位**。承接 `editor_ui/03 §2.2` 缺口 4(IME 组合链不闭环)。CJK 输入是一等公民。

## 1. 目标

1. **中立 IME 契约(已部分存在,补全)**:出站 `UiInputMethodRequest`(光标矩形/周围文本/启停)、入站 `UiImeInputEvent`(preedit/commit/cursor range);本计划补**真实度量光标矩形**(替换硬编码 `font_size*0.6`)与多行环绕定位。
2. **多平台后端抽象**:`zircon_app` 平台层实现 Windows TSF/IMM32、macOS NSTextInputClient、Linux IBus/fcitx(经 winit IME 事件 + 平台扩展)、Web composition;运行时只持中立契约,不绑平台 API。
3. **组合(preedit)链闭环**:set/commit/cancel composition;preedit span 注入布局(下划线/高亮)、组合段光标、候选窗定位锚点;ABI disabled → Cancel。
4. **编辑链完整**:插入/删除/选区/移动(grapheme/word/line 边界,已有)+ 剪贴板 + 撤销重做(已有);与 IME 组合状态正确交互。

## 2. 现状与差距

- `editable_text/ime_context.rs`(373 行):`UpdateCursor` 请求(光标位置/合成范围/周围文本)、preedit/commit 处理在;但光标/合成 rect 用硬编码 `char_advance = font_size*0.6`。
- `edit_state.rs`:`SetComposition`/`CommitComposition`/`CancelComposition` + Insert/Backspace/Delete/MoveCaret/SetSelection 完整。
- `text_keyboard/{edit_actions,clipboard}.rs`:Ctrl+Z/Y/X/C/V/A、剪贴板在。
- iface `dispatch/input/effect.rs`:`UiInputMethodRequest(Kind)`、`UiInputMethodSurroundingText`;`event.rs`:`UiImeInputEvent`(含 cursor range)。
- 缺口:**光标矩形非真实度量**(接 `03` 后修正);**平台候选窗实机定位未完成**(editor_ui/03 缺口 4);多平台后端未抽象统一;preedit span 已注入但基于硬编码度量。

## 3. 参考代码

| 文件 | 应重点阅读 |
|------|-----------|
| `dev/UnrealEngine/.../Framework/Text/PlatformTextField.h` + `GenericPlatformTextField.h` | `IPlatformTextField`/`FGenericPlatformTextField`:平台 TextField 抽象接口(show/hide virtual keyboard、组合)——**多平台抽象主样板** |
| `dev/UnrealEngine/.../Framework/Text/{Mac,Linux,IOS,Android}/` | 各平台 TextField 实现:Mac(NSTextInputClient)、Linux(IBus/fcitx via SDL)、iOS/Android 软键盘——**平台特化参照** |
| `dev/UnrealEngine/.../Framework/Text/ITextInputMethodSystem.h`(若有) | `ITextInputMethodContext`:`GetTextBounds`/`GetScreenBounds`/`GetSelectionRange`/`SetSelectionRange`——**TSF 风格上下文接口**(候选窗定位回调) |
| `dev/godot/servers/text/text_server.h` + DisplayServer IME | godot `DisplayServer::window_set_ime_active`/`window_set_ime_position`——候选窗定位 API 形态 |

**Rust/wgpu 落地**:winit `WindowEvent::Ime(Ime::{Enabled,Preedit,Commit,Disabled})` + `Window::set_ime_allowed`/`set_ime_cursor_area`(候选窗锚定矩形)——这是跨平台基线;平台特化(TSF 富组合、IBus)经 `zircon_app` 平台层在 winit 之上扩展。`ime_context` 出入站契约已与此对齐。

## 4. 目标架构

```
平台(TSF/IMM32/NSTextInputClient/IBus/fcitx/Web)
  ↕ winit IME 事件 + 平台扩展(zircon_app 平台层)
中立契约(iface):入站 UiImeInputEvent{Preedit{text,cursor},Commit,Enabled,Disabled}
                 出站 UiInputMethodRequest{SetCursorArea(rect),SetAllowed,SurroundingText}
  ↕
runtime: ime_context(组合状态)→ edit_state(文本变更)→
         真实度量光标矩形(03 measure)→ 候选窗锚定 rect → 出站请求
```

运行时不知道平台 API;`zircon_app` 把 winit IME 事件翻译成 `UiImeInputEvent`,把出站 `UiInputMethodRequest::SetCursorArea` 翻译成 `window.set_ime_cursor_area(...)`(候选窗跟随光标)。

## 5. 里程碑

### IM-M1 真实度量光标 / 候选窗锚定

实施切片:
1. `ime_context.rs` 光标/合成 rect 改用 `03` 真实度量(`measured_width` 子范围 + 行 baseline/ascent),替换硬编码 `font_size*0.6`;多行环绕行列正确。
2. 出站 `SetCursorArea(rect)` 锚定矩形 = 组合段末光标的屏幕矩形;`zircon_app` 路由到 `window.set_ime_cursor_area`。

测试:`text_ime_cursor_rect_uses_real_metrics`、`text_ime_cursor_area_anchors_at_composition_end`。

### IM-M2 组合链闭环 + 平台事件统一

实施切片:
1. preedit span 注入布局(下划线 + 组合段高亮 + 段内光标),基于真实度量;commit 落文本 + 清组合;cancel/ABI disabled → Cancel 清组合不落字。
2. `zircon_app` 平台层:winit IME 事件 → `UiImeInputEvent` 统一翻译;`set_ime_allowed` 跟随 TextField focus;多平台一致行为。

测试:`text_ime_preedit_span_injected_with_real_metrics`、`text_ime_commit_clears_composition`、`text_ime_disabled_cancels_composition`。

### IM-M3 多平台实机验收

实施切片:
1. Windows(微软拼音/TSF)、macOS、Linux(IBus/fcitx)候选窗定位实机;CJK 输入端到端;Web(wasm)composition(若目标含 web)。
2. 软键盘(移动/触屏,若目标含)show/hide。

测试:实机手验 checklist + `render_product_text_ime_*` 抓帧(组合下划线/候选锚定);CJK 输入回归。

## 6. 工程落地细化(实施权威)

### 模块与文件落点

- runtime:`ui/surface/input/editable_text/ime_context.rs` 光标/rect 改调 `graphics/text/layout::measured_width` + 行度量(经 service);`edit_state.rs`/`mutation.rs` 组合状态机不变,只改度量来源。
- iface:`ui/dispatch/input/effect.rs` 的 `UiInputMethodRequest` 补 `SetCursorArea { rect: UiRect }`(若缺);`UiInputMethodSurroundingText` 补组合范围。
- `zircon_app`:`entry/runtime_entry_app/host_requests/` 增 IME 路由 owner 叶子 `ime/{cursor_area,allowed,surrounding}.rs`——winit ↔ 中立契约翻译;平台特化在 `zircon_app` 平台子模块(不入 runtime)。

### 中立契约(iface,补全)

```rust
pub enum UiInputMethodRequest {
    SetAllowed(bool),                       // focus TextField → true
    SetCursorArea { rect: UiRect },         // 候选窗锚定(屏幕/窗口坐标)
    SurroundingText(UiInputMethodSurroundingText),
}
pub struct UiInputMethodSurroundingText { pub text: String,
    pub cursor: u32, pub anchor: u32, pub composition: Option<(u32,u32)> }
pub enum UiImeInputEvent {
    Enabled, Disabled,
    Preedit { text: String, cursor: Option<(u32,u32)> }, // 组合段内光标 byte range
    Commit { text: String },
}
```

### 光标矩形(真实度量,替换硬编码)

```
组合段末光标 → 行 line_index + 段末 byte offset →
  caret_x = line.origin.x + measured_width(run, line_start_byte, caret_byte, true)
  caret_rect = { x: caret_x, y: line.origin.y, w: 1, h: line.ascent + line.descent }
  → 转屏幕坐标 → SetCursorArea(caret_rect)(候选窗出现在光标下方)
```
多行环绕:caret_byte 落哪行由 `03` 行区间反查;竖排时 x/y 语义对调。

### 平台后端(`zircon_app` 平台层,非 runtime)

| 平台 | 后端 | 备注 |
|------|------|------|
| Windows | winit IME(TSF/IMM32 底层)+ 必要时 TSF 扩展 | 微软拼音/五笔候选窗跟随 `set_ime_cursor_area` |
| macOS | winit IME(NSTextInputClient) | 系统候选窗 |
| Linux | winit IME(IBus/fcitx via XIM/Wayland text-input) | 需用户 IM 环境;fcitx5/ibus |
| Web | winit web IME / hidden input composition | 若构建目标含 wasm |
| 移动/触屏 | 软键盘 show/hide(`set_ime_allowed`) | 若目标含 |

运行时只发 `UiInputMethodRequest`、收 `UiImeInputEvent`;平台差异全在 `zircon_app`。

### 与既有路径硬切换

| 现有 | 切换 |
|------|------|
| `ime_context.rs` `char_advance = font_size*0.6` 光标/rect | 改 `03` 真实度量;删硬编码常量 |
| preedit span 基于硬编码度量 | 基于真实度量(下划线/高亮几何正确) |

### 测试与验收清单

| 测试 | 断言 |
|------|------|
| `text_ime_cursor_rect_uses_real_metrics` | 光标 x = 真实子范围宽,非 `len*size*0.6` |
| `text_ime_cursor_area_anchors_at_composition_end` | SetCursorArea rect = 组合段末光标屏幕矩形 |
| `text_ime_preedit_span_injected_with_real_metrics` | 组合段下划线/高亮几何对真实度量 |
| `text_ime_commit_clears_composition` | commit 落文本、组合清空、光标移到末 |
| `text_ime_disabled_cancels_composition` | ABI disabled → 组合取消、不落字 |
| `text_ime_multiline_cursor_row_col_correct` | 环绕多行下组合光标行列正确 |
| `text_ime_surrounding_text_reports_composition_range` | 周围文本含组合范围 |

里程碑命令:`cargo test -p zircon_runtime text_ime --locked` + `cargo test -p zircon_app text_ime --locked`;IM-M3 实机手验 + 抓帧。

## 7. 风险与回退

- winit IME 能力跨平台不齐(Linux 依赖用户 IM 环境):基线用 winit,平台特化在 `zircon_app` 增量;Linux 无 IM 时退普通键入。
- 候选窗定位以 `set_ime_cursor_area` 为标准路径;平台不支持时尽力而为,不阻塞主链。

## 8. 状态与产出记录

| 日期 | 里程碑/切片 | 状态 | 产出 | 验证 | 后续 |
|------|-------------|------|------|------|------|
| 2026-06-27 | 计划建立 | planned | 中立 IME 契约补全 + 真实度量光标 + 多平台后端抽象 + 候选窗锚定路线 | 文档 | IM-M1 真实度量光标;依赖 03 度量 |
