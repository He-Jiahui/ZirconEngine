---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: dynamic-scene-asset-reload-bounded-singleflight
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/scene/dynamic_scene/asset_reload
  - zircon_runtime/src/scene/dynamic_scene/spawn_task
  - zircon_runtime/src/dynamic_api/session
tests:
  - cargo test -p zircon_runtime --lib dynamic_scene_asset_reload --locked --jobs 1 -- --nocapture --test-threads=1
  - event burst, slow prepare, supersede/cancel and apply-budget fixtures
---

# Runtime04：dynamic scene asset reload有界single-flight交接

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：Runtime dynamic scene非session基础35/35逐Rust文件审查，PERF-MVP-471
- 修复责任计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 交接原因：Runtime04拥有asset event/revision/import generation；Runtime11共同提供cancel与bounded worker/apply budget。
- 生命周期键：`dynamic-scene-asset-reload-bounded-singleflight`

## 失败现象与复现证据

每frame无上限drain所有SceneAsset事件；每个event完整drain+重建pending Vec移除旧revision，形成O(E×P)。pending/latest revision无容量、TTL或asset lifecycle prune；superseded task使用DetachOnDrop继续后台运行。ready结果在同一frame、同一Level world锁内全部spawn，慢/大scene可占满主线程。

## 最低共享层根因

队列以flat Vec存每revision task，没有AssetId keyed single-flight slot、cancel token、frame drain/apply预算或target-world transaction ticket；latest revision只用于事后丢弃，不阻止旧工作占用资源。

## 架构修复验收

- per AssetId最多一个active/latest preparation，new revision原子supersede并请求cancel旧generation；无法抢占的旧任务结果不入ready且计wasted work。
- event drain、schedule与ready apply各有count/time/bytes预算和可续游标；队列发布age/depth/bytes/drop/cancel/overrun诊断。
- latest state随asset remove/catalog generation/TTL prune；pending/result bytes与RSS有硬上限。
- ready只携带Runtime08 compiled spawn transaction；main thread按budget commit，Level world锁不跨I/O/parse/compile或多个无界scene。
- events/pending/assets 1/1k/100k、slow prepare 0/10/1000ms记录pending scans/jobs/cancel/waste/queue/lock：每event不全扫pending、active≤1/asset、锁hold受预算。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- 禁止仅把pending Vec改HashMap但仍为每revision保留无界任务/结果。
- 禁止只在apply前丢stale结果而不取消/抑制旧prepare。
- 禁止在world锁内等待任务、读文件、parse或compile transaction。

## 修复结果与回传

Open state: `实现候选完成，待受管验证`; no pass is claimed.

- 已实现AssetId keyed bounded single-flight、重复supersede物理worker保留、same-revision lifecycle authority、event gap/reconciliation、count/time/bytes预算、TTL/catalog generation清理、资源stage clone字节上限、target generation CAS commit与队列诊断。
- 已补充三次快速revision同一物理worker、空日志eviction gap、resource staging byte rejection等回归测试。
- Windows受管compile receipt：ticket `0e933cadb8814993821c52e5cbe70de7`，request `runtime04-dynamic-reload-archive-r5-compile-20260801-ef7edf57baec`，source manifest `a1c42abdc37f3d636c7b66b00a88b2418a178a3a20827de83e6afc4f8079a9d8`，command `cargo +1.94.1 check -p zircon_runtime --lib --locked --jobs 1`；receipt状态为`queued`，本Session不轮询、不等待。
- 剩余实现：ready artifact到target-bound compiled transaction的后台stage/main-thread bounded commit硬边界，以及1/1k/100k与0/10/1000 ms规模矩阵；完成后执行独立二次审查并由coordinator wakeup收口。

### 2026-08-01 forward repair candidate

- Production construction now requires `ProjectAssetManager`; static queue construction, raw ready collection, and direct world/level spawn helpers are test-only. Production exposes the single bounded `tick_into_level` commit path, where Level capture occurs in the scheduled stage task rather than under the caller's world lock.
- Ready and target-stage residency share one cumulative byte cap. Event, schedule, ready, apply, and target commit all retain count/time/bytes boundaries; deferred results return through the keyed queue instead of bypassing its single-flight state.
- Regression coverage now exercises real Scene resource bursts at 1/1k/100k under one-event/one-task limits, plus three rapid revisions with 0/10/1000 ms blocked worker preparation. The final independent review of the dynamic archive/reload scope reported `0 Critical / 0 Important / 0 Minor`.
- The current source-bound focused receipt remains `78c053989d304cb6a1123954287b6bd7` for `cargo +1.94.1 test -p zircon_runtime --lib dynamic_scene --locked --jobs 1 -- --nocapture --test-threads=1`; it is a materializing receipt, not terminal test evidence. This handoff remains open until that evidence is supplied by the coordinator.
