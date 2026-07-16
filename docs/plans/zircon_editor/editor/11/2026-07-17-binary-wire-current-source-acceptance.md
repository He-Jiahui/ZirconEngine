---
plan_source: docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
related_code:
  - zircon_runtime_interface/Cargo.toml
  - zircon_runtime_interface/src/serialization/
tests:
  - zircon_runtime_interface/src/serialization/tests/binary_contract.rs
  - zircon_runtime_interface/src/serialization/tests/binary_malformed_contract.rs
  - zircon_runtime_interface/src/tests/boundary.rs
status: slice-accepted-managed-native-slice-closeout-ready
---

# Editor11 M3.1 二进制 wire 当前源验收

Plan: docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
Milestone: M3.1
Status: accepted
Files: ["docs/plans/zircon_editor/editor/11/2026-07-17-binary-wire-current-source-acceptance.md"]

## 产出记录与时间

| 时间 | 里程碑/切片 | 状态 | 完成项目与证据 | 后续门禁 |
| --- | --- | --- | --- | --- |
| 2026-07-17 | M3.1 Binary 编码选型与文本等价合同 | `当前源验收-281/281-独立复核待current-hash绑定-原生切片受管关闭就绪` | M3.1 exact25 已由集成提交 `ad2c6f989cfff927ff5679467ca0cc71e2e20c0e` 吸收；验证基线 `6c8957ab86925d0eed3e55a5914157dc891f382e` 至关闭前 HEAD `a381a9bf61ce49f4ef1a386bba01db06101127df` 之间，M3.1 业务代码、测试、既有 child record 与模块文档相对该集成提交无差异；父计划状态是独立的受保护 plan-maintenance 范围，不冒充业务实现。业务 milestone manifest 仅包含本验收记录。33 个 M2.2/M3.1 唯一 Rust 文件的 scoped `rustfmt --edition 2024 --check --config skip_children=true`、scoped `git diff --check` 与计划产出审计通过。Windows canonical reservation `e9437045935c4bcb9dee7268389d5250`、job `f3aaaf1603584611830544ed94a2d1c0`、run `9e37aea93aa64c079bd42cb327317070` 在 target `F:\cargo-targets\zircon-engine\pool\4069e3844d425556a8a455a02feb84e651e896a66296cf6d2d98424da29bde30` 执行 `cargo test -p zircon_runtime_interface --locked --offline -- --test-threads=1`；library 278/278、integration 3/3、doc-tests 0/0，合计 281 passed / 0 failed、exit 0，job 已 `released` 且 `live_process_pids=[]`。M2.2 已由业务 milestone commit `b3842c76` 与父计划维护 commit `a381a9bf` 关闭。 | M3.1 切片当前源已验收；父 M3 仍等待 Plan15 M2.1 所拥有的 CookAssets 二进制消费点、5k-entity 场景体积/耗时基线与完整 M3 matrix，不得在 Plan11 内复制 Python/Rust cook owner 或提前完成 M3.2。Plan11 的 `failure open` 为空；本节点只提交本记录，父计划 M3.1 复选框/链接另走 plan-maintenance，不重复提交已被集成吸收的 exact25 实现。 |

## 架构验收结论

- 永久 wire-v1 固定为 `ZRPAYLD\0`、little-endian `u16` wire version、显式 header 与 flat node stream；field/variant 顺序由 golden bytes 锁定，变更必须提升 wire version。
- Text/Binary 共用 `PayloadHeader`、`VersionedSchema`、迁移链和 JSON 可达值域；writer/reader 共用大小、节点、深度与有限浮点限制，不保留 `UnsupportedFormat(Binary)`、临时 reader 或类型化石 DTO。
- M3.2 的 cook consumer 属于 Plan15 资产流水线 owner；本切片只提供中性 serialization backend，不在 Editor、Python exporter 或 runtime scene 中复制 wire 实现。

## Scope Delivered

- 业务关闭清单仅包含本记录；父计划 M3.1 状态同步由受保护 plan-maintenance 路径提交。M3.1 exact25 实现已在 `ad2c6f989cfff927ff5679467ca0cc71e2e20c0e` 吸收。

## Fresh Testing Evidence

- Windows 受管作业 `f3aaaf1603584611830544ed94a2d1c0` / run `9e37aea93aa64c079bd42cb327317070` 通过 281/281；关闭前 HEAD 的 M3.1 业务实现、测试与模块文档对集成提交无差异。

## Review

- 最终 current-hash independent review evidence 写入协调器；父 M3 保持 pending，M3.2 仍由 Plan15 owner 推进。
