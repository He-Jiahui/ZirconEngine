---
related_code:
  - zircon_runtime/src/scene/ecs/system_stage.rs
  - zircon_runtime/src/scene/ecs/schedule.rs
  - zircon_runtime/src/scene/ecs/schedule_runner.rs
  - zircon_runtime/src/scene/ecs/schedule_parallel_executor.rs
  - zircon_runtime/src/scene/ecs/schedule_conflict_graph.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_runtime/src/core/runtime/time.rs
  - zircon_runtime/src/core/framework/time/clock.rs
  - zircon_runtime/src/core/framework/time/fixed_step_plan.rs
  - zircon_runtime/src/core/runtime/handle/time.rs
  - zircon_runtime/src/core/runtime/frame_clock.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime/src/dynamic_api/session/extract.rs
  - zircon_runtime/src/dynamic_api/runtime_loop.rs
  - zircon_runtime/src/scene/tests/ecs_schedule/fixed_update.rs
  - zircon_runtime/src/tests/runtime_absorption/schedule_frame_loop.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/schedule_frame_loop_boundary.py
  - docs/zircon_runtime/core/frame_schedule.md
  - dev/bevy/crates/bevy_app/src/main_schedule.rs
  - dev/Fyrox/fyrox-impl/src/engine/executor.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/index.md
status: in_progress
last_refined: 2026-06-20
---

# 03 调度与帧循环对齐

## 现状与证据（2026-06-12 重核）

旧文两条核心假设已过时，本次重核矫正如下：

- **阶段表已权威化（矫正）**："无 Bevy `MainScheduleOrder` 式单一权威阶段表"失实——`scene/ecs/system_stage.rs:4-45` 的 `SystemStage` 枚举即权威表：`First → PreUpdate → FixedFirst → FixedUpdate → FixedPostUpdate → Update → PostUpdate → Last → RenderExtract`，带 `ORDER: [Self; 9]`、`FIXED_LOOP`、`rank()` 与 `is_fixed_loop()`。
- **三时钟已对齐 bevy_time（矫正）**："fixed timestep 语义未定稿"半失实——`core/runtime/time.rs` 的 `RuntimeTimeClocks` 已实现 `Time<Real>/Time<Virtual>/Time<Fixed>` 三时钟（`core/framework/time/{clock.rs,fixed_step_plan.rs}`），含 `advance_by(real_delta, max_fixed_steps)`、`accumulate_overstep`、`drain_steps(max_steps) -> FixedStepPlan`（clock.rs:133-140）、虚拟时钟 pause/max_delta/relative_speed，以及 `TIME_FIXED_STEPS_DIAGNOSTIC` 等诊断常量。驱动点：`core/runtime/handle/time.rs:29` `advance_time_by(real_delta, max_fixed_steps)`；动态 session profile 的 owner 是 `dynamic_api/session.rs:359` `max_fixed_steps_per_frame()`，`:551` `tick_time(...)` 注入。
- **`FixedStepPlan` 调度消费与时间权威已收束到单次推进（代码完成，Cargo 待跑）**。当前 `WorldDriver::tick_level`（`scene/module/world_driver.rs:11-73`）在 `FixedFirst` 处按 `SystemStage::FIXED_LOOP` 循环运行 `FixedFirst/FixedUpdate/FixedPostUpdate`，且直接消费上游 `RuntimeTimeAdvance`。`dynamic_api/session.rs:548-553` 仍是动态帧路径唯一 `tick_time(...)` 调用点，`scene/level_system.rs:103-105` 只透传 `RuntimeTimeAdvance`，`WorldDriver` 已删除局部 `MAX_FIXED_STEPS_PER_FRAME = 4` 与二次 `advance_time_by(...)`。
- **帧循环归一审计已完成，UI extract 旁路已定稿为 runtime 03 合法旁路（Cargo 待跑）**。实测链路已写入 `docs/zircon_runtime/core/frame_schedule.md`：`dynamic_api/session.rs` `tick_frame`（C ABI 出口与 `RuntimeDynamicSession::tick_frame`）→ `tick_time` → `LevelSystem::tick` → `WorldDriver::tick_level` → 逐 stage `SceneScheduleRunner::run_stage`（`schedule_runner.rs`，步骤类型 `ScheduledSceneStepRef::{Internal, Native, ApplyDeferred, Hook}`）。runtime 侧 `RenderFrameExtract` 的生产构建点是 `dynamic_api/session/extract.rs` `current_extract()`；UI extract 旁路 `current_ui_extract()` + `session/hud.rs` + `session/menu.rs` + `runtime_loop.rs` 已裁决为"合法 dynamic-session side path"，并由 `session_ui_extract_remains_documented_dynamic_session_side_path` 源守卫锁定。
- **结构审计 owner 已补（静态通过，Cargo 待跑）**。`schedule_frame_loop_boundary` 现覆盖 Runtime 03 的 18 个调度/帧循环 source owner、8 个 guard/test owner、`SystemStage` 9 阶段/3 fixed-loop 阶段、动态 session 唯一 `.tick_time(...)` 调用、`WorldDriver` 无二次 `advance_time_by(...)`、UI extract 合法旁路、显式 stage ordering、schedule runner deferred/hook 行为、parallel executor 诊断锚点、14 个 Runtime 03 测试锚点、13 个 Runtime 03 行为测试锚点与 pending Cargo gate；当前镜像事实为 `behavior_test_anchor_count = 13`、`missing_behavior_test_anchors = []`、`doc_anchors = 10/10`；`runtime_03_schedule_frame_loop_mirror_docs_match_structure_audit_counts` 额外锁定 Runtime 03 计划、`frame_schedule.md`、总索引、M0 review 与 runtime-interface convergence 的镜像数字；当前定向审计报告 `mirror_docs_guard_present = true`、frame schedule module-doc anchors 3/3、`risks = []`。
- **stage 内排序核心已不是纯注册顺序，M1 负例守卫已补（Cargo 待跑）**。`SceneScheduleStagePlan::from_registry` 已做同 stage 拓扑排序，`SceneSystemDescriptor`/native metadata 已有 `order`、`before`、`after`；M0 盘点未发现 runtime-owned builtin 靠注册顺序表达依赖。`schedule_stage_plan_orders_steps_by_explicit_declaration_not_registration` 覆盖同 stage 约束在注册顺序打乱后仍稳定；既有 `plugin_system_constraints_order_registered_native_systems` 覆盖 plugin/native 反向注册顺序。
- **并行执行器已有实质测试，M3.1/M3.2 代码与文档已落地（Cargo 待跑）**：`scene/tests/ecs_schedule/conflict_graph.rs` 覆盖组件/资源/事件写冲突、disjoint query filter、跨 stage 独立与保守并行批次；`scene/tests/ecs_schedule/parallel_executor.rs` 覆盖 batch 经 `JobScheduler` 执行、失败上报、关闭并行回退、诊断计数、代表性批次收益与串并行终态一致性。`ScheduleParallelExecutor` 现有 `with_parallel_enabled(false)`、`run_batches_with_report(...)`、`ScheduleParallelExecutionReport` 与诊断常量 `schedule.parallel_batches` / `schedule.serial_fallbacks`；`representative_schedule_produces_multi_system_parallel_batches` 与 `parallel_and_serial_execution_reach_identical_world_state` 已补代表性 schedule 的批次收益和串并行终态一致性守卫。
- `FrameClock`（`core/runtime/frame_clock.rs`，仅 `tick() -> Duration`）是 real_delta 来源原语，与三时钟分工明确；其归属迁移见子计划 02。
- 参考锚点（每点一行）：Bevy `MainScheduleOrder.labels` + `insert_after` — `dev/bevy/crates/bevy_app/src/main_schedule.rs`；Bevy `bevy_time` Real/Virtual/Fixed 三时钟同形 — `dev/bevy/crates/bevy_time/src`；Fyrox `while lag >= fixed_time_step` 累积循环 — `dev/Fyrox/fyrox-impl/src/engine/executor.rs`。

补充参考锚点（2026-06-13 实测核验，实现型切片动工前先读——index 公约 §7.9）：

- bevy_time 三时钟具体实现（M2 插值因子/虚拟时钟语义对照）— `dev/bevy/crates/bevy_time/src/{fixed.rs,virt.rs,real.rs,time.rs}`
- bevy 调度执行器与 apply_deferred 自动插桩（M1 隐式顺序显式化、M3 并行对照）— `dev/bevy/crates/bevy_ecs/src/schedule/{executor/,graph/,auto_insert_apply_deferred.rs}`
- Godot 固定步 + 渲染插值同步的 C++ 工程实现（M2 对照第二实现，防单一参照偏差）— `dev/godot/main/main_timer_sync.{h,cpp}`；主循环骨架 — `dev/godot/core/os/main_loop.{h,cpp}`

## 目标

1. 帧循环单点权威文档化 + 旁路归一：从 `zircon_app` 入口到 stage 执行与 extract 提交的一帧链路有唯一权威图；UI extract 旁路要么并入 `RenderExtract` stage 语义、要么显式定稿为合法旁路并写明理由。
2. `RuntimeTimeAdvance` 单权威传入调度侧：固定步阶段按 drained steps 循环执行、零 lag 帧跳过、上限截断、剩余 overstep 暴露为插值因子，且一帧只推进一次 runtime time。
3. 并行执行从"有测试"到"可观测"：开关 + 诊断计数 + 串并行一致性证据。

## 非目标

- 不重写 ECS 存储/查询层；不动 render graph 内部 pass 调度（归 render 计划 01-08）。
- 不引入 async 调度框架；沿用现有线程模型（`JobScheduler` + 自研 executor）。
- 不改 `SystemStage` 枚举形状（已是权威表；若审计发现需要新阶段，先回本计划补裁决再动）。
- 三时钟内部语义（clock.rs）已对齐 bevy_time，不重构。

### 全局硬约束（继承总计划 §4，违反即返工）

- 不新增 crate；硬切换不留兼容层；渲染骨架内容归 render 计划 01-08。
- 动态边界（dynamic_api）只传 ABI-safe 值；非网络语义 server 命名是 blocker。

## 执行前检查清单

1. 子计划 02 的 core 散件归属已完成物理 cutover：`frame_clock.rs`、`time.rs` 当前最终路径为 `core/runtime/frame_clock.rs` 与 `core/runtime/time.rs`。
2. 活动会话对齐：`dynamic_api/runtime_loop.rs`、`session.rs` 被 wgpu 渲染主链与 10fps 会话触及过——`git status --porcelain -- zircon_runtime/src/dynamic_api/ zircon_runtime/src/scene/ecs/ zircon_runtime/src/scene/module/`，脏文件避让，禁止回退。
3. 事实重核：
   - `grep -n "ORDER\|rank" zircon_runtime/src/scene/ecs/system_stage.rs`
   - `grep -n "advance_by\|drain_steps\|max_fixed_steps" zircon_runtime/src/core/runtime/time.rs zircon_runtime/src/core/runtime/handle/time.rs zircon_runtime/src/dynamic_api/session.rs`
   - `grep -rn "FixedStepPlan" zircon_runtime/src --include=*.rs`（核调度消费方是否仍为单权威传入）
4. 基线记录：`cargo test -p zircon_runtime --lib ecs_schedule --locked` 与 `--lib time --locked` 通过数，记入状态节。

## 里程碑

### M0 帧循环审计（先证据后设计）

#### 切片 0.1 一帧权威链路图

- 目标文件：`docs/zircon_runtime/core/frame_schedule.md`（新建）。
- 改动形态：纯文档。画出实测链路并补全本计划未覆盖段：`zircon_app` 入口 → `dynamic_api/session.rs::tick_frame`（:301/:548）→ `tick_time`（三时钟推进 + 诊断发布）→ `WorldDriver::tick_level` → 逐 stage `SceneScheduleRunner::run_stage`（四类步骤 + apply_deferred 语义）→ `RenderExtract` stage 与 graphics submit 的衔接点；每段标 owner 文件:行。
- 调用方迁移：无。
- 验收：图中每条边都有源码引用；`runtime_loop.rs` 的角色（与 session.tick_frame 的分工）写清。
- DoD：`frame_schedule.md` 落地且链路图无"未知段"。

#### 切片 0.2 extract 旁路与隐式顺序盘点

- 目标文件：`docs/zircon_runtime/core/frame_schedule.md`（追加节）。
- 改动形态：盘点两类清单——(a) `UiRenderExtract` 旁路：`session.rs:703 current_ui_extract`、`session/hud.rs`、`session/menu.rs`、`runtime_loop.rs:50,:98` 四处的生成/消费关系，与 `RenderExtract` stage 的关系裁决（归一或定稿旁路）；(b) 隐式顺序依赖：盘点 builtin 模块注册的系统/步骤（枚举命令：Grep `iter_sorted_for_stage|SceneSystemDescriptor|ScheduledSceneStep`，path `zircon_runtime/src`）中靠注册顺序而非 stage 表达"必须在 X 之后"的位置。
- 调用方迁移：无。
- 验收：旁路裁决有判词（归一/合法旁路二选一）；隐式依赖清单逐条标注"接受（同 stage 内有序）/ 违规（需显式化）"。
- DoD：两清单落 `frame_schedule.md`，违规清单进 M1 工作集。

#### M0 测试阶段（milestone-first）

- 纯审计：`git status --porcelain` 仅 docs 变更。
- 验收证据：帧序图 + 两清单。

### M1 帧循环归一（按 M0 裁决裁剪）

#### 切片 1.1 隐式顺序显式化

- 目标文件：按 M0 违规清单定（预期落点：`scene/ecs/schedule.rs` 的 stage plan 构建、builtin 模块注册点）。
- 改动形态：把"靠注册顺序"的依赖改为 stage 内显式排序声明（沿用 `iter_sorted_for_stage` 既有排序键扩展，不引入新框架）；被替代的隐式约定注释/常量同切片删除（硬切换）。
- 调用方迁移：按违规清单逐项（M0 产出，预计 ≤10 项全列）。
- 验收：`schedule_stage_plan_orders_steps_by_explicit_declaration_not_registration`（归属 `zircon_runtime/src/scene/tests/ecs_schedule/`，新文件或并入既有 `conflict_graph.rs` 同级）——打乱注册顺序断言执行序不变。
- DoD：违规清单清零；现有 `ecs_schedule` 测试族无回归。

#### 切片 1.2 UI extract 旁路契约（M0 判"合法旁路"）

- 目标文件：`dynamic_api/session.rs`、`dynamic_api/session/{extract.rs,hud.rs,menu.rs}`、`dynamic_api/runtime_loop.rs`。
- 改动形态：M0 已判"合法旁路"，本切片退化为在 `frame_schedule.md` 写明旁路契约，并用 dynamic_api 源守卫固定 capture/present 消费点与 menu-then-HUD 生产顺序。
- 调用方迁移：`runtime_loop.rs:50,:98` 两处签名消费点（实测全列）。
- 验收：`session_ui_extract_remains_documented_dynamic_session_side_path`（dynamic_api tests，参照既有 `dynamic_api/tests/session_lifecycle.rs` 的源断言风格）。
- DoD：extract 生产点定稿为合法旁路，`frame_schedule.md` 与代码一致。

#### M1 测试阶段（milestone-first）

- `cargo check -p zircon_runtime --lib --locked`（切片期）
- `cargo test -p zircon_runtime --lib ecs_schedule --locked -- --nocapture`
- `cargo test -p zircon_runtime --lib session --locked`（dynamic_api 受影响）
- `cargo test -p zircon_app --locked`（宿主循环受影响时）
- 验收证据：阶段表单点定义维持 + 新增负例测试通过。

### M2 RuntimeTimeAdvance 单权威接通固定步调度

#### 切片 2.1 调度侧改为消费单次时间推进计划

- 目标文件：`scene/module/world_driver.rs`、`scene/level_system.rs`、`dynamic_api/session.rs`（把 `tick_time` 产出的 `RuntimeTimeAdvance` 传入 tick_level，并删除 `WorldDriver` 的二次 `advance_time_by`）。
- 改动形态（签名草案，执行时定稿）：

  ```rust
  // world_driver.rs
  pub fn tick_level(&self, core: &CoreHandle, level: &LevelSystem,
                    advance: RuntimeTimeAdvance) -> Result<(), CoreError>
  // 内部：非 fixed-loop 阶段用 real/virtual delta 跑一次；
  // FixedFirst/FixedUpdate/FixedPostUpdate for _ in 0..advance.fixed_step_plan().step_count
  // 以固定 timestep 跑 N 次；step_count == 0 时整体跳过 fixed loop。
  ```

- 调用方迁移（2026-06-12 落地）：`tick_level` 生产调用方**仅 1 处**——`scene/level_system.rs:103`（`driver.tick_level(core, self, advance)`），上行链经 `dynamic_api/session.rs` 的 `tick_frame`；签名改动一次闭合。旧 `delta_seconds: Real` 参数语义与 `WorldDriver::MAX_FIXED_STEPS_PER_FRAME` 同切片删除，不留双签名/双 cap。
- 验收（归属 `zircon_runtime/src/scene/tests/ecs_schedule/` 新文件 `fixed_update.rs`）：
  - `level_tick_repeats_fixed_loop_stages_for_drained_fixed_steps`（lag 累积 3 步 → FixedUpdate hook 执行 3 次）
  - `level_tick_skips_fixed_loop_stages_when_no_fixed_steps_are_drained`
  - `level_tick_fixed_loop_steps_are_capped_by_runtime_time_advance`（对照 `RuntimeTimeAdvance` 的 fixed step cap）
- DoD：三测试绿；`TIME_FIXED_STEPS_DIAGNOSTIC` 数值与实际执行次数一致（测试断言）。

#### 切片 2.2 插值因子暴露

- 目标文件：`core/framework/time/fixed_step_plan.rs`。
- 改动形态：`FixedStepPlan` 暴露 `overstep_fraction() -> f32`（剩余 lag / timestep，clamp 到 `[0.0, 1.0]`）；经 time 服务读口供渲染插值消费（消费方接入归 render 计划，本计划只提供读口）。
- 调用方迁移：无强制迁移（新增只读口）。
- 验收：`fixed_step_plan_reports_overstep_fraction_in_unit_range`（`zircon_runtime/src/tests/time.rs`）。
- DoD：读口测试绿；`frame_schedule.md` 补固定步时序图。

#### M2 测试阶段（milestone-first）

- `cargo test -p zircon_runtime --lib fixed_update --locked -- --nocapture`
- `cargo test -p zircon_runtime --lib time --locked`（三时钟无回归）
- `cargo test -p zircon_runtime --lib ecs_schedule --locked`
- 验收证据：步长守恒/截断/跳过三测试 + 帧序图更新。

### M3 并行执行可观测化

#### 切片 3.1 开关与诊断计数

- 目标文件：`scene/ecs/schedule_parallel_executor.rs`；计数走 `core::diagnostics` 既有通道（不引新依赖）。
- 改动形态：并行执行加可关闭开关（owner 执行时定稿：profile 或 config_store，与子计划 02 的 config 归属一致）；诊断计数（签名草案）：`schedule.parallel_batches`（每帧并行批次数）、`schedule.serial_fallbacks`（串行回退次数）。
- 调用方迁移：无公共面变化（executor 内部 + 诊断登记点）。
- 验收：`schedule_parallel_executor_can_run_parallel_batches_serially_with_report`、`schedule_parallel_execution_report_records_diagnostic_counts`（归属 `scene/tests/ecs_schedule/parallel_executor.rs`）；结构守卫：`schedule_parallel_report_keeps_run_batches_compatible`、`schedule_parallel_disabled_path_runs_serial_batches_with_fallback_counts`。
- DoD：计数经诊断通道可读，开关关闭时回退串行且报告一致；Cargo 回归待共享构建通道空闲后执行。

#### 切片 3.2 串并行一致性与收益证据

- 目标文件：`scene/tests/ecs_schedule/`（新测试）。
- 改动形态：纯测试。代表性 schedule（多读单写混合，复用既有 conflict_graph 测试夹具）下断言：并行批次 > 1；并行与串行执行的 world 终态一致。
- 调用方迁移：无。
- 验收：`representative_schedule_produces_multi_system_parallel_batches`、`parallel_and_serial_execution_reach_identical_world_state`。
- DoD：两测试已落地并写入批次计数基线；Cargo 回归待共享构建通道空闲后执行。

#### M3 测试阶段（milestone-first）

- `cargo test -p zircon_runtime --lib ecs_schedule --locked -- --nocapture`（含既有 11 个 conflict_graph/executor 测试无回归）
- `cargo test -p zircon_runtime --lib schedule_parallel --locked -- --nocapture`
- 验收证据：一致性测试 + 批次计数断言；并行语义与限制写入 `frame_schedule.md`。

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据（命令输出 / 文件 / 测试名） |
|---|---|---|---|---|
| M0 | 0.1 帧链路图 | 完成（纯文档，代码未改） | 2026-06-12 | `docs/zircon_runtime/core/frame_schedule.md` |
| M0 | 0.2 旁路与隐式顺序盘点 | 完成 | 2026-06-12 | `docs/zircon_runtime/core/frame_schedule.md`；M1 守卫见下 |
| M1 | 1.1 隐式顺序显式化 | 代码完成，Cargo 待跑 | 2026-06-12 | `schedule_stage_plan_orders_steps_by_explicit_declaration_not_registration`；既有 `plugin_system_constraints_order_registered_native_systems` |
| M1 | 1.2 UI extract 合法旁路契约 | 文档/源守卫完成，Cargo 待跑 | 2026-06-12 | `docs/zircon_runtime/core/frame_schedule.md`；`session_ui_extract_remains_documented_dynamic_session_side_path` |
| M2 | 2.1 单次 `RuntimeTimeAdvance` 接通 | 代码完成，Cargo 待跑 | 2026-06-12 | `dynamic_api/session.rs`、`scene/level_system.rs`、`scene/module/world_driver.rs`；测试：`world_driver_consumes_runtime_time_advance_without_advancing_clocks_again`、`level_tick_repeats_fixed_loop_stages_for_drained_fixed_steps`、`level_tick_skips_fixed_loop_stages_when_no_fixed_steps_are_drained`、`level_tick_fixed_loop_steps_are_capped_by_runtime_time_advance` |
| M2 | 2.2 插值因子 | 代码完成，Cargo 待跑 | 2026-06-12 | `core/framework/time/fixed_step_plan.rs`；测试：`fixed_step_plan_reports_overstep_fraction_in_unit_range` |
| M3 | 3.1 开关与计数 | 代码完成，Cargo 待跑 | 2026-06-12 | `ScheduleParallelExecutor::with_parallel_enabled(false)`、`run_batches_with_report(...)`、`ScheduleParallelExecutionReport::record_diagnostics(...)`；测试：`schedule_parallel_executor_can_run_parallel_batches_serially_with_report`、`schedule_parallel_execution_report_records_diagnostic_counts`、`schedule_parallel_report_keeps_run_batches_compatible`、`schedule_parallel_disabled_path_runs_serial_batches_with_fallback_counts`；文档：`docs/zircon_runtime/scene/ecs/schedule_parallel_executor.md` |
| M3 | 3.2 一致性与收益 | 代码完成，Cargo 待跑 | 2026-06-12 | `representative_schedule_produces_multi_system_parallel_batches`、`parallel_and_serial_execution_reach_identical_world_state` |
| 横切 | Schedule/frame-loop 结构审计 owner | 静态通过，Cargo 待跑 | 2026-06-13 | `schedule_frame_loop_boundary`: source files 18/18，guard/test files 8/8，`SystemStage` count and variants 9/9，fixed-loop stages 3/3，dynamic-session `.tick_time(...)` calls 1/1，Runtime 03 guard anchors 14/14，no `WorldDriver` second `advance_time_by(...)` references，no dynamic-session raw-delta level tick references，`risks = []` |
| 横切 | Schedule/frame-loop 镜像文档守卫 | mirror_docs_static_passed_cargo_pending | 2026-06-14 | 新增 `runtime_absorption::schedule_frame_loop::runtime_03_schedule_frame_loop_mirror_docs_match_structure_audit_counts`，锁定 Runtime 03 计划、runtime index、M0 review 与 runtime-interface convergence 必须同步 `schedule_frame_loop_boundary` 的 source files 18/18、guard/test files 8/8、`SystemStage` count and variants 9/9、fixed-loop stages 3/3、dynamic-session `.tick_time(...)` calls 1/1、Runtime 03 guard anchors 14/14、no `WorldDriver` second `advance_time_by(...)` references、no dynamic-session raw-delta level tick references 与 `risks = []`。未改调度/帧循环生产代码；Cargo/rustc 仍待 active lanes 清空。 |
| 横切 | Schedule/frame-loop 总索引状态表闭环 | mirror_docs_static_passed_cargo_pending | 2026-06-14 | 本轮把 `Runtime 03 Schedule/frame-loop 镜像文档守卫` 写入 runtime 总索引 `## 状态与产出记录`，并扩展 `runtime_absorption::plan_status::status_output_tables::runtime_index_status_output_records_recent_cross_plan_slices`，要求总索引记录 `runtime_03_schedule_frame_loop_mirror_docs_match_structure_audit_counts`、`schedule_frame_loop_boundary`、standalone rustc 1/1 与 `ecs_schedule/time/session/schedule_parallel Cargo gates pending`。验证：`rustfmt --edition 2021 --check` 通过；`runtime_03_schedule_frame_loop_mirror_docs_match_structure_audit_counts` standalone rustc 1/1 通过；状态表 harness 1/1 通过；Python direct `schedule_frame_loop_boundary_audit` 与 aggregate Runtime 03 assertions 通过；conflict/trailing scans 通过。 |
| 横切 | Schedule/frame-loop module-doc 镜像元数据 | mirror_docs_static_passed_cargo_pending | 2026-06-14 | `schedule_frame_loop_boundary` 新增 `mirror_docs_guard_present = true` 与 frame schedule module-doc anchors 3/3，`runtime_absorption::schedule_frame_loop::runtime_03_schedule_frame_loop_mirror_docs_match_structure_audit_counts` 现在同时锁定 `docs/zircon_runtime/core/frame_schedule.md` 的 guard/test files 8/8、Runtime 03 guard anchors 14/14 与 `runtime_03_schedule_frame_loop_mirror_docs_match_structure_audit_counts`；本切片未改调度/帧循环生产代码，`ecs_schedule/time/session/schedule_parallel` Cargo gates 仍 pending。 |
| 横切 | Schedule/frame-loop 行为测试锚审计同步 | mirror_docs_static_passed_cargo_pending | 2026-06-15 | `schedule_frame_loop_boundary` 现在把 Runtime 03 M1/M2/M3 的 13 个调度/帧循环行为测试锚从 14 项 guard/test 总锚点中拆出单独计数，当前 `behavior_test_anchor_count = 13`、`missing_behavior_test_anchors = []`、Runtime 03 guard anchors 14/14 与 `doc_anchors = 10/10`；`runtime_absorption::schedule_frame_loop::runtime_03_schedule_frame_loop_mirror_docs_match_structure_audit_counts` 要求本计划、runtime index、`docs/zircon_runtime/core/frame_schedule.md`、M0 review 与 runtime-interface convergence 都记录同一组行为锚事实。验证：rustfmt check、Python py_compile、direct `schedule_frame_loop_boundary_audit`、aggregate Runtime 03 + plan-status assertions、standalone schedule_frame_loop 1/1、standalone status-output 2/2；ecs_schedule/time/session/schedule_parallel Cargo gates 仍 pending。 |
| 横切 | Schedule/frame-loop world bootstrap fixed-loop stage guard sync | guard_sync_static_passed_cargo_pending | 2026-06-15 | `world_bootstraps_with_renderable_defaults` 的 stage 断言已同步为 `SystemStage::First`、`SystemStage::PreUpdate`、`SystemStage::FixedFirst`、`SystemStage::FixedUpdate`、`SystemStage::FixedPostUpdate`、`SystemStage::Update`、`SystemStage::PostUpdate`、`SystemStage::Last`、`SystemStage::RenderExtract`，匹配 `SystemStage::ORDER` 与 Runtime 03 九阶段权威表；这是 full `scene::` closeout 31 失败中的 world_basics stale guard 修复，生产调度代码未改。验证：rustfmt check、Python py_compile、direct `runtime_plan_status_boundary_audit` support 27/27 risks=[]、wrapped standalone plan-status 30/30、conflict/trailing scans 与 scoped diff check 通过（仅 LF/CRLF warnings）；Cargo 包级复验因 active editor/export lanes 暂缓。 |
| 横切 | Schedule/frame-loop current audit recheck | schedule_frame_loop_current_audit_static_passed_cargo_pending | 2026-06-20 | 本轮只复核 Runtime 03 当前调度/帧循环结构事实，生产代码未改：`schedule_frame_loop_boundary_audit` 报告 source files 18/18、guard/test files 8/8、`SystemStage` count and variants 9/9、fixed-loop stages 3/3、dynamic-session `.tick_time(...)` calls 1/1、Runtime 03 guard anchors 14/14、`behavior_test_anchor_count = 13`、`missing_behavior_test_anchors = []`、`doc_anchors = 10/10`、frame schedule module-doc anchors 3/3、`mirror_docs_guard_present = true`、no `WorldDriver` second `advance_time_by(...)` references、no dynamic-session raw-delta level tick references、`risks = []`。验证通过：Python py_compile、direct `schedule_frame_loop_boundary_audit` risks=[]、standalone `schedule_frame_loop.rs` 1/1、standalone `plan_status.rs` 32/32；`ecs_schedule/time/session/schedule_parallel` Cargo gates 仍 pending。 |

Cargo validation note（2026-06-12）：

- Attempted `cargo test -p zircon_runtime --lib ecs_schedule --locked --target-dir E:/cargo-targets/zircon-runtime-03-0612 -- --nocapture --test-threads=1`.
- The run failed before executing runtime 03 tests on unrelated UI compile drift: `zircon_runtime/src/ui/tests/asset_dependency_index.rs` imports missing `crate::asset::ui_v2_asset_references`.
- 2026-06-13 新增 `runtime_absorption::plan_status::cargo_gates::runtime_03_schedule_frame_loop_cargo_gate_stays_visible_until_schedule_validation`，把 Runtime 03 继续锁在 `in_progress`：`ecs_schedule/time/session/schedule_parallel` Cargo gates 通过前，M1/M2/M3 行必须保留 `Cargo 待跑`，并继续暴露 `schedule_stage_plan_orders_steps_by_explicit_declaration_not_registration`、`session_ui_extract_remains_documented_dynamic_session_side_path`、`world_driver_consumes_runtime_time_advance_without_advancing_clocks_again`、`level_tick_repeats_fixed_loop_stages_for_drained_fixed_steps`、`level_tick_skips_fixed_loop_stages_when_no_fixed_steps_are_drained`、`level_tick_fixed_loop_steps_are_capped_by_runtime_time_advance`、`fixed_step_plan_reports_overstep_fraction_in_unit_range`、`schedule_parallel_executor_can_run_parallel_batches_serially_with_report`、`schedule_parallel_execution_report_records_diagnostic_counts`、`representative_schedule_produces_multi_system_parallel_batches`、`parallel_and_serial_execution_reach_identical_world_state` 等验证锚点。

基线数值（开工首日记录）：

- `SystemStage` 阶段数基线：9（重核：system_stage.rs `COUNT`）
- `FixedStepPlan` 调度侧消费点基线：1 个消费 owner（`WorldDriver`），当前目标形态为 `RuntimeTimeAdvance` 单权威传入；二次 `advance_time_by` 已删除，Cargo 待跑。
- `schedule_frame_loop_boundary` 静态审计基线：source files 18/18，guard/test files 8/8，`SystemStage` count and variants 9/9，fixed-loop stages 3/3，dynamic-session `.tick_time(...)` calls 1/1，Runtime 03 guard anchors 14/14，`behavior_test_anchor_count = 13`，`missing_behavior_test_anchors = []`，`doc_anchors = 10/10`，`mirror_docs_guard_present = true`，frame schedule module-doc anchors 3/3，no `WorldDriver` second `advance_time_by(...)` references，no dynamic-session raw-delta level tick references，`risks = []`。
- `ecs_schedule` 冲突图基线：`conflict_graph.rs` 保留冲突检测与保守批次构造职责；执行器行为基线迁入 `parallel_executor.rs`，避免单文件重新越过 1000 行。
- 并行批次计数基线：M3.1 夹具默认并行执行为 `parallel_batches = 1`、`serial_batches = 1`、`serial_fallbacks = 0`；关闭并行后为 `parallel_batches = 0`、`serial_batches = 2`、`serial_fallbacks = 1`。M3.2 代表性 schedule 产出 3 个双系统 batch；默认并行报告为 `parallel_batches = 3`、`serial_batches = 0`、`serial_fallbacks = 0`、`executed_systems = 6`；关闭并行后为 `parallel_batches = 0`、`serial_batches = 3`、`serial_fallbacks = 3`、`executed_systems = 6`，并断言两条路径终态一致。

## 风险与协调

- 依赖子计划 02：`frame_clock.rs`/`time.rs` 迁移后路径变化，本计划所有 core 路径引用以 02 落地为准；02 未动工时按当前路径执行、在状态节标注。
- `dynamic_api/{runtime_loop.rs,session.rs}` 是 wgpu 渲染主链与 10fps 会话（`20260611-0416`）的触及区：M1/M2 切片前逐文件 `git status` 检查，脏文件先避让，**禁止回退其改动**；`tick_frame` 行号（:301/:548/:551）漂移时以重核为准。
- M2 改 `tick_level` 签名波及 scene/module 与 session 装配；与 `20260604-1232` 会话的 touched_modules 对齐后再动。
- FixedUpdate 接通后首个消费者（物理）取决于子计划 01 M3 的物理选型决策；本计划只保证阶段语义与挂接位就绪，不实现任何物理步进。
