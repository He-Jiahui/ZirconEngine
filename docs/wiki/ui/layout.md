---
related_code:
  - zircon_runtime/src/ui/layout/mod.rs
  - zircon_runtime/src/ui/layout/pass
  - zircon_runtime/src/ui/layout/taffy_bridge
  - zircon_runtime/src/ui/layout/virtualization.rs
implementation_files:
  - zircon_runtime/src/ui/layout/constraints.rs
  - zircon_runtime/src/ui/layout/scroll.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine UI Wiki
tests:
  - zircon_runtime/src/ui/tests/taffy_layout_pass.rs
  - zircon_runtime/src/ui/tests/taffy_visual_verification.rs
  - zircon_runtime/src/ui/tests/scroll_virtualization.rs
doc_type: module-detail
---

# 布局与虚拟化

## 布局阶段

`compute_layout_tree` 按 `UI_LAYOUT_PASS_ORDER` 执行：输入快照、样式映射、文本测量、Taffy 计算、几何发布、arranged tree 修补和虚拟窗口更新。增量入口 `compute_incremental_layout_tree_with_text_measure_cache` 只处理受 dirty domain 影响的节点。

`taffy_style_from_ui_layout_style` 将引擎的轴约束、容器族、间距、显示模式映射到 Taffy。支持 flex、grid、stack/overlay、滚动容器以及固定/自适应尺寸。

## 虚拟列表

`compute_virtual_list_window` 根据滚动偏移、viewport、item extent 和 overscan 计算 materialized range；`UiVirtualListSlotMap` 保持逻辑索引到节点槽位的稳定映射。原型池 (`UiVirtualListPrototypePool`) 复用节点，减少滚动时分配。

```rust
use zircon_runtime::ui::layout::{compute_layout_tree, compute_virtual_list_window};
let window = compute_virtual_list_window(scroll_offset, viewport_extent, item_extent, count, overscan);
compute_layout_tree(&mut tree, available_size)?;
```

## 约束与限制

离散 track 数、slot 坐标/span、overscan 均受 4096 上限保护；负数、NaN 或无限几何会被拒绝。虚拟化只保证窗口内节点可交互，业务不得缓存已回收槽位的 `UiNodeId` 作为永久引用。
