---
related_code:
  - zircon_runtime_interface/src/ui/tree
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
reference_sources:
  - dev/slint/internal/core/item_tree.rs
  - dev/slint/internal/compiler/llr/lower_to_item_tree.rs
tests:
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime_interface/src/tests/ui_contract_spine.rs
  - zircon_runtime/src/ui/template/build/tree_builder.rs
  - current-source Windows tree/template tests pending
doc_type: implementation-evidence
status: partial_static_complete_dynamic_pending
---

# Runtime interface UI tree clean subset 性能静态审查（2026-07-23）

## 范围与覆盖

`zircon_runtime_interface/src/ui/tree/**`当前 **7/11** 个clean Rust文件、**202** 行已逐文件阅读；外部dirty `mod.rs`、`node/mod.rs`、`node/tree_node.rs`及foreign untracked `node/pointer_events.rs`未读取、未吸收。已反查runtime template tree builder与大量surface/test插入调用。本轮未修改Rust源码。

## 性能结论

- **PERF-MVP-573 / P0**：`UiTree::insert_root()`和`insert_child()`每次都调用`next_paint_order()`全扫现有nodes求max；runtime template builder对每个compiled node逐项插入，因此N节点构树累计O(N²) paint-order visits。bulk builder应直接按canonical output order赋值；动态树维护generation-owned O(1) cursor，deserialize/import只允许一次重建，不能增加第二套排序truth。
- `UiTemplateNodeMetadata`继续以component/control/classes、四张String/TOML map、bindings、a11y/widget拥有完整immutable metadata，回链 **PERF-MVP-274/276/312**；runtime tree只应持compiled generation handle和mutable state/delta。
- `UiTree`节点通过BTreeMap定位，插入自身为O(log N)，但roots/slots仍为Vec合同；slot索引/arranged查询继续归 **PERF-MVP-260/277**，本clean subset未发现新的独立问题。
- `UiDirtyFlags`、visibility/input policy、layout cache均为Copy紧凑值，visibility判断为const match，是正向基线；tree error只在失败时格式化。
- Slint `ItemRc`/`ItemTreeNode`使用shared item-tree owner与u32 indices，compiler lowering一次生成typed index mapping。Zircon采用dense generation order与derived O(1) cursor原则，不复制其vtable布局。

## 动态验收

1. tree nodes 1/100/10k/100k、roots 1/100、insert/remove/reinsert：记录paint-order node visits/comparisons、BTree probes、wall p95与RSS；bulk paint-order max scans=0，总构建不含O(N²)阶段。
2. deserialize/import/current build：记录cursor rebuild passes、owners和published order；每generation最多一次O(N)重建，随后insert O(1)取order，paint order/serde golden保持一致。
3. metadata 0/1/100 KiB、nodes 1/100/10k：记录String/TOML clone bytes、owners与surface build p95；immutable metadata owner=1。
4. current interface合同、runtime template/tree/layout/input/render tests及F4 large `.zui` preview trace通过。

current-source Cargo、规模counter、F4产品trace及4个dirty/untracked文件独立审查未完成，因此该目录继续保留在 `pending.md`，不进入 `review.md`。
