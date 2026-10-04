---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/pass_graph/execute.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/scene_post_process_resources
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/07-postprocess-color-pipeline.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/bevy/crates/bevy_post_process/src/effect_stack/mod.rs
tests:
  - post-process non-resources root thirty-five of thirty-five Rust files reviewed, 1133 current lines
  - post-process aggregate one hundred seventy-five of one hundred seventy-five current Rust files statically reviewed, 10061 current lines
  - executor-id clone/tree-set source guard RED then fixed effect-mask GREEN
  - nineteen executor labels map to eighteen unique effect bits in regression test code
  - rustfmt and scoped git diff check passed
  - current-source focused Cargo, allocation counters, F2 pixel parity, GPU timestamps and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics post-process root逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`post_process`中`resources/**`之外当前35/35个Rust文件、1,133行，覆盖clear helper、cluster dimensions/constants、fallback texture、GPU ABI、参数结构、pass graph、scene-owned resources与feature flags。结合此前`resources/**` 140/140，整个`post_process/**`当前175/175个Rust文件、10,061行已完成静态审查。

尺寸/dispatch helpers、GPU POD参数和feature flags均为常数时间值类型；fallback textures、layouts、pipelines、parameter buffers及samplers由`ScenePostProcessResources`长期持有。后者仍包含PERF-MVP-371的可选pipeline同步构造，但没有新增逐帧owner错误。`clear_render_target`本身只录制单次clear，真正的问题是disabled effect仍调用它，已由PERF-MVP-370负责graph cull。

## 已直接止损：pass-graph执行记录改为固定effect mask

`record_post_process_graph`在每camera/frame完成真实pass后调用`execute_post_process_pass_graph`。旧正常路径先克隆`RenderGraphExecutionRecord`内全部executor ID，再构造`BTreeSet<String>`，随后对每个post node做字符串树查询；这是纯记录工作，却随pass数重复分配、复制与比较。

本轮先以源码测试锁定RED，再把19个已注册executor label映射为18个`PostProcessEffectKind`位，单遍折叠进栈上`u32` mask，node记录只做一次按位判断。当前正常路径复杂度为O(executed passes + post nodes)，executor ID clone与tree allocation均为0；bloom与bloom-extract共享同一位，现有SSR/bloom/ordered-node行为测试继续作为语义门禁，并新增19→18映射/唯一位回归代码。

旧fallback路径只在没有真实executor记录时按资源依赖模拟执行，仍构造两份`BTreeSet<String>`；同时`RenderGraphStageExecution::record_post_process_graph`仍为report所有权深clone整张graph，记录的node name仍逐项复制。它们归PERF-MVP-372：Render07应复用PERF-MVP-362的immutable compiled post artifact，以dense node/effect identity与executed bitset发布结果；Render17只在UI/capture/log边界物化String。Bevy当前effect stack直接从prepared view query读取optional effect状态、无效果时立即返回，并复用dynamic uniform buffers，说明frame执行层不需要重建字符串图来判断效果存在性。

## 验收

按cameras 1/8、graph nodes 1/18/100、executed passes 1/32/256、normal/fallback、diagnostics off/on记录graph clone bytes、executor label clone、tree/Vec allocation、string comparisons、node-name materialization和CPU p50/p95。当前正常路径要求executor clone/tree alloc=0且mask build=1/camera frame；最终stable compiled generation要求graph clone、fallback set和node-name String物化=0，changed generation≤1次编译，只有显式诊断导出才生成label。

受管focused Cargo、规模allocation counter、F2产品像素、GPU timestamp与RenderDoc完成前，整个目录保持在`pending.md`，不得进入`review.md`。
