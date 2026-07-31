---
related_code:
  - zircon_runtime/src/core/framework/render/post_process/volume_component.rs
  - zircon_runtime/src/core/framework/render/post_process/volume_component
  - zircon_runtime/src/core/framework/render/post_process/volume_evaluator.rs
  - zircon_runtime/src/core/framework/render/post_process/volume_extract.rs
  - zircon_runtime/src/core/framework/render/post_process/volume_profile.rs
  - zircon_runtime/src/core/framework/render/post_process/volume_registry.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/scene/world/render_post_process.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/07-postprocess-color-pipeline.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
tests:
  - post-process volume seven of seven Rust files reviewed
  - frame extract scene extraction camera loop submission context and froxel consumers traced
  - old per-call evaluator construction and entity-only extract ordering source guards changed from one to zero
  - rustfmt and scoped git diff check passed
  - focused Cargo scale counters F2 trace and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime post-process Volume逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`core/framework/render/post_process`的Volume切片当前7/7个Rust文件、2,329行：`volume_component.rs`、`volume_component/{params,tests}.rs`、`volume_evaluator.rs`、`volume_extract.rs`、`volume_profile.rs`与`volume_registry.rs`。另完整读取scene extract、camera loop、submission context及froxel resolved-settings生产调用链。确认MVP存在“每帧构建一次Volume DTO，却被同一相机的主提交和多个froxel消费者重复求值”的P0根因；本轮先消除每次求值重建注册表和已排序输入重复排序，跨消费者共享resolved artifact仍须架构收口。

## PERF-MVP-363：同一相机的Volume结果被多个渲染消费者重复求值

`PostProcessExtract::resolved_settings_for_camera`原先每次先`VolumeEvaluator::default()`，重新构造`Vec<VolumeComponentDescriptor>`并验证15个内建descriptor，再扫描全部Volume、计算shape influence、收集适用项、排序并逐override求值。产品调用不只在`build_frame_submission_context`：froxel binding、media inject、light scatter、integrate及history-quality路径均可再次进入同一函数，因此稳定帧仍重复注册、扫描、临时Vec、排序、String registry lookup和参数Vec分配。

本轮直接止损两点：内建`VolumeEvaluator`由`OnceLock`进程级复用；scene extract在已有entity排序位置改为`priority + entity`一次排序，evaluator只在直接构造或乱序输入时执行fallback sort。这样不改变公开调用语义和手工测试输入的兼容性，并使产品常见输入不再按消费者重复排序。

剩余根因必须由Render07/17在每个camera submission只计算一次`ResolvedPostProcessStack`，以generation-owned handle同时供post-process graph、froxel、history、stats和capture借用；不得在各executor内从`RenderFrameExtract`重新求值。Runtime07负责把scene/profile/transform/layer变化汇成明确Volume generation，稳定generation的resolved build为0或每相机至多1次。

## PERF-MVP-364：Volume extract与override采用每帧展开、字符串路由和多层深clone

`World::collect_post_process_volumes_for_view`每帧遍历全部Volume，逐实体调用hierarchy/render-layer/world-transform查询，并把profile展开为多组`String + Vec<Option<VolumeParamValue>>`；每个相机求值又为每个override线性查15项registry并由descriptor `read_values`新建参数Vec。多相机提交的`CameraLoopPostProcessSourceState`还深clone全部volumes、stack和graph，后续submission通过`clone_from`恢复。局部Volume数量、相机数和froxel消费者数同时增长时，成本近似乘法放大。

Render07联动Runtime07应发布按scene/profile generation维护的immutable compiled Volume set：注册/变更时按priority排序，builtin component使用dense/static identity，override参数采用紧凑连续存储；局部Volume规模超过阈值时增加空间候选索引，per-camera只做候选influence与一次blend。camera loop、post、froxel和stats共享同一`Arc` artifact，不复制Volume/stack/graph。插件动态component仍保留注册期字符串边界，但热路径先解析成descriptor index。

## 参考实现结论

Unreal的`UWorld::InsertPostProcessVolume`在注册/变更边界维护priority有序数组，per-view `AddPostProcessingSettings`按稳定指针直接遍历并blend，不在每个view重新排序或以字符串查descriptor；这直接支持“更新期排序、帧内借用”的目标。Bevy后处理把每个view的typed settings通过extract component进入render world，并在Prepare阶段批量写dynamic uniform buffers，说明Zircon可以保留可扩展authoring schema，同时在render hot path降为typed/dense artifact。

## 验收要求

按volumes 0/1/100/10k、cameras 1/8/100、froxel off/on、global/local混合、stable/1% changed记录registry builds、volume/shape visits、applicable alloc、sort calls/comparisons、component lookups、parameter Vec alloc、resolved builds、Volume/stack/graph clone bytes与CPU p95：registry build进程≤1；stable product extract sort≤1/scene generation；resolved build≤1/camera submission；同camera所有消费者artifact identity相同；stable generation override/String/clone=0。现有priority、mask、box/sphere、exposure/unset-param行为测试，current-source Cargo、F2产品trace、GPU pass/counter及RenderDoc通过前，本切片留在`pending.md`。
