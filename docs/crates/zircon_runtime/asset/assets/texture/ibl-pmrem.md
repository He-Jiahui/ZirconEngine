---
related_code:
  - zircon_runtime/src/asset/assets/texture/ibl_pmrem.rs
  - zircon_runtime/src/asset/assets/texture/mod.rs
  - zircon_runtime/src/core/framework/render/environment/ibl_bake_artifact.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/probe_buffer/upload.rs
implementation_files:
  - zircon_runtime/src/asset/assets/texture/ibl_pmrem.rs
plan_sources:
  - user: 2026-07-10 完善 PBR HDRI 与反射探针资产到渲染链路
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
tests:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/probe_buffer/tests/upload.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/probe_buffer/tests/resources.rs
  - zircon_runtime/src/graphics/tests/project_render/project_scenes/reflection_probe_product.rs
doc_type: module-detail
---

# IBL PMREM Texture Adapter

## 职责

`ibl_pmrem.rs` 是 IBL bake artifact 与通用 `TextureAsset` 之间的唯一适配 owner。它只提取已经烘焙完成的 PMREM 段，不负责 equirectangular 重投影、source cubemap mip 生成或 GGX/cosine 卷积。

该边界避免三类错误：

- source `.zcube` 被当作可直接采样的镜面 PMREM；
- HDR 数据在进入 GPU 前被量化成 RGBA8；
- 旧算法版本或错误 mip/face 布局被静默接受。

## 容器合同

当前格式标签为 `zircon/ibl-pmrem-rgba16f-v1`，GPU 格式为 `rgba16float`。适配后的 `TextureAsset` 必须满足：

| 字段 | 合同 |
|---|---|
| dimension | `Cube` |
| color space | `Linear` |
| face count | 6，顺序为 `+X,-X,+Y,-Y,+Z,-Z` |
| extent | 使用 bake artifact descriptor 的方形 `face_size` |
| mip count | 使用 bake artifact descriptor 的完整 PMREM mip 数 |
| texel | 每通道 IEEE 754 binary16，RGBA 共 8 bytes |
| payload order | mip-major，每个 mip 内六面连续 |

`texture_asset_from_ibl_bake_artifact_pmrem(...)` 先检查 artifact 算法版本和 PMREM section，再构造 Cube descriptor，最后立即调用解码校验。构造成功即表示该纹理可以进入探针 PMREM 上传边界。

## 校验与错误

`IblPmremTextureError` 保留具体失败原因：缺少 PMREM、算法版本过期、容器类型错误、维度错误、面数错误、非线性/非 RGBA16F descriptor、payload 长度错误和尺寸计算溢出。公共适配入口不把这些原因压扁成字符串。

payload 长度按所有 mip 的 `max(face_size >> mip, 1)^2 * 6 * 8` 求和，并使用 checked arithmetic。这样既覆盖 1x1 尾 mip，也拒绝异常尺寸造成的整数溢出。

## 与 ReflectionProbe 的边界

探针上传只接受当前 RGBA16F PMREM 容器。`probe_buffer/upload.rs` 额外固定 V1 探针资源为 128x128、8 mip、6 faces，并明确拒绝 source cubemap。PMREM 中大于 1.0 的 HDR 辐射值必须保持到 `Rgba16Float` cube-array，不能因测试截图输出为 UNORM 而改变资产合同。

## 验证状态

2026-07-10 聚焦资源/GPU 合同共 14 项通过，覆盖：

- artifact PMREM 到 Cube `TextureAsset` 的描述与长度校验；
- source `.zcube` 拒绝；
- RGBA8/错误尺寸/错误 mip 拒绝；
- RGBA16F GPU 上传与大于 1.0 的 HDR 数值保留。

产品级双探针平滑边界已经通过；`WgpuRenderFramework` 的功能关闭回退仍在单独的产品测试阶段，不由本适配模块宣称完成。
