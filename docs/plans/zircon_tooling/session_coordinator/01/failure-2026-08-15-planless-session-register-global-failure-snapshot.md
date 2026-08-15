---
handoff_kind: failure
status: open
failure_scope: local
created_at: 2026-08-15
summary_slug: planless-session-register-global-failure-snapshot
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/server.py
  - tools/session_coordinator/tests/test_server.py
---

# planless-session-register-global-failure-snapshot: 验证失败回写

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：Coordinator failure-chain scope rotation and maintenance Session registration
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：同一编号计划拥有 Session admission 与 failure graph import 边界。

## 失败现象与复现证据

- `session.register` 对没有 `plan_path` 的新 maintenance Session 仍无条件执行 `FailureGraphService.prepare_import_snapshot()`。
- 2026-08-15 生产注册 `coordinator01-planless-session-register-snapshot-20260815` 在 schema 63 healthy/idle daemon 上耗时 54.8 秒；其返回的 failure inventory 来自 619 个 repository artifacts，但 planless Session 的最终 open failure 集本应为空。
- 任一无关编号计划的 `failure-*.md` 在该窗口漂移，都会以 `failure_snapshot_stale` 拒绝 planless maintenance admission。

## 最低共享层根因

`CoordinatorApplication._execute_session_registration_request()` 无条件把 failure snapshot preparation 注册为 admission 前置步骤；只有进入 writer 后的 `_session_registration_open_failures()` 才计算 effective plan 并对 planless Session 返回空集。因此 planless control-plane maintenance 被错误耦合到全仓 failure graph 的文件 I/O、解析、校验和 CAS。

## 架构修复验收

- 请求和已存在 Session 都没有 plan 时，不准备、不导入且不替换 failure graph，注册结果的 `open_failures` 为空。
- 显式 plan 或已存在 Session 的 immutable plan 仍必须在 writer 外准备 snapshot，并在 writer 内做 immutable fingerprint 校验和导入。
- 若 admission 前观察为 planless、但 writer 内 effective plan 因竞态变为非空，必须返回 `failure_snapshot_missing`；不得在写事务内回退到 repository parse/import。
- 重复 planless registration 不读取全仓 failure artifacts；plan-backed registration 的现有 failure priority 行为不变。

## 禁止临时方案

- 不延长 CLI timeout，不缓存可漂移的未绑定 snapshot，不吞掉 `failure_snapshot_stale`。
- 不允许有 plan 的 Session 跳过 failure import，也不得在 SQLite writer 内重新解析 repository。

## 修复结果与回传

- RED：`test_planless_registration_does_not_prepare_or_import_failure_snapshot` 在旧实现中准确失败于无条件 `prepare_import_snapshot()` 调用。
- 架构修复：admission 前通过统一 effective-plan 判定只为显式或既存 plan 准备 immutable snapshot；writer 内重新判定，planless 直接返回空 failure 集，竞态出现 plan 但缺 snapshot 时以 `failure_snapshot_missing` fail closed。
- 本地验证：相关 server 6/6、session-register durability 15/15、完整 `test_server` 66 项可通过；完整套件首次仅 maintenance-stop 的 5 秒 fixture 时序超时，单项串行复验通过。
- 受管验证：copy `2eaa14c0fc9b4596b43fc5a437d04b7e`，input manifest `d648cd9f10ac9e3087bb449e16b0bc7a50dc38d619c82f804d266dd5b0c52057`，run `1143041c6ea84273aa1f33ca90c6a8b6`，`python -m py_compile tools/session_coordinator/server.py tools/session_coordinator/tests/test_server.py` exit 0。
- 受管验证边界：早先 exact-path copy/run `6e4e26818feb424e84a925e1f027310e` / `fcc20a4e2aba4f2dbd74811389893cf2` 在加载测试前因 generic copy 不含未修改 Python import closure 而 exit 1；未将其计为行为验证，也未复用已消费 copy。
- 待回传：实现 commit、post-commit rollover 与生产 planless registration 无全局 failure import 证据。
