---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_font_bake.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/shaders/sdf_text.wgsl
  - zircon_runtime/src/asset/assets/font.rs
  - zircon_runtime_interface/src/ui/surface/render/resolved_style.rs
design_references:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/SlateSdfGenerator.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/SlateSdfGenerator.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts/FontSdfSettings.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/FontGeometryPreprocessing.cpp
  - dev/godot/editor/import/resource_importer_dynamic_font.cpp
  - dev/godot/editor/import/dynamic_font_import_settings.cpp
  - dev/godot/thirdparty/msdfgen
  - dev/godot/modules/text_server_adv/text_server_adv.cpp
plan_sources:
  - docs/plans/zircon_runtime/text/index.md
  - docs/plans/zircon_runtime/text/04-glyph-atlas-and-rasterization.md
  - .codex/plans/UI SDF 字体真实 Bake 收束计划.md
  - docs/plans/zircon_runtime/render/14-2d-stack.md
status: planned
---

# 05 SDF / MSDF 管线(动态 + 预生成)

> 本计划把现有单通道 SDF(fontsdf)升级为 **SDF + MSDF/MTSDF** 双精度路径,并补**离线预生成**烘焙与产物格式。SDF/MSDF 分辨率无关,服务大字号、可缩放 UI、3D 空间文本与省 DPI 重栅格。承接 `.codex/plans/UI SDF 字体真实 Bake 收束计划.md`。

## 1. 目标

1. **SDF(动态)**:保留 fontsdf 单通道 R8 动态烘焙;接入 `04` 统一图集(SDF 页)。
2. **MSDF / MTSDF(动态)**:用 fdsm(纯 Rust)生成多通道距离场——**保留尖角**(单通道 SDF 圆角失真的根治);MTSDF 第四通道存真距离,支持柔和效果与精确 outline。
3. **预生成(离线)**:`zircon_build`/导入期离线烘焙字形 SDF/MSDF 图集 + 元数据(`.zsdf` 产物),运行时直接装载,免运行时烘焙开销;对齐 godot msdfgen 导入语义。
4. **渲染规则**:着色器(median-of-3 MSDF 解码、`fwidth`/`screenPxRange` 抗锯齿)、阈值、描边(outline)、阴影(drop shadow)、发光(glow)、下划线/删除线。
5. **分辨率无关 + 精度**:bake 尺寸固定(如 32–48px em),运行时按 `font_size` 缩放采样;`screenPxRange` 正确推导保证任意缩放清晰。

## 2. 现状与差距

- `ui/sdf_font_bake.rs`:fontsdf 单通道 SDF 动态烘焙(64×64 槽);空字形检测、缓存。
- `ui/sdf_atlas.rs`:LRU 256 槽图集(独立于 bitmap)。
- `ui/sdf_render.rs` + `shaders/sdf_text.wgsl`:R8 采样 + smoothstep(0.42–0.58 硬编码范围)、alpha 混合。
- 缺口:**无 MSDF**(尖角失真)、**无离线预生成**(每次运行时烘焙)、**无 outline/阴影/glow**、`screenPxRange` 未按 bake/显示尺寸正确推导(缩放抗锯齿不稳)、SDF 图集未并入 `04` 统一服务。

## 3. 参考代码

| 文件 | 应重点阅读 |
|------|-----------|
| `dev/UnrealEngine/.../Fonts/SlateSdfGenerator.cpp/.h` | UE 的 SDF/MSDF 生成器:`ESlateSdfGeneratorType`、spread/ppem、`FSlateSdfGenerator` 异步生成接口、`FGeneratedSdfData`。**本计划动态生成接口与异步路径主样板** |
| `dev/UnrealEngine/.../Fonts/FontGeometryPreprocessing.cpp` | 字形轮廓预处理(去重叠/定向修正)——MSDF 生成前的轮廓清理,fdsm 同样需要 |
| `dev/UnrealEngine/.../Fonts/FontSdfSettings.h` | `FFontSdfSettings { Spread, Ppem, ... }`、`EFontRasterizationMode::{Sdf,Msdf}`——bake 参数枚举对照 |
| `dev/godot/editor/import/resource_importer_dynamic_font.cpp` + `dynamic_font_import_settings.cpp` | `multichannel_signed_distance_field` 开关、`msdf_pixel_range`(默认 8)、`msdf_size`(默认 48)——**离线 MSDF 导入参数与产物语义权威** |
| `dev/godot/thirdparty/msdfgen` | msdfgen 算法(edge coloring、distance field、error correction)——fdsm 是其 Rust 等价,理解 median-of-3 解码与 pixel range 用 |
| `dev/godot/modules/text_server_adv/text_server_adv.cpp` | `_font_get_glyph_texture`/MSDF 采样路径、`msdf_pixel_range` 如何进 shader uniform |

**Rust/wgpu 落地**:`fdsm`(Signed/Multi-channel/Multi-channel-true distance field,纯 Rust;`generate_msdf`/`generate_mtsdf` + `correct_sign`/error correction);`ttf-parser` 取 glyph outline 喂 fdsm。着色 median-of-3:`md = median(rgb); sd = (md - 0.5) * screenPxRange; alpha = clamp(sd + 0.5, 0, 1)`(msdfgen 标准)。

## 4. 目标架构

```
glyph outline(ttf-parser) → geometry preprocess(去重叠/定向) →
  [动态] fdsm generate_msdf/mtsdf(bake px) → R8/RGBA8 → 04 atlas(SDF/MSDF 页)
  [离线] zircon_build bake → .zsdf(图集 PNG/KTX2 + 字形元数据 json) → 运行时直装 04 atlas
                                                                  ↓
  shader: sample → median-of-3 → screenPxRange AA → {fill, outline, shadow, glow}
```

格式选择(接 `04` `raster/policy.rs`):
- alpha bitmap:小字号清晰(`04`)。
- SDF(R8):大字号/简单缩放/低显存。
- MSDF(RGB)/MTSDF(RGBA):需尖角保真(图标字体、大标题、3D 文本);MTSDF 额外支持柔和 outline/glow。

## 5. 里程碑

### SM-M1 SDF 并入统一图集 + screenPxRange 定稿

实施切片:
1. fontsdf SDF 烘焙接 `04` 的 `GlyphAtlasSet`(SDF 页),退出独立 256 槽 atlas;bake 尺寸固定 + 缓存键含 bake px。
2. 着色器 `screenPxRange` 正确推导:`screenPxRange = (display_px / bake_em_px) * spread_px`;替换 `sdf_text.wgsl` 硬编码 smoothstep 范围;`fwidth` 抗锯齿。

测试:`text_sdf_screen_px_range_scales_with_font_size`、`render_text_sdf_atlas_unified_with_alpha`。

### SM-M2 MSDF / MTSDF 动态生成

实施切片:
1. `graphics/text/sdf/fdsm_gen.rs`:fdsm 隔离层,glyph outline → MSDF/MTSDF;edge coloring + error correction + 轮廓预处理。
2. MSDF 页(RGBA8)进 `04`;着色器 median-of-3 解码;MTSDF 第四通道真距离供 outline/glow。
3. `raster/policy.rs` 增 MSDF 选路(尖角字体/大字号/3D 文本)。

测试:`text_msdf_preserves_sharp_corners`(对比 SDF 圆角)、`text_msdf_median_decode_matches_msdfgen`、`render_text_msdf_3d_space_sharp_at_distance`。

### SM-M3 离线预生成烘焙

实施切片:
1. `tools/zircon_build.py` 增 font-sdf bake target:离线烘焙字形集(可指定码点子集/全 cmap)→ `.zsdf`(图集图 + 元数据)。
2. 运行时装载 `.zsdf` 直灌 `04` atlas(`GlyphAtlasFormat::{Sdf,Msdf}` 预填),命中直取免烘焙;未预生成字形回退动态生成。
3. 产物格式对齐 godot msdfgen 语义(pixel_range/size 元数据)。

测试:`text_sdf_offline_bake_roundtrip`、`text_sdf_offline_glyph_hits_skip_dynamic_gen`、产物逐字节对拍。

### SM-M4 渲染效果(outline / 阴影 / glow / 装饰线)

实施切片:
1. 着色器扩展:outline(距离阈值偏移 + 描边色)、drop shadow(偏移采样)、glow(MTSDF 真距离软衰减);material 级变体(group2)。
2. 下划线/删除线几何(承接 `editor_layout/17` 装饰)。

测试:`render_text_outline_thickness_matches_distance_offset`、`render_text_shadow_offset_correct`、`render_text_decoration_underline_geometry`。

## 6. 工程落地细化(实施权威)

### 模块与文件落点

实现层 `zircon_runtime/src/graphics/text/sdf/`:

| 文件 | 内容 |
|------|------|
| `mod.rs` | SDF/MSDF 生成调度(薄) |
| `fontsdf_gen.rs` | **fontsdf 隔离层**(承接 `sdf_font_bake.rs`):单通道 SDF |
| `fdsm_gen.rs` | **fdsm 隔离层**:MSDF/MTSDF 生成 + edge coloring + error correction |
| `geometry_preprocess.rs` | glyph outline 去重叠/定向(对照 UE `FontGeometryPreprocessing`) |
| `offline.rs` | `.zsdf` 产物读写(运行时装载) |
| `params.rs` | `SdfBakeParams { mode, bake_em_px, spread_px }`(契约,对照 godot msdf_size/pixel_range) |

着色器 `zircon_runtime/src/graphics/text/shaders/zr_text_sdf.wgsl`(`zr_` 前缀,index §8 命名;替换旧 `sdf_text.wgsl`):统一 SDF/MSDF/MTSDF 解码 + AA + 效果分支(变体 define)。

离线 bake:`tools/zircon_build.py` 增 `--targets font-sdf` 段;`tools/zircon_build_font_sdf.py`(对照既有 `zircon_build_shader_prewarm.py` 形态)+ `tools/tests/test_zircon_build_font_sdf.py`。

### 核心类型与着色

```rust
pub enum SdfMode { Sdf, Msdf, Mtsdf }
pub struct SdfBakeParams {
    pub mode: SdfMode,
    pub bake_em_px: u32,   // godot msdf_size 默认 48;固定 bake 分辨率
    pub spread_px: f32,    // godot msdf_pixel_range 默认 8;距离场范围
}
pub struct SdfGlyphData { pub size: UVec2, pub bearing: Vec2,
    pub data: Vec<u8>, pub channels: u8, pub spread_px: f32 }
```

**着色(WGSL,median-of-3 + screenPxRange)**:
```wgsl
// MSDF 解码;screen_px_range 由 CPU 按 display/bake 尺寸推导进 uniform
fn msdf_alpha(s: vec4<f32>, screen_px_range: f32) -> f32 {
    let sd = median3(s.r, s.g, s.b);             // SDF: 直接 s.r
    let d = (sd - 0.5) * screen_px_range;
    return clamp(d + 0.5, 0.0, 1.0);             // fwidth 抗锯齿亦可
}
```
`screen_px_range = max(display_px / bake_em_px * spread_px, 1.0)`(msdfgen 标准推导)——保证任意缩放下边缘 1px 抗锯齿。outline:第二阈值 `clamp(d + 0.5 + outline_px, 0, 1) - fill`;glow(MTSDF):`smoothstep(glow_range, 0, abs(true_sd))`。

### 与既有路径硬切换

| 现有 | 切换 |
|------|------|
| `sdf_text.wgsl` 硬编码 smoothstep(0.42–0.58) | 删除;`zr_text_sdf.wgsl` 用 screenPxRange 推导 |
| `ui/sdf_atlas.rs` 独立 256 槽 | 并入 `04` `GlyphAtlasSet` SDF 页 |
| `sdf_font_bake.rs` | 迁 `graphics/text/sdf/fontsdf_gen.rs`(隔离层);MSDF 走 `fdsm_gen.rs` |
| `sdf_render.rs` | quad 生成迁 `render/14` `glyph_quads.rs`;着色变体本计划定 |

### 测试与验收清单

| 测试 | 断言 |
|------|------|
| `text_sdf_screen_px_range_scales_with_font_size` | display 16/32/64px 下 screenPxRange 正比,边缘 1px AA |
| `render_text_sdf_atlas_unified_with_alpha` | SDF 页与 alpha 页同 `GlyphAtlasSet`,分组隔离 |
| `text_msdf_preserves_sharp_corners` | MSDF 字形尖角误差 < SDF(对拍角点距离) |
| `text_msdf_median_decode_matches_msdfgen` | median-of-3 解码值对 msdfgen 参考表 |
| `text_msdf_mtsdf_true_distance_channel` | MTSDF 第四通道为真距离(单调) |
| `text_sdf_offline_bake_roundtrip` | `.zsdf` 写入再读出字形元数据/像素一致 |
| `text_sdf_offline_glyph_hits_skip_dynamic_gen` | 预生成命中 → 动态生成计数 0 |
| `render_text_msdf_3d_space_sharp_at_distance` | 3D 空间文本远近均清晰(抓帧) |
| `render_text_outline_thickness_matches_distance_offset` | outline 宽 = 距离偏移×screenPxRange |
| `render_text_shadow_offset_correct` | 阴影偏移/模糊正确 |

里程碑命令:`cargo test -p zircon_runtime text_sdf --locked`、`text_msdf --locked`、`render_text_outline --locked`;离线 `python tools/tests/test_zircon_build_font_sdf.py`。

## 7. 风险与回退

- fdsm error correction 与 godot msdfgen 不逐像素一致:以"尖角保真 + 缩放清晰"为验收口径,非逐像素对拍;关键字形对拍角点距离。
- 离线产物体积:全 cmap 烘焙体积大,默认只烘项目用到的码点子集 + 编辑器 UI 字符集;运行时未命中回退动态。

## 8. 状态与产出记录

| 日期 | 里程碑/切片 | 状态 | 产出 | 验证 | 后续 |
|------|-------------|------|------|------|------|
| 2026-06-27 | 计划建立 | planned | SDF 并入统一图集 + MSDF/MTSDF 动态 + 离线预生成 + outline/阴影/glow 路线 | 文档 | SM-M1 screenPxRange 定稿;依赖 04 图集 |
