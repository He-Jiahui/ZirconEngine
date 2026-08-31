---
related_code:
  - zircon_runtime/src/ui/surface/input/editable_text/ime_context.rs
  - zircon_runtime/src/ui/surface/input/effect/ime_lifecycle.rs
  - zircon_runtime/src/ui/surface/input/effect.rs
  - zircon_runtime/src/ui/surface/input/text_state.rs
  - zircon_runtime/src/ui/surface/text_geometry.rs
  - zircon_runtime/src/ui/surface/input/editable_text/mutation.rs
  - zircon_runtime/src/ui/surface/input/text_keyboard/edit_actions.rs
  - zircon_runtime/src/ui/surface/input/keyboard_clipboard.rs
  - zircon_runtime/src/ui/text/geometry.rs
  - zircon_runtime/src/ui/text/edit_state.rs
  - zircon_runtime/src/ui/text/grapheme.rs
  - zircon_runtime/src/ui/dispatch/input_manager/ime_host_requests.rs
  - zircon_runtime/src/ui/dispatch/input_manager/clipboard_host_requests.rs
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
  - zircon_app/src/entry/runtime_entry_app/host_requests/clipboard/mod.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/clipboard/platform/windows.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/ime/request.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/ime/geometry.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/host_requests.rs
  - zircon_runtime/src/ui/tests/widget_text_input_ime_context.rs
  - zircon_runtime_interface/src/ui/dispatch/input/effect.rs
  - zircon_runtime_interface/src/ui/dispatch/input/event.rs
  - zircon_runtime_interface/src/runtime_api/host/clipboard.rs
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
4. **编辑链完整**:插入/删除/选区/移动已有基础实现；剪贴板的 runtime transfer/result transaction、surface-targeted dynamic ABI 与 Windows App backend 已实现但未受管验收；Undo/Redo 已接入 bounded delta history，但产品 Runtime 仍未受管验收。所有能力必须与 IME 组合状态正确交互。

## 2. 现状与差距

- `ui/surface/input/editable_text/ime_context.rs`: `UpdateCursor` 请求(光标位置/合成范围/周围文本)、preedit/commit 处理在;2026-06-30 首段已优先按当前 editable state 重算 `UiResolvedTextLayout`,并通过 `ui/text/geometry.rs` 用 resolved `glyph_advances` 计算光标/合成 rect。2026-07-04 后简单 LTR、非 tab/justify/ellipsis、source-isomorphic line 会优先用 source-range shaped width 计算 IME caret/composition rect,旧 `char_advance = font_size*0.6` 只作为 layout 不可用 fallback;同日 `ui/surface/text_geometry.rs` 暴露 public caret/range geometry surface,后续 editor/runtime callers 可不再导入私有 `ui::text`。
- `ui/dispatch/input_manager/ime_host_requests.rs`:2026-06-30 已把 UI dispatch 的 `UiInputMethodRequest` 转成 runtime 中立 `ImeHostRequest::{SetCursorArea,SetSurroundingText,Enable,Disable}` 并由 `UiInputManager::drain_ime_host_requests()` 一次性取走;`SetCursorArea` 使用 composition-end cursor rect。
- `edit_state.rs`:`SetComposition`/`CommitComposition`/`CancelComposition` + Insert/Backspace/Delete/MoveCaret/SetSelection 完整。
- `text_keyboard/edit_actions.rs` + `keyboard_clipboard.rs`:X/C/V/A shortcut mapping 在；Undo/Redo 已进入 manager-owned bounded delta history。2026-08-28 clipboard 已补 transfer id、edit revision、typed result、stale fence、surface-targeted dynamic ABI 与 Windows `CF_UNICODETEXT` App backend；受管 Cargo、真实系统 clipboard、跨平台 backend 和故障注入仍缺，不能标记产品完成。
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

### 2026-08-24 安全文本 presentation 投影设计门

状态：`architecture_review_complete / presentation_owner_implemented / text_field_plain_integration_implemented_unvalidated / validation_pending`。`ui/text/presentation.rs` 是不持有原文的 projection owner：一 source grapheme 对一 U+2022 display grapheme、hard-line separator 保留、source/display 原子 boundary map，以及由原始 hard line 捕获的 UAX#9 resolved-level/L1 metadata；其 render editable projection 只输出 mask text、source-offset caret/selection 和无 composition 的状态。`BidiLineSignature` 可在后续软换行后按该元数据重放 L1/L2；不得切片复用 hard-line 的最终 visual order。TextField 的 plain-text 路径现已以该 owner 原子接入，但仍未经过受管 Cargo/WGPU 验证，不能标记为已验收；不能用 `text.replace(.., "•")` 作为修复，因为 UTF-8 byte range、grapheme caret/selection、UAX#9 visual order 与 glyph artifact 会随之失配。

1. `ui/text/presentation.rs` 是唯一 display owner：`UiSecureTextPresentation` 只在 surface TextField resolver 接收原文，生成“一 source grapheme 对一 U+2022 display grapheme”的 display text、display-to-source grapheme range 表，以及原始 UAX#9 的 source-free scalar resolved-level/L1 signature。signature 必须在每条最终物理行确定后重放 L1/L2；full hard-line `visual_indices` 只可用于未软换行的整行。除 editable state/input owner 外，后续 render command 不得再持有原文。
2. 不能对 bullet text 再做 `Auto` BiDi。中性 bullet 会丢掉 Hebrew/Arabic/混排的方向信息；presentation 必须把原文的 `BidiLineSignature::line_order` 提供给 itemization/shaping，令 bullet glyph 的 visual order、direction flags 与原 source cluster 对应。该签名只存方向元数据和 byte offsets，不存 source string，且能处理物理行末 whitespace/isolate 的 UAX#9 L1 reset。参考 `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Text/SlateEditableTextLayout.cpp` 的 `CreateTextOrPasswordRun`：它在 run owner 处分派 `FSlatePasswordRun`，而不是在 paint 末端改写字符串。
3. 缓存分两层：可缓存的是 `mask display + directional signature + style + width` 的 presentation template；每次 secure request 必须以当前 display-to-source table 投影出 fresh `UiResolvedTextLayout`，不得将原密码 byte ranges 或投影后的 layout 放入共享 cache。成功 path 仍可复用 shaped mask/template，错误或 deferred outcome 不可作为 geometry cache entry。
4. `ResolvedTextGlyphArtifact` 需要显式 presentation cluster map。构建时 shape 的 source 是 display text，但 glyph/source range 和 caret/hit-test 对外仍引用原文；artifact/line DTO 必须在同一 publication 中重映射，禁止让 `visual_projection::RunSourceMap` 从掩码字符串反推原 UTF-8 slice，也禁止只重映射 layout 而保留旧 artifact。
5. secure command 的 `text`、plain renderer fallback、rich-text artifact preparation、command-phase prewarm、profiling/diagnostic payload 都只能看到 display text；`UiResolvedTextLayout.editable` 不得 clone raw `UiEditableTextState`，只能携带 display `text`、source-offset caret/selection 和 `composition: None`。`UiTextPaint.composition` 同样必须为 `None`，防止 preedit/restore text 经 render DTO 序列化。caret、selection、accessibility action 和 IME geometry 仍使用原文 source offsets。secure composition/preedit 不参与 display presentation，因为 secure IME 已禁用。

实施状态：presentation/cluster-map pure owner、Latin/CJK/emoji/Arabic/RTL source-map 与 source-free `BidiLineSignature` 已实现；plain TextField 现以一次原子切换绕开直接 layout cache，以 display text 解析 fresh layout，把 signature 传给 visual-order owner，并以标记触发 command-phase display-only glyph artifact publication。`UiRenderCommand`、`UiResolvedTextLayout.editable` 和 `UiTextPaint` 只接收 mask，editable composition 为 `None`；安全字段同时强制 plain-text presentation。当前 MVP 对 VerticalRl、ellipsis、投影不变量失败，以及 `TextEdit`/`multiline` 等未接入的 secure 控件 fail-closed：它们最多发布掩码 command 和空 layout，绝不回退原文；不复用含 source range 的共享 layout。secure presentation template cache 和完整多行 secure editing 仍是后续能力。截图验收必须是实际 WGPU 帧，检查输入原文不存在于 `UiRenderCommand`、`UiResolvedTextLayout.editable`、`UiTextPaint`、artifact source buffer、render batch 和像素输出中，且仅写入 `docs/tests/runtime/text`；没有协调器授权前不生成截图或声称性能/功耗结果。

2026-08-24 security-review forward fix for the secure-presentation sub-slice:
`implementation_complete / scoped_static_validation_complete / managed_validation_pending`.

- The single-line MVP allowlist now has explicit regression coverage for both unsupported
  `TextEdit` and an otherwise supported `TextField` carrying `multiline = true`. Both cases
  publish only a mask plus an empty, overflow-clipped layout and a composition-free editable
  projection; neither can fall back to the ordinary raw multi-line route.
- The `multiline = true` regression also requires zero persistent layout-cache entries/inserts
  and zero render-command prewarm requests. This protects the invariant that a secure source
  does not become shared layout or worker-batch input after the command projection boundary.
- A complete RTL soft-wrap regression now measures the display mask, constrains it to 60% of
  that measured width, and requires every resulting physical row to replay only that row's
  source-owned UAX#9 visual indices and source ranges. This guards against accidentally applying
  a hard line's final visual order to one wrapped row.
- Scoped Rustfmt and `git diff --check` pass for the touched owners. A coordinator-managed
  Windows WGPU attempt selected only `render_text_native_bitmap_layout_product_framebuffer` and
  wrote neither `target` output nor a PNG, because `zr_rhi_wgpu` failed before the test binary
  linked on eight foreign type/import errors. Managed Cargo, real WGPU frame capture, and the
  required source-free pixel inspection therefore remain pending; this does not mark the Text 08
  plan or its platform IME milestones accepted.

2026-08-26 artifact-identity API follow-up: the secure-presentation no-source marker now registers
through `UiRichTextArtifactHandle::from_runtime_artifact_with_identity` with an explicit versioned
owner identity. Two independently allocated markers compare equal and remain distinguishable by
payload type; the deleted identity-free constructor is not restored. The owner stays in
`ui/text/presentation.rs` at 793 lines, below the 800-line review warning, and its focused regression
locks both equality and marker recognition. `layout_engine/secure_presentation.rs` imports the
registration function from the real sibling owner instead of depending on a missing `ui::text`
facade re-export. Exact leaf Rustfmt and the scoped old-constructor scan pass.
This is `compile_repair_complete / scoped_static_validation_complete /
managed_validation_pending`; no Cargo, WGPU, source-free pixel inspection, screenshot, performance,
or power result is claimed.

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

2026-08-27 输入约束基础设施更新：keyboard/text/IME/accessibility已共享typed
`UiTextInputConstraintReceipt`，filter、canonical single-line separator移除与max-grapheme截断不再
静默，CRLF计为一个separator，`max_length=0`与catalog/reducer统一为不限长。constrained preedit
现在以只保存cursor/clause实际端点的UTF-8 byte edit map重映射range，cursor再clamp到grapheme；
完全删除的非空clause有typed drop count，合法空clause保留。production winit/dynamic仍不生产
rich clause，平台UTF-16/ACP转换与实机验证继续开放。single-line Enter按Unreal合同成为零属性写的
handled Submit，repeat只消费不重复commit，不把命令误记为newline constraint rejection。状态为
`typed_constraint_receipt_implemented / constrained_preedit_edit_mapping_implemented /
single_line_enter_submit_implemented / platform_clause_producer_open /
managed_validation_pending`，不关闭IM-M2/IM-M3或平台实机门。

同日secure前置更新：TextField catalog已声明typed `input_kind` enum，WOC
`input_kind=password`与secure aliases进入唯一internal `UiSecureTextPolicy`，render/a11y/clipboard
不再由分叉classifier漏判。后续输入结果投影把secure Change/Submit改为surface-owned opaque
reference，并统一清洗Text/Keyboard/IME/a11y输入、binding、effect、host/component report与
action payload；latest token、跨surface、clone/serde与layout revision均有fail-closed fence。
WOC受信consumer、retained state/history/export/crash/zeroization、公开versioned policy与平台
secure session仍缺失；状态为`classifier_bypass_closed /
secure_event_projection_implemented_unvalidated / trusted_host_session_open /
secure_ime_session_open / managed_validation_pending`，Text08与Runtime11B安全输入P0保持open。

2026-08-28 authored route 前置更新：current-source review 确认 WOC `.zui` 已声明 password
Change/Submit route，`WocShellController` 也有真实 `Option<&str>` 消费入口，但 editable-text
Change/Submit report 原先固定 `template_action=None`，dynamic Runtime 又把完整 dispatch result 降为
handled bool，因此 opaque reference 尚无可授权的产品路由。文本事件投影现按 matching binding 发布
精确 route/action identity；compiled surface 复用 `(node,event)` 索引，热路径为 `O(k)`，无匹配 binding
不签发 secure lease。安全 redaction 继续把 action payload 清空，序列化回归禁止出现密码原文；
focus-loss SecureCommit 同样保留 authored Submit route。trusted session handle、window/seat/surface/route
联合授权、一次性 consume ABI、WOC adapter 与 secure buffer/zeroization 仍开放。审查同时确认仓库无
Runtime 外部 plaintext resolver 消费者，`UiSurface` 解析入口已硬切为 crate-local 且不保留公开 alias。
状态追加为
`authored_text_route_identity_implemented_unvalidated /
public_plaintext_resolver_closed / bounded_content_free_action_delivery_implemented_unvalidated /
dynamic_dispatch_result_action_drop_closed / generic_host_request_drop_closed_unvalidated /
binding_receipt_policy_open /
trusted_session_contract_open / managed_validation_pending`。

同日后续 action delivery 前置已把 pointer、keyboard/text/IME、clipboard result 与
accessibility 的 `UiInputDispatchResult.component_events[].template_action` 统一收集到 typed
`ZrRuntimeHostRequestV1::UiAction`。请求携带 viewport/surface/tree/node/input sequence/action
ordinal、authored invocation 与可选 opaque secure reference，不携带安全明文；安全 payload
入队前必须全为 `Null`，否则拒绝并立即撤销最新 lease。队列共享 host page 的 256-row 上限，
另有 240 KiB aggregate、64 KiB per-row 与 JSON depth reserve；Secure Change 只合并同字段同
route 的待发项，Submit 保持 FIFO。现有 prepare/commit/rollback 提供重试一致性。generic App
没有产品 route adapter，只校验 viewport 并按 1/2/4/... 次数输出不含 payload/reference 的
诊断；WOC trusted registration/consume 仍开放。本切片不改变渲染，因此不生成截图。

同日 generic host reply 基础设施继续对齐 Unreal `FReply -> SlateApplication::ProcessReply`：
`UiInputDispatchResult.host_requests` 中 pointer lock/unlock、high-precision pointer、popup、tooltip、
dismiss transient UI 与 parser-approved link activation 现投影为 typed
`ZrRuntimeHostRequestV1::UiHost`，保留 viewport/surface/tree/input sequence/request ordinal 与
effect index。IME 和 clipboard 仍留在各自事务队列，不重复投影；内部动态 `reason` 不跨 ABI。
Runtime 队列共享 256-row、64 KiB per-row 与 240 KiB aggregate 边界，App 穷尽匹配并校验
viewport；尚无产品 adapter 时仅按 1/2/4/... 输出固定 kind，不格式化 href、popup/tooltip id 或
tree id。Runtime10 host-request 结构清单由 52 更新为 60 个全存在锚点，`runtime_ui.rs` 的四类 drain
下沉到 50 行 child 后 root 为 762 行。binding mutation report 仍是内部诊断/事务证据，未在缺少
产品 consumer、版本与安全策略时伪装成宿主命令。状态追加为
`generic_ui_host_reply_delivery_implemented_unvalidated /
generic_ui_host_queue_bounds_static_passed / binding_receipt_policy_open /
managed_app_runtime_platform_validation_pending`；本切片不改变渲染，不生成策略截图。

同日 clipboard 事务基础更新：`UiClipboardRequest` 已绑定 UUID transfer、Copy/Cut/Paste intent 与
surface-local expected edit revision，入站 `UiInputEvent::Clipboard` 携带 typed read/write/failure outcome。
Cut 不再在 write ack 前删除；匹配成功回执才提交现有 Delete action，失败保留选区。Paste read result
通过同一文本约束 owner 应用；value/caret/selection/composition/read-only/secure/focus 变化使旧结果
stale，unknown/duplicate/wrong-owner/outcome、clone/serde 均 fail closed。每 editable owner 最多一个
pending，pending 不复制全文；无 pending 的普通编辑不写 revision map。2026-08-28 后续已接
surface-targeted dynamic ABI、App event-loop backend 与 Windows `CF_UNICODETEXT`；window/seat/principal/deadline、
timeout/fault injection、跨平台 backend 与 managed 系统剪贴板验证仍开放。状态为
`runtime_transfer_contract_implemented_unvalidated /
cut_delete_after_write_ack_implemented_unvalidated / paste_result_route_implemented_unvalidated /
surface_targeted_dynamic_abi_implemented_unvalidated /
windows_cf_unicodetext_backend_implemented_unvalidated / managed_validation_pending`。

同日 editable state 原子提交基础更新：keyboard/text/IME/clipboard 的
`apply_editable_text_state` 与 accessibility SetValue/ReplaceSelectedText/SetTextSelection
已从逐属性 generic mutation 收敛到同一个 surface property transaction。accessibility 不再
先写 value、再 best-effort 补写 8 项编辑属性；一次动作现在只产生一份
`AccessibilityAction` 来源 binding report，组合态清理也包含此前遗漏的 `composition_clauses`。
prepare 在任何写入前验证 node/metadata、动态 value property 域和
caret/selection/composition grapheme boundary；reserved value property 或非法 state
零写入、无 event/binding。commit 用固定十项属性批次同步 retained metadata、runtime style、
component state 与 binding，只登记一次 dirty 和 clipboard edit invalidation。文本 value 变化
按语义角色发布 layout/text/render dirty 并推进一次 layout revision，纯 caret 变化不推进 text
revision；focus-loss composition 也复用该边界。后续 manager session 已把 exact intent、document
admission、content-free receipt 与 delta history 接到产品 Surface；model-refresh rebase、host ack 与动态验证仍开放。状态为
`surface_property_prepare_commit_implemented_unvalidated /
widget_and_accessibility_projection_converged_unvalidated /
partial_metadata_write_path_closed / composition_clause_clear_path_closed /
semantic_text_dirty_invalidation_implemented /
product_document_transaction_implemented_unvalidated / model_rebase_and_managed_validation_pending`。

同日 generic external property 入口继续收敛：可编辑节点的正文属性不再直接落单个
metadata 字段，而是先从 retained state 生成候选；显示文本实际变化时保留仍合法的 caret，
越界则 clamp 到 grapheme boundary 并转为 downstream affinity，同时清空 selection 和完整
composition，再通过上述固定十项事务一次提交。同值写保持 no-op，不移动 caret 或清理组合态。
派生 `caret/selection/composition` 属性经 generic mutation 单写现在 fail closed，必须由 typed
事务发布。正文存储值与显示文本已分离，因此 `NumberField` 的外部 `Float` 更新保留数值类型，
不被错误字符串化；binding/reflection source kind 由共享映射进入整批 retained/component 更新。
本次限定 Windows `cargo test -p zircon_runtime
surface_external_text_value_change_commits_complete_edit_state_once --no-run` 在 184 秒内无编译器
输出并超时终止，不能算通过。focused bound-text policy、数值字段内部编辑解析、
产品 document authority/rebase 与 managed profile/WGPU 仍开放。状态追加为
`generic_external_projection_converged_unvalidated /
derived_property_write_bypass_closed_unvalidated /
numeric_external_value_type_preserved_unvalidated / managed_validation_pending`。

同日 component editing authority 收敛更新：Material 可编辑文本 descriptor 不再声明原始
`KeyboardText`，`state_reducer/text_input.rs` 中独立重建并写回正文、caret、selection、composition
的第二套编辑实现已删除。字符输入继续只由 surface editable transaction 消费，组件 reducer 只接收
`ValueChanged`、`Commit` 与 `Focus` 做镜像和 validation；菜单 typeahead 与 command palette 的
`KeyboardText` 不受影响。`SearchField`、`FieldEditor`、`SourceEditor` 已复用同一文本输入 descriptor
构造器，统一声明 host text-input capability 与 retained edit properties；行为推断覆盖完整 Material
文本 role/component alias。V1 编译器按 `query -> value -> value_text -> text` 为 TextInput 推断唯一
正文属性，显式 `widget.value_property` 仍最高优先级；Surface fallback 同步支持 `value_text`，避免
`FieldEditor` 编辑 `text` 而 validation 读取陈旧 `value_text`。该选择只发生在编译/状态解析边界，
不增加逐字输入热路径扫描。

Rust 2024 格式化、`git diff --check` 与生产引用静态扫描通过；旧 component edit helper 删除 315 行。
限定 Windows `cargo test -p zircon_runtime_interface
editable_text_component_roles_share_one_behavior_classification` 在 64 秒内仍处于依赖编译且无测试
结果，已终止其 Cargo/Rustc 子进程，不能算测试通过。随后复用同一 E 盘 target 的
`cargo check -p zircon_runtime_interface --lib` 在 114.5 秒完成，只有 9 项既有 warning，确认本轮
interface production lib 可编译；该证据不覆盖 `zircon_runtime`、测试执行或产品路径。状态追加为
`component_raw_keyboard_edit_authority_removed_unvalidated /
canonical_editable_value_property_inference_implemented_unvalidated /
semantic_component_projection_only_unvalidated / interface_lib_compile_passed /
managed_validation_pending`。
`RTE-P1-002` 的当前源码直写旁路静态关闭，但产品 document service 消费、focused binding、数值
字段内部 parse/commit 与动态资格门仍开放，M1/M2 不关闭。

同日 focused bound-text policy 复审没有直接写代码。Unreal
`SlateEditableTextLayout.cpp:3622-3636` 在聚焦时只刷新 password/marshaller 状态，不用 bound text
替换 editable text；`OnBoundTextChanged:4508-4547` 仅在显式 force review 时允许聚焦替换，并按
实际差异发事件。当前 Zircon `UiPropertyMutationRequest` 的 `RuntimeState/Binding` 来源和
`binding_source_kind` 不能区分“外部模型刷新”与“显式 SetText/LoadText”，surface metadata 又同时
充当 bound value 与 edit buffer。此时加入 focus early-return 会丢模型更新，直接 mutation 会覆盖
用户编辑，二者都不可接受。实现前必须先建立独立 model/edit authority、typed refresh origin、每
owner 有界 pending refresh、expected revision 与 blur/rebase 冲突收据；secure pending value 还必须
归 secure store 并在 teardown/policy change 清理。状态为
`unreal_focused_refresh_policy_reviewed / mutation_origin_not_expressive /
bound_editable_value_split_open / product_document_session_required /
focused_refresh_implementation_deferred_without_false_fix`。

同日 NumberField current-source 复审确认另一个独立缺陷：catalog 的 `value` 是 `Float`，但内部
`commit_editable_text_properties` 把所有字符编辑正文无条件构造成 `UiValue::String`；现有测试只覆盖
arrow/drag/直接 Float commit。Unreal `SSpinBox.cpp:937-1076` 使用独立 `EditableText`、typed
`ValueAttribute` 和 `INumericTypeInterface`：输入串先过滤，只有可完整解析时才可发布 per-key value，
commit 再 parse、clamp、typed publish 并更新格式化 cache。本轮先在共享 property transaction
preflight 增加 retained/next `UiValue` variant 一致性检查；`Float -> String` 在首写前返回 typed
`value_kind_mismatch`，value/caret/event/binding/dirty 全部不变，`Float -> Float` 外部更新仍合法。
这关闭类型破坏路径，不代表数值键入可用。独立 edit buffer、locale/type interface、intermediate
invalid、commit/cancel/blur、format cache 与 focused external refresh 仍需产品 edit session。
状态为 `numeric_value_kind_corruption_fail_closed_unvalidated /
number_edit_buffer_open / locale_parse_commit_open / managed_validation_pending`。

同一轮 canonical property 复查还发现 Autocomplete 的 Surface 早已编辑 `query`，但 renderer 在共享
editable classifier 后仍默认从已选 `value` 构造 visible/editable layout state；raw/V2 metadata 会
出现输入改 query、布局/selection/caret 却绑定 selected value。现已提取 metadata-level borrowed
resolver，input 在 transaction 边界只克隆一次属性名，render 直接借用 `&str`；显式 override 后按
`query -> value -> value_text -> text` 选择。新增 resolve 回归同时断言可见文本、editable text 与
caret 使用 query，selected value 不被当作编辑正文。该修复不增加逐帧属性名分配。NumberField
focused runtime 单测尝试约 70 秒仍无输出，随后核对并终止 exact Cargo/Rustc process tree，不能算
动态通过。状态追加 `autocomplete_query_render_edit_property_converged_unvalidated /
canonical_metadata_property_resolver_shared / managed_validation_pending`。

同日内部 document 增加 revision-bound snapshot lease 前置。初始 revision 直接复用 original
`Arc<str>`；真实 edit 在 prepare 成功后只清空新 revision 的 lazy 连续快照槽，第一次明确 lease
请求至多展平一次，后续请求只 clone `Arc`，旧 revision lease 保持稳定，typed no-op 不失效当前
lease。grapheme source index 直接借用该 lease 扫描，删除原先 `snapshot String` 的第二份全文
复制；全文 grapheme rebuild 本身仍为 `O(N)`。document/lease Debug 只输出 identity/revision/长度/
块计数，不输出正文。该实现尚无产品 registry、snapshot byte/retention 的产品阈值或 Surface
consumer，也没有 managed profile。状态为
`revision_bound_snapshot_lease_implemented_unvalidated /
single_flatten_per_requested_revision_implemented_unvalidated /
source_index_secondary_source_copy_removed_unvalidated /
document_debug_source_redacted / product_registry_open /
snapshot_budget_and_managed_profile_pending`。

内部 document authority 随后自行签发并持有 public UUID，snapshot lease 同时绑定该 UUID 与 typed
revision；cache-oriented `owner + revision` 只留作布局复用键。changed receipt 可在不读取正文的
`O(1)` 路径投影公共 receipt，且不再接受调用方传入任意 document id；投影检查 owner/revision、
length delta、`usize -> u32`、old/new bounds 与最终 selection bounds。最终 byte selection 还保留
focus affinity，避免 wrapped/BiDi 边界相同 source offset 丢失视觉 caret 归属。内部显式限额
session store 与 prepare/commit admission 前置见下段；grapheme/source equality 与 Surface consumer
仍开放。

同日公共文本编辑事件完成一次边界硬切。旧 `UiTextEdit` 会把 raw `UiTextEditAction` 以及完整
`before/after UiEditableTextState` 一并序列化到 `UiWidgetEvent`，既没有稳定 document identity 和
revision fence，又会让一次编辑的公共事件成本随文档长度增长。现改为版本化、无正文的
`UiTextEditReceipt`：只发布 document UUID、前后相邻 revision、typed edit kind/source、old/new byte
range 与最终 byte selection；schema、nil identity、revision 跳变/耗尽和反向 range 均可 typed
fail-closed。旧 snapshot/action DTO 已从公共事件字段移除，`UiTextEditAction` 只保留为进程内 intent。
内部 changed receipt 已能签发 public receipt，snapshot lease 也绑定 public UUID/revision；后续
`UiInputManager` product session 已成为 Surface/runtime producer，并在双 preflight 后发布 content-free
receipt。M1 仍因 managed Runtime、产品阈值与 model-refresh/rebase 未验收而不关闭。状态为
`public_text_edit_snapshot_event_removed_unvalidated /
versioned_document_edit_receipt_contract_implemented_unvalidated /
internal_document_receipt_projection_implemented_unvalidated /
surface_product_gateway_implemented_unvalidated /
m1_document_authority_implemented_unvalidated / managed_validation_pending`。

同日 document mutation 增加两阶段 prepare/commit，prepare 在零mutation状态完成expected key、UTF-8、
no-op、checked revision/length、hard-line repair与next piece topology，commit再次校验key。随后新增只允许
`with_limits`构造的surface/session `TextDocumentStore`；没有Default、没有全局manager注册。调用方必须显式
给出document/visible bytes/replacement/retained source/chunk/piece/current snapshot/active lease预算。
changed edit在exact prepare后、commit前admit，拒绝不发布revision、不追加chunk；snapshot在flatten前
校验typed revision和current/active预算，managed lease在Drop释放count/bytes。后续 Surface/UiInputManager
已接线并拥有 topology-gated teardown；产品阈值标定、secure policy 与增量 grapheme edit handles 仍开放。状态为
`document_prepare_commit_boundary_implemented_unvalidated /
explicit_limit_session_store_implemented_unvalidated /
snapshot_lease_admission_and_release_implemented_unvalidated /
global_manager_registration_rejected / surface_input_session_integration_implemented_unvalidated /
product_thresholds_and_managed_profile_pending`。

2026-08-28 补记：`edit_state` 现在在产生最终 editable state 的同一 owner 中发布固定大小
`CommittedTextEditIntent { old, new, kind }`，replacement 直接借用最终 state 的 `new` range，不再要求
Surface 对 edit 前后全文做 `O(N)` prefix/suffix diff。keyboard word delete 的“selection + delete”只允许一个
committed intent；未来若一个 action sequence 意外包含两次正文提交则 typed 拒绝。IME preedit、caret、selection、
composition cancel 与相同正文 replacement 都是 state-only，不推进 document revision；composition commit、
DeleteSurrounding、cut/paste 保留精确 old/new range。意图已经进入内部十属性事务收据，但公共
`UiTextEditReceipt` 仍只能由 document store 成功 commit 后签发；后续 manager gateway 已完成这一生产
签发路径。E盘 current-source edit harness `12/12`，document harness 后续扩展为 `54/54`。状态为
`exact_committed_edit_intent_implemented / action_sequence_single_commit_guarded /
composition_preedit_state_only / surface_property_receipt_integration_implemented /
surface_property_exclusive_prepare_implemented / document_store_exclusive_prepare_implemented /
dual_commit_coordinator_and_product_document_binding_implemented_unvalidated /
document_harness_54_of_54 / managed_runtime_validation_pending`。

- [2026-06-27 至 2026-08-03 IME 里程碑产出记录](08/2026-08-05-ime-milestone-output-records.md)

2026-08-28 product document session current-state update：`UiInputManager` 现持有不参与
`UiSurface` clone/serde 的 crate-local `UiTextDocumentSession`，并以 `(UiTreeId, UiNodeId)` 绑定
`TextDocumentStore` document UUID/revision。keyboard、text、IME commit/delete-surrounding 与 clipboard
cut/paste 的 committed intent 会先完成 document admission/public receipt projection 和十属性 Surface
preflight，再通过薄 dual transaction 提交；成功后只在 `UiInputDispatchResult.widget_events` 发布无正文、
固定大小的 `UiWidgetEvent::TextEditChange`。连续 manager 编辑保持同一 document id 和相邻 revision；
程序化正文替换会推进独立 committed-text epoch 并在下一次编辑重绑新 document，样式/layout revision
不会误换 document。IME preedit/caret/selection 仍是 state-only，不推进该 epoch，因此 composition commit
继续从 preedit 前正文生成 old/new byte range。secure result 只发布不含正文的 receipt，既有 redaction
仍负责输入、binding、effect、host/component payload。

MVP store policy 以命名常量显式限制 document 数、正文/replacement/retained source、piece、snapshot 与
active lease；这些是 fail-closed 初始值，不是已通过产品负载校准的最终阈值。manager product route 已有
连续编辑、外部 source rebind 与 IME preedit/commit 回归；Accessibility TextInput `SetValue` 和
`ReplaceSelectedText` 也携带 exact intent 并复用同一 UUID/revision 链，selection 仍为 state-only。
session 通过 topology generation/node count 和 pending removal 回收 detached owner；普通稳定事件不做
全 binding 扫描。新增 session 生产文件保持 folder-backed，相关 production owner 无
`unwrap/expect/panic/unreachable`，scoped Rustfmt 与 `git diff --check` 通过。一次 Windows managed
validation 已被 coordinator 接受，但 request `940481beb84e4174a1feacd07c4413f6` 在 15 秒内没有终态并以
`command_post_timeout` 返回；按当前执行约束未轮询或重复提交，不能算编译/测试通过。直接
`UiSurface::dispatch_input_event` 兼容路径仍只提交 widget projection 且明确不伪造 document receipt；
focus-loss cancellation 与 delta history 已在 manager 产品路径实现，model-refresh/rebase、managed Runtime/WGPU/
平台实机仍开放。状态为
`input_manager_document_session_implemented_unvalidated /
committed_text_epoch_separated_from_layout_revision /
manager_dual_transaction_and_content_free_receipt_implemented_unvalidated /
ime_preedit_document_identity_guarded_unvalidated /
accessibility_text_document_gateway_implemented_unvalidated /
detached_owner_reclamation_implemented_unvalidated /
surface_instance_document_identity_implemented_unvalidated /
delta_history_and_focus_loss_owner_implemented_unvalidated /
low_level_document_gateway_intentionally_receipt_free / managed_validation_no_terminal_result /
wgpu_and_platform_acceptance_pending`；IM-M2/IM-M3 与整个 Text08 不关闭。

2026-08-28 Surface identity 补记：document session 激活不再只比较可序列化的 `UiTreeId`。
`UiSurface` 现持有一个 `serde(skip)` 的运行时实例令牌；新建、反序列化和 clone 都获得新 owner identity，
同一实例产生的 handle 则以 `Arc::ptr_eq` 稳定比较。`UiInputManager` 每次 owner 同步同时提交 tree id 与
实例 handle，因此复用一个 manager 在两个同 tree-id、同 node-id、同 committed epoch 的 Surface 之间
切换时会先关闭旧 document/binding，不能把第二个 Surface 的 UUID/revision 链带回第一个 Surface。
Surface 语义相等性不包含该纯运行时身份，公开 DTO 和序列化格式未改变。三次切换产品回归已加入但
managed Runtime 尚无终态；focus-loss composition 的 state-only owner 后续已实现并记录于下段。

2026-08-28 document history 补记：`UiTextDocumentSession` 现按 binding 持有增量 undo/redo，entry
只保留 exact old/new byte range、removed/inserted delta 与前后 caret/selection，不保存全文 state 或
document snapshot。正常编辑先按 1 MiB delta 上限判定，超限与 secure edit 都可继续提交但形成 history
barrier；100 条深度与 byte budget 同时约束。外部 source epoch 重绑、detached owner、tree/Surface
identity 切换都会随 document binding 清空历史。Ctrl/Cmd+Z、Ctrl+Y 与 Ctrl/Cmd+Shift+Z 在 clipboard/
payload 前由 focused editable owner 捕获；空栈、read-only 和 composing 也不会向 editor/world route
泄漏。Undo/Redo 生成 typed exact intent 并复用 document admission、public receipt projection、十属性
Surface preflight 与 dual commit，只有 changed dual commit 成功后才移动栈。IME preedit 不入栈，commit
是一条 history unit。E 盘 direct history harness `3/3`、document current-source harness `54/54`；UUID/
revision、redo branch、IME 与 secure 产品回归已加入但 managed Runtime 未取得终态，因此状态为
`delta_document_history_implemented_unvalidated / full_document_history_copy_rejected /
focused_history_commands_owned_unvalidated / secure_and_lifecycle_history_barriers_implemented /
direct_history_3_of_3 / document_harness_54_of_54 / managed_runtime_profile_power_wgpu_pending`。
focus-loss composition 已按 Unreal `CancelComposition -> DeactivateContext -> Commit notification ->
ClearUndoStates` 语义完成结构修复：Surface 只恢复 preedit 前已提交正文并清空 composition，不携带
committed intent、不推进 source epoch、不签发 text edit receipt；非 read-only 的已有 Submit binding
仍收到恢复后正文的 Commit 通知，read-only 只清理 transient state。manager 不再尝试把失焦伪装成
document edit；Surface 以独立于 IME Disable 的 actual focus-loss owner 集合精确清除 owner-local
history，因此 secure/已停用 IME 与两次 manager 调用之间的 programmatic blur/refocus 都不会漏清。
集合去重并以 1024 owner 为上限，超限 fail-closed 清空全部 history。后续真实 Backspace 的产品回归要求复用失焦前
document UUID，并从 revision `1` 推进到 `2`；undo 在失焦后本地 handled/unavailable。E 盘 current-source
focus-loss harness `1/1`、owner queue harness `2/2`，Rustfmt/whitespace 静态检查通过；Runtime 产品回归、平台 IME 与 WGPU 仍未
managed 验收。状态追加为 `focus_loss_preedit_cancel_implemented_unvalidated /
focus_loss_document_epoch_preserved_unvalidated /
focus_loss_owner_history_barrier_implemented_unvalidated /
focused_current_source_1_of_1 / focus_loss_owner_queue_2_of_2 /
managed_runtime_platform_wgpu_pending`，Text08 不关闭。

2026-08-28 Surface edit projection profiling 补记：retained document 已使 document mutation range-local，
但 `editable_text_state_for_node`、property proposed value 与可选 component Change/Commit payload 仍各自
拥有完整正文，Surface 产品路径因此仍至少为 `O(N)`，不能用 document store profile 宣称总体编辑
已增量化。新增 folder-backed `editable_text/profile.rs` 只接收长度/布尔值，以固定 counter 分开记录
state materialization、property clone attempt、admitted projection、committed/state-only/composition 与
component payload，并为 property prepare/commit 提供固定 span；无 profiling feature 时为 no-op。
E盘 current-source path `5/5`、Rustfmt/whitespace/隐私扫描通过。真实 allocator/RSS/延迟/功耗 baseline
与 matched Unreal 尚未执行，因此不开始 document-handle/model-edit cutover。状态为
`surface_full_source_projection_profile_instrumented_unvalidated /
direct_profile_path_5_of_5 / managed_baseline_pending / p1_17_open`。

2026-08-28 clipboard host bridge 补记：`UiInputManager` 现在把 dispatch result 中的 typed
clipboard request 收入每个 Surface 独立的 256-row 有界队列；同 owner 尚未送出的旧请求由新请求替代，
而 Surface pending store 仍以 UUID transfer 和 edit revision 作为结果准入权威。dynamic host ABI 显式携带
`target_viewport + target_surface`，宿主结果只回送给该 Surface，不再按局部 `UiNodeId` 广播。
host-output 原有分页/提交/回滚继续持有未提交 batch，队列或输出上限不会静默丢失已接纳请求。

`zircon_app` 在 winit event-loop owner 上完成平台操作后才立即回送 typed result。Windows backend 对齐
Unreal `FWindowsPlatformApplicationMisc`，使用目标 winit Window 的真实 Win32 HWND 打开 clipboard，读取/
写入 `CF_UNICODETEXT`、`GMEM_MOVEABLE` 和 ownership-transfer RAII；不再使用会破坏 `EmptyClipboard` 后
owner 语义的 NULL HWND。非 Windows 当前明确返回 `Unsupported`，不伪造进程内成功。正文 UTF-8 上限为
32 KiB；最坏 32 KiB NUL 的 JSON `\u0000` 扩张仍低于 256 KiB host/event envelope，读入超限结果会转成
`PayloadTooLarge` 并结束对应 pending transaction。该切片没有渲染变化，因此不生成策略截图；真实系统
clipboard、managed Runtime、timeout/fault injection、macOS/Linux/Web backend、window/seat/principal 与 WGPU
产品验收仍开放。状态追加为
`runtime_surface_targeted_clipboard_abi_implemented_unvalidated /
windows_cf_unicodetext_backend_implemented_unvalidated /
clipboard_payload_and_queue_bounds_implemented_unvalidated /
non_windows_typed_unsupported / managed_system_clipboard_and_product_validation_pending`，Text08 不关闭。

## 2026-08-28 Focused bound model-update gateway

Unreal 参考复审确认了必须保持的分流：`SlateEditableTextLayout::RefreshImpl(nullptr)` 在聚焦时只更新
password/marshaller 状态，`OnBoundTextChanged` 默认不以 bound attribute 覆盖活动 edit buffer；显式
`SetText/LoadText` 则 force review 并把 caret 移到末尾。Zircon 现在以版本化
`UiTextModelUpdateRequest` 表达该差异，请求携带独立 request UUID、tree/node、expected document
UUID/revision 和 `BoundRefresh | ExplicitSetText | ExplicitLoadText` origin。公共
`UiTextModelUpdateReceipt` 不携带正文，并校验状态、失败原因、current document key 与可选 document edit
receipt 的一致性。

产品 owner 是现有 `UiInputManager` 的 folder-backed `bound_text_model_updates` 子模块，不创建第二个
document、tree 或 binding registry。535 行 queue owner 负责 request/pending/terminal 生命周期，282 行
transaction child 复用唯一 document+Surface 双事务，137 行 profile child 只发固定名称计数。聚焦的
`BoundRefresh` 保留每 owner 最新一个 pending 请求并立即返回
`Deferred`；失焦时重新读取 committed document key，完全相等才通过现有 document prepare、Surface 十
属性 prepare 和 dual commit 应用，用户编辑导致 revision 变化则返回 `Conflict/StaleDocument`，不覆盖
edit buffer。新的 unchanged refresh 也会 supersede 旧 pending，避免旧值在随后失焦时复活。显式
Set/Load 不等待失焦；若当前存在 IME preedit，比较和 exact replacement 基于 cancel 后的 committed base，
最终只推进一次 document revision 并清空 composition。

MVP 容量是 pending+terminal receipt 合计 256 行、单正文 4 MiB、pending 正文合计 16 MiB；所有拒绝均
发生在保留/写入前。secure owner 的 pending 正文只进入 `UiSurface` 的 serde-skip/clone-empty secure
store，manager pending 行只保存 identity、revision、origin、长度和 secure 标记；detach、security policy
change、supersession 和 focus-loss terminal path 会清理对应值。generic editable property mutation 仍是
显式正文替换兼容路径，不被宣称为 bound refresh。Surface-owned store 通过只暴露 `clear` 的 opaque
handle 绑定 manager 生命周期；manager 切换 Surface 或 Drop 时也会撤销旧 store，manager 无正文读取接口。

后续明文生命周期复审发现该 store 仍使用普通 `String`：remove、replacement、`clear` 和 Drop 只释放
capacity，不保证擦除。pending secure model value 现由 Surface store 内的
`zeroize::Zeroizing<String>` 持有；supersession、detach、policy change、Surface switch 与 teardown
都会擦除 retained allocation。accepted value 以 `mem::take` 将同一 allocation 移交既有
property/document transaction，不增加第二份全文复制。这里只关闭 pending-store boundary，不宣称端到端
zeroization；request failure、component state、retained document/history、layout/platform buffer 与 crash dump
仍是普通明文 owner，必须由后续 secure document/session architecture 统一治理。

已补 malformed rejection、focused defer、未编辑失焦应用、编辑后 conflict、显式聚焦替换、IME preedit
中显式替换、secure store、security-policy change、owner detach、oversize rejection 和 latest-no-op
supersession 回归源码。Surface switch 与 manager Drop 的 secure revocation 回归也已加入。
Rustfmt、scoped whitespace、owner 行数与无 production panic 静态检查通过。2026-08-28 的受管 Windows
default `zircon_runtime` build 使用批准的 D 盘 pool，进入 Runtime crate 后因共享脏树的 154 个错误失败，
已知尾部包含非本切片 SDF-atlas missing symbol；后续诊断 lane 获取发生 coordinator post-timeout，按策略未
轮询或重试。尚无聚焦行为测试、平台 IME、WGPU/PNG 或功耗通过证据，因此 Text08 不关闭。状态追加为
`focused_bound_model_update_gateway_implemented_unvalidated /
unreal_force_review_semantics_implemented_unvalidated /
revision_cas_blur_resolution_implemented_unvalidated /
ime_preedit_committed_base_guarded_unvalidated /
secure_pending_surface_owned_unvalidated /
secure_pending_drop_zeroization_implemented_unvalidated /
persistent_secure_document_zeroization_open /
bounded_pending_and_terminal_receipts_implemented_unvalidated /
managed_runtime_platform_wgpu_power_pending`。

## 2026-08-28 Secure keyboard word-boundary policy

Unreal 的 password command policy 明确避免通过 Ctrl 导航/删除泄露 word break：Ctrl+Left/Right 跳到
行首/行尾，Ctrl+Backspace/Delete 在无 Alt/Shift 时选择到行首/行尾再删除。Zircon 原实现无条件以 secure
正文调用共享 Unicode word-boundary owner，虽然 copy/cut 已禁用，仍可从 caret 移动量观察分词。

唯一 `text_keyboard/edit_actions` owner 现在显式接收 canonical secure classification。普通文本保持既有
Unicode word navigation；secure 文本的四个 Ctrl 命令只查询 hard-line boundary，Shift 导航仍扩展选择，
删除仍经同一 exact edit intent、document+Surface transaction 和 secure result redaction。回归源码覆盖
左右导航与前后删除，且普通 word-shortcut tests 保持原断言。本切片不改变 glyph/layout/render output，
不生成策略截图；状态为 `unreal_secure_keyboard_policy_implemented_unvalidated /
word_boundary_side_channel_removed_from_keyboard_commands_unvalidated /
managed_runtime_and_product_input_pending`。

## 2026-08-29 NumberField invariant edit-buffer MVP

NumberField 已从“阻止 Float 被 String 破坏”推进到可编辑的最小闭环。Unreal
`SSpinBox::TextField_OnTextChanged/TextField_OnTextCommitted/CommitValue` 与
`SNumericEntryBox::SendChangesFromText` 仍是主参考：活动文本与 typed value 分离，完整解析成功才可发布
typed change，字段 commit 统一执行 parse/policy/typed publication。

Zircon 的 retained metadata 现保存 `Float value + String value_text + bool number_edit_active`；input 与
render 共用该 authority。默认 per-key 编辑只更新 buffer，Enter/blur/Escape 复用一个数值提交事务，
typed component event 携带 `UiValue::Float`。V1 parser 明确限定 invariant ASCII、`.` 小数点与 `e/E`
科学计数法，区分 empty/intermediate/valid/out-of-range/non-finite/invalid-character/invalid-syntax/
invalid-policy/too-long；编辑缓冲区受 128-byte MVP 硬上限保护，min/max、finite 与可选
positive-step snap fail closed。公共 diagnostics 只携带 versioned
status receipt，不携带原文本。

后续 Unreal `SSpinBox::OnKeyDown` 复审修复了组件外层与文本内层的路由顺序：无修饰 Up/Down 在
通用单行光标动作前进入 NumberField typed step，始终从 canonical Float 加减正有限 `step`，成功后
规范化 `value_text` 并退出编辑态；坏 step/policy 或溢出返回 `KeyboardStep + Rejected` receipt，保留
原缓冲区且零写入。该路径复用同一数值属性事务和 typed Commit，不经过 String value 旁路。

状态：`number_field_edit_buffer_implemented_unvalidated /
typed_float_change_and_commit_implemented_unvalidated /
enter_blur_escape_policy_implemented_unvalidated /
unreal_outer_keyboard_step_routing_implemented_unvalidated /
ime_preedit_is_not_field_commit /
locale_precision_rounding_open / focused_external_refresh_implemented_unvalidated /
managed_runtime_platform_wgpu_power_pending`。源码、descriptor 和 DTO 回归已经加入，scoped Rustfmt 与
diff check 通过；本段不宣称 Cargo、真实平台输入、WGPU/PNG 或性能验收通过，Text08 保持开放。

## 2026-08-29 NumberField focused Float model refresh

按 Unreal `SSpinBox` 的 `ValueAttribute`、editable String、`CachedExternalValue` 与
`UpdateNow` authority 分离重新复审后，数值刷新没有复用 String document gateway，也没有给焦点态
增加第二个 pending queue。`NumberField` 现在以 `number_value_revision` 表示 canonical Float revision，
以 `number_edit_base_revision` 表示活动缓冲区观察到的 canonical base；两者由共享数值属性事务和
Float/display/edit/caret/selection/composition 一次性 preflight/commit。

新的版本化 `UiNumberModelUpdateRequest` 使用 manager-owned model UUID、expected numeric revision、
`BoundRefresh/ExplicitSetValue` origin 与 content-free receipt。Bound refresh 立即更新 canonical Float，
但在编辑活动时保留 `value_text` 与旧 base；Enter 遇到 stale base 返回 typed `Conflict` 并保留缓冲区，
blur 则采纳最新 canonical display 并退出编辑。显式 SetValue、a11y、Escape 和 keyboard step 继续保持
明确的 replace/discard 语义。真实 canonical change 才 checked advance revision；畸形 revision、耗尽、
stale CAS 与非有限值均在首写前拒绝。

同轮继续复审 `SSpinBox.cpp:95-141,186-215,904-933,993-1076`：bound `ValueAttribute` 是外部
权威，外部值摄入不因 `MinValue/MaxValue` 自动改写；clamp 与可选 delta snap 属于 typed/spin/arrow
用户提交。Zircon 因此保持 bound Float 原值，且 model key 只有在 canonical 属性仍是 finite TOML
Float、canonical/edit-base/edit-active revision authority 完整时才签发。manager 观察 Surface
layout-order generation 只负责在 topology 变化时触发一次 `O(N)` detached-owner prune。`UiTreeNodes`
现在在唯一 `insert` 边界分配树生命周期内单调且不因 `clear()` 复用的 insertion serial，并由
`UiTree::node_incarnation` 只读投影；numeric identity 按 owner incarnation 重签。retained node pool
detach/reinsert 后同一 `UiNodeId` 不会误命中旧 CAS key，而无关 sibling 增删、布局或属性变化保持
NumberField UUID 稳定。稳定输入路径不扫描 owner map，也不增加第二张 incarnation 表。

源码回归已覆盖 focused preservation、Enter/blur resolution、explicit replace、stale CAS、non-finite/
revision-exhaustion zero-write、generic internal-state bypass rejection 与 Surface session identity
replacement，以及 retained same-node-id reuse 的 UUID 失效/旧 key conflict/零写入和 non-Float
canonical key rejection；无关 sibling insert/detach 回归同时要求跨两次 topology generation 保持
model key 不变。固定五项 profile counters 已接入 request、focused
preserve、conflict、revision advance 与 rejection。状态：
`number_field_numeric_revision_authority_implemented_unvalidated /
focused_float_refresh_gateway_implemented_unvalidated /
stale_edit_commit_policy_implemented_unvalidated /
retained_owner_aba_guard_implemented_unvalidated /
unrelated_topology_key_stability_implemented_unvalidated /
managed_runtime_profile_power_wgpu_pending`。尚无新的 Cargo、真实平台输入、WGPU/PNG、功耗或 Unreal
对拍通过证据，Text08 不关闭，也不生成纯策略截图。

## 2026-08-30 IME and pointer geometry font-owner convergence

`UpdateCursor` 的 caret/composition rect 与 pointer text hit-test 在 glyph artifact 不可用时仍需重新 shaping
简单 source-isomorphic LTR 行。该 recovery 过去隐式读取 process-global 字体集合，即使当前
`UiSurface` 已由 Runtime Core 注入独立 collection；因此 layout、候选窗锚点与点击落点可能使用不同 face
metrics。现在 `ime_context.rs` 在一次 input-method context 更新中从 Surface measure cache 捕获 exact
`FontCollectionSnapshot`，caret 与 composition 共用该 lease；`text_pointer.rs` 的一次命中查询同样捕获并
传入自身 collection。由于 neutral `UiResolvedTextLayout` 不携带 collection revision，只有 Surface 已观察的
layout generation 与 snapshot generation 相等时才允许 source-metric reshape；不相等时 fail closed 到已发布
layout 的 artifact/glyph advances，等待正常 layout pass 更新。无 source metrics 的保守 fallback 与已有 glyph
artifact 第一优先级不变。

Unreal 依据为 `FSlateFontMeasure` 由具体 `FSlateFontCache` 构造并持有该 cache，字符偏移查询与度量共享
同一字体 owner。当前只完成非验收所有权修正：静态 suite 19/19、rustfmt 与 diff-check 通过，collection-bound
provider 行为回归已写但 Cargo 未运行；真实 IME 候选窗、pointer 输入、WGPU/PNG 与 profile/power 仍开放。
状态：`surface_ime_pointer_font_collection_identity_static_implemented /
managed_platform_input_and_product_validation_pending`。
