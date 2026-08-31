---
status: source_complete_validation_not_submitted
created_at: 2026-08-29
implementation_status: ticket-ownership-host-retirement-overlay-and-scene-mode-source-complete-static-verified
managed_validation_status: not_submitted
related_code:
  - zircon_editor/src/core/runtime_event_consumer/error.rs
  - zircon_editor/src/core/runtime_event_consumer/mod.rs
  - zircon_editor/src/core/runtime_event_consumer/registration.rs
  - zircon_editor/src/core/runtime_event_consumer/host.rs
  - zircon_editor/src/core/runtime_event_consumer/host/contribution_lifecycle.rs
  - zircon_editor/src/scene/modes/scene_mode_registration.rs
  - zircon_editor/src/scene/modes/scene_mode_registry.rs
  - zircon_editor/src/scene/modes/scene_mode_stack.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_overlay_providers.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_scene_modes.rs
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/tests/editor_event/runtime/extensions_registration/ticketed_command_revoke.rs
reviewed_code:
  - zircon_editor/src/core/runtime_event_consumer/host/lifecycle.rs
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/ui/host/editor_extension_views.rs
  - zircon_editor/src/ui/workbench/view/view_registry.rs
reference_sources:
  - dev/UnrealEngine/Engine/Plugins/Editor/PluginBrowser/Templates/Advanced/Source/PLUGIN_NAME/Private/PLUGIN_NAME.cpp
  - dev/UnrealEngine/Engine/Plugins/Editor/PluginBrowser/Templates/EditorMode/Source/PLUGIN_NAME/Private/PLUGIN_NAMEModule.cpp
---

# Editor08 runtime consumer ticket ownership

## Current-source architecture review

`EditorRuntimeEventConsumerHost` already has the hard lifecycle mechanics needed to retire live
callbacks: one `execution_state` excludes pump/lifecycle overlap, active consumers carry a local
generation, and retirement removes local state before unsubscribe and `end_session` cleanup. The
registry feeding that host only maps `consumer_id -> registration`, however. When a plugin report is
merged, the owning Store ticket and contribution source are discarded. The host can therefore
retire all consumers at play shutdown, but it cannot select the exact callbacks owned by one revoked
plugin generation.

The adjacent view registry has the same missing owner, but descriptor removal alone would leave
layout panes and open instances referencing retired descriptor ids. Runtime consumers are the first
safe lower layer because their active objects already have an explicit retirement transaction.
Unreal's reloadable editor modules likewise unregister commands, tab spawners, and module-owned
callbacks before module shutdown; Zircon keeps its ticketed Store authority and must preserve the
same owner-before-code-unload ordering.

## Approved phase-one design

1. Every contributed consumer registration stores one private typed identity containing
   `ContributionTicket` and `ContributionSource`. Public plugin construction remains unbound; only
   host-side candidate composition may bind a Store identity.
2. `extend_contribution` consumes a private registry batch, rejects any entry already bound to a
   contribution, binds the entire batch to one identity, then reuses the existing clone-and-swap
   duplicate validation. A late duplicate publishes no entry and no owner metadata.
3. `without_contribution(ticket)` returns a fresh candidate plus deterministic removed consumer ids.
   It does not mutate the live registry. Later host teardown can retire matching active generations
   first and install this candidate only at the lifecycle commit point.
4. Revocation is keyed by ticket, not string plugin id. Two reload generations with the same package
   id must remain distinguishable. `ContributionSource` is retained for provenance and diagnostics,
   not used as an ambiguous bulk-delete key.
5. Rebinding an already-owned batch is a typed error. Silent owner replacement would let one ticket
   revoke another generation's callbacks and is forbidden.

This is a lifecycle correctness slice, not a hot-path optimization. Binding and candidate revocation
are linear in the batch/registry entry count and occur only during plugin registration or teardown;
event delivery and pump lookup remain unchanged. No performance improvement or profiler result is
claimed. The existing borrowed `RawValue` callback path in the transferred current blob is preserved.

## Remaining scope boundary

Phase four now extends the same ticket identity to executable viewport overlay providers and scene
modes. It does not call generic revoke from native module unload or release a native library.
Native callback leases must still drain before release, while view descriptor/layout/instance and
document-toolkit cleanup remain a separate owner-aware slice.

## Approved phase-two host transaction

1. Host candidate preparation clones the current registry and calls the phase-one
   `extend_contribution`; neither duplicate validation nor owner binding may mutate the live host.
2. Ticket retirement enters the existing lifecycle execution guard before reading registry or
   active state. A concurrent pump/lifecycle owner returns the existing typed busy error with no
   mutation.
3. Matching active identities are selected by the ticket carried inside their cloned registration.
   Every identity is retired through the existing generation-qualified removal, unsubscribe, and
   `end_session` path; failures are accumulated only after all matching identities are attempted.
4. The ticket-free registry candidate is published even when remote/callback cleanup reports an
   error. Local active state is removed before callbacks run, so fail-closed publication prevents a
   subsequent capability reconcile from reactivating code whose owner is being unloaded.
5. Quarantine/user-disable diagnostics and a stale round-robin cursor are cleared only for removed
   consumer ids. Deferred remote cleanup remains owned by its qualified gateway origin and is not
   discarded by registry retirement.

The phase-two primitive alone does not authorize native unload. Phase three below connects generic
plugin registration/revoke, while the final native lifecycle coordinator must still pair its cleanup
result with callback-lease quiescence and Store/router/view/mode/overlay publication.

## Approved phase-three registration transaction

1. Generic plugin registration passes its unbound consumer batch into the existing host composition
   transaction. After the private Store candidate allocates the authoritative ticket, consumer
   candidate preparation binds that exact ticket/source before any live Store, router, view, mode,
   overlay, or consumer publication.
2. Direct/builtin extension registration carries no consumer batch. The public direct consumer API
   remains an unbound builtin path and is not eligible for plugin ticket retirement.
3. Plugin revocation first validates the ticket-free Store and command-router candidates, then
   retires contributed consumers. Once active callbacks begin retiring, Store/owner/router local
   revocation is published even if consumer cleanup reports an error; the error is returned only
   after fail-closed local publication.
4. Focused host regression registers two plugin reports with distinct consumer ids, revokes one,
   and proves that its id can be registered again while the other ticket still rejects a duplicate.
   This verifies production wiring without pretending the remaining view instances and document
   toolkits are already unload-safe.

## Approved phase-four executable UI retirement

1. Overlay providers and scene-mode registrations bind the Store ticket/source only inside host
   candidate composition; plugin factories cannot replace an existing owner identity.
2. Overlay retirement builds an O(N) registry candidate, publishes it with Store/router state, and
   drops retired provider objects only after releasing the shell lock.
3. Scene-mode retirement separates registry preparation from live-stack installation. Installation
   rechecks the current stack and uses the trusted built-in Select mode as an infallible fallback,
   so runtime consumer retirement is not followed by a fallible scene publication step.
4. Matching overlays exit top-down, an owned base exits before replacement, other-ticket modes and
   enabled overlay state remain intact, and plugin objects are destroyed outside the shell lock.

## Test plan

- Bind two plugin batches to distinct Store tickets, revoke one candidate, and prove the builtin and
  second ticket remain while removed ids are deterministic.
- Attempt to bind an already-owned registry to a second ticket and require a typed error without
  changing the live target registry.
- Preserve the existing late-duplicate atomicity regression and prove contribution binding does not
  weaken its no-partial-publication guarantee.
- Run scoped format/source guards before requesting a new immutable managed Cargo snapshot. No old
  Editor08 validation request may be reused for this later source.

## 产出记录与时间

| 时间 | 状态 | 完成项目与当前门禁 |
|---|---|---|
| 2026-08-29 | `architecture-reviewed / implementation-pending / managed-validation-not-submitted` | 完成 runtime consumer registry/host/lifecycle、extension registration 与 view registry 的 current-source 全链复核。裁决先实现 `ContributionTicket + ContributionSource` 类型化 registry 归属与私有撤销候选；禁止 string owner bulk delete、live registry 原地删除和 native unload 提前接线。该切片不改 pump 热路径，不属于性能优化，因而不伪造 profiler 数据。 |
| 2026-08-29 | `source-complete / static-verified / managed-validation-not-submitted` | `EditorRuntimeEventConsumerRegistration` 内部保留 ticket/source 类型化归属；`extend_contribution` 在被消费的私有批次上完成 fail-closed bind，再复用 clone-and-swap duplicate validation；`without_contribution` 返回新 candidate 与 BTreeMap 顺序的 removed ids，不原地改 live registry。新增 `ContributionAlreadyOwned` typed error 和 2 个 ticket-focused 回归，并把既有 late-duplicate 原子性回归切换到 contribution bind 路径。`rustfmt --check`、`git diff --check` 通过；静态计数 owner struct/field `1/1`、bind/revoke-candidate API `1/1`、live retain `0`、新聚焦测试 `2`、borrowed/boxed `RawValue` payload `1/0`。旧 Editor08 managed request 早于本 slice 且尚无终态，本轮不重复提交；active host retirement、native lease quiescence 与 unload 接线仍未完成，milestone/commit/企微门禁保持未通过。 |
| 2026-08-29 | `host-transaction-architecture-reviewed / implementation-pending` | 复审 current bounded-pump Host、generation-qualified retirement、qualified gateway cleanup 与 lifecycle execution guard 后，批准 phase two：私有 ticket registration candidate + execution-guarded active retirement + cleanup-error 下仍 fail-closed 发布 ticket-free registry。generic plugin unload 与 native library release 继续禁止提前接线。 |
| 2026-08-29 | `host-transaction-source-complete / static-verified / managed-validation-not-submitted` | 新增独立 `contribution_lifecycle` 模块：Host 可准备 ticket/source-bound registry candidate，并在既有 execution guard 内按 ticket 选择 active generation、逐一走 generation-qualified retirement，随后清理对应 quarantine/user-disable/cursor 状态并发布 ticket-free registry。cleanup error 与“active but no play session”异常均在本地活体移除和 registry fail-closed 发布后进入 report；guard-busy 则在零变更时返回外层错误。新增 3 个 Host 回归，覆盖 builtin/other-ticket preservation、detached unsubscribe 失败时 active/registry/end-session 收敛，以及 busy lifecycle 零发布。`rustfmt --check`、`git diff --check` 通过；静态计数 prepare/retire/report `1/1/1`、guard/filter/retirement call `1/1/1`、Host 聚焦测试 `3`、native unload call `0`。 |
| 2026-08-29 | `registration-transaction-architecture-reviewed / implementation-pending` | 批准 phase three：consumer batch 进入既有 plugin registration candidate，在 Store candidate 分配 ticket 后绑定 owner、所有 live publish 前完成校验；revoke 在 consumer active retirement 启动后必须 fail-closed 发布 Store/owner/router 本地撤销，再返回 cleanup error。direct builtin consumer API 不伪装 plugin ticket owner。 |
| 2026-08-29 | `registration-transaction-source-complete / static-verified / managed-validation-not-submitted` | generic plugin registration 已把 unbound consumer batch 下沉到 Store candidate 分配 ticket 后绑定；duplicate consumer 会在任一 live publish 前拒绝。direct/builtin contribution 传 `None`，旧无 ticket `prepare_registration` API 已硬删除。direct、plugin、template replace、revoke 与 required-capability 五个 contribution 入口统一使用 registration gate；revoke 在 consumer callback 前释放 `shell`，busy 外层错误在重新取锁前返回零发布，cleanup report 则在 Store/owner/router 本地撤销后上抛。ticketed integration 回归增至 `3`，覆盖 consumer id release/remaining-ticket preservation 与 consumer-candidate collision 双 generation 不变。静态顺序守卫 `shell-drop-before-retire=true`、`outer-error-before-relock=true`、`cleanup-after-local-publish=true`；native unload call `0`。旧 managed request 不含本 source，继续禁止复用或重复提交。 |
| 2026-08-29 | `overlay-scene-lifecycle-source-complete / static-verified / managed-validation-not-submitted` | viewport overlay provider 与 scene-mode registration/active stack 已绑定 typed ticket/source。overlay 通过候选 registry 保留其他 owner 的实时 enabled state，retired provider 在 shell 解锁后经 plugin boundary 销毁；scene revoke 采用 prepare/install 两阶段，在 install 时重读实时 stack，并以可信 built-in Select 做不可失败回退，消除 runtime consumer 已退休后 scene fallback 失败造成的半提交。新增 provider 延迟 Drop、其他 ticket 状态保留、owned base fallback、prepare 后新激活 owned overlay 仍被回收及 Host 双插件回归。9 个精确源文件 `rustfmt --check` 与 scoped `git diff --check` 通过；未运行 Cargo，view/layout/session/document-toolkit ticket teardown 与 native provenance/lease quiescence 仍未闭合。 |
