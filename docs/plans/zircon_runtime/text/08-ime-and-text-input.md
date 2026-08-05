---
related_code:
  - zircon_runtime/src/ui/surface/input/editable_text/ime_context.rs
  - zircon_runtime/src/ui/surface/input/effect/ime_lifecycle.rs
  - zircon_runtime/src/ui/surface/input/effect.rs
  - zircon_runtime/src/ui/surface/input/text_state.rs
  - zircon_runtime/src/ui/surface/text_geometry.rs
  - zircon_runtime/src/ui/surface/input/editable_text/mutation.rs
  - zircon_runtime/src/ui/surface/input/text_keyboard/edit_actions.rs
  - zircon_runtime/src/ui/surface/input/text_keyboard/clipboard.rs
  - zircon_runtime/src/ui/text/geometry.rs
  - zircon_runtime/src/ui/text/edit_state.rs
  - zircon_runtime/src/ui/text/grapheme.rs
  - zircon_runtime/src/ui/dispatch/input_manager/ime_host_requests.rs
  - zircon_runtime/src/ui/dispatch/input_manager/manager.rs
  - zircon_runtime/src/core/framework/input/ime.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_runtime/src/dynamic_api/session/host_requests.rs
  - zircon_runtime/src/dynamic_api/tests/host_requests.rs
  - zircon_runtime/src/dynamic_api/tests/support.rs
  - zircon_app/src/entry/runtime_entry_app/frame_loop.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/drain.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/ime/request.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/ime/geometry.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/host_requests.rs
  - zircon_runtime/src/ui/tests/widget_text_input_ime_context.rs
  - zircon_runtime_interface/src/ui/dispatch/input/effect.rs
  - zircon_runtime_interface/src/ui/dispatch/input/event.rs
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
status: in_progress
---

# 08 多平台 IME 输入法接口

> 本计划把文本输入(尤其 CJK/复杂文种组合)从"硬编码度量光标 + 候选窗未实机"收敛为**多平台 IME 接口 + 真实度量光标定位**。承接 `editor_ui/03 §2.2` 缺口 4(IME 组合链不闭环)。CJK 输入是一等公民。

## 1. 目标

1. **中立 IME 契约(已部分存在,补全)**:出站 `UiInputMethodRequest`(光标矩形/周围文本/启停)、入站 `UiImeInputEvent`(preedit/commit/cursor range);本计划补**真实度量光标矩形**(替换硬编码 `font_size*0.6`)与多行环绕定位。
2. **多平台后端抽象**:`zircon_app` 平台层实现 Windows TSF/IMM32、macOS NSTextInputClient、Linux IBus/fcitx(经 winit IME 事件 + 平台扩展)、Web composition;运行时只持中立契约,不绑平台 API。
3. **组合(preedit)链闭环**:set/commit/cancel composition;preedit span 注入布局(下划线/高亮)、组合段光标、候选窗定位锚点;ABI disabled → Cancel。
4. **编辑链完整**:插入/删除/选区/移动(grapheme/word/line 边界,已有)+ 剪贴板 + 撤销重做(已有);与 IME 组合状态正确交互。

## 2. 现状与差距

- `ui/surface/input/editable_text/ime_context.rs`: `UpdateCursor` 请求(光标位置/合成范围/周围文本)、preedit/commit 处理在;2026-06-30 首段已优先按当前 editable state 重算 `UiResolvedTextLayout`,并通过 `ui/text/geometry.rs` 用 resolved `glyph_advances` 计算光标/合成 rect。2026-07-04 后简单 LTR、非 tab/justify/ellipsis、source-isomorphic line 会优先用 source-range shaped width 计算 IME caret/composition rect,旧 `char_advance = font_size*0.6` 只作为 layout 不可用 fallback;同日 `ui/surface/text_geometry.rs` 暴露 public caret/range geometry surface,后续 editor/runtime callers 可不再导入私有 `ui::text`。
- `ui/dispatch/input_manager/ime_host_requests.rs`:2026-06-30 已把 UI dispatch 的 `UiInputMethodRequest` 转成 runtime 中立 `ImeHostRequest::{SetCursorArea,SetSurroundingText,Enable,Disable}` 并由 `UiInputManager::drain_ime_host_requests()` 一次性取走;`SetCursorArea` 使用 composition-end cursor rect。
- `edit_state.rs`:`SetComposition`/`CommitComposition`/`CancelComposition` + Insert/Backspace/Delete/MoveCaret/SetSelection 完整。
- `text_keyboard/{edit_actions,clipboard}.rs`:Ctrl+Z/Y/X/C/V/A、剪贴板在。
- iface `dispatch/input/effect.rs`:`UiInputMethodRequest(Kind)`、`UiInputMethodSurroundingText`;`event.rs`:`UiImeInputEvent`(含 cursor range)。
- 缺口:**平台候选窗实机定位未完成**(editor_ui/03 缺口 4);多平台后端未抽象统一。后端簇反查已由 text-owned resolved glyph artifact 接入 UI 命中测试，vertical-rl `UpdateCursor` 已消费 resolved-layout geometry；RTL isolate/level visual hit 的 caret affinity 已经指针、持久化编辑状态、component reducer、render extract 与 `UpdateCursor` 贯通，逻辑键盘/辅助功能选区明确归一为 downstream。app host 对 ABI 中非有限或负尺寸 cursor area fail-closed。剩余为平台 TSF/IMM32/IBus clause 生产填充与候选窗实机锚定。

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

- runtime:`ui/surface/input/editable_text/ime_context.rs` 光标/rect 改调 `text/layout::measured_width` + 行度量(经 service);`edit_state.rs`/`mutation.rs` 组合状态机不变,只改度量来源。
- iface:`ui/dispatch/input/effect.rs` 的 `UiInputMethodRequest` 补 `SetCursorArea { rect: UiRect }`(若缺);`UiInputMethodSurroundingText` 补组合范围。
- `zircon_app`:`entry/runtime_entry_app/host_requests/` 增 IME 路由 owner 叶子 `ime/{cursor_area,allowed,surrounding}.rs`——winit ↔ 中立契约翻译;平台特化在 `zircon_app` 平台子模块(不入 runtime)。

### 中立契约(iface,补全)

```rust
pub enum UiInputMethodRequest {
    SetAllowed(bool),                       // focus TextField → true
    // (2026-07-02 评审收口,D5)契约定稿:rect 为窗口相对、逻辑像素;必须携带 window id
    SetCursorArea { window: WindowId, rect: UiRect /* 窗口相对、逻辑像素 */ },
    SurroundingText(UiInputMethodSurroundingText),
}
pub struct UiInputMethodSurroundingText { pub text: String,
    pub cursor: u32, pub anchor: u32, pub composition: Option<(u32,u32)> }
pub enum UiImeInputEvent {
    Enabled, Disabled,
    Preedit {
        text: String,
        cursor: Option<(u32,u32)>, // 组合段内光标 byte range
        // (2026-07-02 评审收口)可选 clause 分段:winit 基线路径为空 Vec,
        // TSF/平台扩展路径填充;preedit 按 clause 分下划线样式
        clauses: Vec<(u32, u32, PreeditClauseKind)>,
    },
    Commit { text: String },
}
// (2026-07-02 评审收口)clause 类型(对齐 TSF ATTR_* / IMM32 ATTR_*):
pub enum PreeditClauseKind { Input, Converted, TargetConverted, TargetNotConverted }
```

(2026-07-02 评审收口,D5)`SetCursorArea` 坐标语义定稿:rect 为**窗口相对、逻辑像素**;变换链 = caret 布局局部坐标 → 累加 UI root 偏移(含滚动/嵌套容器变换)→ 窗口逻辑坐标;DPI 换算不在契约内做——`zircon_app` 把逻辑坐标交 winit `Position::Logical`/`Size::Logical`,由 winit Logical 语义承担 scale。当前 dynamic ABI 已以 `target_viewport` 绑定宿主，`ZrRuntimeImeCoordinateSpaceV1::WindowLogical` 固定该逻辑像素含义；runtime 序列化 viewport，app-host 拒绝错误目标。不存在应由文本链重复累加的 scroll/root 偏移，也不应增加会与逻辑像素语义冲突的 raw scale 字段。

(2026-07-02 评审收口)**surrounding text 窗口约束**:超长文本不整串上报——取光标前后各 **≤256 grapheme** 的窗口,`cursor`/`anchor`/`composition` offset 以**窗口起点为基准**重排(契约需注明 offset 基准是窗口而非全文)。**secure 输入**:TextField 增 `secure: bool` 标志(密码框)——secure 时对 IME 发 `SetAllowed(false)` 且**不发 surrounding text**(防输入法进程读取密码上下文)。

### 光标矩形(真实度量,替换硬编码)

```
组合段末光标 → 行 line_index + 段末 byte offset →
  caret_x = line.origin.x + measured_width(run, line_start_byte, caret_byte, true)
  caret_rect = { x: caret_x, y: line.origin.y, w: 1, h: line.ascent + line.descent }
  → 转屏幕坐标 → SetCursorArea(caret_rect)(候选窗出现在光标下方)
```
多行环绕:caret_byte 落哪行由 `03` 行区间反查;竖排时 x/y 语义对调。

(2026-07-02 评审收口,D5)光标矩形变换链定稿:上式 `caret_rect` 为文本布局局部坐标,出站前必须累加 **UI root 偏移**(含父容器/滚动偏移)转为**窗口相对逻辑像素**,再进 `SetCursorArea { window, rect }`;不做物理像素换算(DPI 由 winit Logical 语义承担)。补测试 `text_ime_cursor_area_respects_scale_factor`(scale 1.0/2.0 下逻辑 rect 不变,winit 侧 Logical 提交)。

### IME 事件翻译职责表(2026-07-02 评审收口)

| 职责 | 归属 | 说明 |
|------|------|------|
| winit 基线入站翻译(`Ime::{Enabled,Preedit,Commit,Disabled}` → `UiImeInputEvent`) | zircon_runtime `ui/platform_input`(editor_ui/01 拥有) | 跨平台基线,一处翻译 |
| 平台特化入站(TSF/IMM32/IBus/fcitx clause、候选列表等扩展) | zircon_app 平台层(本计划 IM-M2) | 填充 `Preedit.clauses` 等扩展字段 |
| 出站 host request 应用(`SetCursorArea` → `set_ime_cursor_area` 等) | zircon_app 平台层(本计划 IM-M2) | 现有 `host_requests/ime/*` 即此归属 |
| iface dispatch DTO 变更(`UiImeInputEvent`/`UiInputMethodRequest` 字段) | 与 editor_ui/01 协同,**一次合并** | 避免两计划各改一半契约 |
| focus→IME 生命周期 | runtime dispatch(焦点系统) | 焦点进入可编辑节点 → `SetAllowed(true)`+anchor rect;离开 → commit 当前 preedit → `SetAllowed(false)` |
| popup 抢焦与 Esc 次序 | runtime dispatch | popup 抢焦期间 Esc **先取消组合再关 popup**(组合活跃时 Esc 被 IME 消费) |

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
| `ime_context.rs` `char_advance = font_size*0.6` 光标/rect | 2026-06-30 首段已优先改调 `03` resolved layout geometry;硬编码常量降为 layout 缺失 fallback |
| preedit span 基于硬编码度量 | 合成范围 rects 已优先基于 resolved layout geometry;完整平台候选锚定/竖排仍需后续验收 |

### 测试与验收清单

| 测试 | 断言 |
|------|------|
| `text_input_ime_cursor_rect_uses_resolved_tab_advances` | `a\tb` caret byte offset 2 的 IME cursor rect 使用 resolved tab advance,非 `len*size*0.6` |
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

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

当前状态：Text08 的 IME/文本输入实现与验证仍按既有状态推进；历史里程碑证据已整体迁入编号归档，未改变任何完成、延期或待验证结论。

- [2026-06-27 至 2026-08-03 IME 里程碑产出记录](08/2026-08-05-ime-milestone-output-records.md)
