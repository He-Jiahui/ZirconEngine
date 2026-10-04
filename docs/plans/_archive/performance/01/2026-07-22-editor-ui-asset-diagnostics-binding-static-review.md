---
related_code:
  - zircon_editor/src/ui/asset_editor/diagnostics
  - zircon_editor/src/ui/asset_editor/binding
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
reference_sources:
  - dev/slint/internal/core/model/repeater.rs
tests:
  - UI asset diagnostics inventory 4/4 statically read
  - UI asset binding inventory 5/5 statically read
  - editor binding performance source contracts 4/4
  - editor performance source contracts 34/34
  - focused rustfmt and diff checks
  - current-source Windows Cargo and product-scale tests pending
doc_type: implementation-evidence
status: incremental_static_complete_dynamic_pending
---

# UI Asset Diagnostics/Binding 静态性能审查（2026-07-22）

`zircon_editor/src/ui/asset_editor/{diagnostics,binding}`当前 **9/9** Rust owner已逐文件阅读。diagnostics 4个文件只负责binding/contract/localization诊断映射；binding 5个文件负责inspector fields、payload authoring、suggestions与schema/preview projection。

## 主要瓶颈

- 单个选中binding的schema presentation每次调用`collect_binding_report(document)`处理整份document，完成后才按node/binding过滤；stable inspector refresh仍重复validation、diagnostic ownership和path index构建。
- presentation同时flatten完整nested payload、构建payload suggestions、schema defaults、preview mock values和最终String rows；payload/schema深度与集合规模直接放大主线程成本。
- node/binding selection仍通过document DFS，部分入口曾对同一选择重复查询；根本修复需消费PERF-MVP-305的generation node index。
- 现有功能测试覆盖丰富语义但fixture很小，未记录1/100/10k bindings/targets/payload leaves的report builds、tree/payload visits、clone/label bytes或p95。

## 本轮直接止损

- 删除随后被完整schema projection无条件覆盖的旧`binding_schema_items`构建及其dead literal helper。
- `build_binding_fields`复用一次payload flatten选择当前key；payload删除只materialize一次next map；两处重复selected-node DFS合并为单次borrow。
- payload/route/action单项suggestion直接消费`into_iter().nth(index)`，不再clone整binding或目标String。
- recursive payload/schema table排序借用entries，不复制全部keys；projected-key set移动既有owned key，target diagnostic nested prefix只格式化一次。

## 动态验收

需补1/100/10k nodes、bindings、targets与payload leaves，depth 1/16/64，以及stable/selection/payload edit/preview mock场景的binding report/schema/payload/suggestion build-count、tree/diagnostic/path visits、Value/String/key clone bytes、rows、RSS和main-thread p95。最终stable document/selection generation的full report/schema/payload build=0，changed binding只更新受影响artifact；route/action/payload suggestion、diagnostic path、preview/default rows、undo/redo与source roundtrip等价。还需current-source受管Cargo与F4产品trace。
