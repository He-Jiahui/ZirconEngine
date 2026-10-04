---
doc_type: milestone-detail
status: focused_validation_passed_closeout_pending
plan_sources:
  - docs/plans/astra/features/runtime/08-compile-support-repair.md
  - docs/plans/optimize/zircon_runtime/210-runtime-random-authority-stream-checkpoint-replay-consumer-performance-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/22/failure-2026-08-31-random-checkpoint-authority-generation-mismatch.md
implementation_files:
  - zircon_runtime/crates/zr_contracts/src/random/checkpoint_error.rs
  - zircon_runtime/crates/zr_contracts/src/random/service_checkpoint.rs
tests:
  - zircon_runtime/crates/zr_contracts/src/random/tests/checkpoint.rs
---

# Runtime08 · random checkpoint generation error contract

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M11-Random | 补齐 checkpoint v2 跨 authority generation 的 typed error，并确认构造、serde、restore/replay 与 eviction 正常路径 | `focused_validation_passed_closeout_pending` | 2026-09-18 | Windows managed job `f3c4cc4626fe413b9e3a86db8ee4ede6`: `zr_contracts` checkpoint 4 passed；job `0391f109a80c4cb78763bf7e8500b1ae`: Runtime random 22 passed、0 failed、1 ignored；本地 Runtime02/08/19 + Editor09 聚焦源合同批次 `39/39`；独立审查 Critical/Important/Moderate 均为 0 |

## 当前源码结论

`RandomServiceCheckpoint::validate` 使用的
`RandomServiceCheckpointError::StreamAuthorityGenerationMismatch` 已由同一 contract crate
声明，并携带 `index`、`service_generation` 与 `stream_generation`。当前源码与上述受管
回归绑定的指纹一致；2026-09-07 的 E0599 来自遗漏该脏文件的旧 source snapshot，不能
代表当前源码仍缺少 variant。后续批量验证的 source manifest 必须包含
`checkpoint_error.rs`。

| 文件 | SHA-256 |
|---|---|
| `zircon_runtime/crates/zr_contracts/src/random/checkpoint_error.rs` | `4C306BC801928CAFDCF9EE934F3DA1AA68316AEF38C36D881EB27E75465D3480` |
| `zircon_runtime/crates/zr_contracts/src/random/service_checkpoint.rs` | `34A8C276F4E2C969E2069D30DFD368D31009F7EE7B91CE09055F0C4FD0EDD739` |
| `zircon_runtime/crates/zr_contracts/src/random/tests/checkpoint.rs` | `B6D364AA58267619E393C3FC2C307DFE6C77449C13C20A0DC9B5769E271AEE45` |

本记录只关闭当前 compile-contract 缺口及其聚焦回归。未运行或声明新的性能基准；
Runtime22 failure 的正式 fixing-session closeout、failure return 与更宽 Runtime/Editor
批量验收仍保持 pending。tooling 不在本切片范围内。
