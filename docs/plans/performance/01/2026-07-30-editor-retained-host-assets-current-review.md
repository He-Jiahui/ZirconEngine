---
related_code:
  - zircon_editor/src/ui/retained_host/app/assets
  - zircon_editor/src/ui/retained_host/app/backend_refresh.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/dispatch_effects/side_effects.rs
  - zircon_editor/src/ui/retained_host/app/helpers/animation_assets.rs
  - zircon_editor/src/ui/retained_host/app/helpers/model_staging.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/09-editor-asset-management.md
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
tests:
  - inline tests: 3
  - adjacent backend planner tests: 3
  - rustfmt check: blocked by refresh/events/runtime.rs external import ordering
  - scoped whitespace check: passed
  - current-source managed Windows Cargo pending
  - F0/F4 event-storm and model-import product traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor retained-host assets当前源码复核（2026-07-30）

## 范围

`zircon_editor/src/ui/retained_host/app/assets/**`当前源 **10/10** 个Rust文件、**720** 行、**3** 条`#[test]`已逐文件阅读；path+raw-content SHA-256为`4a27e84da3c3a088f7a788d1848e9249b95e6e3c56cd923ebff621cf1e970de5`。5个tracked文件的外部未提交内容只读纳入，本轮未修改Rust。

| 模块 | 文件 | 行 | 测试 | 当前边界 |
|---|---:|---:|---:|---|
| bridge/controls | 2/2 | 98 | 0 | lazy asset template bridge与固定control dispatch |
| refresh root/apply/counters | 3/3 | 166 | 0 | tick event batch、refresh plan执行与profiling counters |
| refresh events | 3/3 | 267 | 3 | 三流count/time slice、queue age与startup discard |
| snapshots/workspace | 2/2 | 189 | 0 | catalog/resources/details/preview、scene reload与模型导入 |

## 发现

- **PERF-MVP-104当前状态修正**：三流dequeue已不是“无预算”。每条stream最多256项/600us，三条目标总预算2ms，并记录drained、pending、queue-age和drain wall；3条测试锁定count/time/independent slice。这个止损应保留。
- **P0预算只覆盖dequeue，不覆盖消费**：`tick()`每帧调用`refresh_project_assets()`。任意非空batch都会调用一次owned `editor_snapshot()`只读取selected UUID；该snapshot仍完整遍历scene hierarchy、构造selected dynamic inspector reflection、构造asset activity/browser并复制多类UI状态。含asset change的batch还先同步执行全量`refresh_from_runtime_project`。Catalog/Reference plan随后可再次为details调用`editor_snapshot()`，preview路径又构建完整chrome并遍历/clone visible UUID。因此持续backlog会跨tick重复执行无时间/计数预算的全catalog与全UI投影，2ms drain counter不能代表端到端refresh p95。
- resource batch只要非空就调用`list_resources()`，全registry深clone、排序、locator String key后重建`resources_by_locator`，并无差别标记render/presentation dirty；与 **PERF-MVP-500** 共用Runtime04 compact generation，不建立host cache。
- default scene命中asset/resource event时，当前plan在上述manager refresh/resource sync之后同步调用`reload_default_scene()`；它重新`ProjectManager::open`、全项目`scan_and_import`、加载scene、创建runtime level并替换world，全程位于UI tick。已提交的project/resource generation没有被复用，回链 **PERF-MVP-075/496**、Editor10与Runtime04 targeted-import failure。
- PreviewChanged当前发布共享`Arc<EditorAssetCatalogGeneration>`，不是旧版深投影；但仍令catalog sync、完整chrome和visible preview admission在事件tick重复执行。PreviewAdmissionAvailable虽不再sync catalog，仍会重建chrome。验收应分别记录catalog generation pointer publish、chrome builds和实际preview submits，不能只用一个`sync_catalog`布尔计数。
- **PERF-MVP-555 / 产品导入绕过**：`import_model_requested`在dispatch side effect中同步调用`import_model_into_project`，没有使用已经实现single-flight/entry-byte-age预算的Editor09 `EditorAssetImportFlow`。它先在caller读取完整chrome、重新打开`ProjectManager`并可能同步复制外部GLB/OBJ；随后直接调用Runtime `import_asset`。
- Runtime `import_asset(uri)`每次在generation/project锁域clone活动project、执行全`scan_and_import`、prepare/commit全部resources。模型先调用一次；之后UI再用`gltf::import`第二次读取/解析模型，写出1个skeleton和A个clip，并对每个派生URI逐个`import_asset`；默认材质又无条件`import_asset`一次。A个animation的单次按钮操作因此至少触发 **A+3次全项目scan/resource prepare**，再执行`sync_asset_workspace`全catalog/resource refresh。该放大同时归 **PERF-MVP-496/504/555**，Runtime04必须以一次compound/batch candidate transaction完成，Editor09只提交一个可取消产品ticket。
- asset/resource事件没有真实enqueue timestamp；其queue age从“某tick drain后仍pending”才开始计时，完全在一批内排空的老事件会记录0。Editor mailbox有真实delivery age。端到端验收必须为三流统一记录oldest enqueue-to-commit age，避免预算指标假绿。
- 正向边界：空tick的三个`Vec::new()`不分配heap；planner最多扫描本tick有界batch，default scene locator每batch只parse一次；asset surface bridge保持lazy；catalog/details使用共享Arc generation。

### 2026-08-14 Runtime04 generation addendum

- `ProjectAssetManager::asset_ids_by_kind` 已不再扫描 mutable registry、调用 `list_resources()` 或在调用方排序；它读取 `ResourceManagementGeneration` 的 kind query。这一已完成止损不得被后续工作回退。
- 该 generation 目前按 `64` 个 locator-sorted shard 发布。每次 `ResourceManagementScan::next_row()` 都在全部 shard 的当前候选中选择最小 locator；完整 kind list 仍执行约 `64 * N` 次候选检查。`ProjectAssetManager` 随后逐 id 加载资产并重建 model/mesh/scene/material/shader rich records，scene entity 也从新建的 scene records 再投影。因此 stable list/summary 仍不是 generation-owned O(1) 读取，不能从“已无调用方 sort”推导为 warm work 为零。
- Unreal `FAssetRegistryState` 把 class/path/package 等 lookup index 与 registry state 一起维护，并直接枚举对应索引；其 tag index 还明确记录 memory-for-query-latency trade-off。Zircon 的正确演进方向是由 Runtime04 在同一 immutable generation 发布紧凑的 query-specific ordered rows/summary 与 selected-detail handle，Editor 只消费 `(generation, query, page)`，而不是在 Editor 建立第二套缓存或为每页重新扫描 shard。
- 在任何索引结构改动前，基线必须记录 `scan` 调用、每页/全量输出的 shard-candidate 检查数、匹配 row 数、rich record/scene-entity projection 数、row/detail clone bytes、query wall time 与 generation sequence。矩阵保持 assets/scenes/entities `1/1K/100K`、visible page `0/50/1K`、idle/60Hz stable/1% change；只有稳定 generation 的这些数据确认热路径后，才实施并复测前后 p50/p95/RSS。GPU timing 与系统功耗不由该 CPU-side counter 推导，仍需独立校准来源。
- 当前源码已在 `ResourceManagementScan` 的 profiling feature 路径按 scan 生命周期汇总并一次批量发射 `resource_management.scan.instances`、`matching_rows`、`rows_emitted`、`shard_candidate_checks` 和 `filtered_rows_skipped`：五项归属同一帧与时间戳，不会作为五次 recorder 锁争用。基线报告仅在同一场景完整保留五项时标记 asset-management 为 `measured`，缺项必须为 `partial`。它不在默认构建求值，也不按每行写入 timeline；后续基线必须把这些 counter 与同一 capture 的 `project_asset_manager` span、generation sequence 和产品场景关联。rich record/scene entity 的输出行数、clone bytes 与 page 指标仍待管理 projection owner 补齐，不能由现有 scan counter 推断。
- `ResourceManagementGeneration::page()` 是独立的 64-shard merge，不复用 scan cursor；profiling feature 路径现在在该页返回前一次批量发射 `resource_management.page.instances`、`matching_rows`、`candidate_rows`（含 offset 所消耗的有序候选）、`rows_returned`、`shard_candidate_checks` 与 `filtered_rows_skipped`。报告以独立 `asset_management_page` 覆盖率呈现：没有产品 page 场景时为 `not_emitted`，同一场景只保留部分 page counters 才为 `partial`，不借 scan 指标推断 page 成本。该切片仅建立测量基线，不改变合并算法、默认构建或 runtime/editor 缓存所有权。

## 参考与目标

- Bevy `dev/bevy/crates/bevy_asset/src/server/mod.rs:324-325,573-603`对同path复用已存在handle，不重复创建load task，并把阻塞load交给`IoTaskPool`。Zircon产品模型导入应提交一个generation-keyed batch ticket，observer、derived outputs与资源发布共享结果。
- Godot `dev/godot/editor/file_system/editor_file_system.cpp:1083-1140,1702-1721`合并正在进行的scan请求，并将可异步scan置于低优先级线程。Zircon仍需保留自身Runtime04原子candidate/last-good语义，但不得让UI tick逐URI重跑full scan。

Editor09从asset/resource/editor generation delta直接形成一次窄refresh plan；selected UUID、visible UUID和resource page使用generation-owned窄accessor，不构建完整EditorData/Chrome snapshot。Runtime04为default scene与model+derived outputs提供一次prepared transaction和immutable artifact ticket；Runtime11承担stage copy、glTF parse/derive/write/import与scene load的有界、可取消、single-flight jobs。主线程仅提交和短commit，不等待I/O或其他submit owner。

## 动态验收

按assets/resources `1/1K/100K`、scene entities `1/10K/1M`、events/stream `0/1/256/10K`、visible rows `0/50/1K`、model bytes `4KiB/256MiB/1GiB`、animations `0/1/100/1K`运行idle、single/burst/continuous backlog、cold/warm/1% change、cancel/supersede/failure。记录dequeue/apply/commit wall、oldest enqueue age、full editor/chrome snapshots、catalog/resource builds、registry/meta/artifact reads、ProjectManager open/scan、glTF parse、derived writes、import calls、queue entries/bytes/age、UI blocked、jobs、RSS和F0/F4 p95。

验收要求：idle work/allocation=0；每tick端到端asset refresh有硬预算并跨tick保留typed delta；窄selected/visible查询的完整snapshot=0；同generation full catalog/resource build不超过1且warm为0；default scene reload不reopen/full scan；A个animation的模型导入只提交1个Editor09 ticket、Runtime04 scan/transaction不超过1、glTF parse不超过1，UI filesystem/parse/import wall=0。managed Cargo、规模counter、independent review与F0/F4产品trace完成前保留在`pending.md`，不进入`review.md`。
