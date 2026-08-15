---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/parallel_encoder_set.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record.rs
  - zircon_runtime/src/core/framework/render/submission.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md
  - docs/plans/zircon_runtime/render/04-visibility-culling.md
  - docs/plans/zircon_runtime/render/05-lighting-shadows.md
  - docs/plans/zircon_runtime/render/07-postprocess-color-pipeline.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/RenderGraphBuilder.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/RenderGraphEvent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/MeshDrawCommands.cpp
tests:
  - current compiled-scene slice 28 of 28 Rust files reviewed, 6664 lines, 53 inline tests
  - scoped rustfmt 28 of 28 clean
  - current-source Windows Cargo, F2 counters, WPR, GPU timestamps and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_blocked
---

# Graphics compiled-scene current-source结构审查（2026-08-14）

## 当前范围与旧结论修正

`zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/**`当前物理清单28/28个Rust文件：6,664行、6,282个非空行、53条内联测试，fingerprint为`0FF14030F712C45600699957280A627E25A5C24FF631C97F88805B9A1FDF16FA`。相对2026-07-18旧报告新增TAA reactive-mask binding与render tests，旧26文件报告不再代表current source。本轮复读全部28文件以及直接使用的execution resources、execution record、parallel encoder和submission config；28/28通过`rustfmt 1.8.0 --edition 2021 --check --config skip_children=true`。13个tracked modified和2个untracked产品文件属于其他会话，本轮未改生产代码。

四组旧热点已被current source修正，后续任务不能重复按旧事实实施。light-grid、HZB与plugin缺producer时使用`SceneRendererNeutralGraphBuffers`持久中性buffer，binder不再逐帧create/format fallback；SSR复用持久view；首个HZB execution使用`.next()`而非固定槽Vec；空TAA reactive mask导入共享black texture。HZB diagnostics又由默认关闭、显式dispatch门与最多4帧异步队列约束，submit后只启动map，不在产品路径等待。这些正向结构应保留，剩余工作是current-build动态证明warm create/upload/wait均为0。

## P0：compiled pipeline并未生成可直接执行的artifact

`execute_graph_stage.rs:216-248`每次stage调用都动态格式化stage profile名、全扫`pipeline.pass_stages`，然后对每个entry再按String `pass_name`线性扫描`pipeline.graph().passes()`。MVP forward/deferred路径每帧调用约8至14个stage，即使空stage也支付全表扫描；`sprite_stage_selection.rs:18-35`和`render.rs:206`又分别按String重复查sprite active stage与half-resolution pass可用性。当前`CompiledRenderPipeline`虽然跨帧持久，执行仍退回`O(stages * passes + entries * passes)`的名字解释器，而不是dense PassId/range调度。

`execute_graph_stage.rs:640-793`还对每个pass无条件构造动态profile/debug名字，建立owned executor String，并把pass name、dependencies与resources分别clone进execution context和`RecordedGraphPass`；commit阶段再为profile clone pass/executor。`RenderGraphExecutionRecord`因而每帧拥有多张String/Vec元数据，即使diagnostics/timing没有消费。`record_post_process_graph`仍深clone整个graph保存报告。PERF-MVP-378/399必须同时消除名字查找和重复元数据物化，不能只把某一张BTreeMap换成HashMap。

Render01/17应在pipeline/executor generation冻结时发布dense stage offset/range、PassId直索引、resolved executor slot、resource declaration ranges、pass workload与可选静态diagnostic metadata。帧执行只借用compiled metadata；String和图快照只在错误、显式capture/dump或采样窗口内物化。硬切要求不能保留String fallback作为第二权威。

## P0：当前并行录制合同绕开MVP主工作

默认`RenderSubmissionConfig::synchronous`把`parallel_record`设为false，最小阈值仅为2 passes。即使显式开启，`execute_graph_stage.rs:261-292`只要存在mesh draw lists/pipelines或screen-space UI等可变owner便整体禁止并行，因此主要mesh、deferred、lighting与shadow stage仍串行。可并行路径还为每个stage分配graph-length `Vec<Option<_>>`；`parallel_encoder_set.rs:141-209`随后每stage用HashMap重建全compiled graph的topology layer、bucket Vec与encoder set。把录制开关打开只会暴露新的全图调度成本，并不会使MVP主阶段安全并行。

这不是适合局部调threshold的问题。Render01/02/17与Runtime11需要在compile generation预计算parallel-safe pass bucket和workload；mutable renderer owner拆成主线程准备的immutable pass packet与worker-local recorder，只有明确线程亲和性或共享可变状态的窄段留串行。任务数必须由pass/draw workload、可用worker和实测task overhead决定，保留低负载串行fallback、确定性topology顺序与单次queue submit。

## P1：帧资源和场景派生仍重复物化

`render.rs:403-467`每帧新建`RenderGraphExecutionResources`，直接owner含11张`BTreeMap<String, ...>`；import/bind路径重复clone资源名和wgpu handle，alias report再clone逻辑/物理String并排序，随后总是写入execution record。PERF-MVP-366应以compiled dense resource handle、grow-only workspace、bound bitset与按需report替代成功帧String树，持久中性资源修复不等于资源物化问题整体完成。

`render.rs:89-130`在存在irradiance volume时物化全部camera-layer mesh translation Vec并做volume与position选择；每camera/frame还遍历所有extracted meshes及resource revisions计算static shadow caster revision，再建立shadow plan。前者回链PERF-MVP-377；后者应补入PERF-MVP-391，以scene/visibility generation的dirty shadow-caster artifact消除stable frame全mesh扫描，不能只优化atlas slot内部录制。

plugin runtime prepare仍位于submission线程的串行准备段，回链PERF-MVP-379；original indirect args仍有独立owner，回链PERF-MVP-376。当前queue submit保持一次，异步readback在submit后begin-map而不等待，这两点应作为后续结构改造的回归门。

## Unreal Engine本地源码依据

- `RenderGraphBuilder.cpp:1341-1420`在compile阶段遍历dense `FRDGPassHandle`并以`Passes[PassHandle]`直接建立dependency与culling结果；执行期不按pass name线性解释compiled graph。
- `RenderGraphBuilder.cpp:2856-3005`的`SetupParallelExecute`直接消费compiled `FRDGPass*`，结合pass workload、task mode、最小pass数、barrier与merged-render-pass约束预建`ParallelPassSets`；它不为每个stage用HashMap重新拓扑整个graph。
- `RenderGraphEvent.h:479-516`让RDG event宏在minimal/off配置中成为no-op，证明稳定产品帧不应无条件构造调试label、profile String和完整execution report。
- `MeshDrawCommands.cpp:1016-1174,1707-1725`把dynamic mesh command generation、sort、instance-culling setup纳入并行任务，并用worker数与`MinDrawsPerCommandList`定粒度；Zircon不能以固定2-pass门槛且排除mesh owner来宣称MVP录制已并行化。

## 实施顺序与验收

1. Render01/17先把stage/pass/executor/resource声明编译为dense artifact，diagnostic record改为显式启用或采样；以counter证明stable name comparison、metadata clone与graph clone为0。
2. Render01/02与Runtime11再拆immutable prepared pass packet和worker-local recorder，compile generation预建parallel-safe bucket；先量serial prepare、record、merge与task overhead，再选择自适应阈值。
3. Render01把11张String资源map收口为可复用dense workspace；Render05把static caster revision和shadow plan绑定scene/visibility generation；Plugins01把runtime prepare声明affinity、deadline和bounded output。
4. Render17最后用同一current-source产品构建跑F2规模counter、WPR/xperf CPU/CSwitch/ReadyThread/energy、GPU timestamp与RenderDoc draw/dispatch/readback验证，不得使用2026-08-10旧editor二进制替代。

矩阵：passes 1/32/256/1k，stages 1/8/14/32，resources 16/256/1k，draws 0/32/1k/100k，views 1/4/12，threads 1/2/8/64，forward/deferred，plugin/UI/shadow/history/HZB off/on，stable/1/100% changed。记录stage/pass visits、name comparisons、topology/layer/bucket builds、String/metadata/graph clone bytes、resource map/alias work、main/worker wall与occupancy、task/encoder数、CSwitch/ReadyThread、CPU p50/p95/p99、GPU timestamp、RSS与energy。

硬门：stable generation stage visits近executed passes，String dispatch/name comparison/topology rebuild/diagnostic clone=0；parallel-safe MVP mesh阶段可按测得workload并行且低负载不回退；warm resource workspace/GPU neutral create=0；stable shadow caster全mesh scan=0；topology、pass order、single submit、pixels及readback语义等价。当前只有源码清单、rustfmt与测试合同证据；最近managed Windows构建受共享foreign编译错误阻塞，当前无可运行产品二进制，因此WPR、GPU timestamp、energy和RenderDoc均未产生有效样本。本记录留在`pending.md`，不进入`review.md`。
