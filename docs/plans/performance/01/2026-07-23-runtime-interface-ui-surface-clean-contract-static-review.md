---
related_code:
  - zircon_runtime_interface/src/ui/surface
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
  - docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/zircon_runtime/render/17/failure-2026-07-18-runtime-ui-render-command-fanout-and-current-capture.md
reference_sources:
  - dev/slint/internal/core/partial_renderer.rs
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Rendering/ElementBatcher.cpp
tests:
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime_interface/src/tests/ui_contract_spine.rs
  - zircon_runtime_interface/src/ui/surface/render/text_geometry/source_map_tests.rs
  - current-source Windows surface/render/text tests pending
doc_type: implementation-evidence
status: partial_static_complete_dynamic_pending
---

# Runtime interface UI surface clean subset 性能静态审查（2026-07-23）

## 范围与覆盖

`zircon_runtime_interface/src/ui/surface/**`当前 **30/47** 个受跟踪且clean的 Rust文件、**3,177** 行已逐文件阅读；9个dirty tracked与8个foreign untracked文件不计入本批、不吸收。foreign untracked `render/batch/**` 7文件仅作只读预研，用于识别未来接线风险，不作为验收内容。已反查runtime surface/frame/dispatch/text/render消费者、interface tests及`dev/`参考。本轮未修改Rust源码。

## 性能结论

- `UiArrangedTree::get()`线性find，`children_of()`又对每child重复线性get，直接补强 **PERF-MVP-277**；generation应发布node id→dense index，draw order/children只存handle range。
- `UiSurfaceFrame`同时拥有arranged tree、render extract、hit grid、focus、layout/pipeline/ECS；debug snapshot默认开启command/hit/overdraw/overlay全section，timeline snapshot又深持全部frames且只有frame-count容量。分别回链 **PERF-MVP-278/280/281**：stable frame用Arc generation artifact，debug按section显式capture并受entries+bytes+age预算。
- pointer/navigation route携多组node Vec，focus path从bubble route再clone+reverse一份；`UiFocusState.changes/focused_inputs`为无界历史。回链 **PERF-MVP-254/282/293**：route/path共享single generation，current state与bounded observer/debug ring分离。
- brush/image/vector同时拥有resource key和resource state，fallback写入时复制两份resource key；resolved style/brush仍有多组String。归 **PERF-MVP-288/289**：compiled brush/resource/style handle只存一次identity/state，command/batch只持compact handle。foreign untracked batch plan未找到产品caller，只作为Render17只读接线前风险，不计入本批结论。
- `UiTextLineSourceMap::new()`每次重建整行grapheme cluster Vec；selection、composition与caret各自创建map，advance查询再从头扫描/sum。resolved→shaped→paint同时复制source/line/run/cluster String，逐cluster frame计算反复统计grapheme前缀并扫描runs，长行可接近O(G×R)/O(G²)。精确补强 **PERF-MVP-292/296**：text generation持source-map、visual boundary与prefix-advance index，paint只引用shaped run/glyph ranges和shared source。
- text-effect extent 64 px clamp、IME UTF-8 range、Copy typography/range/geometry为正向基线；language/effect normalization仍分配String，应只在typography generation变化时执行，回链 **PERF-MVP-289/292**。
- Slint partial renderer以cache generation+index保存geometry并对dirty region设矩形上限；UE ElementBatcher以batch key合并并复用source vertex/index arrays。Zircon采用generation cache、bounded damage与compact batch handle原则，不复制具体backend结构。

## 动态验收

1. nodes/commands 1/100/10k、stable 300 frames与1% dirty：记录arranged lookups、frame/debug clone bytes、section builds、timeline retained/returned bytes、render cache reuse与p95；stable full artifact clone=0，debug bytes硬有界。
2. route depth 1/16/64、连续1M input/focus events：记录route/path clone bytes、focus history bytes、observer drops/age与p95；current focus state有界，single route owner。
3. text chars/graphemes/runs 1/1k/100k、selection/composition/caret各10k次：记录source-map builds、grapheme/run/prefix visits、source/run String clone bytes、layout/paint p95；每line map≤1/text generation，point/range query近O(logG+spans)，stable projection clone=0。
4. batch elements/resources/effects 1/100/10k：接线后记录sort/key/resource clone、index/node-id bytes、batches/draws；stable compiled handle不重建wide key，Cargo/像素/RenderDoc与当前split语义一致。
5. current interface合同、runtime surface/input/text/render tests及F4 large UI/IME产品trace通过。

current-source Cargo、规模counter、F4/像素/RenderDoc及17个dirty/untracked文件独立审查未完成，因此该目录继续保留在 `pending.md`，不进入 `review.md`。
