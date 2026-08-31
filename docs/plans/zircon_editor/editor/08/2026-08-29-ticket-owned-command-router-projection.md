---
status: source_complete_validation_pending
created_at: 2026-08-29
implementation_status: ticket-owned-router-exact-handle-tool-resource-declaration-runtime-consumer-view-scene-overlay-lifecycle-native-live-action-generation-guard-source-complete-static-verified
managed_validation_status: admission_acquired_wrapper_post_timeout_no_cargo_result
related_code:
  - zircon_editor/src/core/commands/registry.rs
  - zircon_editor/src/core/commands/contribution.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/core/extension/store/batch.rs
  - zircon_editor/src/core/extension/store/model/lifecycle.rs
  - zircon_editor/src/core/extension/store/model/contribution_store.rs
  - zircon_editor/src/core/context/tool_scheduler.rs
  - zircon_editor/src/core/tools/resource_catalog.rs
  - zircon_editor/src/core/plugin/materializer.rs
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/ui/workbench/shell_state.rs
  - zircon_editor/src/ui/retained_host/app/module_plugin_actions/host_actions/live_actions.rs
  - zircon_editor/src/core/runtime_event_consumer/error.rs
  - zircon_editor/src/core/runtime_event_consumer/registration.rs
  - zircon_editor/src/core/runtime_event_consumer/host.rs
  - zircon_editor/src/core/runtime_event_consumer/host/contribution_lifecycle.rs
  - zircon_editor/src/tests/editor_event/runtime/extensions_registration/overlay_lifecycle.rs
  - zircon_runtime_interface/src/editor_contribution.rs
  - zircon_plugins/plugin_sdk/src/editor_contribution.rs
reference_sources:
  - dev/UnrealEngine/Engine/Plugins/Editor/PluginBrowser/Templates/Advanced/Source/PLUGIN_NAME/Private/PLUGIN_NAME.cpp
  - dev/UnrealEngine/Engine/Plugins/Editor/PluginBrowser/Templates/EditorMode/Source/PLUGIN_NAME/Private/PLUGIN_NAMEModule.cpp
---

# Editor08 ticket-owned command router projection

## Current-source architecture recheck

`ContributionStore` already owns immutable ticket generations and removes contributed commands,
operation factories, views, menus, asset types, and other descriptors from its snapshot on
`revoke(ticket)`. The live `EditorCommandRegistry` remains a separate mutable clone, however:
`register_editor_extension_owned` incrementally installs explicit commands, generated view-open
commands, operation factories, capability requirements, and asset-write targets into that clone,
then publishes it after the Store ticket is created. No production path rebuilds or revokes this
router when a ticket retires.

The result is a split authority. A disabled or unloaded plugin can disappear from the immutable
contribution snapshot while its command and operation-factory entries remain invokable. Reversing
individual mutations is not reliable because multiple active tickets may reference the same
built-in operation and contribute the same asset-write target.

Unreal's editor plugin template treats module shutdown as the inverse of startup: command sets,
tool-menu ownership, and tab spawners are unregistered before a reloadable module is unloaded.
Zircon keeps its existing ticketed immutable Store instead of copying Unreal's global registries,
but follows the same lifecycle rule: executable routes may not outlive their contribution owner.

## Approved hard-cut design

1. `ContributionStore` exposes active retained batches in ticket order and supports cheap candidate
   cloning. The shared Store remains the only contribution lifetime authority.
2. Host composition projects a fresh `EditorCommandRegistry::default_workbench()` from all active
   Store batches. Explicit commands, generated view-open commands, factories, capability gates,
   menu bindings, and asset-write targets are recomputed from the same batches.
3. Registration first mutates a private Store candidate, builds and validates the command-router
   candidate, and only then publishes either authority. Rejected registration publishes neither
   Store generation nor router state.
4. Plugin revocation removes its Store ticket in a private candidate, rebuilds the router from the
   remaining active tickets, then publishes the Store/owner state and immediately replaces the
   command registry while holding the lifecycle gate. This is failure-atomic candidate publication,
   not a claim that two independent mutexes form one cross-lock linearizable snapshot.
5. Native unload is not wired to this revoke API until native library leases and in-flight command
   quiescence are explicit. Store snapshots and command dispatch can both retain cloned factory
   registrations; changing call order alone would permit factory vtables to outlive the DLL.
6. The rebuilt command registry publishes exactly one generation after the preceding live router.
   Construction-time descriptor counts are not revisions: equal-sized projections with different
   owners/content must never collide, and the first Store projection must not move backwards from
   the built-in router generation.

No old mutable registration path, compatibility router, command alias, or per-call owner filter is
retained. This slice does not claim full plugin shutdown until live view registrations, scene-mode
instances, viewport overlay providers, and runtime-event consumers have equivalent owner teardown.

The candidate rebuild is lifecycle work, not a frame or command-dispatch hot path. Its cost is
linear in active contribution batches and their command-bearing descriptors, with deterministic
ticket ordering. This deliberately replaces error-prone inverse deltas: shared built-in routes and
asset-write targets make per-ticket mutable rollback ambiguous, while a Store projection has one
clear authority. No runtime performance improvement is claimed, so this structural repair does not
substitute static counts for a profiler report.

## Exact contribution handle and ABA hard cut

The prior host revoke API accepted only `owner_id: &str`. That is not a generation identity: after
plugin generation A retires and a same-id generation B registers, a delayed second teardown from A
would match and revoke B. The tool authority's generation checks cannot repair that host-level ABA
error because the string lookup happened before the scheduler owner was selected.

Plugin registration now returns an `EditorContributionHandle` issued only after all contribution
candidates validate. The handle binds the display owner id, immutable `ContributionTicket`, and
`ToolOwnerGeneration`; `OwnedContribution` stores the same value. Host retirement hard-cuts to
`revoke_editor_plugin_contribution(&handle)` and requires an exact match. The former string revoke
entry point is removed rather than retained as a compatibility path. A stale handle is an
observable idempotent no-op and cannot mutate Store revision, command generation, or tool authority
snapshot for a reloaded same-id plugin.

Tool instance allocation follows the same handle boundary. `allocate_editor_tool_instance` validates
the exact live handle while holding the contribution lifecycle gate, then passes its private owner
generation to `ToolSchedulerService::allocate_instance_id`. Plugin callers do not receive a raw
generation, and a stale handle cannot allocate an instance under the new same-id plugin generation.
Tool authority failures use the dedicated `ToolScheduler` registry error category instead of being
misreported as view-registration failures.

## Tool resource declaration transaction

The previous scheduler foundation exposed owner generation issuance and resource-kind registration
as separate mutations. That split could publish an owner and a prefix of its resource declarations
before a later duplicate, reserved namespace, or capacity failure. It also left resource kinds
outside the serialized plugin contribution contract, forcing callers to know scheduler lifecycle
details that belong to the host.

The hard cut introduces ownerless `ToolResourceKindDeclaration` values in
`EditorExtensionRegistry` and `ContributionBatch`, plus the versioned
`zircon.editor.tool-resource-kind/1` serialized DTO and plugin SDK builder. Deserialization and
materialization canonicalize a non-empty typed scope set and preserve the explicit
`Forbidden | Optional | Required` channel policy. `ContributionStore` is the sole source-namespace
admission owner: plugins are limited to `plugin.<package-id>.*`, and Builtin source cannot publish
extension resource kinds. Rejection leaves the Store generation and snapshot unchanged.

`ToolSchedulerService::register_owner_generation` now accepts the complete declaration set. Under
one authority lock it selects a tentative generation, binds every declaration into a cloned
catalog, and publishes the owner, catalog, lifecycle events, and revision only after the whole set
passes. A failed set leaves generation issuance, active owners, catalog, revision, outbox, and
journal unchanged. Owner revoke removes the generation's catalog entries and also retires active or
queued claims from other owners that depend on those kinds before surviving claims are promoted.
The former owner-bound constructor, independent resource-kind registration mutation, and no-arg
owner registration calls are deleted rather than retained as compatibility paths.

During the required structure review, the Store-to-command projection and its operation-binding
validators moved from the host lifecycle file into `core/commands/contribution.rs`; the registry
error definition moved into the descriptor leaf. Current production owners are 714 lines for
`editor_extension.rs`, 493 before tests for contribution descriptors, 423 before tests for command
contribution projection, and 571 before tests for host registration, all below the repository's
800-line review warning. This is ownership convergence, not a runtime performance claim.

## Cross-owner revoke transaction review

The exact contribution handle closes the same-package ABA hole, but current-source review found
that it does not yet make the complete plugin revoke failure-atomic. The host prepares the candidate
Store/router and scene-mode retirement, then destructively retires extension views, runtime event
consumers, and the tool owner generation before publishing the candidate Store, owner list, and
command registry. Those owners still expose independent fallible boundaries: view close/unregister
can fail after earlier view mutations, runtime-consumer retirement can reject a busy lifecycle
before publication, and tool-owner revoke can reject cleanup mutation or revision exhaustion before
its authority mutation. A later rejection can therefore leave an earlier owner retired while the
Store/router still advertises the contribution.

This is a structural follow-up, not something that call-order changes or best-effort re-registration
can repair. The next immutable scope must include the view-retirement owner, runtime-consumer
contribution lifecycle, `ToolSchedulerService`, scene-mode/overlay retirement, and the host
publication root. Each owner must provide a prepared candidate or retirement lease; all fallible
preflight must complete before one serialized commit publishes every owner generation, while remote
or callback cleanup failures become post-commit receipts. Until that transaction exists, this child
plan claims exact identity, Store/router candidate safety, and fail-closed native live actions only;
it does not claim atomic whole-plugin revoke or hot reload.

## Native binding provenance hardening

The current native binding already carries a loader-issued `plugin_id`, but two host boundaries
previously trusted that field without comparing it to the contribution owner: serialized
materialization accepted a binding from another package, and generic host registration admitted a
binding whose owner differed from the registration package. Both paths now reject the candidate
before publication with an explicit host/materialization diagnostic. The check is
deliberately before Store/router mutation, so a provenance mismatch cannot leave a partially
registered command or an executable callback retained by the wrong ticket. This establishes the
identity check needed by the later native unload/quiescence transaction; it does not claim that
library unload itself is wired or that callback leases are drained by this slice.

## Native unload safety review

The runtime native loader already protects ABI callbacks with
`NativePluginLibraryGenerationOwner`, callback leases, and a lifecycle transition gate. The current
native editor path is narrower: it reads `SerializedContributionBatch` from the loaded report and
materializes host-owned descriptors, so it does not currently create Rust command factories, scene
modes, or overlay-provider trait objects whose vtables live in the DLL.

The generic editor registration path is broader and can carry those executable trait objects. Its
runtime-event consumers are now bound to the Store ticket and retired behind the callback execution
guard during generic revoke. Registration also returns an exact contribution handle, but current
startup and live-action topology still has two independent native generations: project registration
calls `load_discovered_native_editor_plugins(...)` and materializes command bindings from that load
report, while retained-host construction creates a separate default `NativePluginHostHandle` for
Plugin Manager live actions. A later hot reload only replaces the latter host map. The Store/router
continues to invoke the former generation, so package-id equality cannot prove generation identity
and must not be used as an unload bridge.

This slice adds a fail-closed admission gate, not hot-reload completion.
`execute_native_live_action_without_active_contribution(...)` holds `plugin_registration_gate`
across both the active-owner check and backend closure. If the package still owns an exact editor
contribution generation, neither Unload nor Hot Reload reaches the independent live backend;
runtime-only packages with no active editor contribution retain the existing live action. This
prevents new split-authority damage and closes the check/dispatch TOCTOU window, but intentionally
does not revoke a contribution by string id or claim two independent DLL loads are the same
generation.

The complete follow-up must be one generation-aware replacement transaction:

1. load and validate the replacement as a private live-host candidate while retaining its exact
   `LoadedNativePlugin` generation and materialized editor registration;
2. prepare old exact-handle retirement and new Store/router/view/mode/overlay/tool candidates before
   publishing either side;
3. begin the old loader lifecycle transition, which atomically rejects new callbacks and fails busy
   when active leases are nonzero;
4. commit old exact-handle retirement, invoke the old unload callback, install the replacement
   registration, and publish the replacement loader generation as one coordinated transaction; and
5. on any pre-commit rejection, cancel the old transition without publishing; on any post-unload
   replacement failure, remain explicitly fail closed with the old contribution revoked and the
   replacement generation unpublished and cleaned up.

Changing only the before/after call order would not establish these invariants. This review is a
structural safety finding, not a measured performance optimization.

## Test plan

- Begin with a regression that registers two plugin batches, revokes one owner, and proves its
  command, generated view-open route, menu binding, factory, and asset-write mutation disappear
  while the second owner's routes remain.
- Prove a rejected candidate leaves Store generation and command registry generation unchanged.
- Prove owner revoke retires active scene-mode instances and viewport overlay providers in the
  same host transaction, including built-in Select fallback and typed stale-ID rejection.
- Prove a stale handle from a retired generation cannot revoke a reloaded same-id plugin and leaves
  Store, command, and tool authority generations unchanged.
- Keep native unload disconnected from generic owner revocation until typed provenance, callback
  quiescence, and executable runtime-object teardown have their own regression fixture.
- Run scoped format and source guards first; submit source-bound managed Windows Cargo after current
  hashes are attributed. Pending validation is not an accepted milestone.

## 产出记录与时间

| 时间 | 状态 | 完成项目与当前门禁 |
|---|---|---|
| 2026-08-30 | `native-live-action-generation-guard / source-complete / static-verified / managed-validation-admission-post-timeout` | 对照本地 Unreal reloadable editor module 的 Startup/Shutdown 对称注销后，复核 Zircon project native registration、callback lease、lifecycle transition、live-host backend 与 exact contribution handle 全链，确认工程打开的 serialized contribution load report 与 Retained live host 的默认 handle 是两个独立 DLL generation；当前 Hot Reload 只替换后者，Store/router binding 仍指向前者，字符串 package id 不能证明 generation 同一。新增 `execute_native_live_action_without_active_contribution`，在同一个 `plugin_registration_gate` 临界区内完成 active exact owner 检查和 backend closure：有 editor contribution 的 Unload/Hot Reload fail closed 且 backend 调用为零，无 active contribution 的 runtime-only 路径保持可用；未增加 string revoke 或兼容入口。新增纯边界回归覆盖同 owner 拒绝和其他 owner 放行；2 个目标 Rust 文件 `rustfmt --check`、局部 `git diff --check` 通过，host production/total 行数 `571/655`，live action `28` 行。完整 loader candidate + contribution prepare/commit + callback transition + generation publication 事务仍待 runtime/backend 作用域闭合。原 `D:\ZirconBuilds\mvp-test-fixtures-36724` 已不存在；Windows validator 的 `cargo.acquire` request `43c4f327dd2d4198b62470d5a073cf4f` 后续完成并分配 job `1270146887da4a6f96529c1e85914b16`，但包装器已因 `command_post_timeout` 退出，终态证据为 `status=leased`、`started_at=null`、`command=[]`、`live_process_pids=[]`、`cleanup_scheduled=true`，没有 Cargo 进程或退出码。本行不声明编译测试、独立 review、accepted milestone、commit、企微、性能或功耗验收；Failure artifacts 未修改。 |
| 2026-08-30 | `tool-resource-contribution-transaction-and-owner-module-convergence / source-complete / static-verified / managed-validation-blocked` | 工具资源类型声明已经纳入 `zircon.editor.tool-resource-kind/1` 序列化 DTO、SDK 规范化、materializer、`EditorExtensionRegistry`/`ContributionBatch`、Store namespace admission 和 host 发布链；Store 拒绝 builtin 来源及跨插件 namespace，scheduler 以 `register_owner_generation(resource_kinds)` 在克隆候选中原子登记 owner generation 与全部资源类型，成功仅推进一次 authority revision，任一声明失败时生产代码不发布 Store、owner authority 或 tool catalog 候选。旧分裂入口静态扫描 `ToolResourceKindRegistration::new(`、`.register_resource_kind(`、`.register_owner_generation()` 均为 `0`；command contribution 投影定义为 `1`，host 重复 ownership validator 为 `0`。为满足代码结构约定，Store-to-command 投影归并到 command owner module，registry error 归并到 descriptor leaf；生产代码行数为 `714/493/423/537`，Store 测试文件 `705` 行，均低于 `800` 行 review warning。17 个目标 Rust 文件 `rustfmt --edition 2024 --config skip_children=true --check`、局部 `git diff --check` 和 `cargo metadata --no-deps --format-version 1 --locked` 通过；新增失败回归覆盖跨插件 namespace 与 builtin source 均保持 Store generation `0` 且 Store 为空，但受管 Cargo 仍在进程创建前被既有 `D:\ZirconBuilds\mvp-test-fixtures-36724` 阻断，因此本行不声明编译测试、独立 review、accepted milestone、commit、企微、性能或功耗验收；Failure artifacts 未修改。 |
| 2026-08-30 | `exact-contribution-handle-and-tool-allocation-aba-hardcut / source-complete / static-verified / managed-validation-blocked` | 插件注册现在返回宿主签发的 `EditorContributionHandle { owner_id, ContributionTicket, ToolOwnerGeneration }`，shell 保存同一 handle；撤销硬切为 exact-handle 匹配，旧 `owner_id: &str` 撤销入口与调用扫描均为 `0`。工具实例分配也必须提交该 handle，宿主在 lifecycle gate 下验证后才把私有 generation 传给 scheduler；stale handle 返回 typed `StaleContributionHandle`，tool authority 错误使用独立 `ToolScheduler` 分类。新增同名插件 A 撤销、B 重载、A 迟到撤销不得改变 Store revision、command generation、tool authority snapshot 且不得为 B 分配工具实例的回归，并迁移 view/scene/overlay 撤销测试。5 个目标 Rust 文件 `rustfmt --check` 与局部 `git diff --check` 通过；精确撤销调用 `9`、宿主分配调用 `4`、handle 引用 `12`，`cargo metadata --no-deps --format-version 1 --locked` 通过。既有受管 Cargo 仍在进程创建前被 `D:\ZirconBuilds\mvp-test-fixtures-36724` 阻断，本行不声明编译测试、独立 review、里程碑、commit、企微或性能验收；Failure artifacts 未修改。 |
| 2026-08-29 | `architecture-reviewed / implementation-starting / managed-validation-not-submitted` | 完成 current Store/router/host registration 全链复核，并以 Unreal reloadable editor module 的 startup/shutdown 对称清理为参考。裁决使用“活跃 ticket 批次 -> 全量候选命令路由”单向投影，禁止可变路由补丁、调用点 owner filter 或兼容表。本轮先闭合 commands/menu/factory/asset-write 路由；view/mode/overlay/runtime-consumer 的运行时 teardown 仍须后续同形 owner lifecycle，当前不宣称完整 plugin shutdown。 |
| 2026-08-29 | `source-complete / static-verified / managed-validation-request-accepted` | `ContributionStore` 增加可克隆候选和按 ticket 顺序的 active batch 只读投影视图；host 注册与撤销均先构建私有 Store 候选，再从 `default_workbench` 全量重建 command registry，候选失败不发布 Store/command generation。注册与撤销共用 `plugin_registration_gate`。重建后的 router 从上一个已发布 generation 严格 `+1`，禁止“不同内容但等规模”目录复用 revision，也禁止首次 Store 投影从内建 router generation 倒退。新增 2 个 `ticketed_command_router_*` 回归，覆盖显式 command、生成 view-open route、operation factory、asset-write target、menu descriptor、剩余 ticket 与内建命令保留、注册/撤销后 generation 单调前进，以及候选拒绝双 generation 不变。复审 native unload 后撤回了不安全的自动接线：通用 Store old snapshot 与 command dispatch 均可持有 factory clone；当前 native serialized contribution 虽为 host-owned，live action 仍缺类型化 provenance，不能以调用顺序伪装通用安全。`rustfmt --check`、`git diff --check` 通过；静态计数为候选 Store clone `2`、实时 router clone `0`、生命周期 gate 使用点 `2`、router generation 发布点 `1`。E 盘 immutable copy + 固定 `zr_vm` commit 的 `cargo test -p zircon_editor --lib --locked --offline ticketed_command_router` 请求 `070a425110164a1b8e12a6b0cf9b3c5f` 已被协调器接受但无终态，且该 immutable request 早于本次安全性复审，不能作为最终 source snapshot 验收。view/mode/overlay/runtime-consumer teardown 与 native unload provenance/quiescence 尚未闭合，因此 milestone/commit/企微门禁保持未通过。 |
| 2026-08-29 | `runtime-consumer-ticket-lifecycle-source-complete / static-verified / validation-not-resubmitted` | runtime consumer registry/active generation 已绑定 `ContributionTicket + ContributionSource`；Host 提供 execution-guarded prepare/retire/report，cleanup error 下 fail-closed 发布，guard-busy 下零发布可重试。generic plugin registration 在 Store candidate 分配 ticket 后、任一 live publish 前准备 consumer candidate；revoke 在释放 `shell` 后退休回调，再发布 Store/owner/router。五个 contribution 变更入口统一使用 registration gate，旧无 ticket prepare API 删除。新增 registry/Host/integration 聚焦回归 `2/3/3`，其中 consumer collision 证明 Store/command 双 generation 不变；静态顺序守卫全部为 true，native unload call `0`。旧请求 `070a425110164a1b8e12a6b0cf9b3c5f` 不含本 source，禁止复用；剩余门禁为 view/mode/overlay owner teardown、typed native provenance/lease quiescence、最终 managed validation 与 review。 |
| 2026-08-30 | `view-scene-overlay-owner-retirement-source-complete / static-verified / managed-validation-blocked` | 宿主撤销回归覆盖同一插件 ticket 下的活动 scene mode、viewport overlay provider 与普通 toggle command：撤销后 scene mode 强制回退内建 Select，旧 mode/provider ID 均以 typed `UnknownMode`/`UnknownProvider` fail closed，command projection 消失，重复撤销返回 `false`。验证通过目标文件 `rustfmt --check`、局部 `git diff --check` 与 `cargo metadata --no-deps --format-version 1 --locked`。最新受管 `zircon_editor -SkipTest` 请求在 Cargo 前被协调器拒绝：`unmanaged_artifacts_detected`，未登记产物 `D:\ZirconBuilds\mvp-test-fixtures-36724`；无 Cargo 进程/退出码，故不声明编译测试通过。Failure artifacts 未修改。 |
| 2026-08-30 | `native-binding-provenance-source-complete / static-verified / managed-validation-blocked` | Native serialized materializer 与 generic host registration 均在候选 Store/router 发布前核对 binding `plugin_id` 与当前 package/owner；跨插件绑定现在以 fail-closed host/materialization diagnostic 拒绝，新增两条纯边界回归覆盖 mismatch/一致性。`rustfmt --check` 与 `git diff --check` 通过；未重新声称 Cargo 验收，既有 unmanaged artifact 阻断仍有效。Failure artifacts 未修改。 |
| 2026-08-30 | `toolkit-menu-source-api-hardcut / static-verified / managed-validation-blocked` | `toolkit_menu_tests` 已迁移到当前生产 `default_menu_bar_with_sources` 的 i18n/locale-aware 8 参数接口，未增加旧签名兼容层；聚焦工具菜单的跨源去重与 canonical command 门禁测试继续覆盖新架构。目标文件 `rustfmt --check` 与 `git diff --check` 通过；受管 Cargo 验证仍被未登记产物 `D:\ZirconBuilds\mvp-test-fixtures-36724` 在 Cargo 前阻断。Failure artifacts 未修改。 |
| 2026-08-30 | `store-native-binding-provenance-source-complete / static-verified / managed-validation-blocked` | `ContributionStore` 的插件 source namespace 校验现在覆盖每条 native binding：binding `plugin_id` 必须精确等于 `ContributionSource::Plugin` owner；Builtin source 携带 native binding 直接 fail closed。新增 Store lifecycle 纯边界回归，错误保留 command/owner/binding 诊断，未引入兼容路径。目标文件 `rustfmt --check` 与 `git diff --check` 通过；受管 Cargo 仍在 Cargo 前被未登记产物 `D:\ZirconBuilds\mvp-test-fixtures-36724` 阻断。Failure artifacts 未修改。 |
| 2026-08-30 | `tool-scheduler-identity-claim-hardcut / static-verified / managed-validation-blocked` | 修复 `ToolScheduler::acquire_set` 的同一 `ToolId` 覆盖缺陷：活动集合或排队集合已存在时，新的不同集合以 typed `AcquireDenial::AlreadyHeld/AlreadyQueued` fail closed 并发布 `SetDenied`，旧 holder/队列保持不变。Scene viewport 不再复用固定 `editor.scene.viewport` identity；同一 `ToolSchedulerService` 统一分配 `editor.scene.viewport.<ordinal>` 实例 ID，第二 controller 排队撤回及析构不能释放第一 controller 的原子资源集合。构造期工具身份与原子资源集合已硬切为必有状态，删除不可达的 `SceneToolIdentityUnavailable` 兼容分支。新增活动 claim、pending claim 与双 controller 隔离回归；固定 scene tool 字面量、可选 identity/resource 字段、旧错误变体扫描均为 `0`。未引入兼容路径，`rustfmt --check` 与 `git diff --check` 通过；受管 Cargo 仍在 Cargo 前被未登记产物 `D:\ZirconBuilds\mvp-test-fixtures-36724` 阻断。Failure artifacts 未修改。 |
| 2026-08-30 | `tool-transition-outbox-source-complete / static-verified / managed-validation-blocked` | `ToolSchedulerService` 已从“解锁后逐事件发布”硬切为同锁提交 `ToolTransitionRevision + FIFO outbox`、单 dispatcher 按 revision 发布单条 `ToolTransitionBatch`；一次 release/promotion 的多事件不再允许被其它 transition 穿插。旧 `ToolMessage::Lifecycle` 变体删除，delivery retained-bytes 估算改按 batch 聚合。新增可查询 `ToolSchedulerDeliveryHealth`，记录 committed/dispatched revision、delivered/unobserved batch、drop、backpressure 与 dispatch error，并以 `requires_resync` 暴露不完整观察状态。新增 Barrier 控制的反向 caller-dispatch 并发回归和零 subscriber health 回归，既有 single/set bus 测试迁移为严格 revision/batch 断言；测试源码 `4` 组，旧消息变体全仓扫描 `0`、新 batch 入口 `4`。`rustfmt --check` 与 `git diff --check` 通过；本 slice 未运行 Cargo，受管入口仍在 Cargo 前被未登记产物 `D:\ZirconBuilds\mvp-test-fixtures-36724` 阻断。snapshot/cursor resync、typed instance/request/lease ID 与 bounded journal 仍归后续 authority 里程碑，不在本行宣称完成。Failure artifacts 未修改。 |
| 2026-08-30 | `tool-authority-observation-and-fairness-foundation / source-complete / static-verified / managed-validation-blocked` | 完成按需确定性 `ToolSchedulerStateSnapshot`、同一 authority 锁内的 revision-qualified snapshot、容量 `256` 的默认有界 transition journal、consumer cursor 连续读取、future cursor typed error 与 journal gap 原子 resync；subscriber 可在 bus 未观察、drop/backpressure/error 后从权威快照恢复，不在每次 transition 克隆全状态。按 `optimize/zircon_editor/53` 的 G06/G08 修复全局 set head 阻塞无关 single resource 与单步 promotion：set 只保留其重叠资源，重叠 single 仍让位于 FIFO set head，一次 release/withdraw 将连续可运行 set 前缀晋升到有界 fixpoint；release/withdraw outcome 硬切为完整 `ToolSetActivation[]`，旧单晋升字段与 `promote_set_head` 扫描均为 `0`。single-per-resource、global set 与 journal 预算拆为三个独立域，旧 `DEFAULT_MAX_QUEUE_PER_RESOURCE`、`with_max_queue_per_resource` 与同名 getter 全源码扫描为 `0`。新增/迁移聚焦测试共 `33` 个源码用例，其中 observation `7`、snapshot `2`、fairness/fixpoint/预算 `5`；相关生产 owner 为 scheduler `784` 行、service `345`、observation `155`、transition `67`、limits `32`，均低于 `800` 行警戒线。scoped `rustfmt --check`、`git diff --check` 通过；受管 Cargo 仍在编译前被 `D:\ZirconBuilds\mvp-test-fixtures-36724` 非托管产物阻断，故未声明编译/测试、review、milestone、commit、企微或性能验收。Definition/Instance/Request/Lease ID、owner generation、typed scope/resource key、shutdown/fault state 与真实 input capture 仍为后续硬切，Failure artifacts 未修改。 |
| 2026-08-30 | `tool-definition-instance-contract-hardcut / source-complete / static-verified / managed-validation-blocked` | M1 identity 第一阶段完成硬切：删除通用 `ToolId` owner/file/export，新增分别校验的 `ToolDefinitionId` 与 `ToolInstanceId { definition, non-zero ordinal }`；definition 上限 `107` bytes，确保拼接最大 `u64` ordinal 后 qualified instance ID 精确不超过 `128` bytes，instance serde 改为结构化 definition/ordinal。scheduler holder/queue/set、lifecycle event、snapshot、service、Scene viewport 与 Export wizard 全部只传播 typed instance；service 分配入口只接受 typed definition，并以 typed exhaustion/revision error 在 authority 变更前 fail closed。Scene identity 改为首次真实激活时懒分配，Export UI session identity 与可重建的后台 job ID 分离且在 session 生命周期内稳定。`ExclusiveResource + ToolResourceSet` 从接近预算的 scheduler owner 拆到独立 `resource_set.rs`，scheduler 从 `797` 行降至 `716` 行，identity/resource owner 分别为 `208/87` 行；相关生产 owner 未命中 `unwrap/expect/panic`。核心聚焦测试源码为 `35` 项，其中 authority observation/error `9`、snapshot `2`、fairness/fixpoint/预算 `5`；旧 `ToolId`/旧 error/旧 max constant、字符串分配入口、旧 queue API 与 `promote_set_head` 合并扫描 `0`。scoped `rustfmt --check`、`git diff --check` 通过；受管 Cargo 仍在进程创建前被 `D:\ZirconBuilds\mvp-test-fixtures-36724` 阻断，故未声明编译/测试、独立 review、milestone、commit、企微或性能验收。Request/Lease ID、owner generation、typed scope/resource key、shutdown/fault state 与真实 input capture 仍为后续硬切，Failure artifacts 未修改。 |
| 2026-08-30 | `tool-authority-shutdown-fault-foundation / source-complete / static-verified / managed-validation-blocked` | authority 新增可查询且随 snapshot 传播的 `Open -> Quiescing -> Draining -> Closed` 状态；quiesce 与 instance/acquire 在同一 authority 锁内仲裁，quiesce 后新实例和新请求以 typed `AuthorityUnavailable` 零变更拒绝。close 不执行 promotion，按 active set、resource/single FIFO、set FIFO 的确定顺序释放/撤销全部 claim，并返回 released single/set lease 与 withdrawn single/set request 四类计数；同一 instance 同时持有 set 与无关 single 时按 `(instance, resource)` 精确清理，不漏 deactivation。authority/dispatcher mutex poison 不再无条件 `into_inner()` 继续 Open 服务：首次发现写入 revisioned `Faulted` transition、清除 mutex poison flag 后 fail-stop 拒绝普通 mutation，仅 close 可进入 Draining 清空 claim；fault batch 尚未 dispatch 时 `committed_revision != dispatched_revision` 直接要求 resync。`ToolLifecycleEvent::AuthorityStateChanged` 已进入 batch serde 与 retained-byte estimate。核心聚焦测试源码增至 `38` 项，其中 authority/service `11`；新增无 promotion shutdown、quiesce/close 幂等、poison fail-stop 与 revision-qualified fault snapshot 回归。生产 owner 行数为 state `43`、identity `208`、resource `87`、scheduler `767`、service `522`、observation `166`，均低于 `800` 行警戒线；service 旧 poison fallback 与生产 `unwrap/expect/panic` 扫描为 `0`，scoped `rustfmt --check`、`git diff --check` 通过。受管 Cargo 仍在进程创建前被 `D:\ZirconBuilds\mvp-test-fixtures-36724` 阻断，故未声明编译/测试、独立 review、milestone、commit、企微或性能验收。Request/Lease ID、owner generation、typed scope/resource key、真实 input capture 及 project/window/editor host shutdown 调用链仍为后续硬切，Failure artifacts 未修改。 |
