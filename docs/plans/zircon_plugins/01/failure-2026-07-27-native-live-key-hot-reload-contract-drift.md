---
handoff_kind: failure
status: open
created_at: 2026-07-27
summary_slug: native-live-key-hot-reload-contract-drift
origin_plan: docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
fixing_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
origin_child_dir: docs/plans/zircon_editor/editor/08
fixing_child_dir: docs/plans/zircon_plugins/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/keys.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/hot_reload.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/lifecycle.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/hot_reload_state.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/hot_reload_publication.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/runtime_behavior.rs
tests:
  - bulk_reload_reopens_the_old_generation_when_loaded_lock_reacquisition_fails
  - unload_reopens_the_retained_generation_when_loaded_lock_reacquisition_fails
  - hot_reload_reopens_the_retained_generation_when_loaded_lock_reacquisition_fails
  - hot_reload_reports_replacement_cleanup_failure_after_publication_fails
  - hot_reload_keeps_retained_generation_transition_active_when_rollback_restore_fails
  - hot_reload_keeps_retained_generation_transition_active_when_publication_rollback_restore_fails
  - cargo test --package zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 native_hot_reload_owned_identity_reinserts_into_its_module_kind_partition
  - cargo test -p zircon_editor --lib --locked commandlet
---

# Plugins01: native live-key hot-reload contract drift

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md`
- 来源执行切片：plugin-list commandlet current-source managed gate
- 修复责任计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 交接原因：the native live-host registry, reload transition state, and lifecycle rollback all
  belong to the Plugins01 native ABI/hot-reload authority.

## 失败现象与复现证据

Plan08 source-bound managed job `a6ceedf9324f4976b54a96f806f12992` / run
`bbc3a39561d544329ea15aaf55fae384` naturally released with `exit 101` at
`2026-07-26T18:29:59Z`; its commandlet filters never executed. Raw stderr records:

- `E0308` at `native_plugin_live_host/lifecycle.rs:290`: hot-reload construction passes
  `NativePluginLiveKey<'_>` where `NativePluginHotReloadState::new` accepts `String`.
- `E0308` at `native_plugin_live_host/lifecycle.rs:435`: the reload state's `String` key is
  passed to `NativePluginLiveRegistry::insert`, which requires `NativePluginLiveKey<'_>`.

The same terminal run also exposed independently routed UI-template and asset-migration errors;
these two diagnostics are the native live-host subset only.

## 最低共享层根因

`NativePluginLiveRegistry` correctly changed steady-state lookups to a borrowed, module-kind
partitioned `NativePluginLiveKey`, but transition-owned `NativePluginHotReloadState` still stores
an owned `String`. Lifecycle construction and reinsertion cross that ownership boundary without
an explicit conversion, so neither the borrowed registry contract nor the transition state type
is coherent.

## 架构修复验收

- Make the hot-reload transition boundary explicitly convert between its owned plugin identity
  and the borrowed, module-kind-aware `NativePluginLiveKey` required by the registry.
- Preserve module-kind partitioning for both lookup and rollback insertion; ids that share text
  across module kinds must remain distinct.
- Keep the registry as the sole map authority and retain the existing transition/rollback
  lifetime guarantees.
- Run focused hot-reload/rollback regressions, then rerun the originating Plan08 commandlet gate
  against a fresh immutable source snapshot.

## 禁止临时方案

- Do not restore a composite-string registry key, add a second map, or make module kind implicit.
- Do not add a call-site-only overload, compatibility alias, or test bypass.
- Do not claim the Plan08 commandlet filters passed; compilation stopped first.

## 修复结果与回传

Open state: `current-source rollback-atomicity repair accepted / managed focused hot-reload gate
green / originating Plan08 commandlet compilation stopped at external graphics errors`; no fixed
return or originating Plan08 gate pass is claimed.

The transition now retains both `PluginModuleKind` and an owned plugin id. Lifecycle construction
uses `NativePluginHotReloadState::new(module_kind, plugin_id.to_owned(), existing)`, while
reinsertion reconstructs the borrowed registry key with
`live_key(reload_state.module_kind, &reload_state.key)`. This keeps the registry as the only map
authority and preserves separate runtime/editor partitions for identical plugin-id text.

The current-source managed focused attempt used reservation
`d5d140fb5af8404bacfacb09fc917a44`, job `5bf931d5a4804cf79bd2a4388f32d21c`, and run
`c17aa219d74c47b5a8017ae8acbd6e9b`. It naturally released at
`2026-08-01T07:10:18Z` with `exit 101`, no live process ids, and the complete source manifest
unchanged. Compilation stopped before the target test because the shared runtime crate had 17
errors in asset project import/index, dynamic-scene session/spawn, world bootstrap, preference
storage tests, and feature-gated diagnostic logging. None of those paths is owned by Plugins01,
and no target-test pass is claimed from this run.

An independent review of the live-key slice returned `C0/I2/M1`: the Editor fixture still built
a Runtime-shaped entry, rollback coverage did not prove the retained generation transition was
cancelled, and Native/Vm same-text/remove isolation was absent. The owned test boundary now
constructs the requested Runtime or Editor report, verifies failed reload can be followed by a
successful unload, and exercises Runtime/Editor/Native/Vm partitions plus remove isolation.
A later 2026-08-08 current-source review found two additional rollback-atomicity defects: an old
generation was reopened even when restoring its saved state failed, and a replacement generation
was not unloaded after a final publication failure. A fresh follow-up review then identified three
remaining repair gaps: publication errors discarded replacement-cleanup diagnostics, the bridge
publication recovery path held the loaded-registry mutex while calling plugin hooks, and retained
restore failure coverage could falsely pass without calling the retained restore callback. The
lifecycle repair now retains the publication cause together with rollback diagnostics, releases the
loaded-registry mutex before recovery callbacks, and keeps the old generation transition active
until its restore succeeds. The rollback tests are isolated in
`tests/hot_reload_publication.rs`; their retained restore failure uses a distinct callback and
counter, and the cleanup-failure case asserts that the returned error exposes the replacement
unload diagnostic. Scoped Rust 1.94.1 `rustfmt --check` and `git diff --check` pass; no managed
Cargo, fresh review acceptance, commandlet, or fixed return is claimed.

The retried current-source managed focused gate used job
`01d97d074aa64f17a43e64397d695be8`. It was released with `exit 1` on 2026-08-08 after
compilation failed, and executed `0` `native_plugin_live_host::tests` cases. Its source-manifest
compiler diagnostics belong to the Runtime08 ECS Bundle transaction slice: `E0308` at
`scene/world/typed_api/bundle_transaction.rs:530`, plus `E0277` at
`scene/tests/ecs_typed_api/bundle_width.rs:167` and `:183` because `Health` is a `Component`,
not a `Bundle`. These are not Plugins01 paths, so this record does not claim a hot-reload pass or
alter the implementation/review result. At subsequent review the shared worktree already had
uncommitted Runtime08 changes (including a `Some(...)` wrapper at the former E0308 location and
the new `bundle_width.rs`), so the job result is historical source-manifest evidence only. The
lowest owner is `docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md`. Runtime08
session `runtime08-ecs-bundle-width-current-compile-r1-20260808` now owns the width test and its
failure record. It confirmed the current `Bundle` contract deliberately accepts unary tuples,
changed both width test calls to `(Health(...),)`, and verified scoped rustfmt/diff/source shape.
Its pre-edit Windows managed focused request was correctly deferred because compatible pool job
`775162eecde34cacb2e2b7d31584d1d4` owned the FIFO; that job has since released. The post-repair
request failed before materialization with `database is locked`, and the one controlled retry was
not submitted after `cargo.acquire` coordinator health preflight timed out. No Runtime08 test
result exists yet. That session must publish a terminal focused result after coordinator recovery
before Plugins01 retries this unchanged gate.

Runtime08 has now published that narrow current-source recovery result: coordinator-managed job
`999e63ce48254d0f8e08dd7cdad74389` ran on Windows Rust 1.94.1, released with `exit 0`, and
executed both the `zircon_runtime` production build and the `ecs_typed_api::bundle_width` lib
test filter. The test-library compilation includes `hot_reload_publication.rs`, so it also proves
the restored `successful_runtime_command` helper import compiles. This is only the shared compile
recovery prerequisite: no native hot-reload behavior test, fresh behavior-review acceptance, or
originating Plan08 commandlet gate is claimed, and this handoff remains `open`.

After the Runtime08 recovery, Plugins01 ran the current source through the managed Windows
Rust 1.94.1 test lane with `--no-default-features --features core-min`. Both
`cargo build -p zircon_runtime --locked` and the focused
`native_plugin_live_host::tests::hot_reload` library-test filter completed successfully. Two
independent current-source reviews of the rollback follow-up now conclude `C0/I0/M0`; the
publication error keeps cleanup diagnostics, recovery drops the loaded-registry guard before
plugin callbacks, and retained-generation restore failure is tested with a distinct callback and
counter.

The required upward `cargo test -p zircon_editor --lib --locked commandlet` gate was then run in
the same managed Windows target lane. It failed during `zircon_runtime` compilation before any
commandlet filter could execute, with five external diagnostics: Rust-2024-only let-chain syntax
in `render_graph/builder/compile.rs:231`; an unresolved deferred-lighting import in
`scene_renderer_core.rs:20`; a private UI-text raster report field in
`render_framework/.../base_stats.rs:269`; an inferred shadow-frame-plan type in
`render_scene.rs:93`; and missing `FromBuffer`/`PerPixel` compute-dispatch variants in
`compute_workload.rs:105`. These files are outside Plugins01 ownership. The record remains open
until their plan owners repair the shared compile boundary and a fresh Plan08 commandlet run can
execute its target tests.

A 2026-08-10 current-source static re-audit confirms that the five historical file paths have
since been removed by their owning graphics refactors and that their replacement boundaries contain
the expected repairs: the render-pipeline ordering loop uses Rust-2021-compatible `let ... else`,
the scene-renderer core imports `SceneRendererDeferredLightingProfile` from its current owner,
frame statistics use public report accessors, the shadow plan is carried through an explicit
`Option<ShadowFramePlan>` construction, and compute workload accounting handles both `FromBuffer`
and `PerPixel`. This makes the upward retry source-ready, but it is static evidence only. An
unrelated active `zircon_runtime` compilation still owns the shared Cargo process, so no new
Plugins01 Cargo job or Plan08 target-test result is claimed.

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-07-27 | Plan08 managed gate -> Plugins01 live-key handoff | open | Captured the terminal managed run, the two exact E0308 lifecycle locations, the lower shared ownership boundary, and the required upward rerun. |
| 2026-07-30 | Plugins01 current-source live-key transition repair | `implementation_repaired / managed_focused_and_plan08_gate_pending` | `NativePluginHotReloadState` now owns module kind and plugin id across the transition, then recreates the typed borrowed key at reinsertion. `native_hot_reload_owned_identity_reinserts_into_its_module_kind_partition`, the runtime/editor same-id partition regression, and rollback coverage exercise the boundary. Rust 1.94.1 scoped `rustfmt --check` and `git diff --check` passed for keys/hot-reload/lifecycle/test sources; current SHA-256 keys=`F36A953337560A49C237D2375F778C0582E1200046EA794BD420024BADF4A0CC`, hot_reload=`7C55981A19460E7A5A944D92EC65A94EE5C9F57A19FAF5973048CCA1AD68445D`, lifecycle=`73FEA94F128E922F4B0F2C1A882507107833D6577D7103A3AC1EC0E8361CB8F3`, test=`4CD113DA275E63DFF9299FEDBE7362B8A5AD9CE8500172EBA1312F53E50CB152`. No managed Cargo or Plan08 commandlet result is claimed. |
| 2026-08-01 | Plugins01 current-source focused attempt | `managed_compile_failed_before_target / external_owner_repairs_pending` | Reservation `d5d140fb5af8404bacfacb09fc917a44`, job `5bf931d5a4804cf79bd2a4388f32d21c`, run `c17aa219d74c47b5a8017ae8acbd6e9b` released `exit 101`, `0` target tests, and no live PIDs. Job and terminal SHA-256 match for all manifest inputs, including keys=`F36A953337560A49C237D2375F778C0582E1200046EA794BD420024BADF4A0CC`, hot_reload=`7C55981A19460E7A5A944D92EC65A94EE5C9F57A19FAF5973048CCA1AD68445D`, lifecycle=`8E674D16AA6E217091D54C53249FA09A8BEA034F6521D32557594860F5977EFD`, parent tests=`381E33E3F9E6208AA7A870604B55B56A7D82823445A36EEBED1F74D9CDE3F8F5`, and hot-reload tests=`188898D5005D339CC21E730AA63E7A0196389CA931E5B43164B142A3BC7B8079`. The 17 diagnostics are outside the Plugins01 path boundary. |
| 2026-08-01 / 2026-08-08 | Independent live-key review follow-up | `review_followup_repaired_static / fresh_review_and_managed_execution_pending` | The original review repaired the Editor projection, post-rollback unload admission, and four-partition/remove isolation. The later current-source audit found the remaining old-restore admission and unpublished-replacement cleanup defects; its follow-up identified lost cleanup diagnostics, callback-under-lock recovery, and an insufficient retained-restore regression. The lifecycle repair adds publication diagnostics, releases the loaded guard before rollback callbacks, and covers cleanup failure plus distinct retained-restore failure. Rust 1.94.1 `rustfmt --check` and `git diff --check` pass, but no managed focused result, fresh review acceptance, commandlet pass, or fixed return is claimed. |
| 2026-08-08 | Plugins01 current-source focused retry | `managed_compile_failed_before_target / Runtime08_owner_handoff_pending` | Job `01d97d074aa64f17a43e64397d695be8` released `exit 1`; no target tests executed. The run-source compile failed at Runtime08 ECS Bundle transaction/test paths with one `E0308` and two `E0277` diagnostics. The current worktree has uncommitted Runtime08 source drift after that source manifest, so no current-source result is inferred. Runtime08 must reconcile, focus-validate, and return managed evidence before this unchanged Plugins01 gate can retry. |
| 2026-08-08 | Runtime08 ECS Bundle width repair | `test_call_repaired / managed_focused_unavailable_pending_coordinator_recovery` | Runtime08 r1 confirmed `Bundle` is tuple-only and changed the two width spawns to `(Health(...),)`. Rustfmt, diff, and source-shape checks pass. The previous compatible pool job released, but the post-repair request hit `database is locked` before materialization and the one retry was not submitted after `cargo.acquire` health preflight timeout. No current-source test result or upward Plugins01 retry is claimed. |
| 2026-08-08 | Runtime08 current-source compile recovery | `managed_build_and_bundle_test_green / Plugins01_behavior_and_Plan08_gates_pending` | Coordinator job `999e63ce48254d0f8e08dd7cdad74389` ran Windows Rust 1.94.1 in the managed target lane and released `exit 0`: `cargo build -p zircon_runtime --no-default-features --features core-min --locked` and `cargo test -p zircon_runtime --no-default-features --features core-min --lib ecs_typed_api::bundle_width --locked` both passed. Full lib-test compilation includes the repaired `hot_reload_publication.rs` import. No native hot-reload behavior test, fresh behavior-review acceptance, Plan08 commandlet result, or fixed return is claimed. |
| 2026-08-08 | Plugins01 rollback-atomicity follow-up | `managed_hot_reload_green / independent_review_C0_I0_M0 / Plan08_compile_recovery_pending` | The current-source managed Windows Rust 1.94.1 lane passed `cargo build -p zircon_runtime --no-default-features --features core-min --locked` and `cargo test -p zircon_runtime --no-default-features --features core-min --lib native_plugin_live_host::tests::hot_reload --locked`. Follow-up review confirms publication cleanup diagnostics, unlocked recovery callbacks, and non-spurious retained-restore coverage (`C0/I0/M0`). The immediately following Plan08 `cargo test -p zircon_editor --lib --locked commandlet` run failed before commandlet execution on five external render/text compile diagnostics, so no upstream pass or fixed return is claimed. |
| 2026-08-10 | Plan08 compile-boundary static re-audit | `external_compile_repairs_present / managed_upward_retry_pending` | The five historical graphics paths no longer exist. Their current owners contain the Rust-2021 let-else, deferred-profile import, report-accessor, shadow-plan inference, and compute-dispatch match repairs. This is retry-readiness evidence only; an unrelated active Cargo/rustc tree prevented a managed Plugins01 retry. |
