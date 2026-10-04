---
related_code:
  - zircon_runtime/src/ui/tests/layout_slots.rs
  - zircon_runtime/src/ui/tests/layout_slots
  - zircon_runtime/src/ui/layout/pass/slot.rs
  - zircon_runtime/src/ui/layout/pass/measure.rs
  - zircon_runtime/src/ui/layout/pass/arrange.rs
  - zircon_runtime/src/ui/layout/pass/taffy_arrange.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md
reference_sources:
  - dev/bevy/crates/bevy_ui/src/layout/ui_surface.rs
tests:
  - 11 functional tests reviewed; 10 compute_layout calls and 7 surface_frame publications
  - measure source guard for profile/metadata/ordered-payload optimization present but current-source Cargo pending
  - 100/1k/10k child slot-probe and persistent-Taffy counters pending
  - 1k/10k/100k row virtual-scroll product trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI layout slots测试逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已逐文件完整阅读`zircon_runtime/src/ui/tests/layout_slots.rs`与`layout_slots/**`，共4/4个tracked Rust文件、884行、11个测试。测试执行10次`compute_layout`并发布7次`surface_frame`，覆盖linear/free/overlay/flow/grid/masonry/scroll slot的padding、alignment、order、canvas placement、z-order、clip、render、hit与fallback语义。root文件含本轮前已加入的measure源码守卫；因协调器Cargo lane尚未取得执行权，当前只计静态完成。

## PERF-MVP-260/261：功能像素不能证明规模复杂度

测试树最多4个children，能够锁定slot语义和Taffy/Zircon输出一致性，但没有记录slot entries visited、lookup count、sort/alloc，也没有稳定frame的Taffy tree create/insert/style/children/compute计数。因此它们不能验收生产路径对每child重复线性扫描全局`tree.slots`，也不能验收每容器每次arrange新建并丢弃`TaffyTree`的问题。EditorUI02仍需以generation-owned edge slot索引、共享compiled child input和persistent Taffy surface完成PERF-MVP-260/261。

## PERF-MVP-262：virtual window只验证结果

`scrollable_virtual_window_uses_visible_arranged_child_for_render_and_hit_entries`只创建4行、显示2行，验证窗口外item不进入hit grid；它未观察positions构建、offscreen visit或`hide_subtree_layout`递归。1k/10k/100k行连续滚动必须证明每步只访问visible+overscan+edge delta，工作量不随总行数增长，并覆盖focus/accessibility状态交接。

## PERF-MVP-263：局部源码守卫仍待动态闭环

root测试守卫profile flag按subtree复用、metadata不深clone以及ordered desired携带payload一次排序，是已实现止损的RED到GREEN证据；剩余children clone、axis scratch重建、wrap二次扫描和完整selection diagnostics仍缺规模门禁。该精确测试必须在当前源码Cargo通过后才可计入动态证据。

## 验收结论

本切片只补强PERF-MVP-260至263，不新增重复性能ID。EditorUI02 handoff已加入这组测试路径。current-source Cargo、规模counter、MVP workbench滚动trace、像素/RenderDoc验证完成前，4/4继续留在`pending.md`，不进入`review.md`。
