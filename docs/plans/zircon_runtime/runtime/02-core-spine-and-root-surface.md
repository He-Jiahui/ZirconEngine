---
related_code:
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/core/framework/channel.rs
  - zircon_runtime/src/core/framework/error.rs
  - zircon_runtime/src/core/framework/events.rs
  - zircon_runtime/src/core/runtime/config_store.rs
  - zircon_runtime/src/core/runtime/events.rs
  - zircon_runtime/src/core/runtime/frame_clock.rs
  - zircon_runtime/src/core/runtime/tasks/job_scheduler.rs
  - zircon_runtime/src/core/runtime/lifecycle.rs
  - zircon_runtime/src/core/runtime/time.rs
  - zircon_runtime/src/core/runtime/descriptors/service_object.rs
  - zircon_runtime/src/core/framework/state
  - zircon_runtime/src/core/runtime/tasks
  - zircon_runtime/src/core/runtime/modules
  - zircon_runtime/src/core/runtime/diagnostics
  - zircon_runtime/src/foundation/mod.rs
  - zircon_runtime/src/tests/runtime_absorption/root_entries.rs
  - zircon_runtime/src/tests/runtime_absorption/root_surface.rs
  - zircon_runtime/src/tests/runtime_absorption/naming_boundary.rs
  - zircon_runtime/src/tests/runtime_absorption/generated_code_guard.rs
  - zircon_runtime/src/tests/runtime_absorption/core_spine_root_generated.rs
  - docs/zircon_runtime/core/root_surface.md
  - docs/engine-architecture/generated-code-boundary.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/core_spine_root_generated_boundary.py
  - docs/zircon_app/export-bootstrap.md
  - zircon_app/src/entry/export_bootstrap.rs
  - zircon_app/src/entry/entry_runner/bootstrap.rs
  - zircon_app/src/entry/tests/export_bootstrap.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan_feature_provider.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan_platform.rs
  - zircon_runtime/src/plugin/export_build_plan/platform_host_files/browser.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/index.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
status: in_progress
last_refined: 2026-06-15
---

# 02 core spine 与 root surface 收束

## 现状与证据（2026-06-12 重核）

- **散件形态矫正**：core 根散件不是裸公开模块，而是"私有 `mod` + `core/mod.rs` 根部精选 `pub use`"形态（`core/mod.rs:3-13` 全部 `mod xxx;` 私有声明，`:23-44` 逐件再导出）。调用方使用的是 `core::FrameClock`、`core::EventBus`、`core::ZirconError` 这类再导出名，而非 `core::frame_clock::` 全路径——迁移时改的是 `core/mod.rs` 的声明与 `pub use` 来源，调用方 `use` 行多数不变。
- **散件清单（core/ 根实测）**：私有文件 9 件——`channel_util.rs`、`config_store.rs`、`error.rs`、`event_bus.rs`(+`event_bus/` 子目录)、`frame_clock.rs`、`job_scheduler.rs`、`lifecycle.rs`、`time.rs`、`types.rs`；公开目录 4 件——`state/`、`tasks/`、`modules/`、`diagnostics/`。五件套 spine（`runtime/framework/manager/math/resource`）之外共 13 件待归属。
- **双形态误判矫正**：`event_bus.rs` + `event_bus/` 不是迁移债——`event_bus.rs:3-6` 声明 `mod failure/prune/publish/subscribe`，是 file-as-directory-owner 惯例（仓内通行），迁移时整体 `git mv` 两者即可。
- **调用面实测**（Grep 再导出名，zircon_runtime/src 内文件数）：`ConfigStore` 4、`FrameClock` 4、`recv_latest|spawn_named_thread|wait_for` 5（外部仅 `asset/facade/event.rs`、`asset/pipeline/worker_pool.rs`）、`ZirconError` 6、`RuntimeTimeClocks|RuntimeTimeAdvance` 7、`JobScheduler` 9、`EventBus|EngineEvent` ≥12、`TaskPool*` ≥12、`NextState|StateTransitionEvent` ≥12、`modules::` ≥12（builtin/runtime_modules 装配族）、`LifecycleState|StartupMode` ≥12（scene/script/navigation/animation 模块广用）、`core::diagnostics` ≥12（dynamic_api/diagnostic_log）。窄面（≤9）调用方集中在 `core/runtime/runtime.rs`、`core/runtime/state/runtime_inner.rs`、`core/runtime/handle/*` 与 `prelude.rs`。
- **`FrameClock` 极小**（`frame_clock.rs` 25 行，仅 `tick() -> Duration`），固定步长扩展空间归子计划 03。
- **foundation/ 边界澄清**：`foundation/mod.rs` 导出 `FoundationModule` + `ConfigDriver/DefaultConfigManager/EventDriver/DefaultEventManager`——是把 core 的 config/event 原语包装成可注册 runtime module 的装配壳（被 `builtin/runtime_modules/core_modules.rs` 消费），与 core 散件是"原语 vs 模块注册"分层而非重复实现。重叠风险点只在 `foundation/runtime/{config_manager,event_manager}.rs` 是否藏有应属 core 的行为。
- **lib.rs 别名块**（`lib.rs:39-72`，`pub(crate) use` + `#[allow(unused_imports)]`）：约 70 个 graphics 类型 + 8 个 graphics 子模块名（`backend, extract, feature, material, pipeline, runtime, types, visibility`——其中 `runtime`/`types` 与 `core::runtime`/`core::types` 同名，crate 根语义被污染）。实测使用极少：抽查三组（SceneRenderer/GraphicsError/WgpuRenderFramework/ViewportFrame 4 文件、`crate::extract::` 等模块别名 3 文件、HybridGi/VG/Solari provider 1 文件）共 8 个调用文件，**全部位于 graphics 自身内部** + 1 个测试（`tests/plugin_extensions/extension_registry.rs`）。
- **generated 口径矫正**：真实文件头 `// @generated ...` 标记全 src 当前为 0；普通 `@generated` grep 会命中守卫测试里的格式常量，不能当作生成物计数。"generated" 词根（含注释、领域词）当前命中 42 个 Rust 文件——其中混入 `asset/assets/mesh/normals.rs`（法线生成领域词）、`core/runtime/diagnostics/profiling/*`（hotspot 报告用语）、graphics/RHI 测试、export build plan 手写模板源，以及 `runtime_absorption/generated_code_guard.rs` 自身。守卫规范已定稿为文件首行 `// @generated <generator> - do not edit by hand`。
- 参考对照（每点一行）：Bevy crate-per-subsystem 用 crate 边界承担本计划中模块边界的职责 — `dev/bevy/crates`；Fyrox 分层 crate（core/resource/impl）— `dev/Fyrox`；UE 模块树 Runtime 约 189 模块 — `dev/UnrealEngine/Engine/Source/Runtime`。

补充参考锚点（2026-06-13 实测核验，实现型切片动工前先读——index 公约 §7.9）：

- Piccolo 小型引擎单 runtime 分层全栈（core/function/platform/resource 四分，与本仓"单 crate + 内部 spine"同形）— `dev/Piccolo/engine/source/runtime`

## 目标

1. core 根只剩五件套 spine（+ 定稿后的 diagnostics 席位）+ `mod.rs`，13 件散件全部硬切换迁入 owner。
2. `lib.rs` 删除 `pub(crate)` 别名块全部 34 行，8 个实测调用文件改用真实 owner 路径；收尾后 `lib.rs` 无 `#[allow(unused_imports)]`。
3. generated-code 规则固化：显式标记规范定稿 + 结构守卫，生成文件只许 leaf binding/DTO/table。

## 非目标

- 不重写任何被迁移模块的行为；只移动与改路径（`FrameClock` 的固定步长扩展归子计划 03）。
- graphics 内部模块重组归 render 计划 01-08 与 RHI 会话；本计划只处理 crate 根别名块。
- `foundation/` 的 Driver/Manager 装配壳保留现职责，本计划只裁决其与 core 原语的重叠行为（若有），不动其模块注册角色。
- 渲染骨架内容（RDG/MeshDrawCommand/GPUScene/可见性/光照/时域/后处理/permutation）一律归 render 计划 01-08。

### 全局硬约束（继承总计划 §4，违反即返工）

- 不新增 crate；硬切换不留 re-export/alias/shim（`core/mod.rs` 的既有精选 `pub use` 属定稿公共面，按 M1 裁决逐条保留或收回，不属临时桥）。
- generated 产物只许 leaf DTO/table；动态边界只传 ABI-safe 值。
- 非网络语义 server 命名是 blocker。

## 执行前检查清单

开工前逐项完成，未过项禁止动工：

1. **强制前置**：重读 `.codex/sessions/20260604-1232-runtime-architecture-review.md` 最新状态（该会话正在做 root surface / 大文件债切片）；归属矩阵必须与其口径合并。另核对 `.codex/sessions/` 是否有更新的活跃会话触及 `core/**`。
2. worktree 脏文件检查（10fps 会话改动是 live state，脏文件先避让）：
   - `git status --porcelain -- zircon_runtime/src/core/ zircon_runtime/src/foundation/ zircon_runtime/src/lib.rs`
   - `git status --porcelain -- zircon_runtime/src/builtin/ zircon_app/src/entry/`
3. 事实重核（行号漂移以重核为准并回写本计划）：
   - `ls zircon_runtime/src/core/`（核 13 件散件清单）
   - `grep -n "pub use\|^mod \|^pub mod" zircon_runtime/src/core/mod.rs`
   - `grep -n "pub(crate) use" zircon_runtime/src/lib.rs`（核别名块仍在 39-72 一带）
   - 调用面计数重跑：Grep `FrameClock|ConfigStore|JobScheduler|EventBus|LifecycleState`，path `zircon_runtime/src`
4. 确认 `zircon_app`/`zircon_editor` 对 `zircon_runtime::core` 的引用面（迁移波及范围）：Grep `zircon_runtime::core`，path `zircon_app/src` 与 `zircon_editor/src`（实测各 ≥10 文件，M2 测试阶段必须双 crate 回归）。
5. 基线记录：`cargo check -p zircon_runtime --lib --locked` 耗时与 `cargo test -p zircon_runtime --lib core:: --locked` 通过数，记入"状态与产出记录"。

## 里程碑

### M1 core 散件归属裁决（先文档后代码）

#### 切片 1.1 归属矩阵定稿

- 目标文件：本计划"状态与产出记录"节（矩阵落本文件）；`docs/engine-architecture/core-runtime-service-registry.md`（同步归属口径一节）。
- 改动形态：纯文档。逐件归属矩阵（候选口径，与 `20260604-1232` 会话对齐后定稿）：

  | 散件 | 调用面（实测文件数） | 候选归属 | 理由 |
  |---|---|---|---|
  | `frame_clock.rs`（25 行） | 4 | `core::runtime` | 帧时钟原语，03 计划扩展点 |
  | `time.rs` | 7 | `core::runtime` | `RuntimeTimeClocks` 已被 `core/runtime/handle/time.rs` 消费 |
  | `job_scheduler.rs` | 9 | `core::runtime` | 调度内核件，ECS 并行 executor 引用 |
  | `tasks/` | ≥12 | `core::runtime` | TaskPool 族与调度同域 |
  | `lifecycle.rs` | ≥12（最广） | `core::runtime` | LifecycleState/StartupMode 是 runtime 生命周期词汇 |
  | `state/` | ≥12 | `core::runtime` | App state machine（Bevy States 对应物） |
  | `modules/` | ≥12 | `core::runtime` | 被 builtin 装配族消费 |
  | `event_bus.rs`+`event_bus/` | ≥12 | `core::framework` | 中性事件原语 |
  | `channel_util.rs` | 5 | `core::framework` | 通道原语（外部调用方仅 asset 两处） |
  | `types.rs` | — | `core::framework` | `ChannelSender/Receiver/ServiceObject` 共享原语 |
  | `error.rs` | 6 | `core::framework` | `CoreError/ZirconError` 中性契约 |
  | `config_store.rs` | 4 | `core::resource` 或 `core::manager` | 二选一并记录理由（配置即资源定位 vs 配置即受管服务） |
  | `diagnostics/` | ≥12 | `core::runtime::diagnostics` 或 spine 第六席 | 若留第六席必须同步修订收束计划文档 spine 口径 |

- 2026-06-12 定稿判词（按实仓重扫，命中数为 `zircon_runtime/src/**/*.rs` 文件/行级近似，含测试守卫）：

  | 散件 | 重扫命中 | 定稿归属 | root `pub use` 处置 | 理由 |
  |---|---:|---|---|---|
  | `config_store.rs` | 6 文件 / 16 行 | `core::runtime::config_store` | 收回 `ConfigStore` 根再导出 | `ConfigStore` 是 `CoreRuntime` 配置 backing store；`core::manager` 只暴露 `ConfigManagerHandle`，`core::resource` 不拥有进程配置。 |
  | `frame_clock.rs` | 6 文件 / 10 行 | `core::runtime::frame_clock` | 暂保留 curated facade，03 固定步接通后复核 | 帧 delta 原语由 runtime tick 驱动，03 计划会继续扩展。 |
  | `channel_util.rs` | 5 文件 / 12 行 | 拆分：`recv_latest`/`wait_for` -> `core::framework::channel`，`spawn_named_thread` -> `core::runtime::tasks` | 收回三函数根再导出 | 通道等待是中性 primitive；线程创建是 runtime task 执行基础设施。 |
  | `types.rs` | 51 文件 / 355 行 | 拆分：`ChannelSender/Receiver` -> `core::framework::channel`，`ServiceObject` -> `core::runtime::descriptors` | `Channel*` 可经 framework facade，`ServiceObject` 不再从根导出 | channel aliases 是中性 ABI/contract 辅助；`ServiceObject` 是 runtime registry 内部对象槽。 |
  | `error.rs` | 65 文件 / 330 行 | `core::framework::error` | 保留 `CoreError`/`ZirconError` 根 facade | 错误类型穿过 framework trait、manager handle 与 runtime services，是共享契约而非具体行为。 |
  | `event_bus.rs` + `event_bus/` | 20 文件 / 127 行 | 拆分：`EngineEvent` -> `core::framework::events`，`EventBus` 实现 -> `core::runtime::events` | 保留 `EngineEvent`/`EventBus` 根 facade 到事件切片结束 | 事件 DTO 中性；订阅表、delivery lock、prune/publish 行为由 `CoreRuntime` 拥有。 |
  | `time.rs` | 8 文件 / 46 行 | `core::runtime::time` | 保留 `RuntimeTime*` 与诊断常量根 facade | `RuntimeTimeClocks` 消费 `framework::time::{Real,Virtual,Fixed}`，但外层 advance 语义属于 runtime tick。 |
  | `job_scheduler.rs` | 10 文件 / 26 行 | `core::runtime::tasks::job_scheduler` | 保留 `JobScheduler` 到 task 切片结束，随后复核 prelude | 它只是 `TaskPool` 的 compute facade，归 runtime task pool owner。 |
  | `tasks/` | 18 文件 / 125 行 | `core::runtime::tasks` | 保留 task public facade | task pool 是 runtime 执行基础设施；descriptor/kind 已从 `framework::tasks` 复用。 |
  | `state/` | 18 文件 / 142 行 | `core::framework::state` | 保留 state public facade | Bevy-style state contracts 是中性调度契约；具体 runtime handle 只消费 registry。 |
  | `modules/` | 10 文件 / 36 行 | `core::runtime::modules` | 保留 core module public facade | Diagnostics/FrameCount/Log/Tasks/Time 模块属于 runtime core module assembly。 |
  | `lifecycle.rs` | 52 文件 / 683 行 | `core::runtime::lifecycle` | 保留 `LifecycleState`/`StartupMode`/`ServiceKind` 根 facade | 三者定义 module/service/plugin 生命周期与 service kind，是 runtime kernel vocabulary。 |
  | `diagnostics/` | 34 文件 / 159 行 | `core::runtime::diagnostics` | 保留 diagnostics facade，目标不新增 spine 第六席 | diagnostics 是 `CoreRuntime` 与 runtime services 的观测面；不把 `diagnostics` 固化为五件套之外的长期 root 席位。 |

- 调用方迁移：无（纯裁决）。
- 验收：矩阵每行有"归属 + 理由 + 调用面数值"；`config_store` 与 `diagnostics` 两个二选一项有明确判词，无"待定"。
- DoD：矩阵写入本文件且 `core-runtime-service-registry.md` 含同口径一节。

#### 切片 1.2 foundation 重叠裁决

- 目标文件：本计划状态节；`docs/zircon_runtime/`（foundation 镜像文档，执行时核验确切路径：`ls docs/zircon_runtime/`）。
- 改动形态：审计 `foundation/runtime/{config_manager.rs,event_manager.rs}` 是否含应属 core 原语的行为（超出"包装 ConfigStore/EventBus 为可注册 module"的部分），列清单逐条裁决单一 owner。`FoundationModule`/Driver 命名与注册角色保持。
- 2026-06-12 定稿判词：
  - `foundation/runtime/config_manager.rs` 只负责磁盘路径、JSON 文件读写与 `CoreHandle::{store_config_value,load_config_value,snapshot_config_values}` 转接；底层配置存储仍归 `core::runtime::config_store`。
  - `foundation/runtime/event_manager.rs` 只实现 `EventManager` facade，并转接 `CoreHandle::{publish_event,subscribe_events}`；底层 event bus 订阅/投递/剪枝行为仍归 `core::runtime::events`。
  - 重叠行为清单为空；M2 不需要迁移 foundation 代码，只需在 core 散件移动时同步 `use` 路径。
- 调用方迁移：无（裁决期）。
- 验收：重叠行为清单（可为空）+ 每条的 owner 判词。
- DoD：清单落状态节；若清单非空，迁移条目并入 M2 顺序表。

#### 切片 1.3 迁移顺序表

- 目标文件：本计划状态节。
- 改动形态：按调用面从小到大排序的执行序（候选）：`config_store`(4) → `frame_clock`(4) → `channel_util`(5) → `error`(6) → `time`(7) → `job_scheduler`(9) → `event_bus` → `tasks` → `state` → `modules` → `lifecycle` → `diagnostics`。每件标注：是否被脏 worktree 触及（执行时 `git status` 核）、`core/mod.rs` 的 `pub use` 行处置（保留改源 / 收回改全路径——广用类型如 `LifecycleState` 保留 core 根再导出为定稿公共面；窄用如 `ConfigStore`、channel_util 三函数收回，调用方改全路径）。
- 2026-06-12 定稿执行序：
  1. `config_store.rs` -> `core::runtime::config_store`；收回 root `ConfigStore`。
  2. `frame_clock.rs` -> `core::runtime::frame_clock`；暂保留 facade。
  3. `channel_util.rs` 拆分到 `core::framework::channel` / `core::runtime::tasks`；收回三函数 root facade。
  4. `types.rs` 拆分到 `core::framework::channel` / `core::runtime::descriptors`；收回 `ServiceObject` root facade。
  5. `time.rs` -> `core::runtime::time`；保留 `RuntimeTime*` facade。
  6. `job_scheduler.rs` -> `core::runtime::tasks::job_scheduler`；与 `tasks/` 同切片收束。
  7. `error.rs` -> `core::framework::error`；保留 error facade。
  8. `event_bus.rs` + `event_bus/` 拆分 DTO 与实现；保留事件 facade 到切片结束。
  9. `tasks/` -> `core::runtime::tasks`。
  10. `state/` -> `core::framework::state`。
  11. `modules/` -> `core::runtime::modules`。
  12. `lifecycle.rs` -> `core::runtime::lifecycle`。
  13. `diagnostics/` -> `core::runtime::diagnostics`；不新增长期 diagnostics spine 席位。
- 调用方迁移：无（裁决期）。
- 验收：顺序表 13 行齐备，每行有 pub use 处置判词。
- DoD：顺序表落状态节并经 `20260604-1232` 会话口径对齐确认。

#### M1 测试阶段（milestone-first）

- 纯文档里程碑：`git status --porcelain` 确认仅 docs 与本计划文件变更。
- 验收证据：归属矩阵 + 重叠清单 + 迁移顺序表 + 会话对齐记录。

### M2 散件硬切换迁移

> **执行对齐注记（2026-06-12 二次细化）**：本里程碑以 M1"定稿执行序"13 步为准；下方 2.1/2.2 的旧分组与 git mv 清单凡与定稿判词冲突处，以定稿为准执行——尤其：(a) `config_store` 落 `core/runtime/config_store.rs`（已执行）；(b) `channel_util` 拆分为 `core::framework::channel`（recv_latest/wait_for + Channel 别名）与 `core::runtime::tasks`（spawn_named_thread），不是整体迁 framework；(c) `types.rs` 拆分（`Channel*`→framework::channel，`ServiceObject`→runtime::descriptors，51 文件/355 行是最大迁移面）；(d) `job_scheduler` 落 `core/runtime/tasks/job_scheduler.rs`；(e) `event_bus` 拆 DTO（`EngineEvent`→framework::events）与实现（`EventBus`→runtime::events）；(f) **`state/` 归 `core::framework::state`**（中性调度契约），非 runtime；(g) `diagnostics/` 落 `core/runtime/diagnostics`，不设 spine 第六席。facade 处置按定稿表：收回 `ConfigStore`/三函数/`ServiceObject` 根导出，其余保留 facade 改源。每步完成即更新状态节（公约 §7.3）。

#### 切片 2.1 窄面六件迁移（config_store / frame_clock / channel_util / error / time / job_scheduler）

- 目标文件（git mv 源 → 目标，目标子路径按 M1 定稿）：
  - `git mv zircon_runtime/src/core/config_store.rs zircon_runtime/src/core/<owner>/config_store.rs`
  - `git mv zircon_runtime/src/core/frame_clock.rs zircon_runtime/src/core/runtime/frame_clock.rs`
  - split `zircon_runtime/src/core/channel_util.rs` into `zircon_runtime/src/core/framework/channel.rs` and `zircon_runtime/src/core/runtime/tasks/mod.rs`
  - `git mv zircon_runtime/src/core/error.rs zircon_runtime/src/core/framework/error.rs`
  - `git mv zircon_runtime/src/core/time.rs zircon_runtime/src/core/runtime/time.rs`
  - `git mv zircon_runtime/src/core/job_scheduler.rs zircon_runtime/src/core/runtime/tasks/job_scheduler.rs`
  - 同切片更新 `core/mod.rs`（删 6 行 `mod`，按 M1 处置表改/删 `pub use`）与目标 owner 的 `mod.rs`。
- 调用方迁移（实测，≤10 全列）：`prelude.rs`、`tests/prelude.rs`、`core/runtime/runtime.rs`、`core/runtime/state/runtime_inner.rs`、`core/runtime/handle/core_handle.rs`、`core/runtime/handle/time.rs`、`asset/facade/event.rs`、`asset/pipeline/worker_pool.rs`、`asset/pipeline/manager/project_asset_manager/construction.rs`、`scene/ecs/schedule_parallel_executor.rs`。漏网枚举：Grep `core::(config_store|frame_clock|channel_util|error|time|job_scheduler)|ConfigStore|FrameClock|JobScheduler|ZirconError|recv_latest|spawn_named_thread|wait_for|RuntimeTimeClocks`。
- 改动形态：只移动 + 改 `use` 路径；不留任何旧位置 re-export；每移 2–3 件 `cargo check -p zircon_runtime --lib --locked` 轻量确认。
- 验收：`core_root_keeps_only_spine_modules_after_narrow_item_migration`（归属 `zircon_runtime/src/tests/runtime_absorption/root_entries.rs`，该文件已存在）——断言 `core/mod.rs` 源文本不再含六件的根级 `mod` 声明。
- DoD：六件物理位置在 owner 下且 `cargo check -p zircon_runtime --lib --locked` 通过。

#### 切片 2.2 宽面七件迁移（event_bus / tasks / state / modules / lifecycle / diagnostics + foundation 重叠项）

- 目标文件：`EngineEvent` DTO 拆出到 `zircon_runtime/src/core/framework/events.rs`；`git mv zircon_runtime/src/core/event_bus.rs zircon_runtime/src/core/runtime/events.rs` + `git mv zircon_runtime/src/core/event_bus zircon_runtime/src/core/runtime/events`（file-as-directory-owner 成对移动）；`tasks/`、`state/`、`modules/`、`lifecycle.rs` → 各自 M1 定稿 owner；`diagnostics/` 按 M1 判词迁 `core/runtime/diagnostics/`。
- 调用方迁移（>10）：`LifecycleState|StartupMode` ≥12 文件（代表：`scene/mod.rs`、`scene/level_system.rs`、`navigation/module.rs`、`animation/module.rs`、`script/vm/tests.rs`）；`modules::` 装配族（代表：`builtin/runtime_modules/core_modules.rs`、`builtin/runtime_modules/assembly/*.rs`、`plugin/runtime_plugin/registration_report/native.rs`）；`core::diagnostics`（代表：`dynamic_api/session.rs`、`dynamic_api/exports.rs`、`diagnostic_log/diagnostics.rs`）。枚举命令：Grep `LifecycleState|StartupMode|EventBus|EngineEvent|TaskPool|NextState|StateTransitionEvent|core::modules|core::diagnostics`，path `zircon_runtime/src` 与 `zircon_app/src`、`zircon_editor/src`。
- 改动形态：同 2.1；广用类型按 M1 处置表保留 core 根 `pub use`（改源路径），调用方 `use core::LifecycleState` 等不动。
- 验收：`core_module_tree_matches_decided_spine_shape`（root_entries.rs）——断言 `core/` 目录条目 == 定稿口径集合（五件套 + mod.rs + 定稿席位）。
- DoD：`ls zircon_runtime/src/core/` 输出与定稿口径一致。

#### M2 测试阶段（milestone-first）

- `cargo check -p zircon_runtime --lib --locked`
- `cargo test -p zircon_runtime --lib core:: --locked`
- `cargo test -p zircon_runtime --lib runtime_absorption --locked -- --nocapture`（结构守卫）
- `cargo test -p zircon_app --locked`；`cargo check -p zircon_editor --lib --locked`（双下游回归，实测各 ≥10 文件引用 `zircon_runtime::core`）
- 验收证据：`core/` 根目录列表与定稿口径一致；全量 lib 测试无回归。
- 文档：`docs/zircon_runtime/core/**` 按源码镜像同步；`docs/engine-architecture/core-runtime-service-registry.md` 路径引用刷新。

### M3 lib.rs 别名块清理

#### 切片 3.1 模块名别名清除（最高优先：根命名污染）

- 目标文件：`zircon_runtime/src/lib.rs`（删 `:47` 行起 `pub(crate) use graphics::{backend, extract, feature, material, pipeline, runtime, types, visibility, ...}` 中的 8 个模块名别名）；调用方 3 文件（实测全列）：`graphics/pipeline/render_pipeline_asset/compile.rs`、`graphics/pipeline/declarations/compiled_render_pipeline.rs`、`graphics/shader/mod.rs`——`crate::extract::` 等改 `crate::graphics::extract::` 全路径。
- 改动形态：删别名 + 改 3 个调用文件 `use` 行；特别消除 `crate::runtime`/`crate::types` 与 `core::runtime`/`core::types` 的同名歧义。
- 验收：`lib_rs_declares_no_graphics_module_aliases_at_crate_root`（root_entries.rs）——断言 lib.rs 源文本无 `pub(crate) use graphics::{` 中的裸模块名。
- DoD：Grep `crate::(extract|visibility|backend|material|feature|pipeline)::` path `zircon_runtime/src` 0 命中。

#### 切片 3.2 类型别名逐组清除

- 目标文件：`zircon_runtime/src/lib.rs:39-72` 全删；调用方 5 文件（实测全列）：`graphics/runtime/render_framework/submit_frame_extract/record_submission/record.rs`、`record_capture.rs`、`submit_frame_extract/submit/release_previous_history.rs`、`graphics/runtime/render_framework/viewport_record/runtime_states.rs`、`tests/plugin_extensions/extension_registry.rs`——`crate::SceneRenderer` 等改 `crate::graphics::SceneRenderer` 真实路径。漏网枚举：Grep `crate::[A-Z]`，path `zircon_runtime/src`，逐条比对别名清单。
- 改动形态：无使用者的别名（约 70 个中的大多数，`#[allow(unused_imports)]` 即旁证）直接删除；HybridGi/VG/Solari provider 别名删除后确认唯一公共入口是 `graphics::feature`/render feature descriptor 路径（与 render 计划 §5.4 一致，细节归 render 计划）。
- 验收：`lib_rs_has_no_pub_crate_alias_block_and_no_allow_unused_imports`（root_entries.rs）——断言 lib.rs 无 `pub(crate) use` 且无 `#[allow(unused_imports)]`。
- DoD：lib.rs 仅含模块声明、`pub use crate::core::resource`、reflection 宏与 builtin 报告导出，行数 ≤ 45。

#### M3 测试阶段（milestone-first）

- `cargo check -p zircon_runtime --lib --locked`
- `cargo test -p zircon_runtime --lib --locked`（全量，别名清除是横切改动）
- `cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --all-targets --locked`（插件 crate 若经 crate 根引用受影响路径）
- 验收证据：lib.rs 无别名块；插件 workspace 编译通过；结构守卫进入常驻测试树。

### M4 generated-code 守卫

#### 切片 4.1 标记规范定稿与分类裁决

- 目标文件：`docs/engine-architecture/`（generated 边界文档，执行时核验既有文件名：`ls docs/engine-architecture/ | grep -i generat`，有则增量无则新建 `generated-code-boundary.md`）；本计划状态节（42 文件裁决清单）。
- 改动形态：纯文档 + 裁决。规范定稿：真生成物文件首行必须 `// @generated <generator> - do not edit by hand`（当前真实文件头合规项为 0）。42 个词根命中文件三分类：真生成物（补标记）、领域词（白名单：`asset/assets/mesh/normals.rs` 法线生成、`core/runtime/diagnostics/profiling/*` hotspot 用语、测试夹具）、生成器模板源（`export_build_plan/generated_files.rs`、`export_generated_file.rs`、`native_plugin_load_manifest_template.rs` 等——是手写 owner，非生成物）。
- 调用方迁移：无。
- 验收：40 文件清单全部有判词，违规项（生成物含行为逻辑）单列迁移清单。
- DoD：清单落状态节，违规清单为空或每条带 owner 迁移条目。

#### 切片 4.2 行为迁回与结构守卫

- 目标文件：违规清单中的文件（行为迁回手写 owner，生成文件退化为数据）；`zircon_runtime/src/tests/runtime_absorption/`（新文件 `generated_code_guard.rs` + `mod.rs` 加一行声明）。
- 改动形态：新增守卫测试（签名草案，执行时定稿）：

  ```rust
  #[test]
  fn generated_marked_files_stay_leaf_data_only() { /* 扫描 @generated 标记文件，断言无 impl 块控制流热词（match/for/while/if let，按 4.1 盘点定稿规则） */ }
  #[test]
  fn generated_marker_format_is_uniform() { /* 标记行格式统一可机判 */ }
  ```

- 调用方迁移：行为迁回时同切片更新生成器模板与消费方（按 4.1 清单逐项，预计集中在 `plugin/export_build_plan/`）。
- 验收：上列守卫 + 受影响 owner 模块聚焦测试。
- DoD：`cargo test -p zircon_runtime --lib generated --locked` 全绿且违规清单清零（或留明确 backlog 条目）。

#### M4 测试阶段（milestone-first）

- `cargo test -p zircon_runtime --lib generated --locked -- --nocapture`
- `cargo test -p zircon_runtime --lib export_build_plan --locked`（生成器族无回归）
- 验收证据：守卫进 CI 路径；`docs/engine-architecture/` generated 边界文档刷新（该文档在 `20260604-1232` 会话 touched 清单内，执行前对齐）。

## 状态与产出记录

执行时逐切片填写；完成一个切片更新一行，不许批量补记。

| 里程碑 | 切片 | 状态 | 完成日期 | 证据（命令输出 / 文件 / 测试名） |
|---|---|---|---|---|
| M1 | 1.1 归属矩阵 | 完成 | 2026-06-12 | 13 件散件定稿归属：runtime 8、framework 2、split 3；`config_store` 判归 runtime，`diagnostics` 判归 runtime 而非新 spine 席位 |
| M1 | 1.2 foundation 重叠裁决 | 完成 | 2026-06-12 | `foundation/runtime/config_manager.rs` 与 `event_manager.rs` 仅包装 CoreHandle facade；重叠行为清单为空 |
| M1 | 1.3 迁移顺序表 | 完成 | 2026-06-12 | 定稿 13 步迁移序，标明拆分项与 root `pub use` 处置 |
| M2 | 2.1 窄面六件迁移 | 完成 | 2026-06-12 | `config_store.rs` 已迁入 `core/runtime/config_store.rs`，`core` 根 `ConfigStore` 再导出已收回；`frame_clock.rs` 已迁入 `core/runtime/frame_clock.rs` 并通过 runtime facade 保留 `FrameClock`；`channel_util.rs`/`types.rs` 已拆分为 `core/framework/channel.rs`、`core/runtime/tasks/mod.rs`、`core/runtime/descriptors/service_object.rs`，并收回 root `Channel*`/`ServiceObject`/三函数 facade；`time.rs` 已迁入 `core/runtime/time.rs` 并保留 `RuntimeTime*` 与诊断常量 facade；`job_scheduler.rs` 已迁入 `core/runtime/tasks/job_scheduler.rs` 并保留 `JobScheduler` facade；`error.rs` 已迁入 `core/framework/error.rs` 并保留 `CoreError`/`ZirconError` facade；扫描确认无 `crate::core::{time,job_scheduler,error}`/`zircon_runtime::core::{time,job_scheduler,error}` 旧路径；`rustc --edition 2021 --test zircon_runtime/src/tests/runtime_absorption/root_entries.rs` 通过 5 tests；`cargo check -p zircon_runtime --lib --locked` 通过（87 个既有 warning）；此前下游 `zircon_editor` check 已通过，`zircon_plugins` check 被 lockfile 阻止未进入编译，`runtime_absorption` lib-test 仍被无关 graphics 测试编译错误 `partition_mesh_draws` 阻断 |
| M2 | 2.2 宽面七件迁移 | 完成 | 2026-06-12 | `lifecycle.rs` 已迁入 `core/runtime/lifecycle.rs`，并继续通过 `core::runtime` 与 root `core` facade 暴露 `LifecycleState`/`StartupMode`/`ServiceKind`；扫描确认无 `crate::core::lifecycle` 或 `zircon_runtime::core::lifecycle` 旧路径。`modules/` 已迁入 `core/runtime/modules/`，调用方改到 `core::runtime::modules`，root `core` 只保留具体 module descriptor/常量 facade；扫描确认无 `crate::core::modules` 或 `zircon_runtime::core::modules` 旧路径。`tasks/` 已迁入 `core/runtime/tasks/` 并与 `spawn_named_thread`/`JobScheduler` 同 owner；root `core` 保留具体 task-pool 类型 facade 但收回旧 `core::tasks` namespace；扫描确认无 `crate::core::tasks` 或 `zircon_runtime::core::tasks` 旧路径。`state/` 已迁入 `core/framework/state/`，runtime registry 继续由 `CoreRuntimeInner` 持有，root `core` 保留具体 state contract facade 但收回旧 `core::state` namespace；扫描确认无 `crate::core::state` 或 `zircon_runtime::core::state` 旧路径；`rustfmt --edition 2021 --check` 覆盖 state 切片 touched files 通过；`rustc --edition 2021 --test zircon_runtime/src/tests/runtime_absorption/root_entries.rs` 通过 5 tests；`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked` 与 `cargo check -p zircon_app --lib --no-default-features --features core-min --locked` 均通过（既有 warning 保留）。`event_bus.rs` + `event_bus/` 已拆分：`EngineEvent` 迁入 `core/framework/events.rs`，`EventBus` 与 subscribe/publish/failure/prune 实现迁入 `core/runtime/events*`，root `core` 保留事件 facade 但收回旧 `core::event_bus` namespace；事件切片 `rustfmt --edition 2021 --check` 通过，旧 event_bus 路径扫描为 0，`root_entries.rs` 结构守卫 6/6 通过，`zircon_runtime` 与 `zircon_app` 的 core-min `cargo check` 均通过（既有 warning 保留）。`diagnostics/` 已迁入 `core/runtime/diagnostics/`，root `core` 保留 `core::diagnostics` curated facade 但不再拥有 root 物理目录；diagnostics 切片 `rustfmt --edition 2021 --check` 通过，root source guard 通过，`root_entries.rs` 结构守卫 8/8 通过，`zircon_runtime` 与 `zircon_app` 的 core-min `cargo check` 均通过（既有 warning 保留）；`core/` 根目录当前仅 `framework/manager/math/resource/runtime/mod.rs` |
| M2 | 测试阶段 | 进行中 | 2026-06-12 | M2.2 物理迁移完成后进入 milestone testing stage；默认 profile `cargo check -p zircon_runtime --lib --locked --target-dir E:\cargo-targets\zircon-runtime-core-spine-0612 --message-format short --color never` 曾通过（86 warnings）。`runtime_absorption` 首次重跑暴露迁移后 stale source guard：`registry_name.rs` 仍读 `../../lifecycle.rs`，`resolution/structure.rs` 仍读 `../../../diagnostics/devtools.rs`；已改为 `../lifecycle.rs` 与 `../../diagnostics/devtools.rs`。随后 `runtime_absorption` 18/19 运行通过，唯一失败为 naming boundary 未分类新 `core/runtime/diagnostics` editor 词；已同步 test/audit 分类，`rustc --edition 2021 --test zircon_runtime/src/tests/runtime_absorption/naming_boundary.rs` 通过 1/1，`root_entries.rs` 通过 8/8。当前 Cargo lib-test/default/core-min 回归被并行 render/graphics 漂移阻塞：`RenderMeshStaticState` 从 `core::framework::render` 缺失，多个 `RenderMeshSnapshot`/`PendingMeshDraw`/`MeshPassCommandBufferStats` 初始化缺字段（代表文件：`scene/world/render.rs`、`graphics/scene/scene_renderer/mesh/**`、`graphics/tests/**`）；本 runtime core 切片不改这些 render owner 文件 |
| M2 | P2 总表状态复核 | 完成 | 2026-06-13 | 复核 `zircon_runtime/src/core/` 当前目录仅 `framework/manager/math/resource/runtime/mod.rs`；`root_entries.rs` 覆盖 `core_root_retires_channel_and_service_alias_fragments`、`core_root_retires_runtime_kernel_fragment_files`、`core_root_splits_event_dto_from_runtime_event_bus`、`core_root_reexports_runtime_diagnostics_without_root_directory` 与 `core_module_tree_matches_decided_spine_shape`；已同步 runtime 总表 P2 从“归属未定”改为 M2 硬切换完成。未启动新 Cargo：当前机器仍有其他 Cargo/rustc active lanes；M2 测试阶段仍保持“进行中”，M3 `lib.rs` graphics 别名清理继续等待 render owner 稳定 |
| M3 | 3.1 模块名别名清除 | pre_m3_root_surface_guard_static_passed_pending_render_owner | 2026-06-13 | 新增 `zircon_runtime/src/tests/runtime_absorption/root_surface.rs` 与 `docs/zircon_runtime/core/root_surface.md`，先把当前 root public surface 固化为可执行守卫：`lib.rs` 20 个 public module declarations、3 个 public `pub use` sites、禁止 `graphics`/`render_graph`/`rhi`/`ui`/`input`/`scene`/`asset`/`plugin` 等子系统在 crate root 扁平化公开；`core/mod.rs` 继续只允许 `runtime/framework/manager/math/resource` 五件 spine。当前 `pub(crate) use graphics::...` alias block 仍仅作为 crate-private M3 debt 记录，未执行 alias 删除；真正 M3 hard cutover 继续等待 render owner 稳定。`rustfmt --edition 2021 --check zircon_runtime\src\tests\runtime_absorption\root_surface.rs zircon_runtime\src\tests\runtime_absorption\mod.rs` 通过；冲突标记/尾随空白/锚点扫描与 scoped `git diff --check` 通过（仅 LF-to-CRLF warning）；Cargo/rustc 因 active lane 占用未启动，待通道清空后补跑。 |
| M3 | 3.2 类型别名清除 | pre_m3_type_alias_guard_static_passed_pending_render_owner | 2026-06-13 | 扩展 `runtime_absorption::root_surface` 与 `docs/zircon_runtime/core/root_surface.md`，把当前 `lib.rs` 中 `RendererFeatureReferenceListKind`、`GraphicsError`、`SceneRenderer`、`WgpuRenderFramework`、`ViewportFrame`、`HybridGiRuntimeProvider`、`VirtualGeometryRuntimeProvider`、`SolariRuntimeProvider` 等 M3.2 type alias debt 明确锁为 crate-private debt：允许临时存在但禁止升级为 public root `graphics` export。这仍属于 M3 `lib.rs` graphics alias 清理债务；actual type alias deletion not executed；真正删除 `pub(crate) use graphics::...` alias block 与调用方迁移仍等待 render owner 窗口。`rustfmt --edition 2021 --check zircon_runtime\src\tests\runtime_absorption\root_surface.rs zircon_runtime\src\tests\runtime_absorption\mod.rs` 通过；冲突标记/尾随空白/锚点扫描通过；scoped `git diff --check` 通过（仅 LF-to-CRLF warning）。standalone `rustc --edition 2021 --test zircon_runtime\src\tests\runtime_absorption\root_surface.rs` 初次暴露计划文档缺失旧锚点 `M3 `lib.rs` graphics alias`，补回锚点后重跑通过 4/4；未启动 full Cargo。 |
| M3 | root-surface M1 gate 审计同步 | code_static_pending_cargo | 2026-06-13 | `runtime_root_surface.py` 已将 `animation` 与 `navigation` 归类为 Runtime 14 判定的 `runtime-module-entry`；结构审计 JSON 当前为 `public_module_count=20`、`module_decision_count=20`、`unclassified_public_module_count=0`、`root_surface_migration_debt_count=3`、`crate_visible_graphics_reexport_count=80`。同步 `docs/engine-architecture/runtime-root-surface-m1.md` 的旧 75 符号债为 80，并新增 `runtime_absorption::root_surface::root_surface_m1_gate_matches_runtime_14_module_family_seats` 锁定 Runtime 14 root seats、0 未分类和 80 fan-out。Cargo 待 active lanes 清空后补跑。 |
| M4 | 4.1 标记规范与裁决 | 完成 | 2026-06-12 | `docs/engine-architecture/generated-code-boundary.md` 已刷新：真实文件头 `// @generated ...` 标记计数为 0，普通 `generated` 词根命中 42 个 Rust 文件并裁决为领域词/测试/export 模板源/守卫自身；结构审计 `generated_code_boundary` 初始为 `template_file_count=9`、`behavior_location_count=13`、`behavior_decision_count=5`、`unclassified_behavior_label_count=0`、`generated_boundary_migration_debt_count=5`、`m1_gate_status=migration-debt-present` |
| M4 | 4.2 行为迁回与守卫 | 代码完成，Cargo 测试待重跑 | 2026-06-12 | 新增 `zircon_runtime/src/tests/runtime_absorption/generated_code_guard.rs` 并接入 `runtime_absorption/mod.rs`；守卫覆盖 marker 格式、marked generated leaf-only、export-template 行为分类、adapter-only 状态、scan scope folder-backed、entry-template facade 委派，并新增 `export_plugin_selection_template_delegates_registration_execution_to_app_providers` 防止模板复活 `plugin_registration()`/`plugin_feature_registration()` 即时调用。新增 `zircon_app::entry::export_bootstrap` 手写 owner：生成 `main.rs`/平台 `lib.rs` 只调用 `zircon_app::bootstrap_export_runtime*` 和 `zircon_plugins::export_runtime_bootstrap_config()`；`plugin_selection_template.rs` 改为生成 `ExportRuntimePluginRegistrationProvider` / `ExportRuntimePluginFeatureRegistrationProvider` provider 表，由 app 手写层执行注册；provider 行已裁决为允许的 generated table adapter，直接注册调用保留为回归标签；`EntryRunner` 内部新增静态链接报告 + 原生动态报告合并路径。`rustfmt --edition 2021 --check` 通过；`rustc --edition 2021 --test zircon_runtime/src/tests/runtime_absorption/generated_code_guard.rs` 通过 7/7；`python -m py_compile` 覆盖 generated audit helper 通过；结构审计现为 `template_file_count=10`、`behavior_location_count=6`、`allowed_adapter_location_count=6`、`migration_debt_location_count=0`、`behavior_decision_count=3`、`unclassified_behavior_label_count=0`、`generated_boundary_migration_debt_count=0`、`m1_gate_status=classified-and-clear`。`cargo check -p zircon_app --lib --no-default-features --features core-min --locked` 曾在 export-bootstrap 初版通过（既有 warning）；provider 表调整后的前一次重跑被 render/graphics 漂移阻塞于 `zircon_runtime/src/scene/world/render.rs` 未解析 `render_mesh_stable_instance_key` / `render_mesh_transform_revision`，未进入 app 编译；本轮 adapter-only 审计收束未启动新 Cargo，因为机器上已有其他 Cargo/rustc 通道活跃。`cargo test -p zircon_app --lib export_bootstrap ...` 与 `cargo test -p zircon_runtime --lib export_build_plan ...` 此前均因 Windows 测试目标编译超过 300s 超时；M4 Cargo 测试阶段仍待干净编译窗口重跑 |
| 横切 | Cargo/render-owner pending gate | code_static_pending_cargo | 2026-06-13 | 新增 `runtime_absorption::plan_status::cargo_gates::runtime_02_core_spine_root_surface_cargo_gate_stays_visible_until_validation`，锁定 Runtime 02 在 core/root/generated/export_build_plan/app/editor/plugin 验证、default/lib-test 回归、M3 `lib.rs` graphics alias render-owner cutover 和 M4 generated/export Cargo 回归通过前保持 `in_progress`；同步 Runtime 02、本 runtime index P2/P8/子计划行、Runtime 05 M3.2、`docs/zircon_runtime/core/root_surface.md`、`docs/engine-architecture/generated-code-boundary.md` 与 M0 评审。 |
| 横切 | core/root/generated 结构镜像 | structure_audit_static_passed_cargo_pending | 2026-06-14 | 新增 `core_spine_root_generated_boundary` 并接入总结构审计，复用 `runtime_root_surface` 与 `generated_code_boundary` 数据聚合 Runtime 02 当前事实：core root entries 6/6（`framework/manager/math/mod.rs/resource/runtime`）、core public modules 5/5、retired core root entries 0、runtime root public modules 20/20、public `pub use` sites 3/3、crate-visible graphics alias debt 80/80、root-surface M1 gate `migration-debt-present`、generated export templates 10/10、generated behavior 6/6、generated allowed adapters 6/6、generated migration debt 0/0、generated-code M1 gate `classified-and-clear`、root_entries guard tests 13（baseline >=8）、root_surface guard tests 6/6、generated-code guard tests 7/7、missing anchors 0、`risks = []`。这仍是静态结构证据；M2 default/lib-test、M3 render-owner alias cutover、M4 generated/export/app/editor/plugin Cargo gates 继续等待干净验证窗口。 |
| 横切 | interface convergence root-surface 镜像 | root_surface_interface_mirror_static_passed_cargo_pending | 2026-06-14 | 新增 `runtime_absorption::root_surface::root_surface_interface_convergence_mirror_uses_current_audit_counts`，把 `runtime-interface-convergence.md` 的 root-surface 镜像同步到当前审计事实：20 public modules、3 public `pub use` sites、80 crate-visible graphics re-export symbols、direct `rhi_wgpu` backend exposure、M1 gate `migration-debt-present`，并拒绝退回旧 17-module / 75-symbol 文本。未执行 `lib.rs` alias 删除或 RHI 生产切换；Cargo/rustc 当前仍有 active lanes，本切片只完成 rustfmt/结构审计/静态锚点验证。 |
| 横切 | core/root/generated 镜像文档守卫 | mirror_docs_static_passed_cargo_pending | 2026-06-15 | 新增 `runtime_absorption::core_spine_root_generated::runtime_02_core_spine_root_generated_mirror_docs_match_structure_audit_counts`，把 `core_spine_root_generated_boundary` 的镜像事实固定到 Runtime 02、本总索引、`root_surface.md`、`generated-code-boundary.md`、M0 review 与 runtime-interface convergence：core root entries 6/6、core public modules 5/5、retired core root entries 0、runtime root public modules 20/20、public `pub use` sites 3/3、crate-visible graphics alias debt 80/80、root-surface M1 gate `migration-debt-present`、generated export templates 10/10、generated behavior 6/6、generated allowed adapters 6/6、generated migration debt 0/0、generated-code M1 gate `classified-and-clear`、root_entries guard tests 13、root_surface guard tests 6/6、generated-code guard tests 7/7、`guard_test_anchor_count = 21`、`missing_guard_test_anchors = []`、`mirror_docs_guard_present = true`、`risks = []`。验证：rustfmt check、Python py_compile、direct `core_spine_root_generated_boundary_audit`、aggregate `audit_runtime_structure.py --json` Runtime 02 assertions、standalone rustc 1/1、conflict/trailing scans 通过；`core/root/generated/export_build_plan/app/editor/plugin` Cargo gates 与 M3 render-owner alias cutover 仍 pending。 |
| 横切 | generated template count 审计同步 | structure_audit_static_passed_cargo_pending | 2026-06-14 | `source_template_build_plan.rs` 已进入 `export_build_plan` 模板扫描范围，`generated_code_boundary` 当前报告 `template_file_count=10`、`behavior_location_count=6`、`allowed_adapter_location_count=6`、`migration_debt_location_count=0`、`behavior_decision_count=3`、`m1_gate_status=classified-and-clear`；`core_spine_root_generated_boundary`、`runtime_absorption::core_spine_root_generated` 与 6 份镜像文档已同步到 generated export templates 10/10。验证：rustfmt check、Python py_compile、direct `core_spine_root_generated_boundary_audit`、aggregate `audit_runtime_structure.py --json` Runtime 02 assertions、standalone rustc 1/1、stale 9/9 scan、conflict/trailing scans 与 scoped `git diff --check` 通过（仅 LF-to-CRLF warnings）；包级 generated/export/app/editor/plugin Cargo gates 仍 pending。 |
| 横切 | guard-test anchors 审计同步 | mirror_docs_static_passed_cargo_pending | 2026-06-15 | `core_spine_root_generated_boundary` 与 `runtime_02_core_spine_root_generated_mirror_docs_match_structure_audit_counts` 现在统一锁定 root_entries、root_surface 与 generated_code_guard 三组 Runtime 02 守卫锚，当前 `guard_test_anchor_count = 21`、`missing_guard_test_anchors = []`；Runtime 02、本总索引、`root_surface.md`、`generated-code-boundary.md`、M0 review、runtime-interface convergence 与状态输出表守卫已同步。验证：rustfmt check、Python py_compile、direct `core_spine_root_generated_boundary_audit`、aggregate Runtime 02 + plan-status assertions、standalone core_spine_root_generated 1/1、standalone status-output 2/2；`core/root/generated/export_build_plan/app/editor/plugin` Cargo gates 与 M3 render-owner alias cutover 仍 pending。 |

基线数值（开工首日记录，完工时复核漂移）：

- core 根散件数基线：13（9 文件 + 4 目录；重核：`ls zircon_runtime/src/core/`）
- core 散件命中重扫：`config_store` 6 文件、`frame_clock` 6、`channel_util` 5、`types` 51、`error` 65、`event_bus` 20、`time` 8、`job_scheduler` 10、`tasks` 18、`state` 18、`modules` 10、`lifecycle` 52、`diagnostics` 34（2026-06-12 `zircon_runtime/src/**/*.rs` 扫描，含测试）
- lib.rs 行数基线：74；别名块：`:39-72`；调用别名的文件：8（重核命令见执行前检查清单）
- "generated" 词根文件数基线：42；真实文件头 `// @generated ...` 标记：0
- `cargo check -p zircon_runtime --lib --locked` 耗时基线：__（执行时填写）
- `cargo test -p zircon_runtime --lib core:: --locked` 通过数基线：__

## 风险与协调

- **强制前置**：M1 开始前重读 `.codex/sessions/20260604-1232-runtime-architecture-review.md` 最新状态——该会话正在做 root surface / 大文件债切片，归属矩阵必须与其口径合并而不是另起一套；generated 边界文档亦在其 touched 清单内。
- `state/`、`tasks/`、`core/runtime/diagnostics/` 可能被 10fps 或 render 会话的 worktree 改动触及；迁移切片前按文件 `git status --porcelain` 检查，脏文件先避让，**禁止回退其改动**。
- M2 波及 `zircon_app/src/entry/`（≥10 文件引用 `zircon_runtime::core`）与 `zircon_editor`（≥10 文件）；测试阶段必须双 crate 回归，且 `zircon_app::entry` 的源断言测试（活动会话 touched）若锁定 core 路径字面，需同切片更新。
- M3 别名清除横切 graphics——若 render 计划或 RHI 会话正在改 `graphics/runtime/render_framework/submit_frame_extract/**`（实测别名调用方所在地），先对齐再动，避免双写冲突。
- `prelude.rs` 与 `tests/prelude.rs` 是几乎所有散件的汇聚点：每个迁移切片都会触碰，提交粒度按切片而非按文件，避免 prelude 半成品状态跨提交。
