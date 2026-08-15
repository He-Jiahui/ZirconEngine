---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution
  - zircon_runtime/src/graphics/feature/compute_pass_descriptor
  - zircon_runtime/src/render_graph/builder/compile.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - docs/plans/zircon_runtime/render/08-material-shader-permutation.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_plugins/01-plugin-architecture-core.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/RenderGraphBuilder.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/RenderGraphUtils.h
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/ShaderParameterStruct.h
  - dev/UnrealEngine/Engine/Source/Runtime/RHI/Public/PipelineStateCache.h
tests:
  - current graph_execution slice 59 of 59 Rust files reviewed, 18185 lines, 164 inline tests
  - scoped rustfmt 59 of 59 clean
  - current-source Windows Cargo, F2 counters, WPR, GPU timestamps and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_blocked
---

# Graphics graph-execution current-source结构审查（2026-08-14）

## 当前范围与旧报告差异

`zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/**`当前物理清单59/59个Rust文件：18,185行、16,950个非空行、164条内联测试，fingerprint为`C99FB573132214293FDE369F1133A086353DBB7BA1593492F6DA898F03231F72`。2026-07-18旧报告只有47文件、14,478行；本轮以旧逐文件基线加current diff复读全部旧文件，并完整复读新增compute cache、generic compute executor、resource identity及其tests。28个tracked文件和5个untracked状态项属于其他会话，本轮未改生产代码；59/59通过`rustfmt 1.8.0 --edition 2021 --check --config skip_children=true`。

current source有两项应保留的进展。瞬态纹理池现在把`SampledTextureIdentity`与物理texture一起跨帧复用，logical alias继承同一identity；TAA resolve用这些identity缓存bind group，并把实际create数写入record。另一个进展是generic compute cache会缓存成功pipeline与失败WGSL，容量固定16并按LRU驱逐，避免同一无效shader每帧重新做Naga解析。这些修复只解决identity正确性和pipeline miss，不代表stable compute pass已无CPU/GPU对象成本。

## P0：execution record仍是always-on深复制旁路

`RenderPassExecutionContext`当前拥有`String pass_name`、executor ID、dependencies Vec与resources Vec；compiled-scene为每个pass从compiled metadata clone构造context，执行后又把名称、依赖和资源移入`RenderGraphExecutionRecord`。record同时持有8张per-pass Vec、compute dispatch/audit String、post graph、alias report与profile records。`profile_report()`还clone整张profile Vec，`stage_execution_report()`再次扫全部stage；这些结构即使没有capture/diagnostic consumer仍构建。

`audit_compute_workload`对每个已记录dispatch按pass/executor String匹配并建立新的audit String记录；generic compute又无条件构造`storage_write_resources: Vec<String>`供这条旁路。结合调用端的动态stage/pass marker，这使稳定帧的业务录制与诊断录制无法分离。PERF-MVP-399必须改为disabled fast path、轻量numeric counters与按需sealed capture，不能仅优化一张registry map。

executor registry仍是`BTreeMap<RenderPassExecutorId, Arc<_>>`，每pass稳定执行做String树查找；validation cache只跳过全表合法性扫描，并未给compiled pass发布dense executor slot。registry clone会clone完整map且重置validation atomics。该问题继续由PERF-MVP-399和Plugins01负责。

## P0/P1：generic compute在帧循环重新解释shader与schema

新增`compute.generic`当前每次execute都clone pass name，按String解析所有binding并按binding number排序，分配binding-layout Vec与bind-group-entry Vec，clone wgpu buffer/view，创建bind group和compute pass，再clone所有storage-write资源名用于record。所有pass还争用一个`Mutex<ComputePipelineCache>`；即使pipeline命中，也在锁内对完整expanded WGSL、entry point和binding schema重新哈希并做完整key比较。

inline WGSL每帧扫描include roots；asset shader每帧重新resolve asset ID、取得source并展开module includes。cache miss才做Naga parse/validate和pipeline/layout/module创建，但cache key保留完整shader String，固定16容量在插入时先全表计数、满时全表找LRU。`render_graph/builder/compile.rs:149-264`已经验证binding与资源声明，却没有发布resolved resource handle、排序后的layout、shader generation或pipeline ticket，导致执行器重复编译期工作。这个问题单列PERF-MVP-623。

Render01/08与Plugins01应在feature/pipeline/shader generation生成`CompiledComputePass`：resolved shader artifact ID、entry point、workgroup、dense resource handles、binding layout/order、dispatch resolver与pipeline ticket。include展开、Naga reflection/validation、full-source hash和pipeline creation应在generation prepare阶段完成；Runtime11只承接明确可并行且bounded的异步warmup，render submission消费ready/last-good/error artifact，不等待无界编译。稳定资源identity再驱动bind-group cache，不能每pass无条件create。

## P1：资源物化仍以String树和逻辑view重建为核心

`RenderGraphExecutionResources`当前有11张`BTreeMap<String, ...>`，其中包含新增的sampled/owned texture identity索引。每帧import/bind会clone名称和wgpu handle；每个logical transient alias调用`texture.create_view`，mip/full-mip请求也现场create。`resource_alias_report`clone/sort逻辑与物理名，SSR alias还format新String。identity正确性修复没有消除PERF-MVP-366的dense workspace目标。

`transient_materialization.rs`每帧用`BTreeMap<SlotKey, Vec<&Lifetime>>`重新group allocation plan；pool `end_frame`每帧先retain扫描全部texture/buffer entries，再扫描求retained bytes，超预算时另建candidate Vec并排序。稳定pool虽不创建WGPU backing，仍支付全池维护和logical view重建。Render01应在compiled generation持久保存slot groups/view bundle，并以增量age bucket与running retained bytes替代stable full scan；PERF-MVP-366的counter必须包含pool visits、view creates和identity-map probes。

## 并行录制边界

`ParallelEncoderSet`当前按每次调用扫描compiled passes，用HashMap计算dependency layer，再建layer/bucket Vec；`record_parallel`为每bucket创建encoder。结合compiled-scene按stage重复调用、默认关闭和mutable mesh owner排除，当前实现不构成MVP并行收益。具体结构和矩阵由PERF-MVP-622承接，本报告补充其direct owner证据，不重复编号。

## Unreal Engine本地源码依据

- `RenderGraphBuilder.cpp:1341-1420,2856-3005`在RDG compile/parallel setup使用dense pass handle、compiled pass pointer、workload与约束预建执行集合，不在每stage重算HashMap拓扑。
- `RenderGraphUtils.h:451-563`的compute path捕获已解析`TShaderRef`、静态`FShaderParametersMetadata`和parameter struct进RDG pass；execute只设置pipeline/parameters并dispatch，不解析shader源码或按String排序schema。
- `ShaderParameterStruct.h:29-59,188-201`把parameter metadata和shader bindings绑定到shader type/instance，说明资源schema应是shader-generation artifact，而非frame artifact。
- `PipelineStateCache.h:164-185`提供RHI级compute PSO查找/创建入口与hitch统计；Zircon的per-executor 16项源码String cache既不能共享全renderer/device generation，也没有后台precache和hitch合同。

## 实施顺序与验收

1. Render01/17先把execution record改成off/sample/capture三态，numeric hot counters与error路径保留，String/Vec/graph/alias/profile report仅capture物化；同步完成dense executor slot。
2. Render01/08把compute metadata编译为`CompiledComputePass`，Plugins01把shader/registration reload映射为generation invalidation；Runtime11补bounded warmup与last-good policy。
3. Render01把resource workspace、slot groups、default/mip view bundle和pool accounting持久化；TAA identity cache作为其他pass的正确性模板。
4. Render17用同一current-source产品构建跑F2、WPR/xperf、GPU timestamp与RenderDoc；旧editor二进制不能作为数据源。

矩阵：passes 1/32/256/1k，compute variants 1/16/17/128，WGSL 1KiB/64KiB/1MiB，bindings 0/1/16/128，resources 16/256/1k，stable/1/100% shader或resource change，owners 1/8，threads 1/2/8/64，diagnostics off/sample/capture。记录WGSL/include/hash/parse/validate bytes与calls、schema sort/clone、pipeline/layout/module/bind-group/view creates、cache hit/evict/retained bytes/mutex wait、record String/Vec/graph clone、pool visits、CPU p50/p95/p99、CSwitch/ReadyThread、GPU timestamp、RSS与energy。

硬门：stable generation WGSL include/hash/parse/schema sort/pipeline create=0，ready compute pass不等待cache mutex，stable binding identities的bind-group create=0；diagnostics off的record String/metadata/graph/alias clone=0；warm resource grouping/view/pool full scan=0；executor dispatch O(1) dense；plugin reload只重编affected generation且last-good/error有界；dispatch、资源访问、pixels、single submit与错误语义等价。当前无current-source可运行产品二进制，最近managed build仍受共享foreign编译错误阻塞，因此WPR、GPU timestamp、energy和RenderDoc没有有效样本。本记录留在`pending.md`，不进入`review.md`。
