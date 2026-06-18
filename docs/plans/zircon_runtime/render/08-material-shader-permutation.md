---
related_code:
  - zircon_runtime/src/graphics/shader/mod.rs
  - zircon_runtime/src/core/framework/render/shader/definition_value.rs
  - zircon_runtime/src/core/framework/render/shader/pipeline_layout.rs
  - zircon_runtime/src/core/framework/render/material/standard_material.rs
  - zircon_runtime/src/core/framework/render/material/management.rs
  - zircon_runtime/src/graphics/material/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_cache.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/skinning.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_shader_source.rs
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/VertexFactory.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Public/MaterialShader.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Public/MeshMaterialShader.h
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/GlobalShader.h
  - dev/Graphics/Packages/com.unity.shadergraph
  - dev/bevy/crates/bevy_pbr/src/render/mesh.rs
  - dev/bevy/crates/bevy_mesh/src/lib.rs
  - dev/bevy/crates/bevy_material/src/specialize.rs
  - dev/bevy/crates/bevy_shader/src/shader.rs
  - dev/bevy/crates/bevy_shader/src/shader_cache.rs
  - dev/bevy/crates/bevy_pbr/src/render/skin.rs
  - dev/bevy/crates/bevy_pbr/src/render/morph.rs
  - dev/Fyrox/fyrox-material/src/shader/mod.rs
plan_sources:
  - .codex/plans/Rendering 插件选项补齐计划.md
  - .codex/plans/ZirconEngine 资产、Texture、模型、ZShaderZMaterialZMesh 缺口补齐计划.md
---

# 计划 08:材质、光照模型与 shader permutation 管理

## 目标

1. 建立 VertexFactory 等价物 `GeometrySource`:static / skinned / morph / instanced / VG 等几何来源与材质着色正交组合,GPU skinning/morph 对任意材质生效(不再只有 fallback shader)。
2. 建立光照模型(shading model)注册体系:`Unlit` / `BlinnPhong` / `StandardPbr` / 自定义模型,材质声明模型,deferred G-buffer 编码与 forward 着色按模型分发。
3. permutation 管理:(材质 shader, GeometrySource, pass 类型, feature 开关) → 变体键 → 编译缓存(内存 + 磁盘),离线预热工具,缺失变体诊断。
4. `mesh_pipeline_cache` 重构在统一变体键之上,与 shader graph 插件编译路径收敛。

## 现状与差距

- skinning 由 `fallback_mesh_shader_source` 拼接进 fallback WGSL,自定义材质(zshader/shader graph)无法获得 GPU 形变;morph 仅 CPU。
- shader 变体键(`definition_value.rs`)存在但维度不全(无几何源、pass 维度不显式);无磁盘缓存与预热,首次遇到变体即时编译造成卡顿。
- 光照模型单一:standard material 走固定 PBR 路径,无 unlit/blinn-phong/自定义模型位;deferred G-buffer 编码未按模型抽象。
- SRP bind group ABI(group 0 model / 1 material / 2 view)中 skinning 绑定硬编码在 mesh shader,几何源差异没有抽象层。

## 参考代码

| 文件 | 应重点阅读 |
|------|-----------|
| `dev/UnrealEngine/.../RenderCore/Public/VertexFactory.h` | 顶点工厂抽象:几何源声明顶点流/位置获取函数,与材质 shader 正交;permutation 维度如何进 shader map 键 |
| `dev/UnrealEngine/.../Renderer/Public/MaterialShader.h` + `MeshMaterialShader.h` | 材质 shader map:(material, vertex factory, shader type) 三维组合、permutation domain、lazy compile + DDC 缓存的层次 |
| `dev/UnrealEngine/.../RenderCore/Public/GlobalShader.h` | 非材质全局 shader(后处理/compute)的独立管理 —— 对应本引擎 post/compute WGSL 不进材质变体空间 |
| `dev/Graphics/Packages/com.unity.shadergraph` | shader graph → 模板代码生成:graph 输出函数 + 管线模板拼接(本引擎 zshader/shader_graph 插件的生成式参考);Unity 的 multi_compile/shader_feature 即变体开关先例 |

次参考:`dev/bevy/crates/bevy_pbr`(`MeshPipelineKey` 位打包变体键的 Rust 表达;`StandardMaterial` 的 shader defs 注入)。

**Rust/wgpu 落地参照(防凭空实现)**:

| 文件 | 对应本计划机制 | 应重点阅读 |
|------|---------------|-----------|
| `dev/bevy/crates/bevy_pbr/src/render/mesh.rs` | `ShaderVariantKey` 位打包与 pipeline specialization(重点) | `MeshPipelineKey: u64` bitflags 全集与 `MeshPipeline::specialize`(键 → shader defs → pipeline descriptor 的完整链);`ViewKeyCache` 的 per-view 键缓存 |
| `dev/bevy/crates/bevy_mesh/src/lib.rs` | 变体键维度分段 | `BaseMeshPipelineKey`:几何属性位段独立定义、高位保留给上层键合成(与 `packed_dims` 分段思路同构) |
| `dev/bevy/crates/bevy_material/src/specialize.rs` | 材质维度进变体键 | `ErasedMaterialPipelineKey` + specializer 函数指针表:材质类型擦除后统一走 `SpecializedMeshPipelines::specialize` |
| `dev/bevy/crates/bevy_shader/src/shader.rs` | defines header 注入 | `Shader.shader_defs`/`ShaderDefVal`(Bool/Int/UInt 值型 define,等价本计划 `RenderShaderDefinitionValue`) |
| `dev/bevy/crates/bevy_shader/src/shader_cache.rs` | 模板拼接器 + 变体缓存 | `naga_oil::compose::Composer` 的 include 组合与去重、`global_shader_defs`(平台 capability → define,即 `platform_token` 语义)、编译产物缓存键 |
| `dev/bevy/crates/bevy_pbr/src/render/skin.rs` | `SkinnedMesh` 几何源(storage palette) | `SkinUniforms::current_buffer/prev_buffer`:palette 升 storage buffer 与 prev palette 双缓冲(配 `skinning.wgsl` 的 uniform/storage 双形态) |
| `dev/bevy/crates/bevy_pbr/src/render/morph.rs` | `MorphedMesh` 几何源 | morph 权重双帧映射(`prev` 表)与 GPU 权重 buffer 布局(配 `morph.wgsl` 的 `prev_weight_at`) |
| `dev/Fyrox/fyrox-material/src/shader/mod.rs` | zshader 资产形态与 pass 集合 | `ShaderDefinition`:RON 数据驱动 shader(passes/properties/draw_parameters/disabled_passes),封闭 pass 枚举先例 |

`GeometrySource` × 材质 surface 函数的模板拼接(改名注入 `zr_material_surface`、pass 特化裁剪)无完整 Rust 同类参照(bevy 用 naga_oil import 组合而非 surface 函数模板),实现时以 UE `VertexFactory.h`/`MeshMaterialShader.h` 为唯一样板,按 index §8 第 8 条配对拍测试先行;磁盘变体缓存 + 离线预热同样无 Rust 同类参照(bevy 仅内存缓存),以 UE DDC 思路为样板。

## 目标架构

归属:`GeometrySource` 与变体键契约进 `core/framework/render/shader/`;WGSL 组装与缓存在 `graphics/shader/` 与 `graphics/scene/scene_renderer/mesh/`;shading model 注册在 `graphics/material/`。

核心设计:

- `GeometrySourceDescriptor`:声明顶点输入布局、形变阶段代码片段(WGSL include:`fetch_position/fetch_normal/...`)、所需绑定(skinning palette storage、morph 权重、instance index)。内建实现:`StaticMesh`、`SkinnedMesh`(palette 升 storage buffer,解除 256 uniform 限制)、`MorphedMesh`(GPU 权重混合)、组合型 `SkinnedMorphed`;VG 几何源由 virtual_geometry 插件注册。
- 材质 shader 模板:材质(手写 zshader 或 shader graph 产物)只提供 surface 函数(输入插值器,输出 `SurfaceOutput`);管线模板按 (GeometrySource, pass, shading model) 拼接最终 WGSL —— 自定义材质自动获得 skinning/morph/instancing。
- `ShadingModelDescriptor`:注册 forward 着色函数与 deferred G-buffer 编/解码函数;G-buffer 写 shading model id;内建 `Unlit`/`BlinnPhong`/`StandardPbr`,插件可注册自定义模型(描述符含所需 G-buffer 通道,超出当前布局时报诊断而非静默)。
- `ShaderVariantKey` 定稿:位打包 (material id+revision, geometry source, pass type, shading model, feature flags, quality);`mesh_pipeline_cache` 改为以此为键。
- `ShaderVariantCache`:内存 LRU + 磁盘缓存(键哈希 → 编译产物/naga 验证结果);`zircon_build.py` 增加预热步骤(枚举资产引用的变体离线编译);运行时 miss 记录进诊断(缺失变体报告)。

## 里程碑

### MS-M1 GeometrySource 与 WGSL 组装管线

实施切片:
1. `GeometrySourceDescriptor` 契约与四个内建实现;模板拼接器(surface 函数 + 几何源 include + pass 模板)。
2. 内建 standard material 切到模板路径;删除 fallback 拼接中被取代的 skinning 段。

测试阶段:
- `cargo check -p zircon_runtime --lib --locked`;`cargo test -p zircon_runtime shader --locked`(组装产物 naga 验证)+ `render_product` 回归
- 验收证据:同一材质在 static/skinned 两种几何源下产出不同变体且渲染正确。

### MS-M2 GPU skinning/morph 全材质化

实施切片:
1. palette 升 storage buffer;prev palette(供计划 06)同步;morph 权重 GPU 混合。
2. 自定义材质(zshader)走模板获得形变;CPU skinning 收缩为能力回落档。

测试阶段:
- `cargo test -p zircon_runtime skinning --locked` 与动画场景 `render_product`
- 验收证据:自定义材质角色 GPU 蒙皮正确(与 CPU 路径产物对拍);palette 数量超 256 骨骼用例通过。

### MS-M3 shading model 注册体系

实施切片:
1. `ShadingModelDescriptor` 与内建三模型;G-buffer 编码加 model id;deferred lighting 按 id 分发。
2. 材质资产增加 shading model 字段;blinn-phong/unlit 内建材质模板。

测试阶段:
- `cargo test -p zircon_runtime material --locked`(三模型 forward/deferred 产物对拍一致性)
- 验收证据:同场景三模型混用,deferred 与 forward+ 产物一致。

### MS-M4 变体缓存与预热

实施切片:
1. `ShaderVariantKey` 定稿与 `mesh_pipeline_cache` 重构;磁盘缓存。
2. `tools/zircon_build.py` 预热步骤;缺失变体诊断报告。

测试阶段:
- `cargo test -p zircon_runtime shader --locked`(键稳定性:同输入跨进程同键;缓存命中)
- 验收证据:二次启动同场景零运行时编译(诊断计数);预热产物被运行时命中。

## 工程落地细化

本章是本计划的实施权威(见 index.md §8 第 7 条)。bind group 槽位、GPU 数据布局、`zr_` include、测试命名等全局约定直接引用 index.md §8,不在此重定义;facade 固定 `zircon_runtime::core::framework::render`,契约层不出现 `wgpu` 类型,全部硬切换,渲染侧只消费 `RenderFrameExtract`。

### 模块与文件落点

新增(契约归 `core/framework/render`,实现归 `graphics`,不新增 crate):

| 落点 | 内容 |
|------|------|
| `zircon_runtime/src/core/framework/render/shader/geometry_source.rs` | `GeometrySourceId`、`GeometrySourceDescriptor` 契约(无 wgpu) |
| `zircon_runtime/src/core/framework/render/material/shading_model.rs` | `ShadingModelId`、`ShadingModelDescriptor`、`GBufferChannelMask` 契约 |
| `zircon_runtime/src/graphics/shader/geometry_sources/{mod,static_mesh,skinned_mesh,morphed_mesh,skinned_morphed}.rs` | 四个内建几何源描述符构造 + 注册表(插件注册入口,VG 经 virtual_geometry 插件走此口) |
| `zircon_runtime/src/graphics/shader/template/{mod,assemble,include_registry,pass_specialization}.rs` | 模板拼接器:include 注入去重、surface 函数改名、pass 特化裁剪、`ZrVertexInput` 生成 |
| `zircon_runtime/src/graphics/shader/variant_cache/{mod,resolve,disk,prewarm}.rs` | `ShaderVariantCache`:内存 interning + LRU、磁盘缓存、预热清单、缺失变体诊断 |
| `zircon_runtime/src/graphics/material/shading_models/{mod,registry,unlit,blinn_phong,standard_pbr}.rs` | shading model 注册表与三内建模型(含 G-buffer model id 编码) |
| `zircon_runtime/src/graphics/shader/wgsl/zr_geometry_{static,skinned,morphed,skinned_morphed}.wgsl` | 几何源 include(`fetch_*` 族),经 `include_str!` 内嵌 |
| `zircon_runtime/src/graphics/shader/wgsl/{zr_surface_types,zr_gbuffer_encode}.wgsl` | `ZrSurfaceOutput`/`ZrShadingContext` struct 与 G-buffer 编码骨架 |
| `zircon_runtime/src/graphics/shader/wgsl/zr_shading_{unlit,blinn_phong,standard_pbr}.wgsl` | shading include(`shade_forward`/`encode_gbuffer`/`shade_deferred`) |
| `zircon_runtime/src/graphics/shader/wgsl/zr_template_{forward,gbuffer,depth,shadow,velocity}.wgsl` | entry point 模板(`zr_vs_main`/`zr_fs_main`),含拼接占位符 |

修改:

| 落点 | 改动 |
|------|------|
| `zircon_runtime/src/graphics/shader/mod.rs` | 删除旧 `ShaderVariantKey { shader_id, domain, keywords }`;改为模块声明 + 再导出(保持瘦) |
| `zircon_runtime/src/core/framework/render/shader/{mod.rs,variant_key.rs}` | `ShaderVariantKey` 定稿(下节);既有 `RenderShaderVariantKey { entry_point, stage, defines }` 降级为单模块编译请求载体,由新键展开生成 |
| `zircon_runtime/src/core/framework/render/material/{mod.rs,lighting_model.rs,standard_material.rs}` | `RenderMaterialLightingModel::as_token()` → `ShadingModelId` 解析;`StandardMaterialDescriptor.lighting_model` 接入变体维度 |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/{mesh_pipeline_cache,ensure_pipeline,ensure_motion_vector_pipeline,new}.rs` | 以 `pipeline_variant_id` 为键重构;删除 `HashMap<String, ShaderModule>` 与 `PipelineKey` 双轨;motion vector 管线并入 `ShaderPassType::Velocity` 变体 |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/skinning.rs` | CPU `skin_model_primitive` 默认路径删除,收缩为能力回落档;palette 直传 storage 上传路径 |
| `zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_shader_source.rs` | zshader 校验从"整段 WGSL"改为"surface 函数 + 声明清单";fallback 材质走同一模板路径 |
| `tools/zircon_build.py` | 新增 `--prewarm-shaders` 步骤(见下) |
| `zircon_plugins`(shader_graph 生成器、virtual_geometry 注册点) | MS-M1 同步改产 surface 函数;VG 几何源注册(切片内硬切换) |

### 核心类型与接口

契约层(`core/framework/render`,可序列化、无 wgpu):

```rust
// shader/geometry_source.rs
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct GeometrySourceId(pub u8); // 0=StaticMesh 1=SkinnedMesh 2=MorphedMesh 3=SkinnedMorphed; 4..=15 插件注册

pub struct GeometrySourceDescriptor {
    pub id: GeometrySourceId,
    pub token: String,                      // 稳定名,进磁盘键;如 "static_mesh"
    pub wgsl_include: String,               // 提供 fetch_* 函数族的 include 全文
    pub vertex_attributes: Vec<RenderShaderVertexAttribute>, // 生成 ZrVertexInput
    pub object_bindings: Vec<RenderShaderBindingDescriptor>, // 仅允许 group3 槽内(binding 1..=4)
    pub supports_prev_position: bool,       // false 时 velocity pass 回退 instance prev transform
    pub defines: Vec<RenderShaderDefinitionValue>, // 注入拼接 header,等价 UE ModifyCompilationEnvironment
}

// material/shading_model.rs
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShadingModelId(pub u8);          // G-buffer 8bit 编码,上限 256;0=Unlit 1=BlinnPhong 2=StandardPbr;插件自 16 起
pub struct ShadingModelDescriptor {
    pub id: ShadingModelId,
    pub token: String,                      // 对齐 RenderMaterialLightingModel::as_token()
    pub forward_include: String,            // 提供 shade_forward
    pub gbuffer_encode_include: String,     // 提供 encode_gbuffer
    pub deferred_include: String,           // 提供 shade_deferred(解码后着色)
    pub required_channels: GBufferChannelMask, // 超出当前 G-buffer 布局 → 注册时诊断拒绝,非静默
}

// shader/variant_key.rs(定稿)
pub enum ShaderPassType { Forward, GBuffer, DepthPrepass, Shadow, Velocity } // 4bit,余位保留
pub struct ShaderFeatureBits(pub u32);
// bit0 ALPHA_TEST | bit1 RECEIVE_SHADOWS | bit2 DOUBLE_SIDED | bit3 LOD_DITHER_CROSSFADE(计划 10 占用)
// bit4 INSTANCED_PREV_TRANSFORM(计划 06) | bit5..=23 保留 | bit24..=31 插件段
pub struct ShaderVariantKey {
    pub material_shader: ResourceId,        // zshader/graph 产物资源
    pub material_revision: u64,             // resource_revision,热改材质即新键
    pub geometry_source: GeometrySourceId,
    pub shading_model: ShadingModelId,
    pub pass_type: ShaderPassType,
    pub features: ShaderFeatureBits,
    pub quality: ShaderQualityTier,         // Low/Medium/High/Ultra(2bit)
    pub platform_token: String,             // backend + downlevel caps 摘要,只进磁盘哈希
}
impl ShaderVariantKey {
    pub fn packed_dims(&self) -> u64;       // geometry 0..3 | shading 4..11 | pass 12..15 | features 16..47 | quality 48..49;材质维度不打包
    pub fn canonical_string(&self) -> String; // 全字段 + 模板修订号,磁盘哈希输入
}
```

实现层(`graphics/shader/variant_cache`,持 wgpu 对象):

```rust
pub struct ResolvedVariant {
    pub pipeline_variant_id: u32,           // cache interning 分配的稠密索引,写入 MeshDrawCommand(计划 02)
    pub module: wgpu::ShaderModule,
    pub layout: RenderShaderPipelineLayoutDescriptor, // 语义槽声明,重排后 replayer 映射收敛恒等(计划 02)
}
impl ShaderVariantCache {
    pub fn resolve(&mut self, key: &ShaderVariantKey) -> Result<ResolvedVariant, GraphicsError>;
    pub fn miss_report(&self) -> &VariantMissReport; // 帧诊断:本次会话即时编译清单
}
```

归属:`ShaderVariantKey`/两 descriptor 在 framework 契约层;拼接器、cache、注册表实例在 graphics 层并挂在 scene_renderer resources 上,生命周期与 `MeshPipelineCache` 现位置一致。

### GPU 数据布局与 WGSL 约定

bind group 槽位直接引用 index.md §8 第 1 条(group2=material、group3=object/instance),并落到具体 binding:

| group | binding | 资源 | 来源 |
|-------|---------|------|------|
| 0 | — | frame/view uniform(既有) | 模板引用,不重定义 |
| 1 | — | light grid / shadow map+sampler / HZB | 计划 05 的 `zr_light_grid.wgsl`/`zr_shadow.wgsl` 槽位;shadow 采样按计划 03 GS-M2 重排落 group1 |
| 2 | 0 | material uniform | 材质 property uniform(既有 `property_uniform.rs` 路径) |
| 2 | 1..N | 材质纹理/采样器对 | 按材质声明顺序 |
| 3 | 0 | `zr_instance_data`(storage, std430 AoS) | 计划 03 `zr_gpu_scene.wgsl` 提供 `get_instance_data`/`get_primitive_data`,经 per-draw `first_instance` 寻址 |
| 3 | 1 | `zr_joint_palette: array<mat4x4<f32>>`(storage) | Skinned 源;解除 256 uniform 限制 |
| 3 | 2 | `zr_prev_joint_palette`(storage) | 供计划 06 velocity;槽由 GpuScene 管理 |
| 3 | 3 | `zr_morph_deltas`(storage) | Morphed 源 target 增量 |
| 3 | 4 | `zr_morph_weights`(storage) | Morphed 源权重 |

几何源 include 契约 —— 每个 include 必须以固定签名提供下列函数(`ZrVertexInput` 由拼接器按 `vertex_attributes` 生成;沿用本计划"目标架构"既定的 `fetch_*` 命名,计划 15 等下游按此消费):

```wgsl
fn fetch_position(v: ZrVertexInput, instance_index: u32) -> vec3<f32>;      // object space,含形变
fn fetch_prev_position(v: ZrVertexInput, instance_index: u32) -> vec3<f32>; // 上一帧形变位置(计划 06 velocity)
fn fetch_normal(v: ZrVertexInput, instance_index: u32) -> vec3<f32>;
fn fetch_tangent(v: ZrVertexInput, instance_index: u32) -> vec4<f32>;
fn fetch_uv0(v: ZrVertexInput) -> vec2<f32>;
fn fetch_uv1(v: ZrVertexInput) -> vec2<f32>;
fn fetch_color(v: ZrVertexInput) -> vec4<f32>;
```

- Static:直读顶点属性;`fetch_prev_position` 返回当前位置(prev 差异由 instance prev transform 承担)。
- Skinned:palette 偏移取 `get_instance_data(instance_index).skinning_palette_offset`,prev 同理走 binding 2。
- Morphed:`fetch_position` 内做 GPU 权重混合;SkinnedMorphed 先 morph 后 skin,与 CPU 路径 `to_morphed_model_primitive` → `skin_model_primitive` 顺序一致以便对拍。
- 变体位:几何源本身即 `geometry_source` 维度,不再额外占 feature bit。

shading include 接口(`zr_surface_types.wgsl` 定义 struct,材质 surface 函数产出 `ZrSurfaceOutput`):

```wgsl
struct ZrSurfaceOutput {
    base_color: vec4<f32>, normal_ws: vec3<f32>,
    metallic: f32, roughness: f32, occlusion: f32,
    emissive: vec3<f32>, custom0: vec4<f32>,   // 自定义模型扩展通道
}
struct ZrShadingContext { position_ws: vec3<f32>, view_dir_ws: vec3<f32>, ndc: vec4<f32>, instance_index: u32 }
fn shade_forward(surface: ZrSurfaceOutput, ctx: ZrShadingContext) -> vec3<f32>;   // 消费 zr_light_grid/zr_shadow 查询函数(计划 05)
fn encode_gbuffer(surface: ZrSurfaceOutput, ctx: ZrShadingContext) -> ZrGBufferEncoded; // 写 8bit shading model id
fn shade_deferred(surface: ZrSurfaceOutput, ctx: ZrShadingContext) -> vec3<f32>;
```

拼接顺序(固定):defines header → `zr_surface_types.wgsl` → 几何源 include → `zr_gpu_scene.wgsl` → pass 专属 include(Forward:`zr_light_grid`+`zr_shadow`;GBuffer:`zr_gbuffer_encode`)→ shading include → 材质 surface 函数 → entry point 模板。include 按 id 注入一次去重;材质 surface 函数拼接时改名为 `zr_material_surface`,材质源内全局符号若撞 `zr_`/`fetch_`/`shade_` 前缀,naga 解析后诊断拒绝并回落 fallback 材质(报材质 uri)。pass 特化:DepthPrepass/Shadow 只保留 `fetch_position`(ALPHA_TEST 位再保留 `fetch_uv0`+base_color 采样),Velocity 保留 `fetch_position`+`fetch_prev_position` 双投影,均不拼 shading include。

### 帧时序与集成点

入口时序不变(Extract → Prepare → Queue/Sort → Execute):

1. Prepare 早段:`resource_streamer.ensure_shader_source` 产出 surface 函数源 + revision(沿用现 fallback 报告通道)。
2. Prepare 中段(计划 02 的 command build 处,即现 `build_mesh_draws`):每 draw 组装 `ShaderVariantKey`(几何源由 mesh 形变能力判定,shading model 由 `StandardMaterialDescriptor.lighting_model` 解析,pass 由所属 mesh pass 决定,features 由材质/组件旗标合成)→ `ShaderVariantCache::resolve` → `pipeline_variant_id` 写入 `MeshDrawCommand`。命中内存 interning 为 O(1);磁盘命中只做 naga 反序列化校验;全 miss 即时编译并计入 `VariantMissReport`。
3. Queue/Sort:不触碰 —— sort_key 位段归计划 09。
4. Execute:replayer 按语义槽映射绑定;本计划完成 WGSL 槽位重排后映射表收敛为恒等(计划 02 约定)。

硬切换删除项(随对应切片同变更删除,不留双路径):

- `fallback_mesh_shader_source` 中 skinning 拼接段(GS-M2 重排后的 fallback_mesh.wgsl 以模板路径产出)。
- `SkinnedMeshJointPaletteUniform` uniform palette 路径与 256 骨骼上限。
- `build/skinning.rs` 的 CPU 顶点蒙皮默认路径(保留为 storage buffer 能力缺失时的显式回落档,经能力检测进入)。
- `MeshPipelineCache` 的 `shader_modules: HashMap<String, _>` 与 `PipelineKey` 旧键、`motion_vector_mesh_pipelines` 独立表。
- `graphics/shader/mod.rs` 旧 `ShaderVariantKey { keywords }`。
- shader_graph 插件旧"整段 shader 产物"生成路径。

磁盘缓存:根目录默认 `<project>/.zircon-cache/shader_variants/`(`ZR_SHADER_CACHE_DIR` 覆写);布局 `<root>/v<schema_version>/<hash[0..2]>/<hash>.wgsl.zst` + 同名 `.meta`(键 canonical_string、模板修订号、naga/wgpu 版本、创建时间)。`hash = blake3(canonical_string + 全部参与 include 的内容哈希)`,宁可多失效不可错命中(与"风险与回退"口径一致)。并发写:临时文件 + 原子 rename,先到为准;读损坏视为 miss 并删除条目。

prewarm 钩子:`tools/zircon_build.py --prewarm-shaders` 在 stage 完成后,扫描 staged 资产清单中材质引用,生成变体枚举清单(材质 × 适用几何源 × 启用 pass × 默认 quality),调用 runtime 的 headless 入口 `zircon_runtime::dynamic_api::prewarm_shader_variants(manifest, cache_dir)` 离线编译,产物放入 staged payload 的 `ZirconEngine/cache/shader_variants/`,运行时缓存查找链为"运行期目录 → staged 预热目录"。

### 实施切片细化

MS-M1(GeometrySource 与拼接管线;切片期只 `cargo check -p zircon_runtime --lib --locked`):

- 切片 1:新增 `geometry_source.rs` 契约、`graphics/shader/geometry_sources/*` 四内建、`template/*` 拼接器、`zr_geometry_*.wgsl`/`zr_surface_types.wgsl`/entry 模板;`ShaderVariantKey` 结构与内存 interning(供计划 02 的 `pipeline_variant_id`)。完成判据:check 过;拼接产物 static/skinned 双变体均通过 naga 验证(模块内单测)。
- 切片 2:standard material 与 fallback 材质切到模板路径;改 `ensure_pipeline.rs` 消费 `ResolvedVariant`;删除 fallback skinning 拼接段与旧 `ShaderVariantKey`;shader_graph 插件生成器改产 surface 函数。完成判据:check 过;`render_product` 现有场景产物不回归(里程碑测试阶段验证)。

MS-M2(GPU 形变全材质化):

- 切片 1:group3 binding1/2 palette 升 storage(含 prev palette 槽对接 GpuScene)、morph deltas/weights 上 GPU;`build/skinning.rs` 上传路径改写。完成判据:check 过;>256 骨骼用例单测就位。
- 切片 2:zshader 自定义材质经模板获得形变;CPU skinning 改能力回落档并加进入日志。完成判据:`render_product` 蒙皮对拍(GPU vs CPU 参考)通过 —— 本切片是"GPU skinning 全材质可用"验收主线。

MS-M3(shading model 注册体系):

- 切片 1:`shading_model.rs` 契约、`shading_models/*` 三内建与注册表、`zr_shading_*.wgsl`、`zr_gbuffer_encode.wgsl` 写 8bit model id、deferred lighting 按 id 分发。完成判据:check 过;id 编解码 roundtrip 单测。
- 切片 2:`lighting_model.rs` token → `ShadingModelId` 解析接入变体键;blinn-phong/unlit 内建材质模板;自定义模型通道溢出诊断。完成判据:三模型 forward/deferred 对拍一致(里程碑测试)。

MS-M4(变体缓存与预热):

- 切片 1:`variant_cache/disk.rs` 磁盘缓存(布局/哈希/并发如上);`mesh_pipeline_cache` 以 `pipeline_variant_id` 重构,motion vector 管线并入 Velocity 变体;`VariantMissReport` 进帧诊断。完成判据:check 过;键稳定性单测(跨进程同键)。
- 切片 2:`variant_cache/prewarm.rs` + `dynamic_api::prewarm_shader_variants` + `zircon_build.py --prewarm-shaders`。完成判据:预热后二次启动同场景 miss 计数为 0。

### 测试与验收清单

单测放各 owner 模块 `#[cfg(test)]`(命名遵循 index.md §8 第 6 条),产物对拍走既有 `render_product_*` 设施:

| 测试函数 | 断言要点 | 位置 |
|---------|---------|------|
| `render_shader_variant_key_packs_dimensions_stably` | `packed_dims`/`canonical_string` 对 golden 值稳定;字段变更必须显式改 golden | `framework/render/shader/variant_key.rs` |
| `render_shader_template_assembles_static_and_skinned_variants` | 同一 surface 函数 × 两几何源产物不同且均过 naga | `graphics/shader/template/assemble.rs` |
| `render_shader_template_dedupes_includes_and_renames_surface` | include 注入恰一次;`zr_material_surface` 改名生效;前缀冲突材质被诊断拒绝 | 同上 |
| `render_shader_geometry_include_provides_fetch_contract` | 四内建源均解析出全部 `fetch_*` 签名(含 `fetch_prev_position`) | `graphics/shader/geometry_sources/mod.rs` |
| `render_shader_variant_cache_hits_disk_after_restart` | 写盘→新建 cache 实例→resolve 零编译;损坏条目按 miss 处理 | `graphics/shader/variant_cache/disk.rs` |
| `render_shader_variant_miss_report_counts_runtime_compiles` | miss 计数与即时编译次数一致 | `graphics/shader/variant_cache/resolve.rs` |
| `render_skinning_storage_palette_exceeds_256_bones` | 300 骨骼 palette 上传与寻址正确 | `scene_renderer/mesh/skinning` 测试模块 |
| `render_material_shading_model_id_roundtrips_gbuffer_encoding` | 0/1/2/自定义 id 编解码 roundtrip;255 上限 | `graphics/material/shading_models/registry.rs` |
| `render_material_custom_model_channel_overflow_reports_diagnostic` | `required_channels` 超布局 → 注册返回诊断而非静默 | 同上 |
| `render_material_lighting_model_token_resolves_shading_id` | `as_token()` 全枚举映射 + 未注册 custom 回落 Pbr 并诊断 | `framework/render/material/lighting_model.rs` |
| `render_product_skinned_custom_material_matches_cpu_reference` | zshader 材质 GPU 蒙皮 vs CPU 参考对拍(验收主线) | render_product 套件 |
| `render_product_three_shading_models_forward_deferred_parity` | 同场景三模型混用,deferred 与 forward+ 产物一致 | render_product 套件 |
| `render_product_morph_gpu_blend_matches_cpu_reference` | GPU morph 混合 vs CPU `to_morphed_model_primitive` 对拍 | render_product 套件 |

里程碑过滤词:`cargo test -p zircon_runtime shader --locked` / `material` / `skinning`;插件接缝按 index.md §7 跑 `zircon_plugins` 受影响包。

## 状态与产出记录

| 日期 | 里程碑/切片 | 状态 | 产出 | 验证与证据 | 后续 |
|------|-------------|------|------|------------|------|
| 2026-06-15 | MS-M1 GeometrySource and WGSL assembly | 部分完成: GPUScene include 与 group2/3 ABI 已硬切,通用 GeometrySource 管线未完成 | 计划 03 已让内建 forward/normal/shadow/deferred/PBR shader 拼接 `zr_gpu_scene.wgsl`,自定义测试 WGSL 也按 group2 material + group3 GPUScene 读取;但计划 08 的 GeometrySource registry、模板编译 owner 与磁盘缓存未落地。 | 计划 03 GS-M2 状态表记录 shader ABI hard cut、旧 model uniform/cache/replay 删除和 scoped `cargo check` 通过。 | 建立 shader template graph、GeometrySource key 和 shader source provenance dump。 |
| 2026-06-15 | MS-M2 GPU skinning/morph for all materials | 部分完成: GPUScene skinned palette 与 CPU-morphed previous source 已支撑内建路径 | 计划 03/06 已把 current/previous skinned palette、CPU-morphed morph-shape signature、changing-shape previous source velocity 接回 GPUScene/velocity path;但所有材质域统一 skin/morph 仍未完成。 | 计划 06 TP-M1/S23 状态表记录 previous source velocity tests 通过;计划 03 GS-M2 状态表记录 current/previous skinned palette group3 binding。 | 把 GPU skinning/morph 入口从 fallback shader 扩展到所有 shader permutation。 |
| 2026-06-16 | MS-M3-S1 shading model registry contract and deferred id path | 部分完成: 三个内置模型注册底座、运行时 `ShadingModelId` 与 deferred G-buffer id 通路已接入;完整三模型 parity 仍未完成 | 新增 `core/framework/render/material/shading_model.rs` 定义 `ShadingModelId`、`ShadingModelDescriptor`、`GBufferChannelMask` 和注册错误;新增 `graphics/material/shading_models/{mod,builtins,registry}.rs` 注册 `Unlit=0`、`BlinnPhong=1`、`StandardPbr=2`;`MaterialRuntime`/`PipelineKey` 携带 `shading_model_id`;标准材质 uniform 复用 `data8.y` 存 8-bit id 归一化值;deferred geometry 写 `gbuffer_material.a`,deferred lighting 解码并按 Unlit/BlinnPhong/StandardPbr 分发,其中 BlinnPhong 暂走 lit fallback。 | `rustfmt --edition 2021` touched Rust files 通过;`cargo check -q -p zircon_runtime --lib --target-dir D:\cargo-targets\zircon-runtime-shading-model-check-0616` 通过(仅既有 warning);新增源码/单元测试覆盖 id roundtrip、注册表重复/通道溢出、uniform packing、deferred encode/decode 源码合同。`cargo test -q -p zircon_runtime --lib shading_model --target-dir D:\cargo-targets\zircon-runtime-shading-model-check-0616` 在 lib-test 编译阶段被既有 `RenderBloomSettings { intensity: 0.6 }` 缺 `radius`/`threshold` 字段阻塞,未进入本切片测试。 | 下一步把 `lighting_model`/自定义模型诊断接入材质资产与 variant key,补 `zr_shading_*.wgsl`/`zr_gbuffer_encode.wgsl` 模板,实现 BlinnPhong 真正 deferred/forward parity 和三模型产品对拍。 |
| 2026-06-16 | MS-M3-S2 lighting-model asset/runtime diagnostics and stable variant dimension | 部分完成: 内置 lighting model 已解析为稳定 `ShadingModelId` 变体维度;未注册 custom 模型不再静默当 PBR ready | `PipelineKey` 移除重复的 `RenderMaterialLightingModel` 字段,改以 `shading_model_id` 作为当前材质模型变体维度;`MaterialRuntime`/capture seed 保留原始 lighting model 供调试和资产视图;`ResourceStreamer::ensure_material` 对 `custom:<name>` 但未注册的模型写入 `RenderMaterialValidationError::UnregisteredShadingModel` 与 Validation fallback,同时管线 key 使用 StandardPBR fallback id;新增/更新 render-product streamer 断言覆盖 PBR id=2、BlinnPhong id=1、自定义模型显式诊断。 | `rustfmt --edition 2021` touched Rust files 通过;`cargo check -q -p zircon_runtime --lib --target-dir D:\cargo-targets\zircon-runtime-shading-model-check-0616` 通过(仅既有 warning)。测试代码已补,但整包 lib-test 仍受上一行记录的既有 `RenderBloomSettings` 夹具缺字段阻塞,按用户指示测试延后。 | 下一步仍需 `zr_shading_*.wgsl`/`zr_gbuffer_encode.wgsl` 模板化、插件注册自定义 shading model 的公开入口,以及三模型产品/RenderDoc 对拍。 |
| 2026-06-16 | MS-M3-S3 builtin BlinnPhong forward/deferred shader branch | 部分完成: 内建 BlinnPhong 已从 StandardPBR fallback 拆成当前 forward fallback 与 deferred lighting 的真实分支;完整模板化与三模型产品 parity 仍未完成 | `fallback_mesh.wgsl` 从标准材质 uniform `data8.y` 解出 8-bit `shading_model_id`,unlit flag 继续强制走 `Unlit=0`,forward fallback 对 Unlit 直接输出 albedo+emissive,对 BlinnPhong 使用独立 diffuse/specular 函数,其他模型保留 StandardPBR 路径;`deferred_lighting.wgsl` 从 G-buffer material alpha 解码模型 id,把 id 传入 light-grid/shadow 光照函数,并用 `shade_deferred_lit` 分别调度 StandardPBR 与 BlinnPhong 的 diffuse/specular 规则;新增源码合同断言覆盖 forward/deferred 模型分发关键字符串。 | `rustfmt --edition 2021 zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline/fallback_mesh_shader_source.rs zircon_runtime/src/graphics/scene/scene_renderer/deferred/lighting_pipeline/tests.rs` 通过;`cargo check -q -p zircon_runtime --lib --target-dir D:\cargo-targets\zircon-runtime-shading-model-check-0616` 通过(仅既有 warning)。按用户"测试可以暂时延后"指令未运行新 WGSL 源码合同测试;整包 lib-test 仍需先修复既有 `RenderBloomSettings { intensity: 0.6 }` 缺 `radius`/`threshold` 夹具问题。 | 后续仍需把当前内建分支收敛进 `zr_shading_*.wgsl`/`zr_gbuffer_encode.wgsl` 模板体系,开放插件自定义 shading model 注册入口,补 forward/deferred 三模型产品对拍与 RenderDoc 验收。 |
| 2026-06-16 | MS-M4-S0 shader variant key contract foundation | 部分完成: Plan 08 目标 `ShaderVariantKey` 的 framework 契约已落地;mesh pipeline cache、磁盘缓存与预热仍未接入 | 新增 `core/framework/render/shader/geometry_source.rs` 定义内建 `GeometrySourceId` 段(`StaticMesh=0`,`SkinnedMesh=1`,`MorphedMesh=2`,`SkinnedMorphed=3`,插件自 4 起);扩展 `shader/variant_key.rs` 保留旧 `RenderShaderVariantKey` 的单模块编译请求职责,新增 `ShaderPassType`、`ShaderFeatureBits`、`ShaderQualityTier` 与完整 `ShaderVariantKey`;`packed_dims()` 固定位段为 geometry 0..3、shading 4..11、pass 12..15、features 16..47、quality 48..49,`canonical_string()` 产出含 material id/revision/platform 的磁盘缓存哈希输入;通过 `render::shader` 与 top-level render facade 再导出。 | `rustfmt --edition 2021 zircon_runtime/src/core/framework/render/shader/{geometry_source,variant_key,mod}.rs zircon_runtime/src/core/framework/render/mod.rs` 通过;`cargo check -q -p zircon_runtime --lib --target-dir D:\cargo-targets\zircon-runtime-shading-model-check-0616` 通过(仅既有 warning)。新增 key/geometry source 单测代码,但按用户"测试可以暂时延后"未运行 lib-test。 | 下一步把现有 `PipelineKey`/`mesh_pipeline_cache` 接到该 key,补磁盘缓存 `variant_cache/disk.rs`、miss report 和 `zircon_build.py --prewarm-shaders`。 |
| 2026-06-17 | MS-M4-S1a PipelineKey to ShaderVariantKey runtime bridge | 部分完成: 现有 WGPU mesh pipeline variant 已持有派生 `ShaderVariantKey`;磁盘缓存、预热与管线 map 主键切换仍未完成 | `PipelineKey::shader_variant_key(...)`/`shader_variant_key_for_geometry(...)` 现在从材质 shader id/revision、`shading_model_id`、pass type、geometry source 和 platform token 派生 neutral `ShaderVariantKey`;alpha-mask 与 double-sided 映射到 `ShaderFeatureBits`;`MeshPipelineVariantKey` 保留完整 `PipelineKey` 作为当前 WGPU render-pipeline state key,同时缓存派生 `ShaderVariantKey`,并把 Base/TAA reactive 映射 Forward、DepthPrepass 映射 DepthPrepass、ShadowDepth/ShadowDepthAlphaMask 映射 Shadow、Velocity 映射 Velocity。 | `rustfmt --edition 2021 zircon_runtime/src/graphics/scene/resources/pipeline/pipeline_key.rs zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/mesh_pipeline_variant_registry.rs` 通过;`cargo check -q -p zircon_runtime --lib --target-dir D:\cargo-targets\zircon-runtime-shading-model-check-0616` 通过(仅既有 warning)。新增派生 key 单测代码,但按用户"测试可以暂时延后"未运行 lib-test。 | 后续仍需把 `ShaderVariantKey` 升级为 mesh pipeline cache/disk cache/prewarm 的命中主键,补 variant miss report、shader provenance dump 和二次启动 miss=0 验收。 |
| 2026-06-17 | MS-M4-S1b shader variant disk cache and miss diagnostics | 部分完成: base mesh shader module 创建路径已接入 `ShaderVariantKey` 磁盘缓存与帧诊断;prewarm、Velocity/TAA 全路径和最终 pipeline map 主键切换仍未完成 | 新增 `graphics/shader/variant_cache/disk.rs` 以 `blake3(canonical_string + include/source hash)` 生成 shard key,按 `<root>/v1/<hash[0..2]>/<hash>.wgsl.zst` + `.meta` 写入 zstd WGSL 和 JSON metadata,支持 `ZR_SHADER_CACHE_DIR` 覆写、临时文件 + rename、损坏条目删除后 miss;新增 neutral `ShaderVariantMissReport` 并挂到 `RenderStats.last_shader_variant_miss_report`;runtime diagnostics 输出 `render.shader_variant.request_count/memory_hit_count/disk_hit_count/compile_miss_count/disk_write_count/disk_error_count`;`MeshPipelineCache::ensure_pipeline` 在 base mesh shader module 首次创建时读写磁盘缓存,同时保留完整 `PipelineKey` 作为当前 WGPU render-pipeline key。 | `rustfmt --edition 2021` touched shader/cache/render-stats/mesh-cache files 通过;`cargo check -q -p zircon_runtime --lib --target-dir D:\cargo-targets\zircon-runtime-shader-variant-cache-check-0617` 通过(仅既有 warning)。新增 disk hit/corrupt cleanup 与 miss-report 计数测试代码,但按用户"测试可以暂时延后"未运行 lib-test。 | 后续仍需把 Velocity/TAA reactive/deferred/template 变体都接入同一 cache,补 shader provenance dump、prewarm manifest/headless API、`zircon_build.py --prewarm-shaders`,以及二次启动同场景 compile miss=0 验收。 |
| 2026-06-17 | MS-M4-S1c shader prewarm manifest, CLI, and staged cache handoff | 部分完成: 离线预热入口、构建脚本参数和 packaged cache 查找链已落地;资产枚举、真实设备编译与二次启动 miss=0 验收仍未完成 | 新增 `ShaderVariantPrewarmManifest/Request/Report` 中立 DTO;新增 `graphics/shader/variant_cache/prewarm.rs` 通过同一磁盘缓存布局写入预热条目;新增 `dynamic_api::prewarm_shader_variants(...)` 与 `builtin_fallback_shader_prewarm_manifest()`;新增 `zircon_shader_prewarm` 命令行工具,支持 `--manifest` 与 `--builtin-fallback`,并输出 report JSON;`tools/zircon_build.py --prewarm-shaders` 在 runtime/editor assets staging 后运行该工具,把产物写入 `ZirconEngine/cache/shader_variants`;运行时 `ShaderVariantCacheDisk` 现在按"运行期写入目录 → staged 预热目录"查找。同步修复 ECS read-only query `single()` 的 `D: 'world` 生命周期约束以恢复当前 runtime 检查。 | `rustfmt --edition 2021` touched Rust files 通过;`cargo check -q -p zircon_runtime --lib --target-dir D:\cargo-targets\zircon-runtime-shader-prewarm-check-0617` 通过(仅既有 warning);`cargo check -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-prewarm-bin-check-0617` 通过(仅既有 warning);`python -m py_compile tools\zircon_build.py` 通过;`python tools\zircon_build.py --targets runtime --out D:\zircon-shader-prewarm-dry-run --mode debug --prewarm-shaders --dry-run` 通过。新增 prewarm disk-write 测试代码,但按用户"测试可以暂时延后"未运行 lib-test。 | 后续仍需扫描 staged 资产/材质生成完整 variant manifest,把 Velocity/TAA reactive/deferred/template 变体接入同一 prewarm/cache,对预热 WGSL 做真实 naga/wgpu 编译,补 shader provenance dump,并完成二次启动同场景 compile miss=0 产品验收。 |
| 2026-06-17 | MS-M4-S1d asset-scanned shader prewarm manifest | 部分完成: `zircon_shader_prewarm` 已能从资产根扫描 `.zmeta` compound shader、`.zshader` 与 standalone `.wgsl` 并写入磁盘 cache;完整多 pass/多几何源 manifest 与真实运行时 revision 仍未完成 | `zircon_shader_prewarm --asset-root <dir>` 现在会扫描资产根,对 compound `.zmeta` 读取 `uuid/url/source_hash` 并定位同名 shader 包目录;对 `.zshader` 按 `wgsl_files` 顺序合并 WGSL,无 `wgsl_files` 时按包内 `.wgsl` 排序;对独立 `.wgsl` 直接生成预热项;生成 static-mesh Forward/StandardPBR/Mid quality `ShaderVariantKey` 并复用现有 `variant_cache/prewarm.rs` 写盘。`tools/zircon_build.py --prewarm-shaders` 现在把 staged `ZirconEngine/assets` 作为 `--asset-root` 传给预热工具,同时保留 `--builtin-fallback` 兜底。 | `rustfmt --edition 2021 zircon_runtime/src/bin/zircon_shader_prewarm/{args,manifest,run}.rs` 通过;`cargo check -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617` 通过(仅既有 warning);`python -m py_compile tools\zircon_build.py` 通过;`python tools\zircon_build.py --targets runtime --out D:\zircon-shader-asset-prewarm-dry-run --mode debug --prewarm-shaders --dry-run` 确认命令含 `--asset-root ZirconEngine/assets`;`cargo run -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617 -- --project-root <temp> --cache-dir <temp>/cache --report <temp>/report.json --asset-root examples/vampire/assets/shaders --pretty` 写入 4/4 变体。 | 后续仍需把材质资产实例组合成 alpha/double-sided/geometry/pass 完整 manifest,把 Runtime `ResourceManager` revision 导出到 staged manifest,对预热 WGSL 做真实 naga/wgpu 编译,补 Velocity/TAA/deferred/template 统一 cache,并完成二次启动 miss=0 产品验收。 |
| 2026-06-17 | MS-M4-S1e asset-scanned multi-pass shader prewarm | 部分完成: asset-root 预热已从单 Forward 扩到按 `.zshader` entry point 推导 pass 集合;材质实例 feature、非静态几何源和真实 runtime revision 仍未完成 | `zircon_shader_prewarm --asset-root <dir>` 现在给 vertex+fragment `.zshader` 生成 Forward/GBuffer/DepthPrepass/Shadow/Velocity 五个 static-mesh StandardPBR 变体;vertex-only 生成 DepthPrepass/Shadow/Velocity;fragment-only 生成 Forward/GBuffer;compute-only 不进入材质变体预热;standalone `.wgsl` 因无 stage 元数据继续生成完整材质 pass 集合。更新 asset-root 单测预期覆盖 compound `.zshader` 五 pass 展开。 | `rustfmt --edition 2021 zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs` 通过;`cargo check -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617` 通过(仅既有 warning);`cargo run -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617 -- --project-root <temp> --cache-dir <temp>/cache --report <temp>/report.json --asset-root examples/vampire/assets/shaders --pretty` 写入 20/20 变体。 | 后续仍需从材质资产实例组合 alpha-test/double-sided/quality/非静态 geometry source/shading model 维度,把 Runtime `ResourceManager` revision 导出到 staged manifest,对预热 WGSL 做真实 naga/wgpu 编译,并完成二次启动 miss=0 产品验收。 |
| 2026-06-17 | MS-M4-S1f asset-scanned material feature prewarm manifest | 部分完成: asset-root 预热已能把 `.zmaterial` 实例的 alpha-test/double-sided feature 组合进同一 `ShaderVariantKey`;quality、非静态几何源、shading model 与真实 runtime revision 仍未完成 | `zircon_shader_prewarm --asset-root <dir>` 现在额外扫描 `.zmaterial`,通过 `MaterialAsset` 读取 shader 引用、`AlphaMode` 和 `double_sided`,再按 shader URL 或 resource id 关联到已扫描 shader source;`AlphaMode::Mask` 映射 `ShaderFeatureBits::ALPHA_TEST`,`double_sided = true` 映射 `ShaderFeatureBits::DOUBLE_SIDED`,并按 `ShaderVariantKey::canonical_string()` 去重,避免多个共享材质重复膨胀预热清单。更新 asset-root 单测夹具覆盖 mask + double-sided 材质生成 feature 变体。 | `rustfmt --edition 2021 --check zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs` 通过;`cargo check -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617` 通过(仅既有 warning);`cargo run -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617 -- --project-root <temp> --cache-dir <temp>/cache --report <temp>/report.json --asset-root examples/vampire/assets --pretty` 写入 40/40 变体;同命令扫 `examples/vampire/assets/shaders` 仍写入 20/20 变体。 | 后续仍需把 quality tier、非静态 geometry source、shading model、alpha-blend pass 过滤和 Runtime `ResourceManager` revision 导出接入 staged manifest,对预热 WGSL 做真实 naga/wgpu 编译,并完成二次启动 miss=0 产品验收。 |
| 2026-06-17 | MS-M4-S1g asset-scanned built-in shading-model prewarm dimension | 部分完成: `.zmaterial` 内置 lighting model 已进入 asset-root 预热 key;custom shading model 插件 id、quality、非静态几何源和真实 runtime revision 仍未完成 | material feature 预热路径现在调用 `MaterialAsset::lighting_model()` 与 `ShadingModelId::from_lighting_model()`,把 PBR/BlinnPhong/Unlit 分别写入 `ShaderVariantKey.shading_model`;custom 模型因缺 project registry 导出仍回退 StandardPBR,避免预热工具猜测插件 id。更新 asset-root 单测夹具让 mask + double-sided 材质声明 `lighting_model = "blinn_phong"` 并断言后半组变体使用 `SHADING_MODEL_ID_BLINN_PHONG`。 | `rustfmt --edition 2021 --check zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs` 通过;`cargo check -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617` 通过(仅既有 warning);`cargo run -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617 -- --project-root <temp> --cache-dir <temp>/cache --report <temp>/report.json --asset-root examples/vampire/assets --pretty` 写入 40/40 变体;扫 `examples/vampire/assets/shaders` 写入 20/20 变体。 | 后续仍需导出 custom shading-model registry/plugin ids、quality tier、非静态 geometry source、alpha-blend pass 过滤和 Runtime `ResourceManager` revision,再做真实 naga/wgpu 编译与二次启动 miss=0 产品验收。 |
| 2026-06-17 | MS-M4-S1h asset-scanned initial runtime revision alignment | 部分完成: asset-root 预热 key 的 `material_revision` 已对齐运行时初次 ready 资源 revision;编辑后递增 revision 的 registry 导出仍未完成 | 确认运行时 `PipelineKey.shader_revision` 来自 `ResourceStreamer::resource_revision()` 读取 `ResourceRecord.revision`,而 `ResourceManager::register_ready` 初次 ready 资源 revision 为 `1`;`zircon_shader_prewarm --asset-root` 现在将扫描出的 shader 预热请求 revision 固定为 `ASSET_SCAN_INITIAL_RESOURCE_REVISION = 1`,source/include content hash 继续参与磁盘 cache key payload,避免初始 staged cache 因 `.zmeta source_hash` 派生 revision 与运行时 key 错位。单测断言 asset-root 预热请求 revision 为初始运行时 revision。 | `rustfmt --edition 2021 --check zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs` 通过;`cargo check -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617` 通过(仅既有 warning);`cargo run -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617 -- --project-root <temp> --cache-dir <temp>/cache --report <temp>/report.json --asset-root examples/vampire/assets --pretty` 写入 40/40 变体。 | 后续仍需导出项目 registry 中编辑后递增的 `ResourceRecord.revision`,再补 quality tier、非静态 geometry source、custom shading model plugin id、alpha-blend pass 过滤、真实 naga/wgpu 编译与二次启动 miss=0 产品验收。 |
| 2026-06-17 | MS-M4-S1i asset-scanned alpha-blend pass filtering | 部分完成: `.zmaterial` 透明材质实例已按当前透明队列收敛为 Forward-only 预热;基础 shader 源仍保守生成完整 pass 集合 | `MaterialPrewarmSource` 现在携带 material pass filter,`AlphaMode::Blend` 经 `MaterialAsset` 解析后只保留 `ShaderPassType::Forward`,避免为透明材质实例写入当前运行时不会消费的 GBuffer/DepthPrepass/Shadow/Velocity 变体;asset-root 单测夹具新增透明 unlit + double-sided 材质,断言总清单从 base 五 pass + mask 五 pass 增加为仅一个 transparent Forward 变体。 | `rustfmt --edition 2021 --check zircon_runtime/src/bin/zircon_shader_prewarm/manifest.rs` 通过;`cargo check -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617` 通过(仅既有 warning);`cargo run -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-asset-prewarm-check-0617 -- --project-root <temp> --cache-dir <temp>/cache --report <temp>/report.json --asset-root examples/vampire/assets --pretty` 写入 40/40 变体。 | 后续仍需导出项目 registry 中编辑后递增的 `ResourceRecord.revision`,再补 quality tier、非静态 geometry source、custom shading model plugin id、真实 naga/wgpu 编译与二次启动 miss=0 产品验收;透明材质的半分辨率/独立透明路径优化留给计划 19。 |
| 2026-06-17 | MS-M4-S1j runtime shader-quality key wiring | 部分完成: 运行时 viewport shader quality 已进入 mesh variant `ShaderVariantKey`;预热枚举与最终 cache 主键仍未完全收口 | `RenderQualityProfile` 新增 `shader_quality` 与 `with_shader_quality(...)`;quality 从 `ViewportRecordState`、`FrameSubmissionContext`、`ViewportRenderFrame` 传入 `MeshPassBuildContext`,再由 `MeshPipelineVariantRegistry` 写入 `ShaderVariantKey.quality`;不同 quality tier 现在会得到不同 mesh variant id 与 sort-key material bits。`zircon_shader_prewarm` 仍只生成 Medium 静态请求,当前 WGPU render-pipeline map/disk-cache 最终 owner 仍未完全切到 quality-aware `ShaderVariantKey`。 | `rustfmt --edition 2021` touched runtime shader-quality files 通过;`cargo check -q -p zircon_runtime --lib --target-dir D:\cargo-targets\zircon-runtime-shader-quality-check-0617` 通过(仅既有 warning)。新增 `mesh_pipeline_variant_registry_separates_shader_quality_tiers` 测试代码,但按用户"测试可以暂时延后"未运行 lib-test。 | 后续仍需让 asset-root/build profile 枚举 quality tier,把最终 WGPU pipeline/disk-cache owner 全面切到 `ShaderVariantKey`,并与非静态 geometry/custom shading-model plugin id、真实 naga/wgpu 编译和二次启动 miss=0 验收一起收口。 |
| 2026-06-17 | MS-M4-S1k quality-tier prewarm enumeration | 部分完成: asset-root 与内置 fallback 自动预热已能按显式 quality tier 展开;真实运行时写盘探针超时,未完成 miss=0 验收 | `zircon_shader_prewarm` 新增可重复 `--quality-tier low|medium|high|ultra|all`,默认仍为 Medium;自动生成的 built-in fallback 与 asset-root manifest 会按所选 quality 展开,而外部 `--manifest` 保留文件内原始 key;`tools/zircon_build.py --prewarm-shaders` 新增 `--shader-quality-tier` 并转发到预热工具,支持高/超高等目标质量档的 staged cache 生成。 | `rustfmt --edition 2021 --check zircon_runtime/src/bin/zircon_shader_prewarm/{args,manifest,run}.rs` 通过;`python -m py_compile tools\zircon_build.py` 通过;`cargo check -q -p zircon_runtime --bin zircon_shader_prewarm --no-default-features --features target-server --target-dir D:\cargo-targets\zircon-runtime-shader-quality-prewarm-check-0617` 通过(仅既有 warning);`python tools\zircon_build.py --targets runtime --out D:\zircon-shader-quality-prewarm-dry-run --mode debug --prewarm-shaders --shader-quality-tier high --shader-quality-tier ultra --dry-run` 确认命令含 `--quality-tier high --quality-tier ultra`。实际 `cargo run ... --asset-root examples/vampire/assets --quality-tier high --quality-tier ultra` 在 build/run 阶段超时,未作为通过证据。 | 后续仍需补轻量 manifest-count 单测或完成真实工具运行,再把非静态 geometry source、custom shading-model plugin id、编辑后 revision 导出、最终 WGPU pipeline/disk-cache owner 与二次启动 miss=0 验收收口。 |
| 2026-06-17 | MS-M4-S1l base mesh quality-aware cache owner | 部分完成: Base mesh render-command 路径已用 registry `ShaderVariantKey` 做 WGPU shader module、render pipeline 与磁盘 cache 命中主键;Velocity/TAA/deferred/template 尚未迁移 | `MeshPipelineCache` 新增 `mesh_variant_pipelines: HashMap<MeshPipelineVariantId, RenderPipeline>` 并移除旧 Base `PipelineKey` map/direct `ensure_pipeline`;`ensure_pipeline_for_variant` 现在从 `MeshPipelineVariantRegistry` 取 `(kind, PipelineKey, ShaderVariantKey)`,只处理 Base pass,用 `MeshPipelineVariantId` 缓存 render pipeline,用 `material_shader@revision#canonical_shader_variant_key` 缓存 shader module,并把同一个 quality-aware `ShaderVariantKey` 传入 `ShaderVariantCacheDiskKey`。这修正了 S1j 后 registry 已区分 quality、但实际 disk-cache lookup 仍会回退默认 Medium 的断点。 | `rustfmt --edition 2021 zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache/{mesh_pipeline_cache,new,ensure_pipeline}.rs` 通过;`cargo check -q -p zircon_runtime --lib --target-dir D:\cargo-targets\zircon-runtime-shader-quality-cache-check-0617` 通过(仅既有 warning)。按用户"测试可以暂时延后"未运行 lib-test/产品对拍。 | 后续仍需把 Velocity/TAA reactive/deferred/template pass 的 shader/pipeline cache owner 迁到同一 `ShaderVariantKey` 路径,再补非静态 geometry source、custom shading-model plugin id、编辑后 revision 导出、真实工具运行与二次启动 miss=0 验收。 |
| 2026-06-15 | MS-M4 shader variant cache and warmup | 未启动: 仍缺磁盘缓存与预热 | `definition_value.rs` 等变体基础存在,但无几何源/pass 维度完整 key、无磁盘缓存、无预热队列。 | 本文件 `现状与差距` 明确首次遇到变体即时编译造成卡顿。 | 与计划 17 PF-M4 联动建立 compile cache、warmup manifest 与回归指标。 |

### 参考实现精读笔记

仅记录本次实际读到的符号:

- `dev/UnrealEngine/.../RenderCore/Public/VertexFactory.h:313` `FVertexFactoryType`:以 `Name + ShaderFilename + EVertexFactoryFlags` 加一组函数指针(`ShouldCacheType`、`ModifyCompilationEnvironmentType`、`GetPSOPrecacheVertexFetchElementsType`、`ConstructParametersType`)构成注册项,经静态 `GetTypeList()` 链表全局注册。Zircon 对应:`GeometrySourceDescriptor` 把 `ShaderFilename` 数据化为 `wgsl_include` 字符串、把 `ModifyCompilationEnvironment` 收敛为静态 `defines` 列表、把 `ShouldCompilePermutation` 收敛为预热清单枚举时的适用性过滤。取舍:不复刻 UE 的 per-frequency `FVertexFactoryShaderParameters` 动态绑定对象 —— Zircon 绑定固定走 group3 槽表,描述符只声明 binding,不携带绑定逻辑。
- `VertexFactory.h:144` `EVertexFactoryFlags::SupportsPrimitiveIdStream`:UE 按 VF 开关 primitive id 流;Zircon 不需要该开关 —— 计划 03 已定全部 draw 经 `first_instance` ABI + `get_instance_data` 取 GPUScene 数据,几何源一律可拿 `instance_index`。
- `dev/UnrealEngine/.../Renderer/Public/MeshMaterialShader.h:32` `FMeshMaterialShaderPermutationParameters`(`Platform + MaterialParameters + VertexFactoryType + PermutationId + Flags`)与 `:43` `FVertexFactoryShaderPermutationParameters`(额外含 `ShaderType`):UE 变体空间是 (material, vertex factory, shader type, permutation id, platform) 五维。Zircon `ShaderVariantKey` 一一对应为 (material_shader+revision, geometry_source, pass_type, features, platform_token);取舍:UE 开放的 `FShaderType` 集合收敛为封闭 `ShaderPassType` 枚举 —— 本引擎 pass 形态由模板特化定义,不开放任意 shader type 注册,自定义着色经 `ShadingModelDescriptor` 而非新 shader type。
- `MeshMaterialShader.h:67` `FMeshMaterialShader : FMaterialShader`:UE 用类继承分层(global/material/mesh-material)隔离变体空间;Zircon 对应做法是 post/compute WGSL 不进 `ShaderVariantKey` 空间(沿既有 pipeline/post 路径),与计划正文"GlobalShader 独立管理"的口径一致。

## 风险与回退

- 模板拼接与现有 zshader 编译路径冲突:shader_graph 插件的生成器在 MS-M1 同步改为产出 surface 函数,硬切换,不保留旧整段 shader 产物路径。
- shading model 进 G-buffer 占用通道:V1 用 8bit id + 共享通道布局,自定义模型受布局约束并由诊断兜底,不做动态 G-buffer 布局。
- 磁盘缓存跨版本失效:键中含 shader 模板修订号与 wgpu/naga 版本,宁可多失效不可错命中。
