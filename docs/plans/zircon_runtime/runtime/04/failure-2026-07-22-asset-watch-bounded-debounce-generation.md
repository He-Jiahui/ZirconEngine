---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: asset-watch-bounded-debounce-generation
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/asset/watch
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/project_asset_manager.rs
tests:
  - cargo test -p zircon_runtime --lib asset::tests::watcher --locked --jobs 1 -- --nocapture --test-threads=1
  - unique URI, continuous storm, slow callback, overflow reconciliation, rename and shutdown matrices
---

# Runtime04：asset watch有界debounce generation缺失

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：Runtime asset watch 18/18逐Rust文件性能审查，PERF-MVP-501
- 修复责任计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 交接原因：OS事件合并、source authority reconciliation和project generation提交必须共享Runtime04唯一truth，不能由notify callback或Editor建立独立缓存。
- 生命周期键：`asset-watch-bounded-debounce-generation`

## 失败现象与复现证据

notify callback写入无界channel；每收到一个event都重建`after(debounce)`，持续风暴可能永远不flush。原始pending Vec已在本轮改成按AssetUri增量折叠，内存从O(events)降到O(unique URI)，但ingress和ProjectWatcherActivation的changes/errors仍无界。`on_changes`还在watcher线程同步进入project generation/project写锁并执行scan/import、全部resource prepare与broadcast。

## 最低共享层根因

watcher没有有界generation log、max batch latency、overflow reconciliation或轻量callback/后台prepare/短commit阶段边界；同一事件在OS ingress、activation Vec和project transaction之间重复驻留。

## 架构修复验收

- ingress/pending按entry/bytes/age硬有界，按URI/revision合并可覆盖变化并保留rename/remove/failure顺序。
- debounce同时有quiet window和max batch latency；每批event/unique URI/time有预算，continuous 60s仍定期flush。
- overflow不得静默drop，发布dirty-root token并用一次可观测targeted inventory reconciliation最终收敛。
- callback只做轻量enqueue；Runtime11有界jobs准备affected closure，Runtime04短CAS/authority swap且保留last-good。
- raw events/unique URIs 1/1k/1M、callback stall 0/100/1000ms记录entries/bytes/age、coalesce/overflow/reconcile、flush、scan/import、RSS；同URI burst≤1 effective generation。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止通过增大无界channel、debounce时间或activation Vec容量掩盖风暴。
- 禁止overflow静默丢event；禁止callback同步执行全项目重活或私建worker线程。

## 修复结果与回传

Open state: `待修复`; no pass is claimed.
