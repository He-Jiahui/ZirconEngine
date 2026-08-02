---
related_code:
  - zircon_runtime/src/core/framework/scene/system_stage.rs
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
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/dynamic_api/session/ffi.rs
  - zircon_runtime/src/dynamic_api/session/profile.rs
  - zircon_runtime/src/dynamic_api/session/extract.rs
  - zircon_runtime/src/dynamic_api/runtime_loop.rs
  - zircon_runtime/src/scene/tests/ecs_schedule/fixed_update.rs
  - zircon_runtime/src/scene/tests/ecs_schedule/schedule_plan.rs
  - zircon_runtime/src/scene/tests/ecs_schedule/world_driver.rs
  - zircon_runtime/src/tests/runtime_absorption/schedule_frame_loop.rs
  - zircon_runtime/src/tests/runtime_absorption/schedule_frame_loop/mirror_docs.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/schedule_frame_loop_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/schedule_frame_loop_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/schedule_frame_loop_anchor_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/schedule_frame_loop_markdown.py
  - tools/tests/test_runtime_schedule_frame_loop_audit.py
  - docs/zircon_runtime/core/frame_schedule.md
  - dev/bevy/crates/bevy_app/src/main_schedule.rs
  - dev/Fyrox/fyrox-impl/src/engine/executor.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/index.md
status: completed
last_refined: 2026-08-01
---

# 03 调度与帧循环对齐

## 现状与证据（2026-06-12 重核）

旧文两条核心假设已过时，本次重核矫正如下：

- **阶段表已权威化（矫正）**："无 Bevy `MainScheduleOrder` 式单一权威阶段表"失实——`core/framework/scene/system_stage.rs:5-50` 的 `SystemStage` 枚举即权威表：`First → PreUpdate → FixedFirst → FixedUpdate → FixedPostUpdate → Update → PostUpdate → Last → RenderExtract`，带 `ORDER: [Self; 9]`（:19-29）、`FIXED_LOOP`（:30）、`rank()`（:32-44）与 `is_fixed_loop()`（:46-50）。
- **三时钟已对齐 bevy_time（矫正）**："fixed timestep 语义未定稿"半失实——`core/runtime/time.rs` 的 `RuntimeTimeClocks` 已实现 `Time<Real>/Time<Virtual>/Time<Fixed>` 三时钟（`core/framework/time/{clock.rs,fixed_step_plan.rs}`），含 `advance_by(real_delta, max_fixed_steps)`、`accumulate_overstep`、`drain_steps(max_steps) -> FixedStepPlan`（clock.rs:133-140）、虚拟时钟 pause/max_delta/relative_speed，以及 `TIME_FIXED_STEPS_DIAGNOSTIC` 等诊断常量。驱动点：`core/runtime/handle/time.rs:29` `advance_time_by(real_delta, max_fixed_steps)`；动态 session profile 的 owner 是 `dynamic_api/session/profile.rs` 的 `DEFAULT_DYNAMIC_RUNTIME_MAX_FIXED_STEPS_PER_FRAME` 与 `max_fixed_steps_per_frame()`，`dynamic_api/session.rs` 仅在 `tick_frame` 注入 `tick_time(...)`。
- **`FixedStepPlan` 调度消费与时间权威已收束到单次推进（已验证）**。当前 `WorldDriver::tick_level`（`scene/module/world_driver.rs:84-134`）在 `FixedFirst` 处按 `SystemStage::FIXED_LOOP` 循环运行 `FixedFirst/FixedUpdate/FixedPostUpdate`，且直接消费上游 `RuntimeTimeAdvance`。`dynamic_api/session/state.rs:134-138` 是动态帧路径唯一 `tick_time(...)` 调用点，`scene/level_system.rs` 只透传 `RuntimeTimeAdvance`，`WorldDriver` 已删除局部 `MAX_FIXED_STEPS_PER_FRAME = 4` 与二次 `advance_time_by(...)`；当前 `ecs_schedule` 77/77 与 `tests::time::` 4/4 已通过。
- **帧循环归一审计已完成，UI extract 旁路已定稿为 runtime 03 合法旁路（已验证）**。实测链路已写入 `docs/zircon_runtime/core/frame_schedule.md`：`dynamic_api/session/state.rs:134` `tick_frame` → `tick_time` → `LevelSystem::tick` → `WorldDriver::tick_level` → 逐 stage `SceneScheduleRunner::run_stage`；C ABI 转接位于 `dynamic_api/session/ffi.rs:230`，导出位于 `dynamic_api/exports.rs:143`。runtime 侧 `RenderFrameExtract` 的生产构建点是 `dynamic_api/session/extract.rs` `current_extract()`；UI extract 旁路 `current_ui_extract()` + `session/hud.rs` + `session/menu.rs` + `runtime_loop.rs` 已裁决为"合法 dynamic-session side path"，并由 `session_ui_extract_remains_documented_dynamic_session_side_path` 源守卫锁定；当前 `session` 为 165 passed / 0 failed / 10 ignored。
- **结构审计 owner 已补并拆出清单与渲染 owner（静态与动态门槛均通过）**。`schedule_frame_loop_source_inventory.py` 现拥有 Runtime 03 的 19 个调度/帧循环 source owner、11 个 guard/test owner（含 folder-backed `schedule_plan.rs`、`world_driver.rs` 与 `schedule_frame_loop/mirror_docs.rs`）、`SystemStage` 计数、fixed-loop 计数与动态 session `.tick_time(...)` 计数；`schedule_frame_loop_anchor_inventory.py` 拥有 `SystemStage`、`RuntimeTimeAdvance`、`FixedStepPlan`、UI extract、显式 stage ordering、schedule runner、parallel executor、行为测试、镜像文档与 Cargo gate 锚点，并使用精确 `tests::time::` 过滤器；`schedule_frame_loop_boundary.py` 保留审计读取、缺失锚点与风险判定，当前 368 行；`schedule_frame_loop_markdown.py` 拥有 Markdown 渲染，当前 146 行。当前镜像事实为 source files 19/19、guard/test files 11/11、`SystemStage` count and variants 9/9、fixed-loop stages 3/3、dynamic-session `.tick_time(...)` calls 1/1、Runtime 03 guard anchors 14/14、`behavior_test_anchor_count = 13`、`missing_behavior_test_anchors = []`、`doc_anchors = 10/10`、`mirror_docs_guard_present = true`、frame schedule module-doc anchors 3/3、no `WorldDriver` second `advance_time_by(...)` references、no dynamic-session raw-delta level tick references、`risks = []`；`runtime_03_schedule_frame_loop_mirror_docs_match_structure_audit_counts` 额外锁定 Runtime 03 计划、`frame_schedule.md`、总索引、M0 review 与 runtime-interface convergence 的镜像数字。2026-07-14 当前静态回归 3/3、独立 schedule/frame-loop 守卫 2/2 已通过。
- **stage 内排序核心已不是纯注册顺序，M1 负例守卫已通过**。`SceneScheduleStagePlan::from_registry` 已做同 stage 拓扑排序，`SceneSystemDescriptor`/native metadata 已有 `order`、`before`、`after`；M0 盘点未发现 runtime-owned builtin 靠注册顺序表达依赖。`schedule_stage_plan_orders_steps_by_explicit_declaration_not_registration` 覆盖同 stage 约束在注册顺序打乱后仍稳定；既有 `plugin_system_constraints_order_registered_native_systems` 覆盖 plugin/native 反向注册顺序。
- **并行执行器已有实质测试，M3.1/M3.2 代码、文档与验证均已闭环**：`scene/tests/ecs_schedule/conflict_graph.rs` 覆盖组件/资源/事件写冲突、disjoint query filter、跨 stage 独立与保守并行批次；`scene/tests/ecs_schedule/parallel_executor.rs` 覆盖 batch 经 `JobScheduler` 执行、失败上报、关闭并行回退、诊断计数、代表性批次收益与串并行终态一致性。`ScheduleParallelExecutor` 现有 `with_parallel_enabled(false)`、`run_batches_with_report(...)`、`ScheduleParallelExecutionReport` 与诊断常量 `schedule.parallel_batches` / `schedule.serial_fallbacks`；`representative_schedule_produces_multi_system_parallel_batches` 与 `parallel_and_serial_execution_reach_identical_world_state` 已补代表性 schedule 的批次收益和串并行终态一致性守卫；当前 `schedule_parallel` 15/15 已通过。
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
   - `Select-String -Path zircon_runtime/src/core/framework/scene/system_stage.rs -Pattern 'ORDER','rank'`
   - `Select-String -Path zircon_runtime/src/core/runtime/time.rs,zircon_runtime/src/core/runtime/handle/time.rs,zircon_runtime/src/dynamic_api/session/state.rs,zircon_runtime/src/dynamic_api/session/profile.rs -Pattern 'advance_by','drain_steps','max_fixed_steps'`
   - `Get-ChildItem zircon_runtime/src -Recurse -Filter '*.rs' | Select-String -Pattern 'FixedStepPlan'`（核调度消费方是否仍为单权威传入）
4. 基线记录：通过 `validate-matrix.ps1` 的 `ecs_schedule` 与精确 `tests::time::` 受管过滤门记录通过数；禁止直接运行裸 Cargo，也禁止用 `time` 过滤词误匹配 `runtime`。

## 里程碑

### M0 帧循环审计（先证据后设计）

#### 切片 0.1 一帧权威链路图

- 目标文件：`docs/zircon_runtime/core/frame_schedule.md`（新建）。
- 改动形态：纯文档。画出实测链路并补全本计划未覆盖段：`zircon_app` 入口 → `dynamic_api/session/state.rs:134::tick_frame`（C ABI 转接 `session/ffi.rs:230`，导出 `dynamic_api/exports.rs:143`）→ `tick_time`（三时钟推进 + 诊断发布）→ `WorldDriver::tick_level` → 逐 stage `SceneScheduleRunner::run_stage`（四类步骤 + apply_deferred 语义）→ `RenderExtract` stage 与 graphics submit 的衔接点；每段标 owner 文件:行。
- 调用方迁移：无。
- 验收：图中每条边都有源码引用；`runtime_loop.rs` 的角色（与 session.tick_frame 的分工）写清。
- DoD：`frame_schedule.md` 落地且链路图无"未知段"。

#### 切片 0.2 extract 旁路与隐式顺序盘点

- 目标文件：`docs/zircon_runtime/core/frame_schedule.md`（追加节）。
- 改动形态：盘点两类清单——(a) `UiRenderExtract` 旁路：`session/{state,hud,menu}.rs`、`session/extract.rs` 与 `runtime_loop.rs` 的生成/消费关系，与 `RenderExtract` stage 的关系裁决（归一或定稿旁路）；(b) 隐式顺序依赖：盘点 builtin 模块注册的系统/步骤中靠注册顺序而非 stage 表达"必须在 X 之后"的位置。
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

- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -LibTests -TestFilter ecs_schedule`
- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter session`（dynamic_api 受影响）
- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_app`（宿主循环受影响时）
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
  - `fixed_loop_clamps_to_max_steps_per_frame`（对照 `RuntimeTimeAdvance` 的 fixed step cap）
- DoD：三测试绿；`TIME_FIXED_STEPS_DIAGNOSTIC` 数值与实际执行次数一致（测试断言）。

#### 切片 2.2 插值因子暴露

- 目标文件：`core/framework/time/fixed_step_plan.rs`。
- 改动形态：`FixedStepPlan` 暴露 `overstep_fraction() -> f32`（剩余 lag / timestep，clamp 到 `[0.0, 1.0]`）；经 time 服务读口供渲染插值消费（消费方接入归 render 计划，本计划只提供读口）。
- 调用方迁移：无强制迁移（新增只读口）。
- 验收：`fixed_step_plan_reports_overstep_fraction_in_unit_range`（`zircon_runtime/src/tests/time.rs`）。
- DoD：读口测试绿；`frame_schedule.md` 补固定步时序图。

#### M2 测试阶段（milestone-first）

- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter fixed_update`
- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter tests::time::`（三时钟无回归；裸 `time` 过滤词会误匹配 `runtime`）
- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter ecs_schedule`
- 验收证据：步长守恒/截断/跳过三测试 + 帧序图更新。

### M3 并行执行可观测化

#### 切片 3.1 开关与诊断计数

- 目标文件：`scene/ecs/schedule_parallel_executor.rs`；计数走 `core::diagnostics` 既有通道（不引新依赖）。
- 改动形态：并行执行加可关闭开关（owner 执行时定稿：profile 或 config_store，与子计划 02 的 config 归属一致）；诊断计数（签名草案）：`schedule.parallel_batches`（每帧并行批次数）、`schedule.serial_fallbacks`（串行回退次数）。
- 调用方迁移：无公共面变化（executor 内部 + 诊断登记点）。
- 验收：`schedule_parallel_executor_can_run_parallel_batches_serially_with_report`、`schedule_parallel_execution_report_records_diagnostic_counts`（归属 `scene/tests/ecs_schedule/parallel_executor.rs`）；结构守卫：`schedule_parallel_report_keeps_run_batches_compatible`、`schedule_parallel_disabled_path_runs_serial_batches_with_fallback_counts`。
- DoD：计数经诊断通道可读，开关关闭时回退串行且报告一致；当前 Cargo 回归已通过。

#### 切片 3.2 串并行一致性与收益证据

- 目标文件：`scene/tests/ecs_schedule/`（新测试）。
- 改动形态：纯测试。代表性 schedule（多读单写混合，复用既有 conflict_graph 测试夹具）下断言：并行批次 > 1；并行与串行执行的 world 终态一致。
- 调用方迁移：无。
- 验收：`representative_schedule_produces_multi_system_parallel_batches`、`parallel_and_serial_execution_reach_identical_world_state`。
- DoD：两测试已落地并写入批次计数基线；当前 Cargo 回归已通过。

#### M3 测试阶段（milestone-first）

- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -LibTests -TestFilter ecs_schedule`（含既有 11 个 conflict_graph/executor 测试无回归）
- `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter schedule_parallel`
- 验收证据：一致性测试 + 批次计数断言；并行语义与限制写入 `frame_schedule.md`。

## 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

当前结论：`runtime_03_schedule_frame_loop_cargo_gate_records_completed_schedule_validation` 已锁定 `completed`；当前 Runtime 过滤门为 `ecs_schedule` 77/77、`tests::time::` 4/4、`session` 165 passed / 0 failed / 10 ignored、`schedule_parallel` 15/15，`zircon_app` 受管包级门为主测试 135 passed / 0 failed / 1 ignored、PBR viewer 15/15，Runtime 03 已完成。

- 迁入记录：[`03/2026-07-09-schedule-and-frame-loop-alignment-output-records.md`](03/2026-07-09-schedule-and-frame-loop-alignment-output-records.md)
- 2026-07-18 post-completion性能复核：framework fixed clock已把大catch-up plan从逐step空循环改为Duration整数批量计算，百万step plan保持step/delta/elapsed/frame/overstep等价。Runtime03既有schedule语义不重开；requested/executed/capped/deferred-or-dropped lag计数、client/editor/headless profile cap与stall后产品trace由Runtime07承接，见PERF-MVP-328。
- 2026-07-22 post-completion frame-demand接口交接：Runtime10 V3已有`ZrRuntimeFrameDemandV1`，但Editor SessionGateway当前只校验后丢弃kind/delay并恒返true。Runtime03既有schedule完成状态不重开；PERF-MVP-424由Runtime10/App/Editor01贯通OnDemand/SleepUntil/Continuous与focus/visibility cadence，记录30秒idle wake/tick/CPU，禁止consumer把demand降格为bool常量。
- 2026-07-23 App current-source补充：`runtime_entry_app/**`74/74确认DesktopApp已消费`Idle/Immediate/After`并合并Immediate wake；剩余PERF-MVP-424精确为未处理window/device event仍请求frame、每pump两次发布control flow、Game/Mobile无focus/visibility降频、Headless固定16ms以及same-size resize重复event/rebind/presenter work。既有schedule完成状态不重开；按[`03/failure-2026-07-19-app-entry-cadence-and-event-trigger-budget.md`](03/failure-2026-07-19-app-entry-cadence-and-event-trigger-budget.md)补current-source Cargo、30秒idle、1k/10k storm和duplicate resize counter。
- post-completion failure 的代码侧回归锚点为 `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_app -LibTests -TestFilter runtime_entry`；产品侧仍须按 handoff 记录 Desktop/Editor/Headless 各 30 秒 idle、1k/10k event storm 与 duplicate resize counter。`schedule_frame_loop_boundary` 同时锁定 `dynamic_session_tick_time_call_count = 1`，防止 `session/state.rs::tick_frame` 拆分后二次推进复活。该 handoff 不重开已经完成的 M0-M3 schedule 里程碑。
