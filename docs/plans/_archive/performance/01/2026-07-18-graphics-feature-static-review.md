---
related_code:
  - zircon_runtime/src/graphics/feature
tests:
  - graphics feature current source 38 of 38 Rust files and 2739 lines reviewed
  - all 14 tests read; one static dispatch identity regression added
  - clustered compute dispatch construction changed from RED to GREEN
  - scoped rustfmt, source contract and diff check passed
  - current-source Cargo, compile/reload allocation counters, F2 and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Graphics feature静态审查（2026-07-18）

## 当前源覆盖

`graphics/feature/**`当前38/38个Rust文件、2,739行已逐文件静态阅读，14条测试已读：root 2/2、`builtin_render_feature/**` 4/4、`builtin_render_feature_descriptor/**` 24/24、`render_feature/**` 2/2、`render_feature_descriptor/**` 3/3、`render_feature_pass_descriptor/**` 3/3。覆盖内建feature catalog/capability、plugin trait、pass/resource/mutation DTO以及全部MVP内建graph descriptor。

## 直接止损

HZB与motion-vector固定shader contract已经由全局shader模块用`OnceLock`缓存，但clustered-lighting每次构造descriptor仍重新解析固定locator、分配entry/resource描述并运行`ComputeDispatchBuilder::build`校验。本轮把clustered dispatch plan收敛为同样的进程级`OnceLock<ComputeDispatchPlan>`，pass descriptor只借用静态contract再投影自身资源。指针身份测试锁定重复调用返回同一计划，原light-grid workload/resource行为测试继续覆盖语义。

## 剩余根因

整个模块是PERF-MVP-422/423的上游证据。`BuiltinRenderFeature::descriptor()`每次返回全新owned graph；仅`post_process`就重新构造18个pass及大量resource Strings/Vec，pipeline validation与正式compile又会重复调用。pass name与executor ID分别拥有字符串，shader plan资源再clone进feature DTO；plugin `RenderFeature::descriptor()`合同同样只能返回owned payload。`RendererFeatureAsset`、pipeline filter/validation/authoring因此无法共享唯一descriptor generation。

advanced slot catalog最多22项、authoring feature最多40项，当前`find`与mutation `contains`属于小规模compile/editor控制面，未在没有规模数据时单独编号。它们最终应随PERF-MVP-422编译为interned feature/pass/resource IDs、immutable ranges与feature/effect masks；PERF-MVP-423复用这些ranges建立producer/reader adjacency，不再让每个stage和mutation重扫owned descriptor。

本地Bevy `pipeline_specializer.rs`只在specialization key miss时构造descriptor并保存`CachedPipelineId`，`pipeline_cache.rs`把GPU创建延迟到waiting queue；UE RDG按dense pass handle一次编译依赖。采用“固定contract只建一次、variant由稳定key生成、compiled阶段消费dense identity”的原则，不复制其API。

## 验收状态

38/38静态阅读、一条RED→GREEN回归、rustfmt、源码合同与diff门禁完成。Windows Cargo validator仍在启动前`ConvertFrom-Json`失败，14条测试没有current-source执行结果；RenderDoc CLI不可用。features/passes/resources 1/16/1k/100k、stable/1% change与reload 1/8/64下的descriptor builds、String/Vec alloc/clone bytes、shader contract builds、slot/mutation scans、compile queue/lock、CPU p95/RSS及GPU pipeline/timestamp未量化，继续留在`pending.md`，不进入`review.md`。
