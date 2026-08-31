---
title: Editor Input Action、Mapping Context、Binding、Trigger、Modifier、Device、User、Rebinding、Accessibility Authoring 与 Product Integration 当前源码复核
category: zircon_editor
report_id: Editor87
review_date: 2026-08-24
baseline_head: 9a217cce07c574cbec8dda70b3e1142eeedbc9a9
baseline_epoch: 408
final_recheck_head: c4552db650793c4c838efac57ed5858857451bc3
final_recheck_epoch: 414
canonical_owner: Editor29
refreshes:
  - docs/plans/optimize/zircon_editor/29-input-action-mapping-context-binding-trigger-modifier-device-user-rebinding-accessibility-authoring-review.md
related_code:
  - zircon_runtime/src/core/framework/input
  - zircon_runtime/src/input
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_runtime/src/dynamic_api/session/construction.rs
  - zircon_runtime/src/script/vm/gameplay_host.rs
  - zircon_runtime/src/script/vm/gameplay_host/input.rs
  - zircon_runtime/src/asset/project/manifest/project_manifest.rs
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_editor/src/core/asset/type_registry
  - zircon_editor/src/core/asset/dirty
  - zircon_editor/src/core/commands/key_chord.rs
  - zircon_editor/src/core/commands/keymap.rs
  - zircon_editor/src/core/settings/keymap_overrides.rs
tests:
  - zircon_runtime/src/input/tests/action_mapping.rs
  - zircon_runtime/src/input/tests/action_axis_transitions.rs
  - zircon_runtime/src/input/tests/recording_replay.rs
  - zircon_runtime/src/input/tests/input_manager
  - zircon_runtime/src/dynamic_api/session/tests/physical_input_ownership.rs
  - zircon_editor/src/core/commands/keymap/tests.rs
plan_sources:
  - docs/plans/mvp/index.md
  - docs/plans/optimize/zircon_runtime/99r-runtime-input-device-event-frame-state-action-map-focus-gamepad-recording-replay-host-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zb-runtime-gameplay-framework-game-instance-world-context-level-game-mode-game-state-local-player-controller-pawn-possession-spawn-travel-network-save-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/12-settings-preferences-scope-persistence-locale-i18n-appearance-plugin-extensibility-review.md
  - docs/plans/optimize/zircon_editor/29-input-action-mapping-context-binding-trigger-modifier-device-user-rebinding-accessibility-authoring-review.md
  - docs/plans/optimize/zircon_editor/47-runtime-gateway-session-event-consumer-world-sync-generation-backpressure-reconnect-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/53-editor-interactive-tool-scheduler-resource-lease-input-capture-mode-modal-extension-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_editor/59-editor-scene-viewport-interaction-controller-input-picking-selection-highlight-gizmo-transaction-cancel-generation-product-integration-current-source-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/EnhancedInput/Source/EnhancedInput/Public/InputAction.h
  - dev/UnrealEngine/Engine/Plugins/EnhancedInput/Source/EnhancedInput/Public/InputMappingContext.h
  - dev/UnrealEngine/Engine/Plugins/EnhancedInput/Source/EnhancedInput/Public/UserSettings/EnhancedInputUserSettings.h
  - dev/UnrealEngine/Engine/Plugins/EnhancedInput/Source/InputEditor/Private/ActionMappingDetails.cpp
  - dev/UnrealEngine/Engine/Plugins/EnhancedInput/Source/InputEditor/Private/AssetDefinition_InputAction.cpp
  - dev/UnrealEngine/Engine/Plugins/EnhancedInput/Source/EnhancedInputTestSuite/Private/InputPlayerMappableKeysTests.cpp
  - dev/godot/core/input/input_map.h
  - dev/godot/core/input/input_map.cpp
  - dev/godot/editor/settings/action_map_editor.cpp
  - dev/godot/editor/settings/input_event_configuration_dialog.cpp
  - dev/bevy/crates/bevy_input/src/lib.rs
  - dev/bevy/crates/bevy_input/src/button_input.rs
  - dev/bevy/crates/bevy_input/src/keyboard.rs
  - dev/bevy/crates/bevy_input/src/gamepad.rs
  - dev/Fyrox/fyrox-impl/src/engine/input.rs
  - dev/Fyrox/fyrox-ui/src/key.rs
  - dev/Fyrox/editor/src/settings/keys.rs
  - dev/Graphics/Templates/com.unity.template-hd/Assets/InputSystem_Actions.inputactions
  - dev/Graphics/Templates/com.unity.template-hd/Assets/SampleSceneAssets/Controllers/Common/InputSystem/StarterAssets.inputactions
  - dev/Graphics/Templates/com.unity.template-hd/Assets/SampleSceneAssets/Controllers/Common/InputSystem/StarterAssets.inputsettings.asset
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugManager.Input.cs
doc_type: current_source_refresh
review_status: complete
implementation_status: not_started
source_recheck_required: true
canonical_finding_delta:
  p0: 0
  p1: 0
  p2: 0
finding_status:
  open: 62
  partial: 15
  closed: 0
gate_status:
  fail: 32
  partial: 0
  pass: 0
---

# Editor Input Action、Mapping Context、Binding、Trigger、Modifier、Device、User、Rebinding、Accessibility Authoring 与 Product Integration 当前源码复核

## 1. 结论

Editor29 的五项 P0 在当前源码中仍全部成立。Zircon 已有可保留的原始输入与标量 Action evaluator 底座，但没有可创建、保存、重开、编译、cook、安装、调试或玩家改绑的 Input Action / Input Mapping Context 产品。`ResourceKind` 与 Editor builtin asset registry 仍只有 26 类资源且没有 Input 类型；`ProjectManifest` 没有 input source/artifact/profile 选择；`zircon_app` 只选择 Input module，不提供项目 Action 配置；production 对 `module_descriptor_with_config` 与 `evaluate_actions*` 仍没有产品调用链。默认 Input 配置继续是 `enabled = false` 与空 `InputActionMap`。

底层 Action schema 仍是调试级形状。Action、Context 与 Binding identity 是裸 `String`；Action 只有 `id/context/display_name`，Context 只有 `id/priority/enabled`，Binding 只有 button chord 与 scalar gamepad axis；Action state 只有 string set 与 `f32`。没有 Bool/Axis1D/2D/3D typed value、trigger phase、modifier/processor、composite、stable source/binding ID、compiler diagnostic、artifact identity、InputUser、LocalPlayer、device assignment、profile revision或install/rebind receipt。`InputAxisBinding` 还直接序列化 runtime `GamepadId`。

非法 source 会被静默改写语义：duplicate Action/Context 被 helper 忽略，Action 引用的 missing Context 在 generation 中被自动插入并默认 enabled，unknown Action 的 Binding 没有诊断地失去 consumer；Context priority 只被保存，evaluator 没有按 priority 排序执行，空 `active_contexts` 又表示所有 Context active。`set_action_map` 在 mutex 内立即整图替换，没有 expected generation、frame barrier、held-input policy或terminal receipt。

当前工作树出现了一项真实 Runtime 进展。`dynamic_api/session/events.rs` 已把 pointer、mouse、keyboard、touch 与 gamepad 的物理状态提交移动到 Runtime UI dispatch 之前，新增 `physical_input_ownership.rs` 验证 UI capture 停止语义传播时 release 仍进入 `InputManager`。这直接改变 Runtime117 `INP-P0-002` 的旧证据，Runtime owner 应重新判定该 finding；Editor87 不重复关闭或计数该 Runtime finding。它只证明 physical truth 的提交顺序正在修复，不提供 UI consume -> per-user context -> action evaluation schedule、capture cancellation、Action snapshot或创作产品。

脚本与 gameplay 仍绕过 Action Manager。`gameplay.key_pressed(string)` 每次解析 raw key 并查询全局 `InputManager`；Vampire 示例仍用 `"W"` 等字符串。当前 production Rust 对 `LocalPlayer` 与 `InputUser` 的精确词检索均为 0。Runtime Gameplay 当前报告也确认 Dynamic Session 只有一个全局 InputManager 和固定 camera controller，没有 platform user/device -> LocalPlayer -> Controller 路由。

因此本轮不新增 finding，只对 Editor29 的 77 项原账本做 current-source 重判：**5 P0 Open；60 P1 中 46 Open、14 Partial、0 Closed；12 P2 中 11 Open、1 Partial、0 Closed；32 Gate 全部 Fail**。15 个 Partial 只表示 generation/workspace、consumed token、raw focus/device cleanup、raw injection/recording、Editor asset/dirty/keymap 基础和性能测试可复用，不表示 Input authoring、artifact install、per-user evaluation、rebind/profile或accessibility已经交付。

目标架构保持为：

```text
InputActionDocument + InputMappingContextDocument
  -> InputSemanticCompiler
  -> CompiledInputMapArtifact
  -> InputMapInstaller
  -> per-InputUser ActiveInputMapGeneration
  -> InputActionFrameSnapshot

PlayerBindingProfile
  -> RebindCaptureRequest + ConflictQuery
  -> atomic RebindCaptureReceipt / ProfileRevision
  -> same installer and runtime generation
```

“性能和表现优于当前 Unreal”目前没有证据。现有 10K binding 测试只证明局部索引访问与 workspace 复用，不包含 typed trigger/vector、context churn、多用户、多设备、profile overlay、Editor compile、cook/install、debug observation或真实输入延迟。必须先取得同语义功能闭环，再以相同硬件、设备、帧率、映射复杂度和用户拓扑比较 compile/install latency、frame CPU/allocation、input-to-action latency、tail latency、memory、journal bound 与长期 soak；不能以缺少功能得到的低开销冒充性能优势。

## 2. Owner、currentness 与物理冻结

### 2.1 唯一 owner 与不重复计数

| 主题 | 唯一 owner | Editor29 / Editor87 的边界 |
|---|---|---|
| 原始 device/window/focus/event/frame state、stable physical control、UI ownership | Runtime117（Runtime Input） | 只消费 qualified physical snapshot/consumed set，不复制设备 registry 或 frame reducer |
| LocalPlayer、Controller、possess、gameplay tick | Runtime Gameplay（Runtime99zb） | 声明 Action snapshot 的消费合同，不在 Editor 内创建玩家 authority |
| Input Action/Mapping Context source、semantic compiler、artifact | Editor29 + Runtime Input shared schema/compiler | Runtime 拥有 compiler/artifact/install contract，Editor 拥有 transactional authoring、diagnostics与preview projection |
| Editor command shortcut | Editor08 | 保持 command keymap identity/storage/owner 独立，只共享基础 physical token formatter、capture 和 conflict primitive |
| 文档/history/save/conflict | Editor02 | Input document 提交 typed command/path，不自建第二 history 或 dirty authority |
| asset registry/factory/toolkit/reference | Editor04 | Input 类型作为 contribution 接入，不硬编码平行 browser/catalog |
| settings/profile persistence | Editor12 + platform/save owner | Player profile 是 versioned typed contributor，不直接写任意 JSON/TOML |
| interactive capture/session | Editor53 | Rebind 建立 domain state machine并复用 capture lease，不把 viewport tool capture 当 player rebind |
| viewport input/picking/gizmo | Editor59 | 只消费 Editor command/viewport route，不承担 shipping Gameplay Action schema |
| script gameplay API | Runtime script + Gameplay owner | 迁移到 scoped Action query，raw key 降为受限低层 capability |
| cook/package | Runtime asset/compiler + 后续 Tooling owner | 本轮用户明确排除 tooling，只冻结 artifact 输入输出，不展开工具实现 |

Editor29 继续是 Gameplay Input Action / Mapping Context authoring、compiler projection、profile/rebind UX 与 Action debugger 的 canonical owner；Editor87 只是 current-source refresh，`canonical_finding_delta` 为 0。Runtime117 的原始输入 finding、Runtime Gameplay 的 LocalPlayer finding、Editor08 的命令系统 finding 不在本文重复计数。

### 2.2 Currentness 与共享工作树

- 协调 session 为 `optimize-editor87-input-authoring-current-review-r1-20260824`，注册基线为 `9a217cce07c574cbec8dda70b3e1142eeedbc9a9`、epoch `408`；初次写入前冻结为 `2a1299f8bf8e5a3012860ff07a6fcf528e4721d8`、epoch `411`，最终复核为 `c4552db650793c4c838efac57ed5858857451bc3`、epoch `414`。两次复算 selected stats 与 fingerprint 完全一致。
- 共享工作树不是 clean HEAD。92 个 Zircon selected 文件中有 15 个在途路径，包含 raw Input manager/state/tests、Dynamic Session construction/events、未跟踪 physical ownership test、Editor asset type/dirty 基础和 App builtin module。本文按冻结时 working-tree bytes 读取，不回退、不覆盖，也不把未验证的在途改动写成已发布能力。
- `events.rs` 的 physical-first 变化是本轮必须保留的时效差异；`construction.rs` 同时存在 time policy 方向的无关修改，不能归因于 Input authoring。Editor asset registry 的在途 refactor没有新增 Input kind/factory/toolkit。
- 本轮没有开放的 Editor87 failure handoff，四个文档路径已由当前 session 领取 lease。MVP `00-current-source-baseline-recovery` 仍为 `in_progress`；review-only 合法，但不能宣称 M0-M9 实施完成。

### 2.3 冻结语料

指纹算法为：按 normalized relative path 排序，对每个文件计算 lowercase SHA-256，以 `path|hash` 和 LF 连接且无末尾 LF，再对整体取 SHA-256。test declaration 是静态词法计数，不表示执行或通过。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 证据与 fingerprint |
|---|---:|---|
| Runtime Input 全目录 | **58 / 6,195 / 5,460 / 202,124 / 55 / 0** | public raw/action schema、module/config、manager/evaluator/generation/workspace、recording与全部 focused tests；`31c8ebf2d693559341d5baa252110383d7d834cb1a9278488433e1e932fc33c6` |
| 产品集成纵切面 | **10 / 3,037 / 2,814 / 120,219 / 15 / 0** | Dynamic Session ingress/construction、script raw input、Project manifest、ResourceKind、App module composition；`ea0c5515bb576664324cf2fec045194de0f4a803ee184920919d6e2a89f06367` |
| Editor authoring基础 | **24 / 4,685 / 4,193 / 153,945 / 13 / 0** | asset type/toolkit/dirty、command keymap/chord、settings override与catalog；`9e04bffa0fcb40242dfa5c4d08d3bc08cd2937f3163beacc25268da99c5cc623` |
| Zircon selected union | **92 / 13,917 / 12,467 / 476,288 / 83 / 0** | 上述三组互斥 working-tree snapshot；`9469524c35c2c145c513c06f7d542ec66c8c999e10367d0e92cd5a1e0b94428a` |
| Unreal Enhanced Input | **11 / 5,300 / 4,363 / 205,385 / 0 / 0** | Action/Context source与validation、Editor asset/details、user settings/profile tests；`5c9aba5a30b0ad1de37ff6aea18e432dbf9887fc166d65ccfbb607cd854b9092` |
| Godot | **6 / 2,867 / 2,359 / 116,613 / 0 / 0** | InputMap、Project Settings Action Map Editor与event configuration；`e418acb3eea820f54d4bd0765f1f299f180aa248f06ef98fbd6a3e773261d488` |
| Bevy | **5 / 6,148 / 5,717 / 225,565 / 51 / 0** | raw plugin、ButtonInput、keyboard/gamepad与focus schedule；`aa552269049862b588516beba6fe59fc893246eeaaa33299bfc268246bea1ac5` |
| Fyrox | **3 / 952 / 866 / 36,329 / 1 / 0** | raw InputState、HotKey/KeyBinding capture与Editor key settings；`863e85f1d6accd432f6210cfbb8deaec53c894f9e48862deca80781e16f774e9` |
| Unity Graphics | **7 / 3,462 / 3,439 / 133,691 / 0 / 0** | `.inputactions`/`.inputsettings` artifact与DebugManager consumer；`93fc403fd13f498639c8e2aeca20edc24f6800f19881debab97f2691496d28bd` |
| 五引擎参考 union | **32 / 18,729 / 16,744 / 717,583 / 52 / 0** | Unreal 11、Godot 6、Bevy 5、Fyrox 3、Unity Graphics 7；`8075b9cadb0bce4a47582356c75216a28dd0aabc09e09c0264bfe56dd356961e` |
| all selected | **124 / 32,646 / 29,211 / 1,193,871 / 135 / 0** | Zircon与reference按normalized path去重；`cfe101650b64c7759ff14733de9d28c7b870c79c1334a9e3bc340688f8a690d6` |

## 3. 当前实现逐层事实

### 3.1 Action schema 仍不能作为工程 source 或 artifact

1. `InputAction` 只有三个字符串字段；`InputActionContext` 只有 string ID、priority 与 enabled；`InputBinding` 只引用 action string 并保存 button chord/scalar axis。三者没有 schema version、stable ID、source revision、unknown-field policy或migration。
2. `InputButton::Gamepad` 与 `InputAxisBinding` 直接包含当前 runtime `GamepadId`。这只能描述某次连接实例，不能描述可跨启动、重连、同型号多设备、平台或玩家保存的 authored selector。
3. `InputActionState` 以 string set/map 保存 active/just activated/just deactivated 与 scalar `f32`；公开迭代接口会 clone 为 `Vec`。没有 map generation、frame tick、InputUser、typed value、phase/duration、source binding或bounded observation view。
4. serde graph 没有 `deny_unknown_fields`、version或semantic compiler。`add_action`/`add_context` 静默去重，`bind` 只拒绝空 Binding；这会把 source error 变成运行时 first-wins/ignore 行为。

### 3.2 Evaluator 是可保留底座，但不是 compiled product

1. `ActionEvaluationGeneration` 在 map change 时建立 context/action/binding/axis lookup，`ActionEvaluationWorkspace` 复用数组和 consumed index；10/100/1K/10K 测试约束访问范围与容量复用。这些是真实性能基础。
2. generation 会把 Action 引用但未声明的 Context 自动插入为 enabled，并只为已知 Action 编译 binding range；两类 source error 均没有 typed diagnostic。
3. evaluator 以 dominant absolute scalar 聚合 axis；没有 typed vector/composite、trigger/modifier chain或stateful phase machine。Context priority 没有进入排序/冲突执行，`active_contexts.is_empty()` 表示全部 Context active。
4. `DefaultInputActionManager::set_action_map` 立即替换 evaluator generation。没有 artifact digest、expected active generation、frame barrier、LKG、held-input preserve/rebuild/flush policy或install receipt。

### 3.3 Module、Project、App 与 shipping consumer 断开

1. `InputConfig` 把 runtime `InputActionMap` 直接嵌入 module config，默认 `enabled = false` 且 map 为空；这既不是 source asset，也不是 cook artifact。
2. `module_descriptor_with_config` 的 production caller 为 0，现有配置调用只在 Input tests；Dynamic Session construction只 resolve全局 `InputManager`，不 resolve/install/tick `InputActionManager`。
3. `ProjectManifest` 没有 Input Action/Mapping Context locator、default context set、profile或artifact dependency；`zircon_app` builtin composition只选择 module，不从项目装载 action product。
4. `ResourceKind` 与 Editor builtin registry 都没有 Input 类型。当前 exact product symbol `InputActionDocument`、`InputMappingContextDocument`、`CompiledInputMapArtifact`、`InputMapInstaller`、`PlayerBindingProfile`、`RebindCaptureRequest/Receipt` 与 `InputActionFrameSnapshot` 均为 0。

### 3.4 Physical-first 改动修复旧事件顺序，但尚未形成 Action schedule

1. pointer、mouse press/release/wheel/motion、touch、keyboard、gamepad button/axis 现在先调用 `submit_input_event`，再让 Runtime UI 决定是否停止语义传播。
2. 新测试构造真实 Runtime session 与 capture Slider，验证 press 后物理 held 为真、UI capture 后在控件外 release 仍清除 held 并进入 event journal。这是比单元 mock 更强的集成证据，但文件仍未跟踪且本轮未执行。
3. UI dispatch 仍只返回 consumed bool；没有 sequence-qualified ownership receipt、capture generation、per-user consumed set或 Action evaluation producer。Focus/device loss只清理 raw state，没有 Action phase cancellation snapshot。
4. Runtime117 的 `INP-P0-002` 应由其 owner基于当前工作树与动态测试重开裁决；Editor87 只把 P1-57 标记 Partial，不越权关闭 canonical Runtime finding。

### 3.5 Script、LocalPlayer、Profile 与 Rebind 产品为零

1. `gameplay.key_pressed` 每次 resolve `InputManager`，把 script string 映射为 raw `InputButton` 后读取 held state；它不使用 Context、consume、Action phase、profile或user scope。
2. production Rust 中没有 `LocalPlayer`、`InputUser`、`InputDeviceAssignment`、`PlayerBindingProfile` 或 profile/rebind terminal receipt。全局 manager不能证明 split-screen、remote player、seat assignment或device transfer。
3. 没有 rebind capture state machine、noise/threshold/release/cancel/timeout、conflict classification、atomic expected-revision commit或base-map/profile-delta reconcile。
4. Accessibility 只在其他 UI/Editor域有通用目标，没有 Input hold/toggle/sticky、one-handed、sensitivity/deadzone等可组合 profile transform。

### 3.6 Editor 通用基础可复用，但没有 Input 产品贡献

1. asset type registry已有 typed definition、creation template、toolkit与batch registration；dirty/save基础已有 registry、batch与job adapter。这使 P1-46/47 为 Partial，但没有任何 Input kind、factory、document adapter、toolkit或open route。
2. Editor command keymap已有 immutable base + User/Project/Session typed override、normalized chord signature、indexed conflict query与测试。这证明命令快捷键是独立且较成熟的 authority，不应被改名为 Gameplay Input。
3. 当前没有共享的 physical token schema/capture widget library，也没有依赖方向测试防止 Editor command schema 与 Gameplay Action schema互相引用；P1-56 只能是 Partial。

## 4. 五引擎参考结论

### 4.1 Unreal Enhanced Input：工程级主参考

1. `UInputAction` 是独立 asset，表达 Boolean/Axis1D/Axis2D/Axis3D、accumulation、consume/reserve、triggers、modifiers、player-mappable metadata、trigger event/duration与source Action，不把玩法语义压成 raw key或单 `f32`。
2. `UInputMappingContext` 独立保存 mappings、profile overrides、input mode filter、registration tracking与validation；context priority、add/remove/rebuild和held-key policy属于 per-player subsystem 行为。
3. Input Editor 提供 Action/Mapping Context asset definition与 factory 路径；`ActionMappingDetails` 的 mapping增删、reorder与grouped edit走 transaction，而不是修改控件字符串。
4. Enhanced Input User Settings 把 LocalPlayer、mapping context registration、profile、map/unmap/reset与异步保存组成玩家配置产品；TestSuite验证多个独立 profile与player-mappable key状态。
5. Zircon不复制 UObject/Blueprint宏与历史类层次；应提炼 stable identity、source/compiler/artifact、per-user owner、phase/receipt和测试合同，并用更紧凑的 Rust data-oriented runtime实现。

### 4.2 Godot：较低复杂度但真实可用的创作底线

1. `InputMap` 提供 named action、deadzone、InputEvent、project settings load与增删改查；missing/invalid operation使用明确 error，而不是静默自动创建 enabled context。
2. Project Settings 的 Action Map Editor支持 action add/edit/remove/rename/reorder、event add/edit/remove、filter、deadzone与revert；Input Event Configuration Dialog覆盖 keyboard/mouse/joypad、physical/logical/location/device配置。
3. Godot 的全局 singleton与字符串 action不应成为 Zircon 最终架构，但它证明“可在项目中创建、持久化、编辑、诊断和运行”是最低产品线；Zircon当前尚未达到。

### 4.3 Bevy：只约束 raw input，不作为 Action authoring 对标

Bevy本地源码提供清晰的 InputPlugin schedule、ButtonInput transition、keyboard physical/logical/repeat、gamepad raw/filtered event、entity identity与断连处理，并以大量测试约束 reducer行为。它没有同级 Input Action / Mapping Context authoring产品；因此只用于验证原始事件时序、state reduction与设备生命周期，不能被用来证明 Zircon Action 产品足够完整。

### 4.4 Fyrox：raw shortcut 与 capture widget 参考

Fyrox `engine/input.rs` 明确把聚合 `InputState`称为简化 shortcut，并指出其丢失设备 identity，多设备应消费 event；这直接反证 Zircon不能把全局 raw snapshot作为最终 Gameplay API。Fyrox UI已有 serializable `HotKey`/`KeyBinding`和可聚焦的 capture editor，Editor settings也有显式 key bindings；这些可参考 capture/formatter交互，但不是完整 Gameplay Action/profile系统。

### 4.5 Unity Graphics：只使用仓内可见 artifact 证据

本地 Graphics tree不含完整 Unity Input System package源码，不能推测其 Editor或profile实现。但仓内真实 `.inputactions` 已足以证明可交付 artifact至少包含稳定 GUID、Player/UI Action Map、typed action、expected control type、composite binding、control path、groups/control schemes、Hold interaction，以及 `StickDeadzone`、`InvertVector2`、`ScaleVector2`等 processor。`.inputsettings.asset`还保存 update mode、deadzone、press/tap/hold/multitap阈值；`DebugManager.Input.cs`展示 runtime map/action、composite modifier、performed callback与typed value读取。Editor87仅使用这些可见事实，不把缺失源码外推为已验证能力。

## 5. 目标合同与硬边界

| 合同 | 最低字段与不变量 |
|---|---|
| `InputActionDocument` | stable Action ID、schema/source revision、value type、trigger/modifier graph、consume/reserve policy、player-mappable/localization metadata |
| `InputMappingContextDocument` | stable Context/Binding ID、mapping order、activation/priority/block policy、authored `PhysicalInputPattern`、dependencies与unknown preservation |
| `InputSemanticCompiler` | duplicate/missing/type/capability/conflict/reserved校验；canonical ordering；typed diagnostics；deterministic digest；同源用于Editor/cook/tests |
| `CompiledInputMapArtifact` | compiler/schema version、source/dependency digest、dense IDs、compiled composite/trigger/modifier program、capability requirements、migration provenance |
| `InputMapInstaller` | InputUser/LocalPlayer、expected generation、artifact/profile revision、frame barrier、held-input policy、LKG与唯一 terminal receipt |
| `InputDeviceAssignment` | stable platform user/device identity到InputUser/seat的generation-qualified lease；authored bytes永不包含 runtime slot |
| `InputActionFrameSnapshot` | user/player/map generation/frame tick、typed values、Started/Ongoing/Triggered/Completed/Cancelled、source与consumption explanation |
| `PlayerBindingProfile` | base artifact revision + sparse stable Binding delta、device/layout/accessibility scope、migration/orphan/conflict状态与atomic persistence |
| `RebindCaptureRequest/Receipt` | owner/capture lease、device/control filter、noise/threshold/release/cancel/timeout、conflict query/resolution、expected revision与zero-visible failure |
| `InputObservationSnapshot` | bounded、reader-gated、脱敏、不可回写 runtime truth，Editor debugger只投影同一代 Action事实 |

Runtime hot path只消费 validated immutable artifact和per-user dense generation；Editor source、strings、localization、full diagnostics与profile merge不进入每帧扫描。旧 `InputConfig.action_map` 是显式 migration source，不能与新artifact长期双轨。

## 6. P0 当前重判

| Canonical finding | 状态 | 当前证据 | 必须关闭的重构 |
|---|---|---|---|
| Editor29 P0-1：没有 Input Action/Mapping Context asset、factory、toolkit或Editor产品 | Open | 26类 `ResourceKind` 与 builtin registry仍无Input；目标产品符号全为0 | 注册两类source asset、factory/toolkit/document/open route，接shared compiler、diagnostic与reference graph |
| Editor29 P0-2：shipping Action Manager默认空且无project/asset/cook安装桥 | Open | 默认disabled空map；configured descriptor仅测试；ProjectManifest/App/Dynamic Session无artifact install | project选择source/artifact，首帧前frame-barrier安装并发布typed receipt；无artifact时明确Unavailable |
| Editor29 P0-3：serialized binding硬编码临时GamepadId | Open | `InputButton::Gamepad`/`InputAxisBinding`仍持runtime ID | Runtime117提供stable device/control identity；Editor source只保存selector，install按InputUser assignment解析 |
| Editor29 P0-4：非法map被静默接受并改变语义 | Open | duplicate静默忽略、missing context自动enabled、unknown action binding无diagnostic、priority未执行 | source只能经semantic compiler；错误fail-close或显式migration，runtime拒绝任意未验证serde graph |
| Editor29 P0-5：Gameplay脚本绕过Action Manager且无InputUser/rebind profile | Open | `gameplay.key_pressed(string)`与全局manager仍是产品路径；LocalPlayer/InputUser为0 | Runtime Gameplay建立per-user tick与Action facade；迁移产品脚本，建立profile/rebind/accessibility并限制raw capability |

## 7. P1 Source、Compiler、Runtime、User 与 Editor 重构账本

### 7.1 Source schema、compiler 与 artifact（P1-1 至 P1-15）

| 项 | 状态 | 当前差异与目标 |
|---|---|---|
| P1-1 Input Action asset kind | Open | 注册独立source type，不把Action藏在module config或Mapping Context row |
| P1-2 stable Action/Context/Binding IDs | Open | 裸String与数组位置全部迁移到不可伪造stable IDs，并保留redirect/migration |
| P1-3 typed Action value | Open | Bool/Axis1D/Axis2D/Axis3D typed union贯穿source/artifact/runtime/script/debug |
| P1-4 Action metadata | Open | consume/reserve/paused、accumulation、localization、player-mappable、tags与capability metadata |
| P1-5 Mapping Context metadata | Open | stable source identity、activation/input mode、priority/block、dependency与profile override |
| P1-6 Binding stable identity | Open | reorder/rename/merge/profile delta后Binding ID保持稳定，不能靠数组index |
| P1-7 authored control selector | Open | device class/layout/control/logical-physical policy与optional selector取代runtime GamepadId |
| P1-8 typed trigger definitions | Open | pressed/released/tap/hold/pulse/chord/combo等typed definitions与版本化参数 |
| P1-9 typed modifier definitions | Open | deadzone/scale/invert/swizzle/normalize/curve等typed processor及capability validation |
| P1-10 ordered trigger/modifier chains | Open | stable node/edge/order与shared compile semantics，不能在Editor/Runtime/Script分叉 |
| P1-11 schema version/deny unknown/unknown preservation | Open | source version、strict fields、plugin unknown payload保存与migration diagnostic |
| P1-12 semantic compiler | Open | reference/type/collision/capability validation、canonical IR、deterministic output与LKG |
| P1-13 validation diagnostics | Open | stable code/severity/source path/span/fix-it、correlation与stale generation |
| P1-14 conflict graph | Open | 按有效context/user/device/profile区分Blocking/Shadowing/Allowed/Reserved；Editor keymap conflict不是替代品 |
| P1-15 immutable artifact/digest/dependencies | Open | internal evaluator generation不是可保存/cook/验证的artifact；补compiler/schema/source digest与dependency revisions |

### 7.2 Runtime install、context、evaluation 与 observation（P1-16 至 P1-30）

| 项 | 状态 | 当前差异与目标 |
|---|---|---|
| P1-16 artifact installer | Open | 无production install caller；建立admission、capability、LKG、terminal receipt与teardown |
| P1-17 frame-boundary replacement generation | Partial | evaluator已有内部generation，但`set_action_map`立即替换且无expected generation/frame barrier/receipt |
| P1-18 Action state source identity | Open | snapshot携Action/Binding/Control/device/user与consumption provenance |
| P1-19 trigger phase/duration | Open | Started/Ongoing/Triggered/Completed/Cancelled及elapsed/triggered time必须确定 |
| P1-20 typed vector value/aggregation | Open | scalar dominant-abs扩展到typed vector/composite与显式aggregation policy |
| P1-21 per-user context stack | Open | 当前只有调用者字符串slice；建立owner lease、priority与generation-qualified stack |
| P1-22 consume/block/reserve | Partial | consumed button/axis index是真实底座；缺context priority、block/reserve与产品producer |
| P1-23 context owner lease | Open | add/remove/priority不能由匿名caller永久修改；owner revoke与session teardown必须回收 |
| P1-24 rebuild/flush | Partial | map-change generation存在；缺preserve/rebuild/flush枚举、held-input policy与receipt |
| P1-25 held-input rebind policy | Open | 改图/改键期间held key/axis必须按显式policy处理，不能依赖偶然event顺序 |
| P1-26 focus/device loss cancellation | Partial | raw manager会清理state；缺per-user Action phase/source cancellation与snapshot证据 |
| P1-27 multi-window/viewport | Open | raw event没有组成window/view/user-qualified Action route与focus/capture policy |
| P1-28 injection/simulation | Partial | raw submit/recording/replay入口可复用；缺artifact/profile/user-qualified Action injection与权限 |
| P1-29 gameplay/script facade | Open | 只存在raw `key_pressed`；提供typed scoped Action value/phase API并迁移示例 |
| P1-30 bounded observation | Open | Action state clone Vec不是reader-gated bounded journal；debug trace需预算、脱敏与drop report |

### 7.3 InputUser、Device、Profile、Rebind 与 Accessibility（P1-31 至 P1-45）

| 项 | 状态 | 当前差异与目标 |
|---|---|---|
| P1-31 InputUser/LocalPlayer | Open | production 0命中；建立platform user/device set/viewport/controller/profile归属 |
| P1-32 authored selector/runtime identity separation | Open | serialized runtime GamepadId直接违反边界；compiler/install阶段解析且不回写source |
| P1-33 device assignment | Open | seat/join/transfer/unassign使用generation-qualified lease与terminal receipt |
| P1-34 hotplug/reconnect | Partial | raw disconnect cleanup已有测试；缺stable match、assignment restore、profile reconcile与Action cancel |
| P1-35 logical/physical/layout | Partial | raw keyboard与Editor chord有部分区分；缺versioned authored selector、layout/IME/AltGraph平台golden |
| P1-36 gamepad layout/glyph | Open | semantic control、Xbox/PlayStation/Nintendo/generic layout与glyph family均不存在 |
| P1-37 touch/motion/VR extension | Partial | raw touch事件存在；Action selector/composite/gesture/motion/VR capability与artifact扩展不存在 |
| P1-38 player-mappable metadata | Open | 无display category、slot、default/current key、restrictions、localization与query |
| P1-39 base map/profile delta | Open | profile只保存stable Binding delta，不能复制整图；base升级需三方reconcile |
| P1-40 rebind capture state machine | Open | noise/threshold/modifier-only/release/cancel/timeout/device filter均无产品实现 |
| P1-41 typed conflict/resolution | Open | conflict query与Replace/Swap/Keep/Cancel/Reserved rejection需基于effective context |
| P1-42 atomic rebind receipt | Open | expected artifact/profile revision、single terminal receipt、失败零可见与idempotency |
| P1-43 profile persistence/migration/cloud | Open | 接platform/settings/save owner，提供atomic restart recovery、quota/auth/cloud conflict状态 |
| P1-44 accessibility transforms | Open | hold/toggle/sticky、one-handed、sensitivity/deadzone等可组合、可审计、可重置 |
| P1-45 privacy/security | Open | raw key/IME/text/other user监听需capability；capture/log/export默认脱敏并有budget |

### 7.4 Editor 产品、跨域、测试与性能（P1-46 至 P1-60）

| 项 | 状态 | 当前差异与目标 |
|---|---|---|
| P1-46 transactional Input document | Partial | 通用dirty/save/command基础可复用；没有Input document、typed command/path、undo/merge/recovery adapter |
| P1-47 asset factory/toolkit/open route | Partial | registry/factory/toolkit原语存在；没有Input contribution、factory、thumbnail、reference或route |
| P1-48 Action Editor | Open | value type、metadata、trigger/modifier graph、references与diagnostics专用产品不存在 |
| P1-49 Mapping Context Editor | Open | grouped mappings、priority、control scheme、reorder、conflict与profile projection不存在 |
| P1-50 binding capture | Open | 复用Editor53 capture lease，建立device filter/threshold/cancel/terminal lifecycle |
| P1-51 trigger/modifier inspector | Open | schema-driven typed editor、units/ranges/capability、order与live effective value不存在 |
| P1-52 conflict graph UI | Open | effective context/profile/device graph、explanation、resolution preview与transaction不存在 |
| P1-53 live debugger | Open | per-user Action/context/source/phase/consume bounded snapshot与pause/filter不存在 |
| P1-54 PIE multi-player/device simulation | Open | 当前没有LocalPlayer，不能用一个全局manager和synthetic GamepadId伪造多玩家 |
| P1-55 locale/layout/platform/accessibility preview | Open | glyph/layout/profile/platform capability与accessibility effective binding preview不存在 |
| P1-56 Editor keymap与Gameplay map边界 | Partial | 两套authority目前分离且Editor keymap有typed override；缺共享token/capture primitive与依赖方向测试 |
| P1-57 Runtime UI consumed input接Action schedule | Partial | physical-first改动与consumed index是进展；缺sequence ownership、per-user context/evaluate producer与cancel |
| P1-58 settings/save/network owners闭合 | Open | 无profile contributor、save reference、network/replay artifact identity或security policy |
| P1-59 完整test matrix | Partial | 83个selected Zircon test declarations覆盖raw/evaluator局部；缺schema/compiler/rebind/user/device/cook/PIE/fault矩阵 |
| P1-60 性能资格与旧API迁移门 | Partial | generation/workspace与10K测试可保留；缺trigger/vector/context churn、compile/install/rebind/debug benchmark与零旁路inventory |

## 8. P2 完整性与高级能力

| 项 | 状态 | 重构目标 |
|---|---|---|
| P2-1 高级 combo/sequence graph | Open | 跨Action sequence、buffer/cancel branch与deterministic compiled automata |
| P2-2 Input recording/replay/rollback | Partial | raw bounded recording/replay已有局部底座；缺artifact/profile/user/device assignment/frame identity与deterministic Action stream |
| P2-3 自动化控制方案生成 | Open | 从metadata与platform constraints生成候选，冲突可解释且必须有review receipt |
| P2-4 平台认证与保留按键 | Open | console/mobile/desktop/web reserved/system/background/controller政策与真实认证测试 |
| P2-5 Adaptive Input与情境提示 | Open | 最近active device/user/profile驱动glyph，使用hysteresis避免抖动 |
| P2-6 per-device校准与curves | Open | family identity、drift/deadzone/response curve、安全持久化、reset与reconnect匹配 |
| P2-7 本地多人共享设备 | Open | keyboard partition、shared mouse、join/seat reassignment与ownership解释 |
| P2-8 semantic merge | Open | 按stable Action/Context/Binding/Trigger ID三方merge而非整文件文本冲突 |
| P2-9 Mod/plugin Input extensions | Open | capability/签名/预算/owner lease注册token/trigger/modifier/editor schema并安全卸载 |
| P2-10 输入延迟分析 | Open | device timestamp -> reducer -> Action trigger -> gameplay -> present的clock-calibrated trace |
| P2-11 accessibility方案共享与合规 | Open | 可导入导出且脱敏的profile、WCAG/平台检查与真实设备矩阵 |
| P2-12 大规模配置仿真farm | Open | platform/layout/device/profile/context组合的分布式compile/replay与digest/perf验证 |

## 9. 32 项验收门当前状态

| Gate | 状态 | 当前失败证据 |
|---|---|---|
| G01 asset create/save/reopen/rename/reference | Fail | 没有Input asset kind、factory、toolkit或document |
| G02 truthful unavailable/readiness | Fail | 默认disabled空manager仍被module composition构造，无compiled artifact health |
| G03 project/compiler/cook/install同源 | Fail | ProjectManifest/App无input source或artifact install |
| G04 stable source/artifact/runtime identity | Fail | Action/Context为String，Binding无ID |
| G05 invalid source typed diagnostics | Fail | duplicate/missing/unknown仍静默改变语义 |
| G06 authored bytes无runtime device slot | Fail | `GamepadId`可序列化进入binding |
| G07 two-gamepad/two-user isolation | Fail | 无InputUser/device assignment/LocalPlayer |
| G08 typed values/composite/modifiers | Fail | 只有scalar f32与简单axis direction |
| G09 deterministic trigger phases | Fail | 无trigger/phase/duration状态机 |
| G10 context priority/owner/frame barrier | Fail | priority未执行，无owner lease与barrier |
| G11 consume/reserve/UI explanation | Fail | 只有caller提供consumed tokens，无产品schedule/trace |
| G12 held-input replacement policy | Fail | 即时整图替换，无preserve/rebuild/flush |
| G13 same-frame generation consistency | Fail | generation不进入published Action snapshot或receipt |
| G14 scoped script/gameplay Action API | Fail | 产品仍调用raw `key_pressed` |
| G15 keyboard logical/physical/IME/layout golden | Fail | 局部字段存在，无artifact policy与平台矩阵 |
| G16 gamepad semantic layout/glyph | Fail | 无layout/glyph registry |
| G17 rebind capture lifecycle | Fail | 无request/state machine/receipt |
| G18 typed conflict/resolution | Fail | 无effective Gameplay conflict graph |
| G19 atomic expected-revision rebind | Fail | 无profile/revision/transaction |
| G20 base/profile reconcile | Fail | 无base artifact或profile delta |
| G21 durable profile restart/cloud | Fail | 无profile persistence contributor |
| G22 accessibility transform composition | Fail | 无Input accessibility schema/runtime |
| G23 transactional Action/Context Editor | Fail | 无Input document或Editor |
| G24 compiler/simulation currentness | Fail | 无Input compile/simulate job |
| G25 PIE two LocalPlayer/two device | Fail | LocalPlayer为0，全局manager |
| G26 reader-gated bounded live debugger | Fail | 无Action debugger/observation product |
| G27 command/gameplay schema boundary test | Fail | authority分离但无共享primitive与dependency test |
| G28 10K及新语义性能预算 | Fail | 旧10K scalar evaluator通过静态存在，完整语义未测 |
| G29 compiler/install/rebind/profile fault injection | Fail | 四类产品均不存在 |
| G30 privacy与cross-user isolation | Fail | raw key capability无user scope/profile边界 |
| G31 platform validation矩阵 | Fail | 本轮未运行，且schema/device/user产品缺席 |
| G32 long soak无stuck/cross-user/unbounded state | Fail | 无完整Action生命周期与多用户测试拓扑 |

## 10. 分层重构里程碑

### M0：Truthfulness、Inventory 与 Owner 冻结

冻结全部 raw key caller、`InputConfig.action_map`、String IDs、GamepadId serialization、Runtime117 physical identity handoff与Editor08边界；没有artifact install receipt时明确报告 Input Mapping Unavailable。

### M1：Stable Schema、Device 与 User Identity

实现 Action/Context/Binding stable IDs、value type、PhysicalInputPattern、InputUser/LocalPlayer/InputDevice assignment基础和V1 source migration diagnostics；禁止新source写入runtime slot。

### M2：Shared Compiler 与 Immutable Artifact

交付reference/collision/type/capability validation、trigger/modifier/composite IR、canonical artifact/digest/dependency manifest与跨Editor/runtime golden fixtures。

### M3：Installer、Per-user Context 与 Action Snapshot

按frame barrier安装artifact/profile，建立owner-qualified context stack、typed phase/value snapshot、consume/block/reserve、held/focus/device cancellation与bounded observation。

### M4：Asset 与 Transactional Editor

注册Action/Mapping Context asset/factory/toolkit，交付typed document、Action Editor、Mapping Context Editor、Inspector、reference navigation与compile diagnostics，接Editor02/04。

### M5：Profile、Rebind 与 Accessibility

交付capture state machine、conflict query/resolution、base/profile delta、atomic receipt、settings/platform persistence、migration/reconcile与accessibility transform stack。

### M6：Gameplay、Script 与 UI Schedule 迁移

固定physical truth -> UI ownership -> per-user context/evaluate -> gameplay consume顺序；提供scoped Action facade，迁移示例与产品脚本，限制raw key capability。

### M7：PIE、Device、Layout 与 Debugger

接入multi-LocalPlayer/device assignment、virtual device、hotplug/reconnect、keyboard/gamepad layout/glyph、platform capability与reader-gated live debugger。

### M8：Cook、Hard Cutover、Fault 与规模资格

接入artifact cook/install/LKG与schema/profile migration；对旧 `InputConfig.action_map`、raw product script和string query做零引用inventory并硬删除旁路；覆盖10K+ trigger/context churn与故障注入。

### M9：Advanced Input 与发布资格

扩展combo/recording/rollback/calibration/shared device/plugin/latency trace，以真实跨平台设备、长期soak和同语义benchmark关闭shipping gate。

## 11. 禁止的临时修补

1. 禁止只给 `InputActionMap` 加一张 serde 表格就称为 Input Editor。
2. 禁止把 Editor08 command shortcut map 直接复用为 shipping Gameplay Action Map。
3. 禁止在 asset/profile 中保存 `GamepadId`、连接顺序、window handle或当前device slot。
4. 禁止继续让 missing Context 自动 enabled、unknown Binding 静默丢失或duplicate静默first-wins。
5. 禁止把所有Action值压成 `f32` 并用命名约定模拟Axis2D/3D。
6. 禁止Editor、PIE、shipping与script分别实现trigger/modifier/consume语义。
7. 禁止rebind通过无generation整图 `set_action_map` 直接覆盖用户profile。
8. 禁止held key/axis期间改键时依赖偶然event顺序决定Action状态。
9. 禁止把raw `gameplay.key_pressed`继续作为默认玩法控制API。
10. 禁止Input Debugger/capture/log/export记录text field、IME、密码或其他用户原始按键。
11. 禁止只用synthetic `GamepadId(7)`证明hotplug、多用户或跨重启binding正确。
12. 禁止旧embedded `InputConfig.action_map` 与新artifact长期双轨；迁移完成后产品caller必须归零。
13. 禁止因 Bevy/Fyrox 没有同级 Action authoring产品而降低Zircon目标。
14. 禁止用缺少typed trigger、多用户、Editor与profile开销的10K scalar测试宣称性能优于Unreal。

## 12. 本轮验证与产出边界

本轮逐层复核当前 working-tree source、tests、项目/资源/脚本/App集成与五套本地参考源码；对旧 Editor29 的全部 5 P0、60 P1、12 P2 和 32 Gate 逐项重判。静态 exact-symbol与production-caller搜索、文件统计、SHA-256 fingerprint和共享工作树 dirty scan均在写入前完成。

本轮只修改review与索引，没有修改production Editor/runtime/interface/plugin代码或tests，没有运行Cargo、真实Editor、Input compiler、cook/install、PIE、多设备、多用户、rebind/profile、跨平台设备、fault/soak/profile或同语义跨引擎benchmark；tooling按用户要求排除。`physical_input_ownership.rs`是其他session的未跟踪在途测试，本报告只记录其源码证据，不宣称已执行通过。

结论不能作为 Input Action Mapping、device/user assignment、player profile、rebinding、accessibility或shipping integration已通过的声明。实施必须从M0开始，并在每个里程碑重取当前源码、selected manifest、fingerprint、production caller、Runtime117 handoff与动态结果。
