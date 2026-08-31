---
handoff_kind: failure
status: open
created_at: 2026-08-31
summary_slug: ssao-scene-uniform-lightmapped-ambient-offset
origin_plan: docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
fixing_plan: docs/plans/zircon_runtime/render/07-postprocess-color-pipeline.md
origin_child_dir: docs/plans/zircon_runtime/shader/06
fixing_child_dir: docs/plans/zircon_runtime/render/07
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/scene_uniform.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/ssao_spatial_denoise.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process/shaders/ssao_bilateral_upsample.wgsl
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/screen_space_ambient_occlusion.rs
tests:
  - managed Naga validation for ssao_spatial_denoise.wgsl and ssao_bilateral_upsample.wgsl
  - managed WGPU half-resolution GTAO spatial-denoise and bilateral-upsample product lane
---

# Render07: SSAO SceneUniform lightmapped ambient offset drift

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md`
- 来源执行切片：P1-18 authored ambient/lightmapped ambient SceneUniform ABI audit
- 修复责任计划：`docs/plans/zircon_runtime/render/07-postprocess-color-pipeline.md`
- 交接原因：最低共享根因位于 Render07 当前在途的 half-resolution GTAO spatial-denoise / bilateral-upsample shader owner；两份 shader 和其 descriptor 均处于 Render07 的未提交变更集，Shader06 不得拆开覆盖。

## 失败现象与复现证据

CPU `SceneUniform` 的当前合同为 `ambient_color @ 192`、`lightmapped_ambient_color @ 208`、`previous_view_proj_unjittered @ 224`，总大小 496 bytes。canonical `zr_scene_runtime.wgsl` 与当前 `ssao.wgsl` 已保持该字段顺序。

Render07 新增的两份生产 shader 仍在 `ambient_color` 后直接声明 `previous_view_proj_unjittered`：

- `ssao_spatial_denoise.wgsl` SHA-256 `467DD842AAB28CD531E38ED28193A47AEE99C60559CE53E097F6A4A109E515D7`
- `ssao_bilateral_upsample.wgsl` SHA-256 `C772B43A110322614E97385B32C07CBEF9885AB1D34D715AD7A6C9308F7412DE`

两份 shader 均由 `screen_space_ambient_occlusion.rs` 的 builtin `include_str!` 直接装配，并读取 `inverse_view_proj`、`camera_world_position`、`camera_view_direction`。`inverse_view_proj` 位于新增字段之前，仍正确；`previous_view_proj_unjittered` 及之后字段的 WGSL 偏移比 CPU 上传合同少 16 bytes，因此 camera position/direction 会读取错误槽位。当前 source review 不能把 half-resolution AO 标为 ABI-correct 或动态可验收。

## 最低共享层根因

P1-18 扩展共享 `SceneUniform` 时，Render07 同期新增的两个未跟踪 shader 没有消费 canonical SceneUniform 字段序列，也没有通过结构守卫覆盖所有“声明超过 `ambient_color` 的 SceneUniform 镜像”。这是共享 scene ABI 漂移，不是 AO 采样算法或 bind-group schema 问题。

## 架构修复验收

- 两份 Render07 shader 都必须在 `ambient_color` 后声明 `lightmapped_ambient_color: vec4<f32>`，字段顺序与 `zr_scene_runtime.wgsl` 一致；不得改变 AO 算法、资源 binding 或 dispatch。
- 增加一个覆盖 production WGSL 镜像的 SceneUniform prefix/order 守卫：凡镜像声明并读取 `previous_view_proj_unjittered` 或其后字段，必须包含 `lightmapped_ambient_color` 且顺序一致；只读取首字段 `view_proj` 的合法前缀镜像不应被迫复制完整 ABI。
- 两份 shader 的 managed Naga validation 通过，half-resolution GTAO spatial-denoise / bilateral-upsample WGPU 产品 lane 无 validation error，camera motion 与 orthographic/perspective reconstruction 结果保持正确。
- Shader06 P1-18 的广域 SceneUniform ABI gate 重新执行后才能从 partial 更新为 accepted。

## 禁止临时方案

- 不得在 CPU 上传端恢复旧 480-byte layout，或为 SSAO 单独生成旧布局 buffer。
- 不得用匿名 padding、手工 byte offset、调用点重写或 shader fallback 掩盖字段漂移。
- 不得把两份 shader 从 descriptor 拆除来规避验证；修复必须保留 Render07 已声明的 half-resolution AO 拓扑。
- 不得削弱 Naga/WGPU、camera reconstruction 或 SceneUniform 顺序验收。

## 修复结果与回传

Open state: `待修复`; no pass is claimed.
