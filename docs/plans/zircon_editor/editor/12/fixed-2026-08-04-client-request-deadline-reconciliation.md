---
handoff_kind: fixed
status: fixed
created_at: 2026-07-28
summary_slug: client-request-deadline-reconciliation
origin_plan: docs/plans/zircon_editor/editor/12-plugin-management.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_editor/editor/12
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/zircon-session.ps1
  - tools/session_coordinator
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1
tests:
  - coordinator client deadline-after-submit reconciliation regression
  - validator dry-run timeout/reconnect idempotency regression
resolved_at: 2026-08-04
---


# Coordinator01: client request deadline lacks terminal reconciliation

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/12-plugin-management.md`
- 来源执行切片：Editor12 V2 dynamic action source-bound Windows validation
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：session register、validator dry-run、cargo list/status 在有其他受管 Cargo 运行时先后超过客户端 30 秒等待上限。至少一次 register 在客户端超时后仍由服务端创建 Session，说明调用方不能将 deadline 等同于未提交；Coordinator01 是 request lifecycle、重连和幂等 terminal evidence 的唯一 owner。

## 失败现象与复现证据

- `editor12-plugin-v2-dynamic-action-validation-r1-20260727` 的 `session register` 前端在 34 秒超时，随后 `session show` 返回已创建的 Session；原会话不可变 write scope 也使 validator 不能重用它。
- 使用独立 `Cargo validation` Session 后，`validate-matrix.ps1 -DryRun` 在前端 34 秒超时，未返回 request/job terminal；后续 `cargo list` 与 `status` 也分别得到客户端 deadline，而先前可见 Shader06 的真实受管 Cargo PID。
- 这些输出既不能证明命令未到达服务端，也不能证明成功、失败或释放。盲目重试可能重复创建 Session、reservation 或 validation action，调用方只能停止提交并保留现有 snapshot。

## 最低共享层根因

coordinator client protocol 在 request 已提交但响应未在本地 deadline 前返回时缺少稳定的 request/intent correlation 与 terminal poll/reconnect contract。CLI 和 validator 把 transport deadline 暴露给调用方，却没有以原 request identity 查询服务端的 accepted/executing/terminal state，导致“未知结果”无法被无副作用地收敛。

## 架构修复验收

- 所有会创建或改变协调器状态的 client request 必须携带持久 idempotency/request identity；服务端接受后可由该 identity 查询唯一 request、Session、reservation 或 job 的当前 terminal state。
- 客户端 deadline 后必须先重连并 poll 同一 identity，而非重发 mutation；返回必须明确 `not_accepted`、`accepted/executing` 或唯一 terminal evidence。
- `validate-matrix.ps1` 的 dry run 与真实 validation 都必须使用该 reconciliation path；已存在 immutable Session scope 时能引导独立 validation Session，不得改写源会话 scope。
- `cargo list`/`status` 的大历史结果必须有有界响应/分页或摘要路径，活跃 job 查询不得因历史 records 使调用方失去当前 lane ownership。
- focused tests 覆盖：deadline 后服务端实际接受、deadline 前未接受、重复 retry、运行 job 保留、immutable scope、validator reconnect 和 active-job summary；不得创建重复 cargo job。

## 禁止临时方案

- 不得只增大 30 秒 timeout、sleep 后盲目重试、把 deadline 记为成功/失败，或要求调用方绕过 coordinator 直接运行 Cargo。
- 不得通过删除运行 job、重启服务、清空历史 ledger 或修改已有 Session write scope 消除症状。
- 不得将控制面未知终态归因到 Editor12/Rust source、写入 `fixed-*`，或用 `blocked` 隐藏仍可恢复的请求状态。

## 修复结果与回传

- 根因：An unfenced GET 404 can overtake a delayed POST before the request journal row exists, so command_request_not_found did not prove that the mutation was not accepted.
- 架构修复：Keep an unfenced missing request as submission=unknown and return command_post_timeout with the same request identity; never replay the mutation. Add a real concurrent GET-overtakes-POST regression and preserve terminal reconciliation.
- 验证：Managed validation ticket d8801cc9d2144c849045933b847fa2cf passed 18/18 test_client cases in validation copy job 6d9e5fc5c29e42a6b4c9fa2cbc970dd5 (exit 0). Local related regression suite passed 46/46. Editor12 origin validate-matrix DryRun created exactly one job c37fba417fc94d4a83d4f85c97d20740; acquire request 20a9fedae0d5446abd48ae743f6164a9 and release request b07e1ea16a1048f3a936336af8cbb7b5 both terminalized completed.
- 回传：Coordinator01 repaired deadline-after-submit reconciliation and returned fresh managed evidence to Editor12 without duplicate mutation or Cargo job.
