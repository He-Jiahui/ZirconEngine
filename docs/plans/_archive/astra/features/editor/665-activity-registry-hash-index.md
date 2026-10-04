---
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/01/2026-08-26-activity-registry-hash-index.md
related_code:
  - zircon_editor/src/ui/control/service.rs
  - zircon_editor/src/ui/control/service/activity_registry_hash_tests.rs
---

# Editor01 活动注册表哈希索引

Editor UI control service 的 activity view/window 注册表使用 `HashMap` 做按 ID 的直接查找，
注册通过 Entry API 避免重复哈希探测；快照出口显式按 ID 排序，保持原 `BTreeMap` 的确定性
顺序与重复注册错误。

## 计划完成列表

| 批次 | 内容 | 状态 | 验证证据 |
|---|---|---|---|
| Editor01 | view/window 哈希索引、Entry 注册与确定性快照排序 | `implemented_pending_validation` | `activity_registry_hash_tests.rs` 覆盖 lookup、排序与 duplicate rejection；Runtime/Editor 静态合同批次 `1727/1727`（`4.095s`）通过，`service.rs` 生产模块的 non-recursive Rustfmt 与 scoped diff-check 通过。当前源码二进制的三项 owner 测试在共享 selector 中通过，`EDITOR01_ACTIVITY_REGISTRY_HASH_INDEX_BENCH_V1` 报告 ordered P95 `9,824,700ns`、hash P95 `4,566,800ns`（`53.52%` 本地 Debug 降低）；受管 Editor Cargo 与 Release p50/p95 仍待异步验证。 |

本记录不宣称 Editor01 的 retained invalidation、layout/paint、OS accessibility 或产品交互验收已完成。
