---
related_code:
  - zircon_runtime/src/scene/components/mod.rs
  - zircon_runtime/src/scene/components/scene/transform.rs
  - zircon_runtime/src/scene/components/scene/node.rs
  - zircon_runtime/src/scene/components/scene/camera.rs
  - zircon_runtime/src/scene/components/scene/lighting.rs
  - zircon_runtime/src/scene/components/scene/mesh_renderer.rs
  - zircon_runtime/src/scene/components/render2d/sprite.rs
implementation_files:
  - zircon_runtime/src/scene/components/scene
  - zircon_runtime/src/scene/components/render2d
plan_sources:
  - user: 2026-09-09 场景组件说明
tests:
  - zircon_runtime/src/asset/tests/assets/scene
doc_type: module-detail
---

# 场景组件

## 基础与变换

`Name(String)` 是编辑器和路径解析使用的名称；`Hierarchy { parent }` 保存父实体。`LocalTransform` 包含可写的 `Transform`（translation/rotation/scale），`WorldTransform` 与 `WorldMatrix` 是派生的世界空间结果。`ActiveSelf` 是可写局部激活值，`ActiveInHierarchy` 根据祖先计算且只读。`RenderLayerMask(u32)` 控制可见层，默认值来自 `default_render_layer_mask()`。`Mobility` 区分静态和动态对象，影响变换与重挂载合法性。

## 渲染组件

`CameraComponent` 支持 `Core2d/Core3d`、透视/正交投影、裁剪面、HDR、曝光、清屏色、视口、排序和 MSAA。`MeshRenderer` 以 `ResourceHandle<ModelMarker>`、材质句柄和可选 mesh 为主，可配置 primitive 绑定、LOD、morph 权重、队列、深度偏移、材质覆盖、tint 与 alpha mode。`Sprite2dComponent` 引用纹理和可选材质，支持图集区域、翻转、锚点、尺寸、颜色和 z-order。

## 光照、后处理、物理与动画

光照组件为 `AmbientLight`、`DirectionalLight`、`PointLight`、`RectLight`、`SpotLight`，共同描述颜色、强度、范围/尺寸、方向及阴影/体积开关。后处理由 `PostProcessSettingsComponent` 与 `PostProcessVolumeComponent` 承载。物理使用 `RigidBodyComponent`、`ColliderComponent`、`JointComponent` 及 `RigidBodyType`/`ColliderShape`。动画组件族包括 skeleton、player、sequence、graph 和 state machine。

```rust
use zircon_runtime::scene::components::{CameraComponent, LocalTransform, MeshRenderer};
world.insert(camera, CameraComponent { hdr: true, ..Default::default() })?;
world.insert(mesh, MeshRenderer::from_handles(model_handle, material_handle))?;
world.update_transform(mesh, zircon_runtime::core::math::Transform::default())?;
```

## 反射与序列化

带 `ZrReflect` 的组件可通过 `WorldReflection`/`TypeRegistry` 暴露给脚本和编辑器。`LocalTransform.translation`、`scale` 可读写，rotation 只读；`Hierarchy`、`ActiveInHierarchy` 等派生值标记为不可序列化。场景文档应持久化 authoring 组件，不要把派生世界矩阵写回资产。

## 错误与限制/状态

资源句柄只表达 ID，不保证已加载；渲染器在资源未就绪时由资产管线报告 readiness。静态节点的变换和 parent 修改会返回 `SceneError`。组件定义和默认值已实现，物理/动画行为按 feature 和相应运行时驱动可用。

## 源码与测试

[components/mod.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/scene/components/mod.rs)、[transform.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/scene/components/scene/transform.rs)、[node.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/scene/components/scene/node.rs)、[mesh_renderer.rs](https://github.com/He-Jiahui/ZirconEngine/blob/main/zircon_runtime/src/scene/components/scene/mesh_renderer.rs)；场景资产测试见 [asset/tests/assets/scene](https://github.com/He-Jiahui/ZirconEngine/tree/main/zircon_runtime/src/asset/tests/assets/scene)。
