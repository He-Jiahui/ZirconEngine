---
related_code:
  - zircon_runtime/src/graphics/pipeline
  - zircon_runtime/src/graphics/runtime/render_framework/reload_pipeline/reload_pipeline.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/resolve_viewport_record_state.rs
tests:
  - graphics pipeline current source 46 of 46 Rust files statically reviewed
  - current slice 44 files and 6727 lines; all 41 current slice tests read
  - compiled graph cache 2 files and 10 tests covered by its earlier evidence
  - five behavior/source regressions added; scoped rustfmt, source contracts and diff check passed
  - current-source Cargo, F2 compile/reload scale counters, GPU timestamps and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics pipeline静态审查（2026-07-18）

## 当前源覆盖

`graphics/pipeline/**`当前46/46个Rust文件已完整静态阅读。此前`compiled_graph_cache.rs`及其测试2/2、10条测试已有独立证据；本轮覆盖其余44/44个文件、6,727行、41条目录内测试：`compile_options/**` 3/3、`validation/**` 4/4、`declarations/**` 17/17、`render_pipeline_asset/**` 20/20。另追踪默认管线句柄在viewport record与reload产品调用点的使用。

## 直接止损

viewport record原本每camera/frame只为取得默认handle，就分别调用`default_core2d()`或`default_forward_plus()`，构造完整String/Vec/renderer descriptor资产；reload也重复构造forward-plus资产。本轮集中三种内建handle常量，默认资产构造器与`builtin()`共同引用同一事实源，运行时按`CorePipelineKind`直接取得Copy handle。行为测试验证常量与三种资产一致，两个产品源码守卫禁止热路径恢复完整资产构造。

后处理descriptor过滤原来每个可过滤pass都重建整份active-resource `BTreeSet<String>`并clone资源名；现builtin每descriptor只构造一次，plugin通过`OnceCell`仅在首次真正进入post filter时构造并供后续pass复用。图资源authoring原来先借用完整resource-plan map创建句柄，再深clone整表String/plan作为执行索引；现直接接收所有权并移动到`AuthoredGraphResources`。两条源码回归把这两项从RED锁到GREEN，既有routing、external binding和stage语义测试不变。

## 剩余根因

PERF-MVP-422负责compiler重复物化与所有权。`compile_with_options`为validation构造全部enabled feature descriptors，正式compile又构造一次；`validate_feature_descriptors`调用`pipeline_graph_resources`后丢弃结果，`author_render_graph`再次分析同一资源集合。`RendererFeatureAsset::feature_name()`和`descriptor()`返回owned String/完整descriptor，feature filtering、inactive-name集合、plugin apply/reload及report查询继续重复clone/scan；effect enabled和routing也按pass多次线性扫描stack。Render01/07/08与Plugins01须按pipeline revision、options、executor/plugin generation发布唯一immutable compiled descriptor/resource analysis，Runtime07只消费single-flight ticket，Editor07按generation读summary/detail。

PERF-MVP-423负责pass ordering算法。每个stage先扫描并clone全部matching descriptors；唯一producer排序对每个write重新扫描全部pass/resource统计writers，再扫描全部readers，以`BTreeSet`建边，最后clone整份original并再次clone到排序结果。pass replacement与resource extension也按每条mutation重扫全pass。最坏成本随passes、resources和mutations呈平方到近立方放大。Render01须一次建立resource→writers/readers dense index与stable adjacency/range，以索引或move完成稳定拓扑顺序，并把stage编译成dense pass ranges。

本地Bevy `pipeline_specializer.rs`以specialization key缓存`CachedPipelineId`，`pipeline_cache.rs`把待创建pipeline放入waiting queue；UE `RenderGraphBuilder.cpp::Compile`按dense `FRDGPassHandle`线性设置依赖、裁剪和compile pass，并为状态数组预留容量。Zircon采用“稳定key只物化一次、编译结果用dense identity、分析结果供后续阶段复用”的原则，不复制它们的ECS、RHI或descriptor API。

## 验收状态

46/46静态阅读、五条新增回归、rustfmt、源码合同和目标diff门禁完成。Windows Cargo validator仍在启动前`ConvertFrom-Json`失败，51条pipeline目录测试及两个runtime源码守卫没有current-source执行结果；RenderDoc CLI不可用且本切片无capture。features/passes/resources/mutations 1/16/1k/100k、stable/1% change、reload并发1/8/64下的descriptor builds/clone bytes、resource analyses、stack scans、writer/reader visits、edge alloc、sort/move、compile queue age、state-lock hold、CPU p95/RSS与GPU pipeline/timestamp仍待测，因此继续留在`pending.md`，不进入`review.md`。
