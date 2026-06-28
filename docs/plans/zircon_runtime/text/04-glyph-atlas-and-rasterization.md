---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs
  - zircon_runtime/src/ui/text/raster.rs
  - zircon_runtime/src/ui/text/measure_cache.rs
  - zircon_runtime_interface/src/ui/surface/render/resolved_style.rs
design_references:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts/FontTypes.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/SlateFontRenderer.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts/FontRasterizationMode.h
  - dev/bevy/crates/bevy_text/src/font_atlas.rs
  - dev/bevy/crates/bevy_text/src/font_atlas_set.rs
  - dev/Fyrox/fyrox-ui/src/font/mod.rs
  - dev/slint/internal/core/textlayout/sharedparley.rs
plan_sources:
  - docs/plans/zircon_runtime/text/index.md
  - docs/plans/zircon_runtime/text/02-shaping-unicode-and-bidi.md
  - docs/plans/zircon_editor/editor_layout/17-text-rendering-and-typography.md
  - docs/plans/zircon_runtime/render/14-2d-stack.md
status: planned
---

# 04 字形栅格化 / 字形图集 / 分辨率精度

> 本计划把 `02` 的 `ShapedGlyph.glyph_id` 栅格成像素并装进 GPU 图集。它是 `editor_layout/17 G2`(字形随 DPI 重栅格,根治像素化)的实现,统一现有 glyphon bitmap atlas 与 SDF atlas 的分配/上传策略。

## 1. 目标

1. **栅格器选型与统一**:bitmap 路径用 swash(彩色 emoji + outline alpha + subpixel);SDF/MSDF 路径见 `05`。栅格输入按物理像素。
2. **图集化生成**:shelf(货架行)分配器、多页管理、脏矩形增量上传、页级 LRU 逐出;`R8Unorm`(alpha/SDF)与 `Rgba8Unorm`(彩色/MSDF)分组分页。
3. **分辨率精度**:`physical_px = logical_px × scale_factor`;scale 变即重栅格;subpixel 定位(水平 1/3 量化或整像素吸附);hinting 策略;atlas key 含 scale 量化桶。
4. **统一图集服务**:UI 与场景 2D 共用 `GlyphAtlasSet`(`render/14` 已起名),替换现有各自为政的 glyphon `TextAtlas` 与 `sdf_atlas`。

## 2. 现状与差距

- `graphics/.../ui/text.rs`:glyphon 自管 `TextAtlas` + `SwashCache`,栅格/装箱/上传都在 glyphon 内部,ZirconEngine 不可控、不能与 SDF 共享。
- `ui/sdf_atlas.rs`:自有 LRU 图集(256 槽、固定 64×64、8×8 网格),`sdf_upload.rs` 脏槽上传**已设计未启用**(仅全纹理上传)。
- `ui/text/raster.rs`:`raster_path_for`/`UiGlyphRasterPolicy` 选 SDF vs bitmap 的策略在,但栅格本身分散。
- **无统一 shelf 分配器**、无 DPI 重栅格契约(scale 变不重栅格 → 放大像素化,`editor_layout/17 G2`)、无 subpixel、无 hinting 策略书面化。

## 3. 参考代码

| 文件 | 应重点阅读 |
|------|-----------|
| `dev/bevy/crates/bevy_text/src/font_atlas.rs` | `FontAtlas { dynamic_texture_atlas_builder, glyph_to_atlas_index, texture }`;`add_glyph_to_atlas`(栅格→装箱→页满建新页);`get_outlined_glyph_texture`;padding=2。**Rust 图集落地主样板** |
| `dev/bevy/crates/bevy_text/src/font_atlas_set.rs` | `FontAtlasKey { font_size_bits, variations_hash, hinting, font_smoothing }`——**图集缓存键含 scale/hinting 的权威**;按 (font, size) 分图集 |
| `dev/Fyrox/fyrox-ui/src/font/mod.rs` | `Page` + `RectPacker` + `FontGlyph { bitmap_top/left, tex_coords, page_index }`,多页扩展、字形过大处理(`GlyphTooLarge`) |
| `dev/UnrealEngine/.../Fonts/FontTypes.h` | `FSlateFontAtlas`/`FSlateTextureAtlas`:动态装箱、按内容类型(Alpha/ColorBgra/MCDF)分纹理、`FAtlasedTextureSlot` 链表分配 |
| `dev/UnrealEngine/.../Fonts/SlateFontRenderer.cpp` | FreeType 栅格、subpixel、`FCharacterRenderData`(像素 + bearing);hinting/LCD 过滤 |
| `dev/UnrealEngine/.../Fonts/FontRasterizationMode.h` | `EFontRasterizationMode::{Bitmap,Msdf,Sdf}`——栅格模式枚举对照 |
| `dev/slint/.../textlayout/sharedparley.rs` | parley + swash 栅格缓存的轻量组织 |

**Rust/wgpu 落地**:swash `ScaleContext`/`Scaler`/`Render::new(&[Source::ColorOutline, Source::ColorBitmap, Source::Outline]).format(Format::Alpha|SubpixelMask)`(bevy `font_atlas.rs` 同款);`etagere`(shelf/guillotine 装箱,可选)。`render/14` §目标架构已定 shelf 分配器 + 1024×1024 R8 页 + padding 2 + 页级 LRU。

## 4. 目标架构

```
ShapedGlyph(glyph_id, font_id, style{size, scale, format}) →
  GlyphRasterKey { face, glyph_id, px_size_bucket, subpixel_bin, format, hinting } →
    [miss] swash rasterize(物理像素) → GlyphBitmap(R8 / RGBA8) →
      shelf alloc(按行高分桶) → page(脏矩形累积) → GPU upload(每页≤1次/帧) →
        GlyphAtlasRef { page, format, uv_min/max, bearing, px_size }
```

`GlyphAtlasSet` 持两组页(alpha/color)× 两格式(bitmap/SDF——SDF/MSDF 烘焙见 `05`,装箱共用本服务)。

## 5. 里程碑

### AT-M1 swash 栅格 + shelf 图集(替换 glyphon 自管)

实施切片:
1. `graphics/text/atlas/`:shelf 分配器、页(1024×1024,R8/RGBA8)、脏矩形上传(graph 资源节点声明 IO)、页级 LRU。
2. `graphics/text/raster/swash.rs`:swash 栅格隔离层(alpha + 彩色 emoji);bearing/px_size 提取。
3. UI 文本绘制改消费 `GlyphAtlasRef`(从 glyphon 自管 atlas 切到统一 atlas);glyphon 退为"按 atlas 坐标画 quad"或整体由 `render/14` 的 sprite 批接管。

测试:`render_text_atlas_shelf_allocates_same_height_into_one_row`、`render_text_atlas_evicts_lru_page`、`text_raster_swash_emoji_rgba_glyph`。

### AT-M2 DPI 重栅格 + subpixel + hinting

实施切片:
1. atlas key 含 `px_size_bucket`(`logical_px × scale_factor` 量化)与 `subpixel_bin`(水平 1/3 或整像素吸附);scale 变换触发重栅格(接 `editor_layout/17 §3.4`)。
2. hinting 策略:`HintingMode::{None,Vertical,Full}`(默认 Vertical,对小字号清晰);font_smoothing 开关。
3. 整像素吸附:文本/1px 边框整像素吸附(`render/14`/`editor_layout/21 §3.5`),自由内容不吸附。

测试:`text_atlas_key_rebuckets_on_scale_change`、`text_raster_subpixel_bins_distinct`、`render_text_dpi_rerasterize_at_2x_sharp`。

### AT-M3 脏矩形增量上传定稿

实施切片:
1. 启用脏矩形/脏槽增量上传(现有 `sdf_upload.rs` DirtySlots 设计落地);每页本帧新增 glyph 合并为最小覆盖矩形,单次 `write_texture`。
2. 过大字形(超页)降级:大字号走 SDF(`05`)或独立纹理。

测试:`render_text_atlas_partial_upload_merges_dirty_rects`、`text_atlas_oversized_glyph_falls_back_to_sdf`。

## 6. 工程落地细化(实施权威)

### 模块与文件落点

实现层 `zircon_runtime/src/graphics/text/`:

| 文件 | 内容 |
|------|------|
| `atlas/mod.rs` | `GlyphAtlasSet`(按 `GlyphAtlasFormat` × {alpha,color} 分组页) |
| `atlas/shelf_allocator.rs` | shelf 行分配(行高向上取整到 8px 桶,行内 x 递增,padding=2) |
| `atlas/page.rs` | 1024×1024 页(R8Unorm/Rgba8Unorm)、脏矩形合并、页级 LRU、帧戳 |
| `atlas/upload.rs` | 脏矩形→`write_texture`(graph 资源 IO);每页≤1 次/帧 |
| `raster/mod.rs` | 栅格调度:format/policy 选 swash vs SDF(05) |
| `raster/swash.rs` | **swash 唯一隔离层** —— `ScaleContext`/`Scaler`/`Render`;alpha + 彩色;出口 `GlyphBitmap` |
| `raster/policy.rs` | `raster_path_for`(承接 `ui/text/raster.rs`):按字号/格式/face 选路径 |

### 核心类型与键

```rust
pub struct GlyphRasterKey {
    pub face: InstancedFaceId,   // 含变量轴(01)
    pub glyph_id: u16,
    pub px_size_bucket: u32,     // round(logical_px × scale_factor / QUANT) × QUANT
    pub subpixel_bin: u8,        // 0..3 水平 1/3 量化(整像素吸附时恒 0)
    pub format: GlyphAtlasFormat,// AlphaMask | Sdf | Msdf | Color
    pub hinting: HintingMode,
}
pub enum HintingMode { None, Vertical, Full }
pub struct GlyphBitmap { pub size: UVec2, pub bearing: Vec2,
    pub data: Vec<u8>, pub channels: u8 /*1=R8,4=RGBA8*/ }
// GlyphAtlasRef 见 render/14(page/format/uv_min/max/bearing/px_size)
```

bevy 对照:`FontAtlasKey { font_size_bits, variations_hash, hinting, font_smoothing }` → 本仓 `GlyphRasterKey`(加 subpixel_bin + format)。

### 分辨率精度规则(接 `editor_layout/17 G2`)

1. **物理像素栅格**:栅格尺寸 = `logical_px × scale_factor`,量化到 `QUANT`(默认 1px;高频缩放场景可设更粗桶避免抖动)。
2. **scale 变即重栅格**:`scale_factor` 改 → `px_size_bucket` 变 → key miss → 重栅格;旧桶页随 LRU 自然逐出。
3. **subpixel**:水平方向 3 个 bin(0/⅓/⅔);竖排或整像素吸附时关闭(`subpixel_bin=0`)。文本基线整像素吸附避免抖动。
4. **bitmap vs SDF 边界**(`raster/policy.rs`):小字号(≤ ~32px 物理)走 bitmap(更锐利);大字号/可缩放/3D 空间文本走 SDF/MSDF(`05`,分辨率无关、省重栅格)。彩色 emoji 恒 bitmap RGBA。

### 图集分配(shelf,对照 bevy + `render/14`)

- 行高桶:glyph 高度向上取整到 8px;同桶进同 shelf 行;行内 x 递增,glyph 间 padding=2(防双线性渗色)。
- 页:1024×1024;每格式每色组上限 8 页。
- 逐出:页级 LRU——glyph 命中刷新页帧戳;页满且需新页时,逐出最旧**未被本帧引用**页,整页清空重建(glyph 映射一并失效,UE flush 风格,不逐字搬迁)。
- 过大字形:超页尺寸 → 降级 SDF 或独立纹理(`GlyphTooLarge` 对照 Fyrox)。

### 与既有路径硬切换

| 现有 | 切换 |
|------|------|
| glyphon `TextAtlas`/`SwashCache` 自管栅格装箱 | 切 `GlyphAtlasSet` + `raster/swash.rs`;glyphon 退为坐标画 quad,或由 `render/14` sprite 批接管 glyph quad |
| `ui/sdf_atlas.rs` 固定 64×64/256 槽 | 统一进 `atlas/`(SDF 页与 alpha 页同分配器,见 05);保留语义,改 shelf |
| `ui/text/raster.rs` 策略 | 迁 `raster/policy.rs`;签名保留 |

### 测试与验收清单

| 测试 | 断言 |
|------|------|
| `render_text_atlas_shelf_allocates_same_height_into_one_row` | 同高桶 glyph 同 shelf 行,x 递增,padding=2 |
| `render_text_atlas_evicts_lru_page` | 页满逐出最旧未引用页;本帧引用页不可逐出 |
| `render_text_atlas_partial_upload_merges_dirty_rects` | 本帧新增 glyph 合并为最小矩形,每页 1 次上传 |
| `text_raster_swash_emoji_rgba_glyph` | 彩色 emoji 栅格为 RGBA8,落 color 页 |
| `text_atlas_key_rebuckets_on_scale_change` | scale 1.0→2.0 致 px_size_bucket 变、key miss、重栅格 |
| `text_raster_subpixel_bins_distinct` | 3 个 subpixel bin 产不同位图;吸附模式恒 bin0 |
| `render_text_dpi_rerasterize_at_2x_sharp` | 2x 下字形物理像素=2×逻辑,非放大模糊(抓帧) |
| `text_atlas_oversized_glyph_falls_back_to_sdf` | 超页字形降级 SDF,不 panic |

里程碑命令:`cargo test -p zircon_runtime render_text_atlas --locked`、`text_raster --locked`。

## 7. 风险与回退

- glyphon 深度耦合:若一步切走 glyphon 风险大,AT-M1 可先让 glyphon 与 `GlyphAtlasSet` 并存(glyphon 仅 latin 快路径),AT-M3 后全切;但不留双布局路径(布局恒走 02/03)。
- subpixel 与 wgpu 混合:subpixel AA 需特殊 blend,V1 用整像素 + 灰度 AA(覆盖多数场景),subpixel 为 feature。

## 8. 状态与产出记录

| 日期 | 里程碑/切片 | 状态 | 产出 | 验证 | 后续 |
|------|-------------|------|------|------|------|
| 2026-06-27 | 计划建立 | planned | swash 栅格 + shelf 图集 + DPI 重栅格 + 脏上传路线;统一 UI/SDF 图集 | 文档 | AT-M1 替换 glyphon 自管;喂 05 SDF/MSDF 装箱、render/14 quad |
