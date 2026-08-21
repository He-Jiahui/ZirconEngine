---
title: Runtime Text Editing、Document、Selection、Caret、Hit Test、IME Composition、Clipboard、Secure Text 与 Product Integration 当前源码工程化差距
category: zircon_runtime
report_id: Runtime82
review_date: 2026-08-21
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_runtime/src/ui/text/edit_state.rs
  - zircon_runtime/src/ui/text/grapheme.rs
  - zircon_runtime/src/ui/text/grapheme
  - zircon_runtime/src/ui/text/hit_test.rs
  - zircon_runtime/src/ui/text/geometry.rs
  - zircon_runtime/src/ui/surface/input/editable_text.rs
  - zircon_runtime/src/ui/surface/input/editable_text
  - zircon_runtime/src/ui/surface/input/keyboard_clipboard.rs
  - zircon_runtime/src/ui/surface/input/text_keyboard.rs
  - zircon_runtime/src/ui/surface/input/text_keyboard
  - zircon_runtime/src/ui/surface/input/text_pointer.rs
  - zircon_runtime/src/ui/surface/input/text_state.rs
  - zircon_runtime/src/ui/surface/input/text_constraints.rs
  - zircon_runtime/src/ui/surface/input/effect/text_services.rs
  - zircon_runtime/src/ui/surface/focus.rs
  - zircon_runtime/src/ui/dispatch/input_manager/ime_host_requests.rs
  - zircon_runtime/src/ui/surface/render
  - zircon_runtime/src/ui/component/state_reducer/text_input.rs
  - zircon_runtime/src/ui/accessibility
  - zircon_runtime_interface/src/ui/surface/render/editable_text.rs
  - zircon_runtime_interface/src/ui/dispatch/input
  - zircon_runtime_interface/src/ui/window/input
  - zircon_runtime_interface/src/runtime_api/host_requests.rs
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_runtime/src/dynamic_api/session/host_requests.rs
  - zircon_runtime/src/ui/platform_input/winit_translation.rs
  - zircon_app/src/entry/runtime_entry_app/ime_input
  - zircon_app/src/entry/runtime_entry_app/host_requests/ime
tests:
  - zircon_runtime/src/ui/tests/widget_text_input_ime_context.rs
  - zircon_runtime/src/ui/tests/widget_text_input_ime_context
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard_clipboard.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard_hard_line.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard_text.rs
  - zircon_runtime/src/ui/tests/widget_text_input_mui.rs
  - zircon_runtime/src/ui/tests/widget_text_input_pointer.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/11b-runtime-text-font-shaping-layout-editing-ime-review.md
  - docs/plans/optimize/zircon_runtime/77-runtime-ui-input-dispatch-routing-focus-navigation-pointer-capture-gesture-drag-drop-ime-window-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/78-runtime-ui-accessibility-semantic-tree-name-description-relation-state-action-live-region-platform-adapter-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/81-runtime-text-shaping-unicode-bidi-script-run-cluster-line-break-wrap-layout-product-integration-current-source-review.md
  - docs/plans/zircon_runtime/text/08-ime-and-text-input.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Text/SlateEditableTextLayout.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Text/SlateEditableTextLayout.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Public/GenericPlatform/ITextInputMethodSystem.h
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Public/Windows/WindowsTextInputMethodSystem.h
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Private/Windows/WindowsTextInputMethodSystem.cpp
  - dev/godot/scene/gui/line_edit.cpp
  - dev/godot/scene/gui/text_edit.cpp
  - dev/godot/servers/display/display_server.h
  - dev/godot/platform/windows/display_server_windows.cpp
  - dev/bevy/crates/bevy_text/src/text_edit.rs
  - dev/bevy/crates/bevy_ui_widgets/src/text_input.rs
  - dev/bevy/crates/bevy_ui/src/widget/text_input_layout.rs
  - dev/bevy/crates/bevy_feathers/src/controls/text_input.rs
  - dev/Fyrox/fyrox-ui/src/text_box.rs
doc_type: current_source_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Runtime Text Editing、Document、Selection、Caret、Hit Test、IME Composition、Clipboard、Secure Text 与 Product Integration 当前源码工程化差距

## 1. 结论

当前文本输入链已经不是纯占位：编辑操作按grapheme执行插入和删除，selection/caret/preedit有中立DTO，pointer hit-test能够优先复用resolved layout，IME surrounding text有grapheme与byte上限，App也会把winit IME事件和cursor/surrounding-text请求接到平台窗口。这些底座应保留。

但产品级authority仍停留在“控件属性里放一份`String`”的阶段。widget input与component reducer各自从TOML metadata重建编辑状态，再通过多次独立property mutation写回text、caret、selection和composition；没有document identity、revision、expected-revision、transaction、operation receipt、undo/redo或外部编辑rebase。selection公开入口只保证UTF-8 char boundary，垂直移动、Home/End和triple-click仍按source hard line工作，无法以Runtime81产出的visual cluster/line geometry作为唯一真值。

IME链在UI层拥有composition rects和surrounding text，但转为core host request时丢掉composition rects；动态ABI只携带preedit string/cursor并在production构造空clause。composition直接写入可提交`String`并保存明文restore副本，focus-loss提交又是best-effort。剪贴板更严重：Runtime能生成`UiClipboardRequest`，但App、Editor和动态ABI没有产品consumer或result route；cut会先删除文档再等待一个实际上没有回执的write，paste也没有可关联的read result。

安全文本的既有P0仍开放且更加确定：WOC密码字段使用`input_kind = "password"`，当前secure classifier却只识别三个布尔metadata key；原文随后进入render command、editable snapshot、component event、binding report和accessibility value。该阻断继续由Runtime11B唯一计数，本报告不新增P0。Runtime77拥有一般dispatch effect非原子P0，本报告只登记编辑域内具体事务差距。结论为 **0项新增P0、48项P1、12项P2、48项资格门**；任何实现不得以再加widget分支、同步系统剪贴板调用或隐藏paint文字冒充工程化闭环。

## 2. 审查边界与可复现冻结

### 2.1 Zircon当前源码冻结

统计规则：路径去重；物理行以文件换行计数；test attributes统计`#[test]`与`#[tokio::test]`；fingerprint为按仓库相对路径排序后，对`path<TAB>file_sha256<LF>`再次做SHA-256。冻结时117个入选文件均未处于dirty状态。

| 范围 | 文件 / 行 / bytes / test attributes | fingerprint | 本轮证据 |
|---|---:|---|---|
| Editing core | 8 / 1,735 / 56,781 / 16 | `2248d8e0bcfa070fc27ba13aa0a1a0da591d756d285d7d5d04c8e6e19e69ec85` | edit action、grapheme/word、caret geometry、hit-test |
| Input / document / IME / clipboard | 20 / 3,801 / 123,475 / 11 | `eb5712bd6504f9193024fa62355e7738fb05eade3b59690f3eeda56bdd8a44ff` | widget dispatch、keyboard/pointer、constraints、focus、host effect |
| Render / component / accessibility | 52 / 6,865 / 223,757 / 14 | `67c328a840e8e49be2052c86275d1e8b2f2e9d683fb82849c047d83d1236619d` | render extract、secondary reducer、semantic value/action |
| Interface / ABI / App host | 21 / 4,292 / 147,707 / 25 | `1366309952dd6be503aead1940098f54b8f77c05a6078372b8c89f7917ee0e34` | public DTO、window event、dynamic ABI、winit/App IME |
| Product ZUI / focused tests | 16 / 5,419 / 196,140 / 117 | `2be19962e42a27a3c078cea44ca4b86bfdcfb4ee9c6d332c65010d26c7704f06` | WOC auth/recovery及IME/keyboard/pointer测试 |
| 去重合计 | **117 / 22,112 / 747,860 / 183** | `4921228228e82e4f1b8164b9b626fd2f8a5c7e1a20596803ab5b1849589c55ac` | 当前专题完整冻结 |
| Production子集 | **96 / 15,713 / 521,889 / 37** | `9720d179a2d271e0328d8899c0646d300c98fe71da2755f84d0f741ef0e9387d` | 排除tests目录、test文件与examples |

### 2.2 参考引擎冻结

| 引擎 | 文件 / 物理行 / 非空行 / bytes | fingerprint | 采用的工程合同 |
|---|---:|---|---|
| Unreal | 9 / 9,042 / 7,413 / 343,468 | `0f538da526615150dd0be9aa9f7241e2671ec80fbdde66d4993da7a67ea0b674` | Slate transaction/undo、preferred cursor offset、TextInputMethod context/notifier/platform implementation |
| Godot | 8 / 27,273 / 22,979 / 992,240 | `07638b007a4ad2da9302992f76d749ee5f7e4e9a2502bc24ab17e28ecfda4b0f` | LineEdit secret/clipboard/IME、TextEdit operation history/version/multi-caret/visual wrap |
| Bevy | 4 / 2,390 / 2,199 / 90,846 | `ae09e18bae0f111d331c1aecc99fcba2cdde2e76419845a778c9a772f358c454` | committed value与preedit分离、Parley editor geometry、async clipboard success-before-cut |
| Fyrox | 1 / 1,824 / 1,706 / 74,982 | `9364b70269a41de63db480f047273d40f6b19791e0dc296593925eb9ae7b4540` | retained formatted text、selection/caret、mask/filter/commit mode下界 |
| 去重合计 | **22 / 40,529 / 34,297 / 1,501,536** | `68303f040321f46de5841868c31aafaefa287cd4741e679f8dc6ffe1d4c3ae94` | 只引用本专题直接相关源文件 |

Unity Graphics本地corpus是render pipeline/post-processing/atlas shader侧源码，不包含Unity TextCore、UI Toolkit text editor或TextMeshPro编辑引擎实现，因此本专题不从该树推断document、IME、clipboard或secure text行为，也不把其样例/shader计入参考冻结。

### 2.3 未执行的验证

本轮是current-source静态review，没有修改production/test源码，没有运行Cargo、App、Editor、真实窗口、真实IME、系统剪贴板、screen reader、密码管理器、移动软键盘、跨平台golden、fuzz、fault injection、soak或benchmark。测试属性数量仅说明现有源码规模，不表示测试已通过或已达到产品资格。

## 3. 当前值得保留的底座

1. `apply_text_edit_action`的插入、backspace与delete围绕grapheme boundary工作，避免了最基础的UTF-8破坏；word/grapheme helper有集中实现，而非散落到每个widget。
2. `hit_test_text_position_with_resolved_layout`与caret geometry能够消费resolved artifact，说明Runtime81的cluster/visual layout authority可以继续向编辑链下沉，不必重写一套文本引擎。
3. `UiEditableTextSnapshot`、`UiTextSelection`、`UiTextComposition`、`UiTextEditAction`和IME/clipboard effect已经形成跨crate可序列化DTO雏形。
4. IME surrounding text按两侧最多256 grapheme并受byte limit约束，且优先使用当前source metrics，不会默认把整篇文档无界复制给平台。
5. winit preedit/commit/delete-surrounding translation与App窗口IME enable/cursor/surrounding-text调用已经存在，平台所有权方向与既定架构一致。
6. focused测试覆盖基本编辑、word shortcut、hard line、pointer selection、preedit/commit/delete surrounding、focus lifecycle和geometry；后续可以从这些behavior test迁移到document/session contract test。

这些底座只能证明“可继续收敛”，不能证明当前widget state就是最终document model，也不能证明host request已经闭合。

## 4. P0状态与唯一归属

### 4.1 本报告不新增P0

本轮没有发现新的compile-time或独立于既有owner的产品数据破坏类别，因此新增P0为0。以下两个阻断仍是当前源码事实，但必须避免跨报告重复计数：

| 仍开放阻断 | 当前证据 | 唯一owner |
|---|---|---|
| Secure字段可显示、复制、事件传播并暴露给accessibility的明文，同时通过禁用IME规避安全输入 | WOC使用`input_kind = "password"`，classifier不识别；render/snapshot/a11y仍复制原文 | Runtime11B secure text P0 |
| dispatch reply的多effect顺序提交，尾项失败不回滚已应用前缀 | text mutation同样受影响，但属于通用input/effect transaction问题 | Runtime77 effect atomicity P0 |

### 4.2 旧计划状态需要纠正

`docs/plans/zircon_runtime/text/08-ime-and-text-input.md`仍把Ctrl+Z/Y、undo/redo、cut/copy/paste描述为已完成。当前`UiTextEditAction`和keyboard mapping没有Undo/Redo，源码中没有edit history；clipboard只有无consumer的request，没有产品read/write result闭环。该计划必须在实现阶段改回`in_progress`的真实子状态，不能用测试中的request shape冒充平台能力。

## 5. P1工程化差距

### 5.1 Document authority、revision与事务

| ID | 当前差距 | 当前源码证据 | 工程级要求 |
|---|---|---|---|
| RTE-P1-001 | Widget属性仍是编辑文档authority | `text_state.rs`每次从TOML metadata与value重建`UiEditableTextSnapshot` | Runtime service拥有稳定document/session，widget只投影snapshot并发送intent |
| RTE-P1-002 | Component reducer形成第二套编辑authority | `state_reducer/text_input.rs`独立实现selection、composition、write-back | reducer必须调用同一document transaction API，不得复制编辑语义 |
| RTE-P1-003 | 没有document id与revision | public snapshot只有text/selection/composition | 所有snapshot、geometry、host request和receipt绑定`TextDocumentId + Revision` |
| RTE-P1-004 | 每个输入事件复制整份`String`和metadata | state重建、render snapshot、component event、binding report均clone原文 | retained buffer、snapshot lease与delta projection，复制量受byte budget约束 |
| RTE-P1-005 | 没有大文档增量存储 | edit action直接对`String`切片/replace | 引入rope/piece-tree等经benchmark选择的数据结构及stable marker |
| RTE-P1-006 | 单次语义编辑不是原子事务 | text/caret/selection/composition以多个generic mutation依次提交 | prepare/validate/commit/publish一体，失败不留下部分状态 |
| RTE-P1-007 | Generic property mutation可绕过文档不变量 | surface/component/accessibility都能分别写text与selection字段 | 唯一typed edit gateway，派生属性只读且由commit同步发布 |
| RTE-P1-008 | 外部model更新没有rebase或stale拒绝 | action无expected revision，hit-test也无generation | 明确replace/rebase/conflict policy并返回typed stale receipt |

### 5.2 History、约束与验证

| ID | 当前差距 | 当前源码证据 | 工程级要求 |
|---|---|---|---|
| RTE-P1-009 | 没有Undo/Redo | action enum、keyboard mapping与state均无history | operation-based history，恢复text/selection/composition前后状态 |
| RTE-P1-010 | 没有transaction grouping/coalescing | 每个key/preedit都独立写属性 | typing、IME commit、paste、accessibility edit有明确group boundary |
| RTE-P1-011 | 没有edit origin与receipt | action只有payload，dispatch结果不描述document mutation | 记录keyboard/pointer/IME/clipboard/a11y/script来源、revision与change range |
| RTE-P1-012 | Constraints全量扫描且静默截断 | max grapheme/filter在replacement前后重扫`String` | 增量验证、typed rejection/truncation reason和可配置policy |
| RTE-P1-013 | 没有只读、protected与marked range | snapshot只有全字段`read_only`语义 | range-level editability与marker随operation稳定迁移 |
| RTE-P1-014 | 只有grapheme count，没有byte/work/memory预算 | retained prefix/suffix与filter成本可随文档增长 | admission同时限制bytes、ops、allocation、deadline与cancel |
| RTE-P1-015 | 输入过滤是简单char predicate | number/email/password等产品语义没有locale grammar | typed input policy，locale-aware parse与intermediate-invalid state |
| RTE-P1-016 | 验证绑定generic component写回 | 无async validator generation或server reconciliation | sync/async validation与document revision绑定，stale result不可覆盖新输入 |

### 5.3 Selection、caret、hit-test与视觉导航

| ID | 当前差距 | 当前源码证据 | 工程级要求 |
|---|---|---|---|
| RTE-P1-017 | 公开selection/composition setter可切开grapheme | 仅用`is_char_boundary` clamp | 所有用户可见range使用cluster/grapheme合法边界与affinity |
| RTE-P1-018 | Left/Right仍按logical byte/grapheme移动 | keyboard helper未消费visual caret stops | 复用Runtime81 visual cluster map，支持BiDi strong/weak caret |
| RTE-P1-019 | Up/Down按source hard line和grapheme column | wrapped visual line、font advance和BiDi均不参与 | 基于paragraph layout visual line与screen-space x导航 |
| RTE-P1-020 | 没有preferred x | 垂直连续移动每次重算grapheme column | caret保留preferred screen offset，水平移动后再更新 |
| RTE-P1-021 | Home/End只认hard line | soft wrap、visual direction与platform convention未建模 | visual-line/text-line/document四类动作显式区分 |
| RTE-P1-022 | Hit result不绑定layout/document generation | pointer用当前render list位置直接写selection | stale geometry拒绝或重新hit-test，receipt记录artifact generation |
| RTE-P1-023 | Double/triple click选择规则过于简化 | word由alphanumeric分类，triple-click只选source hard line | locale/semantic word、visual paragraph/line policy和拖拽扩展规则 |
| RTE-P1-024 | 只有单selection/caret | snapshot为单anchor/focus | Editor/code场景支持multi-caret、rectangular selection与primary caret |

### 5.4 IME composition与平台会话

| ID | 当前差距 | 当前源码证据 | 工程级要求 |
|---|---|---|---|
| RTE-P1-025 | Preedit直接混入committed `String` | composition保存`restore_text`明文副本 | committed document与ephemeral composition overlay分离 |
| RTE-P1-026 | 没有composition session id/generation | SetComposition只有range/text/clauses | activate/begin/update/commit/cancel/end状态机与stale event拒绝 |
| RTE-P1-027 | Preedit cursor只校验UTF-8 boundary | cursor/range可落在grapheme或cluster内部 | 以IME/platform约定映射UTF-16/UTF-8后再落到cluster caret stop |
| RTE-P1-028 | Sanitize改变字符串后直接丢clause | `state_transition.rs`无法保留原range映射 | clause/attribute通过edit mapping重映射，失败返回typed diagnostic |
| RTE-P1-029 | Production preedit clause永远为空 | winit/dynamic session只构造string与cursor | ABI保留clause/attribute/segment并按平台能力降级 |
| RTE-P1-030 | `composition_rects`在host转换时丢失 | UI request有rect list，core `ImeHostRequest`没有 | host contract携带document/session/generation和完整range geometry |
| RTE-P1-031 | IME context事件同步刷新render extract | 每次相关事件调用`refresh_render_extract_for_current_tree`，fallback用uniform advance | document/layout订阅异步生成qualified context，禁止输入线程全树刷新 |
| RTE-P1-032 | Focus loss提交是best-effort且无平台ack | mutation失败被忽略，无cancel/reset receipt | teardown先结束composition并确认host state，再发布focus/session关闭 |

### 5.5 Clipboard与异步host闭环

| ID | 当前差距 | 当前源码证据 | 工程级要求 |
|---|---|---|---|
| RTE-P1-033 | 没有产品clipboard backend | Runtime只生产`UiDispatchHostRequestKind::Clipboard`，App/Editor无consumer | App平台服务拥有clipboard，Runtime通过中立异步合同访问 |
| RTE-P1-034 | Clipboard没有进入dynamic ABI | `ZrRuntimeHostRequestV1`只有IME、rumble、cursor等变体 | versioned request/result ABI并保留unsupported/error语义 |
| RTE-P1-035 | Request无transfer id/revision/result | 只有kind、owner、text | `ClipboardTransferId`、document revision、principal、deadline和typed result |
| RTE-P1-036 | Cut在write成功前删除选择 | keyboard handler先应用delete再发write | 先写clipboard成功，再按expected revision原子删除；失败保留选择 |
| RTE-P1-037 | Paste没有read result应用与关联 | read request发出后无返回通道 | async result路由到原session，stale focus/revision时拒绝或显式rebase |
| RTE-P1-038 | 只支持无类型文本 | 无MIME、primary selection、rich fragment或payload cap | typed offer、plain-text mandatory fallback、size/policy/admission budget |
| RTE-P1-039 | Clipboard没有secure policy | copy/cut不检查secure，paste也无审计 | secure字段默认禁止export，paste遵循principal/field policy并redact receipt |
| RTE-P1-040 | Host请求不绑定window/seat/shortcut policy | owner不能表达platform focus与多窗口身份 | per-window/per-seat host session、平台keymap/command routing和teardown |

### 5.6 Secure text、accessibility与真实产品

| ID | 当前差距 | 当前源码证据 | 工程级要求 |
|---|---|---|---|
| RTE-P1-041 | `input_kind=password`不触发secure | classifier只读取`secure`、`secure_input`、`secureInput`布尔键 | schema编译阶段生成typed `SecureTextPolicy`，未知/冲突配置fail-closed |
| RTE-P1-042 | Render command持有原文 | visible text解析与node visual data写入raw text | paint只接收masked glyph/display projection，不复制secret model |
| RTE-P1-043 | Public editable snapshot含原文和restore副本 | `UiEditableTextSnapshot`序列化`String`与composition | secure snapshot使用opaque handle/redacted projection，禁止跨ABI明文默认序列化 |
| RTE-P1-044 | Accessibility value暴露原文 | semantic extract把value/text直接写入accessible value | password role/value/selection按平台安全规范投影，不泄露字符 |
| RTE-P1-045 | Event/binding/report复制secret | component event、UiValue、binding mutation report携带text | 数据分类贯穿event/log/diagnostic/telemetry，默认redact且禁止Debug泄露 |
| RTE-P1-046 | Secure字段通过禁用IME规避风险 | focus/effect明确拒绝secure IME enable | secure-aware IME context，最小surrounding disclosure与平台能力降级 |
| RTE-P1-047 | 没有secret reveal/store生命周期 | 无last-character reveal timeout、zeroization或secret lease | 可配置reveal、内存/日志最小化、teardown清理及crash dump policy |
| RTE-P1-048 | 真实产品没有安全输入资格 | WOC auth/recovery字段只设`input_kind=password` | App/Editor/WOC跨平台密码、IME、clipboard、a11y与录屏/诊断泄露测试 |

## 6. P2产品完整度差距

| ID | 差距 | 后续方向 |
|---|---|---|
| RTE-P2-001 | Caret形状、宽度、颜色和blink policy仍是控件级静态行为 | theme/accessibility/overwrite mode统一caret presentation policy |
| RTE-P2-002 | 没有overwrite/overtype编辑模式 | document action与caret shape共同表达insert/overwrite |
| RTE-P2-003 | 没有文本drag-and-drop move/copy | selection data offer、drop hit-test与单事务move |
| RTE-P2-004 | 没有primary selection与middle-click paste | 作为平台capability，不污染跨平台核心合同 |
| RTE-P2-005 | Context menu与命令可用态不完整 | 基于selection/read-only/secure/clipboard capability生成command state |
| RTE-P2-006 | 没有spellcheck/autocorrect/grammar标记 | revision-qualified annotation provider，不直接改写document |
| RTE-P2-007 | 没有code/editor language word-boundary policy | 可插拔navigation/token policy与Unicode默认回退 |
| RTE-P2-008 | 没有候选窗、手写和语音输入诊断面 | platform session暴露低基数能力与失败原因，不复制secret content |
| RTE-P2-009 | 没有rich text/structured fragment clipboard | typed fragment schema、plain fallback与schema/version compatibility |
| RTE-P2-010 | 没有multi-stage composition可视化与annotation样式合同 | clause/segment style由platform attribute映射到presentation artifact |
| RTE-P2-011 | 没有文本编辑性能与泄露telemetry | 只记录长度桶、延迟、revision conflict、redaction状态等低基数指标 |
| RTE-P2-012 | 移动端virtual keyboard/autofill/content type策略未闭合 | per-field input purpose、return key、autocap、autofill与secure entitlement |

## 7. 目标架构与所有权

### 7.1 固定所有权

| 层 | 必须拥有 | 不得拥有 |
|---|---|---|
| `zircon_runtime` text service | document storage/revision、edit transaction/history、selection/composition session、constraint/validation、layout binding、secure policy、clipboard intent | OS window handle、系统剪贴板对象、平台IME对象 |
| `zircon_runtime_interface` | versioned neutral snapshot/intent/receipt、host request/result、redacted secure projection | 平台句柄、widget TOML真值、无版本裸String协议 |
| Runtime UI | widget projection、command enablement、pointer/keyboard intent、paint/a11y projection | 第二份document、独立undo stack、直接系统调用 |
| `zircon_app` | per-window/per-seat IME与clipboard adapter、platform keymap、request/result routing、teardown | 文档mutation语义、密码原文日志、widget classifier |
| `zircon_editor` | code/authoring document consumer、multi-caret/product workflow、diagnostic UI | 独立平台IME/clipboard真值、复制Runtime编辑器核心 |

### 7.2 核心合同

```text
TextDocumentId + TextDocumentRevision
    -> TextDocumentSnapshot / TextSelectionSet / TextCompositionSession
    -> TextEditIntent(expected_revision, origin, operations, grouping)
    -> TextEditTransaction::prepare / validate / commit
    -> TextEditReceipt(new_revision, changed_ranges, selection, history_state)
    -> DocumentLayoutSession(document_revision, layout_generation)
    -> CaretHit(document_revision, layout_generation, cluster, affinity)

TextInputHostSession(window, seat, document, secure_policy)
    -> ImeHostRequest(session_generation, surrounding_projection, geometry)
    <- ImeHostEvent(session_generation, composition_id, clauses, commit/delete)

ClipboardTransferId + ClipboardOffer + ClipboardPolicy
    -> ClipboardHostRequest
    <- ClipboardHostResult
    -> revision-qualified document transaction
```

建议最小类型集合为：`TextDocumentId`、`TextDocumentRevision`、`TextDocumentSnapshot`、`TextEditOperation`、`TextEditTransaction`、`TextEditOrigin`、`TextEditReceipt`、`TextSelectionSet`、`VisualCaretAffinity`、`TextCompositionSession`、`TextInputHostSession`、`SecureTextPolicy`、`SecureTextProjection`、`ClipboardTransferId`、`ClipboardOffer`、`ClipboardHostResult`。名称可调整，但identity、revision、transaction、generation、redaction与result语义不可删除。

### 7.3 强制不变量

1. Committed document从不包含ephemeral preedit；composition commit只能通过document transaction产生新revision。
2. Selection、caret、composition、hit result和layout artifact必须能证明同一document revision；stale input不静默写入。
3. 一次edit要么同时发布text、selection、composition/history变化，要么完全不发布。
4. Undo/Redo恢复语义operation及selection，不依赖整个widget metadata快照覆盖。
5. Clipboard cut必须在host write成功后、仍满足expected revision时删除；paste result必须可关联且可拒绝过期session。
6. Secure text原文不进入render command、accessibility value、binding/event/report、Debug、trace或默认ABI serialization。
7. 平台IME与clipboard adapter只在App层持有OS对象；Runtime合同保持window/seat/session-qualified且跨平台中立。
8. 所有大文档、surrounding text、clipboard、history、validation和layout联动路径都有byte/work/deadline/cancel预算。

## 8. 分层实施里程碑

| Milestone | 内容 | 完成定义 |
|---|---|---|
| M0 Truthful security | 修正typed password schema、mask/render/a11y/event/clipboard/IME策略 | WOC真实字段不泄露且不再靠禁用IME |
| M1 Document authority | 建立document id/revision、retained buffer、snapshot lease与唯一service | widget/reducer/accessibility不再直接写多属性真值 |
| M2 Transaction/history | operation、prepare/commit/receipt、undo/redo/coalescing、validation | fault injection证明无partial mutation且history可重放 |
| M3 Selection/geometry | visual caret stops、affinity/preferred-x、wrapped/BiDi navigation、stale hit拒绝 | 与Runtime81同artifact的跨脚本golden一致 |
| M4 IME session | committed/preedit分离、composition lifecycle、clause/geometry、host generation | Windows/macOS/Linux真实IME生命周期与teardown通过 |
| M5 Clipboard host | versioned request/result ABI、async transfer、success-before-cut、policy | App/Editor/WOC copy/cut/paste及故障/焦点漂移通过 |
| M6 Product convergence | UI、component reducer、accessibility、Editor统一consumer | 删除重复authority和string classifier，产品只有一条路径 |
| M7 Scale/platform | large document、multi-caret、mobile input purpose、budgets、telemetry | million-character、multi-window/seat、soak/fuzz受控 |
| M8 Qualification | reference workload、cross-platform golden、security/fault/perf receipt | 同负载数据可复现；性能优于Unreal只能由receipt证明 |

M0不是临时mask补丁：它必须先建立能够贯穿所有projection的secure policy和redaction边界。M1-M6完成前，不得把新增rich editor或code editor功能直接堆入现有widget state。

## 9. 资格门

### 9.1 Security与当前产品真值门

| Gate | 必须满足 |
|---|---|
| RTE-GATE-001 | WOC auth/recovery所有`input_kind=password`编译为typed secure policy |
| RTE-GATE-002 | secure字段paint、render command和GPU capture不含可恢复原文 |
| RTE-GATE-003 | accessibility tree、screen reader event与action result不含密码原文 |
| RTE-GATE-004 | copy/cut/drag/export默认拒绝secure内容且有redacted receipt |
| RTE-GATE-005 | component event、binding report、diagnostic、trace与panic格式不含secure原文 |
| RTE-GATE-006 | secure字段在支持平台可使用IME，surrounding disclosure遵循最小policy |
| RTE-GATE-007 | reveal-last-character timeout、focus loss、teardown与crash policy有自动测试 |
| RTE-GATE-008 | password manager/autofill/content purpose跨App、Editor、WOC产品实测通过 |

### 9.2 Document与transaction门

| Gate | 必须满足 |
|---|---|
| RTE-GATE-009 | 每个编辑surface绑定稳定document id/revision，重复widget实例不串状态 |
| RTE-GATE-010 | text/selection/composition/history一次commit原子发布，任一点失败零可见变化 |
| RTE-GATE-011 | stale expected revision返回typed conflict且不能覆盖新文档 |
| RTE-GATE-012 | keyboard/IME/paste/a11y/script edit均生成origin与change-range receipt |
| RTE-GATE-013 | Undo/Redo跨typing coalescing、selection replace、IME commit和paste正确恢复 |
| RTE-GATE-014 | 外部model update与本地pending edit按显式replace/rebase/conflict policy处理 |
| RTE-GATE-015 | protected/read-only range、marker和selection在insert/delete/undo后保持不变量 |
| RTE-GATE-016 | million-character document edit延迟、allocation与RSS满足冻结阈值 |

### 9.3 Unicode selection与visual geometry门

| Gate | 必须满足 |
|---|---|
| RTE-GATE-017 | 所有selection/composition boundary不能切开grapheme或非法cluster caret stop |
| RTE-GATE-018 | mixed LTR/RTL的Left/Right按visual policy并正确表达affinity |
| RTE-GATE-019 | wrapped proportional text的Up/Down保持preferred screen x |
| RTE-GATE-020 | Home/End明确区分visual line、hard line与document boundary |
| RTE-GATE-021 | pointer hit、drag、double/triple click消费同一qualified layout artifact |
| RTE-GATE-022 | stale layout generation的hit result不能提交selection mutation |
| RTE-GATE-023 | Latin/CJK/Indic/Arabic/Hebrew/emoji/combining sequence selection golden通过 |
| RTE-GATE-024 | Editor multi-caret/rectangular selection与primary caret规则有确定性测试 |

### 9.4 IME session门

| Gate | 必须满足 |
|---|---|
| RTE-GATE-025 | preedit不改变committed document revision，commit只产生一次operation |
| RTE-GATE-026 | activate/begin/update/commit/cancel/end及focus/window teardown状态机完整 |
| RTE-GATE-027 | stale composition/session generation事件被拒绝并产生低基数diagnostic |
| RTE-GATE-028 | cursor、clause、attribute和range完成平台offset到cluster映射 |
| RTE-GATE-029 | composition rects端到端穿过Runtime Interface、dynamic ABI与App host |
| RTE-GATE-030 | surrounding text受grapheme/byte/security policy约束并绑定document revision |
| RTE-GATE-031 | IME事件不触发同步全树render refresh，context生成受deadline/cancel控制 |
| RTE-GATE-032 | Windows TSF、macOS、Linux IME真实组合/候选/删除/焦点golden通过 |

### 9.5 Clipboard、accessibility与host门

| Gate | 必须满足 |
|---|---|
| RTE-GATE-033 | App存在真实clipboard backend且dynamic/static Runtime都走同一中立合同 |
| RTE-GATE-034 | 每次read/write有transfer id、window/seat/session、deadline与typed result |
| RTE-GATE-035 | write失败时cut不删除；成功后revision变化时不错误删除新内容 |
| RTE-GATE-036 | paste result在focus/session/revision过期时按显式policy拒绝或rebase |
| RTE-GATE-037 | large/invalid/MIME-rich clipboard payload受size/schema/admission限制 |
| RTE-GATE-038 | screen reader replace/selection动作与keyboard edit共用transaction gateway |
| RTE-GATE-039 | multi-window/seat clipboard与IME teardown不向错误surface投递结果 |
| RTE-GATE-040 | unsupported/permission-denied/timeout/cancel均有用户可解释result而非silent drop |

### 9.6 Scale、故障与性能资格门

| Gate | 必须满足 |
|---|---|
| RTE-GATE-041 | history/source/snapshot/clipboard/composition resident bytes均有owner与上限 |
| RTE-GATE-042 | constraints/validation只处理依赖range，恶意replacement不能触发无界工作 |
| RTE-GATE-043 | document/window/session teardown回收history、marker、layout和host pending transfer |
| RTE-GATE-044 | fuzz覆盖UTF-8/UTF-16 offset、grapheme、operation sequence、undo和IME乱序 |
| RTE-GATE-045 | fault注入覆盖allocation、host disconnect、clipboard timeout、platform IME reset |
| RTE-GATE-046 | App、Editor、WOC真实产品路径无test-only backend或第二套state authority |
| RTE-GATE-047 | 同文本/字体/IME/clipboard/文档规模对比Unreal记录CPU/RSS/p50/p95/p99 |
| RTE-GATE-048 | qualification artifact绑定source/reference fingerprint、平台、依赖、阈值和non-ignored结果 |

## 10. 跨报告owner与去重边界

| 主题 | 唯一owner | Runtime82只负责 |
|---|---|---|
| Font blob/fallback/glyph resolution | Runtime80、Runtime11B | 编辑会话如何引用resolved font/layout，不复制font问题 |
| Shaping/BiDi/line layout/cluster artifact | Runtime81 | selection/caret/hit如何消费artifact及stale规则 |
| UI painter order/icon/atlas/GPU submit | Runtime79、Runtime11C | secure display projection不得携带原文 |
| 通用input dispatch/effect原子性、window lifecycle | Runtime77 | document mutation transaction与host result关联 |
| 通用accessibility platform adapter | Runtime78 | secure value、text range与edit action接入document gateway |
| Widget/component catalog行为 | Runtime75 | 删除text input双authority并接统一service |
| Runtime/App动态ABI版本与session lifecycle | Interface01/07、App01、Runtime43 | 文本/IME/clipboard所需qualified contract |

## 11. 禁止的临时修补

- 不得只给`resolve_editable_text_value`返回圆点字符串；原文仍会从snapshot、event、binding和accessibility泄露。
- 不得继续以禁用IME作为secure input实现；密码输入必须有secure-aware platform session。
- 不得在keyboard handler直接调用Windows clipboard；平台对象属于App，且必须保留异步result与跨平台合同。
- 不得只把Undo/Redo action加进enum而继续保存整份widget TOML快照；history必须基于document operation/revision。
- 不得用更多`is_char_boundary` clamp冒充Unicode selection正确；必须使用Runtime81的cluster/grapheme authority。
- 不得让Up/Down继续按grapheme column猜测比例字体、wrap和BiDi视觉位置。
- 不得用同步refresh整个render tree保证IME geometry“最新”；使用generation-qualified artifact与stale policy。
- 不得把clipboard request被测试捕获当成产品集成；必须有App consumer、ABI result、故障语义与真实系统测试。
- 不得以test attribute数量、source string guard或单个平台手测关闭任何资格门。
- 不得在字体、平台、负载、document规模、warm/cold状态不一致时声称性能优于Unreal。

## 12. 本轮产出边界

本文只完成current-source静态review、物理冻结、参考对照、差距登记、目标架构与资格合同；没有实施Rust/Cargo/ZUI修改。建议下一专题转向Runtime UI animation/timeline或继续按未覆盖物理域推进；进入文本实现时必须从M0 secure truth与M1 document authority开始，不能先向现有widget state追加rich-editor功能。工具链按用户要求暂不纳入本轮优化专题。
