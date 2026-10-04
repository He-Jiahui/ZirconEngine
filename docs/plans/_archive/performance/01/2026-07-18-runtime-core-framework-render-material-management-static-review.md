---
related_code:
  - zircon_runtime/src/core/framework/render/material/management
  - zircon_runtime/src/core/framework/render/material/readiness_report.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors/material_diagnostics.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/08-material-shader-permutation.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/zircon_editor/editor/09-editor-asset-management.md
tests:
  - material management thirty of thirty Rust files reviewed
  - resource streamer management record-set caller chain traced
  - sort filter and selection algorithm source guards RED to GREEN
  - existing order filter paging facet and selection tests inspected
  - rustfmt and scoped git diff check passed
  - focused Cargo scale counters and editor product polling pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime render material management逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`core/framework/render/material/management/**`当前30/30个Rust文件、3,839行，包括20个production leaf、module test root与9个tests文件，并追踪resource streamer material diagnostics accessors。三处局部算法浪费已直接止损：material-name sort比较器不再为每次比较分配两份小写String，text filter不再为每个candidate分配小写副本，selection由O(K²)+O(K×N)改为保持请求顺序的HashSet去重与一次HashMap索引。剩余根因是查询前全量深快照与多次派生索引重建；当前未确认编辑器产品consumer，按基础编辑器接线前的P1合同风险记录，不冒充现有frame实测。

## PERF-MVP-360：management查询全量clone详细readiness并重复多轮派生

`ResourceStreamer::material_management_records`对全部prepared materials调用`readiness_report.management_record`；每条record深clonevalidation/fallback/diagnostic、property values、uniform fields/unsupported与两组texture states。`RenderMaterialManagementRecordSet::from_records`随后分别扫描records构造summary、status index和issue index。overview/query再clone全量compact rows，filter、full sort、三次summary/index扫描后才分页；query selection又按page ids回取并clone完整records。顶层`asset_management_record_sets`还会同时构造所有asset family record sets。若编辑器按UI刷新率调用，该成本与总materials及详细诊断大小增长，而不是与可见page或changed set增长。

Render08/17联动Editor09应在resource/material generation边界发布唯一immutable management snapshot：summary/status/issue index与compact rows一次增量构建，查询只借用stable rows并对可见page投影；完整prepared/issue details按选中id懒取或Arc共享。UI保存`generation + query key + page`缓存，stable poll零重建/深clone，changed materials只更新对应row与bucket。asset-family聚合复用各family snapshot，不在每个getter中重建全部family。

## 验收要求

按materials 1/1k/100k、details 0/10/1k fields、page 20/100、UI poll 1/10/60Hz、stable/1% changed记录management record builds、detail/row clone bytes、record visits、sort comparisons、summary/status/issue passes与CPU/RSS：stable generation全量build/clone/sort=0；changed visits近changed+page；每generation summary/status/issue build≤1；page query返回bytes与page size而非total materials成正比。现有sort大小写/None、filter、request-order dedupe、missing ids、paging/facets/actions测试及Cargo通过，并在真实Editor09 asset/material pane轮询trace验证后，方可进入`review.md`。
