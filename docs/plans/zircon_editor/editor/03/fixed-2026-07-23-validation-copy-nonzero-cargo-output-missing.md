---
handoff_kind: fixed
status: fixed
created_at: 2026-07-19
summary_slug: validation-copy-nonzero-cargo-output-missing
origin_plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_editor/editor/03
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/workspace_copy.py
  - tools/session_coordinator/workspace_copy_terminal.py
  - tools/session_coordinator/tests/test_workspace_copy.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_workspace_copy -v
resolved_at: 2026-07-23
---


# Coordinator01：validation-copy 非零 Cargo 丢失可诊断输出

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md`
- 来源 Session：`editor03-editor16-context-hardcut-atomic-closeout-r2-20260719`
- 来源执行切片：Editor03 M2 / Editor16 CLI Context hard-cut 原子 current-source Cargo 门
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：Editor03/Editor16 的 immutable Cargo 门已得到持久 exit code，但 Coordinator01 删除副本前未保存任何可诊断 stdout/stderr，业务 owner 无法定位 exit 101。

## 失败现象与复现证据

2026-07-19 10:27 +08:00，failure-bound reservation `df1a1260317643b587564bf78ac030e0`（compatibility key `4868b627b1bab67d8c000c57f17b76db2b545a09d955f5900705f2e6a587d00e`）绑定 job `5682435a212f4921b9959edd5609c7f6`，outer run `b7a3e0bbd09e4a50a8eee133d1af8618` 执行：

```text
python -m tools.session_coordinator.cli validation-copy run +  --session-id editor03-editor16-context-hardcut-atomic-closeout-r2-20260719 +  577b33e189c94b51937d990114acdcfa -- +  cargo +1.94.1 test -p zircon_editor --lib --locked --jobs 1 +  --color never -- --test-threads=1
```

outer run exit 0 只表示 wrapper 成功返回证据；inner validation run `82329a4e961e4ce3ad3894768f9be29c` 在 1.296 秒后 exit 101。数据库 `validation_copy_runs` 的 `stdout_text` 与 `stderr_text` 均为空，outer stdout 也只包含同一空字符串证据。随后副本 `577b33e189c94b51937d990114acdcfa` 状态转为 `removed`，源与 target 均不可再检查。

该运行基于 HEAD `9cbc07ca2316f752b05dbef95ade9d70e893afeb`、snapshot `608`、M2 manifest `4d56cc596a1545c8ade20e56775683c7`（hash `c6adfb93713583c90354b8f497f0b8f935954cbbc5e220b7fab6832bedd9ee87`），完整副本 24,608 项、31 个 overlay 预检零差异；因此不能把空输出归因于 source race 或 manifest 漂移。

## 最低共享层根因

`WorkspaceCopyService.run` 在子进程非零终态后依赖 `ValidationCopyTerminalLifecycle.collect/persist` 保存输出，然后无条件清理已完成副本。当前链路允许“exit 101 + 两个空流”成为最终证据并删除唯一可检查工作目录，缺少对非零无诊断证据的 fail-closed 保留策略，也没有把 launch/collect 层的原始错误或 stderr 文件独立持久化。

## 架构修复验收

- 非零 validation 子进程必须在副本删除前持久化可诊断原始 stdout/stderr；若两个流都为空，必须保留副本和 target，并返回明确的 evidence-incomplete 错误。
- outer wrapper、`validation_copy_runs`、CLI status 和受管 Cargo run 证据必须引用同一个 durable run id，重启后仍可读取。
- focused 测试覆盖非零有输出、非零空输出、启动失败、输出持久化失败和 cleanup 失败；空输出场景不得丢失副本。
- 修复回放 Editor03 原命令后，若 Cargo 仍 exit 101，必须能据原始输出定位对应业务 owner；只有 inner exit 0 才能返回 Editor03 GREEN。

## 禁止临时方案

- 禁止把 outer exit 0 当作 inner Cargo GREEN。
- 禁止仅延长内存 tail、打印到当前终端或依赖副本删除前人工抢读。
- 禁止让 Editor03 直接绕过 validation-copy 在共享工作树运行 Cargo。

## 修复结果与回传

- 根因：The validation-copy terminal lifecycle allowed a nonzero Cargo result with no persisted stdout or stderr to be cleaned up, destroying the only actionable failure evidence.
- 架构修复：Terminal collection now persists bounded diagnostic output under the durable run identity and blocks cleanup when a nonzero result has incomplete evidence, preserving the copy and target for managed recovery.
- 验证：Current-source validation-copy gates passed 38/38; affected broad passed 153/153; failure-closeout deletion contract passed 17/17.
- 回传：Editor03 may create a fresh exact validation copy after the managed commit and controlled daemon reload; historical terminal evidence remains immutable.
