---
title: Runtime UI Surface、Input、Focus、Pointer Capture、IME、Accessibility 与 Frame Authority 当前工作树复审
category: zircon_runtime
report_id: Runtime200
review_date: 2026-08-31
baseline_head: 14c89f9776bed828cc85e05e4b9914b3f8d1e784
verification_head: 14c89f9776bed828cc85e05e4b9914b3f8d1e784
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/77-runtime-ui-input-dispatch-routing-focus-navigation-pointer-capture-gesture-drag-drop-ime-window-lifecycle-product-integration-review.md
canonical_parent_owners:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/78-runtime-ui-accessibility-semantic-tree-name-description-relation-state-action-live-region-platform-adapter-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/82-runtime-text-editing-document-selection-caret-hit-test-ime-composition-clipboard-secure-text-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/163-runtime-platform-input-process-host-current-source-review.md
related_code:
  - zircon_runtime/src/ui/dispatch/input_manager
  - zircon_runtime/src/ui/surface/input
  - zircon_runtime/src/ui/surface/focus.rs
  - zircon_runtime/src/ui/surface/focus
  - zircon_runtime/src/ui/accessibility
  - zircon_runtime/src/ui/platform_input
  - zircon_runtime/src/ui/module.rs
  - zircon_runtime/src/dynamic_api/session/runtime_ui.rs
  - zircon_runtime/src/dynamic_api/session/runtime_ui
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_runtime_interface/src/ui
  - zircon_app/src/entry/runtime_entry_app
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateUser.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Application/SlateUser.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Input/Reply.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Text/SlateEditableTextLayout.cpp
  - dev/godot/scene/main/viewport.cpp
  - dev/godot/scene/gui/control.cpp
  - dev/godot/scene/main/window.cpp
  - dev/bevy/crates/bevy_input_focus/src/lib.rs
  - dev/bevy/crates/bevy_input_focus/src/directional_navigation.rs
  - dev/bevy/crates/bevy_input_focus/src/tab_navigation.rs
  - dev/bevy/crates/bevy_winit/src/accessibility.rs
  - dev/Fyrox/fyrox-ui/src/lib.rs
  - dev/Fyrox/fyrox-ui/src/message.rs
  - dev/Fyrox/fyrox-ui/src/widget.rs
  - dev/Fyrox/fyrox-ui/src/text_box.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugManager.cs
doc_type: current-working-tree-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Runtime200 · UI Surface / Input / Focus / IME / Accessibility / Frame Authority 当前工作树差距

## 1. 结论

当前工作树已经有一套明显超过“临时 demo”的 UI 底座：typed input/event/effect/result、preview/bubble route、per-pointer active table、modal focus restore、navigation index、IME surrounding text、clipboard transfer id、bounded action/host queues、AccessKit tree codec、dirty-domain frame publication、render segment `Arc` 复用，以及 App 对 IME/cursor/clipboard host request 的真实处理。它们应保留。

但这些能力仍未组成一个工程级 Runtime UI authority。生产产品的实际 owner 是 `dynamic_api::session::RuntimeUiSurfaceSet`，而 `UiRuntimeDriver` 只是空 unit struct；前者私有保存 `Vec<RuntimeUiSurface>`、sequence、跨 surface capture map 和输出队列，后者没有 surface registry、window session、clock、scheduler、publication 或 shutdown。`RuntimeUiManager` 又只在 `#[cfg(test)]` 下导出。这意味着模块声明、产品会话、Editor retained host 和测试 harness 仍是多套 owner，而不是同一个 runtime service 的不同 adapter。

Runtime77 的唯一 canonical P0 已得到局部修复：reply effect 现在会按 write set 捕获 `UiSurfaceMutationSnapshot`，多 effect 或 composite effect 失败时能够 restore。但是当前实现仍在输入热路径 clone 整个 tree/runtime style/invalidation/component state；`append_dispatch_effect_to_result` 添加的 default effect 不进入原 transaction；`UiInputManager` 自身的 timer、pointer table、text session 预修改也不在 snapshot 中。因此不能把旧 P0 标为 Closed，只能是 Partial。

产品闭环同样只是局部改善。Dynamic Runtime 现在会把 component template action 和 generic host request 放入有界队列，但 App 对 `UiAction`/`UiHost` 只做指数采样的 unhandled warning；dispatch 入口仍只返回 `bool`，binding report、widget event、rejected effect、diagnostic 和 commit generation 没有产品 receipt。App 已真实执行 IME enable/disable/cursor/surrounding-text request，这是进展；但请求没有 session/document generation、host ack 或 stale rejection。Clipboard 有 transfer id/result，是当前唯一接近异步闭环的分支。

时间、窗口和可访问性仍是直接阻断。`RuntimeUiSurfaceSet::next_input_metadata` 使用默认零 timestamp 和 `saturating_add` sequence；`UiInputManager::tick` 的 synthetic event 全部使用 sequence 0，且 Dynamic Runtime 没有 production tick caller。App 的 lifecycle、window status 和 file drag 只进入 core input，没有进入已经存在的 `UiWindowInputPumpEvent`。AccessKit 只存在 neutral snapshot 与 `TreeUpdate`/`ActionRequest` codec，生产 App/Editor 没有 `accesskit_winit::Adapter`、per-window action queue、activation/deactivation 或 close teardown。

本轮不新增 canonical P0，沿用 Runtime11A/77/78/82 的 owner。聚焦范围内 6 项继承 P0 当前为 `3 Open / 3 Partial / 0 Closed`；Runtime77 的 48 项 P1 当前为 `29 Open / 18 Partial / 1 Closed`；12 项 P2 仍未形成产品闭环；28 道当前工程闸门为 `20 Fail / 7 Partial / 1 Pass`。唯一明确 Closed 的 P1 是 Dynamic Runtime 已先更新 core physical input state，再依据 UI disposition 决定 camera/gameplay route；这不能外推为整个 UI 子系统已经统一。

## 2. 扫描范围与可复现快照

本轮 focused 选择集为 **265 个去重 Rust 文件、39,734 行、36,722 非空行、1,372,129 bytes、231 个测试属性、27 个 ignored marker**；fingerprint 为 `0a7463071b66bea4f931585437f59037b7f75f44f736989283e0a427de446263`。范围包括：

- `zircon_runtime/src/ui/dispatch/input_manager/**`、`surface/input/**`、focus、mutation snapshot、frame publication、accessibility 和 platform input；
- `zircon_runtime/src/dynamic_api/session/runtime_ui.rs`、其 action/host/clipboard drain 子模块、session events/state/construction/preview；
- `zircon_runtime_interface/src/ui/dispatch/**`、window、accessibility 和 focus state DTO；
- `zircon_app/src/entry/runtime_entry_app` 的 window/lifecycle/pointer/keyboard/IME/file drag 与 host request consumer；
- `UiRenderSubmission` 的 node-id projection，以及 `UiModule`/`UiRuntimeDriver`/public frame 边界。

27 个 ignored marker 主要来自已有平台/managed/performance evidence 测试。本报告只把可读取的实现和测试合同当作静态证据，不把 ignored 测试、source assertion 或测试 helper 当成产品资格。

## 3. 当前真实所有权与数据流

```text
winit RuntimeEntryApp
  -> ZrRuntimeEventV1 ABI
  -> RuntimeDynamicSession::events
       -> Core InputManager physical state
       -> private RuntimeUiSurfaceSet
            -> rebuild_dirty(viewport)
            -> UiInputManager + UiSurface route/effect
            -> bool handled
            -> bounded action/host/IME/clipboard queues
  -> drain ZrRuntimeHostRequestV1
       -> App handles IME/cursor/clipboard
       -> App logs UiAction/UiHost as unhandled

UiModule
  -> registers empty UiRuntimeDriver
  -> does not own the product flow above
```

工程级目标必须改成：

```text
UiRuntimeDriver
  -> UiSurfaceRegistry (qualified surface/window/session generations)
  -> UiWindowInputSessionRegistry (translation, identity, clock, queue)
  -> UiInputScheduler (tick/deadline/frame budget)
  -> UiDispatchTransactionService (proposal/preflight/commit/receipt)
  -> UiHostRequestService (async result/ack/compensation)
  -> UiAccessibilitySessionRegistry (per-window native adapter/action queue)
  -> UiFramePublicationService (same-generation layout/hit/focus/a11y/render)
```

`RuntimeDynamicSession`、App 和 Editor 只能持有 facade/handle 与 pinned snapshot，不能各自拥有私有 surface/input authority。

## 4. 继承 P0 当前状态

| Canonical owner | 当前 | 当前工作树证据 | 关闭条件 |
| --- | --- | --- | --- |
| Runtime77 `RUII-P0-001` reply effect partial commit | Partial | `effect.rs:78-92` 已 prepare/abort；`transaction.rs:15-28,35-79` 可 restore；但 `effect.rs:97-105` 的 append effect 绕过 transaction，manager state 不在 snapshot | 所有 route/default/manager/host proposal 进入同一 preflight/commit；失败不留下 surface 或 manager 前缀；返回 typed terminal receipt |
| Runtime11A P0-1 dispatch result 降成 bool | Partial | `runtime_ui.rs:319-344,442-444` 仍返回 bool；455 后只投影 action/host request；App `ui_action.rs`/`ui_host_request.rs` 只记录 unhandled | 产品消费完整 bounded receipt，包含 outcome、effects、binding/widget/component、host results、diagnostic code 与 generation |
| Runtime11A P0-2 无真实时钟/tick | Open | `runtime_ui.rs:447-452` timestamp 为 default；`manager.rs:197-290` tick 无 Dynamic caller，synthetic sequence 为 0 | window session 分配 monotonic timestamp/sequence；scheduler 驱动 timer/IME/tooltip/toast，暂停/恢复有 clock discontinuity receipt |
| Runtime11A P0-3 window/DPI/lifecycle 未进入 UI | Open | `events.rs:359-369,428-490` 只提交 core event；已有 `window_pump.rs` 未被产品调用 | App 事件经单一 translator/window pump 进入 UI；resize/focus/occlude/close/destroy 同代更新并执行 teardown |
| Runtime11A P0-4 UiModule 非 lifecycle owner | Open | `module.rs:18-50` 的 config 只有 enabled，driver 为空；实际 owner 在 private dynamic session | `UiRuntimeDriver` 创建、激活、tick、quiesce、destroy registry/service；产品只通过 handle 使用 |
| Runtime11A P0-5 无权威 gameplay action/data binding | Partial | action queue可投影 template invocation，但 App 只 warning；无 action handler result/reconcile；binding reports 未出产品边界 | 建立 action endpoint registry、model transaction、authorization、reply/reconcile；无 handler 必须 typed reject，不能日志后丢弃 |

Runtime11A P0-6 hit-grid allocation和 P0-7 public tree invariants 不在本轮 focused 复评范围，继续由 Runtime11A/Runtime76 的 canonical owner 管理，不能因本报告未重复列出而视为关闭。

## 5. 关键实现差距

### 5.1 Transaction 仍是 clone rollback，不是可证明的 commit protocol

`UiInputWriteSet` 是有价值的第一步，但它只选择要 clone 的 mutation domain。涉及 tree 的 transaction 会复制 `UiTree`、runtime style、invalidation 和 dirty-node set；focus/input/component/navigation 同样 clone。其成本随 surface 状态增长，并且 snapshot allocation 自身没有 budget/failure contract。commit 只向 `Vec<String>` 写一条 note，没有 commit generation、write-set digest、conflict check、owner identity 或 durable receipt。

更严重的是 transaction 边界不完整：keyboard clipboard、rich link、editable-text mutation 等路径会在初始 reply commit 后调用 `append_dispatch_effect_to_result`；`UiInputManager::dispatch_input_event` 也会在 fallible surface dispatch 前同步 text owner、清 tooltip/timer、准备 pointer/double-click 状态。后续 `UiTreeError` 不会回滚这些 manager mutation。目标不是继续扩大 clone，而是把 route 结果标准化为 immutable proposal，preflight 所有 surface/manager/host write，最后一次提交 generation-qualified delta。

### 5.2 Identity、focus 和 capture 仍不能支持 multi-seat/multi-window

`UiInputEventMetadata` 的 user/device/window/surface/pointer 全部 optional，默认 timestamp/sequence 为零。`UiFocusState` 只有一个 `focused/previous/captured/pressed/hovered` 集合；`UiSurfaceInputState` 的 capture key 只有 `UiPointerId`，IME/high-precision/pointer-lock owner 只有 node id；Dynamic 的跨 surface capture map更退化为 `BTreeMap<Option<u64>, usize>`，普通鼠标 `None` 在整个 session 只有一份 capture。

`UiActivePointerTable` 使用线性 `Vec` 查找，按钮状态只有 primary/secondary/middle 三位 mask，没有 user/device/window/surface generation，也没有 tablet pressure/tilt/twist。modal restore、focus path 和 navigation index已有真实能力，但仍是 per-surface single-seat state；多用户、虚拟用户、多个 viewport 或窗口重建后无法拒绝 stale event/capture。

### 5.3 产品 window pump 与 scheduler 未接线

Runtime 已有 `translate_winit_window_event` 和 `UiWindowInputPumpEvent`，能够翻译 resize、scale、move、focus、occlusion、cursor enter/leave、pointer、keyboard、IME 与 redraw；`window_pump.rs` 也能清 hover/transient state并更新 surface metrics。Editor retained host 已有真实 caller，但 RuntimeEntryApp 仍手工构造 ABI event，Dynamic session 再手工转 UI event。两条转换 authority 导致产品绕过 UI window lifecycle。

`UiInputManager::tick` 可以驱动 typeahead、submenu、tooltip、toast 和 IME lifecycle，但 production search 没有 Dynamic Runtime caller。timer synthetic event又都使用 sequence 0。当前 frame cadence、core clock 和 UI deadline 不共享 owner，因此 input latency、timer order、pause/resume 和 replay都不可证明。

### 5.4 Host queue 有 bounds，但没有 terminal delivery

`RuntimeUiHostRequestQueue` 和 `RuntimeUiActionRequestQueue` 都限制 row、单 row encoded bytes、aggregate bytes，并对 secure value做撤销/合并，这是应保留的安全底座。但 queue full、oversize、serialization failure 只递增 profile counter后 `continue`，普通 action/host request没有 rejection receipt或 compensation。每个 request 在输入线程同步 `serde_json::to_vec` 计算大小，热路径仍承担序列化和分配。

generic request 只携 target viewport/surface index、input sequence、request/effect index和 tree id；没有 surface generation、operation id、deadline、idempotency、request hash或 owner lease。App 对 link activation、popup、tooltip、高精度 pointer等 generic UI host request只输出 bounded warning，对 gameplay/template action也没有 handler。bounded drop 不等于工程级 backpressure；每个 admitted request必须最终是 Completed/Rejected/Expired/Cancelled/Compensated。

### 5.5 IME/clipboard 有局部产品接线，session 模型仍缺失

`UiInputMethodRequest` 已包含 cursor rect、composition rects 与 bounded surrounding text，App 也真实调用 winit IME enable/disable、cursor area和 surrounding text接口。缺口是 request 只有 kind+node owner，没有 window/surface/user/document generation；host没有 ack，窗口切换、surface rebuild、document edit 后的迟到 request 无法判 stale。`Enable/Disable/Reset/UpdateCursor` 也没有 terminal result。

Clipboard 已有 transfer id、owner 和 result event，优于其他 host request；但仍只有 text read/write，没有 MIME/data offer、selection、policy、deadline和 edit transaction compensation。cut/edit 先提交、系统剪贴板后失败时，不能原子恢复文档历史。

### 5.6 Accessibility 是 codec，不是 platform service

`accessibility/accesskit.rs` 能生成完整 `TreeUpdate`、映射角色/状态/action、处理 text selection与多 root synthetic node；但所有 production reference 都停留在该文件，调用点仅存在于 tests。与 Bevy `bevy_winit/src/accessibility.rs` 的 per-window `AccessKitAdapters`、action handler queue、active update和 window close cleanup相比，Zircon没有原生 adapter owner。

Dynamic accessibility capture会同步 `rebuild_dirty`，合并多个 surface并让后绘制 root 的 focus覆盖前者，然后只返回 JSON snapshot；action按 48-bit node id投影拆 surface并返回 bool。snapshot没有 publication generation/window identity，action没有 exposed-action generation，capture和action之间存在 stale race。`UiRenderNodeIdProjection::project` 用 mask静默截断 local node id，surface index又以高 16 位编码，没有 checked admission或 namespace exhaustion result。

### 5.7 Frame publication 有 retained domain，但还不是统一 frame authority

`UiSurfaceFramePublication` 已按 layout/render/hit/focus/pipeline/window domain维护 generation，并通过 `Arc` 复用未变 domain；Dynamic aggregate render cache也按 viewport与 per-surface render generation复用 segment。这是本轮重要的正向证据。

但 publication由 `RefCell` 懒刷新，generation使用 `saturating_add`；input、accessibility capture和render submission会各自触发 `rebuild_dirty`，没有一个 pinned frame token保证 route hit-test、focus、a11y snapshot和render属于同一 generation。aggregate cache只比 render generation，不绑定layout/hit/focus/window/asset/font generation或 surface lifetime。需要 `UiFramePublicationId` 和 immutable multi-domain snapshot，而不是由每个 consumer临时刷新。

## 6. Runtime77 P1 当前状态账本

| ID | 当前 | 当前差距 / 必须重构 |
| --- | --- | --- |
| RUII-P1-001 | Open | metadata关键 identity仍 optional；建立generation-qualified `UiInputIdentity`，缺失即 typed reject |
| RUII-P1-002 | Open | product timestamp为零、sequence saturating；由 window session 统一分配 clock/sequence/frame |
| RUII-P1-003 | Partial | action/a11y/host已有局部 item/bytes/depth budget；仍缺统一 ingress/event/effect/path allocation budget |
| RUII-P1-004 | Open | key/control/popup/link等仍混用 String；建立versioned typed code和extension namespace |
| RUII-P1-005 | Open | synthetic/platform/replay/a11y缺统一 origin/trust/lineage envelope |
| RUII-P1-006 | Partial | 已有 `UiInputWriteSet`，但不覆盖manager/host/default effect且无precondition/conflict |
| RUII-P1-007 | Partial | surface/frame有generation，dispatch result没有base/commit generation和terminal outcome |
| RUII-P1-008 | Partial | request已有sequence/index，仍缺operation id、owner generation、deadline、idempotency/hash |
| RUII-P1-009 | Partial | clipboard有result；IME/popup/link/pointer-lock等没有ack/reconcile/compensation |
| RUII-P1-010 | Open | route、manager、default behavior仍走不同effect apply路径；统一proposal/committer |
| RUII-P1-011 | Open | handled/propagation/default prevention/effect validity未形成正交 typed state |
| RUII-P1-012 | Open | rejected reason和diagnostic notes仍是String；改为bounded code+redacted context |
| RUII-P1-013 | Open | focus仍是per-surface single-seat；建立per-user/per-window focus publication |
| RUII-P1-014 | Partial | Dynamic按反向surface order route并有capture surface；仍无global focus/modal arbiter与surface z snapshot |
| RUII-P1-015 | Open | capture只按pointer id/Option id；改为user/device/window/surface/generation qualified lease |
| RUII-P1-016 | Open | active pointer是linear Vec和三键mask；建立bounded indexed contact/tool/button state |
| RUII-P1-017 | Partial | frame发布focus path、modal restore已存在；仍缺per-seat path和严格lost/gained/within transaction |
| RUII-P1-018 | Partial | navigation index已出现；仍需证明随tree/layout generation原子更新且无route full scan |
| RUII-P1-019 | Partial | modal group/trap/restore已有实现；tab boundary、wrap policy和多seat scope仍不完整 |
| RUII-P1-020 | Open | spatial navigation仍缺transform/clip/RTL/manual edge的统一score与differential corpus |
| RUII-P1-021 | Open | focus commit没有同代bring-into-view/ancestor scroll-chain receipt |
| RUII-P1-022 | Open | Runtime没有gesture recognizer arena、competition、team和cancel authority |
| RUII-P1-023 | Open | tap/hold/pan/pinch/rotate/fling未统一，double-click timer只是局部规则 |
| RUII-P1-024 | Open | callback/subscription仍缺统一 unregister token、generation和retire barrier |
| RUII-P1-025 | Open | drag session缺source/target/window/surface/user generation lease |
| RUII-P1-026 | Open | drag payload/accept缺typed MIME、operation、policy/data-offer negotiation |
| RUII-P1-027 | Open | App file drag仍只进入core input，未桥接 UI external drag/drop |
| RUII-P1-028 | Partial | retained drag/capture/overlay有局部实现；threshold/autoscroll/cross-surface/cancel rollback未统一 |
| RUII-P1-029 | Open | IME owner仍只有node；建立per-window text input session+document revision |
| RUII-P1-030 | Open | IME request没有host ack、stale拒绝和terminal result |
| RUII-P1-031 | Partial | winit/App已接composition/delete/lifecycle/geometry；仍缺跨平台同一conformance与session receipt |
| RUII-P1-032 | Partial | clipboard有text transfer id/result；仍缺MIME、selection、policy、deadline |
| RUII-P1-033 | Open | cut/edit与clipboard host outcome未绑定一个history transaction |
| RUII-P1-034 | Open | 仍只有AccessKit codec，没有per-window adapter/action queue/close teardown |
| RUII-P1-035 | Open | Runtime App手工ABI转换，Editor才使用winit translator；收敛为一个window input session |
| RUII-P1-036 | Partial | internal window pump有清理顺序；产品未调用且无幂等teardown receipt |
| RUII-P1-037 | Partial | action/host output不再全丢，但产品仍返回bool且App generic consumer只warning |
| RUII-P1-038 | Open | Dynamic Runtime没有UI tick/deadline scheduler owner |
| RUII-P1-039 | Closed | `events.rs` 已先提交 core physical input，再依据UI handled决定camera/gameplay route |
| RUII-P1-040 | Open | Dynamic/Editor/其他产品仍有多套翻译、dispatch和gesture authority |
| RUII-P1-041 | Open | gesture/file drag/tablet/extra button/lock capability矩阵不完整，unsupported未结构化 |
| RUII-P1-042 | Partial | translator可带pointer point，但Dynamic wheel/多入口仍不能证明同代location语义 |
| RUII-P1-043 | Partial | navigation index、segmented frame降低局部clone；transaction full-state clone和route allocation仍在热路径 |
| RUII-P1-044 | Partial | render segment已Arc共享；跨surface input仍clone event/route context和payload |
| RUII-P1-045 | Open | 产品没有按window/source分舱的input queue、move coalescing和per-frame admission budget |
| RUII-P1-046 | Open | replay/journal没有完整identity、route、proposal、commit和host result |
| RUII-P1-047 | Partial | 有局部profile counters；缺route depth/stale/host latency/commit/fault的低开销关联指标 |
| RUII-P1-048 | Open | 缺真实多窗口、多DPI、多seat、IME/a11y、fault/soak/perf产品资格 |

## 7. P2 产品化与性能

Runtime77 的 `RUII-P2-001..012` 继续有效：proposal/committed命名、versioned input profile、compact contact/button store、route/effect scratch arena、generated platform map、typed modal facet、input inspector、gesture/capture visualizer、capability evidence、structured diagnostic、supported命名清理和标准 workload catalog。当前局部 frame/profile counter只能把其中部分标为 Partial，不能减少这 12 项的验收范围。

新增实现不得用以下方式伪装性能完成：在输入线程深 clone整个 surface作rollback；用同步 JSON serialize作queue admission；用 `saturating_add` 隐藏generation/sequence耗尽；或以cache hit counter代替P50/P95/P99 latency、allocation、queue age和跨帧stale率。

## 8. 参考引擎差异

| 参考 | 可核对的工程结构 | Zircon 当前差异 |
| --- | --- | --- |
| Unreal Slate | `FSlateApplication::ProcessReply`集中处理reply；`FSlateUser`按user保存focus、pointer captor和last widgets；capture使用user+pointer index；application统一tick/finish frame/window route | Zircon reply application、manager pre/post state和App host output仍分裂；focus/capture没有user/window generation；module driver不拥有产品session |
| Godot | `Viewport::_gui_input_event`拥有GUI route、mouse focus mask、drag/focus cleanup；Control直接向AccessibilityServer发布节点/动作；Window/Viewport生命周期是一条产品路径 | Zircon内部window pump存在但产品绕过；external file drag未进UI；accessibility只生成离线snapshot/JSON，没有平台server/adapter |
| Bevy | `InputFocus`/`FocusedInput`与window traversal由plugin schedule拥有；`AccessKitAdapters`按window建表，action queue、active update和WindowClosed cleanup明确 | Zircon UI module没有schedule owner；Dynamic private set无window entity/session；AccessKit codec没有adapter生命周期 |
| Fyrox | `UserInterface`集中拥有message queue、captured node、keyboard focus、OS event处理和poll；text commit/focus消息在同一owner内 | Zircon surface、input manager、dynamic set、App ABI和host queues跨owner；失败/queue drop没有同一poll/receipt边界 |
| Unity Graphics | 本地 `Graphics` 参考只提供 SRP Rendering Debugger 的 `DebugManager`、panel/widget registration和runtime debug UI开关 | 该仓不是通用 UI Toolkit/input/accessibility权威，只能参考debug UI registration/lifecycle；不能用它证明Zircon通用Runtime UI完成度 |

这些参考用于提取 owner、lifecycle、identity、publication 和 adapter 结构，不要求逐API复制，也不把参考引擎自身限制当成Zircon目标上限。

## 9. 工程闸门

| Gate | 当前 | 通过条件 |
| --- | --- | --- |
| G-UI-01 | Fail | `UiRuntimeDriver`实际拥有surface/window/input/a11y/frame registry及lifecycle |
| G-UI-02 | Fail | 每个产品event有非零monotonic timestamp、唯一sequence和clock domain |
| G-UI-03 | Fail | user/device/window/surface/pointer/session generation完整且关键字段不可缺省 |
| G-UI-04 | Partial | reply effect可rollback，但default/manager/host mutation必须进入同一atomic commit |
| G-UI-05 | Partial | clipboard有result；所有host effect都需operation/deadline/ack/compensation |
| G-UI-06 | Fail | 产品获得完整typed dispatch terminal receipt，不再只收bool |
| G-UI-07 | Fail | per-seat/per-window focus、focus-visible、modal restore和notification顺序可证明 |
| G-UI-08 | Fail | capture是qualified lease，window/surface重建后stale event必拒绝 |
| G-UI-09 | Fail | gesture arena覆盖tap/hold/pan/pinch/rotate/fling与deterministic cancel |
| G-UI-10 | Fail | internal/cross-surface/OS drag使用同一data-offer/operation protocol |
| G-UI-11 | Partial | App IME接口已接线；仍需session generation、ack和三平台conformance |
| G-UI-12 | Partial | clipboard text correlation已存在；仍需MIME/policy/history transaction |
| G-UI-13 | Fail | Runtime App所有window event经一个translator/window pump进入UI |
| G-UI-14 | Fail | deactivate/occlude/close/destroy执行幂等quiesce/drain/cancel receipt |
| G-UI-15 | Fail | 每个产品window有真实AccessKit/platform adapter、action queue和teardown |
| G-UI-16 | Partial | surface domain generation已存在；a11y/input/render仍需同一pinned publication id |
| G-UI-17 | Fail | global node/surface id使用checked namespace，不静默mask/truncate/exhaust |
| G-UI-18 | Partial | queue有item/byte bounds；full/oversize必须返回typed terminal backpressure |
| G-UI-19 | Fail | `UiInputManager` timer/pointer/text state与surface state同一transaction |
| G-UI-20 | Fail | production scheduler调用tick并处理pause/resume/deadline/sequence |
| G-UI-21 | Partial | retained frame domain存在；hit/focus/a11y/render必须证明同代 |
| G-UI-22 | Fail | App对UiAction/UiHost有真实handler或typed reject，不是warning sink |
| G-UI-23 | Fail | Windows/macOS/Linux真实窗口、多DPI、多输入设备与AT smoke矩阵通过 |
| G-UI-24 | Fail | effect、queue、host、window close、adapter activation有fault injection |
| G-UI-25 | Fail | pointer storm、deep route、large tree、IME、a11y有P95/P99 latency和allocation预算 |
| G-UI-26 | Fail | canonical input/dispatch/host journal可deterministic replay并检测stale |
| G-UI-27 | Fail | Dynamic/App/Editor只消费同一Runtime UI authority，不保留双轨兼容入口 |
| G-UI-28 | Pass | Dynamic产品已先更新core physical input state，再处理UI disposition |

## 10. 分阶段重构顺序

1. **冻结 owner 和公共合同**：让 `UiRuntimeDriver` 拥有 `UiSurfaceRegistry`、window session、scheduler、host/a11y/frame service；定义 qualified identity、surface/window generation、publication id、operation id、typed error和terminal receipt。此阶段先保留现有 surface算法，不重写widget。
2. **收敛输入 transaction**：route只生成proposal；preflight write set、target lease、tree/surface generation和host capacity；一次commit surface+manager delta。删除 `append_dispatch_effect_to_result` 旁路和full-state clone rollback，失败返回无前缀的rejected receipt。
3. **硬切 product window path**：RuntimeEntryApp直接使用同一 winit translator/window session；ABI只承载中立event envelope；resize/focus/occlude/file drag/close/destroy统一进入window pump。接入真实clock与tick scheduler。
4. **完成 host async protocol**：IME、clipboard、cursor/lock/high-precision、link、popup、template action都采用admit -> host -> result -> reconcile；queue full/timeout/shutdown产生terminal receipt，secure payload维持现有redaction/revocation。
5. **完成 focus/capture/gesture**：per-seat focus、qualified pointer contact/capture、modal/focus restore、navigation graph和gesture arena共享window/surface generation；建立external drag data offer和跨surface route。
6. **完成 accessibility/frame publication**：每个window创建native adapter，action绑定已发布generation；layout/hit/focus/a11y/render消费同一immutable frame snapshot；node id namespace checked，window close移除adapter并取消pending action。
7. **建立资格矩阵**：先做deterministic/fault tests，再做Windows/macOS/Linux真实IME/a11y、多窗口、多DPI、多seat；最后以large tree/pointer storm/deep route/host stall/long soak证明P95/P99、allocation和queue-age预算。

## 11. Review-only 验证记录

- 已逐文件统计 265 个 focused Rust 文件，并对 transaction、manager、surface state、Dynamic product path、App window/host path、interface DTO、accessibility codec和frame publication建立当前工作树证据链。
- 已核对 Runtime11A、Runtime77、Runtime78、Runtime82、Runtime163 的 canonical owner；本轮不重复登记既有 P0/P1。
- 已重读本地 Unreal、Godot、Bevy、Fyrox 与 Unity Graphics 参考切片；Unity Graphics 仅作为 Rendering Debugger 生命周期参考。
- 本轮只修改 review 文档、索引和 coverage；没有修改 Runtime/App/Editor/Interface 生产代码或测试。
- 未运行 Cargo、真实窗口、IME、screen reader、fault injection、UI automation、scale、soak或benchmark；27个ignored marker不作为通过证据。
- 未查询、轮询、等待或实时跟踪协调器状态；全部结论来自本地当前工作树和本地参考源码。

实现会话应从 G-UI-01、02、04、06、13、19、20、22 开始；在统一 owner、clock、transaction 和 product window path 前，不应继续堆叠新的widget或只增加更多测试helper来宣称Runtime UI已工程化。
