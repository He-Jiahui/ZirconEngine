---
related_code:
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache/tests.rs
  - zircon_runtime/src/graphics/pipeline/declarations/render_pipeline_compile_options.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/build_frame_submission_context/compile_pipeline.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/bevy/crates/bevy_render/src/render_resource/pipeline_cache.rs
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/RenderGraphBuilder.cpp
tests:
  - compiled graph cache two of two Rust files reviewed
  - cache key fingerprint hit miss resize feature revision and LRU tests reviewed
  - duplicate hit fingerprint source guard changed from one to zero
  - rustfmt and scoped git diff check passed
  - current-source Cargo scale trace and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics compiled graph cache逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`graphics/pipeline/compiled_graph_cache.rs`与`compiled_graph_cache/tests.rs`当前2/2个Rust文件、857行，并完整读取compile-options声明和frame-submission产品caller。缓存能以`Arc<CompiledRenderPipeline>`稳定命中并保持16-entry LRU，但命中路径仍构造、深clone并hash宽key；miss又在framework全局state锁内同步执行完整pipeline/graph compile。它是MVP稳定帧CPU和首次/变更帧主线程stall的P0问题，编号PERF-MVP-365。

## PERF-MVP-365：cache hit仍复制宽key，cache miss持全局锁同步编译

`compile_submission_pipeline_with_options`先clone `RenderPipelineCompileOptions`以写shader quality/OIT override，随后`CompiledGraphCacheKey::from_inputs`再次clone options作为HashMap key。options含多个`BTreeSet`、plugin feature Strings、完整owned `PostProcessStackDescriptor`及IBL request；每次lookup还hash这些集合、Strings和stack。稳定帧即使最终命中，仍支付key clone/hash。capacity满时LRU扫描16项可接受，主要成本不是逐出而是宽key。

caller在`framework.lock_state()`后调用`get_or_compile_with_status`，miss closure在锁内执行`pipeline_asset.compile_with_options`和capability validation；shader/graph首次构建或feature/size变化会独占整个framework state。正确方向是按pipeline/options/post/feature/capability generations构造紧凑Copy fingerprint，cache key只含dense IDs；miss以single-flight compile ticket移出锁和frame thread，完成后短锁发布`Arc`，同key并发请求共享任务与fallback policy。

本轮直接删除cache-hit后的`debug_assert_eq!(saved_fingerprint, extract_compile_fingerprint(...))`。extract在同一函数内以immutable borrow保持不变，该断言只是立即重复刚完成的同一fingerprint计算，无法检测跨帧stale state，却在debug/editor每个hit重复view/render size和advanced-lighting字段读取。fingerprint字段覆盖仍由现有resize/HDR/MSAA/target/particle/cookie/irradiance/transmission测试负责。

## 参考与验收

Bevy PipelineCache把queued pipeline转为waiting/ok/err状态，并可把pipeline creation交给AsyncComputeTaskPool；Unreal RDG为compile设置独立计时、容量预留和可选parallel setup/compile。Zircon先实现compact generation key、短锁single-flight和明确fallback，再用规模证据决定是否并行graph内部阶段。

按stable/1% changed、pipelines 1/16/100、cameras 1/8、effects/plugins 0/1/100、passes 16/64/256/1024记录options clones/bytes、key hash visits/bytes、fingerprint calls、lock wait/hold、compile calls/time、single-flight joins、fallback frames与cache hit/miss/evict：stable hit owned clone/hash bytes=0或固定Copy key，fingerprint≤1/submission，miss compile lock hold=0，同key compile≤1，steady main graph miss=0。Cargo、F2产品trace、graph/pass/像素及RenderDoc对拍完成前留在`pending.md`。
