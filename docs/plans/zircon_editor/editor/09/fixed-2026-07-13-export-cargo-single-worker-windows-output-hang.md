---
handoff_kind: fixed
status: fixed
created_at: 2026-07-13
summary_slug: export-cargo-single-worker-windows-output-hang
origin_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
fixing_plan: docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
origin_child_dir: docs/plans/zircon_editor/editor/09
fixing_child_dir: docs/plans/zircon_editor/editor/15
related_code:
  - zircon_editor/src/ui/host/export_cargo_process.rs
  - zircon_editor/src/ui/host/export_process_support/output_capture.rs
  - zircon_editor/src/ui/host/export_process_support/process_tree.rs
tests:
  - cargo test -p zircon_editor --lib --locked ui::host::export_cargo_process::tests::cargo_capture_and_poll_complete_on_a_single_runtime_worker -- --test-threads=1 --nocapture
  - cargo test -p zircon_editor --lib --locked --jobs 1 -- --test-threads=1
resolved_at: 2026-07-13
---


# Editor15：Windows 单 worker 导出进程输出捕获停滞

## 产出记录与时间

| 状态 | 记录日期 | 完成项目与当前门禁 |
|---|---|---|
| `FIXED / 归属校正` | 2026-07-13 | Editor09 M1 当前源码完整门在第 1755/3157 项停在 `cargo_capture_and_poll_complete_on_a_single_runtime_worker`；但同一 current binary 的 fully-qualified exact 随后 1/1、19.65s 自然通过，独立 Windows 双流命令也在 14.8s 完成。已证实该测试只是 full-harness 累积状态下的触发点，不是 Editor15 独立功能失败；未修改生产代码，缺失自然 summary 继续由 Runtime11/Editor14 资源生命周期记录接管。 |

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/09-editor-asset-management.md`
- 来源执行切片：M1 完整 Windows lib-test acceptance
- 修复责任计划：`docs/plans/zircon_editor/editor/15-build-export-and-publishing.md`
- 交接原因：测试与生产实现均位于 Editor15 的 export process/output capture 边界；Editor09 只是完整门消费方。

## 失败现象与复现证据

当前受管完整门 job `e81ed19d256f40c28ddb2437e9a18460` 已通过编译并开始串行执行 3157
项测试。日志在 2026-07-13 23:31:01 完成
`feature_status_rejects_secondary_primary_dependency` 后停止前进；`--list` 证明下一项正是：

```text
ui::host::export_cargo_process::tests::cargo_capture_and_poll_complete_on_a_single_runtime_worker
```

现场进程证据：

- test binary PID `28168`，子进程 PID `13012`；子进程命令为 Windows `cmd /C`，分别生成
  5000 行 stdout 与 5000 行 stderr。
- 子进程从 23:31:01 持续到 23:41 后仍未退出，CPU 时间缓慢增长。
- `%TEMP%/zircon-export-28168-2-stdout.log` 与 `...-stderr.log` 在整个停滞期间均为 0 字节。
- 前一轮独立完整门日志
  `.codex/tmp/runtime02_editor_full_after_queued_fixture.stdout.log` 也终止在同一测试名，形成跨轮复现。
- Editor09 当前完整门日志：`.codex/tmp/editor09-m1-full-lib-test-r2-20260713.log`。

## 最低共享层根因

初步现场把触发边界定位到 Editor15 的 Windows export subprocess/output-capture 测试；后续用同一 current
test binary fully-qualified exact 复验为 1 passed / 0 failed / 3156 filtered out，19.65s 自然退出，且
stdout/stderr 末行断言均通过。独立 Windows `cmd` 5000+5000 双流负载也在 14.8s 完成。因此最低共享
问题不是 Editor15 capture 合同，而是 full harness 历史状态/系统资源下缺少自然结束；该 owner 已由既有
Runtime11/Editor14 failure 接管。

## 架构修复验收

- focused exact 测试在 Windows 自然退出并验证 stdout/stderr 末行，不依赖外部人工终止。
- 输出捕获与进程轮询不会因单 worker、双流背压或 Windows process tree 进入无界等待；失败必须返回
  typed error，而不是永久占用完整门。
- 重跑 Editor15 export-process focused group，再向上重跑完整 Editor lib-test 并取得自然 summary。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- Do not weaken tests or plan acceptance criteria to hide the failure.
- 禁止仅 `#[ignore]`、减少到不覆盖双流背压的输出量、在 Editor09 添加全局超时或直接杀进程来伪造通过。
- 禁止恢复旧 blocking pipe reader 或建立第二套 export subprocess 路径；修复必须收口现有唯一 owner。

## 修复结果与回传

- 根因：The full-suite process stalled while executing the Editor15 export-capture test, but the identical current test binary passes that fully-qualified exact in isolation; the trigger is accumulated full-harness/system resource state, not an Editor15 export subprocess contract defect.
- 架构修复：No Editor15 production compatibility path or timeout was added. The existing single-worker typed output-capture implementation remains canonical; ownership of the missing natural full-suite summary is returned to the existing Runtime11/Editor14 harness resource-lifecycle failures.
- 验证：The same current zircon_editor test binary ran ui::host::export_cargo_process::tests::cargo_capture_and_poll_complete_on_a_single_runtime_worker exact with one test thread and nocapture: 1 passed, 0 failed, 3156 filtered out, natural exit 0 in 19.65 seconds; stdout and stderr end-line assertions passed. A standalone Windows cmd 5000+5000 dual-stream probe also exited naturally in 14.8 seconds.
- 回传：Editor15 export capture is focused-green and was only the full-suite stall trigger; no Editor15 code fix is warranted. Editor09 still lacks a natural full summary and tracks that under Runtime11/Editor14.
