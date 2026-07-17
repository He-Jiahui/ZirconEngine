---
related_code:
  - zircon_editor/src/ui/retained_host/primitives.rs
  - zircon_editor/src/ui/retained_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/retained_host/ui/apply_presentation
  - zircon_editor/src/ui/retained_host/ui/template_node_conversion.rs
  - zircon_editor/src/ui/retained_host/ui/workbench_window_projection.rs
  - zircon_editor/src/ui/retained_host/ui/workbench_window_projection
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/05/failure-2026-07-17-template-projection-deep-copy-and-cache-generation.md
  - docs/plans/zircon_editor/editor_ui/08/failure-2026-07-17-editor-event-full-reflection-rebuild.md
reference_sources:
  - dev/bevy/crates/bevy_ui/src/layout/ui_surface.rs
  - dev/slint/internal/core/model/repeater.rs
tests:
  - model_mapping_borrows_source_rows
  - visible_welcome_size_borrows_floating_window_rows
  - workbench_projection_uses_a_memoized_node_index
  - workbench_projection_memoizes_collapsed_ancestor_visibility
  - workbench_projection_treats_parent_cycles_as_not_render_visible
  - existing presentation/workbench projection parity suites
  - current-source Windows zircon_editor focused Cargo pending
  - 10k-node property clone-byte/deep-tree/pane-patch trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor retained presentation core 逐文件性能静态审查（2026-07-17）

## 范围与覆盖

本切片完整覆盖 `ui/apply_presentation.rs`及其2个子模块 **3/3**、`ui/workbench_window_projection.rs`及其4个子模块 **5/5**、`ui/template_node_conversion.rs` **1/1**，合计 **9/9** 个 Rust 文件。动态 Cargo、current-source retained editor trace、deep-tree/large-pane scale和clone-byte门禁尚未完成，范围继续留在 `pending.md`。

## 已直接优化

- PERF-MVP-133：`ModelRc::iter`让template mapping借用源行；options直接写入结果String，不再row clone、二次String clone、临时Vec和join；floating Welcome尺寸查找借用窗口行。
- PERF-MVP-134：workbench node id改为一次HashMap index；祖先visibility与最近controlled parent一次memoize，删除每node父链BTreeSet和visibility replace/lowercase分配。cycle明确不渲染，duplicate id保持map last-write语义，输出仍按projection原顺序。
- 根据 `modularize-large-files`，索引责任抽到 `workbench_window_projection/node_index.rs`，主converter从963行降到906行；没有增加新的misc/helper堆叠。
- menu item先生成structured rows，再把原Vec移入host model，删除整批menu String clone。

源码RED→GREEN、`rustfmt`和`git diff --check`已过；这些不是动态验收。

## 已移交架构热点

- PERF-MVP-135/EditorUI05：每个可见node把完整retained property BTreeMap递归深转TOML，随后style/options/canvas和数十个scalar helper再次读取/格式化。需要compiled typed property view与changed-node generation，不能在converter加无失效cache。
- PERF-MVP-106/135/EditorUI08：`apply_presentation`深clone完整旧presentation只为保留交互字段；四dock/floating pane clone中间DTO，最后整份replace。结构generation、interaction state与pane/window delta必须分离。
- `root_template_overlay`通常为空；显式overlay仍按node调用preview image loader。媒体缓存/asset generation应由既有template/asset owner提供，不能由root overlay私建第二份image cache。

## 参考实现约束

Bevy UI surface以stable entity hash index访问layout nodes；Slint repeater以dirty row和可见实例增量更新。Zircon本轮采用相同的稳定identity/memoization原则处理父链，但完整验收仍要求property和presentation也由generation/delta驱动，而不是只把O(N×D)改为O(N)后继续每event重建。

## 待动态验收

协调器下运行9个文件的focused tests及相关retained projection suites；构造1/100/10k深树、collapsed ancestor、cycle、control-less wrapper、large options/menu、四dock/floating pane场景，记录visited nodes、String/TOML/row clone bytes、full presentation get/set、pane conversion count和p95。全量/增量输出bytes、视觉、route、focus、viewport image和interaction state parity通过前不进入 `review.md`。
