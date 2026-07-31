---
related_code:
  - zircon_editor/src/core/commandlet
  - zircon_editor/src/core/editor_plugin_sdk
  - zircon_editor/src/core/export
  - zircon_editor/src/core/script_build
  - zircon_editor/src/core/sync
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
  - docs/plans/zircon_editor/editor/13-script-compilation.md
  - docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
reference_sources:
  - dev/bevy/crates/bevy_asset/src/io/file/file_watcher.rs
  - dev/godot/editor/file_system/editor_file_system.cpp
  - dev/godot/editor/export/editor_export_platform.cpp
  - dev/UnrealEngine/Engine/Source/Developer/HotReload/Private/HotReload.cpp
tests:
  - standalone rustc script_build tests 10/10
  - tools.tests.test_editor02_world_sync_watch_map_contract 7/7
  - tools.tests.test_editor13_script_build_orchestrator_contract 5/5
  - tools.tests.test_editor15_export_generation_inventory_contract 9/9
  - current-source Windows Cargo and F4/export product traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor core commandlet/SDK/export/script/sync逐文件性能静态审查（2026-07-22）

## 范围与覆盖

已逐文件阅读commandlet生产2/2、editor_plugin_sdk 3/3、export生产8/8、script_build生产3/3、sync生产2/2，共 **18/18生产文件**；另完整阅读script_build与sync tests 2个，export tests仅阅读本轮相关区段。`zircon_editor/src/core`累计 **234/257**，至此生产文件全部静态覆盖，剩余23个tests留在`pending.md`。受管Cargo仍由其他Session占用；本轮运行standalone script Rust tests 10/10和三组Python静态合同21/21，没有产品/RenderDoc证据。

## 已确认的性能形状

- ScriptBuild滑动debounce原把每个unique PathBuf存入BTreeSet，持续watch storm既推迟deadline又无限增长。此次在第21条唯一path立即切换full-rebuild sentinel并清空set，常驻≤20 paths；snapshot的last failure改`Arc`，轮询不再clone诊断String。显式Command/Play FIFO仍无上限且持续watch仍可饥饿，登记PERF-MVP-557。
- ExportGenerationInventory已有generation内重叠去重、persistent strong file identity和tool probe cache，但cache miss原整块`fs::read`后hash。此次使用64KiB chunk流式更新同一BLAKE3 framing，完整文件owner归零。stable generation仍全树walk/stat/canonicalize，Drop还全量clone/pretty encode/sync cache，更新PERF-MVP-071与Editor15 failure。
- Export pipeline prepare/execute失败已经拥有partial report，过去构造error又深clone全部stage inputs/outputs/diagnostics；此次显式match错误并直接move report。CompileHost完整日志已流式落盘且内存tail 64KiB，但仍每次export手建2个blocking reader thread，统一线程/queue owner继续归Editor15/14/Runtime11既有计划。
- WorldWatchMap只遍历dirty tokens且runtime SubscriptionTable已direct-index；同view多个token过去每次clone ViewInstanceId再由BTreeMap合并。本轮新增crate-private borrowed mark，只有首次view insert拥有ID。每batch三套BTreeSet duplicate/unknown验证与transport Vec无预算继续归PERF-MVP-468。
- Commandlet只在显式headless migration运行，主要成本在Runtime04 migration 511/512；SDK files为静态descriptor/lifecycle DTO，没有per-frame loop、I/O或thread owner，不单独立项。

## 参考引擎核对

- Bevy file watcher直接采用full debouncer集中事件批次；Zircon除debounce外必须有first-event max latency与有界generation，不能让持续事件无限延迟build。
- Unreal HotReload把module compile process、ticker与directory watcher分离；Zircon应让watch只合并source generation，compile由EditorJobSystem typed ticket执行，Play只挂latest resume intent。
- Godot export `FileExportCache`持久化文件mtime/MD5/path；Zircon现有强file identity+content digest更严格，但cache持久化和stable directory inventory也应显式调度，不在Drop同步完成。

## 直接止损与待验收

四组RED→GREEN：watch path storage常数化+outcome共享、64KiB streaming hash、failure report move、borrowed view dirty merge。standalone script tests **10/10**，Editor02/13/15静态合同 **21/21**，rustfmt/source guards/scoped diff check通过；`pipeline.rs`格式写入曾遇到外部mapped-file锁，但当前文本格式与diff check通过，未覆盖外部修改。

动态门：paths/requests/tokens/files **1/1k/100k**，watch/output storm **1M**，file/log **1MiB/1GiB**，记录queue/path/batch bytes+age、build generations、walk/stat/read/hash、scratch/RSS、view clone、main/worker p95与thread数。要求watch max latency和queues硬有界、hash scratch≤64KiB、stable export walk/stat接近0、Drop I/O=0、normal invalidation不建三套dedup tree；F4 script edit/Play/world mutation与export cold/warm/resume/cancel产品trace、current-source Cargo完成前不进入`review.md`。
