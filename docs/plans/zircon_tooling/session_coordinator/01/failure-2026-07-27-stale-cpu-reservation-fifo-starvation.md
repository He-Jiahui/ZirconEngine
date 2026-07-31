---
handoff_kind: failure
status: open
created_at: 2026-07-27
summary_slug: stale-cpu-reservation-fifo-starvation
origin_plan: docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_runtime/shader/06
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/cargo_jobs.py
  - tools/session_coordinator/reserved_starts.py
  - tools/session_coordinator/command_requests.py
tests:
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -RepoRoot E:\Git\ZirconEngine -Package zircon_runtime -LibTests -TestFilter light_grid_normalizes_surface_inputs_once_per_pixel -SkipBuild -VerboseOutput
  - .\tools\zircon-session.ps1 session heartbeat --session-id shader06-m5-current-source-20260726 -Json
  - .\tools\zircon-session.ps1 lease heartbeat --session-id shader06-m5-current-source-20260726 -Json
---

# Coordinator01: stale CPU reservation causes FIFO starvation

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md`
- 来源执行切片：EC-M5 current-source PBR hot-path Rust/Naga gate。
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：CPU reservation 生命周期、FIFO admission 和 coordinator command preflight 都由 Coordinator01 control plane 统一持有；Shader06 不能释放或重写另一个 Session 的 reservation。

## 失败现象与复现证据

2026-07-27 本地三次受管启动均在创建 Cargo job 前失败，返回同一错误：

```text
cargo_cpu_lane_reserved
reservationId: 74981a9137264e67b8ea4bc479b2d0e9
sessionId: plugins09-compact-validate-report-closeout-r4-20260726
```

`plugins09-compact-validate-report-closeout-r4-20260726` 的 coordinator state 仍为
`resolving_failure`，但其最后 heartbeat 是 `2026-07-26T15:50:18Z`；期间本机没有存活的
`cargo` 或 `rustc` 进程。随后来源 Session 的 `session set-status` 与 `lease heartbeat`
也在 coordinator health preflight / shell deadline 超时，不能可靠地续约。三次 Rust gate
均未创建 Cargo job，不能被记为 Shader06 编译或测试失败。

同一 2026-07-27 会话后续复核仍未恢复 gate：一次 acquire 在授权后返回
`admission_checkpoint_stale`，紧接着的单次协议重试在 preflight 阶段返回
`command_preflight_timeout`（`submission: not_submitted`）。两次都没有创建 Cargo job，说明
reservation/fairness 变更期间 control plane 仍不能提供可消费的 CPU admission。

在 Shader06 完成 fallback PBR 静态契约收敛后的单次受管复测中，
`validate-matrix.ps1` 再次在创建 Cargo job 前返回 `cargo_cpu_lane_reserved`：

```text
reservationId: b91bdc6a0a4f4ffdbd2f01704c092a27
sessionId: plugins09-compact-validate-report-closeout-r4-20260726
```

该 reservationId 与最初观测不同，但仍由相同陈旧 owner 占据 CPU lane；本次复测同样没有
提交 Cargo job。因此这不是 Shader06 Rust/Naga 失败，且说明 stale-reservation starvation
尚未修复。

## 最低共享层根因

已证明的最低边界是：Coordinator01 允许一个没有存活 Cargo tree、且 owner heartbeat 已陈旧
的 CPU reservation 持续占据 FIFO head，并在 control-plane command timeout 时没有把它安全地
终态化或给调用者可恢复的排队结果。reservation 的最终数据库状态和具体清理分支尚待 fixing
owner 在 `cargo_jobs.py` / `reserved_starts.py` 中诊断；来源计划不得直接写 coordinator 状态。

## 架构修复验收

- stale / owner-lost reservation 必须按可审计的生命周期规则自然 terminalize 或由其 owner 恢复；
  它不能无限占据 CPU FIFO head。
- 仍有效的 reservation 必须让 owner 能在 bounded 时间内 consume，且其他 Session 得到有界的
  queued/retry contract，不得反复返回同一永久 reservation block。
- `session heartbeat`、`lease heartbeat` 和 `session set-status` 在 Cargo reservation 存在时
  仍须在其 command deadline 内返回。
- 修复后重新运行本 artifact 的 Shader06 focused gate；只有产生真实 Cargo job 后才可判定其
  编译/测试结果。

## 禁止临时方案

- 不得由 Shader06 释放、修改或绕过 `plugins09` 的 reservation，也不得共享工作树直接 Cargo。
- 不得通过延长 timeout、无限轮询、伪造 heartbeat、忽略 FIFO 或把历史 Cargo 结果当作当前源码
  验证来掩盖该问题。
- 不得削弱 Shader06 的 Rust/Naga、viewer、RenderDoc 或里程碑验收门槛。

## 修复结果与回传

Open state: `待 Coordinator01 修复`; no Cargo or product pass is claimed.
