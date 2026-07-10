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
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_root_surface.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/runtime_root_surface_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/core_spine_root_generated_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/core_spine_root_generated_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/generated_code_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/generated_code_markdown.py
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
last_refined: 2026-07-10
---

# 02 core spine 与 root surface 收束

## 现状与证据（2026-06-12 重核）

- **散件形态矫正**：core 根散件不是裸公开模块，而是"私有 `mod` + `core/mod.rs` 根部精选 `pub use`"形态（`core/mod.rs:3-13` 全部 `mod xxx;` 私有声明，`:23-44` 逐件再导出）。调用方使用的是 `core::FrameClock`、`core::EventBus`、`core::ZirconError` 这类再导出名，而非 `core::frame_clock::` 全路径——迁移时改的是 `core/mod.rs` 的声明与 `pub use` 来源，调用方 `use` 行多数不变。
- **散件清单（core/ 根实测）**：私有文件 9 件——`channel_util.rs`、`config_store.rs`、`error.rs`、`event_bus.rs`(+`event_bus/` 子目录)、`frame_clock.rs`、`job_scheduler.rs`、`lifecycle.rs`、`time.rs`、`types.rs`；公开目录 4 件——`state/`、`tasks/`、`modules/`、`diagnostics/`。五件套 spine（`runtime/framework/manager/math/resource`）之外共 13 件待归属。
- **双形态误判矫正**：`event_bus.rs` + `event_bus/` 不是迁移债——`event_bus.rs:3-6` 声明 `mod failure/prune/publish/subscribe`，是 file-as-directory-owner 惯例（仓内通行），迁移时整体 `git mv` 两者即可。
- **调用面实测**（Grep 再导出名，zircon_runtime/src 内文件数）：`ConfigStore` 4、`FrameClock` 4、`recv_latest|spawn_named_thread|wait_for` 5（外部仅 `asset/facade/event.rs`、`asset/pipeline/worker_pool.rs`）、`ZirconError` 6、`RuntimeTimeClocks|RuntimeTimeAdvance` 7、`JobScheduler` 9、`EventBus|EngineEvent` ≥12、`TaskPool*` ≥12、`NextState|StateTransitionEvent` ≥12、`modules::` ≥12（builtin/runtime_modules 装配族）、`LifecycleState|StartupMode` ≥12（scene/script/navigation/animation 模块广用）、`core::diagnostics` ≥12（dynamic_api/diagnostic_log）。窄面（≤9）调用方集中在 `core/runtime/runtime.rs`、`core/runtime/state/core_runtime_state.rs`、`core/runtime/handle/*` 与 `prelude.rs`。
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
- 调用方迁移（实测，≤10 全列）：`prelude.rs`、`tests/prelude.rs`、`core/runtime/runtime.rs`、`core/runtime/state/core_runtime_state.rs`、`core/runtime/handle/core_handle.rs`、`core/runtime/handle/time.rs`、`asset/facade/event.rs`、`asset/pipeline/worker_pool.rs`、`asset/pipeline/manager/project_asset_manager/construction.rs`、`scene/ecs/schedule_parallel_executor.rs`。漏网枚举：Grep `core::(config_store|frame_clock|channel_util|error|time|job_scheduler)|ConfigStore|FrameClock|JobScheduler|ZirconError|recv_latest|spawn_named_thread|wait_for|RuntimeTimeClocks`。
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

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

- 迁入记录：[`02/2026-07-09-core-spine-and-root-surface-output-records.md`](02/2026-07-09-core-spine-and-root-surface-output-records.md)
