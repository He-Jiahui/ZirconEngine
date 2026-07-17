---
related_code:
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_drag_overlay.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_drag_overlay/**/*.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
reference_sources:
  - dev/slint/internal/core/item_rendering.rs
tests:
  - no dedicated drag-overlay test module present
  - current-source Windows Cargo baseline failed on unrelated source guards
  - inactive and 1000-move drag overlay build/text/geometry/command trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Editor template drag overlay逐文件性能静态审查（2026-07-17）

## 范围与覆盖

`template_drag_overlay*`共 **9/9** 个Rust文件、**303** 行已逐文件阅读。覆盖identity/inactive gate、preview/indicator geometry、fixed style、surface/icon/text commands。当前没有本组专用测试；current-source baseline全批次另有2个无关source-guard漂移，pointer-move产品trace未完成，因此仍留在`pending.md`。

## P1：高频move复制不变label并重建完整overlay

Inactive drag会在任何preview/style/text工作前退出，行为正确。Active path算术O(1)、不读theme lock，且固定最多surface/icon/text/indicator四条commands，不存在无界command风暴。然而每次pointer move都从四个候选字段复制同一payload label并重建所有commands，虽然通常只有x/y和drop target geometry变化。

PERF-MVP-210要求drag start/change generation构建`DragOverlayPaintSpec`，持有shared label、drop-allowed style和固定尺寸；pointer move只patch preview/icon/text frames与indicator动态段，不重新复制label或重建静态payload。PERF-MVP-178拥有compiled segment，native pointer damage仍由PERF-MVP-163/176限定。

## 动态验收

新增inactive/active/allowed/blocked/with-without-indicator聚焦测试，并在1,000次same-payload pointer move及100次payload/target切换记录identity、label bytes、static/dynamic build、commands、damage与frame p95。Inactive全部=0；same-payload move label copy/static build=0且仅patch必要geometry；每帧commands<=4。保持fallback label priority、cursor offsets、target edges、colors、clip/order和GPU/Softbuffer pixels parity。
