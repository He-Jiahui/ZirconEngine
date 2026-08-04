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

| 日期 | 里程碑/切片 | 状态 | 产出 | 验证 | 后续 |
|------|-------------|------|------|------|------|
| 2026-08-03 | IM-M1 decoration source-map 复用 + IM-M2 preedit/visual-caret 前向修复 | implementation_complete_secondary_static_review_green_validation_deferred | `text_geometry` 先收集预编辑高亮、selection 与 clause underline 的 range 规格，再为每个 resolved line 仅构建一次 `UiTextLineSourceMap`，复用 cluster 投影与 exact-advance cache。命中测试在有效 resolved glyph artifact 已反查 source caret 时不再构建 allocation-backed source map，仅在工件缺失或过期时走同一 source-map 回退。新的 `CompositionHighlight` 使用完整文本 frame 和半透明 `#4d89ff24`；规格顺序稳定为高亮、selection、每个 clause underline、caret，GPU 计划将高亮/selection 放在文本前、underline/caret 放在文本后。RTL isolate visual hit 保留 `UiTextHitTest.affinity`，并通过 metadata `caret_affinity` 经 surface input、focus-loss、component reducer、render extract 和 IME `UpdateCursor` 贯通；逻辑键盘、语义选区与辅助功能选区显式写入 downstream。TextField/textarea schema 声明默认 downstream；app host 仅将有限、非负尺寸的 ABI cursor area 提交给 winit。D5 `target_viewport` 绑定宿主、`WindowLogical` 固定逻辑像素，resolved line frame 已是绝对 surface 坐标。 | scoped Rust 2021 `rustfmt --check` 与 scoped `git diff --check` 通过（仅 LF/CRLF 提示）；二次静态审查复核新增 kind 分派、pre/post-text 批次、真实度量 frame、绘制顺序、source-map 单次构建、artifact fast path、全部 `caret_offset` 状态 writer、RTL isolate affinity 在真实布局、metadata 和 render extract 中的回归、vertical-rl `UpdateCursor` 与 app-host 非法 cursor area guard，P0/P1=0。未按 Runtime04 receipt 约束启动 Cargo/WGPU；未生成或声称纯文本策略截图。 | 协调器在受管 Windows 车道运行 clause/IME focused Cargo 回归及真实产品 framebuffer，PNG 仅写入 `docs/tests/runtime/text`；平台 TSF/IMM32/IBus clause 生产填充与候选窗实机锚定仍 pending。 |
| 2026-08-03 | IM-M1 clause 共享状态 owner 二次审查前向修复 | implementation_complete_secondary_static_review_green_validation_deferred | 二次审查发现 component reducer 从 `UiComponentState` 重建 `UiTextComposition` 时固定清空 clause，现已前向修复。`ui/editable_text_composition.rs` 是 TOML 与 `UiValue` 的唯一无状态编解码/UTF-8 校验 owner；surface input、render resolve 与 component reducer 共同使用，旧 surface 私有副本已硬删除。TextField/textarea state schema 显式声明默认空数组 `composition_clauses`；组件状态的多字节 CJK clause 经读取、编辑模型和回写后保持逐字节范围及类型。 | scoped Rust 2021 `rustfmt --check` 与 scoped `git diff --check` 通过（仅 LF/CRLF 提示）；共享 owner 无 surface/dispatch 反向依赖、旧 owner 引用为零、所有 `UiTextComposition` 构造点显式处理 clause、catalog schema 静态检查通过。未按 Runtime04 receipt 约束启动 Cargo/WGPU；未生成或声称纯文本策略截图。 | 协调器在受管 Windows 车道执行 focused interface/runtime/component Cargo 回归及真实产品 framebuffer 截图，PNG 仅写入 `docs/tests/runtime/text`；平台 TSF/IMM32/IBus clause 生产填充、候选窗实机锚定、window id/scale、RTL isolate affinity、backend cluster reverse lookup 与竖排 IME geometry 仍 pending。 |
| 2026-08-03 | IM-M1 preedit clause 契约、状态传播与标记绘制 | implementation_complete_static_review_green_validation_deferred | `UiImeInputEvent` / `UiWindowPlatformInputEvent` 具备 `Input`、`Converted`、`TargetConverted`、`TargetNotConverted` 四类 UTF-8 byte-range clause；构造与运行时路由拒绝非 Preedit、越界、非 UTF-8 边界、逆序及重叠 payload，winit 基线路径保持空 clause。`UiTextByteRange`、clause、错误与无状态校验归属于 surface/render DTO，既有 `UiIme*` 为 dispatch 兼容别名，因此 render 与共享状态不反向依赖输入事件层。有效 clause 通过 `UiTextComposition`、`composition_clauses` 结构化属性和 render extract 保留；surface 共享叶子 `editable_text_composition.rs` 是该属性的唯一 TOML/`UiValue` 编解码 owner，输入和渲染不再各自解释协议。文本约束改写 preedit 时保守清空 clause，commit/focus-loss 清空持久化 clause。文本绘制按 clause 将组合下划线投影为独立 range/color，现有无 clause 组合态保持单一兼容下划线。新增窗口输入、运行时 render extract 和 interface decoration 回归，覆盖序列化、无效载荷不变性、结构化状态与可见标记语义。 | scoped `rustfmt --edition 2021 --check` 通过；scoped `git diff --check` 无错误（仅 LF/CRLF 提示）；render/shared composition 的 no-dispatch 静态扫描通过，所有 `UiTextComposition` 构造点静态核对已显式处理 clause 字段。按当前 Runtime04 receipt 约束未启动 Cargo/WGPU；未生成或声称纯文本策略截图。 | 协调器在受管 Windows 车道执行 focused interface/runtime Cargo 回归及真实产品 framebuffer 截图，PNG 仅写入 `docs/tests/runtime/text`；平台 TSF/IMM32/IBus clause 生产填充、候选窗实机锚定、window id/scale、RTL isolate affinity、backend cluster reverse lookup 与竖排 IME geometry 仍 pending。 |
| 2026-08-02 | IM-M2 focus lifecycle + DeleteSurrounding | implementation_complete_review_green_validation_deferred | `UiSurface::focus` 成为焦点到 IME 生命周期的唯一状态 owner，`input/effect/ime_lifecycle.rs` 仅排出已入队的结果，普通可编辑节点获焦写入 `Enable`，secure 节点只排出 `Disable`；modal 重定向以实际焦点为准。失焦、隐藏、禁用、无效 owner 与节点回收均在结构修改前经过同一 helper 提交 preedit、排出 `Commit` 与 `Disable`；`UiInputManager::tick` 同批清空延迟的 component/host 生命周期，避免脱附节点上的 stale Commit。`DeleteSurrounding` 按完整 grapheme 扩张局部 byte range、以 committed 文本恢复并刷新 host context，零 payload 与无效边界保持 composition。IME surrounding text 保持每侧最多 256 grapheme、字节上限及 secure fail-closed；测试按职责拆分为 `widget_text_input_ime_context.rs` 与 `widget_text_input_ime_context/focus_lifecycle.rs`，未引入 interface ABI、renderer 特判或平台 API 判断。 | scoped `rustfmt --edition 2021 --check` 通过；scoped `git diff --check` 无错误（仅 LF/CRLF 提示）；二次静态审查最终无 P0/P1。受管 `check-conventions.ps1 -Only structure` 在编译前被共享工作区的 `Cargo.lock` 与 manifest 不一致的 `--locked` 前置条件拒绝，未运行 Cargo 测试；当前所需 `runtime_text_mvp_foundation_product_framebuffer_20260801.png` 不存在，真实产品 framebuffer PNG 仍须协调器受管 Windows 车道生成并写入 `docs/tests/runtime/text`，不得以纯文本策略图替代。 | 协调器同步锁文件后执行受管 dispatch/app-host 回归及产品 framebuffer 截图；平台候选窗实机锚定、window id/scale 契约、RTL isolate affinity、backend cluster reverse lookup 与竖排 IME geometry 仍 pending。 |
| 2026-08-02 | IM-M1 surrounding-text bounded context | implementation_complete_validation_deferred | `editable_text/ime_context.rs` 的 IME surrounding text 改为以 caret 前后各最多 256 个完整 grapheme 为初始窗口，并在 `< UI_INPUT_METHOD_SURROUNDING_TEXT_BYTE_LIMIT` 的字节上限内仅移除完整 grapheme；cursor/anchor 在 committed-text 映射后按窗口起点重基。无法表达的远端 anchor 或紧邻 caret 的超长单一 grapheme 均 fail-closed 省略 surrounding text。无 composition 的常用路径以 `Cow::Borrowed` 直接借用 `state.text`，不再为上报上下文复制整篇文本；有 composition 时仍先恢复 committed text 再映射，保持既有语义。新增 `text_input_ime_surrounding_text_is_bounded_to_graphemes_around_the_caret`、`text_input_ime_surrounding_text_trims_a_wide_grapheme_window_to_the_byte_limit`、`text_input_ime_surrounding_text_omits_a_single_grapheme_that_exceeds_the_contract_limit` 与模块内 selection/borrow 回归。 | scoped `rustfmt --edition 2021 --check` 通过；scoped `git diff --check` 无错误（仅 LF/CRLF 提示）；二次静态审查无 P0/P1。Cargo 与产品 WGPU 截图仍由协调器受管 Windows 验证车道执行，本切片不产生纯文本策略验收图。 | secure 输入的 `SetAllowed(false)` 焦点生命周期、window id/scale 契约、平台候选窗实机锚定、RTL isolate affinity、backend cluster reverse lookup 与竖排 IME geometry 仍 pending。 |
| 2026-07-04 | IM-M1 public cursor/range geometry surface | runtime_text_public_cursor_geometry_surface_rustfmt_static_visual_cargo_deferred | `ui/surface/text_geometry.rs` 新增 `text_caret_frame_for_layout(...)` 与 `text_range_frames_for_layout(...)`,把 source-range-aware caret/range geometry 作为 public surface API 暴露给已有 `UiResolvedTextLayout` 的 IME/editor callers;私有 geometry owner 仍在 `ui/text/geometry.rs`,tab/复杂映射 fallback 不变 | scoped rustfmt 通过;scoped `git diff --check` 仅 LF/CRLF 提示;尾随空白扫描通过;验证图 `docs/tests/runtime/text/runtime_text_public_cursor_geometry_surface_preview_20260704.png` SHA256 `120E15B98B1A8705A9E6B6F193E9C26D8EAFB11C72DFE2946750EC0FDBC0FD1F`;验证日志 `docs/tests/runtime/text/runtime_text_public_cursor_geometry_surface_validation_20260704.log` SHA256 `EB81F8C62A1BAEF683E54F84C85D5DA33F7DB772D0FFBE0D5AE516EF851CDA21`;同名 target/cargo-target 扫描为 0;focused Cargo 因既有 cargo/rustc lanes 活跃而 deferred | public surface 首段关闭;平台候选窗实机锚定、window id/scale 契约补全、secure surrounding text、RTL isolate affinity、backend cluster reverse lookup 与竖排 IME geometry 仍 pending |
| 2026-07-04 | IM-M1 source-range caret/composition geometry consumer | runtime_text_ime_source_range_geometry_rustfmt_static_visual_cargo_deferred | `ui/text/geometry.rs` 新增带 source text/style 的 caret 与 range frame 入口,`ime_context.rs` 的 `InputMethodTextLayout` 保留 resolved style 并传入 `state.text`,让简单 LTR IME 光标/组合矩形直接走 shaped source-range width;tab/justify/rich/non-isomorphic 映射继续使用 resolved `glyph_advances` fallback | scoped rustfmt 通过;scoped `git diff --check` 仅 LF/CRLF 提示;尾随空白扫描通过;验证图 `docs/tests/runtime/text/runtime_text_ime_source_range_geometry_preview_20260704.png` SHA256 `138A6385C2A0B4A6AC20DCEB649ACEF4F855D09BDED310ACB060C36F0C04A32A`;验证日志 `docs/tests/runtime/text/runtime_text_ime_source_range_geometry_validation_20260704.log` SHA256 `DDF9395D4B8F670E486E4297B0DC287DD9B05102FC22BCBAE54F7AC290226569`;同名 target/cargo-target 扫描为 0;focused Cargo 因既有 cargo/rustc lanes 活跃而 deferred | IME 简单 LTR source-range 几何首段关闭;平台候选窗实机锚定、window id/scale 契约补全、secure surrounding text、RTL isolate affinity、backend cluster reverse lookup 与竖排 IME geometry 仍 pending |
| 2026-06-30 | IM-M1 app host IME request pump source guard slice | runtime_text_im_m1_app_host_ime_pump_static_passed_cargo_timeout_no_result | 新增 `zircon_app/src/entry/tests/runtime_entry_source_guards/host_requests.rs`，锁定生产 app host pump 的源码顺序：`frame_loop.rs` 在 runtime tick 后、redraw 前调用 `apply_runtime_host_requests`；`host_requests/drain.rs` 从 `RuntimeSession::drain_host_requests()` 取 batch 并逐条 route；`host_requests/routing.rs` 把 `ZrRuntimeHostRequestV1::Ime` 交给 `apply_runtime_ime_host_request(window, request)`；`host_requests/ime/request.rs` 把 `SetCursorArea` / `SetSurroundingText` 转成 winit `ImeRequest::Update`；`geometry.rs` 保持 cursor-area logical position/size 转换 owner。 | `rustfmt --edition 2021 --check zircon_app\src\entry\tests\runtime_entry_source_guards\mod.rs zircon_app\src\entry\tests\runtime_entry_source_guards\host_requests.rs` 通过；scoped `git diff --check` 仅 LF/CRLF 提示。focused `cargo test -p zircon_app runtime_entry_applies_runtime_ime_host_requests_from_session_drain --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-app-ime-host-pump-source-0630 --message-format short --color never -- --nocapture --test-threads=1` 904 秒超时且无 Rust diagnostics，owned cargo/rustc 进程已停止，未计通过。验证图 `docs/tests/runtime/text/runtime_text_ime_app_host_pump_preview_20260630.png` 已检查，SHA256 `365D2DA267707FE7E524C2BE4661B7BD99D4D2AECEF7D4865CCD08581853880A`，repo `target` 与 `E:\cargo-targets` 同名匹配为 0。 | app host pump 源码结构链路已锁定；仍需补 `zircon_app` focused guard 绿跑、平台候选窗实机锚定、backend cluster reverse lookup、RTL affinity 与竖排 IME geometry。 |
| 2026-06-30 | IM-M1 dynamic session IME host-request drain contract slice | runtime_text_im_m1_dynamic_ime_host_request_drain_static_passed_focused_test_timeout | 新增 `dynamic_api/tests/host_requests.rs::dynamic_session_drains_runtime_ime_cursor_area_and_surrounding_text_requests_once`，锁定 `RuntimeDynamicSession` 通过 ABI `ImeCursorArea` / `ImeSurroundingText` 事件进入 `DefaultInputManager` 后，`drain_host_requests` 输出 `ZrRuntimeHostRequestV1::Ime(SetCursorArea)` 与 `SetSurroundingText`，并确认第二次 drain 为空。生产路径审计确认 `RuntimeDynamicSession::drain_host_requests` 已 drain input manager 的 IME/gamepad/cursor 队列，`zircon_app` host-request routing 已有 IME apply 入口；本切片不把测试专用 `RuntimeUiManager` 推到生产路径。 | `rustfmt --edition 2021 --check zircon_runtime\src\dynamic_api\tests\host_requests.rs zircon_runtime\src\dynamic_api\tests\support.rs` 通过；`git diff --check -- zircon_runtime/src/dynamic_api/tests/host_requests.rs zircon_runtime/src/dynamic_api/tests/support.rs` 仅 LF/CRLF 提示。focused `cargo test -p zircon_runtime dynamic_session_drains_runtime_ime_cursor_area_and_surrounding_text_requests_once --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0630-ime-dynamic-host-drain --message-format short --color never` 两次在 Windows lib-test 编译阶段超时且无 Rust diagnostics，受并行 Cargo 编译车道影响，未计为通过。验证图 `docs/tests/runtime/text/runtime_text_ime_dynamic_host_drain_preview_20260630.png` 已检查，SHA256 `C7CB01D3DB297A34B12AD35BE56B210EFA17CB4858C2A95D37D8DE7F81D66ACA`，repo `target` 与 `E:\cargo-targets` 同名匹配为 0。 | dynamic session ABI drain contract 首段已锁定；full production UI pump 到 app host-request 的端到端运行链、平台候选窗实机锚定、backend cluster reverse lookup、RTL affinity 与竖排 IME geometry 仍 pending。 |
| 2026-06-30 | IM-M1 composition-end cursor area host-request route slice | runtime_text_im_m1_cursor_area_host_request_route_focused_tests_passed | 新增 `ui/dispatch/input_manager/ime_host_requests.rs` 作为 UI input-method host-request 到 runtime `ImeHostRequest` 的 leaf owner,把 `UpdateCursor.cursor_rect` 转为 `ImeHostRequest::SetCursorArea`、把已校验 surrounding text 转为 `ImeHostRequest::SetSurroundingText`。`UiInputManager` 记录 input/window/tick dispatch 结果中的 IME host requests,新增 `drain_ime_host_requests()`；`widget_text_input_ime_context.rs::text_ime_cursor_area_anchors_at_composition_end` 断言软换行 preedit `"WXYZQ"` 替换 `"bcde"` 后,候选窗锚点使用 composition-end caret(offset 6) 的真实 resolved rect,并确认队列只 drain 一次。 | `rustfmt --edition 2021 --check zircon_runtime/src/ui/dispatch/input_manager/ime_host_requests.rs zircon_runtime/src/ui/dispatch/input_manager/manager.rs zircon_runtime/src/ui/dispatch/input_manager/mod.rs zircon_runtime/src/ui/tests/widget_text_input_ime_context.rs` 通过；`cargo test -p zircon_runtime text_ime_cursor_area_anchors_at_composition_end --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0630-ime-cursor-area-route --message-format short --color never` 通过 1/1；`cargo test -p zircon_runtime text_input_ime --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0630-ime-cursor-area-route --message-format short --color never` 通过 8/8(含先前 tab-resolved cursor rect 回归)。验证图 `docs/tests/runtime/text/runtime_text_ime_cursor_area_route_preview_20260630.png` 已检查,SHA256 `B9CBC75E424466F4CD27FDA0C2DEB705B43E6F08E7B960402E9A16C471E2DC65`,repo `target` 与 `E:\cargo-targets` 同名匹配为 0。 | composition-end cursor area 的 runtime UI manager 中立 host-request 路由已关闭；平台实机候选窗锚定、production UI pump 到 app host request 的完整运行链、backend cluster reverse lookup、RTL affinity 与竖排 IME 几何仍 pending。 |
| 2026-06-30 | IM-M1 / IM-M2 explicit composition source range + wrapped layout rect slice | runtime_text_im_m1_m2_composition_source_range_soft_wrap_focused_tests_passed | `ui/text/edit_state.rs` 不再把显式 `SetComposition` source range 按 preedit 长度外扩；连续 preedit 更新会先恢复原始 source span 再重用恢复后的 source range，避免 `"abcdef"` 中 `bcde` 被 `"WXYZQ"` 替换时吞掉尾部 `f`。`editable_text/ime_context.rs` 在 editable state 已经包含 preedit 文本后不再二次 `with_preedit(...)`，cursor rect 与 composition rects 直接来自当前 resolved layout。`widget_text_input_ime_context.rs` 新增软换行 composition rect 回归，断言 layout 行区间为 `0..3` / `3..7` 且组合范围 `1..6` 跨行。 | `rustfmt --edition 2021 --check` 覆盖 IME/edit-state/mesh 支撑文件通过；`cargo test -p zircon_runtime set_composition_ --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0630-ime-composition-range --message-format short --color never -- --nocapture --test-threads=1` 通过 2/2；`text_input_ime_preedit_rects_follow_soft_wrapped_composition_range` 通过 1/1；`prepared_queue_stats_count_gpu_morphed_sources_as_dynamic_geometry` 通过 1/1；`cargo check -p zircon_runtime --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0630-ime-geometry-check-final2 --message-format short --color never` 通过(既有 warnings)。验证图 `docs/tests/runtime/text/runtime_text_ime_composition_range_soft_wrap_preview_20260630.png` 已检查，SHA256 `DAA61025233DD0F684884573459D85B86AA86BE673B1EEBAD7AC68F8A5AE61AA`，repo `target` 与 `E:\cargo-targets` 同名匹配为 0。 | 显式 composition range、连续 preedit 更新与软换行 IME rect 首段已关闭；平台候选窗实机锚定、composition-end cursor area 端到端路由、backend cluster reverse lookup、RTL affinity 与竖排 IME 几何仍 pending。 |
| 2026-06-30 | IM-M1 / LB-M1 public IME cursor rect resolved geometry slice | runtime_text_lb_m1_ime_cursor_rect_resolved_geometry_check_passed_focused_test_blocked | `ui/text/geometry.rs` 成为 runtime resolved source/visual geometry owner,`ime_context.rs` 在 `UpdateCursor` 时优先重算 `UiResolvedTextLayout` 并用该 owner 生成 cursor/composition rects；旧 fixed-column estimate 仅保留为 layout 不可用 fallback。`resolve_style` 只在 `crate::ui::surface` 内部复用,没有新 public facade 或第二套 style parser。 | scoped rustfmt 通过；`cargo check -p zircon_runtime --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0630-ime-geometry-check --message-format short --color never` 通过(既有 warnings)。focused `text_input_ime_cursor_rect_uses_resolved_tab_advances` lib-test 首跑被无关当前 `morph_payload_upload.rs:274` moved-value 编译错误阻断；最终复跑在 Windows lib-test 编译阶段 304s 超时且无 Rust diagnostics,两次均未进入断言,不计测试通过。验证图 `docs/tests/runtime/text/runtime_text_ime_cursor_tab_preview_20260630.png` 已检查,SHA256 `486AF719C9CF9F447AA8A75166C546F24EB84C9337E5BBE5329EEA57DD9BA1B9`,repo `target` 与 `E:\cargo-targets` 同名匹配为 0。 | IM-M1 真实度量光标首段已接入;仍需平台候选窗实机锚定、composition-end cursor area 端到端路由、backend cluster reverse lookup、RTL affinity 与竖排 IME 几何。 |
| 2026-06-27 | 计划建立 | planned | 中立 IME 契约补全 + 真实度量光标 + 多平台后端抽象 + 候选窗锚定路线 | 文档 | IM-M1 真实度量光标;依赖 03 度量 |
