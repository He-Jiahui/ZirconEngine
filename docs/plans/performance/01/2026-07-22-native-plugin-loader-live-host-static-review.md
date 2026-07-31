---
related_code:
  - zircon_runtime/src/plugin/native_plugin_loader/discover.rs
  - zircon_runtime/src/plugin/native_plugin_loader/discover/authority.rs
  - zircon_runtime/src/plugin/native_plugin_loader/collect_manifests.rs
  - zircon_runtime/src/plugin/native_plugin_loader/discover_load_manifest.rs
  - zircon_runtime/src/plugin/native_plugin_loader/candidate_from_manifest.rs
  - zircon_runtime/src/plugin/native_plugin_loader/load_discovered.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_report.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_load_report
  - zircon_runtime/src/plugin/native_plugin_loader/loaded_native_plugin.rs
  - zircon_runtime/src/plugin/native_plugin_loader/behavior_calls.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
  - docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_editor/editor/12-plugin-management.md
reference_sources:
  - dev/godot/core/extension/gdextension_manager.cpp
  - dev/godot/core/extension/gdextension_interface.cpp
  - dev/bevy/crates/bevy_app/src/app.rs
tests:
  - zircon_runtime/src/plugin/native_plugin_loader/discover/tests.rs::discovery_report_duplicate_index_borrows_snapshot_identity_and_reserves_output
  - zircon_runtime/src/tests/plugin_extensions/native_plugin_loader.rs::native_loader_validates_owned_load_manifest_candidates_without_clone_or_shift_remove
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/runtime_behavior.rs::native_live_host_runtime_snapshot_restore_borrows_state_payload_after_unlock
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/hot_reload_state.rs::native_hot_reload_moves_the_saved_state_blob_into_restore_and_rollback
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/hot_update_application.rs::native_load_report_bridge_lifecycle_borrows_loaded_plugin_ids
  - zircon_runtime/src/plugin/native_plugin_loader/host_callbacks.rs::tests::native_host_capability_probe_streams_delimited_tokens_without_owned_list_projection
  - current-source Windows Cargo and F0/F2/F4 product traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Native plugin loader与live host逐文件性能静态审查（2026-07-22）

## 范围与覆盖

`zircon_runtime/src/plugin/native_plugin_loader/**`当前生产 **47** 个Rust文件，本切片逐文件阅读其中非validation **42/47**：discovery authority、cold/export manifest discovery、candidate/load、load-report projection、ABI declaration/probe、bridge binding、compatibility/string/error、host callbacks、host adapter/context、registration manifest、loaded stable library/behavior call，以及`native_plugin_live_host` root与11个生产child全组。测试文件只读取并修改了与源码守卫直接相关的局部，不冒充11个native-loader tests已全量审查；剩余5个`behavior_validation{.rs,/**}`生产文件和全部tests继续留在`pending.md`。

受管Cargo再次申请时CPU lane已改由reservation `f02e44954d8f47b680ad33e822e0d9fa`、Session `runtime02-descriptor-snapshot-r3-20260722`占用；没有运行raw Cargo。该切片无GPU执行路径，RenderDoc继续留给F2/Render17门。

## 已确认的性能形状

- discovery authority已有canonical alias、16-root LRU、generation snapshot、bounded cold traversal和watcher增量应用；unchanged filesystem工作为0。但`discover()`仍在root mutex内应用watcher并读取/解析变化manifest，随后为owned public report深clone全部candidate/package manifest/diagnostics。PERF-MVP-539仅删除额外duplicate索引和load-manifest中转复制；完整warm report ownership/锁内I/O继续由既有`native-plugin-discovery-recursive-rescan` failure收口。
- `NativePluginLoadProjection`已经能让live-host一次operation共享manifest/diagnostic projection；但`NativePluginLoadReport`的多个public convenience getter各自调用`self.projection()`，跨getter仍会重复clone/merge/sort。PERF-MVP-038与`native-load-report-repeated-projection`保持open，要求consumer持同一immutable projection，而不是为每getter建cache。
- foreign callback已从全局loaded-table mutex外移，动态库用`Arc<NativePluginStableLibrary>`延寿，原全局长锁根因已修复。不过每次同plugin callback仍锁per-owner callback-state并做四类共享原子诊断；高频native system/command会争用同一cache line，新增PERF-MVP-541。
- command callback每次构造`CString`，owned output由plugin分配、host复制到Vec再调用free；大payload产生双owner峰值和内存带宽，新增PERF-MVP-542。Godot用`StringName`取得稳定identity，并先解析`MethodBind`再调用，支持Zircon把命令/方法identity编译到load generation，而不是每call重建字符串。
- live-host表以格式化`"runtime:{plugin_id}"` String作为key；bridge scope/single method查询仍clone完整package manifest和bindings并线性扫method，绕开已落地的registration replay shared context，新增PERF-MVP-543。
- native host capability callback原先每probe复制/parse完整granted list；PERF-MVP-544已改为borrowed CStr token流并早停，entry grant projection也以requested/granted set替代双线性scan。`HostContextRegistry`稳定lookup已无writer mutex，但每个新slot仍clone完整slot Arc Vec，bridge call还多一层context Arc clone和BTreeMap method lookup，新增PERF-MVP-545。
- hot update仍在caller同步完成delta pack staging/promotion/receipt、manifest parse、DLL open/entry、逐plugin reload与诊断排序；大变更会堆积主线程。执行预算回链Runtime11，transaction/last-good/reload quiescence仍归Plugins01/Runtime06。

## 本轮直接止损

1. **PERF-MVP-539**：discovery report为输出Vec精确reserve，duplicate package索引借用snapshot plugin-id/path；export load manifest按`pop → validate → push`移动candidate，删除完整candidate clone与`Vec::remove`移位，按entry数reserve并只规范化一次export root。
2. **PERF-MVP-540**：play-mode bulk restore在释放loaded-table锁后直接借用输入snapshot bytes；hot reload把state owner中的blob move到restore/rollback，删除完整状态深clone；load-report bridge lifecycle借用loaded plugin id列表。
3. **PERF-MVP-544**：`host_has_capability`直接借用granted capability CStr并流式token匹配，删除每probe String/Vec分配；entry capability grant使用borrowed HashSet membership，保留首次module/manifest顺序。

六个源码/行为守卫均先观察RED再实现GREEN；scoped `rustfmt --edition 2021`与`git diff --check`通过。candidate/report顺序、duplicate/path escape诊断、capability delimiter/exact-boundary、callback锁外执行、schema mismatch、failed restore/rollback和bridge lifecycle报告语义不变。

## 动态验收

需要覆盖plugins/manifests/interfaces/methods **1/100/10k**，same-plugin callback **1/2/16/64 threads × 1M**，state/command/output **0/1KiB/1MiB/256MiB**，stable/toggle/reload/failed rollback。记录filesystem enumerate/stat/read/parse、root lock wait/hold、report/projection builds、candidate/manifest/binding/String/state clone bytes、CString alloc、callback mutex/atomic RMW、payload copy/RSS、DLL/entry wall、job queue age与main-thread p95。current-source Cargo、native fixture、F0/F2/F4产品trace完成前，本切片不得进入`review.md`。
