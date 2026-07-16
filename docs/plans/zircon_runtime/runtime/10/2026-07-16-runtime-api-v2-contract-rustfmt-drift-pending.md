# Runtime10 V2 contract rustfmt drift pending handoff

Plan: docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
Milestone: M1.1
Status: accepted
Files: ["docs/plans/zircon_runtime/runtime/10/2026-07-16-runtime-api-v2-contract-rustfmt-drift-pending.md"]

## 产出记录与时间

| 状态 | 日期 | 失败现象、责任边界与后续完成项目 |
|---|---|---|
| `PENDING / RUNTIME10 OWNER` | 2026-07-16 | Editor02 M1.1 的全工作区 `cargo fmt --all -- --check` 被 `zircon_runtime_interface/src/tests/contracts.rs:59` 阻断。该文件包含尚未提交的 Runtime API `ZrRuntimeApiV1`→`ZrRuntimeApiV2` 合同改造且当前无 live lease，不属于 Editor02 exact 4-file scope。后续 owner 必须取得该代码文件租约，运行 scoped rustfmt，复核 V2-only ABI size/offset assertions，再完成 `zircon_runtime_interface` 受管验证与原子提交。 |
| `PENDING / PLAN MAINTENANCE OWNER` | 2026-07-16 | Runtime10 父计划仍缺 M1–M3 `zircon-workflow`、M1.1–M3.1 canonical checklist 与 M0 归档映射。业务 milestone 尝试携带父计划定义时被 `protected_plan_definition` 拒绝，因此本记录只提交 child output；plan maintenance owner 后续须用维护权限单独补齐、审计并提交父计划定义，不得把它伪装成 Runtime10 业务文件。 |

`Status: accepted` 仅表示本责任交接记录已经验收；上表两个功能/维护状态仍为 `PENDING`，不表示 `contracts.rs` 或受保护父计划提交已完成。

## Scope delivered

- 本记录固化 Editor02 外部 rustfmt 失败、受保护父计划维护失败的责任边界与后续验收条件。
- exact manifest 仅含本 child record，不包含父计划定义、`zircon_runtime_interface/src/tests/contracts.rs` 或其他 Session 文件。

## Fresh testing evidence

- 计划产出记录审计与 exact-1 `git diff --check` 通过。
- 协调器受管 `coordinator-actions` 验证以当前 exact-1 manifest 通过；功能代码验证仍留待 Runtime10 owner 修复 `contracts.rs` 后执行。

## Review

- 独立 exact-1 review 的 Critical / Important / Minor 为 `0 / 0 / 0`；review 明确认可“记录已验收、两项后续仍 pending”的双层状态。

## 失败证据

- 全工作区格式门要求把 `UiBindingExpression` 与 `UiCompileCacheKey` 合并到同一格式行；当前工作树仍保留拆行。
- 同一文件还把 runtime API table 测试从 `ZrRuntimeApiV1` / 104 bytes 改为 `ZrRuntimeApiV2` / 152 bytes，并增加 operation/plugin-event callbacks；这不是 Editor02 可以代为格式化或提交的孤立文本漂移。
- Editor02 自身的 `query.rs` 与 `world_sync_contracts.rs` 已通过 scoped `rustfmt --edition 2021 --check`，因此该 foreign failure 不进入 Editor02 M1.1 的 node failure gate。

## 后续验收

- `rustfmt --edition 2021 --check zircon_runtime_interface/src/tests/contracts.rs` 通过。
- V2-only table 的 size、offset、required callback 与旧 V1 零兼容面通过 Runtime10 独立 review。
- Windows 受管 `zircon_runtime_interface` validation 通过；提交只包含 Runtime10 current-hash manifest，不吸收 Editor02 或其他 Session 文件。
