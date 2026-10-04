---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/effects.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/temporal.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/terminal.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - docs/plans/zircon_runtime/render/07-post-processing.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
tests:
  - GPU post-process dispatch root effects temporal and terminal four of four Rust files reviewed, 1350 lines
  - compiled resolver access and attachment semantics traced
  - no separate source edit; remaining work routes to PERF-MVP-362 and PERF-MVP-366
  - downstream execute resource constructors still pending file-by-file review
  - current-source Cargo scale trace and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics GPU post-process dispatch层逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读GPU context的`post_process.rs`、`effects.rs`、`temporal.rs`与`terminal.rs`当前4/4个Rust文件、1,350行。该层主要把compiled resource declarations解析为borrowed WGPU views/buffers并调用下游`ScenePostProcessResources::execute_*`，resolver-backed路径已走O(1)索引，没有发现新的独立根因。重复graph状态扫描回链PERF-MVP-362，view生命周期回链PERF-MVP-366；下游execute目录中的bind-group/buffer/pipeline行为仍须逐文件审查，不能用本切片覆盖。

## 已确认热点边界

`record_post_process_stack`为SceneComposite、Blur、Bloom、DepthOfField和MotionBlur分别调用`post_process_graph_has_node`，稳定pass最多重复线性扫描graph nodes五次。图规模目前只有十余effect，该局部成本不值得再建编号；PERF-MVP-362的compiled dense post artifact应直接提供effect bitset，dispatch层读取O(1)标志。

color-LUT bake先要求owned physical texture，再对它调用default `create_view`，虽然materialization已为同一逻辑resource建立默认view。这属于PERF-MVP-366的per-backing view bundle；直接替换为通用bound view会弱化“必须owned transient texture”的错误合同，所以本轮不做未经回归的局部改写。

effects/temporal/terminal其余函数只做固定数量resolver lookup、scalar feature/history判断和borrowed参数传递。`latest_scene_color*`按最多五个候选做O(1) lookup；当compiled post artifact提供最终input handle后，这些候选探测也应消失，但当前不是算法瓶颈。

## 验收

PERF-MVP-362按effects 0/1/11、passes 1/16、stable/changed记录graph-node visits，最终dispatch层effect discovery visits=0；PERF-MVP-366记录LUT/default/mip `create_view`与resource-name lookup，warm stable view create=0。继续逐文件审查`post_process/resources/execute_*`，统计每effect bind-group/buffer/upload/pass创建；在其Cargo、F2产品trace和RenderDoc证据完成前，本切片保留在`pending.md`。
