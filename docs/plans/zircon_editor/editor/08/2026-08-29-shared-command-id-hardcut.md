---
plan: docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
child_plan: docs/plans/zircon_editor/editor/08
status: source_complete_validation_pending
date: 2026-08-29
---

# Editor08 Shared Command ID Hard Cut

## 目标

消除 plugin wire DTO、plugin SDK 与 editor host 对 command/operation ID 的重复语法实现，以一个共享的类型化 identity 契约阻断历史两段式 ID，确保注册、菜单引用、执行路由和远端操作使用同一条三段式稳定路径。

## 架构产出

- `zircon_runtime_interface::EditorCommandId` 是 wire boundary 的唯一 command ID grammar owner；单次 byte scan 校验 ASCII lowercase/digit/underscore segment，至少三个非空 dot segment。
- `EditorOperationPath` 包装共享 `EditorCommandId`，不再复制 host-local parser；serde、显示、查找键和调用 DTO 保持字符串 wire representation。
- `SerializedContributionBatch::new` 原子校验 command contribution ID 与 menu command reference；非法旧 ID 通过 `InvalidCommandId` 拒绝，不能进入 materializer 或 registry。
- plugin SDK 保留链式字符串 authoring ergonomics，但 `.build()` 必须经过共享 batch 校验，因此 SDK 和直接 DTO 构造无法绕过同一 hard-cut gate。
- 第一方 runtime/SDK/materializer 测试 fixture 已迁移到三段式稳定 ID；两段式 `sample.command` 只保留为明确的负向兼容拒绝证据。

## 结构复审结论

原实现把同一个 identity contract 分散在 host parser、wire DTO 字符串和 SDK builder 中，导致 wire/SDK 可生成 host 必然拒绝的两段式命令。该问题属于跨边界契约缺失，不是局部字符串校验性能问题；本次先把 grammar owner 下沉到最小共享 runtime interface，再由 host newtype 保留领域语义。

## 验证计划

- 静态：Rust 2021 rustfmt、scoped `git diff --check`、两段式 fixture 扫描、共享 parser source ownership 扫描。
- 受管：在 `E:\cargo-targets\editor08-command-id` 运行 runtime interface、plugin SDK 与 editor operation/materializer 定向测试；不在 C 盘生成 target。
- 独立复核：验证 serde wire shape 不变、批次校验原子性、菜单引用与 command registration 使用同一 grammar，并确认没有 legacy fallback。

## 后续边界

共享 ID 只解决 identity contract。serialized plugin command 当前仍缺少可执行 endpoint/factory owner，后续必须先复审 native plugin callback/manifest/runtime ticket 拓扑，再设计 ticket-owned executor binding；不得用临时空 factory 或字符串分派掩盖该缺口。

## 产出记录与时间

| 日期 | 项目 | 状态 | 证据 |
| --- | --- | --- | --- |
| 2026-08-29 | shared command ID grammar owner | `completed` | 新增 `zircon_runtime_interface::EditorCommandId` 与错误类型；单次 byte scan、三段式 grammar、serde golden fixture 已落地；`EditorOperationPath` 委托共享 parser。 |
| 2026-08-29 | wire/SDK batch hard cut | `completed` | `SerializedContributionBatch::new` 同时校验 command ID 与 menu command reference；runtime interface 和 SDK 均新增 `sample.command` 拒绝测试，合法 fixture 全部迁移为三段式 ID。 |
| 2026-08-29 | source static gates | `completed` | touched Rust files `rustfmt --edition 2021` pass；scoped `git diff --check` pass；目标范围 `fixture.command` 命中 `0`，`sample.command` 仅命中负向拒绝测试。 |
| 2026-08-29 | managed Cargo validation and independent review | `pending` | `cargo.acquire` 已被协调器接受但回执对账超时；按非阻塞策略继续 executor 架构复审，不声明测试、C/I/M、commit 或企微完成。 |
