---
related_code:
  - zircon_runtime/src/rhi/ui_surface.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/batching.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/batching/dependency_depths.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/tests.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Rendering/ElementBatcher.cpp
  - dev/slint/internal/core/partial_renderer.rs
tests:
  - zircon_runtime/src/rhi_wgpu/ui_surface/batching.rs::batch_plan_prunes_axis_disjoint_overlap_candidates
  - zircon_runtime/src/rhi_wgpu/ui_surface/tests.rs::full_redraw_without_damage_borrows_the_original_draw_list
  - zircon_runtime/src/rhi_wgpu/ui_surface/tests.rs::cache_bootstrap_full_redraw_owns_an_unclipped_damage_list
  - current-source Windows zircon_runtime ui_surface tests pending
  - current-source GPU/Softbuffer/RenderDoc comparison pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# RHI/WGPU UI surface batching聚焦性能审查（2026-07-18）

## 范围与覆盖

`zircon_runtime/src/rhi/ui_surface.rs`及`rhi_wgpu/ui_surface/{root,batching.rs,batching/dependency_depths.rs,tests.rs}`当前源 **5/5** 个Rust文件、**2,331** 行已逐文件阅读；调用链覆盖neutral draw-list stats、headless/native present、batch plan、full/damage模式和测试。整个RHI/RHI-WGPU当前源因新增child变为56文件，本证据不冒充其余51文件已读完。

## 热点与直接修复

原`dependency_depths`为每个later item扫描全部earlier items，即使1000个纵向完全不相交rows也执行499,500次rect check；10k items固定49,995,000次，发生在WGPU present主线程、GPU command recording之前。PERF-MVP-225已抽出interval index：按总width/height选择覆盖更小的x/y轴，以平衡interval tree查询axis-overlap candidates，再按完整rect确认依赖；items仍按原painter order计算longest depth，不保留O(K) edge表。稀疏rows/columns趋近O(N log N + candidates)，全部重叠时仍支付真实N(N-1)/2 dependencies。

原FullRedraw无论输入是否已有`damage=None`都clone完整draw list，只为把clone的damage再设None，图像RGBA与text String随之深复制。现以`Cow`借用正常full frame；只有retained cache未初始化且收到damage patch时，才owned clone一份去掉damage以完成正确的cache bootstrap。

本轮复核又直接删除image cache常见路径两类小分配：单帧upload去重集合借用draw-list内`resource_key`，不再为set/cache lookup反复clone String；cache entries≤256时`prune_image_cache`直接返回，不再每present先collect全部`(timestamp,key)` Vec。超过预算时原确定性LRU排序/逐出语义保持不变。

剩余重复成本未在本切片硬改：`batch_draw_plan/draw_items`、`UiSurfaceDrawList::stats`、`prepare_image_resources`和text prepare分别扫描commands/可见性；GPU presenter构stream时固定`include_image_bytes=true`，因此damage frame内的静态image/atlas仍可反复`queue.write_texture`。interval index也每present重建。Render17应按command/image/damage generation复用compiled batch/spatial/upload plan，并让一次visible projection同时产出batch、stats与upload rows。

## 参考引擎对照

Unreal Slate把element先归入显式layer/batch key，稳定排序batch indices后只在同layer内合并compatible batches，并为batch creation单独打cycle stat；它不为所有UI rectangles无条件建立全pair依赖。Slint partial renderer缓存item bounding geometry/transform并先计算dirty regions，使未变/离开dirty scope的item不进入后续绘制。Zircon保留更激进的non-overlap reorder收益，但必须以空间索引与generation cache限制CPU，而不能让draw-call优化先消耗O(N²)主线程预算。

## 动态验收

待受管Cargo运行全部`ui_surface`测试。构造1/100/1k/10k disjoint rows、columns、mixed clips与all-overlap：记录axis candidates、exact dependencies、plan p50/p95、alloc和clone bytes；1k disjoint candidates≤1k，all-overlap dependency仍为N(N-1)/2。产品profile记录stable generation plan/index build=0、每present visibility scan≤1、full redraw clone bytes=0、cache≤256 prune visits=0、static image key alloc/upload=0。最后以current-source GPU/Softbuffer截图与RenderDoc pass/draw/resource对拍确认order、clip、text/image、upload与像素等价；完成前保持`pending.md`，不进入`review.md`。
