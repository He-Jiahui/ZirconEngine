---
related_code:
  - zircon_runtime/src/core/framework/render/sprite/sprite.rs
  - zircon_runtime/src/core/framework/render/sprite/atlas.rs
  - zircon_runtime/src/core/framework/render/sprite/extract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/sprite_renderer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/sprite/prepared_batches.rs
  - zircon_runtime/src/core/framework/render/core_pipeline/phase_sort.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/2D/Renderer2DData.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/2D/Rendergraph/Renderer2DRendergraph.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/2D/Rendergraph/DrawRenderer2DPass.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/2D/Lights/Light2D.cs
  - dev/godot/scene/2d/tile_map_layer.cpp
  - dev/godot/scene/2d/tile_map_layer.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/FontCache.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/FontCacheHarfBuzz.cpp
  - dev/bevy/crates/bevy_text/src/pipeline.rs
  - dev/bevy/crates/bevy_text/src/font_atlas.rs
  - dev/bevy/crates/bevy_sprite/src/texture_slice/slicer.rs
  - dev/bevy/crates/bevy_sprite_render/src/texture_slice/computed_slices.rs
  - dev/bevy/crates/bevy_sprite_render/src/tilemap_chunk/mod.rs
plan_sources:
  - .codex/plans/UI SDF 字体真实 Bake 收束计划.md
  - .codex/plans/ZirconEngine Bevy-Level Rendering Completion Plan.md
---

# 计划 14:2D 渲染栈(文本 / 图像 / 九宫 / tilemap / 图集)

## 目标

把 2D 从"sprite 批渲染"补成完整 2D 栈:

1. 场景文本渲染与排版:world/screen 双空间文本渲染器;排版(换行、对齐、富文本最小集、双向/复杂文种经 shaping 库);SDF 字体(承接既有 SDF bake 计划)与位图字体双路径;字形图集动态分配。
2. 图像渲染:`ImageRenderer`(独立于 sprite 的 UI 风格九宫/平铺/填充模式)与 sprite 的九宫切片(nine-slice)、边框拉伸。
3. tilemap:`TilemapRenderer` 支持矩形/六边形/等距三种网格;tile set 资产(图集化、tile 属性、地形过渡);画笔(brush)契约面向编辑器(矩形/线/填充/随机/规则画笔),runtime 只消费 chunk 数据;chunk 化网格生成 + 按 chunk 剔除与增量重建。
4. 排序:sorting layer / order in layer / y-sort(等距必需)接入计划 09 统一排序键;2D 与 3D 混排正确。
5. 2D 灯光(URP Light2D 风格)列为远期占位,不在本计划里程碑内。

## 现状与差距

- sprite 渲染器与图集管理(`atlas.rs`)可用,prepared_batches 有批组织;但无文本场景渲染器(文本仅 UI 内部)、无九宫、无 tilemap、无 y-sort。
- UI 的文本/排版能力(SDF bake 计划)在 UI 模块内,场景 2D 不能直接复用,需要把 shaping/atlas 下沉为共享服务。

## 参考代码

| 文件 | 应重点阅读 |
|------|-----------|
| `dev/godot/scene/2d/tile_map_layer.cpp/.h` | tilemap 的权威实现:chunk(quadrant)组织、脏 quadrant 增量重建、六边形/等距坐标系换算、y-sort 与 layer 交互 —— 本计划 tilemap 数据面与重建策略的主样板 |
| `dev/Graphics/.../Runtime/2D/Rendergraph/Renderer2DRendergraph.cs` + `DrawRenderer2DPass.cs` | URP 2D renderer 的 pass 组织:按 sorting layer 分批、与 light2d 纹理的交互(远期) |
| `dev/UnrealEngine/.../SlateCore/Private/Fonts/FontCache.cpp` + `FontCacheHarfBuzz.cpp` | 字形缓存与 HarfBuzz shaping 集成:shaped glyph run 缓存键、图集页管理 |
| `dev/bevy/crates/bevy_text/src/pipeline.rs` + `font_atlas.rs` | Rust 文本管线:cosmic-text/parley 风格 shaping → 字形图集 → mesh 生成的完整链路,落地首选参照 |

次参考:`dev/slint/internal/renderers/femtovg/font_cache.rs`(轻量字体缓存);既有 `.codex/plans/UI SDF 字体真实 Bake 收束计划.md` 的 SDF 产物格式。

**Rust/wgpu 落地参照(防凭空实现)**:

| 文件 | 对应本计划机制 | 应重点阅读 |
|------|---------------|-----------|
| `dev/bevy/crates/bevy_sprite/src/texture_slice/slicer.rs` | TD-M2 nine-slice:`TextureSlicer` 的 corner/side/center 切片与 `SliceScaleMode::{Stretch,Tile}` | `corner_slices` 的角尺寸钳制(对照本计划断点表的 k 系数)、tile 模式按 stretch 比例重复切分 |
| `dev/bevy/crates/bevy_sprite_render/src/texture_slice/computed_slices.rs` | nine-slice 切片到渲染侧 extract 快照的投影 | `compute_sprite_slices`(Sliced/Tiled 双模式分流)与 `extract_slices` 的 flip/anchor 偏移处理 |
| `dev/bevy/crates/bevy_sprite_render/src/tilemap_chunk/mod.rs` | TD-M3 tilemap chunk 渲染的 Rust 实绩:chunk mesh 缓存 + tile 数据驱动重建 | `TilemapChunkMeshCache`(同尺寸 chunk 共享 mesh)、`TileData`/`TilemapChunkTileData` 变更触发的重建路径 |

## 目标架构

归属:文本 shaping/字形图集下沉为 `graphics/text/` 共享服务(UI 与场景 2D 共用,UI 既有路径切换为消费方);tilemap/图像渲染器经计划 10 注册表;契约在 `core/framework/render/` 下新增 `text/` 与 `tilemap/`。

核心设计:

- `TextShapingService`:字体资产 → shaping(unicode 分段 + 换行 + 对齐)→ `ShapedGlyphRun`(缓存键:字体+字号+文本 hash);字形图集(动态分配页,SDF 与 alpha 位图双格式);场景 `TextRenderer` 组件输出 glyph quad 批(走 sprite 批管线)。
- `ImageRenderer` 与九宫:nine-slice 切片参数进 sprite/image 资产;顶点生成 9 子矩形(中段平铺/拉伸模式);UI 与场景共用切片算法函数。
- `TilemapAsset`:tile set(图集引用 + tile 定义 + 碰撞/自定义属性槽)+ 网格类型(Rect | Hex(pointy/flat) | Isometric);`TilemapRenderer`:chunk(如 32x32)→ 静态网格生成(顶点烘焙 UV)→ 脏 chunk 增量重建;chunk AABB 进计划 04 剔除;画笔操作契约(编辑器调用的 set_tiles 批接口 + 规则画笔在编辑器侧)。
- 排序:sorting layer/order in layer 字段在 `RendererCommon`(计划 10)上生效;等距 tilemap 与 sprite 启用 y-sort(同 layer 内按 y 排序,sort_key 低位);transparent 2D 批切分遵守排序边界(prepared_batches 升级)。

## 里程碑

### TD-M1 文本服务下沉与场景文本渲染器

实施切片:
1. `graphics/text/` 服务(shaping/换行/对齐/字形图集);SDF 产物接入;UI 文本路径切换为消费方(硬切换)。
2. `TextRenderer` 组件(world/screen)经渲染器注册表接入。

测试阶段:
- `cargo check -p zircon_runtime --lib --locked`;`cargo test -p zircon_runtime text --locked` + UI 文本回归
- 验收证据:中英混排换行/对齐用例;UI 既有文本测试不回退;场景内 3D 空间文本渲染正确。

### TD-M2 九宫与图像渲染器

实施切片:
1. nine-slice 参数与顶点生成(拉伸/平铺两模式);`ImageRenderer` 填充模式(fill amount,径向/横向)。

测试阶段:
- `cargo test -p zircon_runtime sprite --locked`(九宫顶点 UV 断言)
- 验收证据:任意尺寸下九宫边角不变形(抓帧);填充模式动画用例。

### TD-M3 tilemap 数据面与渲染器

实施切片:
1. `TilemapAsset`/tile set 资产与导入;三网格坐标换算模块(godot 对照单测)。
2. chunk 网格生成 + 脏重建 + 剔除;`TilemapRenderer` 注册。

测试阶段:
- `cargo test -p zircon_runtime tilemap --locked`(坐标换算对照表;脏 chunk 只重建一个的断言)
- 验收证据:三种网格各一示例场景渲染正确;大地图滚动时仅可见 chunk 提交(stats)。

### TD-M4 排序与画笔契约

实施切片:
1. sorting layer/order in layer/y-sort 接计划 09 排序键;2D/3D 混排用例。
2. 画笔操作契约(批量 set_tiles + 规则匹配钩子)供编辑器消费(编辑器画笔 UI 另立编辑器计划)。

测试阶段:
- `cargo test -p zircon_runtime phase --locked` 2D 排序用例;tilemap 操作 API 单测
- 验收证据:等距场景角色绕物体前后穿行排序正确;sprite 与透明 3D 混排正确。

## 工程落地细化

本章为本计划的实施权威(index.md §8 第 7 条)。全局约定(bind group 槽位、storage buffer 布局、`zr_` WGSL include、RenderQueueValue 数值段、sort_key 唯一布局、测试命名)一律引用 index.md §8,不再重述。跨计划契约名原样使用:计划 04 `ViewVisibilityContext`、计划 09 `RenderQueueValue`/`RenderLayer`(sorting_layer 位段消费)、计划 10 `RendererCommon`/`RendererTypeDescriptor`、本计划 `TextShapingService`/`ShapedGlyphRun`/`TilemapAsset`。

### 模块与文件落点

facade 固定 `zircon_runtime::core::framework::render`;文本/tilemap 契约新增于其下;shaping 与字形图集实现下沉 `graphics/text/` 共享(UI 与场景 2D 共用);不新增渲染 crate;framework 契约层不 import `wgpu`。

**新增文件(契约层,`zircon_runtime/src/core/framework/render/`)**

| 文件 | 内容 | 归属 |
|------|------|------|
| `text/mod.rs` | 模块声明 + 曲面 re-export(薄) | 契约 |
| `text/shaped_run.rs` | `ShapedGlyphRun`、`ShapedGlyph`、`ShapedLine`、`TextDirection` | 契约 |
| `text/shaping_service.rs` | `TextShapingService` trait、`TextShapeRequest`、`TextStyle`、`ShapedTextCacheKey` | 契约 |
| `text/glyph_atlas.rs` | `GlyphAtlasFormat`、`GlyphAtlasLocation`、`GlyphAtlasRef`(纯数据,无 wgpu) | 契约 |
| `text/extract.rs` | `RenderTextSnapshot`、`TextExtract`、`TextSpace` | 契约 |
| `tilemap/mod.rs` | 模块声明(薄) | 契约 |
| `tilemap/asset.rs` | `TilemapAsset`、`TileSetAsset`、`TileDefinition`、`TileCell` | 契约 |
| `tilemap/grid.rs` | `TilemapGridKind`、`TilemapGridDescriptor`、`cell_to_world`/`world_to_cell` 纯函数 | 契约 |
| `tilemap/chunk.rs` | `TilemapChunkData`(32×32)、`chunk_coords_for_cell`、chunk AABB 计算 | 契约 |
| `tilemap/extract.rs` | `RenderTilemapSnapshot`、`TilemapExtract` | 契约 |
| `tilemap/brush.rs` | `TilemapSetTilesBatch`、`TilemapBrushRuleHook`(编辑器消费的画笔契约) | 契约 |
| `sprite/fill_mode.rs` | `RenderImageFillMode`(横向/纵向/径向 fill amount)挂入 `RenderSpriteImageMode` 旁 | 契约 |

**新增文件(实现层,`zircon_runtime/src/graphics/`)**

| 文件 | 内容 |
|------|------|
| `text/mod.rs` | 共享文本服务装配(薄) |
| `text/shaping/mod.rs` | `SharedTextShapingService`(实现 `TextShapingService`,持缓存与图集) |
| `text/shaping/cosmic.rs` | cosmic-text 集成层 —— `cosmic_text` 类型只允许出现在本文件,出口一律 `ShapedGlyphRun` |
| `text/shaping/cache.rs` | shaped run 缓存(`ShapedTextCacheKey -> Arc<ShapedGlyphRun>`,LRU + 帧戳) |
| `text/atlas/mod.rs` | `GlyphAtlasSet`(按 `GlyphAtlasFormat` 分两组页) |
| `text/atlas/shelf_allocator.rs` | shelf(货架行)分配器,页内行高分桶 |
| `text/atlas/page.rs` | 1024×1024 R8 页、脏矩形合并、逐出 |
| `text/sdf.rs` | SDF bake 产物(UI SDF 计划)接入:预烘焙字形直接落 SDF 页 |
| `scene/scene_renderer/text/mod.rs` + `text_renderer.rs` | 场景 `TextRenderer` prepare:shape → glyph quad,经计划 10 注册表注册 |
| `scene/scene_renderer/text/glyph_quads.rs` | glyph quad 顶点生成(复用 `SpriteVertex`,进 sprite 批) |
| `scene/scene_renderer/tilemap/mod.rs` + `tilemap_renderer.rs` | `TilemapRenderer` prepare/queue,经注册表注册 |
| `scene/scene_renderer/tilemap/chunk_mesh.rs` | chunk 顶点烘焙(UV 烘进顶点) |
| `scene/scene_renderer/tilemap/chunk_rebuild.rs` | 脏 chunk 增量重建与顶点缓冲槽管理 |
| `zircon_runtime/src/asset/importer/tilemap/mod.rs` | `TilemapAsset`/tile set 导入(json/二进制 chunk 流) |

**修改文件**

| 文件 | 改动 |
|------|------|
| `core/framework/render/mod.rs` | 增 `text`/`tilemap` 模块声明与 re-export(仅声明) |
| `core/framework/render/frame_extract.rs` | `RenderFrameExtract` 增 `texts: TextExtract`、`tilemaps: TilemapExtract` 字段 |
| `core/framework/render/sprite/mod.rs`、`sprite/sprite.rs` | 挂 `fill_mode: Option<RenderImageFillMode>` 进 snapshot(image 类 sprite 用) |
| `graphics/scene/scene_renderer/sprite/build_sprite_vertices.rs` | 增 fill 模式顶点裁剪分支(横向/径向);nine-slice 既有 `sliced_image_slices` 不动 |
| `graphics/scene/scene_renderer/sprite/prepared_batches.rs` | `batch_sprite_draw_items` 升级:切批边界 = (phase 顺序, texture, 材质变体);text/tilemap quad 复用该入口 |
| `core/framework/render/core_pipeline/phase_queue.rs` | phase item 增 text/tilemap mesh source 变体(对齐既有 `RenderPhaseMeshSource::SpriteIndex` 形态) |
| 计划 10 注册表落点(`RendererTypeDescriptor` 注册处) | 注册 `TextRenderer`/`ImageRenderer`/`TilemapRenderer` 三描述符 |

**UI 文本路径硬切换清单**(TD-M1 切片 1;闸门 = UI 全量文本测试)

| # | 现有文件 | 现状 | 切换步骤 |
|---|---------|------|---------|
| 1 | `zircon_runtime/src/ui/text/shaper.rs` | `UiTextShaperStack`/`UiHeuristicTextShaper`;`active_layout_backend_for_intent` 永远回落 `UiTextBackendIntent::Heuristic`,`fallback_reason_for_backend` 报"backend not connected" | `UiTextShaperStack` 改为持 `&dyn TextShapingService` 的适配器:`shape_text` 把 `UiResolvedStyle` 映射为 `TextStyle`、调 `service.shape`、把 `ShapedGlyphRun` 投影为 `UiResolvedTextLayout`;删除 `UiTextBackendIntent`、`active_layout_backend_for_intent`、`fallback_reason_for_backend` 整个回退机制(硬切换,无双路径) |
| 2 | `zircon_runtime/src/ui/text/layout_engine.rs`(+ `layout_engine/tests.rs`) | 启发式 char_advance 布局:`wrap_source_runs`/`append_word_wrapped_segment`/`apply_visual_order`/`ellipsize_line` 等约 800 行 | 换行/对齐/省略号/双向语义全部迁入 `graphics/text/shaping/`(cosmic-text 提供 shaping 与断行;省略号与 `apply_visual_order` 语义在共享层重实现);本文件删除;`tests.rs` 断言改对共享服务(期望值按真实字形度量重标定) |
| 3 | `zircon_runtime/src/ui/text/hit_test.rs` | `hit_test_text_layout` 按 fragment 矩形 + 均匀 advance 推 `source_offset` | 改基于 `ShapedGlyph.source_range`(cluster→源文本字节区间)反查;函数签名与 `UiTextHitTest` 返回类型不变 |
| 4 | `zircon_runtime/src/ui/text/mod.rs` | re-export `layout_text`/`measure_text_size`/`hit_test_text_layout` | re-export 指向新实现;`grapheme.rs`/`edit_state.rs`/`rich_text.rs` 为纯文本编辑逻辑,不动 |
| 5 | `zircon_runtime/src/ui/surface/render/extract.rs`(L114 `layout_text(`) | 直接调启发式 `layout_text` | 经注入的 service 适配器调用;调用点签名不变 |
| 6 | `zircon_runtime/src/ui/surface/render/text_fields.rs`(L179) | 同上 | 同上 |
| 7 | `zircon_runtime/src/ui/surface/render/text_measure.rs` | `measure_text_size` 启发式估宽 | 改调 `service.measure`(taffy measure 闭包入口,注意缓存:measure 必须走 shaped cache) |
| 8 | `zircon_runtime/src/ui/surface/input/text_pointer.rs`(L254) | 消费 `hit_test_text_layout` | 行为不变,随 #3 的 cluster 语义自动升级 |
| 9 | `zircon_runtime/src/ui/surface/mod.rs`(L18 `pub use crate::ui::text::layout_text`) | 公开 re-export | 保留导出名,指向新实现(`ui/tests/boundary.rs` L981 断言依赖该符号名) |
| 闸门 | `ui/tests/{text_shaper,text_layout,text_hit_testing,render_text_fields,widget_text_input_pointer,surface_dirty_mui}.rs` | 现有全量文本测试 | 切换后全绿才许合入;失败修服务,不回退双路径 |

### 核心类型与接口

契约层(`core/framework/render/text/`,serde 可序列化,无 wgpu):

```rust
// shaped_run.rs
pub struct ShapedGlyphRun {
    pub glyphs: Vec<ShapedGlyph>,
    pub lines: Vec<ShapedLine>,
    pub size: Vec2,                  // 排版后包围尺寸
    pub base_direction: TextDirection,
}
pub struct ShapedGlyph {
    pub glyph_id: u16,
    pub font_id: ResourceId,         // 字体回退后实际命中的 face
    pub source_range: (u32, u32),    // cluster -> 源文本字节区间(hit test / 光标)
    pub offset: Vec2,                // 行内 pen 位置(含 bearing 前)
    pub advance: f32,
    pub line_index: u32,
    pub atlas: GlyphAtlasRef,        // 页 id + uv + 像素偏移/尺寸 + 格式
    pub direction: TextDirection,
}
pub struct ShapedLine { pub glyph_range: (u32, u32), pub baseline_y: f32,
    pub width: f32, pub ascent: f32, pub descent: f32 }

// shaping_service.rs
pub struct TextStyle {
    pub font: ResourceHandle<FontMarker>,
    pub font_size: f32,
    pub line_height: f32,
    pub letter_spacing: f32,
    pub features_hash: u64,          // OpenType features + locale 的稳定 hash
    pub format: GlyphAtlasFormat,    // Sdf | AlphaMask(承接 UiTextRenderMode)
}
pub struct TextShapeRequest<'a> {
    pub text: &'a str,
    pub style: &'a TextStyle,
    pub wrap_width: Option<f32>,
    pub wrap: TextWrapMode,          // None | Word | Glyph
    pub align: TextAlign,
    pub base_direction: TextDirection, // Auto | Ltr | Rtl
}
pub trait TextShapingService {
    fn shape(&self, request: &TextShapeRequest<'_>) -> Arc<ShapedGlyphRun>;
    fn measure(&self, text: &str, style: &TextStyle, wrap_width: Option<f32>) -> Vec2;
}
// 缓存键:font_id + size + features + wrap + 文本 hash(不持文本引用,UE FCachedShapedTextKey 改造)
pub struct ShapedTextCacheKey {
    pub font_id: ResourceId,
    pub font_size_bits: u32,         // f32::to_bits,浮点稳定
    pub features_hash: u64,
    pub wrap_width_bits: u32,        // None -> u32::MAX
    pub wrap_align_dir: u16,         // wrap/align/direction/format 打包
    pub text_hash: u64,              // FxHash64(text)
}

// glyph_atlas.rs
pub enum GlyphAtlasFormat { AlphaMask, Sdf }   // 都是 R8,两组页隔离
pub struct GlyphAtlasRef { pub page: u16, pub format: GlyphAtlasFormat,
    pub uv_min: Vec2, pub uv_max: Vec2,
    pub bearing: Vec2, pub px_size: Vec2 }      // bearing = 左上像素偏移

// extract.rs
pub enum TextSpace { World, Screen }
pub struct RenderTextSnapshot {
    pub entity: EntityId, pub transform: Transform,
    pub space: TextSpace,
    pub text: String, pub style: TextStyle,
    pub bounds: Option<Vec2>, pub align: TextAlign,
    pub color: Vec4,
    pub common: RendererCommon,      // 计划 10;layer_mask/queue override 由此来
    pub sorting_layer: u16, pub order_in_layer: i32, pub y_sort: bool,
}
```

契约层(`core/framework/render/tilemap/`):

```rust
// asset.rs
pub struct TileSetAsset {
    pub texture: ResourceHandle<TextureMarker>,
    pub grid: TilemapGridDescriptor,
    pub tiles: Vec<TileDefinition>,
}
pub struct TileDefinition {
    pub atlas_region: RenderSpriteAtlasRegion,   // 复用 sprite 契约
    pub y_sort_origin: f32,                      // godot tile_data->get_y_sort_origin 对应
    pub collision_slot: Option<u16>,             // V1 只留槽
    pub custom_properties: Vec<(String, TilePropertyValue)>, // 规则钩子消费
}
#[repr(transparent)]
pub struct TileCell(pub u32); // 位段:tile_index:24 | flip_x:1 | flip_y:1 | transpose:1 | 保留:5;0 = 空
pub struct TilemapAsset {
    pub tile_set: ResourceHandle<TileSetMarker>,
    pub chunks: Vec<TilemapChunkData>,
}
// grid.rs
pub enum TilemapGridKind { Rect, HexPointyTop, HexFlatTop, Isometric }
pub struct TilemapGridDescriptor { pub kind: TilemapGridKind, pub cell_size: Vec2 }
pub fn cell_to_world(grid: &TilemapGridDescriptor, cell: IVec2) -> Vec2;  // 返回 cell 中心(godot map_to_local 约定)
pub fn world_to_cell(grid: &TilemapGridDescriptor, world: Vec2) -> IVec2;
// chunk.rs
pub const TILEMAP_CHUNK_SIZE: u32 = 32;
pub struct TilemapChunkData { pub chunk_coords: IVec2, pub cells: Box<[TileCell; 1024]> }
pub fn chunk_coords_for_cell(cell: IVec2) -> IVec2; // 负坐标向下取整(godot _coords_to_quadrant_coords 语义)
pub fn chunk_world_aabb(grid: &TilemapGridDescriptor, chunk_coords: IVec2) -> RenderSpriteBounds;
// brush.rs —— 编辑器消费,runtime 只执行
pub struct TilemapSetTilesBatch { pub edits: Vec<(IVec2, TileCell)> } // TileCell(0) = 擦除
pub trait TilemapBrushRuleHook {
    /// 编辑器规则画笔(地形过渡/随机)在写入前对每格重解析;V1 钩子,完整规则系统随编辑器画笔计划
    fn resolve(&self, cell: IVec2, requested: TileCell, neighborhood: &[TileCell; 8]) -> TileCell;
}
```

实现层归属:`SharedTextShapingService`(graphics/text)注册为 runtime 服务,UI 模块与场景 renderer 同一实例;`cosmic_text::{FontSystem, Buffer, ShapeBuffer}` 等类型不得越出 `graphics/text/shaping/cosmic.rs`。字形图集:shelf 分配器(行高向上取整到 8px 分桶,行内 x 递增分配,glyph 间 padding 2px,bevy `DynamicTextureAtlasBuilder::new(size, 2)` 同值);页 1024×1024、`R8Unorm`,SDF 页与 alpha 页分组管理;每格式上限 8 页;逐出策略:页级 LRU —— glyph 命中即刷新所在页帧戳,页满且需新页时整页清空重建(glyph 映射表一并失效,UE FSlateFontCache flush 风格,不做逐字搬迁),本帧已引用的页不可逐出。

### 坐标换算与顶点生成

**三网格 cell↔world**(`tilemap/grid.rs` 纯函数;统一返回 cell 中心;`cs = cell_size`;godot 对照见各条):

1. Rect:
   - `cell_to_world: world = (cell + 0.5) * cs`
   - `world_to_cell: cell = floor(world / cs)`
   - godot 对照:`TileSet::map_to_local` 方形分支 `(ret + (0.5,0.5)) * tile_size`(tile_set.cpp L1626)。
2. Hex pointy-top(axial 坐标 `q,r`;尖顶,行按 0.75 高度叠压):
   - `cell_to_world: x = cs.x * (q + r * 0.5); y = cs.y * 0.75 * r`(再各 + 0.5*cs 取中心由公共项处理)
   - `world_to_cell`:`r_f = y / (0.75 * cs.y); q_f = x / cs.x - r_f * 0.5`,然后 cube round:`s_f = -q_f - r_f`,各分量取整后把舍入误差最大的分量改为 `-(其余两量之和)`。
   - godot 对照:hexagon = half-offset square 族 + `overlapping_ratio = 0.75`(tile_set.cpp L1613-1616);其 STACKED 偏移坐标 `col = q + floor(r/2)`,单测对照表用该换算互转。
3. Hex flat-top(平顶,列按 0.75 宽度叠压;轴交换):
   - `cell_to_world: x = cs.x * 0.75 * q; y = cs.y * (r + q * 0.5)`
   - `world_to_cell`:对称地 `q_f = x / (0.75 * cs.x); r_f = y / cs.y - q_f * 0.5`,cube round 同上。
   - godot 对照:`TILE_OFFSET_AXIS_VERTICAL` 分支(`ret.x *= 0.75`,tile_set.cpp L1617-1624)。
4. Isometric(菱形 2:1,godot `TILE_SHAPE_ISOMETRIC`、`overlapping_ratio = 0.5`):
   - `cell_to_world: x = (cell.x - cell.y) * cs.x * 0.5; y = (cell.x + cell.y) * cs.y * 0.5`
   - `world_to_cell`:`a = x / (cs.x * 0.5); b = y / (cs.y * 0.5); cell = (round((a + b) * 0.5), round((b - a) * 0.5))`(round = 最近整数;菱形边界归属与 godot smart-floor + 顶角三角形叉积修正等价,见 tile_set.cpp L1666-1667 `in_top_left_triangle`/`in_top_right_triangle`)。
   - godot 对照:`TILE_LAYOUT_DIAMOND_DOWN` 分支 `ret = ((x - y) / 2, y + x)` 后 `ret.y *= 0.5`(tile_set.cpp L1580-1581、L1611-1616)。
   - 单测以 godot 公式表驱动:每网格各取 ≥12 个 cell(含负象限)双向 round-trip。

**nine-slice 顶点表**(现状:sprite 路径已由 `build_sprite_vertices.rs::sliced_image_slices`(L314)+ `tile_slice`(L416)实现;本节定稿其共享化形式,UI 与 `ImageRenderer` 复用同一函数):

设绘制矩形 `[P0, P3]`、源 UV `[U0, U3]`、边框 `b = RenderSpriteSliceBorder`、角缩放 `k = min(1, w/(b.left+b.right), h/(b.top+b.bottom), max_corner_scale)`:

| 轴向断点 | 位置 | UV |
|---------|------|----|
| `x0..x3` | `P0.x`, `P0.x + b.left*k`, `P3.x - b.right*k`, `P3.x` | `U0.u`, `U0.u + b.left/tex_w`, `U3.u - b.right/tex_w`, `U3.u` |
| `y0..y3` | `P0.y`, `P0.y + b.top*k`, `P3.y - b.bottom*k`, `P3.y` | `U0.v`, `U0.v + b.top/tex_h`, `U3.v - b.bottom/tex_h`, `U3.v` |

9 子矩形 = `(col,row) ∈ {0,1,2}²`,quad(col,row) 取 `[x_col, x_{col+1}] × [y_row, y_{row+1}]` 与对应 UV;4 角恒为 Stretch;边/中按 `RenderSpriteSliceScaleMode`:`Stretch` 单 quad,`Tile{stretch_value}` 按源尺寸×stretch_value 重复切分(上限沿用 `MAX_SPRITE_IMAGE_SLICES = 1000`)。

**ImageRenderer 填充模式顶点裁剪**(`sprite/fill_mode.rs` 契约 + `build_sprite_vertices.rs` 分支):

- `Horizontal { origin: Left|Right, amount }`:`x1' = x0 + (x3 - x0) * amount`(Right 则从右收),UV 同比例;单 quad,2 三角。
- `Vertical` 对称。
- `Radial { origin_angle, clockwise, amount }`:`θ = amount * 2π`;把矩形按四象限切扇:每整 90° 段输出固定三角(中心、两角点);末段顶点 = 中心 + 射线 `(cos, sin)(origin_angle ± θ)` 与矩形边求交;输出 ≤ 10 三角;UV 取位置在矩形内的双线性参数。函数 `radial_fill_triangles(rect, uv, origin_angle, clockwise, amount) -> Vec<[SpriteVertex; 3]>`,纯函数可单测。

**字形 quad 生成**(`glyph_quads.rs`):对 `ShapedGlyphRun` 每 glyph:`pos_min = line_origin + glyph.offset + atlas.bearing * scale`,`pos_max = pos_min + atlas.px_size * scale`;`scale = style.font_size / atlas_baked_px`(SDF 页按 bake 尺寸缩放,alpha 页 1:1,scale 恒 1);SDF quad 四边各外扩 `spread_px * scale`;UV 直接取 `atlas.uv_min/uv_max`。world 空间:再乘 snapshot.transform;screen 空间:走相机 viewport 像素系。输出 `SpriteVertex`(position/uv/color)序列,texture = 图集页 —— 与 sprite 共用 `prepare_sprite_draw_batches` 通道;SDF 着色为 material 级变体(group2,距离阈值 + `fwidth` 抗锯齿),WGSL include `zr_text_sdf.wgsl`(index §8 命名约定)。

### 帧时序与集成点

逐帧顺序(全部在既有 Extract → Prepare → Queue/Sort → Execute 骨架内,pass 经 graph,无旁路):

1. **Extract**:`TextRenderer`/`ImageRenderer`/`TilemapRenderer` 组件经计划 10 注册表的 extract 段写入 `RenderFrameExtract.texts / .tilemaps / .sprites`(纯数据快照,不触 shaping 服务)。渲染模块只消费 `RenderFrameExtract`。
2. **Prepare — shaping 缓存**:场景文本与 UI 同一 `SharedTextShapingService`;`ShapedTextCacheKey` 命中直接取 `Arc<ShapedGlyphRun>`;未命中执行 cosmic-text shaping + 断行 + 对齐,新 glyph 进图集(shelf 分配);缓存 LRU 上限 1024 run / 8 MiB,帧末 trim;本帧引用的 run 与图集页打帧戳。
3. **Prepare — 图集上传**:本帧新增 glyph 按页合并脏矩形,每页至多一次 texture 上传(graph 资源节点声明 IO,经计划 01 资源图)。
4. **Prepare — chunk 重建**:`set_tiles` 批在模拟帧落到 `TilemapAsset` 时把触碰 cell 映射为脏 chunk 集(`HashSet<IVec2>`,`chunk_coords_for_cell` 负坐标向下取整);prepare 期只对脏 chunk 调 `chunk_mesh.rs` 重建该 chunk 顶点段(顶点缓冲按 chunk 槽位管理,非脏 chunk 零拷贝复用);y-sort 启用时 chunk 内按 cell 行拆子段(godot y-sort quadrant 行桶语义),每子段独立 sort 深度。
5. **可见性**:`chunk_world_aabb` 全集注册进计划 04 `ViewVisibilityContext`;仅可见 chunk 的段进 phase queue(大地图滚动验收的 stats 即来自此)。
6. **Queue/Sort**:2D 项 sort_key 一律调用计划 09 的编码函数(sorting_layer → order_in_layer → y-sort/深度位段);本计划不自定义位段。y 值取 `world_y + tile.y_sort_origin`(tilemap)或 snapshot 平移 y(sprite/text)。
7. **批与提交**:text/tilemap/image 的 quad 统一进 `prepare_sprite_draw_batches` 通道;`batch_sprite_draw_items` 升级为按(phase item 顺序, texture, material 变体)切批 —— 排序边界优先于纹理合并,跨 sorting_layer 不合批。
8. **UI 集成**:UI 文本路径按硬切换清单消费同一服务;screen-space UI 渲染链(GPU Command Stream)不改,仅其文本布局数据源替换。

**硬切换删除项**(与 TD-M1 同一变更内删除,不留兼容层):`ui/text/layout_engine.rs` 启发式布局全体、`ui/text/shaper.rs` 的 `UiTextBackendIntent`/`active_layout_backend_for_intent`/`fallback_reason_for_backend` 回退机制、`text_measure.rs` 对启发式 measure 的直接调用。

### 实施切片细化

**TD-M1 文本服务下沉与场景文本渲染器**

| 切片 | 触碰文件 | 改动要点 | 完成判据 |
|------|---------|---------|---------|
| 1a 契约 + 服务骨架 | 新增 `core/framework/render/text/*`;`graphics/text/{mod,shaping/*,atlas/*}.rs`;Cargo.toml 增 `cosmic-text` 依赖 | `ShapedGlyphRun`/`TextShapingService` 定稿;cosmic.rs 隔离层;shelf 图集 + 缓存 | `cargo check -p zircon_runtime --lib --locked`;`render_text_shape_*`/`render_text_atlas_*` 单测绿 |
| 1b SDF 产物接入 | `graphics/text/sdf.rs` | UI SDF bake 计划产物 → SDF 页直灌;`GlyphAtlasFormat::Sdf` 路径可用 | SDF 字形 quad UV/spread 断言绿 |
| 1c UI 硬切换 | 硬切换清单 #1–#9 全部文件 | 按清单逐文件执行;删除启发式路径 | 闸门:`cargo test -p zircon_runtime --lib --locked` 中 ui 文本全量(text_shaper/text_layout/text_hit_testing/render_text_fields/widget_text_input_pointer/surface_dirty_mui/boundary)全绿 |
| 2 场景 TextRenderer | 新增 `text/extract.rs`、`scene_renderer/text/*`;改 `frame_extract.rs`、phase_queue.rs、计划 10 注册表 | snapshot/extract 进 `RenderFrameExtract`;glyph quad 走 sprite 批;world/screen 双空间 | `render_text_renderer_*` 绿;`render_product_text_scene` 抓帧对拍 |

**TD-M2 九宫与图像渲染器**

| 切片 | 触碰文件 | 改动要点 | 完成判据 |
|------|---------|---------|---------|
| 1 | 新增 `sprite/fill_mode.rs`;改 `sprite/sprite.rs`、`build_sprite_vertices.rs`、UI 九宫调用点 | fill 横向/纵向/径向顶点裁剪;nine-slice 断点表函数从 `sliced_image_slices` 抽为共享纯函数供 UI 复用(签名搬移,行为不变) | `cargo test -p zircon_runtime sprite --locked` 全绿(既有 `sprite_image_vertices_slice_custom_size_into_nine_regions` 等不回退);`render_image_fill_*` 新测绿 |

**TD-M3 tilemap 数据面与渲染器**

| 切片 | 触碰文件 | 改动要点 | 完成判据 |
|------|---------|---------|---------|
| 1 资产 + 坐标 | 新增 `tilemap/{asset,grid,chunk}.rs`、`asset/importer/tilemap/` | 三网格公式落地(本章公式);导入器产 chunk 流 | `render_tilemap_*_roundtrip_matches_godot_table` 三网格全绿 |
| 2 渲染器 | 新增 `tilemap/extract.rs`、`scene_renderer/tilemap/*`;改 `frame_extract.rs`、注册表、计划 04 注册点 | chunk mesh 烘焙;脏增量重建;AABB 进 `ViewVisibilityContext` | `cargo test -p zircon_runtime tilemap --locked`;滚动场景 stats 仅可见 chunk 提交 |

**TD-M4 排序与画笔契约**

| 切片 | 触碰文件 | 改动要点 | 完成判据 |
|------|---------|---------|---------|
| 1 排序接入 | `scene_renderer/{sprite,text,tilemap}` queue 段、`prepared_batches.rs` | 全部 2D 项改调计划 09 编码函数;y-sort 子段;批切分尊重排序边界 | `cargo test -p zircon_runtime phase --locked`;`render_2d_sort_*`、混排 `render_product_*` 对拍 |
| 2 画笔契约 | 新增 `tilemap/brush.rs`;runtime 侧 `set_tiles` 执行入口 | 批量 set_tiles + `TilemapBrushRuleHook`;编辑器画笔 UI 另立编辑器计划 | `render_tilemap_set_tiles_*` 单测;脏 chunk 精确性断言 |

前置依赖:TD-M4 切片 1 依赖计划 09 CO-M3(sort_key 编码函数)落地;TD-M1/M2/M3 不被阻塞(过渡期沿用现状 `RenderPhaseSortComponents::with_order_in_layer` 通道,TD-M4 一次性切换)。

### 测试与验收清单

单测(`cargo test -p zircon_runtime <过滤词> --locked`;命名遵循 index §8 `render_<topic>_*`):

| 测试函数 | 断言 | 位置 |
|---------|------|------|
| `render_text_shape_cache_hits_for_identical_request` | 同 key 二次 shape 返回同一 `Arc`(ptr_eq),miss 计数不增 | `graphics/text/shaping/cache.rs` tests |
| `render_text_shaped_run_maps_clusters_to_source_ranges` | 合字/组合字符 `source_range` 覆盖完整字节区间且单调 | `graphics/text/shaping/` tests |
| `render_text_wrap_mixed_cjk_latin_lines_match_expected_breaks` | 中英混排在给定宽度下行数与断点匹配期望表 | 同上 |
| `render_text_atlas_shelf_allocates_same_height_glyphs_into_one_row` | 同高桶字形落同 shelf 行,x 递增,padding=2 | `graphics/text/atlas/shelf_allocator.rs` tests |
| `render_text_atlas_evicts_lru_page_and_invalidates_glyph_refs` | 页满逐出最旧未引用页;本帧引用页不可逐出 | `graphics/text/atlas/page.rs` tests |
| `render_text_renderer_world_space_quads_enter_sprite_batches` | glyph quad 出现在 prepared batch,texture=图集页,批数符合预期 | `scene_renderer/text/` tests |
| `render_nineslice_vertices_keep_corner_size_under_resize` | 任意目标尺寸下 4 角尺寸 = border*k,k 钳制正确 | `build_sprite_vertices.rs` tests(既有 9 宫测试旁) |
| `render_nineslice_tile_center_repeats_by_stretch_value` | Tile 模式中段 quad 数 = ceil(span/(src*stretch)) | 同上 |
| `render_image_fill_horizontal_clips_quad_and_uv` | amount=0.25 时 x1'/u1' 同比例;amount 0/1 退化正确 | 同上 |
| `render_image_fill_radial_segments_cover_expected_angle` | amount=0.625 输出三角覆盖角域 [origin, origin+1.25π],面积比≈amount | 同上 |
| `render_tilemap_rect_cell_world_roundtrip_matches_godot_table` | ≥12 cell(含负象限)双向换算对 godot 公式表 | `tilemap/grid.rs` tests |
| `render_tilemap_hex_pointy_cell_world_roundtrip_matches_godot_table` | 同上 + cube round 边界格归属 | 同上 |
| `render_tilemap_hex_flat_cell_world_roundtrip_matches_godot_table` | 同上 | 同上 |
| `render_tilemap_isometric_cell_world_roundtrip_matches_godot_table` | 同上 + 菱形顶角三角归属(godot 叉积修正等价) | 同上 |
| `render_tilemap_set_tiles_marks_only_touched_chunks_dirty` | 跨 chunk 批编辑只 dirty 触碰 chunk;负坐标 chunk 映射正确 | `tilemap/chunk.rs` / brush tests |
| `render_tilemap_rebuild_regenerates_single_dirty_chunk_buffer` | 改 1 cell 仅 1 chunk 重建(重建计数器断言) | `chunk_rebuild.rs` tests |
| `render_tilemap_chunk_aabbs_register_into_view_visibility` | AABB 数 = chunk 数;视锥外 chunk 不进 phase queue | `scene_renderer/tilemap/` tests |
| `render_tilemap_ysort_rows_emit_row_buckets_with_distinct_sort_keys` | y-sort 开启时每行子段 sort 深度严格递增 | 同上 |
| `render_2d_sort_key_orders_sorting_layer_before_order_before_y` | 消费计划 09 编码:构造三元组矩阵断言全序 | `core_pipeline` phase tests |

产物对拍(`render_product_*` + `ZR_RENDERDOC_CAPTURE_NEXT=1` 抓帧):`render_product_text_scene`(world 空间中英混排 + SDF/alpha 双路径)、`render_product_nineslice_resize`(三尺寸九宫边角不变形)、`render_product_tilemap_three_grids`(rect/hex/iso 各一示例场景)、`render_product_tilemap_isometric_ysort`(角色绕物体前后穿行帧序)、`render_product_2d_3d_mixed_transparency`(sprite 与透明 3D 混排)。

里程碑命令:TD-M1 `cargo test -p zircon_runtime text --locked` + UI 全量文本闸门;TD-M2 `cargo test -p zircon_runtime sprite --locked`;TD-M3 `cargo test -p zircon_runtime tilemap --locked`;TD-M4 `cargo test -p zircon_runtime phase --locked`。切片期一律 `cargo check -p zircon_runtime --lib --locked`(milestone-first)。

## 状态与产出记录

| 日期 | 里程碑/切片 | 状态 | 产出 | 验证与证据 | 后续 |
|------|-------------|------|------|------------|------|
| 2026-06-15 | TD-M1 text service and scene text renderer | 未启动: 文本仍主要在 UI 内部 | UI 文本/排版能力存在,但场景 2D 无共享 shaping/atlas 服务和 scene text renderer。 | 本文件 `现状与差距` 明确文本仅 UI 内部、场景 2D 不能直接复用。 | 下沉 font/shaping/atlas 服务,实现 scene text draw command 和 batching。 |
| 2026-06-15 | TD-M2 nine-slice and image renderer | 部分完成: sprite/atlas 基础存在,九宫和 image renderer 未完成 | sprite renderer 与 atlas management 可用,prepared batches 有批组织;九宫、image renderer、sprite material override 仍待计划实现。 | 本文件 `现状与差距` 记录 sprite 渲染器和图集管理可用但无九宫。 | 实现 nine-slice mesh generation、image renderer asset contract 和 batching stats。 |
| 2026-06-15 | TD-M3 tilemap data plane and renderer | 未启动: tilemap 数据面缺失 | 无 tilemap chunk、palette、collision/render separation 或 streaming 机制。 | 本文件 `现状与差距` 明确无 tilemap。 | 建立 tilemap asset/chunk renderer、dirty region upload 和 material atlas binding。 |
| 2026-06-15 | TD-M4 sorting and brush contract | 部分完成: UI 内部排序自洽,场景 2D 混排未统一 | UI z-index 与 painter 内部已有规则,但 sprite/world-space UI/3D 统一排序仍依赖计划 09。 | 计划 09 状态表记录 unified sort key 尚未完成;本文件记录无 y-sort。 | 与计划 09 CO-M3 对齐 sort key、y-sort、layer 和 brush priority。 |

### 参考实现精读笔记

1. **godot `scene/2d/tile_map_layer.cpp/.h`**(quadrant 组织与脏重建)
   - `TileMapLayer::_coords_to_quadrant_coords`(L58):负坐标向下取整的整除(`(x - (size-1)) / size`),Zircon `chunk_coords_for_cell` 直接采用该语义。
   - 脏跟踪:`dirty.flags[DIRTY_FLAGS_MAX]` 布尔位 + `dirty.cell_list`(侵入式 `SelfList<CellData>` 链,tile_map_layer.h L419-421);`quadrant_shape_changed`(L251-253)在 y-sort 开关、tile_set 更换、quadrant size 改变时全清重建,否则仅 `_rendering_quadrants_update_cell` 把受影响 quadrant 挂入 `dirty_rendering_quadrant_list` 增量重绘。Zircon 对应:cell 级编辑 → chunk 脏集;grid/tile_set 级变更 → 全 chunk 脏。
   - y-sort:启用时 quadrant 退化为"行桶"——`canvas_items_position = (0, map_to_local(coords).y + tile_y_sort_origin + y_sort_origin)`、`quadrant_coords = canvas_items_position * 100`(L552-553),即按世界 y(放大 100 取整)聚簇;`rendering_quadrant_size` 默认 16。Zircon 取舍:保留固定 32×32 chunk 做剔除与缓冲管理,仅在 y-sort 开启时 chunk 内按行拆提交子段 —— 不学 canvas item 重聚簇,因为我们直接产顶点缓冲、排序交给计划 09 sort_key。
2. **godot `scene/resources/2d/tile_set.cpp`**(坐标换算权威)
   - `TileSet::map_to_local`(L1556-1627):half-offset square / hexagon / isometric 三形共用同一半偏移公式族,差异仅 `overlapping_ratio`(hex=0.75、iso=0.5,L1610-1624);返回 cell 中心 `(ret + (0.5,0.5)) * tile_size`。
   - `TileSet::local_to_map`(L1629-1755):先除 overlapping_ratio,再"smart floor" + `in_top_left_triangle`/`in_top_right_triangle` 叉积测试(L1666-1667)修正顶角三角形归属。Zircon 取舍:对外用 axial hex / diamond iso 坐标(公式更可逆、便于 cube round),单测以 godot stacked / DIAMOND_DOWN 布局数值做对照表,保证渲染语义一致而不复刻其六种 `tile_layout` 全集。
3. **bevy `crates/bevy_text/src/pipeline.rs`**(shaping→图集→quad 链路)
   - `TextPipeline::update_text_layout_info`(L301):遍历 `layout.lines()` 的 `PositionedLayoutItem::GlyphRun`,组 `FontAtlasKey { id, index, font_size_bits, variations_hash, hinting, font_smoothing }`,逐 glyph `get_glyph_atlas_info` miss 时 `add_glyph_to_atlas`,产 `PositionedGlyph { position, atlas_info, section_index, line_index }`;`RunGeometry` 记 underline/strikethrough 几何。注意:本仓 vendored bevy 已切 parley 后端(`Layout<TextBrush>`/`FontCx`),而非 cosmic-text —— 链路结构同构,Zircon 按既定选型用 cosmic-text,`ShapedGlyphRun` 隔离层保证后端可替换正是为此。
   - 借鉴:`font_size_bits = f32::to_bits` 进缓存键(Zircon `ShapedTextCacheKey.font_size_bits` 同法);quad position = 图集尺寸/2 + glyph 位置 + `atlas_info.offset`(bearing),Zircon `glyph_quads.rs` 公式同源。
4. **bevy `crates/bevy_text/src/font_atlas.rs`**(字形图集)
   - `FontAtlas { dynamic_texture_atlas_builder, glyph_to_atlas_index: HashMap<GlyphCacheKey, GlyphAtlasLocation>, texture_atlas, texture }`(L29);`add_glyph_to_atlas`(L130)先 `get_outlined_glyph_texture` 栅格化再逐页尝试,页满建新页;padding=2(`DynamicTextureAtlasBuilder::new(size, 2)`,L62)。Zircon 取舍:用 shelf 行分配器替代其动态 builder(字形高度聚类下碎片更低、实现更可控);`R8Unorm` 单通道页替代其 `Rgba8UnormSrgb`(SDF 与 alpha 都只需单通道,显存 1/4);增加页级 LRU 逐出(bevy 无逐出,长会话字形累积)。
5. **UE `SlateCore/Private/Fonts/FontCache.cpp` + `Slate/Framework/Text/ShapedTextCache.h`**(shaped run 缓存键)
   - `FShapedGlyphSequence`(FontCache.cpp L200):GlyphsToRender + TextBaseline + MaxTextHeight + SourceTextRange;`FShapedGlyphEntry { GlyphIndex, SourceIndex, XAdvance, XOffset, Kerning, NumCharactersInGlyph, NumGraphemeClustersInGlyph, TextDirection, bIsVisible }`(FontCache.h L149-184)—— Zircon `ShapedGlyph.source_range` 即 `SourceIndex + NumCharactersInGlyph` 的区间化,cluster 命中测试 `GetGlyphAtOffset`(L349)对应 `hit_test` 新实现。
   - 缓存键 `FCachedShapedTextKey { TextRange, Scale, TextContext, FontInfo }`(ShapedTextCache.h L13-50):UE 持文本区间 + 完整 FontInfo 等值比较;Zircon 取舍:改 `text_hash + font_id + font_size_bits + features_hash`,避免缓存键持有文本/字体对象引用(ABI 与生命周期更干净),代价是理论 hash 碰撞,以 64 位 hash + 字体 id 复合键压到可忽略。
   - shaping 入口对 `FSlateFontCache::ShapeBidirectionalText / ShapeUnidirectionalText`(L1222/L1232)的双向拆分:cosmic-text 内置 bidi,Zircon 不需要该双入口,`TextShapeRequest.base_direction` 单入口即可。

## 风险与回退

- shaping 库选型(cosmic-text vs 自研 harfbuzz 绑定):优先 cosmic-text(纯 Rust,bevy 同源);若 CJK 排版细节不足再评估,接口以 `ShapedGlyphRun` 隔离选型。
- UI 文本硬切换风险大:TD-M1 以 UI 全量文本测试为闸门,失败即修服务而不是回退双路径。
- tilemap 动画 tile/自动地形(terrain tile)规则:V1 只留属性槽与规则钩子,完整规则系统随编辑器画笔计划推进。
