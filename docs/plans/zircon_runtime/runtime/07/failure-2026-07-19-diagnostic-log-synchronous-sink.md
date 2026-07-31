---
handoff_kind: failure
status: open
created_at: 2026-07-19
summary_slug: diagnostic-log-synchronous-sink
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/07
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/diagnostic_log/sink.rs
  - zircon_runtime/src/diagnostic_log/level.rs
  - zircon_runtime/src/diagnostic_log/diagnostics.rs
tests:
  - multi-thread slow-sink log storm
  - shutdown/crash durability and rotation parity
---

# Runtime07：diagnostic log同步sink

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行切片：PERF-MVP-434 diagnostic sink bounded-queue and durability gate
- 修复责任计划：`docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md`
- 交接原因：diagnostic sink 的过滤、队列、I/O、flush 与 durability 合同由 Runtime07 所有。

## 失败现象与复现证据

每条允许日志都在caller线程生成wall-clock timestamp与整行String，逐条线性扫描scope rules，随后争用单一file mutex、同步write并立即flush。高频诊断或慢盘会直接阻塞runtime/editor线程；disabled前已构造的message也无法回收调用方成本。

## 最低共享层根因

日志格式化、文件 I/O 与逐条 flush 仍由 caller 同步承担，并通过单一 file mutex 串行化所有生产者。

## 架构修复验收

- 启动时编译scope filter，调用点提供lazy gate；disabled时timestamp/format/message alloc均0。
- 通过有界queue交给单一sink owner，按count/time/bytes批量write/flush；verbose/debug drop可观测，warn/error durability和crash flush明确。
- 1/1k/100k logs/s、1/64 threads、0/10/100ms sink记录caller lock/I/O/flush=0、queue age/depth/drop、RSS与p95；顺序/rotation/shutdown等价，回传PERF-MVP-434。

## 禁止临时方案

不得以 0-test 执行、foreign compile blocker、无界队列或 caller 同步 critical fallback 宣称通过；所有接受结论必须来自 current-source 目标断言和独立性能证据。

## 修复结果与回传

### 2026-07-22 current-source 完成项

- 已将 scoped filter 编译为启动期复用的最长前缀匹配结构，并提供完整 level/scope lazy entry API；禁用、无输出和 shutdown 后路径不执行 message closure。
- 已建立单一 worker owner 的 bounded FIFO。caller 不再生成 timestamp/整行、不持有 file mutex、不执行 write/flush；worker 按 count/time/bytes 批量输出。
- 队满时 `verbose/debug/log` 按级别记录 drop，`warn/error` 使用可观测 backpressure；metrics 覆盖 queue depth/high-water、dequeue/write、bytes/batches、queue age、drop、critical backpressure、output error 与 closed。
- 显式 flush/shutdown 使用有界 ack；file durability 包含 `sync_data`。panic hook 跳过 sink worker 后执行 bounded flush，再委托旧 hook；editor/runtime binary 在 EntryRunner 前安装 hook，并在捕获 result 后 shutdown，避免 commandlet early return 绕过 drain。
- diagnostic-store bridge 与 10 个 allocation-heavy producer 已迁移 lazy formatting；公开模块文档已从“每条 caller 立即 flush”更新为 bounded/batched/durable 契约。
- `diagnostics.rs` inline tests 已按 `engine-code-structure-convention.md` R4.1/R4.2 拆为 folder-backed `format_schedule`、`lazy_callsite_guards`、`ownership` 三个行为族；生产 owner 104 行，测试文件 49/71/73 行。
- PERF-MVP-434 ignored harness 已覆盖 54 个组合：1/1k/100k global attempts/s、1/64 callers、0/10/100ms worker sink delay。所有 caller 先 barrier-ready，再消费同一 future start；100 个 pacing buckets、共同 deadline 与 cancel 提供超时检测，RAII 对 sink/RSS sampler 执行有界 best-effort 清理并避免无界等待。
- 性能证据采集包括 caller p95、queue depth/high-water/age/drop、critical backpressure、worker write/flush/sync、native Windows working-set baseline/active peak/after 与 live sample count。低速场景验证精确完整 sequence set；高压慢 sink 必须触发 bounded drop；独立 blocking-output companion 锁定 warn/error backpressure 与四条原始输出。

### 当前证据

- bounded sink 与 PERF-MVP-434 exact24 snapshot `832`、process lifecycle exact3 snapshot `803`、lazy producer/docs folder-backed exact10 snapshot `811` 均完成独立只读复审 `Critical 0 / Important 0 / Minor 0`。
- scoped `rustfmt +1.94.1 --check` 与 `git diff --check` 通过；仅有仓库行尾转换提示。
- app lifecycle reservation `7ea414f887334e3e8d1f7736c04c9184` 已消费为 job `835ae0a9ff4b46fba734b09c7c63e60e` / run `aa2c07d120d84c5088bddd1d6d78139e`，自然 terminal 并 released：exit 101、无存活 PID、stdout 0 bytes、目标测试 0 个执行。真实阻断是当前共享源的 7 个 rustc errors：plugin manifest projection 私有重导出/导入 4 个、extension registry `E0373`、font source `E0502`、scene event lifetime 1 个；不得把该作业记作 lifecycle red/green。
- 下层 owner 路由已只读核实：Plugins09 的 4 个 projection 可见性错误继续归 `docs/plans/zircon_plugins/09/failure-2026-07-17-export-profile-validation-quadratic-scans.md`；extension registry `E0373` 归 Performance01 `PERF-MVP-535` borrowed-iterator 切片，当前只有 `docs/plans/performance/01/2026-07-22-runtime-plugin-catalog-registration-static-review.md` implementation record、没有 canonical failure，等待原 owner 恢复并建立正式 failure/fixed 生命周期；font source `E0502` 当前源码已有 Text01 owner 的未验证局部修复，证据在 `docs/plans/zircon_runtime/text/01/2026-07-17-text-mvp-font-raster-foundation-closeout.md`；scene event lifetime 归 `docs/plans/zircon_plugins/01/failure-2026-07-22-plugin-event-drain-frame-budget.md` 的 EventCursor/consumer closure。Runtime07 不修改这些 foreign paths，也不以当前源码漂移推断其中任何一项已 fixed。
- sink focused reservation `0bc64d163acf42a89f4a680bceb368af` 已消费为 job `e78311352edd4d0091766612e0d37b90` / run `d03dd45da2464ceb8584bdaf7db965de`，自然 terminal 并 released：exit 1、无存活 PID、stdout 0 bytes、目标测试 0 个执行。stderr 仅到依赖编译进度后停止，没有 rustc/Cargo diagnostic；该结果只证明受管执行层未完成编译，不能记作 sink red/green。exact24 current-source manifest 随后复核为 24/24 hash-stable，并以相同 compatibility/retained target 申请 warm retry reservation `67ecdb0b3fb94e5a9b0b723c6d19147e`。
- 上述 sink focused terminal、0-test 边界、retained-target reuse 与 PERF 未排队状态已经独立只读终审，结果为 `Critical 0 / Important 0 / Minor 0`。
- 仍等待 FIFO 的 source-bound CPU reservations 为 sink focused warm retry `67ecdb0b3fb94e5a9b0b723c6d19147e`（snapshot 832）和 lazy source guard retry `93575b3106df4338b9157225c19a66e3`（snapshot 811）。旧 sink reservation `144b3f81a49943229c422a67de27e3fa` 已因加入性能 harness 主动释放，旧 lazy reservation `a61fa28a1b1c48ef9f7e93148efa5019` 已过期，均不作为验收证据。

### 剩余验收

- 先由 plugin/asset/scene 各自 owner 消除上述 7-error current-source compile blocker，再重新申请 source-bound app lifecycle gate；受管 Windows focused gates 必须真实执行目标断言，不能用上层 compile blocker 或旧产物替代。
- PERF-MVP-434 harness 已完成实现与终审；待 sink focused warm retry 真实执行目标测试并取得有效终态后，按同一 snapshot 832 申请独立 `--ignored --exact` 受管运行并回传 54-case 原始报告，不能用 0-test 执行层失败或 focused gate 的 compile/pass 代替性能接受。
- 完成顺序、shutdown/crash durability 和 rotation parity 后，才可执行 failure return、计划状态关闭与 managed milestone commit。

Open state: `Runtime07 bounded sink、lazy producers、process lifecycle、结构拆分与 PERF-MVP-434 harness 已完成静态/独立复审；app lifecycle source-bound gate 已被 7 个 foreign current-source rustc errors 阻断且 0 test executed，sink focused 首次受管运行也以无 diagnostic 的 exit 1/0-test 执行层失败结束，现等待 warm retry、其余有效 focused gates、真实性能矩阵和 parity terminal evidence，不得返回 fixed`。
