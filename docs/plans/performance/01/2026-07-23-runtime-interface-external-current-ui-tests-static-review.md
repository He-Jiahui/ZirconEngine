---
related_code:
  - zircon_runtime_interface/src/ui
  - zircon_runtime_interface/src/tests
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/slint/internal/core/partial_renderer.rs
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Rendering/ElementBatcher.cpp
tests:
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime_interface/src/tests/render_contracts.rs
  - zircon_runtime_interface/src/tests/input_response_contracts.rs
  - zircon_runtime_interface/src/ui/focus/focus_tests.rs
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime interface external-current UI/tests 性能静态审查（2026-07-23）

## 范围与覆盖

本批按当前hash只读完成 `zircon_runtime_interface/src/ui/**` 的18个dirty tracked+10个foreign untracked文件（**4,001**行），以及`src/tests/**`的6个dirty tracked+2个foreign untracked文件（**5,546**行），合计 **36** 文件、**9,547** 行；不吸收、不修改外部owner源码。结合此前批次，interface物理current source **391/391** 个Rust文件已静态阅读，其中UI **218/218**、tests **31/31**。

## 性能结论

- `focus_chain()`每次递归整树、用`BTreeSet`防环、收集全部candidate并排序；大树每次Tab仍为O(N log N)，直接补强 **PERF-MVP-253**。tree/layout generation应发布预排序focus index，按键只查scope/neighbor。
- `UiHitPath`同时持root-to-leaf与反向bubble Vec；`UiDispatchReply::merge_route()`无条件构建每step trace和merged effect Vec，full input result又重复分类effect payload。分别补强 **PERF-MVP-254/293/294**：single route/effect owner，release默认只产compact outcome，完整trace按显式capture预算生成。
- V2 asset/repeat仍以wide TOML maps和多份String为authoring DTO；`metadata_value()`每次重建table并clone5组文本。继续归 **PERF-MVP-274/276/312**，compiled generation只发布dense handle/typed side table，runtime frame不得重投影metadata。
- **PERF-MVP-178 已部分止损但未完成**：`UiRenderCommand::to_paint_elements*`现在每command只调用一次`serde_json::to_writer(StableHashWriter)`并让1–4个element共享generation，删除了临时JSON Vec；但runtime和Editor每次conversion仍完整序列化wide command/style/text-layout/text。stable generation没有skip，且text paint继续复制source/shaped/font/atlas/glyph identities。
- foreign batch plan尚未成为runtime submit caller，但产品Runtime Diagnostics会调用`UiRenderDebugSnapshot::from_render_extract()`。该入口同时重建elements、sort batch plan、cache、visualizer与parity；batch key逐element clone clip/resource/effects，plan同时保存ordered indices、per-batch source indices与node ids。接线前必须收为single generation artifact，不能把debug DTO变成第二套render authority。
- debug复杂度精确补强 **PERF-MVP-280**：cache为判断batch reuse临时收集`Vec<&UiPaintElement>`；parity逐paint扫描全部batches并在source indices中`contains`，且重复构建batch key；visualizer重复batch lookup到每paint/clip/line/glyph，resource binding用Vec线性find/contains，overdraw对paint pair逐一求交后再全扫elements、排序node ids并线性查重regions，最坏可由O(P³)继续逼近O(P⁴)。现有2–4元素合同无法暴露该增长。
- shared slider tick 256+pixel-column双上限、Copy pointer/cursor contracts、固定8-stage export和fixed ABI layouts为正向基线。ABI safety test递归读全部interface源码属于test-only wall-clock成本，不进入产品热点；若测试波次成为瓶颈，应复用suite-scoped source inventory而非改变产品合同。
- 本批没有适合直接机械修改的简单Rust修复：关键文件均由外部owner修改中，且正确方案涉及generation ownership、debug opt-in与render/runtime边界，局部替换可能固化第二套缓存。

## 动态验收

1. current-source interface合同通过；UI nodes/commands/paint/batches 1/100/1k/10k、stable 300 frames与1% dirty记录command JSON visits/bytes、element builds、key/resource/effect clone、sort与p95，stable generation conversion/hash/build=0。
2. debug off/on、overlap none/all、glyphs/resources 1/100/10k：记录section builds、paint→batch probes、pair/intersection/region/resource visits、temporary/retained/returned bytes与p95。debug off工作=0；explicit capture接近O(P log P + intersections)、有count+bytes+time预算，不允许O(P³/⁴)。
3. focus nodes 1/100/10k、depth 1/16/64、连续10k Tab/方向键：stable candidate rebuild/sort/BTree visits=0，Tab近O(1)；mutation仅更新affected scope。
4. route depth 1/16/64、effects 1/10/1k、连续1M events：normal full trace alloc=0、route/effect payload owner=1、handler数不增加clone bytes，capture有entries+bytes+age上限。
5. F4 Runtime Diagnostics/Widget Reflector/input产品trace与当前UI像素通过；batch真正接入GPU submit后再做RenderDoc draw/batch/resource parity，不能用interface测试DTO代替产品capture。

全部391个current-source文件的静态阅读已经完成，但current-source Cargo、规模counter、F0/F2/F4产品trace、像素与RenderDoc仍未完成，因此整个crate继续保留在 `pending.md`，不进入 `review.md`；任一外部hash漂移必须重新打开对应文件。
