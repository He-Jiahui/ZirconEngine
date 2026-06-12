---
related_code:
  - zircon_runtime/src/core/framework/render/light/snapshots.rs
  - zircon_runtime/src/core/framework/render/light/readiness.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred/geometry_pipeline/mod.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/shadows.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_deferred.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/default_forward_plus.rs
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/LightGridInjection.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/LightRendering.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ShadowSetup.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/ShadowDepthRendering.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Shadows/ShadowSceneRenderer.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/VirtualShadowMaps/VirtualShadowMapArray.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/ForwardLights.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/DeferredLights.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/ShadowUtils.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/Passes/MainLightShadowCasterPass.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/Passes/AdditionalLightsShadowAtlasLayout.cs
plan_sources:
  - .codex/plans/Hybrid GI Lumen-Style V1 三阶段计划.md
  - .codex/plans/Rendering 插件选项补齐计划.md
  - .codex/plans/ZirconEngine Bevy-Level Rendering Completion Plan.md
---

# 计划 05:光照与阴影管理

## 目标

解除"灯光走场景 uniform、上限 4 点光"的限制,建成 clustered light grid + GPUScene light buffer 的光照底座,
并把阴影从单一 shadow map 升级为 cascade 完整化 + 多光源 shadow atlas 的管理体系。完成后:

1. Forward+ 与 Deferred 共用同一份 light grid 与 light buffer(对齐 HGI 计划"同一套 GI 算法"原则)。
2. 场景灯光数量仅受 buffer 容量约束(数百级),支持 directional/point/spot/rect 全类型进 grid。
3. 方向光 CSM(级联划分、稳定化、过渡带)与 point/spot 阴影共享 atlas,槽位按优先级与距离动态分配。
4. 阴影渲染 view 经计划 04 独立剔除、计划 02 的 ShadowPassProcessor 生成命令。

## 现状与差距

- 灯光快照类型齐全(`light/snapshots.rs`),但消费端落在 BASIC_SCENE_UNIFORM,数量受限;cluster grid 已创建却没有 per-cluster light list 注入,clustered 描述符是空壳。
- `shadow_map_renderer.rs` 有方向光级联与点光基础,但级联划分/稳定化(texel snapping)/级联过渡未成体系,无 atlas 管理,多光源阴影互斥。
- 无 per-light 阴影参数面(bias/normal bias/分辨率档位)契约。

## 参考代码

| 文件 | 应重点阅读 |
|------|-----------|
| `dev/UnrealEngine/.../Renderer/Private/LightGridInjection.cpp` | froxel grid 构建 compute:cluster AABB 与灯光形状求交、灯光索引链表压缩为紧凑列表 |
| `dev/UnrealEngine/.../Renderer/Private/LightRendering.cpp` | deferred 光照 pass 的组织:per-light 体积光栅化 vs tiled/clustered 全屏 pass 的取舍 |
| `dev/UnrealEngine/.../Renderer/Private/ShadowSetup.cpp` | 级联划分(对数/线性混合 split)、阴影视锥构造、caster 收集边界 |
| `dev/UnrealEngine/.../Renderer/Private/ShadowDepthRendering.cpp` | shadow depth pass 与专用 pass processor(bias、slope-scale 状态) |
| `dev/Graphics/.../Runtime/ForwardLights.cs` | URP Forward+ 的 light buffer/cluster 打包(zbin + tile 列表),比 UE 更贴近本引擎规模,推荐作为 grid 数据布局首选样板 |
| `dev/Graphics/.../Runtime/Passes/AdditionalLightsShadowAtlasLayout.cs` + `ShadowUtils.cs` | 多光源 shadow atlas 槽位分配、按分辨率档位打包、texel snapping 稳定化 |
| `dev/UnrealEngine/.../VirtualShadowMaps/VirtualShadowMapArray.cpp` | VSM 远期预研(本计划只读不实施) |

## 目标架构

归属:`graphics/scene/scene_renderer/` 下新增 `lighting/`(grid 构建与光照 pass)并升级 `shadow/`;灯光契约扩展在 `core/framework/render/light/`。

核心设计:

- `GpuLightData`:进计划 03 GpuScene 的 light buffer(类型、位置/方向、颜色强度、范围/锥角、阴影槽位索引、layer mask);extract 端 `light/snapshots.rs` 增加稳定 light id。
- `LightGridPass`(compute):URP 风格 zbin + tile 灯光列表(比 UE 链表更简单,wgpu storage buffer 友好);输出 grid header + 紧凑索引表。forward 着色与 deferred lighting pass 同读此 grid。
- deferred lighting 改造:从"场景 uniform 灯光循环"切到"grid 驱动循环";forward+ 同一 WGSL include 共享采样函数。
- `ShadowAtlas`:单张大 depth 纹理 + 槽位分配器(2 的幂档位,按光源屏幕占比/优先级分配与降档);方向光 CSM 占独立 atlas 区段;级联划分采用对数线性混合 + texel snapping + 级联间过渡带。
- per-light 阴影参数契约:bias/normal bias/分辨率偏好/是否投影进 `light/` 契约与编辑器面板对接;`casts_shadow` relevance 由计划 04 提供。
- 多 render layer:灯光 layer mask 与计划 09 的 RenderLayer 对齐,着色时按 mask 过滤(Unity Light culling mask 语义)。

## 里程碑

### LS-M1 GpuLightData 与灯光上限解除

实施切片:
1. light buffer 进 GpuScene(依赖计划 03 GS-M1);extract 增加 light id 与脏更新。
2. 内建 shader 灯光循环改读 buffer(暂全量循环,不分簇);删除 BASIC_SCENE_UNIFORM 灯光段。

测试阶段:
- `cargo check -p zircon_runtime --lib --locked`;`cargo test -p zircon_runtime light --locked` 与 `render_product` 回归
- 验收证据:>4 点光场景全部生效(产物对拍);灯光增删改增量上传断言。

### LS-M2 clustered light grid

实施切片:
1. zbin+tile grid 构建 compute pass(graph 节点);grid 资源经计划 01 瞬态池。
2. forward 与 deferred 着色切换为 grid 驱动;共享 WGSL include。
3. grid 统计(平均/峰值每簇灯光数)进 RenderStats。

测试阶段:
- `cargo test -p zircon_runtime lighting --locked`(grid CPU 参考实现对拍 compute 结果)
- 验收证据:百灯场景着色成本与灯光局部密度相关而非总数(统计);两管线产物一致性对拍。

### LS-M3 CSM 完整化与 shadow atlas

实施切片:
1. 级联划分/稳定化/过渡带;`ShadowAtlas` 槽位分配器与降档策略。
2. point(立方体 6 面或双抛物面)/spot 阴影入 atlas;`GpuLightData` 写阴影槽位与矩阵。
3. shadow pass 接计划 02 `ShadowPassProcessor` 与计划 04 per-light view 剔除。

测试阶段:
- `cargo test -p zircon_runtime shadow --locked`(分配器单测:档位、驱逐、降档)+ `render_product` 阴影场景
- 验收证据:相机平移时方向光阴影边缘无 swimming(texel snapping 生效,抓帧对比);多 spot 阴影并存。

### LS-M4 PCF 质量与 contact shadow(可选 feature)

实施切片:
1. PCF 核(质量分档:1/5/9 tap)与 per-light bias 体系定稿。
2. contact shadow(屏幕空间短距离 ray march,读 HZB)作为 rendering 插件可选 feature 接入。

测试阶段:
- `cargo test -p zircon_runtime shadow --locked` 与 `cargo test --manifest-path zircon_plugins/Cargo.toml -p <rendering feature crate> --locked`
- 验收证据:质量档位切换产物差异符合预期;feature 关闭时 graph 无对应 pass。

## 工程落地细化

本章为计划 05 的实施权威(见 index.md §8 第 7 条)。bind group 槽位、std430 约定、WGSL include 前缀、测试命名等全局约定直接引用 index.md §8,不在此重定义。跨计划契约名原样使用:计划 01 `RgTextureHandle`/`RgBufferHandle`/`TransientResourcePool`、计划 02 `MeshPassProcessor`(语义槽命令)、计划 03 `GpuScene`/`GpuSceneIdAllocator`、计划 04 `ViewVisibilityContext`/`PrimitiveRelevance`、计划 09 `RenderLayer`、计划 10 `RendererCommon`。

### 模块与文件落点

新增文件(facade 固定 `zircon_runtime::core::framework::render`,实现归 `graphics/`):

| 路径 | 内容 | 层 |
|------|------|----|
| `zircon_runtime/src/core/framework/render/light/gpu_light.rs` | `GpuLightData`、`GpuLightType`、`SHADOW_SLOT_NONE`(纯 POD + bytemuck,无 wgpu) | framework 契约 |
| `zircon_runtime/src/core/framework/render/light/shadow_settings.rs` | `LightShadowSettings`(bias/normal bias/分辨率偏好/strength,编辑器面板对接面) | framework 契约 |
| `zircon_runtime/src/graphics/scene/scene_renderer/lighting/mod.rs` | 模块 wiring(thin) | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_buffer.rs` | light buffer 进 `GpuScene`:打包 snapshots → `GpuLightData`、脏更新、容量增长 | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_grid_builder.rs` | CPU zbin+tile 构建(URP Jobs 形态的 Rust 移植)+ `LightGridParams` 计算 | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_grid_pass.rs` | `lighting.light-grid` executor:buffer 写入与 grid 统计进 RenderStats | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/lighting/shaders/zr_light_grid.wgsl` | 共享 include:grid 查询函数(无 entry point) | WGSL |
| `zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/mod.rs` | `ShadowAtlas`:atlas 纹理生命周期 + 槽位表 | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/shadow/atlas/allocator.rs` | `ShadowAtlasAllocator`(shelf 分配器)+ 滞回/降档策略 | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/shadow/cascade.rs` | `CascadeSplitConfig`、log/linear 混合分割、texel snapping、过渡带计算 | graphics 实现 |
| `zircon_runtime/src/graphics/scene/scene_renderer/shadow/shaders/zr_shadow.wgsl` | 共享 include:slot 采样、CSM 选择、PCF 核 | WGSL |

修改文件:

| 路径 | 改动 |
|------|------|
| `zircon_runtime/src/core/framework/render/light/snapshots.rs` | 各 snapshot 增加 `light_id: u64`(稳定 id,extract 端生成)、`layer_mask: u32`、`shadow: Option<LightShadowSettings>` |
| `zircon_runtime/src/core/framework/render/light/readiness.rs` | 删除 `BASIC_SCENE_UNIFORM_*_LIMIT` 与 `ready_point_light_count` 等截断逻辑;readiness 改为 buffer 容量导向(LS-M1 硬切换) |
| `zircon_runtime/src/core/framework/render/light/mod.rs` | re-export 更新(thin) |
| `zircon_runtime/src/core/framework/render/post_process/stack.rs` | `PostProcessGraphResourceNames` 新增 `LIGHT_BUFFER`、`LIGHT_ZBINS`、`LIGHT_TILE_MASKS`、`SHADOW_ATLAS`、`SHADOW_SLOTS`;既有 `LIGHT_LIST`/`SHADOW_MAP` 名删除 |
| `zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/scene_uniform.rs` | 删除 `light_dir`/`light_color`/`point_light_position_range`/`point_light_color_intensity`/`point_light_params` 字段(ambient 保留) |
| `zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs` | 删除 `authored_point_light_data`/`empty_point_light_data` |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl` + `mesh/mesh_pipeline/fallback_mesh_shader_source.rs` | 灯光循环改读 light buffer(LS-M1 全量循环→LS-M2 grid 驱动);shadow 采样按计划 03 GS-M2 槽位重排进 group1 |
| `zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/{create.rs,shader_source.rs}` + `deferred/lighting_bind_group_layout/` | deferred lighting 从 uniform 灯光循环切 grid 驱动;group1 layout 与 forward 共用同一布局常量 |
| `zircon_runtime/src/graphics/scene/scene_renderer/shadow/shadow_map_renderer.rs` | LS-M3 整体替换:`shadow_light`/`shadow_scene_uniform`/`shadow_view_projection` 删除,命令生成走计划 02 `MeshPassProcessor`,view 来自 `cascade.rs` + atlas 槽位 |
| `zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/clustered_lighting.rs` | pass 改名 `light-grid-build`,IO:`write_buffer(LIGHT_ZBINS)` + `write_buffer(LIGHT_TILE_MASKS)`(不再 read SCENE_DEPTH,zbin/tile 为 CPU 构建,见帧时序) |
| `zircon_runtime/src/graphics/feature/builtin_render_feature_descriptor/feature_descriptors/shadows.rs` | `write_texture_with_ops(SHADOW_ATLAS, ...)`;executor id 改 `shadow.atlas` |
| `zircon_runtime/src/graphics/pipeline/render_pipeline_asset/{default_forward_plus.rs,default_deferred.rs}` | feature 列表不变(`Shadows`/`ClusteredLighting` 复用),无新增 stage |

### 核心类型与接口

framework 契约层(`light/gpu_light.rs`,无 wgpu):

```rust
#[repr(u32)]
pub enum GpuLightType { Directional = 0, Point = 1, Spot = 2, Rect = 3 }

pub const SHADOW_SLOT_NONE: u32 = 0xFFFF_FFFF;

/// std430,96 B/灯;偏移见下节布局表。
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, PartialEq)]
pub struct GpuLightData {
    pub position_range: [f32; 4],     // xyz 位置(directional 不用),w = range
    pub color_intensity: [f32; 4],    // rgb 线性色,w = intensity
    pub direction_type: [f32; 4],     // xyz 方向,w = f32::from_bits(GpuLightType)
    pub spot_angles_size: [f32; 4],   // x cos(inner) y cos(outer);rect: zw 半宽/半高
    pub shadow_slot_layer: [u32; 4],  // x 首 shadow 槽(或 SHADOW_SLOT_NONE),y layer_mask,z light_id 低 32 位,w flags
    pub shadow_params: [f32; 4],      // x strength,y depth_bias,z normal_bias,w 级联数/槽数
}
```

```rust
// light/shadow_settings.rs(framework 契约)
pub struct LightShadowSettings {
    pub casts_shadow: bool,
    pub depth_bias: f32,          // 以 shadow texel 世界尺寸为单位(URP GetShadowBias 语义)
    pub normal_bias: f32,
    pub strength: f32,
    pub resolution_preference: ShadowResolutionTier, // 见 atlas 档位表
}
```

graphics 实现层:

```rust
// lighting/light_grid_builder.rs
pub(crate) struct LightGridParams {
    pub zbin_scale: f32, pub zbin_offset: f32,
    pub bin_count: u32, pub words_per_tile: u32,
    pub tile_resolution: [u32; 2], pub tile_size_px: u32, pub light_count: u32,
}
pub(crate) struct LightGridCpuOutput { pub zbins: Vec<u32>, pub tile_masks: Vec<u32>, pub params: LightGridParams }
pub(crate) fn build_light_grid(lights: &[GpuLightData], view: &LightGridViewInfo) -> LightGridCpuOutput;

// shadow/atlas/allocator.rs
pub(crate) enum ShadowResolutionTier { T128 = 128, T256 = 256, T512 = 512, T1024 = 1024, T2048 = 2048 }
pub(crate) struct ShadowSlotRequest { pub light_id: u64, pub face_index: u8, pub tier: ShadowResolutionTier, pub priority_score: f32 }
pub(crate) struct ShadowSlotAllocation { pub slot_index: u32, pub atlas_rect_px: [u32; 4], pub tier: ShadowResolutionTier }
impl ShadowAtlasAllocator {
    pub fn allocate_frame(&mut self, requests: &[ShadowSlotRequest]) -> Vec<ShadowSlotAllocation>; // 含滞回与降档
}

// shadow/cascade.rs
pub(crate) struct CascadeSplitConfig { pub cascade_count: u32 /*<=4*/, pub max_distance: f32, pub log_linear_lambda: f32 /*默认 0.7*/, pub fade_fraction: f32 /*默认 0.1*/ }
pub(crate) fn compute_cascade_splits(cfg: &CascadeSplitConfig, near: f32) -> [f32; 5];
pub(crate) fn snapped_cascade_view_proj(light_dir: Vec3, frustum_slice: &FrustumSlice, resolution: u32) -> Mat4; // texel snapping
```

阴影 view 剔除经计划 04 `ViewVisibilityContext`(per-light view,`casts_shadow` relevance 由 `PrimitiveRelevance` 提供);shadow pass 命令经计划 02 `MeshPassProcessor` 生成语义槽命令,不直接 `set_pipeline`。

### GPU 数据布局与 WGSL 约定

`GpuLightData` std430 偏移(96 B/灯,light buffer 为 `array<GpuLightData>` storage,进 `GpuScene` 管理,id 由 `GpuSceneIdAllocator` 分配):

| 偏移 | 字段 | 类型 |
|------|------|------|
| 0 | position_range | vec4f |
| 16 | color_intensity | vec4f |
| 32 | direction_type | vec4f(w 位转 u32 类型) |
| 48 | spot_angles_size | vec4f |
| 64 | shadow_slot_layer | vec4u |
| 80 | shadow_params | vec4f |

zbin buffer(`array<u32>`):每 bin 跨度 `2 + words_per_tile` 个 u32。word0 = 灯光 header `min_index & 0xFFFF | max_index << 16`(URP `ZBinningJob.EncodeHeader` 同构;空 bin 为 `0xFFFF | 0 << 16`);word1 = 保留给计划 11 反射探针 header;word2.. = 灯光位掩码(bit i = 灯 i 与该 bin 相交)。bin 索引(透视):`bin = log2(view_z) * zbin_scale + zbin_offset`,其中 `zbin_scale = ZR_MAX_ZBIN_WORDS / ((log2(far) - log2(near)) * (2 + words_per_tile))`、`zbin_offset = -log2(near) * zbin_scale`(URP ForwardLights.cs 同式;正交用线性 z)。常量:`ZR_MAX_ZBIN_WORDS = 4096`、`ZR_MAX_TILE_WORDS = 8192`。

tile mask buffer(`array<u32>`):`tile_masks[(tile_y * tile_res.x + tile_x) * words_per_tile + word]`,bit i = 灯 i 覆盖该 tile。`words_per_tile = (light_count + 31) / 32`;tile 边长从 8px 起倍增直至 `tile_res.x * tile_res.y * words_per_tile <= ZR_MAX_TILE_WORDS`(URP `actualTileWidth` 同策略)。着色时灯光遍历集合 = `zbin_word & tile_word` 的按位与(URP 双重裁剪同构)。

group1(pass 级,index.md §8;槽位编号基于计划 03 GS-M2 重排后的 fallback_mesh.wgsl,forward 着色 pass 与 deferred lighting pass 同一布局):

| binding | 资源 | 类型 |
|---------|------|------|
| 0 | light_buffer | `var<storage, read> array<GpuLightData>` |
| 1 | light_grid_params | `var<uniform> ZrLightGridParams` |
| 2 | light_zbins | `var<storage, read> array<u32>` |
| 3 | light_tile_masks | `var<storage, read> array<u32>` |
| 4 | shadow_atlas | `texture_depth_2d` |
| 5 | shadow_sampler | `sampler_comparison` |
| 6 | shadow_slots | `var<storage, read> array<ZrShadowSlot>` |
| 7 | shadow_globals | `var<uniform> ZrShadowGlobals`(级联 split vec4、fade 带宽、atlas 尺寸) |

`ZrShadowSlot`(std430,96 B):`view_proj: mat4x4f`(offset 0)、`atlas_scale_bias: vec4f`(64,xy scale zw offset,归一化 atlas UV)、`params: vec4f`(80,x depth_bias y normal_bias z slot_texel_size w flags)。

`zr_light_grid.wgsl`(include,只含函数与 struct):

```wgsl
struct ZrLightGridParams { zbin_scale: f32, zbin_offset: f32, bin_count: u32, words_per_tile: u32,
                           tile_resolution: vec2<u32>, tile_size_px: u32, light_count: u32 }
fn zr_light_zbin_index(view_z: f32, p: ZrLightGridParams) -> u32
fn zr_light_tile_base(frag_coord: vec2<f32>, p: ZrLightGridParams) -> u32   // tile_masks 基址
fn zr_light_zbin_header(bin: u32, p: ZrLightGridParams) -> vec2<u32>        // (min_index, max_index)
fn zr_light_mask_word(tile_base: u32, bin: u32, word: u32, p: ZrLightGridParams) -> u32 // zbin_word & tile_word
fn zr_light_count(frag_coord: vec2<f32>, view_z: f32, p: ZrLightGridParams) -> u32      // 统计/debug 用
```

WGSL 无闭包,`for_each_light` 以调用方循环骨架落地(forward 与 deferred 共用同一段拼接模板,由计划 08 模板机制注入):外层 `for word in 0..words_per_tile`,取 `zr_light_mask_word`,内层 `while mask != 0u { let i = firstTrailingBit(mask); mask &= mask - 1u; shade(light_buffer[word*32u+i]); }`,并以 `zr_light_zbin_header` 的 min/max 截断 word 区间。layer 过滤:`(light.shadow_slot_layer.y & view_layer_mask) != 0u`(`RenderLayer` 语义,计划 09)。

`zr_shadow.wgsl`(include):

```wgsl
fn zr_shadow_pcf(atlas_uv: vec2<f32>, depth_ref: f32, taps: u32, texel: f32) -> f32   // taps ∈ {1,5,9}
fn zr_sample_shadow_slot(slot: u32, world_pos: vec3<f32>, n: vec3<f32>) -> f32        // slot 矩阵投影 + bias + PCF
fn zr_sample_csm(world_pos: vec3<f32>, view_z: f32, n: vec3<f32>) -> f32              // 级联选择 + fade 带过渡
fn zr_csm_cascade_index(view_z: f32) -> u32
```

CSM 约定:级联即 shadow slot 0..cascade_count-1;`zr_sample_csm` 在 `split[i] - fade` 区间内对 i 与 i+1 级联结果 lerp(UE `SplitNearFadeRegion`/`FadePlaneOffset` 语义的单带简化)。分割公式(log/linear 混合):`split_i = lerp(near + (far-near)*i/N, near*(far/near)^(i/N), lambda)`,`lambda` 默认 0.7(UE 等价物为 `ComputeAccumulatedScale` 指数分布,exponent 默认 4;两者曲线接近,取公式更简的混合式)。texel snapping:级联包围球半径取 slice 八角点最大距(半径跨帧锁定到 1% 步进),`world_per_texel = 2*radius/resolution`,将 light view 空间下的视锥中心 `fmod` 对齐到 `world_per_texel` 网格再回变换(UE ShadowSetup.cpp `SnapX/SnapY` 同构)。

PCF 档位表(进 `ZrShadowSlot.params.w` flags 与 pipeline 常量):

| 档位 | taps | 适用 |
|------|------|------|
| Low | 1(硬件 comparison 双线性) | 默认/低端 |
| Medium | 5(十字) | spot/point |
| High | 9(3x3) | 方向光近级联 |

ShadowAtlas:单张 `4096x4096` D32Float(capability 不足降 2048,档位整体降一档)。方向光 CSM 固定占顶部 `4096x1024` 行(4 x 1024²);其余区域 shelf 分配。

### 帧时序与集成点

帧内顺序(全部经 graph 节点 + executor id,无旁路):

1. extract:`RenderFrameExtract` 灯光 snapshots(带 light_id/layer_mask/shadow settings)。
2. prepare(CPU):`light_buffer.rs` 打包 + 脏更新上传(GpuScene grow);`cascade.rs` 计算级联;`ShadowAtlasAllocator::allocate_frame` 产出槽位;`build_light_grid` CPU 构建 zbin/tile(对齐 URP:URP 的 zbin/tile 即 CPU Jobs 构建,不依赖 depth;故 `light-grid-build` 节点不读 SCENE_DEPTH)。
3. graph 节点 `light-grid-build`(executor `lighting.light-grid`,QueueLane::AsyncCompute 保留):LS-M2 形态为 buffer 上传 + 统计;后续若切 GPU compute 构建,IO 合同不变。zbin/tile buffer 经计划 01 `TransientResourcePool` 以 `RgBufferHandle` 申请。
4. graph 节点 `shadow-atlas`(executor `shadow.atlas`):按槽位批次渲染,命令由 `MeshPassProcessor` 生成,per-light view 剔除来自计划 04。
5. depth prepass → forward 着色 / deferred geometry + lighting:同读 group1 的 grid 与 atlas。

硬切换删除清单(LS-M1/M3 各自变更内完成,不留双路径):

- `scene_uniform.rs`:`light_dir`、`light_color`、`point_light_position_range`、`point_light_color_intensity`、`point_light_params` 字段及 `from_frame.rs` 的 `authored_point_light_data`/`empty_point_light_data`。
- `readiness.rs`:`BASIC_SCENE_UNIFORM_DIRECTIONAL_LIGHT_LIMIT`、`BASIC_SCENE_UNIFORM_POINT_LIGHT_LIMIT`、`ready_directional_light_count`、`ready_point_light_count`(spot `ready=0` 的硬编码一并删除)。
- `fallback_mesh.wgsl` 与 deferred `lighting_pipeline/shader_source.rs`:scene uniform 灯光循环段。
- `shadow_map_renderer.rs`:`shadow_light`、`shadow_light_from_directional`、`shadow_scene_uniform`、`shadow_view_projection`、`shadow_bounds_from_frame`、`ShadowMapRenderer` 自持 pipeline/`record_with_attachment_ops` 直录路径(LS-M3)。
- `post_process/stack.rs`:`LIGHT_LIST`、`SHADOW_MAP` 资源名(被 `LIGHT_ZBINS`/`LIGHT_TILE_MASKS`/`SHADOW_ATLAS` 取代),消费点同变更内改名。

### 实施切片细化

LS-M1(灯光上限解除):
1. 触碰:`gpu_light.rs`(新)、`snapshots.rs`、`light_buffer.rs`(新)、`readiness.rs`。要点:`GpuLightData` 定稿 + 打包/脏更新;readiness 截断逻辑删除。判据:`cargo check -p zircon_runtime --lib --locked` 过;layout 单测(96 B + 偏移断言)过。
2. 触碰:`fallback_mesh.wgsl`、deferred `shader_source.rs`、`scene_uniform.rs`/`from_frame.rs`、`stack.rs`。要点:着色循环改读 light buffer 全量循环(group1 binding0/1);scene uniform 灯光段删除。判据:`render_product` 既有场景产物不回归;>8 点光生效。

LS-M2(clustered grid):
1. 触碰:`light_grid_builder.rs`(新)、`light_grid_pass.rs`(新)、`clustered_lighting.rs`、`zr_light_grid.wgsl`(新)。要点:CPU zbin/tile 构建 + transient buffer 上传 + graph IO 改 `LIGHT_ZBINS`/`LIGHT_TILE_MASKS`。判据:CPU 参考(暴力逐灯求交)与 builder 输出 mask 全等单测过。
2. 触碰:`fallback_mesh.wgsl`、deferred lighting shader、两处 bind group layout。要点:全量循环切 grid 驱动(同一 include),保留全量循环作为 capability 回落档(同一 buffer ABI)。判据:forward/deferred 产物对拍一致;grid 统计进 RenderStats。

LS-M3(CSM + atlas):
1. 触碰:`cascade.rs`(新)、`atlas/`(新)、`shadow_settings.rs`(新)。要点:分割/snapping/fade 计算与 shelf 分配器(滞回:槽位保留 8 帧,抢占需连续 4 帧得分高 25%;降档:URP `EstimateScaleFactorNeededToFitAllShadowsInAtlas` 同策略整体除 2)。判据:分配器单测全过(档位/驱逐/滞回/降档)。
2. 触碰:`shadows.rs` descriptor、`shadow_map_renderer.rs`(删旧)、`zr_shadow.wgsl`(新)、`GpuLightData.shadow_slot_layer` 写入。要点:atlas pass 经 `MeshPassProcessor` + per-light 剔除;point 光取立方体 6 面(6 槽,双抛物面留作降档预研不实施)。判据:多 spot 阴影并存;相机平移抓帧无 swimming。

LS-M4(PCF 与 contact shadow):
1. 触碰:`zr_shadow.wgsl`、`shadow_settings.rs`。要点:1/5/9 tap 档位 + per-light bias 定稿(URP GetShadowBias 的 texel 尺度语义)。判据:档位切换产物差异符合预期。
2. 触碰:`zircon_plugins/` rendering feature crate(contact shadow,读计划 04 HzbBuilder 输出)。要点:经 RenderFeature descriptor 接入;feature 关闭时 compiled graph 无该 pass。判据:插件测试 + graph 节点存在性断言。

切片期一律 `cargo check -p zircon_runtime --lib --locked`;里程碑末 `cargo test -p zircon_runtime --lib --locked`(过滤词 `light` / `lighting` / `shadow`)。

### 测试与验收清单

| 测试函数 | 断言要点 | 位置 |
|----------|---------|------|
| `render_light_buffer_layout_matches_wgsl_offsets` | `size_of::<GpuLightData>() == 96` + `offset_of!` 六字段 | `light/gpu_light.rs` |
| `render_light_buffer_uploads_only_dirty_lights` | 增删改后上传区间仅含脏灯 | `lighting/light_buffer.rs` |
| `render_light_grid_zbin_header_encodes_min_max` | header 编码/空 bin 哨兵与 URP 语义一致 | `lighting/light_grid_builder.rs` |
| `render_light_grid_cpu_matches_brute_force_reference` | 随机灯集:builder mask == 暴力求交 mask | 同上 |
| `render_light_grid_tile_words_respect_budget` | tile 边长倍增至满足 `ZR_MAX_TILE_WORDS` | 同上 |
| `render_light_grid_layer_mask_filters_lights` | mask 不匹配的灯不进着色集合 | 同上 |
| `render_shadow_atlas_allocates_tiers_descending` | shelf 按档位降序铺排,无重叠 | `shadow/atlas/allocator.rs` |
| `render_shadow_atlas_evicts_lowest_priority_on_pressure` | 超容时低分槽被驱逐/降档 | 同上 |
| `render_shadow_atlas_hysteresis_prevents_flapping` | 分数交替互换时槽位 8 帧内不换主 | 同上 |
| `render_shadow_cascade_splits_blend_log_linear` | lambda=0/1 退化为线性/对数;单调递增 | `shadow/cascade.rs` |
| `render_shadow_cascade_snapping_quantizes_origin` | 平移相机半 texel,snapped 矩阵不变 | 同上 |
| `render_product_many_point_lights` | 64 点光场景,forward/deferred 对拍一致 | render_product 套件 |
| `render_product_csm_directional` | 4 级联边界 fade 无硬缝 | 同上 |
| `render_product_multi_spot_shadows` | ≥3 spot 阴影同帧并存 | 同上 |

### 参考实现精读笔记

- `ForwardLights.cs::CreateShadowCullingDataAsync`(zbin/tile 主流程):`zBinScale/zBinOffset` 公式(透视 log2、正交线性)、`wordsPerTile = (itemsPerTile + 31) / 32`、tile 边长自 8 起 `actualTileWidth <<= 1` 直至满足 `maxTileWords`、`LightMinMaxZJob → ZBinningJob` 与 `TilingJob → TileRangeExpansionJob` 双链并行。Zircon 对应:`build_light_grid` 单函数串行先行(灯数百级无需 Jobs),公式与 word 布局原样移植;探针项暂不进 grid,header word1 保留。
- `Tiling/ZBinningJob.cs`:`EncodeHeader(min,max) = min&0xFFFF | max<<16`、空 header `(0xFFFF, 0)`、`FillZBins` 对每灯 `[minBin, maxBin]` 区间 OR 位掩码并收紧 header。Zircon 原样移植为 `zr_light_zbin_header` 的逆操作;取舍:不做 batch 切分(无 Jobs 并行需求)。
- `AdditionalLightsShadowAtlasLayout.cs`:`ShadowResolutionRequest{visibleLightIndex, perLightShadowSliceIndex, requestedResolution, offsetX/Y, allocatedResolution}`、按分辨率排序后 `EstimateScaleFactorNeededToFitAllShadowsInAtlas` 整体降档、过小请求整灯剔除(按 `GetPunctualLightShadowSlicesCount` 回退)、`m_VisibleLightIndexToSortedShadowResolutionRequestsFirstSliceIndex` 反查表。Zircon 对应:`ShadowSlotRequest`/`allocate_frame`;取舍:URP 每帧全量重排,Zircon 加跨帧滞回(URP 无此机制,是其阴影闪烁来源之一)。
- `ShadowUtils.cs::GetShadowBias`:bias 以 shadow texel 世界尺寸为单位缩放(`texelSize = frustumSize / shadowResolution`),透视投影下随深度变化;`ApplySliceTransform` 将 slice 矩阵乘 atlas scale/bias。Zircon 对应:`LightShadowSettings.depth_bias/normal_bias` 取同语义,`ZrShadowSlot.atlas_scale_bias` 即 slice transform。
- `LightGridInjection.cpp`:`GetLightGridZParams`(`DepthDistributionScale = 4.05` 的非线性 z 切片)、`RWCulledLightLinks` 链表 + 紧凑化为 `CulledLightDataGrid`(16bit 元素)、`NumCulledLightsGridStride = 2`。取舍:UE 的 3D froxel + GPU 链表压缩需要两道 compute 与原子分配,Zircon 取 URP zbin+tile(2D tile x 1D zbin 近似 3D),无压缩 pass、storage buffer 一次写成,wgpu 友好;`PackRG16/PackRGB10` 的灯数据压缩暂不采用(96 B 直存,数百灯无带宽压力)。
- `ShadowSetup.cpp` + `DirectionalLightComponent.cpp`:`ComputeAccumulatedScale(Exponent, CascadeIndex, CascadeCount)` 指数级联分布(`GetEffectiveCascadeDistributionExponent` 无烘焙时固定 4)、`GetSplitDistance` 由累计比例插值 near/far、`CascadeSettings.SplitNearFadeRegion/SplitFarFadeRegion/FadePlaneOffset/FadePlaneLength` 过渡带、snapping 以 `FMath::Fmod` 在 light view 空间求 `SnapX/SnapY` 后回变换。Zircon 对应:`compute_cascade_splits`(取 log/linear lambda 混合式替代指数式)、`snapped_cascade_view_proj` 同构 snapping;fade 取单带简化(仅 far 端),UE 双带不引入。
- `VirtualShadowMapArray.cpp`:仅记录差距 —— VSM 为页表化 16k 虚拟分辨率 + 按需物理页分配 + 静态/动态缓存双层,依赖 GPU 页表 compute 生态;Zircon atlas 方案与其无共享 ABI,远期若引入按插件走,不在本计划范围。

## 风险与回退

- grid 构建 compute 在低端设备成本高:保留 LS-M1 的全量循环作为 capability 回落档(同一 buffer ABI,只是不分簇),不维护双 shader 语义。
- atlas 槽位抖动(灯光优先级频繁互换导致阴影闪烁):分配器加滞回(槽位保留若干帧);单测覆盖。
- VSM 不在本计划范围:仅在 LS-M3 文档中记录与 `VirtualShadowMapArray.cpp` 的差距,避免过度设计。
