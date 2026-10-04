---
handoff_kind: failure
status: open
failure_scope: cross_plan
plan_link_mode: child_record_only
created_at: 2026-08-13
summary_slug: editorui10-test-budget-message-runtime-event
origin_plan: docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md
fixing_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
origin_child_dir: docs/plans/zircon_editor/editor_ui/10
fixing_child_dir: docs/plans/zircon_editor/editor/02
related_code:
  - zircon_editor/src/tests/editor_event/runtime/integration
  - zircon_editor/src/tests/editor_message/bus/backpressure
  - zircon_editor/src/tests/runtime_event_consumer_bounded_pump
  - tools/tests/test_editor02_runtime_event_consumer_bounded_pump_contract.py
tests:
  - python -B .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_editor_structure.py --json --repo-root E:\\Git\\ZirconEngine
  - cargo test -p zircon_editor --lib editor_event --locked
  - cargo test -p zircon_editor --lib editor_message --locked
  - cargo test -p zircon_editor --lib runtime_event_consumer_bounded_pump --locked
  - python -m unittest tools.tests.test_editor02_runtime_event_consumer_bounded_pump_contract -v
  - rustfmt --edition 2024 --check zircon_editor/src/tests/runtime_event_consumer_bounded_pump/mod.rs
---

# Editor02: messaging and runtime-event test owners exceed the 800-line budget

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor_ui/10-code-structure-and-module-conventions.md`
- 来源执行切片：M3.T1 test-file budget gate
- 修复责任计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 交接原因：runtime event、message backpressure 和 bounded pump 是 Editor02 data-sync/messaging 行为，必须在该计划内保持测试边界。

## 失败现象与复现证据

结构审计报告 0 个豁免下的 3 个 Editor02 owner：
`tests/editor_event/runtime/integration.rs`（961 行）、`tests/editor_message/bus/backpressure.rs`（964 行）和
`tests/runtime_event_consumer_bounded_pump.rs`（983 行）。它们使 zero-tolerance structure gate 保持 RED。

## 最低共享层根因

runtime event integration、bus backpressure 与 bounded-consumer pump 的独立时序/容量行为被持续追加到 flat
测试 owner，缺少按协议、capacity 和 event-loop 行为划分的 folder-backed 测试边界。

## 架构修复验收

- 将三项按单一 messaging/runtime-event 行为拆分为 folder-backed tests，薄 `mod.rs` 挂载，所有文件不超过 800 行。
- 保留 backpressure、bounded pump、runtime event 顺序与容量断言语义；共享 fixture 必须唯一归属。
- 不得留下 `#[path]` mount、旧 flat 文件、duplicate test tree 或 budget exemption。
- 重审计不再报告这三项；全部 owner 清零后受管 structure gate 才能 GREEN。

## 禁止临时方案

- 不得提高预算、删除时序/容量覆盖或将消息测试移入无关 host/UI 计划。

## 修复结果与回传

Open state: `源码修复完成，受管动态验收待完成`。稳定 Session：`failure-roll-01a07160-editor02`。

- 2026-09-06 现行源码核对：前两项已采用 `integration/`、`backpressure/` 目录，薄根分别为 4/3 行，最大子文件为 344/679 行；复用已有迁移，保留上方原始历史路径和行数。
- 本轮将当前 830 行的 bounded-pump flat owner 迁入规范目录。根文件为 8 行声明；共享 consumer/gateway 夹具唯一归属 `support.rs`，ABI allocation/backlog 夹具唯一归属 `abi_fixture.rs`。所有 9 个文件不超过 539 行，没有旧 flat 文件或 `#[path]` mount。
- 23 个 Rust 测试、83 个函数的函数体在格式化归一后逐项一致；两个 ignored 的 1K/10K ABI 性能测试保留原名称和 ignore 条件。此核对仅证明迁移保真，不代表动态测试已执行。
- 修正 Python 合约守卫对拆分前 host、pending、lifecycle 的路径假设，按当前 `PendingDelivery`、借用 RawValue 解码及 typed-panic 隔离契约校验；全部 7 项本地检查通过。当前文档的测试路径同步迁至规范目录。
- 当前结构审计不再报告原始三项，剩余 14 个超预算测试文件、3 个生产文件、2 个重复测试树，豁免数为 0。全仓结构门禁仍未通过。
- 按用户指令跳过 `zr_vm`，不重复已知 Cargo 准入阻塞；`editor_event`、`editor_message`、bounded-pump 受管 Rust 回归、原始性能验收和独立审查仍待完成。未生成 fixed 回传、closeout SHA 或企微通知。
- 精确源码快照为 `2818`（18 个路径，含旧 flat 删除）；受管源码合约请求 `failure-roll-01a07160-editor02-pump-source-contract-20260906-r1` 已领取票据 `7e590a8f3e58466ca630cd03361f39f0`，准入无 blocker。该票据仅执行明确的回归清单/round-robin 源码合约用例，回执当时为 `queued`，不代表 Rust 动态验收通过。生产 host 和 registration 另有会话改动，未混入本 Session 的验证 overlay。
- 后续任务边界收取该票据结果：`passed`，实际执行 1 项、0 失败（0.006 秒），退出码 0，job `cfc55338c6d94a94882256eb04c411b9`。11 个验证输入的当前哈希全部匹配票据，manifest hash 为 `83dd1b1d2c9ae57fc1ee6636849bb4f14f9bb2a01d3cc6d85c23bf0d95fb6c06`；该源码合约通过不替代尚待执行的 Rust 与性能验收。

## 产出记录与时间

| 时间 | 里程碑/切片 | 状态 | 完成项目与证据 | 后续门禁 |
| --- | --- | --- | --- | --- |
| 2026-08-13 | M3 messaging/runtime-event test-budget handoff | `open` | 从准确 48/0 审计隔离 3 个 Editor02 owner，均超过 960 行。 | 取得源码 lease 后 folder-backed 拆分，受管 messaging/runtime-event 回归和结构审计复验。 |
