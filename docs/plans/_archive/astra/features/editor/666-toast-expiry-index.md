---
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/10/2026-08-24-toast-expiry-index.md
related_code:
  - zircon_editor/src/core/notifications/toast/center.rs
---

# Editor10 Toast 到期索引

Toast center 维护按 deadline 分组的 `BTreeMap<Duration, BTreeSet<NotificationId>>`。发布与快照
清理只检查最早 deadline，到期时按组移除精确 ID；`expires_at == now` 的边界、通知 ID 顺序
和可见快照语义保持不变。

## 计划完成列表

| 批次 | 内容 | 状态 | 验证证据 |
|---|---|---|---|
| Editor10 | 到期 deadline 索引与分组清理 | `implemented_pending_validation` | `optimization_wave_20260824c_editor10_expiry_index_preserves_deadline_groups` 与边界回归已落源；Runtime/Editor 静态合同批次 `1727/1727`（`4.095s`）、Rustfmt 与 scoped diff-check 通过。10,000 条 live toast 模型将每次无到期清理从全表扫描降为最早 deadline 探测；受管 Editor Cargo 与 Release p50/p95 仍待异步验证。 |

本记录不宣称通知持久 journal、duplicate aggregation、可见生命周期或产品通知入口已完成。
