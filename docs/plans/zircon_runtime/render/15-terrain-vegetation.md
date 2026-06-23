---
related_code:
  - zircon_runtime/src/core/framework/render/mesh/descriptor.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_model/mod.rs
  - zircon_runtime/src/graphics/visibility/mod.rs
  - zircon_plugins/rendering/plugin.toml
  - dev/UnrealEngine/Engine/Source/Runtime/Landscape/Private/LandscapeRender.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Landscape/Private/LandscapeCulling.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Landscape/Private/LandscapeGrass.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Landscape/Private/LandscapeGrassMapsBuilder.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Landscape/Private/LandscapeNaniteComponent.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Landscape/Private/LandscapeEdit.cpp
  - dev/Fyrox/fyrox-impl/src/scene/terrain/mod.rs
  - dev/Fyrox/fyrox-impl/src/scene/terrain/quadtree.rs
  - dev/Fyrox/fyrox-impl/src/scene/terrain/geometry.rs
  - dev/Fyrox/fyrox-impl/src/scene/terrain/brushstroke/mod.rs
  - dev/Fyrox/fyrox-impl/src/scene/terrain/brushstroke/strokechunks.rs
  - dev/godot/scene/resources/3d/height_map_shape_3d.cpp
  - dev/godot/scene/3d/multimesh_instance_3d.cpp
  - dev/learn-wgpu-zh/code/intermediate/tutorial13-terrain/src/terrain.rs
plan_sources:
  - .codex/plans/M5 Nanite-Like Virtual Geometry 全链收束计划.md
  - .codex/plans/ZirconEngine 独立插件补齐计划.md
---

# 计划 15:地形与植被(terrain / grass / tree)

## 目标

以插件为主体(新建 zircon_plugins/terrain 包族)建立地形与植被能力,runtime 只增中立接缝:

1. terrain:heightmap 地形(分块 component + 四叉树 LOD + 接缝处理),splat 权重多层材质混合(weight map 走 Texture2DArray,计划 13),洞(hole mask),与可见性/阴影/物理的对接。
2. 编辑面:高度/权重笔刷的数据修改契约(GPU 回写 + 增量法线重建),编辑器工具 UI 另立编辑器计划。
3. grass/细节层:权重图驱动的程序化散布 + GPU instancing 渲染(计划 03),按距离淡出与密度档。
4. tree:植被实例系统(手摆 + 散布),LOD 链 = 真实 mesh 级联(计划 10 LodGroup)+ billboard imposter 末级;风场动画(顶点 WGSL,SpeedTree 风格层级摆动参数);远期与 VG(Nanite-like)衔接走 `LandscapeNaniteComponent` 同型思路。

## 现状与差距

- 引擎无任何 terrain/植被能力;mesh/instancing/LOD/纹理数组等地基分别由计划 03/10/13 提供,本计划是其上层消费者,因此排在能力层最后段执行。

## 参考代码

| 文件 | 应重点阅读 |
|------|-----------|
| `dev/UnrealEngine/.../Landscape/Private/LandscapeRender.cpp` | 地形渲染核心:component 分块、LOD 选择(屏幕尺寸驱动)、邻接 LOD 接缝(顶点插值消缝) |
| `dev/UnrealEngine/.../Landscape/Private/LandscapeCulling.cpp` | 地形专用剔除与通用可见性的衔接方式 |
| `dev/UnrealEngine/.../Landscape/Private/LandscapeGrass.cpp` + `LandscapeGrassMapsBuilder.cpp` | grass map 生成(GPU 渲染权重导出)与按密度/距离实例化散布 |
| `dev/UnrealEngine/.../Landscape/Private/LandscapeEdit.cpp` | 高度/权重编辑的数据回写与增量更新边界(编辑契约样板) |
| `dev/UnrealEngine/.../Landscape/Private/LandscapeNaniteComponent.cpp` | 地形转 Nanite 表示(远期 VG 衔接,只读) |

次参考:`dev/godot`(scene/3d 中 GPUParticles 散布思路对照);SpeedTree 风格风动画以 URP/UE 的 SpeedTree 顶点风模型为概念参考(层级:主干弯曲 + 枝条摆动 + 叶片抖动,参数化进材质)。

**Rust/wgpu 落地参照(防凭空实现)**:

| 文件 | 对应本计划机制 | 应重点阅读 |
|------|---------------|-----------|
| `dev/Fyrox/fyrox-impl/src/scene/terrain/mod.rs` | TV-M1 数据面:Rust 引擎完整 heightmap 地形(`Chunk` 网格排布、`Layer` splat 层、`hole_mask`、高度读写) | `Chunk` 字段族与 `ChunkHeightData`/`ChunkHeightMutData` 读写、`replace_height_map`、`create_margin` 的边缘共享样本处理(对照本计划 components*255+1 语义) |
| `dev/Fyrox/fyrox-impl/src/scene/terrain/quadtree.rs` | TV-M1 四叉树 LOD:`QuadTree`/`QuadTreeNode::select` 自顶向下选层产出 `SelectedNode` | `select`/`aabb`(节点高度区间进剔除 AABB,同 `height_min_max`)、`height_mod_count` 脏版本号驱动重建 |
| `dev/Fyrox/fyrox-impl/src/scene/terrain/geometry.rs` | 共享 patch mesh:`TerrainGeometry::new(mesh_size)` 单网格全 chunk 复用 | 顶点/索引构建(对照本计划 8 级 patch mesh,差异:Fyrox 无顶点 morph 消缝) |
| `dev/Fyrox/fyrox-impl/src/scene/terrain/brushstroke/mod.rs` | TV-M2 编辑契约:`BrushStroke` 笔刷消息流 + `UndoData` 撤销载体 | `BrushSender::draw_pixel` 的像素消息化、`StrokeData` 累积被覆盖前值 → undo(对照 `TerrainEditOpResult::inverse`) |
| `dev/Fyrox/fyrox-impl/src/scene/terrain/brushstroke/strokechunks.rs` | 脏区按 chunk 聚集:`StrokeChunks` 记录每 chunk 修改像素集 | chunk_size 映射与按 chunk 应用/清空周期(对照 `dirty_sections` 合并) |
| `dev/godot/scene/resources/3d/height_map_shape_3d.cpp` | 高度场碰撞资产形态(`TerrainHeightFieldQuery` 物理消费侧的对照) | `map_width/map_depth` + heights 数组的资产表达与 min/max 高度缓存 |
| `dev/godot/scene/3d/multimesh_instance_3d.cpp` | 散布实例渲染的实例集挂接形态 | `MultiMesh` 资源 setter/节点接线(Zircon 对应物是 `GpuScene` span,无独立实例节点) |
| `dev/learn-wgpu-zh/code/intermediate/tutorial13-terrain/src/terrain.rs` | wgpu 地形 chunk 的端到端实绩:compute 生成 chunk 顶点 + 渲染管线 | `Terrain::gen_chunk`/`TerrainPipeline`(`gen_terrain_compute` entry)与逐 chunk 缓冲管理 |

`terrain 渲染器` bevy 无同类参照,以 Fyrox terrain + UE Landscape 双样板;`billboard imposter 八面体烘焙` 与 `SpeedTree 层级风动画` 无 Rust 同类参照,实现时以 UE 为唯一样板,按 index §8 第 8 条配对拍测试先行。

## 目标架构

归属:`zircon_plugins/terrain`(runtime + editor crate,按插件工作区规范);runtime 侧只在 `core/framework/render/` 增加地形 extract 段契约与高度场查询接口(物理/导航消费);渲染经计划 10 渲染器注册表 + 计划 02/03 命令与 instancing 管线,不开旁路。

核心设计:

- `TerrainAsset`:heightmap(R16)、weight maps(Texture2DArray)、层定义(每层材质引用 + tiling)、洞 mask、分块尺寸(如 64m component / 255 quad);
- `TerrainRenderer`(插件注册渲染器):component 四叉树 LOD(屏幕误差驱动),邻接级差 ≤1,接缝顶点向低级插值(UE 同型);几何为共享 patch mesh + 高度采样位移(GeometrySource 之一,计划 08 模板);材质 = splat 混合 shading(权重数组采样,层数档位 4/8)。
- 编辑契约:`TerrainEditOp`(高度/权重/洞的区域写)→ GPU 纹理回写 + 脏区法线/LOD 误差重建;undo 粒度按 op。
- `FoliageScatterLayer`:散布规则(密度图=权重层、坡度/高度过滤、随机种子)→ 实例集合烘焙(编辑期)或流式生成(运行期 chunk);渲染走 GPU instancing + 距离淡出(dither),进计划 04 剔除与计划 05 阴影。
- `ImposterBaker`:树 LOD 末级 billboard imposter(八面体多视角图集,编辑期烘焙);风动画 WGSL include(层级摆动参数进材质,SpeedTree 概念对齐)。

## 里程碑

### TV-M1 terrain 数据面与渲染器(插件骨架)

实施切片:
1. zircon_plugins/terrain 包族骨架(manifest/lifecycle 按插件规范);`TerrainAsset` 与导入。
2. patch mesh + 高度位移 GeometrySource;四叉树 LOD 与接缝;splat 混合材质(4 层档)。

测试阶段:
- `cargo check --manifest-path zircon_plugins/Cargo.toml -p <terrain runtime crate> --locked`;`cargo test` 同范围 + runtime `render_product` 地形场景
- 验收证据:LOD 切换无接缝裂缝(抓帧);四层混合正确;插件关闭时 runtime 无地形符号。

### TV-M2 编辑契约与增量更新

实施切片:
1. `TerrainEditOp` 与 GPU 回写、脏区重建;高度场查询接口(物理消费)。

测试阶段:
- 插件范围 `cargo test`(op 应用后高度查询/法线一致性断言)
- 验收证据:笔刷区域修改只重建脏 component(stats);undo 往返一致。

### TV-M3 grass 散布层

实施切片:
1. `FoliageScatterLayer` 规则与实例生成(chunk 流式);instancing 渲染 + 距离 dither 淡出;密度质量档。

测试阶段:
- 插件范围测试(散布确定性:同种子同结果)+ instancing stats
- 验收证据:百万草实例场景 draw 数受控(计划 03 合批 stats);移动相机 chunk 增删平滑。

### TV-M4 tree 与 imposter

实施切片:
1. 树实例(手摆经场景、散布经 scatter 层);LodGroup 链 + imposter 末级(八面体图集烘焙)。
2. 风动画 include 与材质参数。

测试阶段:
- 插件范围测试 + `render_product` 植被场景
- 验收证据:近-远过渡 mesh→imposter 无明显 pop;风动画分层摆动可调。

## 工程落地细化

本章是计划 15 的实施权威(见 index.md §8 第 7 条)。bind group 槽位、std430、`zr_` include 清单、queue 数值段、`sort_key` 归属、测试命名等全局约定直接引用 index.md §8,本章不重定义。跨计划契约名原样引用:`GpuScene` / `IndirectDrawBatcher`(计划 03)、`ViewVisibilityContext` / `HzbBuilder`(计划 04)、`GeometrySourceDescriptor` / `ShaderVariantKey`(计划 08)、`RendererCommon` / `LodGroup` / `RendererTypeDescriptor`(计划 10)、`TextureMetadata` / `Texture2DArrayAsset`(计划 13)。

### 模块与文件落点

`zircon_plugins/terrain` 包族骨架已存在(`plugin.toml` + `runtime`/`editor` 两 crate,plugin workspace members 已含 `terrain/runtime`、`terrain/editor`;crate 名 `zircon_plugin_terrain_runtime` / `zircon_plugin_terrain_editor`),本计划在其上扩展,不另建包。现有 stub 中的 `DiagnosticOnlyAssetImporter` 注册在 TV-M1 被真实导入后端取代(硬切换)。

manifest 字段(对照现有 `terrain/plugin.toml` 与 `particles/plugin.toml` 样板,逐项):

| 字段 | 现值/目标值 | 说明 |
|------|------------|------|
| `id` / `version` / `sdk_api_version` | `terrain` / `0.1.0` / `0.1.0` | 不变 |
| `display_name` / `category` / `maturity` | `Terrain` / `authoring` / `beta` | 不变 |
| `supported_targets` | `["client_runtime", "editor_host"]` | 不变 |
| `capabilities` | `["runtime.plugin.terrain"]` | 不变 |
| `default_packaging` | `["source_template", "library_embed"]` | 不变 |
| `[[capability_statuses]]` | `runtime.plugin.terrain` 维持 `status = "partial"`,TV-M4 完成后改 `"complete"` | 状态推进随里程碑 |
| `[[modules]]` | 既有 `terrain.runtime`(kind=runtime)与 `terrain.editor`(kind=editor)两条,字段 `name`/`kind`/`crate_name`/`target_modes`/`capabilities` 不变 | 不新增 module 条目 |
| `[[optional_features]]`(新增) | `id = "terrain.foliage"`(capabilities `runtime.feature.terrain.foliage`,primary 依赖 `terrain`/`runtime.plugin.terrain`)与 `id = "terrain.imposter_baking"`(capabilities `editor.feature.terrain.imposter_baking`) | 对照 particles 的 `[[optional_features]]` + `[[optional_features.dependencies]]` 写法;foliage/imposter 代码仍在 terrain 两 crate 内,feature 仅控制注册 |

runtime crate 布局(对照 `particles/runtime/src/{lib,module,asset,component,package,service}.rs` 形态;`Cargo.toml` 增加 `wgpu` 依赖与 `naga`(wgsl-in)dev-dependency,对照 particles):

| 路径(`zircon_plugins/terrain/runtime/src/`) | 内容 |
|------|------|
| `lib.rs` | thin:mod 声明 + curated re-export;现有 descriptor/manifest 函数迁入 `package.rs`(硬切换) |
| `package.rs` | `runtime_plugin_descriptor()` / `runtime_package_manifest()` / `plugin_registration()`(自 lib.rs 迁入) |
| `module.rs` | `TerrainModule`(EngineModule)+ `TerrainManager` 的 `ManagerDescriptor`(`StartupMode::Lazy`,对照 particles `module.rs` 的 `qualified_name`/`factory` 用法) |
| `component.rs` | 既有 `terrain.Component.Terrain` 迁入;新增 `terrain.Component.FoliageScatter` 描述符 |
| `service.rs` | `TerrainManager`:terrain 资产实例表、edit op 队列、height field 注册 |
| `asset/mod.rs` | thin |
| `asset/terrain_asset.rs` | `TerrainAsset` / `TerrainLayerDesc` / `TerrainHeightmap` / `TerrainHoleMask` |
| `asset/heightmap.rs` | R16 编解码、反量化、component 网格数学 |
| `asset/importer.rs` | 真实 heightfield importer(raw/r16/png → `TerrainAsset`),替换 `DiagnosticOnlyAssetImporter` |
| `lod/mod.rs` + `lod/quadtree.rs` | component 之上的四叉树(层级剔除用,叶 = component) |
| `lod/screen_error.rs` | `TerrainLodSettings` 屏幕比例表预计算 + 小数 LOD 计算 |
| `lod/neighbor_clamp.rs` | 邻接级差 ≤1 约束传播 |
| `geometry/patch_mesh.rs` | 共享 patch mesh 8 级(LOD 0..7)构建 |
| `geometry/geometry_source.rs` | `GeometrySourceDescriptor` 注册(`TerrainPatch` 几何源,计划 08) |
| `geometry/shaders/zr_terrain_patch.wgsl` | patch 位移 include(无 entry point) |
| `material/splat.rs` | 层定义 → group2 绑定;4/8 层档 `ShaderVariantKey` feature flag |
| `material/shaders/zr_terrain_splat.wgsl` | splat 混合 surface 片段 include |
| `edit/op.rs` | `TerrainEditOp` / `TerrainEditOpResult` |
| `edit/apply.rs` | CPU 权威副本写入、脏区合并、inverse op 生成 |
| `edit/gpu_writeback.rs` | `queue.write_texture` 脏 rect 回写 + 法线/LOD 误差重建 compute 调度 |
| `height_field/mod.rs` + `height_field/query.rs` | CPU R16 副本、`TerrainHeightFieldQuery` 实现与服务注册 |
| `foliage/scatter.rs` | `FoliageScatterLayer` 规则与确定性散布纯函数 |
| `foliage/chunk_stream.rs` | 相机环带 chunk 流式生成/增删与帧预算 |
| `foliage/render.rs` | foliage 实例 → `GpuScene` span 登记,instancing 经 `IndirectDrawBatcher` |
| `renderer.rs` | `TerrainRenderer`:`RendererTypeDescriptor` 注册 + RenderFeature descriptor(terrain pass / normal-rebuild compute 节点)+ pass processor 工厂 |

editor crate 布局(`zircon_plugins/terrain/editor/src/`,既有 authoring batch 与 `plugins://terrain/editor/authoring.zui` 模板保留):

| 路径 | 内容 |
|------|------|
| `lib.rs` | thin,既有注册保持;挂载新模块 |
| `brush.rs` | 笔刷几何 → `TerrainEditOp` 序列;undo 栈(逐 op inverse) |
| `foliage_authoring.rs` | 散布层参数编辑投影与重散布触发 |
| `imposter/mod.rs` + `imposter/baker.rs` | `ImposterBaker` 烘焙流程 |
| `imposter/octahedral.rs` | (半)八面体视角映射与 atlas 排布数学 |

runtime 接缝修改表(`zircon_runtime`,全部为中立接缝,framework 契约层无 wgpu):

| 路径 | 改动要点 |
|------|---------|
| `zircon_runtime/src/core/framework/render/terrain/mod.rs`(新增) | thin:mod 声明 + re-export |
| `zircon_runtime/src/core/framework/render/terrain/extract.rs`(新增) | `TerrainSectionSnapshot` 等 extract 段契约结构(serde、ABI 安全),经计划 10 `RendererTypeDescriptor` 的 extract 段 schema 挂入 `RenderFrameExtract`(`plugin_renderer_outputs.rs` 既有通道) |
| `zircon_runtime/src/core/framework/render/terrain/height_query.rs`(新增) | `TerrainHeightFieldQuery` trait + `TerrainHeightSample`(物理/导航消费的中立接口,经服务注册表暴露) |
| `zircon_runtime/src/core/framework/render/mod.rs` | 挂载 `terrain` 子模块(仅 mod 声明) |
| `zircon_runtime/src/core/framework/render/backend_types.rs` | `RenderStats` 增加 `last_terrain_sections_drawn`、`last_terrain_sections_dirty`、`last_foliage_instances`、`last_foliage_chunks_built` |
| `zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/zr_wind.wgsl`(新增) | 全局 include(index §8 清单内),层级风 struct 与函数,由计划 08 模板拼接消费;terrain 插件与未来粒子/布料共用 |

新增文件落点共 24 个(插件 runtime 18 + 插件 editor 4(不含既有 lib.rs)+ runtime 接缝 4 中的新文件 4;zr_wind.wgsl 计入 runtime 侧)。

### 核心类型与接口

framework 契约层(`core/framework/render/terrain/`,不得出现 wgpu):

```rust
// height_query.rs
pub struct TerrainHeightSample {
    pub height: f32,
    pub normal: [f32; 3],
    pub layer_weights: [f32; 8],   // 不足 8 层补 0
}
pub trait TerrainHeightFieldQuery: Send + Sync {
    /// 世界 XZ → 高度;hole 内返回 None(物理/导航以此判定不可通行)
    fn sample_height(&self, world_x: f32, world_z: f32) -> Option<f32>;
    fn sample(&self, world_x: f32, world_z: f32) -> Option<TerrainHeightSample>;
    fn world_bounds(&self) -> [f32; 6];   // min/max xyz
}

// extract.rs(经计划 10 extract 段 schema 注册)
pub struct TerrainSectionSnapshot {
    pub section_index: u32,        // 全地形线性 component 索引
    pub origin_world: [f32; 3],
    pub lod_value: f32,            // 小数 LOD(prepare 期每 view 计算后回填)
    pub neighbor_lod: [f32; 4],    // -X/+X/-Z/+Z 邻接 component 的 lod_value
    pub height_uv_rect: [f32; 4],  // heightmap 图集内 uv origin + scale
    pub height_min_max: [f32; 2],  // 该 component 高度区间(剔除 AABB 用)
}
```

插件 runtime crate(可用 wgpu):

```rust
// asset/terrain_asset.rs
pub struct TerrainAsset {
    pub component_quads: u32,      // 固定 255(2^8 - 1),component 顶点 256x256
    pub component_size_m: f32,     // 默认 64.0;quad = 64/255 ≈ 0.251 m
    pub components_x: u32,
    pub components_z: u32,
    pub height_min: f32,           // R16 反量化区间
    pub height_max: f32,
    pub heightmap: TerrainHeightmap,
    pub layers: Vec<TerrainLayerDesc>,        // 长度 4 或 8(层数档)
    pub weight_array: AssetRef,               // Texture2DArrayAsset(计划 13):4 层 = 1 slice RGBA8,8 层 = 2 slices
    pub hole_mask: Option<TerrainHoleMask>,
}
pub struct TerrainLayerDesc { pub name: String, pub material: AssetRef, pub tiling: [f32; 2] }
/// R16 字节布局:little-endian u16,行主序;逐 component 256x256 样本,
/// component 间共享边缘行列(全图有效样本 = components*255 + 1)
pub struct TerrainHeightmap { pub width: u32, pub height: u32, pub texels: Vec<u16> }
/// 每 quad 1 bit,行主序,逐 component 255x255 bit,按 u32 word 打包
pub struct TerrainHoleMask { pub words: Vec<u32> }
```

高度反量化:`world_h = height_min + (texel / 65535.0) * (height_max - height_min)`;component `(cx, cz)` 的样本 `(i, j)` 世界位置 = `origin + (cx*64 + i*64/255, world_h, cz*64 + j*64/255)`。

```rust
// lod/screen_error.rs(UE LODScreenRatioSquared 同型,见精读笔记)
pub struct TerrainLodSettings {
    pub lod0_screen_size: f32,         // 默认 1.0
    pub lod0_distribution: f32,        // LOD0→1 除数,默认 1.75,夹 > 1.01
    pub lod_distribution: f32,         // LOD1+ 除数,默认 2.0,夹 > 1.01
    pub max_lod: u8,                   // log2(256) - 1 = 7
    pub screen_ratio_squared: [f32; 8],
    pub mip_height_error: [f32; 8],    // LOD 间最大高度差(edit 后增量重算)
}
/// 阈值表区间内反插值得小数 LOD(整部 = 级,小数部 = morph 因子)
pub fn compute_fractional_lod(s: &TerrainLodSettings, screen_size_sq: f32) -> f32;

// lod/neighbor_clamp.rs:迭代松弛,仅向更细方向降级,O(sections * max_lod) 收敛
pub fn clamp_neighbor_lod(levels: &mut [u8], grid_w: u32, grid_h: u32);

// edit/op.rs
pub enum TerrainEditOp {
    WriteHeight { rect: TexelRect, texels: Vec<u16> },
    WriteWeight { layer: u8, rect: TexelRect, texels: Vec<u8> },
    WriteHole   { rect: QuadRect, words: Vec<u32> },
}
pub struct TerrainEditOpResult {
    pub inverse: TerrainEditOp,        // undo 粒度 = 按 op(存被覆盖前的 rect 数据)
    pub dirty_sections: Vec<u32>,
}

// foliage/scatter.rs
pub struct FoliageScatterLayer {
    pub seed: u64,
    pub density_per_m2: f32,
    pub density_weight_layer: Option<u8>,  // 取 weight array 某层作密度图
    pub jitter: f32,                       // 0..0.99(UE PlacementJitter 同义)
    pub slope_range: [f32; 2],
    pub height_range: [f32; 2],
    pub scale_range: [f32; 2],
    pub random_yaw: bool,
    pub align_to_surface: bool,
    pub renderable: AssetRef,              // mesh 或 LodGroup(计划 10)
    pub fade_start_m: f32,                 // 距离 dither 淡出(计划 08 变体 flag)
    pub fade_end_m: f32,
    pub chunk_size_m: f32,                 // 默认 32.0
}
/// 纯函数、确定性:同 (layer, chunk) 同结果;cell 级独立 hash,可并行
pub fn scatter_chunk(
    layer: &FoliageScatterLayer,
    chunk: (i32, i32),
    height: &dyn TerrainHeightFieldQuery,
) -> Vec<FoliageInstance>;
```

散布算法(确定性 jitter grid,UE FAsyncGrassBuilder 同型,差异见精读笔记):`sqrt_cells = ceil(chunk_size_m * sqrt(density_per_m2))`;cell `(x, y)` 取 `h = hash64(seed, chunk, y * sqrt_cells + x)`,从 h 派生 4 个 fraction:jitter_x/jitter_y(位置 = cell 中心 + (f*2-1) * clamp(jitter,0,0.99) * cell*0.5)、密度保留判定(`weight >= f_keep`,weight 采自密度层)、yaw/scale;再过 slope/height 区间过滤与 hole 排除;`align_to_surface` 时取高度场法线构造姿态。

editor crate:

```rust
// imposter/baker.rs
pub struct ImposterBakeSettings {
    pub frames_per_axis: u32,   // 默认 16(16x16 视角网格)
    pub atlas_size: u32,        // 默认 2048
    pub hemi_octahedron: bool,  // 树默认 true(只烘上半球)
}
pub struct ImposterBaker;
impl ImposterBaker {
    /// 步骤:LOD0 mesh → 逐(半)八面体网格视角正交渲染 albedo/normal/depth 三 MRT
    /// → 写入 atlas tile → 产出三张图集 + 视角网格元数据 → 生成 imposter 材质资产
    /// → 写回 LodGroup 末级(billboard 四边形 + imposter 材质)
    pub fn bake(&self, source: &AssetRef, s: &ImposterBakeSettings) -> ImposterBakeProduct;
}
```

### GPU 数据布局与 WGSL 约定

terrain pass 按 §8 槽位:group0 frame/view、group1 pass 级输入不变;group2 material 级绑定如下;group3 = `GpuScene` instance(计划 03,`@builtin(instance_index)` + indirect args first_instance;instance payload 槽存 patch record 索引):

| binding(group2) | 资源 | 说明 |
|------|------|------|
| b0 | uniform `TerrainMaterialParams` | 层 tiling x8、layer_count、height_min/max、component_size、flags;std140 小块 |
| b1 | texture_2d_array weight(rgba8unorm) | 4 层档 = slice0 RGBA;8 层档 = slice0+slice1(`Texture2DArrayAsset`) |
| b2 | sampler weight | linear, clamp |
| b3 | texture_2d_array albedo | 每层一 slice(计划 13 array 资产) |
| b4 | texture_2d_array normal | 同上 |
| b5 | sampler layer | aniso, repeat |
| b6 | texture_2d heightmap(r16unorm) | filterable 能力缺失时变体回落 `textureLoad` + 手工双线性 |
| b7 | sampler height | linear, clamp |
| b8 | storage<read> `terrain_patch_data: array<ZrTerrainPatch>` | 插件持有,prepare 期写 |
| b9 | storage<read> `section_lod: array<f32>` | 每 view 小数 LOD(UE SectionLODBiasBuffer 同型) |
| b10 | storage<read> `hole_bits: array<u32>` | 每 quad 1 bit;仅 `TERRAIN_HAS_HOLES` 变体绑定 |

`zr_terrain_patch.wgsl`(`GeometrySourceDescriptor` 的形变 include,计划 08 契约函数 `fetch_position` / `fetch_normal`):

```wgsl
struct ZrTerrainPatch {
    origin_xz: vec2<f32>,        // offset 0
    size_m: f32,                 // offset 8
    lod: f32,                    // offset 12(小数)
    neighbor_lod: vec4<f32>,     // offset 16(-X/+X/-Z/+Z)
    height_uv_origin: vec2<f32>, // offset 32
    height_uv_scale: vec2<f32>,  // offset 40
    section_index: u32,          // offset 48
    flags: u32,                  // offset 52(bit0 = has_holes)
    _pad: vec2<u32>,             // stride 64
}
fn zr_terrain_height(uv: vec2<f32>) -> f32;
fn zr_terrain_morph_uv(grid_uv: vec2<f32>, lod_value: f32, edge_lod: vec4<f32>) -> vec2<f32>;
fn fetch_position(v: ZrVertexInput, instance_index: u32) -> vec3<f32>;
fn fetch_normal(v: ZrVertexInput, instance_index: u32) -> vec3<f32>;
```

- patch 顶点输入只有归一化网格坐标(0..1),世界位置 = `origin_xz + grid * size_m`,高度采样自 b6。
- **接缝方案 = 顶点 morph(UE 同型),不做 index 退化**:`f = fract(lod_value)`;LOD L 网格中"在 L+1 消失的奇数顶点"把高度采样 uv 向其 L+1 父网格中点插值(`h = mix(h_self, h_parent_mid, f)`),位置 XZ 不动;component 边界顶点取 `lod_eff = max(lod_value, neighbor_lod[edge])` 做同样 morph。邻接整数级差经 CPU clamp ≤ 1,故边界两侧顶点高度严格一致,无 T 缝、无 pop。
- 法线从高度差重建(中心差分,边缘 clamp):`n = normalize(vec3(h(x-1,z) - h(x+1,z), 2 * texel_world_size, h(x,z-1) - h(x,z+1)))`。

`zr_terrain_splat.wgsl`:4 层档一次 `textureSample` weight slice0 得 vec4 权重;8 层档变体(`ShaderVariantKey` feature flag `TERRAIN_LAYERS_8`,走计划 08 变体位)再采 slice1;权重归一化后逐层 albedo/normal array 加权混合输出 `SurfaceOutput`。洞:`TERRAIN_HAS_HOLES` 变体在 fragment 按 quad 索引查 b10 并 `discard`(仍在 Geometry queue;取舍:不为洞建逐 component index buffer 变体,省内存与命令切分,代价是洞 component 失去部分 early-z,洞占比小可接受)。

`zr_wind.wgsl`(全局 include,index §8 清单;time 取 group0 frame uniform 既有字段,风参数进各材质 group2 uniform,不改 frame 布局):

```wgsl
struct ZrWindParams {
    dir_xz: vec2<f32>,        // offset 0,全局风向(单位向量)
    strength: f32,            // offset 8
    gust_freq: f32,           // offset 12,阵风频率
    trunk_stiffness: f32,     // offset 16,主干刚度(弯曲量反比)
    branch_amplitude: f32,    // offset 20
    branch_freq: f32,         // offset 24
    leaf_amplitude: f32,      // offset 28
    leaf_freq: f32,           // offset 32
    _pad: vec3<f32>,          // stride 48
}
fn zr_wind_trunk_bend(pos_os: vec3<f32>, height01: f32, p: ZrWindParams, t: f32) -> vec3<f32>;
fn zr_wind_branch_sway(pos_os: vec3<f32>, branch_phase: f32, p: ZrWindParams, t: f32) -> vec3<f32>;
fn zr_wind_leaf_flutter(pos_os: vec3<f32>, leaf_phase: f32, p: ZrWindParams, t: f32) -> vec3<f32>;
fn zr_apply_wind(pos_os: vec3<f32>, vertex_color: vec4<f32>, height01: f32,
                 p: ZrWindParams, t: f32) -> vec3<f32>;   // 三层叠加入口
```

层级约定(SpeedTree 概念对齐,无 `.st` 导入承诺):主干弯曲随 `height01` 二次方加权;枝条相位烘焙在顶点色 R、叶片相位在顶点色 G;草材质只用 trunk 层(`height01` = 草叶 v 坐标)。

GPU 回写约定:`queue.write_texture` 按脏 rect 提交,`bytes_per_row` 须对齐 `COPY_BYTES_PER_ROW_ALIGNMENT`(256B);R16 整行 256 texel = 512B 天然对齐,子 rect 由 `edit/gpu_writeback.rs` 的 `aligned_texel_rect()` 统一补宽(或逐行提交)收口。法线重建 compute(`terrain_normal_rebuild` 节点,经 RenderFeature descriptor 注册,有 graph 资源 IO 声明)只跑脏 rect + 1 texel 裙边;`mip_height_error` 表(LOD 间最大高度差,UE mip-to-mip max delta 同型)在 CPU 对脏 section 增量重算。

### 帧时序与集成点

对齐 `submit_frame_extract` 的 Extract → Prepare → Queue/Sort → Execute 时序:

1. **帧首(Extract 前)**:`TerrainManager` 取走累积的 `TerrainEditOp` 队列 → 应用到 CPU 权威副本(height/weight/hole)→ 产出脏 rect 与 `dirty_sections`;foliage chunk 流式:按相机环带 diff 计算待建/待删 chunk,投递 worker(预算默认 4 chunk/帧),收割已完成 chunk → `GpuScene` instance span 增删(走计划 03 update 路径)。
2. **Extract**:terrain extract 段(经 `RendererTypeDescriptor` schema)产出可见候选 `TerrainSectionSnapshot` 列表(component 四叉树自顶向下粗 frustum,节点 AABB 用 `height_min_max`);foliage 实例已常驻 `GpuScene`,不逐帧重 extract。
3. **Prepare**:每 view 对每 section 算小数 LOD(`compute_fractional_lod`,UE `ComputeSectionsLODForView` 同型)→ `clamp_neighbor_lod` → 写 b9 `section_lod` 与 b8 `terrain_patch_data`;HZB 遮挡(`ViewVisibilityContext` / `HzbBuilder`,计划 04)裁掉的 section 不入队;edit 脏区的 `queue.write_texture` 与 `terrain_normal_rebuild` compute 节点排在本帧 graph 前段、terrain geometry pass 之前(依赖由 graph 资源 IO 声明,不旁路)。
4. **Queue/Sort**:terrain patch draw 入 Geometry 段(RenderQueueValue = 2000,`sort_key` 走计划 09 布局);同 LOD 的 component 实例合入一个 indirect batch(`IndirectDrawBatcher`);foliage 剔除 compute 改写 indirect args word1 = instance_count(计划 03 已定);shadow pass 复用 patch 几何 + LOD bias(默认 +1)。
5. **Execute/跨帧**:undo 栈只存在于 editor crate;runtime 只见 op 流与 inverse op,无编辑会话状态。

### 实施切片细化

**TV-M1 切片 1(数据面)**:触碰 `terrain/plugin.toml`(optional_features)、runtime crate `Cargo.toml`(+wgpu/naga,对照 particles)、`lib.rs`→`package.rs` 拆分、`asset/*`、`component.rs`、`module.rs`、`service.rs`。要点:R16 编解码与 grid math、`Texture2DArrayAsset` 引用一致性校验、importer 实装并删除 `DiagnosticOnlyAssetImporter` 注册。完成判据:`terrain_asset_*` 测试绿;`cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_terrain_runtime --locked` 通过;既有 `terrain_runtime_plugin_contributes_component_and_importers` 测试随新 manifest 更新。

**TV-M1 切片 2(渲染面)**:触碰 `geometry/*`、`lod/*`、`material/*`、`renderer.rs`、两个插件 wgsl include;runtime 接缝 `core/framework/render/terrain/{mod,extract}.rs`、`backend_types.rs` stats 字段、`zr_wind.wgsl` 占位(struct + 空实现签名)。要点:8 级共享 patch mesh、`GeometrySourceDescriptor` 与 `RendererTypeDescriptor` 注册、4 层档 splat、小数 LOD + 顶点 morph + 邻接 clamp。完成判据:include 过 naga 验证;`render_product_terrain_splat_four_layers` 对拍;两邻接 component 强制 LOD 0/1 抓帧无裂缝;插件关闭时根 workspace 无 terrain 符号(`cargo check -p zircon_runtime --lib --locked` 不受影响)。

**TV-M2(单切片,编辑契约)**:触碰 `edit/*`、`height_field/*`;runtime 接缝 `terrain/height_query.rs` + 服务注册表暴露。要点:op → CPU 副本 → `write_texture` → normal rebuild compute → `mip_height_error` 增量;inverse op 生成。完成判据:`terrain_edit_*` 测试绿;`last_terrain_sections_dirty` stats 断言只含笔刷触及 section;undo 往返高度逐 texel 一致。

**TV-M3(单切片,grass)**:触碰 `foliage/{scatter,chunk_stream,render}.rs`、`component.rs`(FoliageScatter 组件);dither 淡出变体 flag 接计划 08。要点:确定性 hash 散布、预算化流式、`GpuScene` span 增删。完成判据:`foliage_scatter_*` 确定性测试;百万实例场景 draw 数受控(`IndirectDrawBatcher` 合批 stats 断言);连续移动相机 100 帧 chunk 建删不超预算(断言)。

**TV-M4 切片 1(tree/imposter)**:触碰 editor crate `imposter/*`、`LodGroup` 末级接线、手摆树实例经场景路径。完成判据:`foliage_imposter_octa_mapping_roundtrip` 测试;烘焙产物 `render_product` 对拍;近-远过渡抓帧序列无明显 pop。

**TV-M4 切片 2(风)**:`zr_wind.wgsl` 实装、树/草材质参数档。完成判据:wind include naga 验证;`render_product_foliage_wind` 对拍(固定 time 的确定性帧);三层参数独立调节在抓帧中可辨。

### 测试与验收清单

插件侧单测(`cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_terrain_runtime --locked`):

| 测试函数 | 断言 |
|------|------|
| `terrain_asset_r16_roundtrip_preserves_heights` | 编码→解码逐 texel 相等;反量化误差 ≤ (height_max-height_min)/65535 |
| `terrain_grid_math_maps_component_texel_to_world` | (cx,cz,i,j) → 世界坐标与公式一致;边缘共享样本两 component 同值 |
| `terrain_weight_array_layer_tiers_validate` | 4/8 层档 slice 数校验;9 层报错 |
| `terrain_hole_mask_word_packing_roundtrip` | quad bit 读写往返一致 |
| `terrain_lod_fractional_interpolates_between_thresholds` | 阈值点处整数、区间内单调小数 |
| `terrain_lod_neighbor_clamp_converges_to_delta_one` | 随机 lod 网格 clamp 后任意邻接差 ≤1 且只降不升 |
| `terrain_seam_edge_vertices_match_neighbor_grid` | CPU 复算 morph:边界顶点在 lod_eff 网格上的高度与邻 component 同点一致 |
| `terrain_edit_op_height_write_updates_query_and_dirty_sections` | op 后 `sample_height` 反映新值;dirty_sections 精确 |
| `terrain_edit_op_undo_roundtrip_restores_heights` | apply + inverse 后 CPU 副本逐字节一致 |
| `terrain_height_query_returns_none_in_hole` | hole quad 内 None,外 Some |
| `terrain_patch_include_validates_with_naga` | 拼接最小 entry 后 naga 验证通过(wgsl-in dev-dep) |
| `terrain_wind_include_validates_with_naga` | 同上(TV-M4) |
| `foliage_scatter_same_seed_same_instances` | 同 (layer, chunk) 两次调用逐实例 bit 级一致 |
| `foliage_scatter_density_filter_respects_weight_map` | 全 0 密度层产 0 实例;权重梯度下实例数单调 |
| `foliage_scatter_respects_slope_height_and_holes` | 区间外/hole 上无实例 |
| `foliage_chunk_stream_budget_respected` | 每帧新建 chunk ≤ 预算;离开环带的 chunk 被释放 |

editor crate(同 manifest `-p zircon_plugin_terrain_editor`):`terrain_brush_ops_compose_undo_stack`、`foliage_imposter_octa_mapping_roundtrip`(方向→tile→方向往返角误差阈值内)。

`render_product_*` 场景(由插件 runtime crate 集成测试承载,dev-dependencies 引 runtime 测试 harness;根 workspace 关闭插件时不得出现 terrain 符号):`render_product_terrain_splat_four_layers`、`render_product_terrain_lod_seam`、`render_product_terrain_hole_discard`、`render_product_foliage_instancing`、`render_product_foliage_wind`。

验收证据归档:LOD 连续切级抓帧序列与接缝特写(`ZR_RENDERDOC_CAPTURE_NEXT=1`)、百万草实例 stats(draw 数、`last_foliage_instances`)、编辑脏区 stats,按 milestone 记入 `docs/zircon_runtime/**` 模块文档。

## 状态与产出记录

| 日期 | 里程碑/切片 | 状态 | 产出 | 验证与证据 | 后续 |
|------|-------------|------|------|------------|------|
| 2026-06-23 | Render index 当前状态总览拆分 | 全部未启动 | 从 docs/plans/zircon_runtime/render/index.md 的第 9 节迁入本计划；本行保留 15 Terrain/Vegetation 的当前事实，render 总索引不再维护计划级明细。 | 文档重组；本次未改生产代码，render/index.md 只保留状态路由说明。 | 仍未完成：terrain renderer/plugin skeleton、editor delta、grass scatter、tree/imposter；验收缺口：需要等 03/04/08/10/13 地基稳定后进入实现和产品场景验收 |
| 2026-06-15 | TV-M1 terrain data plane and renderer plugin skeleton | 未启动: 等待 mesh/GPUScene/texture 地基稳定 | 当前引擎无 terrain renderer、heightfield asset、patch mesh 或 terrain material path。 | 本文件 `现状与差距` 明确无任何 terrain/植被能力。 | 在计划 03/10/13 基础完成后创建 terrain plugin skeleton 与 patch renderer。 |
| 2026-06-15 | TV-M2 editor contract and incremental updates | 未启动: 等待 TV-M1 | 无 terrain editing delta、height/weight map update 或 editor brush contract。 | 本文件将 terrain/vegetation 定位为上层消费者,依赖前置计划。 | 定义编辑器 brush operation、dirty tiles、undo/redo 和 runtime upload。 |
| 2026-06-15 | TV-M3 grass scatter layer | 未启动: 等待 GPU instancing/LOD/texture array | grass scatter、density map、wind animation 与 culling 均未实现。 | 计划 03/10/13 状态表显示 GPUScene/renderer/texture 地基仍有后续验收项。 | 基于 GPUScene instance batches 实现 grass scatter 与 visibility culling。 |
| 2026-06-15 | TV-M4 tree and imposter | 未启动: 等待 LOD Group 与 texture pipeline | tree billboard/imposter、LOD transition、wind 和 shadow integration 未实现。 | 计划 10 RF-M3、计划 13 TX-M3 均未完成。 | 等 LOD Group、array/cubemap/atlas 能力后再实施 tree/imposter。 |

### 参考实现精读笔记

以下符号均为实际读到的代码(`dev/UnrealEngine/Engine/Source/Runtime/Landscape/Private/`):

| 参考符号 | 要点 | Zircon 对应物 | 取舍 |
|------|------|------|------|
| `LandscapeRender.cpp` proxy 构造中的 `LODScreenRatioSquared` 预计算 | 以 `LOD0ScreenSize / CVarStaticMeshLODDistanceScale` 起步,先除 `LOD0Distribution` 再逐级除 `LODDistribution`(均夹 >1.01),逐级平方存表;并存 `LODSettings.LOD0ScreenSizeSquared` / `LOD1ScreenSizeSquared` / `LODOnePlusDistributionScalarSquared` / `LastLODScreenSizeSquared` | `TerrainLodSettings::screen_ratio_squared` 同型除数链 | 不引入 `r.StaticMeshLODDistanceScale` 抵消逻辑(Zircon 无该 CVar);除数链与夹值保留 |
| `FLandscapeRenderSystem::ComputeSectionsLODForView` + `FLandscapeSectionInfo::ComputeLODForView` | 每 view 计算并缓存逐 section 的小数 LOD(`TResourceArray<float>`,按 ViewStateKey 持久或按 view 临时) | prepare 期每 view 的 section 小数 LOD 计算 | 不做 per-view 持久 state map,按帧重算(section 数量级小,缓存复杂度不值) |
| `SectionLODBiasBuffer` / `SectionLODBiasSRV`(`RHICmdList.LockBuffer` 写 float 数组,顶点工厂按 section 索引取值) | 小数 LOD 经 GPU buffer 进顶点着色器驱动 morph | b9 `section_lod` storage buffer | 同型;Zircon 经 graph 声明的 prepare 写入,不裸锁 buffer |
| `LODSettings.VirtualShadowMapInvalidationLimitLOD` 的阈值反插值循环(在 `LODScreenRatioSquared[i]` 区间内线性求小数 LOD) | 屏幕比例平方 → 小数 LOD 的区间反插值即 UE 的"连续 LOD"数学 | `compute_fractional_lod` 的插值公式 | 直接采用 |
| `LandscapeGrass.cpp` `FGrassBuilderBase` / `FAsyncGrassBuilder`(ctor 携带 `SqrtSubsections`/`SubX`/`SubY`/`InHaltonBaseIndex`) | grass 按 component 细分子块异步构建;Halton 与 jitter grid 两条散布路径 | `scatter_chunk` worker 任务 | 只采 jitter grid 路径,Halton 分支不采(分布质量足够,实现简单) |
| jitter grid 路径:`MaxJitter1D = FMath::Clamp<float>(PlacementJitter, 0.0f, .99f) * Div * .5f`,每 cell 两次 `RandomStream.GetFraction()` 求偏移;保留条件 `Weight > AllowedDensityRange.Min && Weight <= Max && Weight >= RandomStream.GetFraction()`;`SampleLandscapeAtLocationLocal` 返回位置 + 权重 | 密度过滤 = 权重与随机数比较;jitter 上限 0.99 防越 cell | 散布算法的 jitter 公式与密度保留判定逐条同型 | `RandomStream` 是整 component 单流、顺序耦合;Zircon 改为 per-cell 独立 `hash64(seed, chunk, cell)`,换取 chunk 并行与增量重建的确定性 |
| `bAlignToSurface` 分支:用相邻 instance 位置 `(PosX1-PosX2) ^ (PosY1-PosY2)` 叉积求法线再构造对齐矩阵 | 贴地姿态来自局部位置差分 | `align_to_surface` 直接用 `TerrainHeightFieldQuery` 法线 | 高度场法线已有解析重建,不需邻居位置缓存 |
| `InstanceBuffer.AllocateInstances` + `SetInstance(InstanceIndex, OutXForm, RandomStream.GetFraction())` 烘焙输出 | 实例集合一次性写入 instance buffer | `Vec<FoliageInstance>` → `GpuScene` instance span(计划 03) | Zircon 无独立 HISM buffer,统一走 GpuScene |

## 风险与回退

- 本计划依赖面最广(03/04/08/10/13):启动条件 = 阶段 B 完成 + 计划 13 TX-M3 落地;在此之前不开工,避免私有旁路。
- speedtree 资产格式导入不做承诺:风模型参数化为通用材质参数,`.st` 导入器列为远期独立项。
- 地形物理(碰撞 heightfield)只暴露查询接口,物理对接归 Physics 计划。
