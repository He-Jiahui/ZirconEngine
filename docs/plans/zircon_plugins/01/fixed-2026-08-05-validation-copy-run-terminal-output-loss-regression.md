---
handoff_kind: fixed
status: fixed
created_at: 2026-07-30
summary_slug: validation-copy-run-terminal-output-loss-regression
origin_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_plugins/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/workspace_copy.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/workspace_copy_terminal.py
  - tools/session_coordinator/tests/test_workspace_copy.py
tests:
  - validation_copy.run persists nonzero Cargo stdout/stderr or a bounded typed terminal diagnostic
  - cargo +1.94.1 test -p zircon_runtime --lib native_live_host_editor_hot_reload_keeps_same_id_runtime_plugin --locked --jobs 1 --color never -- --exact --nocapture --test-threads=1
resolved_at: 2026-08-05
---


# Coordinator01: validation-copy run terminal output loss regression

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 来源执行切片：M5 native live-key hot-reload failure 的受管 Runtime 精确回归
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：validation-copy 子进程启动、stdout/stderr 捕获、非零终态持久化和副本清理都由 Coordinator01 控制；Plugins01 不能从共享工作树或未受管 target 重跑以替代不可变证据。

## 失败现象与复现证据

Plugins01 先以 `materialize-cargo` 正确固定四个 native live-host overlay 与
`zr_vm@5ad90fea757f5329bd36ce5824668959766c9d78`。job
`22a39605b51042ce84903cbd9b54ff21` 成功物化，input manifest 为
`6806cb6b3d1d71fb5cc0c9125f6c9e07741a9ef205c79f43b28e5fcc322626f7`；副本内四个
overlay SHA-256 与当前租约源码逐一一致。

同一副本 run `82dc077d5c1641d29ac9d6e49137c472` 于 2026-07-29T17:02:08Z 启动，
2026-07-29T17:21:58Z 以 `exit 101` 终态结束。持久记录的 `stdout_text` 与
`stderr_text` 长度都为零，随后副本在 2026-07-29T17:22:23Z 自动删除，未保留可读
fingerprint 诊断。该 run 的准确命令是：

```powershell
cargo +1.94.1 test -p zircon_runtime --lib native_live_host_editor_hot_reload_keeps_same_id_runtime_plugin --locked --jobs 1 --color never -- --exact --nocapture --test-threads=1
```

`exit 101` 且双空输出不能区分 Plugins01 编译错误、测试断言失败、子进程监督终止或
控制面捕获错误。因此它不是 native live-key 修复的反证，也不能作为通过、fixed return
或 Plan08 commandlet 上溯证据。

## 最低共享层根因

已证明的最窄边界是 Coordinator01 的 `validation_copy.run` 非零终态证据持久化：正确的
Cargo 闭包、不可变 manifest 和受管子进程均已存在，但 run 记录在清理前丢失了使非零
退出可诊断的全部输出。根因尚不能在 Plugins01 源码中归因。

## 架构修复验收

- 对每个非零 validation-copy Cargo run，原子保留受限 stdout/stderr 摘要，或写入等价的
  typed terminal diagnostic、失败阶段和原始错误摘要；自动清理不能先于该证据持久化。
- 增加 Coordinator01 回归，覆盖 Cargo 子进程返回 `101` 且输出错误文本的路径，并断言
  status/journal 能在 job 清理后检索终态诊断。
- 使用同一 Plugins01 command、同一四文件 overlay 和相同 `zr_vm` pin 重放新的不可变
  Cargo 闭包；只有取得可诊断终态后才能判定 native live-key failure 的下一步。

## 禁止临时方案

- 不得以共享工作树 Cargo、另建未受管 target、伪造绿色结果或忽略 `exit 101` 绕过证据缺失。
- 不得把调用方的短响应超时、空日志或副本自动清理解释为 Plugins01 测试通过。
- 不得削弱原始热重载回归、跳过 rollback 族或直接关闭来源 failure。

## 修复结果与回传

- 根因：validation-copy.run used communicate() to collect complete stdout/stderr before truncation, leaving collection unbounded; Windows text=True also used strict locale decoding, so invalid Cargo bytes raised UnicodeDecodeError before terminal evidence was inserted.
- 架构修复：Drain stdout and stderr concurrently into independent fixed 65,536-character tails, terminate with a typed capture error on read failure, and launch sync/async validation-copy processes with explicit UTF-8 replacement decoding so exit code and bounded evidence survive cleanup.
- 验证：Local combined workspace-copy suite 49/49 passed in 87.840s; immutable managed ticket 1dd901b1e49c4518b83e5e150dba6aee passed 49/49 in copy 9c5c0c4e1e3f4de4b787fa7f40dea1f2 in 89.593s; same-manifest Plugins01 replay job 2e1eb52882ae44e4869eb99f4df0c82c persisted run 19249a8d65764c8386c05aae9a4fd3b6 with exit 101 and 23,092 stderr characters after cleanup, diagnosing 17 missing renderable-empty template inputs rather than losing output.
- 回传：Coordinator01 terminal-output-loss is fixed and returned. Plugins01 now has durable diagnostics for its separate validation-copy input coverage issue; this return does not claim the original Cargo test is green.
