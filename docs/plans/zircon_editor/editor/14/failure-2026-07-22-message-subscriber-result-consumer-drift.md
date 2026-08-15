---
handoff_kind: failure
status: open
created_at: 2026-07-22
summary_slug: message-subscriber-result-consumer-drift
origin_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
fixing_plan: docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
origin_child_dir: docs/plans/zircon_editor/editor/02
fixing_child_dir: docs/plans/zircon_editor/editor/14
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/jobs/tests/background_storm_contract.rs
  - zircon_editor/src/core/jobs/tests/pump_contract.rs
tests:
  - editor jobs pump and background storm compile against checked subscriber registration
  - existing pump timing, lifecycle ordering, progress and storm behavior remains green
---

# Editor14: message subscriber checked registration consumer drift

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 来源执行切片：Editor02 checked subscriber registration hard cut consumer migration（Session `editor02-message-inbox-backpressure-r6-20260722`）
- 修复责任计划：`docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md`
- 交接原因：Editor02 为消除 subscriber ID 饱和复用，将 `register_subscriber` 硬切为 typed `Result`；jobs 测试属于 Editor14，且 `pump_contract.rs` 同时包含其未提交的 elapsed-clock、terminal progress 与并发 pump 业务回归，不能被 Editor02 business commit 吸收。

## 失败现象与复现证据

两个 jobs consumer 仍按旧 infallible 签名保存 `EditorSubscriberId`。Editor02 API 硬切后，它们会把 `Result<EditorSubscriberId, EditorMessageBusError>` 传给 delivery drain/stats，形成编译错误。不能在 Editor02 exact manifest 中整体提交这些文件，因为那会同时吸收 Editor14 的独立 jobs 改动。

当前工作树已完成最小 consumer 迁移：每个注册点显式 `.unwrap()`，测试夹具继续把注册失败视为不可恢复的测试环境错误；生产 jobs API 未新增兼容入口。该源码修复尚未取得 Editor14 独立 snapshot/Cargo/review，因此 failure 保持 open。

## 最低共享层根因

Editor02 已把 subscriber registration 硬切为 typed `Result`，但 Editor14 jobs 测试仍按 infallible ID 消费，且这些测试文件同时承载 Editor14 独立业务回归，不能归入 Editor02 commit。

## 架构修复验收

- 两个 jobs test 文件由 Editor14 Session 独立归属和提交，Editor02 manifest 不包含它们。
- focused jobs pump/background storm 与 broader editor message/job 门通过，证明 Result 迁移未改变 lifecycle 顺序、配额、进度 coalesce 或 pump budget。

## 禁止临时方案

不恢复 infallible registration wrapper，不用 sentinel subscriber ID，不让 Editor02 commit 吸收 Editor14 其它脏改动。

## 修复结果与回传

Open state: `checked registration consumer 源码已迁移并通过静态复审；等待 Editor14 focused/broad Cargo、source snapshot 与 fixed return`。

## 产出记录与时间

| 日期 | 状态 | 完成项目与证据 |
| --- | --- | --- |
| 2026-07-22 | open / consumer source patched / validation pending | 两个 jobs test 的 checked registration consumer 已迁移；exact3 Session `editor14-message-subscriber-result-consumer-r1-20260722` 已建立并持有对应租约。等待 Editor14 snapshot、focused/broad Cargo、独立复审与 fixed return。 |
| 2026-07-22 | source frozen / static green / validation pending | exact3 中 8/8 `register_subscriber` 测试注册点均显式处理 typed `Result`；两个 Rust 文件经 rustfmt，scoped diff-check 通过。生产 jobs 实现未改，受管 Cargo 尚未运行；等待独立复审、source snapshot、focused/broad Cargo 与 fixed return。 |
| 2026-07-22 | review clean / managed Cargo pending | 独立只读复审 `Critical/Important/Minor = 0/0/0`；确认 8 个 `.unwrap()` 全部位于测试 setup，注册失败会显式失败测试，不恢复生产 infallible wrapper 或 sentinel。归属仍为 Editor14 exact3，受管 Cargo 与 fixed return 尚待完成。 |
| 2026-08-10 | source frozen / focused managed validation queued | exact3 已封存为 snapshot `1588`；合并当前 Editor14/17 jobs shared snapshots 的 focused `core::jobs::tests` 验证 receipt 为 `78b516ef40384038b373e64da58653ca`。 | receipt 状态为 `queued`，未轮询、不把提交 receipt 当作 Cargo 通过；failure 保持 `open`，等待 terminal evidence 与 return。 |
