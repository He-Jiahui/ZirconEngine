---
related_code:
  - zircon_runtime/src/core/framework/render/environment/reflection_probe.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/probe_buffer/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/probe_buffer/gpu_layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/probe_buffer/resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/probe_buffer/slot_allocator.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/probe_buffer/upload.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_environment.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/shaders/deferred_lighting.wgsl
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/tests/runtime_camera_core_pipeline_contract.rs
  - zircon_runtime/tests/runtime_environment_reflection_probe_product_contract.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/probe_buffer/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/probe_buffer/gpu_layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/probe_buffer/resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/probe_buffer/slot_allocator.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/probe_buffer/upload.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_environment.wgsl
plan_sources:
  - user: 2026-07-10 完善真实 HDRI PBR 反射、方向、掠射与多视角验证
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
  - dev/UnrealEngine/Engine/Shaders/Private/ReflectionEnvironmentShared.ush
  - dev/UnrealEngine/Engine/Shaders/Private/ReflectionEnvironmentComposite.ush
  - dev/cmft/src/cmft/cubemapfilter.cpp
  - dev/cmftStudio/src/shaders/fs_mesh.shdr
tests:
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/probe_buffer/tests/gpu_layout.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/probe_buffer/tests/reference_parity.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/probe_buffer/tests/resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/probe_buffer/tests/slot_allocator.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/probe_buffer/tests/upload.rs
  - zircon_runtime/src/graphics/tests/project_render/project_scenes/reflection_probe_product.rs
  - zircon_runtime/tests/runtime_camera_core_pipeline_contract.rs
  - zircon_runtime/tests/runtime_environment_reflection_probe_product_contract.rs
doc_type: module-detail
---

# Reflection Probe GPU Buffer

## 职责与边界

`probe_buffer/` 拥有已烘焙 ReflectionProbe 从 `EnvironmentExtract` 到 GPU 可采样资源的帧级准备过程。它不负责捕获场景、烘焙 PMREM，也不拥有 post-process 的旧投屏探针 DTO。

模块按 owner 拆分：

- `gpu_layout.rs`：CPU/WGSL ABI 与 group 1 bindings；
- `resources.rs`：buffer、cube-array、候选选择和每帧上传；
- `slot_allocator.rs`：资源 revision 感知的 LRU 槽位；
- `upload.rs`：PMREM 资产校验和逐 mip/face 上传。

## GPU ABI

V1 最多同时激活 64 个探针。每个探针固定 96 bytes，按六个 `vec4` 对齐：

| 字段 | 内容 |
|---|---|
| `position_blend` | 世界位置 xyz，w 为 blend distance |
| `box_min` | 本地影响范围最小值 xyz，w 为 priority |
| `box_max` | 本地影响范围最大值 xyz，w 为 shape（box=0, sphere=1） |
| `proj_params` | box projection half extents xyz，w 为启用标记 |
| `rotation` | probe local-to-world quaternion |
| `misc` | intensity、mip count、cube-array slice、layer mask bits |

group 1 使用以下固定 binding：

| Binding | 资源 |
|---|---|
| 16 | read-only storage buffer：`GpuReflectionProbe[]` |
| 17 | uniform buffer：16-byte probe count header |
| 18 | filterable `texture_cube_array<f32>` PMREM |

PMREM cube-array 为 `Rgba16Float`，每槽 128x128、6 faces、8 mips，总层数 `64 * 6 = 384`。设备请求的 array-layer limit 必须覆盖该固定 ABI。

## 帧级选择与上传

`prepare(...)` 先记录 authored probe 数量。功能关闭时只把 GPU header 写为 0，因此 shader 必然回退到 sky IBL，旧帧槽内容不会被误采样。

功能开启时依次执行：

1. 丢弃没有 baked cubemap、intensity 为 0 或与相机 layer 不相交的探针；
2. 按相机到 influence 的距离升序、priority 降序、probe id 升序稳定排序；
3. 截断到 64 个候选；
4. 校验当前 128x128x6x8 RGBA16F PMREM；
5. 依据 `ResourceId + revision` 分配或复用 power-of-two LRU 槽；
6. 仅在首次分配或 revision 改变时重传所有 mip/faces；
7. 写 storage buffer 和最终 probe count header。

上传失败不会把无效探针写入 active buffer；报告保留 extracted、active、uploaded、rejected 数量和首个 typed rejection reason。

## Shader 采样

`zr_environment.wgsl` 对每个世界位置计算 box/sphere influence，选出权重最高的两个探针，并按剩余权重回退到 sky PMREM。该路径同时被 standard material template、fallback forward shader 和 deferred lighting 使用。

盒投影在 probe local space 中求反射射线与 AABB 的远交点，再旋转回世界方向；sphere 使用球形 influence，不做盒投影。cube face 方向和 PMREM face order 与 cmft/UE 约定保持一致。

## 验证状态

2026-07-10 已通过 14 个聚焦合同，覆盖 96/16-byte ABI、bindings、LRU/revision、候选排序、box/sphere 权重、top-2、盒投影、PMREM 拒绝规则、GPU buffer/cube-array readback 和 HDR 值保留。

产品截图测试 `render_product_probe_blend_boundary_smooth` 已通过，证明左右红/蓝 PMREM 在重叠区连续、单调混合。公开 WGPU 产品合同 `runtime_environment_reflection_probe_product_contract` 也已通过：同一正交 Core3d 场景分别提交探针开启、feature 关闭、无探针天空三帧，关闭态与天空逐像素一致 (`MAE=0.000000`)，开启态与天空保持明显差异 (`MAE=5.126736`，门槛 `>=4.0`)。

框架边界诊断确认旧问题来自投影和核心管线错误耦合：正交 PBR 相机曾被推断为 Core2d，导致编译后的 Core3d 管线没有 reflection-probe feature，而直接 renderer 路径仍能上传探针。相机现在显式携带 `core_pipeline` 且缺省为 Core3d，和 `ProjectionMode` 独立；产品夹具保持正交构图并在提交前断言 Core3d。公共相机合同 3/3 与公开 WGPU feature on/off 产品合同 1/1 均已通过，ReflectionProbe 资源、混合和功能关闭回退门槛已闭合；捕获/编辑器烘焙流程仍属于后续切片。
