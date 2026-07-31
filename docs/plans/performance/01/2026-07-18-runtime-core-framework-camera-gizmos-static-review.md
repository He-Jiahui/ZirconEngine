---
related_code:
  - zircon_runtime/src/core/framework/camera_controller
  - zircon_runtime/src/core/framework/gizmos
  - zircon_runtime/src/dynamic_api/camera_controller.rs
  - zircon_runtime/src/scene/world/query.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_navigation.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - dev/bevy/crates/bevy_gizmos/src/gizmos.rs
  - dev/bevy/crates/bevy_gizmos/src/circles.rs
  - dev/bevy/crates/bevy_gizmos_render/src/lib.rs
tests:
  - eighteen of eighteen camera-controller Rust files reviewed
  - six of six gizmo Rust files reviewed
  - source-guard RED to GREEN for idle math, matrix reuse and streamed circles
  - rustfmt and scoped git diff check passed
  - current-source Cargo, allocation counters and F2/F4/RenderDoc traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime core framework camera controller与gizmos逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`core/framework/camera_controller/**` 18/18（820行）与`core/framework/gizmos/**` 6/6（773行），并追到runtime tests、dynamic API camera controller、Editor05 viewport navigation和scene scalar/full-node read口。Orbit controller是基本编辑器及dynamic runtime拖动/滚轮可达路径；free/pan当前生产调用未命中。framework gizmos当前除tests外没有生产caller，不能以其局部micro优化冒充F4产品改善。

## PERF-MVP-333：相机scalar read与gizmo几何重复投影

相机控制数学本身无堆分配，但原`CameraControllerOutput::from_transform`即使before==after仍计算translation/scale delta及quaternion inverse；free camera速度已经为零时仍每update执行`exp`阻尼。更大的调用方问题在`dynamic_api/camera_controller.rs`：每次orbit/pan/zoom只需active camera id与local transform，却调用owned `Scene::find_node`，从而clone名称和全部可选camera/mesh/light/physics/animation components。

本轮已让unchanged output直接返回、零velocity跳过阻尼；新增`World::local_transform`窄读口并把三条dynamic camera action硬切到该读口，避免完整SceneNode projection。该公开契约已在`docs/zircon_runtime/scene/ecs.md`明确为scalar hot-path owner，并补local-vs-world transform行为测试与source guard。

gizmo extract原来为每个line endpoint/vector调用`Transform::matrix()`，rect/cube组合还把矩阵分解回Transform再重建；circle/sphere每个圆先分配points Vec再转成lines，最终lines Vec无形状容量预算。本轮改为每buffer仅构建一次Mat4、child shape直接矩阵相乘、预估精确line count并reserve、圆弧逐segment流式写line，删除圆临时Vec。现有shape count/retained transform行为保持；source guards先RED后GREEN，rustfmt与scoped diff check通过，Cargo仍待受管testing stage。

## 剩余架构热点与参考引擎结论

`GizmoOverlayExtractRequest`为少量buffer/retained refs各持一个Vec，extract仍每调用创建完整line output并把每条线重复存两端点/颜色；retained asset只有command级缓存，没有按asset/config/transform generation缓存编译后的line strip，也没有与Editor05现有interaction/gizmo extract统一。固定32段圆对屏幕尺寸无自适应预算，稳定相机/稳定retained gizmo仍会重建CPU lines和后续GPU upload。AABB/axis的retained non-identity transform语义还需单独正确性对账，不能借性能迁移悄悄改变。

Bevy gizmos将line-list与line-strip positions/colors保存在可clear/swap复用的storage中，圆由iterator直接写line strip，render侧按连续vertex buffers消费。这支持后续硬切generation-compiled `GizmoFrameStorage`：immediate buffer按帧clear保留容量，retained commands按asset/config generation编译，transform在GPU实例或一次CPU批处理应用；Editor05与runtime render只消费一份overlay事实源。framework无生产caller前先定owner并迁移，禁止保留editor/runtime两份永久实现。

## 验收要求

按camera actions 1/100/10k/1M、scene node optional payload 0/1/32、gizmo commands 1/100/10k、circle/sphere 1/100/10k、segments 3/32/256、retained instances 1/1k/100k、stable/changed generation记录SceneNode/local-transform clone bytes、matrix builds、sin/cos、line realloc、command expansion、upload bytes、draw/pass数与CPU/GPU p50/p95/p99：每camera action full-node projection=0，unchanged inverse/exp=0；每buffer parent matrix build≤1，circle temp alloc=0，stable retained generation CPU rebuild/upload=0。Orbit/pan/zoom/cursor、shape/color/order/transform parity、focused Cargo、Editor05 F4 trace及RenderDoc稳定帧全部通过前，两目录继续留在`pending.md`。
