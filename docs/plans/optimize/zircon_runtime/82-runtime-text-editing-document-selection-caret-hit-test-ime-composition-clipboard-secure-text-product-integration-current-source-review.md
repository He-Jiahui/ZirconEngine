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
  - zircon_runtime_interface/src/runtime_api/host/host_requests.rs
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
implementation_status: in_progress
source_recheck_required: true
---

# Runtime Text Editing、Document、Selection、Caret、Hit Test、IME Composition、Clipboard、Secure Text 与 Product Integration 当前源码工程化差距

## 2026-08-29 NumberField invariant edit-session update

NumberField 已完成 invariant MVP edit session：canonical `value` 保持 Float，活动文本进入
`value_text` 并由 `number_edit_active` 控制 input/render；共享 property transaction 一次
preflight/commit typed value、buffer、active flag 和 caret/selection/composition，不增加第二 authority。
V1 content-free receipt 固定 format/parse/commit 状态。默认 per-key 只改 buffer，显式 opt-in 仅对完整
finite/range-valid 输入发布 Float Change；Enter/blur/Escape 统一 parse、clamp、optional snap 并发布
Float Commit、保留非法 Enter 或恢复失焦/Escape。IME commit 只结束 preedit，不等价于字段提交。

源码回归覆盖 buffer、typed clamp commit、intermediate reject、cancel 和 blur restore；Rustfmt/diff check
通过。managed Cargo、平台 IME/a11y/clipboard、WGPU/PNG、profile/power、locale type interface、focused
numeric refresh 和 format cache 仍开放。状态更新为
`number_field_invariant_edit_session_implemented_unvalidated /
typed_numeric_publication_implemented_unvalidated /
number_field_commit_cancel_blur_policy_implemented_unvalidated /
managed_runtime_platform_wgpu_profile_pending`。

## 1. 结论

当前文本输入链已经不是纯占位：编辑操作按grapheme执行插入和删除，selection/caret/preedit有中立DTO，pointer hit-test能够优先复用resolved layout，IME surrounding text有grapheme与byte上限，App也会把winit IME事件和cursor/surrounding-text请求接到平台窗口。这些底座应保留。

但产品级authority仍停留在“控件属性里放一份`String`”的阶段。widget input仍从TOML metadata重建编辑状态；2026-08-27 current source 已将其 value/caret/selection/composition 写回收敛为一次轻量 prepare/commit，关闭 reserved value property 导致的半提交，并按语义 value 统一 text dirty/revision。accessibility与generic external正文更新已进入同一事务，generic派生编辑字段单写也已fail closed；Material editable descriptor的raw `KeyboardText`与component reducer内第二套edit-state/write-back随后删除，reducer只消费semantic ValueChanged/Commit/Focus，当前surface/component直写旁路均已静态收敛。仍没有产品document identity、revision、operation receipt、undo/redo、focused bound-text policy或外部编辑rebase。selection公开入口只保证UTF-8 char boundary，垂直移动、Home/End和triple-click仍按source hard line工作，无法以Runtime81产出的visual cluster/line geometry作为唯一真值。

IME链在UI层拥有composition rects和surrounding text，但转为core host request时丢掉composition rects；动态ABI只携带preedit string/cursor并在production构造空clause。composition直接写入可提交`String`并保存明文restore副本，focus-loss提交又是best-effort。剪贴板的当前源码已在2026-08-27补上runtime transfer/revision/result transaction，cut不再先删除，paste可在匹配回执上走约束化编辑；但App、Editor和动态ABI仍没有产品consumer/result producer，surface-local edit revision也不是产品document revision，因此产品闭环仍开放。

安全文本的既有P0仍开放，但2026-08-27 current source已关闭最前端classifier绕过，并完成未验收的输入结果投影：WOC `input_kind=password`进入唯一internal typed policy；secure Change/Submit改发surface-owned opaque reference，input/effect/host/component/binding/action result统一redact。focused bound-refresh pending store现也会在overwrite/discard/clear/Drop时zeroize allocation，但这只覆盖一个transient owner。P0仍不能关闭，因为WOC尚无受信reference consumer，retained state/history/export/crash路径和端到端zeroization未收敛，secure focus仍禁用IME，公开versioned policy/host session、真实WOC/Cargo/capture均未完成。该阻断继续由Runtime11B唯一计数，本报告不新增P0。Runtime77拥有一般dispatch effect非原子P0，本报告只登记编辑域内具体事务差距。结论仍为 **0项新增P0、48项P1、12项P2、48项资格门**；任何实现不得以再加widget分支、同步系统剪贴板调用或隐藏paint文字冒充工程化闭环。

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
| Secure字段的classifier/render/a11y/clipboard与input-result事件投影已收敛，但retained secret owner、受信host consumer和secure IME仍缺 | `input_kind=password`进入shared policy；Change/Commit使用opaque reference且dispatch result统一redact；WOC未消费该引用，surface state仍持有原文 | Runtime11B secure text P0 |
| dispatch reply的多effect顺序提交，尾项失败不回滚已应用前缀 | text mutation同样受影响，但属于通用input/effect transaction问题 | Runtime77 effect atomicity P0 |

### 4.2 旧计划状态需要纠正

`docs/plans/zircon_runtime/text/08-ime-and-text-input.md`仍把Ctrl+Z/Y、undo/redo、cut/copy/paste描述为已完成。当前`UiTextEditAction`和keyboard mapping没有Undo/Redo，源码中没有edit history；clipboard只有无consumer的request，没有产品read/write result闭环。该计划必须在实现阶段改回`in_progress`的真实子状态，不能用测试中的request shape冒充平台能力。

## 5. P1工程化差距

### 5.1 Document authority、revision与事务

| ID | 当前差距 | 当前源码证据 | 工程级要求 |
|---|---|---|---|
| RTE-P1-001 | Widget属性仍是编辑文档authority | `text_state.rs`每次从TOML metadata与value重建`UiEditableTextSnapshot` | Runtime service拥有稳定document/session，widget只投影snapshot并发送intent |
| RTE-P1-002 | Current-source直写旁路已关闭、动态验收待完成 | Material editable descriptor不再声明raw `KeyboardText`；`state_reducer/text_input.rs`已删除selection/composition/edit write-back，reducer只消费semantic event做mirror/validation | 保持Surface/document transaction为唯一raw edit owner；component层只能消费已提交semantic projection，不得恢复编辑语义 |
| RTE-P1-003 | 产品合同没有document id与revision；2026-08-27新增的crate-private底座已有owner+revision但未接任何surface/service；surface临时layout key已修复revision exhaustion但不是产品document authority | public snapshot仍只有text/selection/composition；内部`text/document`不能证明产品authority，`node_id + layout_revision`也不能替代服务分配的document identity | 所有snapshot、geometry、host request和receipt绑定`TextDocumentId + Revision` |
| RTE-P1-004 | 每个产品输入事件仍复制整份`String`和metadata | state重建、render snapshot、component event、binding report均clone原文；内部piece owner未被消费 | retained buffer、snapshot lease与delta projection，复制量受byte budget约束 |
| RTE-P1-005 | 产品路径没有大文档增量存储；内部piece storage现有separator-aware stable hard-line ID与局部edit envelope，但grapheme index仍全文snapshot重建 | edit action仍直接对`String`切片/replace；内部line owner、stable marker与paragraph reflow均未接入产品 | 经benchmark选择并接入唯一authority的数据结构、stable marker与bounded index repair；内部line ID只能算前置，不得冒充M1完成 |
| RTE-P1-006 | current widget/accessibility/generic external surface 多属性半提交已关闭，产品document事务仍开放；内部document replace已有prepare-before-mutation与typed `Unchanged/Changed` | keyboard/text/IME/clipboard、a11y SetValue/Replace/Selection与generic外部正文更新以固定十项prepare/commit原子发布value/caret/selection/composition，reserved property与非法state零写入，value change只登记一次text revision；authority仍是TOML metadata且无产品receipt/history消费者 | 接入唯一document authority的prepare/validate/commit/publish，fault injection证明失败零发布；history只消费`Changed`receipt |
| RTE-P1-007 | Current surface generic与component raw edit绕过已关闭，产品document gateway仍开放 | accessibility保留`AccessibilityAction`来源进入共享事务；generic正文更新也进入同一事务，派生caret/selection/composition单写拒绝，NumberField外部Float保留类型；component只消费semantic projection，但仍无focused binding/rebase政策和产品document owner | 唯一typed edit gateway，派生属性只读且由commit同步发布；外部model写入必须消费同一document transaction |
| RTE-P1-008 | 产品外部model更新仍没有rebase或stale拒绝；内部replace已要求expected key并typed拒绝stale/exhaustion；surface layout revision也已在耗尽时停止发布retained key | widget action与hit-test仍无document revision/generation，两个内部合同均无产品edit消费者 | 明确replace/rebase/conflict policy并把typed stale receipt贯穿唯一edit gateway |

### 5.2 History、约束与验证

| ID | 当前差距 | 当前源码证据 | 工程级要求 |
|---|---|---|---|
| RTE-P1-009 | 没有Undo/Redo | action enum、keyboard mapping与state均无history | operation-based history，恢复text/selection/composition前后状态 |
| RTE-P1-010 | 没有transaction grouping/coalescing | 每个key/preedit都独立写属性 | typing、IME commit、paste、accessibility edit有明确group boundary |
| RTE-P1-011 | 没有edit origin与receipt | action只有payload，dispatch结果不描述document mutation | 记录keyboard/pointer/IME/clipboard/a11y/script来源、revision与change range |
| RTE-P1-012 | Constraints仍对当前prefix/suffix做全文grapheme计数，但过滤/单行移除/容量截断已不再静默 | replacement以单趟filter+canonical hard-line admission生成`UiTextInputConstraintReceipt`，accepted buffer原地截断；keyboard/text/IME/a11y共享该结果，`max_length=0`恢复不限长 | 将retained document/index接入增量验证；补byte/work/deadline预算、typed policy与managed规模profile |
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
| RTE-P1-028 | current-source约束映射已关闭；不得回退为sanitize后直接丢clause或用旧offset解释新串 | sanitizer单趟记录cursor/clause实际引用的UTF-8边界，cursor再clamp到grapheme；range移动/完全删除均有typed receipt，合法空clause保留 | 保持filter/CRLF/truncation/空clause回归；平台UTF-16/ACP转换归host，真实clause producer归RTE-P1-029 |
| RTE-P1-029 | Production preedit clause永远为空 | winit/dynamic session只构造string与cursor | ABI保留clause/attribute/segment并按平台能力降级 |
| RTE-P1-030 | `composition_rects`在host转换时丢失 | UI request有rect list，core `ImeHostRequest`没有 | host contract携带document/session/generation和完整range geometry |
| RTE-P1-031 | IME context事件同步刷新render extract | 每次相关事件调用`refresh_render_extract_for_current_tree`，fallback用uniform advance | document/layout订阅异步生成qualified context，禁止输入线程全树刷新 |
| RTE-P1-032 | Focus loss属性半提交已关闭，但仍无平台ack | composition commit复用同一surface property transaction，prepare失败不写属性或发commit event；仍无host cancel/reset receipt | teardown先结束composition并确认host state，再发布focus/session关闭 |

### 5.5 Clipboard与异步host闭环

| ID | 当前差距 | 当前源码证据 | 工程级要求 |
|---|---|---|---|
| RTE-P1-033 | 没有产品clipboard backend | Runtime只生产`UiDispatchHostRequestKind::Clipboard`，App/Editor无consumer | App平台服务拥有clipboard，Runtime通过中立异步合同访问 |
| RTE-P1-034 | Clipboard没有进入dynamic ABI | `ZrRuntimeHostRequestV1`只有IME、rumble、cursor等变体 | versioned request/result ABI并保留unsupported/error语义 |
| RTE-P1-035 | runtime transfer合同已补，产品session合同仍不完整 | request已有UUID transfer/intent/surface-local edit revision，入站有typed read/write/failure outcome与receipt；无principal/deadline/ABI/product document identity | 接入`TextDocumentId + Revision`、principal、deadline和versioned host ABI，保留当前typed result |
| RTE-P1-036 | 当前keyboard先删后写已修复，产品原子性仍未验收 | cut只在匹配WriteText成功回执后执行Delete；失败、stale、wrong owner/outcome不改文本；App尚无真实write producer | 真实host write成功后按产品document revision原子删除；fault/timeout/teardown证明失败保留选择 |
| RTE-P1-037 | runtime paste result route已补，产品read producer仍缺 | ReadText回执按transfer/owner/edit revision匹配并走共享constraint owner；duplicate、clone/serde、focus/edit stale fail closed | App/ABI把async result路由到原window/session，补timeout、cancel、rebase与真实系统测试 |
| RTE-P1-038 | 只支持无类型文本 | 无MIME、primary selection、rich fragment或payload cap | typed offer、plain-text mandatory fallback、size/policy/admission budget |
| RTE-P1-039 | internal secure clipboard policy已部分收敛，公开session policy仍缺 | WOC-shaped password由统一classifier禁止copy/cut；secure paste result与公开dispatch payload统一redact，但无principal/window policy和产品审计 | secure字段默认禁止export，paste遵循principal/field policy并发布无原文receipt |
| RTE-P1-040 | Host请求不绑定window/seat/shortcut policy | owner不能表达platform focus与多窗口身份 | per-window/per-seat host session、平台keymap/command routing和teardown |

### 5.6 Secure text、accessibility与真实产品

| ID | 当前差距 | 当前源码证据 | 工程级要求 |
|---|---|---|---|
| RTE-P1-041 | internal classifier前置已关闭，public/versioned secure policy仍缺失 | TextField catalog声明input-kind enum；唯一`UiSecureTextPolicy`识别password/type/secure aliases，冲突/畸形/未知fail-closed；尚未进入Runtime Interface/host session | 将policy升级为document/session-qualified中立合同并迁移event/binding/IME，而不是恢复route-local TOML解析 |
| RTE-P1-042 | Render command持有原文 | visible text解析与node visual data写入raw text | paint只接收masked glyph/display projection，不复制secret model |
| RTE-P1-043 | Public editable snapshot含原文和restore副本 | `UiEditableTextSnapshot`序列化`String`与composition | secure snapshot使用opaque handle/redacted projection，禁止跨ABI明文默认序列化 |
| RTE-P1-044 | Accessibility value暴露原文 | semantic extract把value/text直接写入accessible value | password role/value/selection按平台安全规范投影，不泄露字符 |
| RTE-P1-045 | input dispatch投影已关闭，跨系统secret分类仍开放 | secure Change/Submit发布latest surface-owned opaque reference；input/effect/host/component/binding/action result统一redact，clone/serde不复制lease；日志、crash、plugin/export与WOC host consumer未验收 | 将同一分类扩展到session/log/diagnostic/telemetry/export并以managed corpus证明无Debug/serde泄露 |
| RTE-P1-046 | Secure字段通过禁用IME规避风险 | focus/effect明确拒绝secure IME enable | secure-aware IME context，最小surrounding disclosure与平台能力降级 |
| RTE-P1-047 | secret reveal/store生命周期仅关闭pending子集 | focused bound-refresh pending值使用`Zeroizing<String>`，覆盖、拒绝、切换与teardown擦除；component/document/history/layout/platform/crash仍为普通明文owner，且无last-character reveal timeout | 可配置reveal、统一secure document/session custody、全owner内存/日志最小化、teardown与crash dump policy |
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

原始冻结只完成current-source静态review。2026-08-27 follow-up新增内部revision基础设施硬切：expected key成为replace必填输入，stale key与revision exhaustion均在mutation前typed fail closed，并修正多行replacement byte-delta回归。后续内部前置已建立separator-aware stable hard-line ID：edit只重扫带前后context的局部line envelope，split新增ID、merge保留左ID，并发布old/new reanalyzed ordinal span；grapheme index仍全文重建。内部replace现也以跨piece、allocation-free range equality区分typed `Unchanged/Changed`：相同source不推进revision、不追加chunk、不重扫hard line、不失效grapheme index，MAX revision下的no-op仍合法；真实变化的late mismatch额外比较成本须进入edit-scale profile。surface临时layout revision也已从wrap改为不可发布的exhausted sentinel，耗尽只禁用retained reuse而不丢失layout。该surface key仍只是`node_id + layout_revision`，内部document owner也未接UI/service/public snapshot，二者都不能视为M1 Document authority完成；stable marker、paragraph reflow、history grouping、external model rebase、managed Cargo和产品验证仍开放。进入文本实现仍必须从M0 secure truth与M1 document authority收敛，不能先向现有widget state追加rich-editor功能。

同日输入约束follow-up把filter、canonical hard-line移除和max-grapheme截断收敛为共享typed `UiTextInputConstraintReceipt`，贯穿keyboard/text/IME与accessibility，并修复catalog默认`max_length=0`被热路径误解释为零容量的问题。constrained preedit现以只保存cursor/clause实际端点的UTF-8 byte edit map重映射range；cursor再落到grapheme boundary，完全删除的非空clause发布typed drop count，合法空clause保留。single-line Enter也按Unreal与Editor19合同硬切为handled Submit：不写十个unchanged属性、不伪造newline约束receipt，repeat只消费不重复commit。当前文档prefix/suffix仍重复计数grapheme，production平台clause仍为空。状态为`typed_constraint_receipt_implemented / canonical_single_line_separators_implemented / zero_max_length_unbounded_contract_restored / constrained_preedit_edit_mapping_implemented / single_line_enter_submit_implemented / incremental_validation_open / platform_clause_producer_open / managed_validation_pending`；RTE-P1-028的current-source sanitize mapping已关闭，RTE-P1-012与RTE-P1-029继续开放。

同日secure classifier follow-up为TextField声明typed `input_kind` enum，并建立唯一internal `UiSecureTextPolicy`供render与input/a11y/clipboard/IME分类入口消费。WOC形态`input_kind=password`不再绕过masked render、a11y redaction和clipboard deny；畸形secure alias、未知/非字符串input kind fail-closed。该前置随后由下一段event projection继续收敛；secure focus、public/versioned policy与host session仍缺，因此Runtime11B P0和Runtime82 M0保持open。

随后secure event projection把安全字段Change/Submit硬切为`UiSecureTextValueRef`，由surface保存每个node/property最新UUID lease并用非耗尽layout revision校验；下一事件、跨surface、clone/serde、policy变化或revision exhaustion均fail closed。普通dispatch和direct reply出口统一清洗原始Text/Keyboard/IME/a11y输入、binding previous/value、reply/applied/rejected effect、IME surrounding、clipboard write、host/component report和template action payload，并发布typed `secure_text_redacted`证据。WOC仍没有受信consumer，retained state/history/export/crash/zeroization与secure IME session仍开放，因此P0/M0不关闭。状态为`secure_event_projection_implemented_unvalidated / latest_reference_fence_implemented / trusted_host_session_open / secure_ime_session_open / managed_validation_pending`。

同日clipboard transaction follow-up新增UUID transfer、intent、surface-local edit revision、typed result/failure与receipt，并把host completion作为中立`UiInputEvent::Clipboard`路由。keyboard cut不再提前删除，只有匹配的WriteText成功回执才提交Delete；paste ReadText回执复用共享constraint/edit owner。相关edit/focus/policy变化使revision stale，unknown/duplicate/wrong owner/outcome、clone/serde/detach均fail closed；每editable owner最多一个pending且不保存全文，无pending的普通编辑不更新revision map。该实现尚无App/Editor producer和dynamic ABI，surface revision也不等于产品document revision，因此RTE-P1-033至040产品簇继续open。状态为`runtime_transfer_contract_implemented_unvalidated / cut_delete_after_write_ack_implemented_unvalidated / paste_result_route_implemented_unvalidated / app_backend_open / dynamic_abi_open / managed_validation_pending`。

同日editable property transaction follow-up关闭当前widget input、accessibility与generic external正文入口的逐属性半提交：动态value property、node/metadata和完整grapheme state先prepare，失败零写入；成功以固定十项batch提交value/caret/selection/composition，统一同步style/component/binding并只登记一次dirty与clipboard revision。a11y SetValue/Replace/Selection保留`AccessibilityAction`来源进入同一事务，旧8–9份binding report收敛为一份，并补齐composition clauses清理。generic外部正文按Unreal `SetEditableText`的保守子集保留合法caret或按grapheme clamp，清selection/composition后一次提交；同值no-op，派生编辑字段单写拒绝。存储`UiValue`与显示文本分离，NumberField外部Float保持数值类型。value变化按语义角色强制layout/text/render dirty，任意业务属性名不再漏掉reshape；caret/selection-only不推进text revision。focus-loss composition复用同一边界。该实现没有全树snapshot，候选属性在栈上，只有变化字段分配记录；限定Windows no-run Cargo在184秒无输出后超时终止，不能算通过。产品document authority、edit origin/change-range receipt、history/rebase、focused bound-text policy、host ack与数值内部编辑parse/commit仍未收敛，managed profile/WGPU未执行；component reducer旁路由下一段继续关闭。状态为`surface_property_prepare_commit_implemented_unvalidated / widget_and_accessibility_projection_converged_unvalidated / generic_external_projection_converged_unvalidated / derived_property_write_bypass_closed_unvalidated / numeric_external_value_type_preserved_unvalidated / partial_metadata_write_path_closed / composition_clause_clear_path_closed / semantic_text_dirty_invalidation_implemented / product_document_transaction_open / managed_validation_pending`。

随后component authority follow-up把Material editable descriptor统一硬切为`Focus/ValueChanged/Commit` semantic事件，不再声明raw `KeyboardText`；`state_reducer/text_input.rs`删除独立正文、caret、selection、composition重建与write-back约315行，菜单typeahead/command palette不受影响。SearchField、FieldEditor、SourceEditor复用同一text-input descriptor合同；`UiWidgetBehavior`补齐完整role/component alias，V1 compiler按`query -> value -> value_text -> text`推断canonical value property且保留显式override，Surface fallback同步识别`value_text`。FieldEditor因此不会再写`text`却让validation读取旧`value_text`。该推断在编译/状态边界完成，不进入逐grapheme/glyph热循环。Rust 2024 formatter、scoped diff check与production reference scan通过；限定Windows interface窄测在64秒仍处依赖编译，终止对应Cargo/Rustc后无测试结果，不能算测试通过。后续`cargo check -p zircon_runtime_interface --lib`复用E盘target并在114.5秒通过，只有9项既有warning；runtime与测试执行仍未验证。`RTE-P1-002`当前源码旁路静态关闭，M1/M2、RTE-P1-006/007、focused binding、numeric parse/commit、managed profile/WGPU仍开放。状态为`component_raw_keyboard_edit_authority_removed_unvalidated / canonical_editable_value_property_inference_implemented_unvalidated / semantic_component_projection_only_unvalidated / interface_lib_compile_passed / managed_validation_pending`。

focused bound-text policy follow-up对照Unreal `SlateEditableTextLayout.cpp:3622-3636,4508-4547`确认：普通bound refresh在聚焦时不得替换editable text，只有显式SetText/LoadText式force review可以覆盖并修正caret。当前Zircon mutation request只有反射source与下游binding source kind，不能表达bound refresh、显式replace和edit projection的差异；Surface metadata还同时承担model value与edit buffer。直接覆盖会丢用户编辑，focus early-return会丢模型更新，因此本轮没有加入假修复。正确前置是产品document/edit session拆开model/edit authority，携带typed origin与expected revision，每owner只保留一个latest pending refresh，在blur/commit时兼容应用或发布typed conflict/rebase receipt；secure pending value归secure owner并随detach/policy/session失效。常态lookup/storage为`O(1)`且按editable session有界，禁止在Surface再建第二份全文cache。状态为`unreal_focused_refresh_policy_reviewed / mutation_origin_not_expressive / bound_editable_value_split_open / product_document_session_required / focused_refresh_implementation_deferred_without_false_fix`。

2026-08-28 current-source correction：前述前置现已接入 paired `UiInputManager` document session。
版本化 `UiTextModelUpdateRequest/Receipt` 区分 `BoundRefresh`、`ExplicitSetText` 与
`ExplicitLoadText`，并以 expected document UUID/revision 做 CAS。聚焦 bound refresh 每 owner 只保留
最新 pending，失焦 exact match 才走 existing document+Surface dual transaction；本地 edit 改变
revision 时返回 content-free `StaleDocument` conflict。显式 Set/Load 聚焦立即应用，IME preedit 先恢复
committed base，因此临时组合串不进入 document range/revision。secure pending 正文只在 Surface store；
clear-only opaque handle 覆盖 detach/policy/supersede/Surface switch/manager Drop，manager 不获得正文读取
接口。lifetime follow-up 将 store value 从普通 `String` 改为 `Zeroizing<String>`；覆盖、丢弃、clear 与
Drop 擦除 pending allocation，accepted transfer 以 `mem::take` 无全文复制移交现有持久状态。该改动不
覆盖 request rejection、component/document/history/layout/platform/crash 明文 owner，RTE-P1-047 仍开放。
queue/transaction/profile owners 为 535/282/137 行，容量为 256 rows、4 MiB/value、16 MiB
aggregate；16 个固定 profile counters 不含动态 identity 或正文。lookup 为 `O(log E), E<=256`，不再
错误声称 `O(1)`；accepted Surface replacement 仍至少 `O(N)`。31-sample managed matrix、allocation/
RSS/p50/p95/p99、power、matched Unreal、dynamic binding producer、平台 IME 与 WGPU 仍开放，未执行前
不改容器、锁、阈值或 merge 算法。状态更新为
`focused_bound_model_update_gateway_implemented_unvalidated /
revision_cas_conflict_implemented_unvalidated /
secure_pending_lifecycle_implemented_unvalidated /
secure_pending_drop_zeroization_implemented_unvalidated /
persistent_secure_document_zeroization_open /
fixed_profile_counters_implemented_unvalidated /
managed_profile_power_wgpu_pending`。

NumberField follow-up确认current-source存在typed schema破坏：catalog `value`为Float，而内部字符/IME text transaction无条件用`UiValue::String(state.text)`提交canonical正文。Unreal `SSpinBox.cpp:937-1076`和`SNumericEntryBox.h:719-741`以独立EditableText、typed ValueAttribute、INumericTypeInterface parse/clamp与formatted cache保持双authority，中间输入不直接改变typed value。本轮先完成不依赖产品session的安全底座：共享editable transaction preflight比较retained/next `UiValue` variant，Float目标收到String edit时以`value_kind_mismatch`在首写前拒绝；Float value、caret、event、binding和dirty保持不变，外部Float-to-Float仍合法。该常数时间guard不是性能优化，也不代表NumberField可键入；独立edit buffer、locale parser、intermediate-invalid、typed change/commit/cancel/blur、format cache和focused external refresh仍开放。状态为`numeric_value_kind_corruption_fail_closed_unvalidated / number_edit_buffer_open / locale_parse_commit_open / managed_validation_pending`。

canonical property follow-up进一步修正Autocomplete render/input split：Surface原有query编辑测试已证明`query`是edit authority且`value`是selected model，但renderer在共享editable classifier后仍默认用value生成visible/editable layout state。现将metadata-level resolver提取为共享borrowed `&str` owner，显式override优先，否则按`query -> value -> value_text -> text`；input只在transaction边界克隆，render的visible/editable/caret/selection复用借用，不引入逐帧属性名allocation。resolve回归固定query与selected value分离。NumberField focused runtime窄测约70秒无输出后终止exact Cargo/Rustc tree，无动态pass。状态为`autocomplete_query_render_edit_property_converged_unvalidated / canonical_metadata_property_resolver_shared / managed_validation_pending`。

public edit event follow-up硬删除旧`UiTextEdit`携带的raw action和完整before/after editable snapshot，`UiWidgetEvent::TextEditChange`改为固定大小、无正文的versioned document receipt。receipt只包含document UUID、strictly consecutive revision、typed kind/source、old/new byte range与最终selection；schema、nil identity、revision跳变/耗尽和反向range均在消费前typed fail closed。该改动消除公共事件随document长度线性复制的结构性缺陷，但内部`TextDocument`尚未接产品registry/producer，product snapshot lease绑定、range/source/grapheme验证、history/rebase和secure document session仍开放；禁止把非Clone document塞进clone/serde Surface形成第二authority。Rustfmt、旧DTO reference scan和diff check只构成静态证据，本轮接口改动尚无Cargo pass。状态为`public_text_edit_snapshot_event_removed_unvalidated / versioned_document_edit_receipt_contract_implemented_unvalidated / runtime_document_receipt_producer_open / product_snapshot_lease_integration_open / m1_document_authority_open`。

revision-bound snapshot follow-up为内部piece document增加每revision一个lazy `OnceLock<Arc<str>>`：初始source直接复用original Arc；同revision lease只clone Arc；changed edit在全部fallible prepare通过后为新revision清空连续snapshot槽，旧lease继续稳定；typed no-op保持当前pointer identity。grapheme source index改为借用lease，删除原先先构造临时`String`再全文扫描的第二份N-byte复制，但boundary rebuild仍为`O(N)`。TextDocument与lease的Debug只输出identity/revision/byte length/chunk count，不输出正文。该前置没有产品registry/Surface consumer，也没有snapshot byte/age/count budget、retention/zeroization或managed allocation/RSS/power数据；M1不关闭。状态为`revision_bound_snapshot_lease_implemented_unvalidated / single_flatten_per_requested_revision_implemented_unvalidated / source_index_secondary_source_copy_removed_unvalidated / document_debug_source_redacted / product_registry_open / snapshot_budget_and_managed_profile_pending`。

document receipt producer preflight继续把public identity绑定进非Clone document authority：document构造时自行签发UUID，snapshot lease携带UUID+typed revision，changed receipt携带UUID及previous/current byte length；cache-oriented `u64 owner + revision`不穿过产品边界。public projection不再接受调用方提供的document id或length，仅接node/source/kind/final selection，并以固定字段检查owner、consecutive revision、range/document length delta、u32 narrowing、old/new bounds及selection bounds；byte selection新增focus affinity以保留wrapped/BiDi边界caret归属。公共receipt的serde反序列化也调用同一validate，nested widget event不能接收unsupported schema、nil UUID、跳号/耗尽revision或反向range。整个projection为`O(1)`且不读/hash/copy正文。bounded service registry/session、Surface gateway、snapshot-bound source/grapheme消费和managed Cargo仍开放。状态为`document_authority_uuid_bound_unvalidated / content_free_changed_receipt_projection_implemented_unvalidated / wire_receipt_deserialization_validation_implemented_unvalidated / runtime_document_service_gateway_open / surface_consumer_open / managed_validation_pending`。

document storage residency follow-up没有直接修改piece算法，而是先固定内容无关的观测边界。当前每次非空replacement追加一个immutable addition `Arc<str>`，仅同chunk连续range可coalesce；回归明确记录8次单字符尾插入产生8个addition chunk与8个piece，说明metadata随edit count线性增长的结构风险。`TextDocumentStorageReport`报告original/addition bytes、chunk/piece数量与capacity、hard-line/grapheme index capacity、当前flattened snapshot及retained-byte lower bound；allocator header、Arc control block和外部旧lease明确不计，因此不能充当admission limit。Unreal Slate参考确认editable transaction在retained line text/run/view owner上更新，但不为Zircon选择容器或阈值。算法保持不变，先执行1/100/1k/10k edit stream、一百万字符base、31次cold/warm、snapshot/no-snapshot、allocation/RSS/p50/p95/p99/index rebuild/power与matched Unreal对照，再决定append batching、compaction、gap buffer、tree piece table或rope。状态为`document_storage_residency_report_implemented_unvalidated / linear_tail_insert_chunk_growth_encoded / storage_algorithm_unchanged / compaction_policy_deferred_pending_profile / managed_allocation_rss_power_profile_pending`。

surface-session document store follow-up先复核生命周期：产品`RuntimeUiSurface`是一组`UiSurface + UiInputManager`，而Surface本身可Clone/serde；把非Clone document塞进Surface会分叉authority，注册全局manager又会让不同surface的node identity串状态。对应Unreal的`FSlateEditableTextLayout`也属于editable owner并以scoped transaction提交，不是全局document单例。因此内部replace拆成零mutation `prepare_replace`和重验expected key的`commit_replace`；新增store只有`with_limits`构造，无Default/全局注册，显式限制document/visible bytes/replacement/retained source/chunk/piece/current snapshot/active lease count+bytes。changed edit在exact prepare后、commit前admit，snapshot在flatten前admit，managed non-Clone lease在Drop释放预算；错误/报告不含正文。生产阈值仍需profile，Surface/UiInputManager接线、teardown、secure policy、grapheme handles与public receipt product publication保持open。默认Runtime Cargo在到达text前被未跟踪`zr_rhi_wgpu` readback的三处u32/u64错误阻断，text-only编译与focused test继续尝试。状态为`document_prepare_commit_boundary_implemented_unvalidated / explicit_limit_session_store_implemented_unvalidated / snapshot_lease_admission_and_release_implemented_unvalidated / global_manager_registration_rejected / surface_input_session_integration_open / product_thresholds_and_managed_profile_pending`。
