---
related_code:
  - zircon_runtime/src/core/framework/render/camera_ordering.rs
  - zircon_runtime/src/core/framework/render/camera_stack.rs
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/surface.rs
  - zircon_runtime/src/core/framework/render/temporal_jitter.rs
  - zircon_runtime/src/core/framework/render/view_matrix_pair.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/04-visibility-culling.md
  - docs/plans/zircon_runtime/render/06-temporal-pipeline.md
  - docs/plans/zircon_runtime/render/09-camera-render-ordering.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/bevy/crates/bevy_render/src/camera.rs
tests:
  - camera and view root six of six Rust files reviewed
  - source-guard RED to GREEN for schema-v1 layer fast paths, lazy camera augmentation, borrowed sequence inputs and zero-jitter matrices
  - rustfmt and scoped git diff check passed
  - current-source Cargo, scale counters, F2 traces and RenderDoc pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime render相机与视图root逐文件性能静态审查（2026-07-18）

## 范围与结论

已完整阅读`camera_ordering.rs`、`camera_stack.rs`、`camera.rs`、`surface.rs`、`temporal_jitter.rs`与`view_matrix_pair.rs`当前6/6个Rust文件、1,337行，并聚焦追踪scene extract、camera-loop和矩阵产品调用点。surface是固定Copy DTO，Halton period计算无分配且默认周期很小，不构成独立瓶颈；热点集中在layer mask、相机序列所有权和跨pass相机矩阵。

## PERF-MVP-344：legacy layer检测创建临时heap mask

scene对每个entity/camera调用`intersects_scene_schema_v1_mask`时，原实现先循环32 bit构造`RenderLayerSet(Vec<u64>)`，再走通用zip相交；schema mask构造和lossy导出也做逐bit/逐layer扫描。本轮TDD已把32-bit构造、导出和相交改为first-block O(1)快路，相交不再分配，宽layer语义保持。

剩余常见layer≤64仍由heap Vec承载，scene为mesh/sprite/light/camera创建或clone mask时继续产生小分配。Render04应把`RenderLayerSet`收口为inline first block加可选overflow（或等价small storage），serde与>64 layer行为不变；visibility/extract只引用generation-owned mask，避免每renderable再clone。

## PERF-MVP-345：多相机序列重复深拷贝与平方查找

camera loop原在没有planar capture的普通帧也clone完整camera Vec，borrowed resolver随后再clone全部active descriptors，输出sequence又clone base/overlay；每个base stack reference还在线性active Vec中find，复杂度为O(S×C)。本轮已让planar camera Vec按确实追加时才clone，并让borrowed resolver以引用排序、只在最终owned sequence复制descriptor。

scene camera order report仍每次新建Vec/BTreeSet/BTreeMap并再次携带完整descriptor，camera列表与order report形成双owner；sequence查找仍为O(S×C)。Bevy将`SortedCameras`作为复用resource先`clear()`再填充，并把compact extracted camera与view留在render world。Render09/17应建立generation camera artifact和entity→descriptor index，排序storage复用，order/sequence/submission共享索引或handle，最终descriptor payload只保留一个owner。

## PERF-MVP-346：同一camera矩阵在多个pass重复计算

scene uniform、froxel、post-process、velocity、subsurface和Hybrid GI调用点会各自调用`ViewProjectionMatrixPair::from_camera`，重复projection、view和matrix multiplication。本轮先让zero-jitter路径直接复用unjittered matrix，删除TAA关闭时的一次identity matrix构造与乘法；但主计算仍按pass重复。

Render06/17应在camera/render-region generation变化时一次生成prepared camera matrices（jittered/unjittered/inverse/previous）并由所有pass借用；viewport、projection、transform或jitter变化才失效，不让feature executor自建矩阵。Bevy在extract阶段生成compact `ExtractedView`并由render-world consumers共享，Zircon应采用同类唯一prepared-view权威。

## 验收要求

PERF-MVP-344按entities 1/1k/100k、cameras 1/8、layers 0/1/32/64/1k记录mask builds/clones/alloc/blocks visited：legacy intersection alloc=0且O(1)，最终≤64 layer构造/clone heap alloc=0。PERF-MVP-345按cameras 1/8/100、bases/stacks/planar captures记录descriptor clone bytes、Vec/map/tree alloc、find probes和sort comparisons：no-capture camera-list clone=0、borrowed resolver预拷贝=0、最终lookup近O(C+S)、stable generation sort/build=0。PERF-MVP-346按passes 1/10/100、cameras 1/8、jitter off/on记录projection/view/inverse builds与matrix multiplications：zero-jitter额外multiply=0，最终matrix pair build≤1/camera/render-region generation。current-source Cargo、ordering/layer/viewport/TAA回归、F2 trace与RenderDoc通过前，本批继续留在`pending.md`。
