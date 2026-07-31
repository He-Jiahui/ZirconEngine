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
tests:
  - native plugin live-host hot-reload rollback focused regressions
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

Open state: `current-source repair and static validation complete / managed focused and Plan08
commandlet gate pending`; no fixed return or originating Plan08 gate pass is claimed.

The transition now retains both `PluginModuleKind` and an owned plugin id. Lifecycle construction
uses `NativePluginHotReloadState::new(module_kind, plugin_id.to_owned(), existing)`, while
reinsertion reconstructs the borrowed registry key with
`live_key(reload_state.module_kind, &reload_state.key)`. This keeps the registry as the only map
authority and preserves separate runtime/editor partitions for identical plugin-id text.

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-07-27 | Plan08 managed gate -> Plugins01 live-key handoff | open | Captured the terminal managed run, the two exact E0308 lifecycle locations, the lower shared ownership boundary, and the required upward rerun. |
| 2026-07-30 | Plugins01 current-source live-key transition repair | `implementation_repaired / managed_focused_and_plan08_gate_pending` | `NativePluginHotReloadState` now owns module kind and plugin id across the transition, then recreates the typed borrowed key at reinsertion. `native_hot_reload_owned_identity_reinserts_into_its_module_kind_partition`, the runtime/editor same-id partition regression, and rollback coverage exercise the boundary. Rust 1.94.1 scoped `rustfmt --check` and `git diff --check` passed for keys/hot-reload/lifecycle/test sources; current SHA-256 keys=`F36A953337560A49C237D2375F778C0582E1200046EA794BD420024BADF4A0CC`, hot_reload=`7C55981A19460E7A5A944D92EC65A94EE5C9F57A19FAF5973048CCA1AD68445D`, lifecycle=`73FEA94F128E922F4B0F2C1A882507107833D6577D7103A3AC1EC0E8361CB8F3`, test=`4CD113DA275E63DFF9299FEDBE7362B8A5AD9CE8500172EBA1312F53E50CB152`. No managed Cargo or Plan08 commandlet result is claimed. |
