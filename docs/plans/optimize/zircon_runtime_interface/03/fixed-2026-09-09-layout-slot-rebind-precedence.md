---
handoff_kind: fixed
failure_scope: local
status: fixed
created_at: 2026-09-08
summary_slug: layout-slot-rebind-precedence
origin_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
fixing_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
origin_child_dir: docs/plans/optimize/zircon_runtime_interface/03
fixing_child_dir: docs/plans/optimize/zircon_runtime_interface/03
plan_link_mode: child_record_only
related_code:
  - zircon_runtime_interface/src/ui/tree/node/ui_tree.rs
tests:
  - cargo test -p zircon_runtime_interface --no-default-features --locked --lib ui::tree::node::ui_tree::tests::
  - cargo test -p zircon_runtime_interface --no-default-features --locked --lib
resolved_at: 2026-09-09
---

# Interface03: preserve slot precedence after rebinding an existing edge

## 来源执行者

- 来源计划：`docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md`
- 来源执行切片：完整接口库受管测试中的 retained layout slot 回归。
- 修复责任计划：`docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md`
- 交接原因：唯一 slot edge authority 位于共享 `UiTree`；Runtime 的
  `ui/layout/pass/slot.rs` 直接消费其 first-index 与 kind-index 查询。

## 失败现象与复现证据

Windows 受管作业 `f95f64a6a06345d3940884140d9e3e50` 执行完整接口库，
739 passed、23 failed、101 ignored；
`slot_rebind_preserves_flat_slot_precedence_on_an_existing_edge` 实际失败。
slot 0 从 child 1 重新绑定到已有 slot 1 的 child 2 后，查询返回 `Some(1)`，应为 `Some(0)`。
输入 `interface-library-consumers-3137-20260908` 的 manifest 为
`484b58e5512bb5619941864b4906bbfaaeef0cff78f89267586fe6e4b00d2b63`；
原始日志和受管 receipt 保留在该输入的 `results/interface-library-3137.{json,log}`。

## 最低共享层根因

`UiLayoutSlotAuthority::rebuild` 与 `insert_if_initialized` 保持每条 edge 的 slot index 升序，
`rebind_if_initialized` 却对目标 edge 使用尾插。已有目标 slot 时，索引查询的优先级与
flat slot 源数据不一致；按 kind 的查询也会受影响。

## 架构修复验收

- 原始 rebind 复现通过，同时覆盖 first-index、kind-index、旧 edge 消失与 rebuild count 不增加。
- 原有反序列化单次重建、same-cardinality rebind、bulk retention、布局失效及 10k 节点回归通过。
- 完整接口库验收和独立审查完成；Runtime slot 消费者继续通过同一共享查询。
- 不将无关 paint-order release 性能项的 ignored 状态解释为通过。

## 禁止临时方案

- 不改优先级断言、不全量重建来掩盖增量维护错误、不增加 Runtime 侧第二份 slot authority。

## 修复结果与回传

- 根因：UiLayoutSlotAuthority::rebind_if_initialized appended a replacement edge at the tail, so first-index and kind-index queries diverged from flat slot precedence when the destination edge already existed.
- 架构修复：After removing the old edge membership, rebind reuses insert_if_initialized to preserve ordered first-index and kind-index authority without a full rebuild or a second Runtime slot model; regressions cover old-edge removal, kind lookup and stable rebuild count.
- 验证：Managed Windows interface job b6fbfec9ae5f4dc0bf766460167377a4 executed the original rebind regression plus the new kind-index and old-edge assertions; full library result was 744 passed, 18 failed and 101 ignored. Existing independent review reported Critical 0, Important 0, Moderate 0 for the source snapshot; unrelated full-library failures and ignored performance gates remain explicit.
- 回传：Returned the shared UiTree slot-precedence repair at source hash 26fa8cb3b147be23f8b453bac829f277108310fadb0e800679b813f87ff812c4. The incremental ordered insertion path is validated for rebind, while no full-library green claim is made.
