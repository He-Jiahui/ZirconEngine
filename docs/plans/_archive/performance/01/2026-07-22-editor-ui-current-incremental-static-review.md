---
related_code:
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard
  - zircon_editor/src/ui/retained_host/app/build_export_projection
  - zircon_editor/src/ui/retained_host/app/welcome_session
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench
  - zircon_editor/src/ui/sample_grid
  - zircon_editor/src/ui/workbench/asset_content_layout
  - zircon_editor/src/ui/workbench/project
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor/06-ui-extension-framework.md
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
reference_sources:
  - dev/godot/editor/editor_node.cpp
  - dev/godot/editor/export/editor_export_platform.cpp
tests:
  - tools.tests.test_editor15_export_generation_inventory_contract 10/10
  - tools.tests.test_editor06_workbench_toolbar_priority_contract 1/1
  - current-source Windows Cargo and F0/F4 product traces pending
doc_type: implementation-evidence
status: incremental_static_complete_dynamic_pending
---

# Editor UI当前新增owner性能静态审查（2026-07-22）

## 范围

当前物理UI清单为4,211个Rust文件；相对Git index有17个新增owner与9个删除owner。本轮完整阅读新增 **17/17**：export output capture/tail、Build/Export projection cache、Welcome project probe、asset creation menu、toolbar priority/run state、asset compact column、welcome geometry、sample-grid generation/tests、asset-content paint metadata、resolution context与project load/save。旧计划4,193计数需按其既有分片继续核对，本记录不宣称整个UI动态通过。

## 直接止损

- export `IncrementalLineBuffer`新增suffix scan cursor；1-byte chunk只回看最后一个待判定字节，并在pending为空时直接接管输入Vec，删除16KiB partial line近O(line²)重扫。Editor15静态合同10/10。
- toolbar priority原为约39个control各自全扫`tree.nodes.values()`；本轮每次layout只建一次借用`ToolbarControlIndex`，后续哈希查询。Editor06新合同1/1。

## Open热点与回链

- PERF-MVP-107：Build/Export cache命中仍逐preset metadata并深clone base/pane DTO；需要source generation与共享snapshot。
- PERF-MVP-558：output tail满后每行`Vec::remove(1)`搬移约511个String，line同时clone给tail/event；stdout、stderr、manifest结束时顺序`sync_all`，需ring/shared line owner及Runtime11持久化ticket。
- PERF-MVP-559：Welcome每次draft变化立即cancel+submit，probe中间没有取消点；输入风暴可让大量过时代际job进入Editor14队列。
- PERF-MVP-560：asset creation menu每次layout重建labels/count/map/set，action click又为单个action重建整map；需template/asset generation发布稳定action index。toolbar tree scan已局部止损。
- PERF-MVP-214已具immutable sample-grid generation/preformatted tick；batched grid/marker primitive仍open。PERF-MVP-219已把asset paint identity/visible groups装入ModelRc metadata，但generation build仍分配String-key BTreeMap，visible查询还clone fixed rows并排序，应继续改为compiled typed slot/ordered ranges。
- project load/save仍同步scene load/serialize/write，复用PERF-MVP-075/453；不得重复立项。

current-source Cargo、preset/output/input storm规模counter、F0 project select/open、F4 asset/menu/export/viewport产品trace与像素未完成，所有相关目录继续留在`pending.md`。
