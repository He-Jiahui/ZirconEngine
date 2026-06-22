---
related_code:
  - zircon_runtime/src/asset/facade/handle.rs
  - zircon_runtime/src/asset/facade/assets.rs
  - zircon_runtime/src/asset/facade/load_state.rs
  - zircon_runtime/src/asset/facade/manager.rs
  - zircon_runtime/src/asset/facade/event.rs
  - zircon_runtime/src/asset/pipeline/worker_pool.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime.rs
  - zircon_runtime/src/asset/pipeline/manager/resource_sync/register_project_resource.rs
  - zircon_runtime/src/asset/watch/asset_watcher.rs
  - zircon_runtime/src/asset/watch/watch_loop.rs
  - zircon_runtime/src/asset/artifact/cache_payload.rs
  - zircon_runtime/src/asset/artifact/cache_payload/scene.rs
  - zircon_runtime/src/asset/assets/texture/descriptor.rs
  - zircon_runtime/src/asset/assets/texture/texture_asset.rs
  - zircon_runtime/src/asset/importer
  - zircon_runtime/src/asset/importer/ingest/import_texture.rs
  - zircon_runtime/src/asset/management.rs
  - zircon_runtime/src/core/resource
  - zircon_plugins/texture_importer/runtime/src/importers.rs
  - zircon_runtime_interface/src/resource/resource_record.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_pipeline.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_anchor_inventory.py
  - dev/bevy/crates/bevy_asset/src
  - dev/Fyrox/fyrox-resource/src
plan_sources:
  - docs/plans/zircon_runtime/runtime/index.md
  - .codex/plans/Bevy-Style Asset Stack Completion Plan.md
  - .codex/plans/资产 .zmeta 与 Shader Material 资产化计划.md
status: in_progress
last_refined: 2026-06-22
---

# 04 资产管线对齐

## 现状与证据（2026-06-12 重核）

旧文低估了已落地程度，三处矫正：

- **句柄已强类型（矫正，但语义有缺口）**：`asset/facade/handle.rs:12-52` 的 `Handle<TAsset: Asset>` 是 `Copy` 的类型化 ID 包装（内含 `ResourceId` + `PhantomData`，经 `TAsset::Marker` 携带 `ResourceKind`，可降级 `UntypedResourceHandle`）。**与 `bevy_asset/handle.rs` 的关键差异：无引用计数、无 strong/weak 之分**——句柄不管资产存活，存活由 `core::resource` 的记录与显式卸载管理。这是"裁决保留差异还是列债"的核心条目，不是"句柄是否强类型"。
- **加载状态已显式（矫正）**：`asset/facade/load_state.rs` 已有 `AssetLoadState { NotLoaded, Loading, Loaded, Failed, Reloading }`（:7-13）+ `DependencyLoadState`（:69）+ `RecursiveDependencyLoadState`（:97）三级，Bevy 同形。状态是投影而非存储：`AssetLoadState::from_resource(record, runtime_state, has_payload)`（:32-64）从 `core::resource::{ResourceState（Pending/Ready/Error/Reloading）, RuntimeResourceState}` 映射——**真正的状态机在 `core/resource` 层**，转移合法性测试应打在那里。
- **事件族已成型（矫正）**：`asset/facade/event.rs` 已有 `AssetEventKind { Added, Modified, Removed, Renamed, ReloadFailed }`（:42-48）与带 `revision: u64` 的类型化 `AssetEvent<TAsset>`（:52-）；`AssetEventReceiver` 自带 shutdown 通道（:14-17），由 `core::resource::ResourceEvent` 桥接。失败事件（ReloadFailed）已存在。
- **worker pool 原始缺口基线（已由 M2/M11-M2.4 收束）**：2026-06-12 初始核验时 `asset/pipeline/worker_pool.rs:11-73` 仍为 `AssetWorkerPool::new(worker_count)`（调用方注入线程数，`.max(1)`），request/completion 均 `crossbeam_channel::unbounded`（:20-21，**无背压**），`request()` 直发**无去重**；失败传播已有（`CpuAssetPayload::Failure { request, message }` :79-89）；Drop 关闭发送端并 join（:65-73）。当前状态以本文件“状态与产出记录”M2.1-M2.3、Runtime 11 M2.4 与 `docs/zircon_runtime/asset/worker_pool.md` 为准。
- **watcher 去抖已存在（矫正）**：`asset/watch/watch_loop.rs:11` `const WATCH_DEBOUNCE: Duration = Duration::from_millis(120)`。缺的是去抖行为测试（保存风暴 N 写 1 reload）与监视失败路径（目录删除、权限）测试。
- 参考锚点（每点一行）：`bevy_asset` 分层 loader/handle/server/processor/meta — `dev/bevy/crates/bevy_asset/src/{loader.rs,handle.rs,server/,processor/,meta.rs}`；Fyrox 状态机/事件 — `dev/Fyrox/fyrox-resource/src/{manager.rs,loader.rs,state.rs,event.rs}`。
- 既有计划承接：格式与 meta 层归 `.codex/plans/Bevy-Style Asset Stack Completion Plan.md` 与 `.zmeta` 计划；本计划只做架构对齐与缺口收束，不重复其条目。

补充参考锚点（2026-06-13 实测核验，实现型切片动工前先读——index 公约 §7.9）：

- Godot 线程化资源加载（请求去重/进度查询/缓存模式，M2 去重与背压的第二实现对照）— `dev/godot/core/io/resource_loader.{h,cpp}`；import→artifact 边界 — `resource_importer.{h,cpp}`；稳定资源 ID — `resource_uid.{h,cpp}`
- Piccolo 轻量资产管理（单引擎全栈最小参照）— `dev/Piccolo/engine/source/runtime/resource`

## 目标

1. 产出 `zircon_runtime::asset` ↔ `bevy_asset`/`fyrox-resource` 逐项差距表，每项裁决"有意取舍 / 债"（重点：句柄无引用计数语义）。
2. 状态机收口：`core/resource` 转移表测试完备（每个非法转移有负例）+ `Failed` 终态可查询原因。
3. worker pool 的并发、去重、背压策略显式化、可测试，并接诊断计数。
4. watcher 去抖与失败路径有行为测试，重载经统一事件级联。

## 非目标

- 不改资产格式与 `.zmeta` 设计（归既有计划）；不动 importer 的具体格式解析。
- 渲染资源上载（GPU readiness）归 render 计划 01-08 与 graphics owner。
- 不引入新依赖（含 benchmark 依赖）；背压如需 bounded channel 用既有 crossbeam-channel 能力。

### 全局硬约束（继承总计划 §4，违反即返工）

- 不新增 crate；硬切换不留兼容层；generated 产物只许 leaf DTO/table；非网络语义 server 命名是 blocker（注意：bevy 的 "asset server" 词汇在本仓不得照搬命名，用 manager/facade 既有词）。

## 执行前检查清单

1. 活动会话对齐：`asset/**` 在 `20260604-1232` 会话 touched_modules 内（importer registry、serialization_guard 等）；开工前重读该会话最新状态。
2. worktree 脏文件检查：`git status --porcelain -- zircon_runtime/src/asset/ zircon_runtime/src/core/resource/`。
3. 事实重核：
   - `grep -n "unbounded\|bounded" zircon_runtime/src/asset/pipeline/worker_pool.rs`（记录当前有界/无界队列形态；原始"无背压"假设已由 M2.1 收束）
   - `grep -n "WATCH_DEBOUNCE" zircon_runtime/src/asset/watch/watch_loop.rs`
   - `grep -n "pub enum" zircon_runtime/src/asset/facade/load_state.rs zircon_runtime/src/asset/facade/event.rs`
   - `AssetWorkerPool::new` 调用方：已核验 2 处（construction.rs:42 生产、tests/pipeline/worker_pool.rs:6 测试），开工时确认无新增
4. 与 `.zmeta` 计划口径核对：差距表中 meta/processor 行的裁决回写该计划，不另开口径。
5. 基线记录：`cargo test -p zircon_runtime --lib asset --locked` 通过数记入状态节。

## 里程碑

### M0 对照差距表

#### 切片 0.1 五件对照表（预填已知行，执行时补全）

- 目标文件：`docs/zircon_runtime/asset/facade.md`（已存在，扩展架构节；执行时核验：`ls docs/zircon_runtime/asset/`）。
- 改动形态：纯文档。按 bevy_asset 五件 + fyrox state/event 逐项对照，已核实行预填：

  | bevy/fyrox 锚点 | 本仓对应物 | 已知语义差异 | Runtime 04 裁决 |
  |---|---|---|---|
  | `handle.rs`（Arc 强弱句柄） | `asset/facade/handle.rs` `Handle<TAsset>` + `core::resource::ResourceHandle` | 无引用计数/strong-weak；Copy ID 包装，payload 驻留由 `ResourceLease<T>` 与 resource record 控制 | 保留当前差异；M1.1 锁定悬空 handle 查询返回 `NotLoaded`，不得 panic 或暗示驻留所有权 |
  | `server/`（加载入口与状态查询） | `asset/facade/` + `pipeline/manager/service_contracts/asset_manager_contract.rs` | 命名为 manager/facade（server 命名禁用）；查询面已分布在 `ProjectAssetManager`、`Assets<TAsset>` 与 `AssetManager` service trait | 保留 manager/facade/service 查询面，不引入 asset server 词汇；`runtime_04_asset_facade_query_surface_stays_manager_owned_and_server_free` 锁定当前盘点 |
  | `loader.rs` | `asset/importer/` + format-specific `asset/load/*` helpers | importer 选择、诊断-only backend 与原始 decode helper 分离；部分 loader 产出多 entry 或诊断记录 | 保留 importer/load split；具体格式解析继续由 importer/load owner 负责，Runtime 04 只收紧 facade/state/worker/watcher 行为 |
  | `processor/`（import→artifact 缓存） | `asset/importer/ingest/` + `asset/artifact` + `asset/project` + `.zmeta` entries | artifact/processor 语义已绑定 `.zmeta` per-entry UUID、dependency locator、package root 与 shader/material assetization | 不重开第二套 processor 设计；Runtime 04 记录边界，schema/processor 演进交给 `.zmeta` 与 shader/material assetization 计划 |
  | `meta.rs` | `.zmeta` 计划地盘 | — | 引用既有计划，不重复 |
  | fyrox `state.rs` | `core/resource` 的 `ResourceState/RuntimeResourceState` + `load_state.rs` 投影 | 状态机在 resource 层、asset 层只读投影 | 保留分层，补转移表测试（M1） |
  | fyrox `event.rs` | `asset/facade/event.rs`（5 类事件 + revision） | 已对齐 | 保留 |

- 调用方迁移：无。
- 验收：表中每行有"对应物路径 + 语义差异 + 裁决"三列齐备，不留下 unresolved 对照占位。
- DoD：差距表落 `facade.md`，meta/processor 行的裁决已回写 `.zmeta` 计划。

#### M0 测试阶段（milestone-first）

- 纯审计：`git status --porcelain` 仅 docs 变更。

### M1 句柄与状态机语义定稿

#### 切片 1.1 句柄引用计数语义裁决

- 目标文件：`asset/facade/handle.rs`（仅当裁决"列债并修"时改）；`docs/zircon_runtime/asset/facade.md`（裁决记录必写）。
- 改动形态：二选一并记录理由——(a) **保留差异**：显式声明"句柄即 ID、存活归 resource 记录管理"为本引擎语义，文档化悬挂句柄行为（查询 `NotLoaded`/`Failed` 而非 panic）；(b) **列债**：引入强弱句柄（对照 `bevy_asset/handle.rs` 的 `Arc<StrongHandle>` 形态），作为独立后续里程碑排期，本计划不实现。倾向 (a)：cdylib 边界与 `Copy` 序列化友好是既有架构约束。
- 调用方迁移：(a) 无；(b) 也无（裁决期）。
- 验收：`dangling_handle_queries_report_not_loaded_instead_of_panicking`（归属 `zircon_runtime/src/asset/tests/`，挂既有测试树；断言对未注册 `ResourceId` 的 load_state 查询返回 `NotLoaded`）。
- DoD：裁决判词落文档 + 悬挂句柄行为测试绿。

#### 切片 1.2 状态转移表测试完备化

- 目标文件：`core/resource` 的状态机测试位（执行时核验确切文件：Grep `ResourceState`，path `zircon_runtime/src/core/resource`，找转移实施点与既有测试）；`asset/facade/load_state.rs` 投影测试。
- 改动形态：纯测试 + 必要的转移校验收紧。枚举合法转移（候选表，按实仓核验定稿）：`Pending→Ready/Error`、`Ready→Reloading`、`Reloading→Ready/Error`；非法转移（如 `Error→Ready` 不经 Reloading）每条一个负例。`Failed` 终态原因可查询：核验 `ResourceRecord` 是否存失败 message（执行时核验：Grep `Error`，path `zircon_runtime/src/core/resource`）；缺则补字段（签名草案：`ResourceRecord::failure_reason: Option<String>`，执行时定稿）。
- 调用方迁移：补字段时的构造点枚举：Grep `ResourceRecord`，path `zircon_runtime/src`。
- 验收（测试名草案）：
  - `resource_state_rejects_error_to_ready_without_reloading`
  - `asset_load_state_projection_matches_resource_record_matrix`（投影函数 :32-64 的全分支矩阵）
  - `failed_asset_exposes_failure_reason_through_facade`
- DoD：转移矩阵全分支有测试；失败原因经 facade 可读。

#### M1 测试阶段（milestone-first）

- 切片期：`cargo check -p zircon_runtime --lib --locked`
- 里程碑末：`cargo test -p zircon_runtime --lib load_state --locked -- --nocapture`；`cargo test -p zircon_runtime --lib resource --locked`；`cargo test -p zircon_runtime --lib asset:: --locked`
- 验收证据：转移表测试族 + 失败可观察性测试；文档 `docs/zircon_runtime/asset/management.md` 同步。

### M2 worker pool 背压与去重定稿

#### 切片 2.1 并发参数与背压策略

- 目标文件：`asset/pipeline/worker_pool.rs`；并发参数 owner 收束进 config（与子计划 02 的 `config_store` 归属定稿衔接）。
- 改动形态（签名草案，执行时定稿）：
  - `AssetWorkerPool::new(options: AssetWorkerPoolOptions)`，`AssetWorkerPoolOptions { worker_count: usize, queue_depth: Option<usize> }`——`queue_depth: Some(n)` 时 request 通道改 `crossbeam_channel::bounded(n)`，满时行为定稿（候选：`request()` 返回 `ZirconError::Backpressure` 显式错误，不静默阻塞主线程）；`None` 保持 unbounded 并在文档声明理由。
  - 旧 `new(worker_count)` 签名同切片删除（硬切换），调用方改 options。
- 调用方迁移（2026-06-12 二次细化实测，全列 2 处）：生产装配 `asset/pipeline/manager/project_asset_manager/construction.rs:42`（`AssetWorkerPool::new(self.default_worker_count)`——线程数 owner 即 `project_asset_manager` 构造器的 `default_worker_count` 字段，options 化时由此注入 config）；测试 `asset/tests/pipeline/worker_pool.rs:6`（`new(1)`）。
- 验收：`worker_pool_bounded_queue_rejects_overflow_with_explicit_error`、`worker_pool_unbounded_mode_is_explicit_opt_in`（归属 worker_pool 同级测试位）。
- DoD：背压行为可测、参数经 config 注入、无遗留旧签名。

#### 切片 2.2 请求去重

- 目标文件：`asset/pipeline/worker_pool.rs` 或其 manager 调用层（去重位置执行时定稿：池内按 `AssetRequest` 键去重 vs manager 层按 locator 合并——倾向 manager 层，池保持纯执行器）。
- 改动形态：同资产并发请求合并为单次解码 + 多路完成通知；键为 `AssetRequest` 的 locator/kind（形状核验命令修正：`AssetRequest`/`CpuAssetPayload` 定义不在 `asset/types.rs` 单文件——执行时用 Grep `pub enum AssetRequest`，path `zircon_runtime/src/asset` 定位）。
- 调用方迁移：仅去重层内部；`request_sender()` 旁路若绕过去重，同切片裁决。2026-06-20 已裁决为收回公开 channel clone，`AssetWorkerPool::request(...)` 是唯一公开请求入口。
- 验收：`concurrent_requests_for_same_asset_decode_once_and_notify_all`。
- DoD：去重测试绿；`request_sender` 旁路有判词。

#### 切片 2.3 诊断计数

- 目标文件：`asset/pipeline/worker_pool.rs`（计数登记），走 `core::diagnostics` 既有通道。
- 改动形态：计数项（草案）：`asset.worker.in_flight`、`asset.worker.completed`、`asset.worker.failed`、`asset.worker.queue_peak`。
- 调用方迁移：无公共面变化。
- 验收：`worker_pool_diagnostics_track_in_flight_and_failure_counts`。
- DoD：四计数可经诊断读取且测试断言。

#### M2 测试阶段（milestone-first）

- `cargo test -p zircon_runtime --lib worker_pool --locked -- --nocapture`（去重合并、背压行为、失败传播三组）
- `cargo test -p zircon_runtime --lib asset:: --locked`（全族无回归）
- 验收证据：计数断言测试 + 并发加载冒烟。

### M3 watcher 与热重载收尾

#### 切片 3.1 去抖行为测试与失败路径

- 目标文件：`asset/watch/watch_loop.rs`、`asset/watch/asset_watcher.rs`（失败路径若缺处理则补）；测试挂 `asset/tests/` 既有树。
- 改动形态：`WATCH_DEBOUNCE`（120ms，watch_loop.rs:11）从魔法常量升级为可注入参数（测试需要短窗）；监视失败（目录删除、权限拒绝）转化为可观察事件/日志而非静默（执行时核验现状：Read `asset_watcher.rs` 错误分支）。
- 调用方迁移：watch_loop 构造点（执行时枚举：Grep `watch_loop|AssetWatcher::new`，path `zircon_runtime/src/asset`）。
- 验收：
  - `rapid_successive_writes_within_debounce_window_emit_single_reload`（N 次快速写入 → 1 次重载）
  - `watcher_failure_on_removed_directory_surfaces_observable_error`
- DoD：两测试绿；去抖参数有注入口。
- 注意：保持 `WATCH_DEBOUNCE` 默认值语义不变（120ms），本切片不调参，调参属性能议题归 07。

#### 切片 3.2 重载事件与状态机对齐

- 目标文件：纯测试（`asset/tests/`）+ `docs/zircon_runtime/asset/importer.md`、`facade.md` 状态刷新。
- 改动形态：断言重载链路 `Loaded→Reloading→Loaded/Failed` 经 M1 转移表合法；下游（render 资源、场景实例）经 `AssetEvent::Modified`/`ReloadFailed`（event.rs:42-48）级联而非轮询（轮询点盘点：Grep `try_recv|poll`，path `zircon_runtime/src/asset`，违规列清单归对应 owner）。
- 调用方迁移：无（审计 + 测试）。
- 验收：`hot_reload_transitions_through_reloading_state_and_emits_modified_event`、`reload_failure_emits_reload_failed_event_and_lands_failed_state`。
- DoD：两测试绿；轮询违规清单为空或移交 owner 计划。

#### M3 测试阶段（milestone-first）

- `cargo test -p zircon_runtime --lib watch --locked -- --nocapture`
- `cargo test -p zircon_runtime --lib asset:: --locked`
- 验收证据：去抖测试、失败路径测试；文档刷新完成。

`runtime_04_asset_pipeline_cargo_gate_stays_visible_until_asset_validation` 保持 Runtime 04 为 `in_progress`，直到 broader `asset::` / `worker_pool` Cargo filters 在干净编译窗口中补齐真实通过证据；已有 `artifact_store_roundtrips_scene_assets_with` 4/4 与 `watcher` 7/7 只证明 M3 的聚焦修复，不替代 M1/M2 broader gate。

`asset_pipeline_source_inventory.py` 结构审计清单 owner 现在承接 Runtime 04 source/guard file list、source/guard expected counts、worker diagnostic count、artifact-store roundtrip count 与 watcher acceptance count；`asset_pipeline_anchor_inventory.py` 承接 handle/load-state、resource reload、worker pool、worker diagnostics、watcher、artifact cache、Runtime 04 guard/behavior/doc/Cargo gate anchors。`asset_pipeline_boundary.py` 保留审计读取、缺失锚点计算与风险聚合，当前为 328 行；`asset_pipeline_markdown.py` 承接 Markdown 渲染，当前为 117 行。当前静态事实为 `expected_source_file_count = 22`、`expected_guard_file_count = 11`、`worker_diagnostic_count = 7`、`expected_worker_diagnostic_count = 7`、`artifact_store_roundtrip_count = 4`、`expected_artifact_store_roundtrip_count = 4`、`watcher_acceptance_reference_count = 1`、`expected_watcher_acceptance_count = 7`、`artifact_acceptance_reference_count = 3`、`test_anchor_count = 24`、`behavior_test_anchor_count = 20`、`missing_behavior_test_anchors = []`、`missing_doc_anchors = []`、`missing_cargo_gate_anchors = []`、`retired_worker_new_references = []`、`retired_worker_request_sender_references = []`、`old_watch_debounce_references = []`、`mirror_docs_guard_present = true` 与 `risks = []`。`runtime_04_asset_pipeline_mirror_docs_match_structure_audit_counts` 锁定 Runtime 04、runtime index、asset facade/worker/watcher/artifact/core-resource docs、M0 review 与 runtime-interface convergence 的镜像计数。

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据（命令输出 / 文件 / 测试名） |
|---|---|---|---|---|
| M0 | 0.1 差距表 | 文档完成；status gate 受既有 asset 脏树限制 | 2026-06-12 | `docs/zircon_runtime/asset/facade.md` 新增 Reference Asset Stack Gap Table；`.codex/plans/资产 .zmeta 与 Shader Material 资产化计划.md` 回写 processor/meta owner 口径；`git status --porcelain -- zircon_runtime/src/asset/ zircon_runtime/src/core/resource/ ...` 显示大量既有 asset 改动，无法证明 M0 纯 docs 工作区 |
| M0 | 0.1 差距表占位收束守卫 | static_passed_cargo_pending | 2026-06-14 | Runtime 04 计划表已同步 `docs/zircon_runtime/asset/facade.md` 的 handle/loader/processor 最终裁决；`runtime_absorption::asset_surface::runtime_04_asset_facade_query_surface_stays_manager_owned_and_server_free` 现在同时拒绝 pending-query、pending-comparison、pending-decision 占位回流。Cargo/rustc 独立验证仍待 active lanes 清空。 |
| M1 | 1.1 句柄语义裁决 | code_complete_static_passed，Cargo 待空闲窗口 | 2026-06-12 | 裁决为保留 Zircon `Copy` typed ID handle 差异：payload residency 继续归 `core::resource` 记录与 `ResourceLease<T>`；新增 `zircon_runtime/src/asset/tests/facade/handle_lifecycle.rs::dangling_handle_queries_report_not_loaded_instead_of_panicking`，并从超长 `facade.rs` 以子模块挂接；`rustfmt --edition 2021 --check` 通过；conflict-marker / `git diff --check` 通过 |
| M1 | 1.2 转移表测试 | code_complete_static_passed，Cargo 待空闲窗口 | 2026-06-12 | `core::resource` 收紧状态边界：`Error -> Ready` 必须先 `start_reload`，`Ready -> Error` 不能绕过 reload；项目资源同步在失败后成功导入时显式走 `Error -> Reloading -> Ready`；`ResourceRecord::failure_reason()` 复用现有 `ResourceDiagnostic` 作为失败原因来源，`Assets<T>` / `ProjectAssetManager` 暴露 `failure_reason(handle)`；新增/更新测试名：`resource_state_rejects_error_to_ready_without_reloading`、`resource_state_recovers_from_error_only_through_reloading`、`resource_state_rejects_reload_failure_without_reload_boundary`、`asset_load_state_projection_matches_resource_record_matrix`、`failed_asset_exposes_failure_reason_through_facade`；`docs/zircon_runtime/core/resource.md` 与 `docs/zircon_runtime/asset/facade.md` 已同步；`rustfmt --edition 2021 --check`、conflict-marker scan、`git diff --check` 通过，Cargo 因活动编译通道暂缓 |
| M2 | 2.1 背压策略 | code_complete_static_passed，Cargo 待空闲窗口 | 2026-06-12 | `AssetWorkerPool::new(worker_count)` 硬切为 `AssetWorkerPool::new(AssetWorkerPoolOptions)`；`queue_depth: None` 显式保留无界模式，`Some(n)` 使用 bounded channel；`request(...)` 改为 `try_send(...)`，队列满返回 `ZirconError::ChannelSend("asset request queue full: ...")`；生产装配 `ProjectAssetManager::spawn_worker_pool` 已迁移；新增 `worker_pool_unbounded_mode_is_explicit_opt_in`、`worker_pool_bounded_queue_rejects_overflow_with_explicit_error`；当日仍记录 `request_sender()` 旁路暂留，去重/诊断归 M2.2/M2.3；2026-06-20 已由 `2.2 request_sender hard-cutover` 收回该旁路；`rustfmt --edition 2021 --check`、conflict-marker scan、`git diff --check` 通过，Cargo 因活动编译通道暂缓 |
| M2 | 2.2 请求去重 | code_complete_static_passed，Cargo 待空闲窗口 | 2026-06-12 | `AssetWorkerPool` 新增 in-flight `HashMap<AssetRequest, usize>` 计数；首个请求入队，重复 key 只增加等待计数，不再入队第二次解码；worker 完成后按等待计数向 completion channel 发送同一 payload 的多份通知；当日 `request_sender()` 暂留为低层旁路并在 `docs/zircon_runtime/asset/worker_pool.md` 标注会绕过 pool-level coalescing；2026-06-20 已由 `2.2 request_sender hard-cutover` 删除该公开旁路并加入回流守卫；新增 `concurrent_requests_for_same_asset_decode_once_and_notify_all`，用 workerless test harness 证明同 key 只占一个 bounded 队列槽且完成时通知两个等待者；`rustfmt --edition 2021 --check`、conflict-marker scan、`git diff --check` 通过，Cargo 因活动编译通道暂缓 |
| M2 | 2.2 request_sender hard-cutover | asset_worker_request_sender_hard_cutover_static_passed_cargo_deferred | 2026-06-20 | `AssetWorkerPool::request_sender(...)` 已删除，`AssetWorkerPool::request(...)` 成为唯一公开请求入口，防止外部 caller 绕过 bounded `try_send(...)`、in-flight coalescing、queue-full rollback 与 diagnostics；`asset_pipeline_boundary` 改为报告 `retired_worker_request_sender_references = []` 并在源码回流时产生 risk；`asset_worker_pool_matches_runtime_04_and_11_decisions` 与 `runtime_04_asset_pipeline_mirror_docs_match_structure_audit_counts` 同步负向守卫。本轮轻量验证已补齐：Python py_compile 通过，direct `asset_pipeline_boundary_audit` 报告 source/guard 22/11、`worker_diagnostic_count = 7`、`behavior_test_anchor_count = 20`、`retired_worker_request_sender_references = []`、`missing_doc_anchors = []`、`risks = []`，standalone `asset_pipeline` 1/1 与 `asset_worker_policy` 1/1 通过；broader `asset::` / `worker_pool` Cargo filters 后续补跑。 |
| M2 | 2.3 诊断计数 | code_complete_static_passed，Cargo 待空闲窗口 | 2026-06-12 | `AssetWorkerPoolDiagnostics` 新增 `in_flight`、`completed`、`failed`、`queue_peak` 四个请求计数；Runtime 11 M2.4 追加 `thread_budget_source` / `budgeted_threads` 预算记账字段；`AssetWorkerPool::record_diagnostics(...)` 写入 `asset.worker.in_flight`、`asset.worker.completed`、`asset.worker.failed`、`asset.worker.queue_peak` 与 `asset.worker.budgeted_threads`；`request(...)` 改为先登记 in-flight 再入队，bounded enqueue 失败会回滚登记，避免快 worker 完成早于 in-flight 注册导致悬挂计数；新增/更新 `worker_pool_diagnostics_track_in_flight_and_failure_counts`、`worker_pool_options_can_derive_threads_from_runtime_io_budget`、`project_asset_manager_default_workers_use_runtime_io_budget_source` 与 bounded overflow 诊断断言；`docs/zircon_runtime/asset/worker_pool.md`、`docs/zircon_runtime/asset/facade.md` 与 `docs/zircon_runtime/core/job_system.md` 已同步；`rustfmt --edition 2021 --check`、conflict-marker scan、`git diff --check` 通过，Cargo 因活动编译通道暂缓 |
| M2 | worker pool 当前状态守卫 | code_static_pending_cargo | 2026-06-13 | 新增 `zircon_runtime/src/tests/runtime_absorption/asset_worker_policy.rs::asset_worker_pool_matches_runtime_04_and_11_decisions`，以源码/测试/文档/Runtime 04/Runtime 11/总表六点锁定 `AssetWorkerPoolOptions`、bounded 背压、in-flight 去重、worker diagnostics、`TaskPoolIo` 预算来源与 `asset.worker.budgeted_threads`；并防止旧缺口标题复活为当前状态。`rustfmt --edition 2021 --check` 通过；冲突标记、尾随空白与 scoped `git diff --check` 通过（仅 LF/CRLF 提示）；Cargo/rustc 独立测试待 active lanes 清空。 |
| M3 | 3.1 去抖与失败路径 | code_complete_static_passed，artifact cache 与 watcher 回归已通过 | 2026-06-12 | `AssetWatcherOptions` 与 `ASSET_WATCH_DEFAULT_DEBOUNCE=120ms` 已落地，`watch_loop` 支持测试注入短 debounce；notify `Err(...)` 与 watch-triggered project scan/resource sync 失败转成 `AssetWatchError { assets_root, paths, message }`；`AssetManager::subscribe_asset_watch_errors()` 与项目管理器独立 error 订阅面已落地；新增 `rapid_successive_writes_within_debounce_window_emit_single_reload`、`watcher_failure_on_removed_directory_surfaces_observable_error`；`docs/zircon_runtime/asset/watcher.md` 已新增；`rustfmt --edition 2021 --check`、conflict-marker scan、`git diff --check` 通过；`cargo test -p zircon_runtime --lib watcher --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-04-asset-0612 ...` 第一次 904s 超时，第二次进入测试阶段：4 个低层 watcher 用例通过，3 个既有 manager watcher 用例在 `open_project(...)` 前置路径失败，根因定位为 Scene `.zasset` bincode 读回遇到内部标记/自定义 serde 形状；第一次 artifact focused retry 中 camera/physics cache 用例通过、既有 mesh-reference 用例暴露 `skip_serializing_if` 二进制字段错位；`asset/artifact/cache_payload/scene.rs` 已补 scene mesh/camera/collider/joint cache wire type，`asset/artifact/cache_payload.rs` 降到 938 行，`docs/zircon_runtime/asset/artifact.md` 已记录缓存边界；`cargo test -p zircon_runtime --lib artifact_store_roundtrips_scene_assets_with --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-04-asset-0612 --message-format short --color never -- --nocapture` 通过 4/4；`cargo test -p zircon_runtime --lib watcher --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-04-asset-0612 --message-format short --color never -- --test-threads=1 --nocapture` 通过 7/7 |
| M3 | 3.2 重载事件对齐 | code_complete_static_passed，artifact cache 与 watcher 回归已通过 | 2026-06-12 | 不新增 watcher-local 热重载事件通道；`core::resource` 继续负责 `start_reload` / `register_ready` / `fail_reload` 状态机，typed facade 将 `Updated` / `ReloadFailed` 投影为 `AssetEvent::Modified` / `AssetEvent::ReloadFailed`；新增 `asset/tests/facade/hot_reload.rs::{hot_reload_transitions_through_reloading_state_and_emits_modified_event,reload_failure_emits_reload_failed_event_and_lands_failed_state}`；`docs/zircon_runtime/asset/facade.md`、`docs/zircon_runtime/asset/watcher.md` 与 `docs/zircon_runtime/asset/artifact.md` 已同步；`asset/tests/assets/artifact_store.rs` 新增/覆盖 mesh reference、camera target、physics scene cache 与 script binding JSON 回归，`cache_payload/scene.rs` 承担 scene cache wire type；`cargo test -p zircon_runtime --lib artifact_store_roundtrips_scene_assets_with --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-04-asset-0612 --message-format short --color never -- --nocapture` 通过 4/4；watcher acceptance 已通过 7/7；`rustfmt --edition 2021 --check`、conflict-marker scan、`git diff --check` 通过 |
| M4 | 验证门守卫 | cargo_validation_pending_guarded | 2026-06-13 | 新增 `runtime_absorption::plan_status::runtime_04_asset_pipeline_cargo_gate_stays_visible_until_asset_validation`，要求 Runtime 04 在 broader `asset::` / `worker_pool` Cargo filters 有真实通过证据前保持 `in_progress`，并锁定 Runtime 04 计划、总索引 P7/子计划行、asset facade / worker docs 与 M0 评审里的句柄语义、状态机、worker pool、watcher/artifact 证据和剩余 Cargo gate。Cargo 待 active lane 清空后运行 broader asset validation。 |
| 横切 | asset_pipeline_boundary 结构审计 owner | static_passed_cargo_pending | 2026-06-13 | 新增 `.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_boundary.py` 并接入 `audit_runtime_structure.py`；当前审计报告 `expected_source_file_count = 22`、`expected_guard_file_count = 11`、`worker_diagnostic_count = 7`、`expected_worker_diagnostic_count = 7`、`artifact_store_roundtrip_count = 4`、`expected_artifact_store_roundtrip_count = 4`、`watcher_acceptance_reference_count = 1`、`expected_watcher_acceptance_count = 7`、`artifact_acceptance_reference_count = 3`、`test_anchor_count = 24`、`behavior_test_anchor_count = 20`、`missing_behavior_test_anchors = []`、`missing_doc_anchors = []`、`missing_cargo_gate_anchors = []`、`retired_worker_new_references = []`、`retired_worker_request_sender_references = []`、`old_watch_debounce_references = []`、`mirror_docs_guard_present = true` 与 `risks = []`。Cargo/rustc 独立验证仍待 active lanes 清空，Runtime 04 保持 `in_progress`。 |
| 横切 | facade 查询面盘点守卫 | query_surface_static_passed_cargo_pending | 2026-06-14 | 新增 `runtime_absorption::asset_surface::runtime_04_asset_facade_query_surface_stays_manager_owned_and_server_free`，锁定加载/状态查询面分布在 `ProjectAssetManager`、typed `Assets<TAsset>` 与 `AssetManager` service trait：`load_state`、`failure_reason`、dependency/readiness state、loaded predicates、typed event subscription、importer capability/current project/status/list/watch-error service queries均保持 manager/facade/service 词汇；拒绝旧 pending-query 文本和 `AssetServer`/`asset_server` 源码回退。未改 asset 生产代码；Cargo/rustc 仍待 active lanes 清空后补跑。 |
| 横切 | Asset pipeline 镜像文档守卫 | mirror_docs_static_passed_cargo_pending | 2026-06-14 | 新增 `runtime_absorption::asset_pipeline::runtime_04_asset_pipeline_mirror_docs_match_structure_audit_counts` 并在 `runtime_absorption/mod.rs` 挂接，锁定 Runtime 04 计划、runtime index、asset facade/worker/watcher/artifact/core-resource docs、M0 review 与 runtime-interface convergence 必须同步 `asset_pipeline_boundary` 的计数字段；验证：rustfmt check、Python py_compile、direct `asset_pipeline_boundary_audit`、standalone rustc 1/1、stale old-count scan 通过；broader `asset::` / `worker_pool` Cargo filters 仍 pending。 |
| 横切 | Asset pipeline 行为测试锚审计同步 | mirror_docs_static_passed_cargo_pending | 2026-06-15 | `asset_pipeline_boundary` 与 `runtime_04_asset_pipeline_mirror_docs_match_structure_audit_counts` 现在锁定 Runtime 04 M1/M2/M3 的 20 个行为测试锚，当前 `behavior_test_anchor_count = 20`、`missing_behavior_test_anchors = []`；Runtime 04、runtime index、asset facade/worker/watcher/artifact/core-resource docs、M0 review 与 runtime-interface convergence 已同步。验证：rustfmt check、Python py_compile、direct `asset_pipeline_boundary_audit`、aggregate Runtime 04 + plan-status assertions、standalone asset_pipeline 1/1、standalone status-output 2/2；broader `asset::` / `worker_pool` Cargo filters 仍 pending。 |
| M2 | 2.3 asset worker 帧级采样诊断 | asset_worker_frame_sampler_static_passed_cargo_deferred | 2026-06-17 | `AssetWorkerPoolFrameSampler` / `AssetWorkerPoolFrameDiagnostics` 已在 worker pool owner 内落地；采样器从累计 `AssetWorkerPoolDiagnostics` 计算每帧 `completed_delta` / `failed_delta`，并写入 `asset.worker.frame_completed` 与 `asset.worker.frame_failed`，同时保留 `asset.worker.in_flight` / `asset.worker.budgeted_threads` 当前值。新增 `worker_pool_frame_sampler_records_per_frame_completion_deltas` 锁定同 key 合并完成、失败完成与下一帧零增量；`asset_pipeline_boundary` 当前同步为 `worker_diagnostic_count = 7`、`expected_worker_diagnostic_count = 7`、`test_anchor_count = 24`、`behavior_test_anchor_count = 20`、`missing_doc_anchors = []`、`risks = []`。验证：rustfmt check、Python py_compile、direct `asset_pipeline_boundary_audit`、standalone `asset_pipeline.rs` 1/1、standalone `performance_hotspots.rs` 6/6、standalone `plan_status.rs` 32/32 通过；focused `cargo test -p zircon_runtime --lib worker_pool_frame_sampler_records_per_frame_completion_deltas ...` 240s timeout no result，残留 cargo/rustc/rustdoc scan 为空，包级 worker_pool gate 后续补跑。 |
| M2 | 2.3 worker-pool manager frame sampler entry | asset_worker_manager_sampler_static_passed_cargo_deferred | 2026-06-17 | `ProjectAssetManager::spawn_worker_pool_with_frame_sampler(...)` 已成为 manager-owned worker-pool + frame-sampler 配对入口；旧 `spawn_worker_pool()` 复用该入口后只返回 pool，避免 frame owner 自行猜测采样游标初始化点。新增 `project_asset_manager_spawns_worker_pool_with_frame_sampler` 锁定显式 worker count、预算来源、首帧 `budgeted_threads` / `in_flight` / completion delta；`asset_pipeline_boundary` 当前同步为 `test_anchor_count = 24`、`behavior_test_anchor_count = 20`、`missing_behavior_test_anchors = []`、`missing_doc_anchors = []`、`risks = []`。验证先按用户要求走静态/轻量 lane；broader `asset::` / `worker_pool` Cargo filters 后续补跑。 |
| M3 | 3.2 artifact cache payload owner split | artifact_cache_payload_owner_split_static_passed_cargo_deferred | 2026-06-17 | `cache_payload.rs` 的 JSON canonical value、Mesh wire payload 与 TOML table/value conversion 已拆入 `cache_payload/{json_value,mesh,toml_value}.rs`，根文件保留 `ArtifactCacheAsset` 派发与 cache conversion entry；`asset_pipeline_boundary` 当前同步为 `expected_source_file_count = 22`、`missing_artifact_cache_anchors = []`、`artifact_store_roundtrip_count = 4`、`expected_artifact_store_roundtrip_count = 4`、`missing_doc_anchors = []` 与 `risks = []`。新增 Runtime 07 守卫 `runtime_07_artifact_cache_payload_owner_split_keeps_wire_types_folder_backed` 防止线缆类型回流根文件；broader `asset::` / `worker_pool` Cargo filters 按用户要求后续补跑。 |
| 横切 | Asset pipeline current audit recheck | asset_pipeline_current_audit_static_passed_cargo_pending | 2026-06-20 | 本轮只复核 Runtime 04 当前资产管线结构事实，生产代码未改：`asset_pipeline_boundary_audit` 报告 source files 22/22、guard/test files 11/11、worker diagnostics 7/7、artifact-store scene roundtrip guards 4/4、watcher acceptance evidence references 1（expected watcher tests 7）、artifact acceptance references 3、Runtime 04 guard anchors 24/24、behavior-test anchors 20/20、missing doc/Cargo gate anchors 均为空、`retired_worker_new_references = []`、`retired_worker_request_sender_references = []`、`old_watch_debounce_references = []`、`mirror_docs_guard_present = true`、`risks = []`。验证通过：Python py_compile、direct `asset_pipeline_boundary_audit` risks=[]、standalone `asset_pipeline.rs` 1/1、standalone `asset_worker_policy.rs` 1/1；broader `asset::` / `worker_pool` Cargo filters 仍 pending。 |
| 横切 | Asset pipeline inventory split | asset_pipeline_inventory_split_static_passed_cargo_deferred_tests_deferred | 2026-06-21 | `asset_pipeline_source_inventory.py` now owns Runtime 04 source/guard file inventory plus expected source/guard, worker diagnostic, artifact roundtrip, and watcher acceptance counts; `asset_pipeline_anchor_inventory.py` now owns handle/load-state, resource reload, worker-pool, diagnostics, watcher, artifact cache, guard, behavior, doc, and Cargo gate anchors; `asset_pipeline_boundary.py` now remains the audit reader / missing-anchor / risk layer at 328 lines, and `asset_pipeline_markdown.py` owns the Markdown layer at 117 lines. Direct audit reports source files 22/22, guard/test files 11/11, worker diagnostics 7/7, artifact-store scene roundtrip guards 4/4, watcher acceptance evidence references 1, artifact acceptance references 3, Runtime 04 guard anchors 24/24, behavior-test anchors 20/20, missing doc/Cargo anchors [], retired worker constructor/request-sender and old debounce references [], `mirror_docs_guard_present = true`, and `risks = []`. Validation: Python py_compile, direct `asset_pipeline_boundary_audit` risks=[], standalone `asset_pipeline.rs` 1/1, standalone `asset_worker_policy.rs` 1/1, standalone `plan_status.rs` 33/33; broader `asset::` / `worker_pool` Cargo gates remain deferred while external compile lanes are active. |
| 横切 | Asset pipeline Markdown renderer split | asset_pipeline_markdown_split_static_passed_cargo_deferred_tests_deferred | 2026-06-21 | `asset_pipeline_markdown.py` now owns `render_asset_pipeline_boundary_markdown`, and `audit_runtime_structure.py` imports the renderer from that Markdown owner instead of `asset_pipeline_boundary.py`; `asset_pipeline_boundary.py` now owns audit read, missing-anchor calculation, and risk aggregation at 328 lines, while the Markdown owner is 117 lines. Direct audit reports source files 22/22, guard/test files 11/11, worker diagnostics 7/7, artifact-store scene roundtrip guards 4/4, behavior-test anchors 20/20, missing doc/Cargo anchors [], retired worker constructor/request-sender and old debounce references [], `mirror_docs_guard_present = true`, and `risks = []`. Validation: Python py_compile and direct `asset_pipeline_boundary_audit`; standalone `asset_pipeline.rs` 1/1, standalone `asset_worker_policy.rs` 1/1, and standalone `plan_status.rs` 33/33; broader `asset::` / `worker_pool` Cargo gates remain deferred while external compile lanes are active. |
| 横切 | F7 asset artifact/importer typed errors | asset_artifact_importer_typed_errors_coremin_passed | 2026-06-22 | `AssetImportError::Registry` 已改为 `#[from] AssetImporterRegistryError`，删除 lossy `error.to_string()` 转换；`asset/artifact/cache_payload.rs` 与 `cache_payload/toml_value.rs` 的 artifact cache 转换入口改为 `AssetImportError`，TOML serialize/deserialize、cached datetime、UI/UI v2 document、bincode serialize/deserialize 均保留 source；`ArtifactStore` 不再把 cache conversion 包进 `Parse(String)`。新增 `asset_import_error_preserves_registry_error_source` 与 `runtime_absorption/code_review_findings.rs::review_f7_asset_artifact_errors_use_asset_import_error_sources`。验证：scoped rustfmt、F7 结构守卫 1/1、status-output 守卫 1/1、core-min `cargo check` 通过；focused `cargo test ... asset_import_error_preserves_registry_error_source` 两次在测试二进制编译阶段超时无结果，F7 残留 cargo/rustc 已清理；broader `asset::` / `worker_pool` Cargo gates 仍由 `runtime_04_asset_pipeline_cargo_gate_stays_visible_until_asset_validation` 保持 pending。 |
| 横切 | F8 texture import settings apply API | texture_import_settings_apply_api_coremin_check_passed | 2026-06-22 | `TextureAssetDescriptor::apply_import_settings(...)` 与 `TextureAsset::apply_import_settings(...)` 现在承接 texture import settings 的可失败解析/校验；runtime 内置 texture ingest 和 `zircon_plugins/texture_importer` 调用点同步迁移，不保留旧 fallible with-entry shim、alias 或转发。新增 `runtime_absorption/code_review_findings.rs::review_f8_texture_import_settings_use_fallible_apply_not_with`，状态表锚点锁定 `apply_import_settings` 与 old fallible with-entry absent。验证：scoped rustfmt、F8 结构守卫 1/1、status-output 守卫 2/2、runtime core-min `cargo check` 通过（既有 warnings）；`zircon_plugin_texture_importer_runtime` package `cargo check` 当前被既有 `graphics/runtime_provider/registration.rs` trait-object Clone drift 阻塞，未计通过。RuntimePluginDescriptor public-field convergence remains pending；本行只关闭 F8 的 texture import settings 子切片，Runtime 04 broader `asset::` / `worker_pool` Cargo gates 仍保持 pending。 |

基线数值（开工首日记录）：

- 加载状态枚举基线：5 态 ×3 级（load_state.rs；重核：`grep -c "pub enum" .../load_state.rs`）
- worker 通道形态基线：request/completion 均 unbounded（worker_pool.rs:20-21）
- `WATCH_DEBOUNCE` 基线：120ms（watch_loop.rs:11）
- `cargo test -p zircon_runtime --lib asset --locked` 通过数基线：未运行；M0 只做文档/计划对齐，且当前 `zircon_runtime/src/asset/` 已存在大量并行改动

## 风险与协调

- `asset/**` 在 `20260604-1232` 会话 touched_modules 中：每个里程碑开始前对齐其最新状态；importer registry / serialization_guard 区域脏文件先避让。
- 与 `.zmeta` 计划共享 importer/processor 路径：差距表中 meta/processor 行的裁决回写该计划，禁止双口径。
- M2 改 `AssetWorkerPool::new` 签名是硬切换：调用方一次迁完，禁止保留旧签名重载。
- "server" 命名禁令在本计划尤其敏感（bevy 词汇是 asset server）：新增类型/文档一律用 manager/facade/pool 词汇。
- worker 线程数与 rayon（ECS/资产并行）共存的 CPU 配额问题若在 07 性能取证中显形，调参归 07，本计划只保证参数可注入。
