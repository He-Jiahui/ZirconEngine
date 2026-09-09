---
related_code:
  - zircon_runtime/src/asset/assets/mesh/mesh_asset.rs
  - zircon_runtime/src/asset/assets/texture/texture_asset.rs
  - zircon_runtime/src/core/framework/render/material/standard_material.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mod.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/resources/runtime
  - zircon_runtime/src/graphics/scene/resources/resource_streamer
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh
  - zircon_runtime/src/graphics/scene/scene_renderer/material
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/assets-and-rendering/runtime-surface-and-assets-rules.md
  - docs/assets-and-rendering/runtime-physics-animation-assets.md
tests:
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache
  - zircon_runtime/src/graphics/scene/render_product_streamer_tests
  - zircon_runtime/src/asset/tests
doc_type: module-detail
---

# 网格、材质与纹理

## 资产到绘制的路径

```text
res:// source / imported model
  -> MeshAsset / TextureAsset / MaterialAsset
  -> RenderFrameExtract 中的 mesh/material/image 描述
  -> ResourceStreamer 按 ResourceId/revision 确保 GPU residency
  -> vertex/index/texture upload + bind group/pipeline cache
  -> phase queue 排序
  -> mesh draw / shadow / prepass / forward 或 deferred Pass
```

资产对象是可序列化 CPU 事实，GPU resource 是设备代际和 revision 限定的派生缓存。不要在资产层存 `wgpu::Buffer` 或 RHI handle。

## MeshAsset

`MeshAsset` 包含 URI、拓扑、命名 attributes、可选 indices、usage、morph targets、skin、Mesh SDF 和 Virtual Geometry payload。

内置 attribute 名称和格式：

| Attribute | 格式 | 用途 |
| --- | --- | --- |
| position | `Float32x3`，必需 | 顶点位置与 bounds |
| normal | `Float32x3` | 光照 |
| tangent | `Float32x4` | 法线贴图切线空间 |
| uv0 / uv1 | `Float32x2` | 材质纹理和 lightmap |
| color | `Float32x4` | 顶点色 |
| joint index | `Uint16x4` | 蒙皮关节 |
| joint weight | `Float32x4` | 蒙皮权重 |

`validate()` 检查 position、attribute 长度、内置格式、index 范围和 topology element 数。`try_render_mesh_descriptor()` 在验证后生成中立 descriptor；`render_mesh_descriptor()` 是容错投影，不应替代导入验证。

```rust
use std::collections::BTreeMap;
use zircon_runtime::asset::{AssetUri, MeshAsset, MeshAttributeValues};
use zircon_runtime::core::framework::render::RenderMeshTopology;

let mut attributes = BTreeMap::new();
attributes.insert("position".into(), MeshAttributeValues::Float32x3(vec![
    [-1.0, -1.0, 0.0], [1.0, -1.0, 0.0], [0.0, 1.0, 0.0],
]));
let mesh = MeshAsset::new(
    AssetUri::parse("res://meshes/triangle.zrmesh")?,
    RenderMeshTopology::TriangleList,
    attributes,
    None,
)?;
assert_eq!(mesh.render_primitive_count()?, 1);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Morph 与 skinning 有 GPU path；是否产生 velocity 还取决于 motion vector 设置和 previous transform/palette。Virtual Geometry payload 是 advanced provider 输入，不等价于普通 mesh 自动使用 Nanite-like 路径。

## StandardMaterialDescriptor

标准材质支持 base color、normal、metallic/roughness、occlusion、emissive 纹理及每槽 texture transform/UV channel，另有 alpha mode、lighting model、unlit、double-sided、shadow flags、render queue、depth bias、TAA reactive mask、separate translucency 和 advanced PBR features。

约束：

- roughness 的内部稳定下限由 `STANDARD_MATERIAL_MIN_ROUGHNESS` 给出；
- 当前标准材质纹理 UV channel 数为 2，`unsupported_texture_uv_channels()` 可在提交前诊断越界；
- normal scale 与 occlusion strength 有独立字段；
- `resolved_render_queue_value()` 将新版 typed queue 与旧 authored integer 统一；
- `alpha_mode` 决定 opaque/alpha-mask/transparent phase，不能只靠颜色 alpha；
- `cast_shadows` 和 `receive_shadows` 独立。

Advanced PBR 的 clearcoat、anisotropy、transmission/IOR 等有真实 shader/pass 支持，但应按 capability/profile 视为**可选可用**。Subsurface 还需要 profile table 和对应 Pass executor。

## MaterialDomain 与着色模型

`MaterialDomain` 分为 `Surface`、`PostProcess`、`DebugOverlay`、`LightFunction`。Domain 决定允许的资源和 Pass，不是渲染队列别名。Shading model 由稳定 ID/descriptor 注册；内置 Standard PBR、Unlit、Blinn-Phong，插件模型需要 module source 和 pipeline template 共同接入。

## TextureAsset

`TextureAsset` 支持：

- `new_rgba8`：CPU RGBA8，默认 sRGB 2D descriptor；
- `new_container`：容器/压缩 payload，记录 format、mip 和 array layer；
- `TextureAssetDescriptor`：维度、色彩空间、mip、normal convention、compression/usage 等导入语义；
- row count/row height 形式的 2D array layout 转换。

`apply_import_settings` 会验证 array 必须来自单层 2D RGBA8，宽高/byte length 必须匹配。`render_image_descriptor()` 将资产 metadata 投影为中立渲染描述。

纹理上传必须遵守 format block、bytes-per-row、mip/layer 范围与 usage。资源流送器提供 fallback texture 和 readiness report；缺贴图不应产生未绑定采样，而应根据材质 fallback policy 使用明确替代或拒绝绘制。

## 资源缓存与热更新

资源身份通常由 `ResourceId + revision + device generation + relevant descriptor` 决定。热更新会使 material runtime、texture view/bind group 和 pipeline key 中相应部分失效。只修改文件时间而不产生新 revision 不应被依赖为稳定行为。

Pipeline cache 按几何源、shader pass、材质 feature、格式、MSAA、shadow/forward/deferred 变体分键。大量材质组合应配合 shader prewarm，否则首帧可能出现编译开销或明确 fallback。

## 状态说明

- 静态/索引 Mesh、PBR 材质、常用纹理、mip/array、蒙皮与 morph 为**默认可用**。
- Mesh SDF、Virtual Geometry payload、advanced PBR 为**可选/实验性**，需要消费者路径。
- Sparse texture/bindless residency 是能力合同，不能假定所有 WGPU adapter 支持。
