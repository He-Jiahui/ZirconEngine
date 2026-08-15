---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: native-systems-conservative-world-writer-serialization
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_plugins/01
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/plugin/native_plugin_loader/registration_manifest.rs
  - zircon_runtime/src/plugin/native_plugin_loader/registration_manifest/system_access.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/registration_replay.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter.rs
  - zircon_runtime/src/plugin/extension_registry/register/system_registration.rs
  - zircon_runtime/src/scene/ecs/schedule_stage_plan.rs
  - zircon_runtime/src/scene/ecs/schedule_runner.rs
  - zircon_runtime/src/scene/ecs/native_system_schedule_diagnostics.rs
tests:
  - disjoint_worker_safe_native_systems_overlap_in_the_production_stage_runner
  - conflicting_worker_safe_native_systems_remain_serial
  - main_thread_only_external_system_runs_on_the_schedule_caller
  - panicking_worker_safe_system_is_restored_before_the_panic_resumes
  - native_system_schedule_diagnostics_record_conflicts_latency_and_utilization
---

# Plugins01：native systems 全部作为保守 World 写者串行化

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：MVP native registration replay 与 runtime schedule 静态审查
- 修复责任计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 共同验收：`docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md`、`docs/plans/zircon_runtime/runtime/11-job-system-task-model.md`
- 交接原因：这是插件 ABI access/thread-safety 契约与 ECS scheduler 的共同边界，不能由调度器猜测 foreign callback 是否安全。

## 失败现象与复现证据

`NativePluginRegistrationSystem` 已反序列化 `access: Vec<String>`，但 validation 与 replay 都不读取该字段。
`register_bridge_replay_system` 无条件调用
`register_native_system::<NativeDynamicAccess, _>(...)`；`NativeDynamicAccess` 通过
`SystemParamAccess::add_conservative_world_access()` 声明保守世界写访问。

因此每个逐帧 native system 都与所有 component/resource world access 冲突；多个彼此不相交的 native plugins
也无法进入 ECS parallel batch。把 callback 直接送入 worker 同样不安全，因为当前 ABI 没有 thread-safe、
main-thread-only 或外部状态同步契约。

## 最低共享层根因

NativeDynamic 注册 manifest 的 access 文本没有被编译为稳定、可验证的 `SystemParamAccess`，同时缺少 callback
thread-affinity/reentrancy 声明。当前 conservative writer 是正确安全 fallback，但会把插件扩展能力变成全局串行点。

## 架构修复验收

- 定义稳定 component/resource id 的 read/write access schema；注册期解析、resolve 并拒绝未知、重复冲突或越权声明。
- 增加明确 thread affinity：worker-safe、main-thread-only；默认继续保守，不得默认信任 DLL。
- 通过验证的 access 构造真实 `SystemParamAccess`；仅 unknown/legacy path 使用 conservative world writer，并输出可观测诊断。
- 两个 disjoint worker-safe native systems 在 schedule trace 中有并行 overlap；同一资源写/读写冲突确定串行。
- main-thread-only callback 始终在主线程；panic/重入/卸载 generation 与 in-flight callback 安全契约不退化。
- 记录 conflict count、ready delay、worker utilization 与 callback p95，纳入 Runtime08/11 性能预算。

## 参考引擎原则

- Bevy ECS 把系统 component/resource access 冻结进 schedule conflict graph，worker 并行来自显式 access 而非运行期猜测。
- Zircon 应迁移这一原则，同时保留 native ABI 的 capability、thread affinity 与动态库生命周期约束；不复制 Rust trait-object ABI。

## 禁止临时方案

- 不得把 `NativeDynamicAccess` 改成空 access 来制造伪并行。
- 不得仅凭 manifest 自报字符串就允许 worker 执行，必须 resolve 稳定 id、校验 capability 与 thread-safety。
- 不得为“并行”而让 hot reload/unload 提前释放仍在 worker 调用中的动态库。

## 修复结果与回传

Open state: `implementation_complete / current-source static review complete / managed focused-broad and Runtime08/11 performance acceptance pending`。

### 2026-07-22 Plugins01 实现状态

| 里程碑 | 状态 | 完成日期 | 完成项目与剩余证据 |
|---|---|---|---|
| stable access / affinity contract and production runner | `implementation_complete / managed_broad_and_perf_acceptance_pending` | 2026-07-22 | registration manifest v3 现解析 `read|write:component|resource:<stable-id>` 与 `main-thread-only` / `worker-safe`；empty/独占 `write:world` 保持保守主线程 fallback，mixed world、未知 ID、重复/读写冲突、未授予 worker capability 与越权 foreign access 均显式拒绝。replay 使用 descriptor requested + runtime module capability 的实际宿主授权，编译真实 `SystemParamAccess`；direct ABI-v3 注册仍保持 conservative/main-thread hard fallback。`SceneScheduleStagePlan` 缓存 native conflict graph，生产 runner 只批处理无约束、worldless、worker-safe 且互不冲突的相邻 system，通过 `JobScheduler::join` 在 World lock 外执行；main/runtime/hook/internal/deferred/conflict 均为 barrier。callback panic 会先恢复 boxed system、poisoned callback mutex 与 stage deferred-flush 状态，再传播 panic，generation-owned `NativeHostBridgeCallScope` 继续持有 DLL owner。每帧新增 `conflict_count`、`ready_delay_ms`、`worker_utilization`、`callback_p95_ms`、worker batch、callback 与 `conservative_world_writer_count` 七条 DiagnosticStore 指标；SDK DTO/fixture/docs 同步 hard cut，旧 `read:scene.time` 占位语法已清除。scoped rustfmt、`git diff --check` 与 Rust 1.94 tuple/slice + fixed-array 类型探针通过；`python tools/audit_plugin_structure.py --json` 通过（37 manifests、39 dist targets，capability/registration/skeleton/SDK mirror 违例均为 0）。current-source reservation `443ed28a879c42228127596358928d6a` / job `a2f858fdd9894cb88df122fb92780da9` / run `255a862363d74b699f0b23cf308dfe3b` 已自然 `exit 0`，完整 lib-test 构建耗时 19m53s；`native_callback` 过滤组为 `5 passed / 0 failed / 1 ignored`。随后不可变二进制的 access-manifest 分组暴露 `physics.Body` 已登记 descriptor 但尚无实例时没有 ECS `ComponentId`，导致 `6 passed / 1 failed`；根因是 `World::register_component_type` 未预分配 dynamic access id。生产修复已让 component descriptor/reflection 注册成功后同步建立稳定 ECS id。旧 r10 `ed339385ebc94690a60ef20d83a8be1a` 与 r11 `7b0564dd8fe94805ae300bb697d78d51` 分别因共享 projection 变化和随后发现的 event-test 私有模块导入而主动释放；r12 又因 core-min 不编译 linked-plugin test 而主动释放。包含边界修复并启用默认 feature 的 current-hash r13 `41a0329396404c4d830c6987fe663225` 正按 FIFO 等待复验。尚需该 GREEN、其余 focused/broad、真实 trace 的 overlap/冲突/main-thread/panic 与 Runtime08/11 性能预算动态证据，因此本 failure 保持 open。 |

同一 `r9` 不可变二进制中，与 ECS id 增量修复无关的生产调度回归已通过：
`scene::ecs::schedule_runner::tests` 为 `4 passed / 0 failed`（disjoint overlap、conflict serial、
main-thread affinity、panic restore），native schedule diagnostics 为 `1/1`，scheduled native systems
为 `7/7`。registration replay 为 `6 passed / 1 failed / 1 ignored`，唯一失败同样是
`physics.Body` access id 未安装，没有暴露第二根因；`apply_to_world` 已确认组件先于系统应用，
因此 eager dynamic id 修复覆盖该集成路径。这些结果关闭对应行为回归疑点，但不替代 `r13`
对 access-id 修复的 current-source GREEN。

### 2026-07-30 current-source static re-audit

本轮只完成了不启动 Cargo 的当前源码审计，failure 继续保持 `open`，不得据此声明已修复或验收：

- manifest/replay 与 direct ABI 都复用
  `NativeSystemAccessPlan`。manifest 路径把 `access` 与 `thread_affinity` 解析为受 capability
  约束的 stable component/resource access，并经 `register_external_native_system` 在 build 时
  生成实际 `SystemParamAccess`；V3 direct ABI 仍保守，V4 的独立
  `ZrSystemRegistrationV2` 在 layout、access、affinity、stable ID 和 capability 校验后走同一
  external-system 路径，没有扩展 V1 布局或引入第二个 scheduler authority。
- 当前源码的 interface、adapter、manifest/replay 和对应 contract test 已执行
  `rustfmt +1.94.1 --check --edition 2024 --config skip_children=true ...`，结果 exit 0；同范围
  `git diff --check` 结果 exit 0（仅报告既有 LF/CRLF 提示）。
- `python tools/audit_plugin_structure.py --json` 结果 exit 0：38 个 manifest 与 40 个
  distribution target 均通过，capability、registration、skeleton 违例均为 0。
- 参考对照：Bevy 的
  `dev/bevy/tests/ecs/ambiguity_detection.rs` 以 schedule conflict graph 统计冲突系统；Godot 的
  `dev/godot/core/extension/gdextension.cpp` 与
  `dev/godot/tests/compatibility_test/src/compat_checker.c` 展示版本化 C ABI 入口在取得函数表后
  显式设置 initialize/deinitialize 与最低初始化级别。Zircon 采用相同的“显式契约后可调度”原则，
  但不复制 Rust trait-object ABI 或 Godot 的 class API。
- 尚未执行任何新的 managed Cargo。Coordinator FIFO 当时由
  `runtime15-structure-guard-path-repair-20260729` 的 job
  `191a43af42de46b18f2d3529a48a875a` 占用；已 materialize 的 validation-copy
  `5945e3ef29d74bd69602adca02e243b5` 不是 Cargo run，未被重建、重试或清理。仍待 current-source
  focused/broad、真实 DLL/runner trace 与 Runtime08/11 性能预算证据，再决定 failure return。

### 2026-08-11 conservative fallback worker-dispatch correction

- 本轮从 `SceneScheduleStagePlan` 到 `SceneSystem::supports_worker_dispatch` 做了当前源码复审，
  发现 `WorkerSafe + worldless + no ordering constraints` 之前不足以表达 worker eligibility：带
  `SystemParamAccess::add_conservative_world_access()` 的 callback 仍可进入 `flush_worker_batch`，
  在 `World` 锁外执行。这与未知/legacy access 必须保守主线程串行的 ABI 回退契约冲突。
- `supports_worker_dispatch` 现在同时要求 `!access().has_conservative_world_access()`；stage plan
  本来已经只经该单一策略形成 `worker_safe`，因此没有增加第二个判断 authority。新增
  `conservative_world_writer_is_not_dispatched_to_a_worker` 回归用例，构造 `WorkerSafe` 但保守
  world writer 的外部系统，并断言它在 schedule caller 执行、`worker_batch_count=0`，同时保留
  conservative writer 诊断。
- `rustfmt +1.94.1 --edition 2024 --check` 与 scoped `git diff --check` 均通过；静态路径守卫确认
  stage plan 继续调用 `supports_worker_dispatch()`，且该策略明确拒绝 conservative access。
- 该新 Rust 用例尚未运行：唯一已物化 validation-copy
  `5945e3ef29d74bd69602adca02e243b5` 属于另一 Session，coordinator 拒绝跨 Session 消费；本轮没有
  重建、重试、清理该副本或直接运行 Cargo。failure 继续保持 `open`，待合法 current-source managed
  focused/broad、worker/main-thread trace 与 Runtime08/11 性能预算证据。

### 2026-08-11 current-source materialization preflight

- 为验证本轮 scheduler 修复，Plugins01 已在 live lease 下为三项 source 和三项 plan/test input
  写入 current-byte attribution。随后以当前 workspace 请求 coordinator validation-copy materialization；
  该请求在启动 Cargo 前被治理门拒绝，未产生 source copy 或 target output。
- coordinator 的 artifact audit 精确报告一个未登记、空、无 owner 的目录
  `E:\ZirconBuilds\mvp-perf`（创建于 2026-08-10 16:53 UTC）。项目工件策略禁止在存在该目录时创建
  D/E/F validation output；Plugins01 不拥有该目录，未删除、清理或登记它。
- 因此本项仍只有 static GREEN：格式、whitespace、scoped diff-check 与单一 policy-path guard 已通过；
  `conservative_world_writer_is_not_dispatched_to_a_worker` 的 managed Rust RED/GREEN、focused/broad、
  trace 和性能预算仍是 pending。旧 foreign validation-copy 也保持 untouched。

### 2026-08-11 current-source static recheck

- 当前源码重新确认 production eligibility 仍只有 `SceneSystem::supports_worker_dispatch()` 一处：
  `WorkerSafe`、worldless、无 ordering constraint 之外，必须同时满足
  `!access().has_conservative_world_access()`。`SceneScheduleStagePlan` 继续只消费这一策略，
  所以 legacy/unknown 的 conservative writer 不会重新进入 worker batch。两个 production 文件的
  Rust `1.94.1` rustfmt、scoped `git diff --check` 与 policy-path source guard 均为 PASS。
- 新回归用例仍正确约束 conservative writer 在 schedule caller 执行，并报告
  `worker_batch_count=0`、`callback_count=1` 与 `conservative_world_writer_count=1`。不过当前
  `zircon_runtime/src/scene/tests/ecs_scheduled_native_systems.rs` 在无关的静态断言格式上不满足
  Rust `1.94.1` rustfmt；Plugins01 未修改该范围外现有差异。因此不能把包含该文件的整组
  current-source rustfmt gate 记为 GREEN。
- coordinator artifact audit 现同时报告未登记的
  `E:\ZirconBuilds\mvp-perf` 与
  `E:\ZirconBuilds\mvp-product-inputs-profile-20260811-current-source`。Plugins01 不拥有、
  不清理、不登记二者，且未启动 Cargo 或重建/重试/清理 foreign validation-copy
  `5945e3ef29d74bd69602adca02e243b5`。failure 保持 `open`，等待工件治理后合法的 current-source
  focused/broad、trace 与 Runtime08/11 性能预算证据。
