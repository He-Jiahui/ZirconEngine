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
  - zircon_runtime/src/asset/tests/assets/artifact_store/scene_components.rs
  - zircon_runtime/src/asset/tests/assets/artifact_store/scene_script.rs
  - zircon_runtime/src/asset/assets/texture/descriptor.rs
  - zircon_runtime/src/asset/assets/texture/texture_asset.rs
  - zircon_runtime/src/asset/importer
  - zircon_runtime/src/asset/importer/ingest/import_texture.rs
  - zircon_runtime/src/asset/management.rs
  - zircon_runtime/src/core/resource
  - zircon_plugins/texture_importer/runtime/src/importers.rs
  - zircon_runtime_interface/src/resource/resource_record.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_pipeline.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_pipeline/inventory.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_pipeline/mirror_docs.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_pipeline/support.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_pipeline/split_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_surface.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_surface/registration.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_surface/namespace_surface.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_surface/facade_query.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_surface/support.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_surface/split_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_worker_policy.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_worker_policy/worker_pool.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_worker_policy/split_layout.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_pipeline/mirror_docs.rs
  - zircon_runtime/src/tests/runtime_absorption/plan_status/cargo_gates/early/runtime_04.rs
  - zircon_runtime/src/tests/runtime_absorption/code_review_findings.rs
  - tools/tests/test_runtime_asset_pipeline_audit.py
  - tools/tests/test_frameworks_02_core_error_single_source.py
  - tests/acceptance/runtime-asset-pipeline-audit-owner-sync.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_anchor_inventory.py
  - dev/bevy/crates/bevy_asset/src
  - dev/Fyrox/fyrox-resource/src
plan_sources:
  - docs/plans/zircon_runtime/runtime/index.md
  - docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
  - docs/plans/zircon_runtime/frameworks/02/2026-07-16-m1-current-source-acceptance.md
  - .codex/plans/Bevy-Style Asset Stack Completion Plan.md
  - .codex/plans/资产 .zmeta 与 Shader Material 资产化计划.md
status: in_progress
last_refined: 2026-07-31
---

# 04 资产管线对齐

2026-07-10 当前审计 owner 同步：`expected_source_file_count = 22`、`expected_guard_file_count = 17`、`test_anchor_count = 24`、`behavior_test_anchor_count = 20`、`artifact_store_roundtrips_scene_assets_with` 4/4、`missing_behavior_test_anchors = []`、`mirror_docs_guard_present = true`、`risks = []`。17 个 guard/test owner 显式包含 artifact scene children、asset facade query、worker policy、mirror docs 与 Runtime 04 Cargo-gate child；此计数取代下文保留的 11-owner 历史镜像，计划仍因 broader `asset::` / `worker_pool` Cargo filters 保持 `in_progress`。

2026-07-31 status correction: the preceding audit row is a historical snapshot,
not current acceptance. The worker-pool test parent was split into
`diagnostics.rs`, `single_flight.rs`, and `task_pool.rs`, but the source
inventory still declares 17 guard files and the boundary reader still excludes
those child bodies. The declared Python audit consequently reports six missing
behavior anchors and one failing test. The exact forward repair and subsequent
status update are owned by
[`04/failure-2026-07-31-asset-pipeline-audit-behavior-test-discovery.md`](04/failure-2026-07-31-asset-pipeline-audit-behavior-test-discovery.md);
Runtime 04 remains `in_progress` independently of Cargo validation.

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
  - `AssetWorkerPool::new(options: AssetWorkerPoolOptions)`，`AssetWorkerPoolOptions { worker_count: usize, queue_depth: Option<usize> }`——`queue_depth: Some(n)` 时 request 通道改 `crossbeam_channel::bounded(n)`，满时由 `request()` 通过 `CoreResult` 返回显式 `CoreError`，不静默阻塞主线程；`None` 保持 unbounded 并在文档声明理由。错误合同只使用 `CoreError` / `CoreResult`，不恢复旧错误枚举或兼容别名。
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

`asset_pipeline_source_inventory.py` 结构审计清单 owner 现在承接 Runtime 04 source/guard file list、source/guard expected counts、worker diagnostic count、artifact-store roundtrip count 与 watcher acceptance count；`asset_pipeline_anchor_inventory.py` 承接 handle/load-state、resource reload、worker pool、worker diagnostics、watcher、artifact cache、Runtime 04 guard/behavior/doc/Cargo gate anchors。`asset_pipeline_boundary.py` 保留审计读取、缺失锚点计算与风险聚合，当前为 328 行；`asset_pipeline_markdown.py` 承接 Markdown 渲染，当前为 117 行。当前静态事实为 `expected_source_file_count = 22`、`expected_guard_file_count = 17`、`worker_diagnostic_count = 7`、`expected_worker_diagnostic_count = 7`、`artifact_store_roundtrip_count = 4`、`expected_artifact_store_roundtrip_count = 4`、`artifact_store_roundtrips_scene_assets_with` 4/4、`watcher_acceptance_reference_count = 1`、`expected_watcher_acceptance_count = 7`、`artifact_acceptance_reference_count = 3`、`test_anchor_count = 24`、`behavior_test_anchor_count = 20`、`missing_behavior_test_anchors = []`、`missing_doc_anchors = []`、`missing_cargo_gate_anchors = []`、`retired_worker_new_references = []`、`retired_worker_request_sender_references = []`、`old_watch_debounce_references = []`、`mirror_docs_guard_present = true` 与 `risks = []`。`runtime_04_asset_pipeline_mirror_docs_match_structure_audit_counts` 锁定 Runtime 04、runtime index、asset facade/worker/watcher/artifact/core-resource docs、M0 review 与 runtime-interface convergence 的镜像计数。

The preceding static-fact row is the 2026-07-10 snapshot only. Current status
is the open child-test-discovery failure above: the six worker-pool behavior
anchors are present in Rust but absent from the audit's source sets, so its
`missing_behavior_test_anchors` and `risks` are non-empty until the explicit
20-file guard inventory repair has landed and the Python gate has run green.

## 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

- 迁入记录：[`04/2026-07-09-asset-pipeline-alignment-output-records.md`](04/2026-07-09-asset-pipeline-alignment-output-records.md)
- fixed 已修复：[stale-subasset-reference-repair](../frameworks/02/fixed-2026-07-14-stale-subasset-reference-repair.md)
- fixed 已修复：[zrpack-blake3-contract-drift](../../zircon_plugins/07/fixed-2026-07-14-zrpack-blake3-contract-drift.md)
- fixed 已修复：[zr-vm-host-modules-runtime-test-owner-drift](04/fixed-2026-07-14-zr-vm-host-modules-runtime-test-owner-drift.md)
- 2026-07-14 migration journal GREEN：受管 job `b337b21337c84d248905915d3ceaf875` 从当前源码通过 `minted_sidecar_commit_crash_is_whitelisted_and_next_apply_converges` 1/1；Plugins08 owner handoff 已 fixed 返回。core-min scene filter 为 595/596，唯一失败位于其他 owner 的 Scene reflection `JsonNumber` 类型漂移；broad `asset::` 仍未关闭，因此本计划继续 `in_progress`。
- Editor03 完整 Runtime 回归门发现 Virtual Geometry debug snapshot integration fixture 仍调用退役的无 resolver TOML API：`待修复（open）`；[failure 交接](04/failure-2026-07-15-virtual-geometry-debug-snapshot-project-toml-consumer-drift.md)。修复必须迁移到 project resolver-aware 序列化合同，不得恢复 `to_toml_string()` 兼容入口。
- fixed 已修复：[text-hard-cut-runtime-consumer-type-drift](04/fixed-2026-07-15-text-hard-cut-runtime-consumer-type-drift.md)
- Frameworks05 library gate 转绿后，Runtime04 focused VG test 已真实执行，但根级 Virtual Geometry support fixture 缺少 Plugins13 已规定的 AsyncCompute workload：`待修复（open）`；[failure 交接](../../zircon_plugins/13/failure-2026-07-15-virtual-geometry-runtime-support-compute-workload-drift.md)。受管 job `1e7cdd7825024a08b236b2edd07c67b9` 为 `0 passed / 3 failed / 4 ignored`；三个失败均发生在 descriptor compile，不能计作 Runtime04 project-TOML 修复失败或通过。
- 2026-07-18 core resource性能交接：typed get/acquire的完整record snapshot、register/reload重复record owner/revision compare及ready sort UUID格式化已止损；acquire payload读取与ref-count增加仍非事务，last release可夹入移除authority，每lease还分配释放闭包且drop同步拿runtime/payload锁，subscriber为无界锁内fanout。Runtime04联动Runtime07以per-resource Arc entry、generation发布、小型last-drop事件、有界回收/event lane与frame batch/cursor收口；见PERF-MVP-327及`docs/plans/performance/01/2026-07-18-runtime-core-manager-resource-static-review.md`。
- 2026-07-18 IBL/import 性能交接：source-cubemap compile options 在稳定帧同步读取完整 `.zribl` 并多次深拷贝 payload，cache miss 同帧还可能二次读；外部 cubemap decode/staging 的 production 入口仍串行，测试 executor 最多只有每 mip 六个 face 任务。Runtime04 联动 Render11/17按 request identity + cache generation 异步加载并发布 resident `Arc` artifact，按 mip×face×tile 调度 asset compute pool 或 GPU bake，稳定帧 frame I/O/decode/clone 与 caller blocking 均为 0；见 PERF-MVP-352/354。
- 2026-07-18 shader asset/prewarm 性能交接：prewarm manifest按variant复制WGSL/include/version并串行validate/compress/write，shader package与IDE/template消费者各自重复解析include。Runtime04联动Render08/17与Editor09发布content-addressed source table、bounded asset worker queue及generation-owned单遍parse artifact；同source正文只存一次、provenance hash=1、changed source scan=1、stable scan=0；见PERF-MVP-357/358。
- 2026-07-18 environment upload artifact交接：IBL resident artifact到GPU之间仍缺预编码边界，source/PMREM/irradiance在render submission线程同步f32→RGBA16F并按face×mip碎片上传。Runtime04联动Render11/13/17让asset/bake worker发布版本化row-aligned upload artifact并由持久staging arena单batch提交；stable转换/上传=0、changed artifact build≤1/generation，复用PERF-MVP-352/354唯一resident owner。见PERF-MVP-380。
- 2026-07-18 mesh deform/resident artifact交接：mesh import/compile须把morph target静态delta、skeleton引用与content/revision keyed GPU-upload payload发布为immutable artifact；render frame不得重新展开target×vertex，也不得对稳定`Dynamic`/CPU-morphed source逐draw调用`GpuMeshResource::from_asset`。Runtime04联动Plugins04/Render03 single-flight发布，stable artifact build/read/copy=0、changed≤1/content generation。见PERF-MVP-385/386/389。
- 2026-07-18 scene resource streamer交接：Runtime04须由asset events发布一次resource/dependency revision generation与批量只读snapshot，CPU I/O、decode、mesh hash/可选wire、material/shader依赖解析及upload command在有界single-flight jobs完成；render线程只按byte/object/time预算应用ready artifact并保留last-good。当前frame unique ensure及material/shader稳定命中已止损，最终stable registry逐资源锁/load/decode=0、changed近dirty resources。见PERF-MVP-404及scene-resources静态证据。
- 2026-07-18 GPUScene delta来源交接：Runtime04/scene extract须发布added/changed/removed dense records和generation，不让Render03每帧从完整draw列表重建live HashSet/entry HashMap；morph/VG静态payload沿用content artifact，pose/weight/transform只携dirty identity。stable extract到GPUScene records=0、changed近delta，见PERF-MVP-405。
- 2026-07-18 shader module artifact补充：当前template/IDE env每次重建builtin includes并重复extract/strip/hash，IDE每stub又全表找依赖、拼接全文和Naga parse。Runtime04按PERF-MVP-358发布content-addressed parsed module与indexed dependency DAG，changed source只更新受影响closure，稳定env/preview/compile scan/hash/parse=0；后台job有in-flight/RSS预算并向Render08/Editor09发布generation ticket。
- 2026-07-18 plugin shading include索引补充：当前每descriptor的forward/GBuffer/deferred token各自全扫ready shader records并同步load正文。Runtime04把normalized include token→resource id/revision/parsed module Arc纳入同一shader generation DAG，构建时报告duplicate，consumer O(1)借用；stable record scan/path normalize/load/source clone=0，见PERF-MVP-358/404。
- 2026-07-22 offline tool补充：`zircon_shader_prewarm`当前对同一asset root重复resource/shader/registry/material遍历并按source重走include DAG，归PERF-MVP-448与Render08 failure；`zircon_export_pack`把全部asset bytes、determinism inputs与delta/target pack多轮复制，归PERF-MVP-449与Editor15 failure。Runtime04提供唯一content-addressed staged asset/source/chunk artifact和revision inventory，consumer不得各自重扫目录或复制完整payload。
- 2026-07-22 Scene LevelManager consumer补充：trait load/save每次按project root重开ProjectManager并全量`scan_and_import`，单scene save还深clone World并在caller线程同步serialize/write。Runtime04把该入口纳入open `project-source-index-targeted-import` failure与PERF-MVP-453：消费prepared project generation/targeted transaction，save走immutable scene artifact ticket+bounded I/O atomic publish；不得保留Scene私有project cache或第二条full-scan truth。
- 2026-07-22 World project I/O补充：legacy save内部第二次World clone、normalize id Vec及builtin locator重parse已止损（PERF-MVP-462）；Level snapshot、宽SceneAsset/NodeRecord投影、完整pretty JSON String和同步fs write仍在caller线程。Runtime04继续按PERF-MVP-453发布generation scene artifact，Runtime11 bounded I/O lane完成single-flight serialize/atomic replace/shutdown flush；见project I/O静态证据与既有open failure。
- 2026-07-22 dynamic scene asset reload交接：每frame无上限drain，逐event完整重建pending Vec形成O(E×P)，superseded DetachOnDrop任务继续耗worker，ready scenes在一个Level world锁内无预算apply。Runtime04联动Runtime11建立per-AssetId latest-only/cancel generation、bounded drain/apply与lifecycle prune；见PERF-MVP-471和`04/failure-2026-07-22-dynamic-scene-asset-reload-bounded-singleflight.md`。
- 2026-07-22 dynamic scene session archive artifact交接：save/load/capture在archive、World、DynamicScene、Value与String之间重复深clone/parse/normalize，稳定generation也没有唯一sealed payload。Runtime04发布project/scene/schema generation-owned immutable archive artifact，summary/index/preview/typed serde共享一次capture/validation；Runtime11只消费sealed artifact执行I/O。见PERF-MVP-474和`04/failure-2026-07-22-dynamic-scene-session-archive-artifact.md`。
- 2026-07-22 typed asset event交接：PERF-MVP-492已删除每个`AssetEventReceiver<T>`的专用过滤线程和二级无界队列；底层`ResourceManager::subscribe/broadcast`仍为每subscriber无界排队并持全局subscriber锁逐项clone+send。Runtime04需发布共享有界generation event log/ring与consumer cursor，和PERF-MVP-471场景reload预算共同验收；见`04/failure-2026-07-22-typed-asset-event-shared-bounded-dispatch.md`。
- 2026-07-22 asset readiness交接：generic facade readiness在一次查询内重复clone root、遍历direct/recursive依赖并逐node分别读取registry/runtime/payload；Runtime04按PERF-MVP-493在import/reload generation维护聚合state/dependency revision并提供单generation bulk snapshot，完整report每node最多fetch一次；见`04/failure-2026-07-22-asset-readiness-generation-snapshot.md`。
- 2026-07-22 project/registry静态审查补充：`.zmeta`双TOML parse、dependency O(D²)去重与refresh edge-list深拷贝已按PERF-MVP-494/495止损；`scan_and_import`仍对同root执行约5轮inventory/meta load，stable source仍整文件read/hash，watch单path仍全量meta scan/edge refresh/registry persist。Registry还缺AssetId、reverse dependency与source slots索引，referencer/反解/remove为全表或changes×entries扫描。Runtime04按PERF-MVP-496/497扩充既有`project-source-index-targeted-import` failure，联动Runtime11 bounded I/O与Editor10 generation consumer。
- 2026-07-22 project generation/residency交接：open/watch/import/reimport仍在generation/project锁内构造深候选、执行全scan并同步读取/prepare全部artifact，随后clear/commit整套resource state；lazy `ensure_resident`因此未降低MVP启动和单文件热重载I/O/RSS。Runtime04按PERF-MVP-499在锁外构造metadata/delta generation、短CAS发布、startup working set按需single-flight驻留，联动Runtime11有界jobs；见`04/failure-2026-07-22-project-generation-lazy-residency-publish.md`。
- 2026-07-30 Editor F0消费链补充：current `open_prepared_project`的generation guard覆盖watcher prepare、全量scan/import、resource prepare/commit与broadcast，返回后Editor09又同步全registry读meta/artifact并建catalog，随后载入watcher/document。Runtime04的PERF-MVP-499 candidate generation必须同时供runtime residency与Editor09 catalog消费，不能只缩短manager open却让Editor二次全量I/O；长I/O锁外、短CAS publish、MVP working-set residency与last-good rollback按1/1K/100K assets验收。证据见`../../performance/01/2026-07-30-editor-ui-host-startup-project-current-review.md`。
- 2026-07-22 management projection交接：kind/list/overview/status/issue重复全registry scan+sort并深clone资产，scene records/entities重复加载同一scene，resource list比较器还分配locator String。Runtime04按PERF-MVP-500发布generation-owned compact rows/indices/summary，Editor09仅投影visible page与selected detail，Render17记录stable 60Hz build/clone/sort；见`04/failure-2026-07-22-asset-management-generation-projection.md`。
- 2026-07-23 runtime-interface resource合同补充：`resource/**` 14/14静态审查确认retained editor在任意resource event batch后消费`list_resources() -> Vec<ResourceRecord>`，因此仍会全registry深clone、以locator String排序并在主线程发布宽snapshot；继续并入PERF-MVP-500的唯一generation-owned compact rows/typed status/detail设计。locator normalize与AssetReference/ResourceId stable identity还存在replace、component String Vec、locator formatting和joined hash多层中间分配，按PERF-MVP-564以bit-for-bit golden UUID约束的单遍canonical writer/hash sink收口；owned event fanout继续归PERF-MVP-492，不另建队列truth。
- 2026-07-22 watcher背压交接：debounce原始事件Vec已按PERF-MVP-501直接改为AssetUri增量折叠，暂存O(E)→O(unique URI)；notify ingress、Pending/Draining changes/errors仍无界，每个event重置debounce可在持续风暴下长期不flush，callback还同步执行project锁内scan/import/resource prepare。Runtime04发布有capacity/max-latency/overflow-reconcile的watch generation，Runtime11只准备affected closure；见`04/failure-2026-07-22-asset-watch-bounded-debounce-generation.md`。
- 2026-07-22 importer registry交接：PERF-MVP-502已删除select逐matcher规范化String分配；registry仍为Vec全扫，capability ranking clone诊断文本，register按new matcher×existing matcher重算key，Default/active registry重复构造/clone。Runtime04联动Plugins12按PERF-MVP-503发布extension/full-suffix/id/plugin immutable generation indices；见`04/failure-2026-07-22-asset-importer-generation-index.md`。
- 2026-07-22 importer source/cook补充：owned root/model/material/glTF material深clone与shader WGSL二次文件读取已按PERF-MVP-502止损；glTF仍source bytes预parse后按path二次read/parse，OBJ重开path，subasset复制mesh/image/VG payload并O(D²)依赖去重/重复hierarchy，IBL cache hit前完整decode，font反复read/decode/metadata。Runtime04/11按PERF-MVP-504发布content/revision keyed source reader与唯一parse/decode/cook/shared artifact；WOC glTF、352/354/358和Text01 font failure承接具体格式，不另建并行truth。
- 2026-07-22 artifact store交接：`artifact/**` 16/16静态审查确认普通cache深拷贝wire DTO并整块serialize/read/decode，UI document还经TOML String中转；IBL三类store整blob读写、candidate再次clone blob且source environment复制texels。PERF-MVP-505已删除zstd独立compressed Vec；Runtime04按PERF-MVP-506发布content-addressed manifest/chunk generation、流式atomic write与按需shared decode，并与352/354唯一IBL resident owner合并。见`04/failure-2026-07-22-asset-artifact-chunked-generation-store.md`。
- 2026-07-22 OBJ format补充：`runtime_asset_path` 2/2与`formats` 9/9静态审查完成；PERF-MVP-507已删除OBJ逐face token Vec并保留少顶点错误优先级。decoder按path整文件read与importer source ticket脱节继续由PERF-MVP-504收口，禁止为formats层建立第二个source cache。
- 2026-07-22 VG cook generation交接：`virtual_geometry_cook` 5/5静态审查完成，PERF-MVP-508已把page offset O(P²)降为O(P)并复用dump排序/借用cluster ids。runtime及glTF/OBJ/model插件仍对每个非蒙皮primitive无条件同步cook且无content+config generation cache；Runtime04按PERF-MVP-509发布唯一immutable VG artifact，feature-off为0 cook，联动Plugins12请求策略与Runtime11有界并行stage。见`04/failure-2026-07-22-virtual-geometry-cook-generation-policy.md`。
- 2026-07-22 asset migration交接：`migration/**` 17/17静态审查完成，PERF-MVP-510已把report改为单String直写。一次命令仍对root执行至少4轮递归并在每reference重复filesystem probe，归PERF-MVP-511的single typed inventory；transaction每document前后完整重写全journal形成O(D²)，并多次整文件read/hash/copy，归PERF-MVP-512的compact durable state log与streaming atomic replace。见`04/failure-2026-07-22-asset-migration-single-inventory-generation.md`和`04/failure-2026-07-22-asset-migration-streaming-transaction-journal.md`。
- 2026-07-22 zrpack底层交接：`pack/**` 17/17静态审查完成；PERF-MVP-513已把manifest/reader/delta lookup切到sorted binary search，unique chunk validation不再按asset复制+重复hash，writer/delta删除全path/全target row clone。base+delta+rebuilt、writer inputs+pack output和promotion validation仍整包多份驻留并同步I/O，继续由PERF-MVP-449及Editor15既有failure统一流式收口；Runtime04提供506 content chunks，不另建pack私有chunk truth。
- 2026-07-22 model/mesh资产补充：`assets/model` 4/4与`assets/mesh` 12/12静态审查完成；PERF-MVP-514已删除model→mesh整份primitive clone和normal/tangent完整index临时Vec，VG ordinal在joint-index属性投影时原位编码。剩余morph/skin/GPU resident静态payload继续由PERF-MVP-385/386/389的content/revision immutable artifact收口，VG请求与cook继续归509；禁止mesh层建立第二套payload/generation cache。
- 2026-07-22 material资产补充：`assets/material` 14/14静态审查完成；PERF-MVP-515已把management summary从9次records遍历降为1次。overview/dependency/readiness/descriptor仍重复物化slot/reference/error集合，dependency dedup与slot反查及shader layout校验可达O(T²/P×schema)，parent chain深clone完整maps。Runtime04按PERF-MVP-516扩充358/360/404的material+parent+shader+texture revision DAG，发布唯一effective material、indexed contract与compact readiness；Render08/Editor09只消费generation artifact。
- 2026-07-22 shader资产补充：`assets/shader` 8/8静态审查完成；PERF-MVP-517已把management summary 14次records遍历降为1次并删除entry stage lowercase分配。property packing首适配最坏O(P²)，variant按entry复制全部defines，readiness/management重复持有宽report。Runtime04按PERF-MVP-518扩充355..358/404的content/schema/include generation artifact，发布确定性近线性layout、共享defines/entry/layout/WGSL与compact counters；full detail只显式请求。
- 2026-07-22 scene资产补充：`assets/scene` 13/13静态审查完成；PERF-MVP-519已把scene/entity aggregate从17/18次records遍历降为各1次，entity list不再先建scene aggregate并clone宽rows。entity overview仍为计数clone完整reference Vec，scene management内嵌全部entity rows且consumer会再次复制。Runtime04按PERF-MVP-520扩充453/474/500的scene generation，发布compact rows/counters/reference indices与selected detail handle；见既有`asset-management-generation-projection` failure。
- 2026-07-22 texture资产补充：`assets/texture` 22/22静态审查完成；PERF-MVP-521已让metadata消费唯一owned descriptor并把Cube LUT默认descriptor构造3→1。readiness仍反复normalize descriptor、parse container/format并拥有format String，Runtime04按PERF-MVP-522发布content+descriptor+device-capability keyed normalized/parsed upload generation；Bevy previous-descriptor GPU复用与texture cache作为对照，consumer不得各自重建plan。
- 2026-07-22 texture payload/chunk补充：array/cubemap/lightmap/IBL/`.zcube`与external cubemap存在source+output+scratch整块峰值、header二次parse和per-face/mip临时Vec复制。PERF-MVP-523并入504/506/352/354/380/404的唯一content chunk truth：worker直接写最终upload-ready chunks，Render13按dirty mip/face/layer预算提交；不照搬Godot 3D texture先consolidate全部image data的全块策略。
- 2026-07-22 root/project asset补充：assets根11/11、`project_document`4/4、`ui`2/2、`sprite_atlas`3/3静态审查完成。PERF-MVP-524..526已删除WAV逐sample checked Result、TTC逐table临时Vec、project document TOML String/重复parse、UI URI String与atlas name成功路径分配。Runtime04按527发布Data/project sealed typed generation，按528发布sound shared source/metadata，按529发布UI direct-reference与sprite name index；全部复用504/506唯一content truth。
- 2026-07-22 Editor asset projection/import补充：EditorAssetIndex pending reconcile已删path clone+二次remove，
  stable rows collect/sort与registry replacement全量validate继续并入PERF-MVP-500/556的唯一ordered asset
  generation；Editor09 import flow的同URI mutex只串行而不single-flight，Runtime04向Editor09暴露稳定
  source/import generation identity与唯一AssetManager ticket，实际import≤1/UUID/generation，不能让Editor复制
  worker或第二import truth。队列budget归Editor14/Runtime11，见PERF-MVP-555。
- 2026-07-23 runtime-interface project合同补充：`project/**` 39/39确认manifest每读走TOML Value→JSON Value→typed，template create还clone全embedded bytes并重复parse/encode，Editor落盘后再load/save。Runtime04按PERF-MVP-568发布content-generation绑定的唯一typed manifest artifact，直接投影summary并给Editor10/Hub03共享；JSON中间层与consumer私有parser/cache硬删。AssetRef/RelPath/project-name分配和asset-root O(R²)按569以borrowed serde、单遍canonical writer、indexed overlap与hard budget收口；迁移walker继续归511/512。
