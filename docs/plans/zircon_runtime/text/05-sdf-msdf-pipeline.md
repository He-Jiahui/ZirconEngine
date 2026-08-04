---
related_code:
  - zircon_runtime/src/text/atlas/mod.rs
  - zircon_runtime/src/text/atlas/page.rs
  - zircon_runtime/src/text/atlas/page_residency.rs
  - zircon_runtime/src/text/atlas/shelf_allocator.rs
  - zircon_runtime/src/text/atlas/dirty.rs
  - zircon_runtime/src/text/atlas/upload.rs
  - zircon_runtime/src/text/raster/mod.rs
  - zircon_runtime/src/text/raster/policy.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text_pixel_snap.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/font_id_report.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/sdf_fallback.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_advances.rs
  - zircon_runtime/src/text/sdf/font_bake.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/text/sdf/font_bake/tests.rs
  - zircon_runtime/src/text/sdf/font_bake/tests/cache_generation.rs
  - zircon_runtime/src/text/font/shared.rs
  - zircon_runtime/src/text/font/database/equivalence.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas/tests/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas/tests/plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas/tests/allocation.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas/tests/cache_report.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas/tests/owner.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_upload.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_upload/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests/draw_plan.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests/shader_contract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests/layout_placement.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/tests/prepare_report.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/shaders/zr_text_sdf.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/atlas_resources.rs
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
status: in_progress
last_refined: 2026-07-14
---

# 05 SDF / MSDF 管线(动态 + 预生成)

> 本计划把现有单通道 SDF(fontsdf)升级为 **SDF + MSDF/MTSDF** 双精度路径,并补**离线预生成**烘焙与产物格式。SDF/MSDF 分辨率无关,服务大字号、可缩放 UI、3D 空间文本与省 DPI 重栅格。承接 `.codex/plans/UI SDF 字体真实 Bake 收束计划.md`。

## 1. 目标

1. **SDF(动态)**:保留 fontsdf 单通道 R8 动态烘焙;接入 `04` 统一图集(SDF 页)。
2. **MSDF / MTSDF(动态)**:用 fdsm(纯 Rust)生成多通道距离场——**保留尖角**(单通道 SDF 圆角失真的根治);MTSDF 第四通道存真距离,支持柔和效果与精确 outline。
3. **预生成(离线)**:`zircon_build`/导入期离线烘焙字形 SDF/MSDF 图集 + 元数据(`.zsdf` 产物),运行时直接装载,免运行时烘焙开销;对齐 godot msdfgen 导入语义。
4. **渲染规则**:着色器(median-of-3 MSDF 解码、`fwidth`/`screenPxRange` 抗锯齿)、阈值、描边(outline)、阴影(drop shadow)、发光(glow)、下划线/删除线。
5. **分辨率无关 + 精度**:bake 尺寸固定(如 32–48px em),运行时按 `font_size` 缩放采样;`screenPxRange` 正确推导保证任意缩放清晰。

```zircon-workflow
{
  "schema": 1,
  "workflow_id": "zircon-runtime-text-sdf-msdf-pipeline",
  "goal": "完成共享 SDF/MSDF/MTSDF 生成、图集、GPU 解码、离线产物、效果与布局一致性主链",
  "milestones": [
    {"id": "M1", "title": "动态 SDF 与统一图集", "depends_on": []},
    {"id": "M2", "title": "MSDF/MTSDF 动态生成与产品证明", "depends_on": []},
    {"id": "M3", "title": "离线预生成与运行时预填", "depends_on": []},
    {"id": "M4", "title": "效果、装饰与变换下采样", "depends_on": []},
    {"id": "M5", "title": "Native 与距离场段落布局一致性", "depends_on": []}
  ]
}
```

## 2. 现状与差距

- `ui/sdf_font_bake.rs`:fontsdf 单通道 SDF 动态烘焙(64×64 槽);空字形检测、缓存。
- `ui/sdf_atlas.rs`:SDF cache 已从固定单页扩展到 `text/atlas::GlyphAtlasSet` 的 `Sdf` page identity、共享 shelf rect、page residency/LRU 与 page-keyed upload/render 数据面；2026-07-02 已在 plan 层接入 `GlyphAtlasSet`,用共享 shelf allocator 生成 slot rect、共享 dirty-rect owner 输出 cache/upload report 数据面,同时由 `text/atlas/page_residency.rs` 持有页上限、缺页分配、oldest-unreferenced LRU eviction 与本帧引用页不可逐出的决策数据面。`SdfAtlasSlot.page_key`、`SdfAtlasCacheReport.dirty_pages` 与 `SdfAtlasUploadReport.dirty_pages` 已补齐 page-keyed dirty/upload entries,renderer 已通过 `texture_2d_array` 和 vertex `page_index` 消费 page-keyed upload commands；shelf overflow 现在使用固定 page size 并通过 `GlyphAtlasSet::reserve_page_for_format(...)` 申请 page[1+] slot；evicted/rebuilt page 现在会进入 `SdfAtlasPlan.rebuilt_pages`,并在 cache report 中输出 full-page dirty rect；allocation failure 已按 run 汇总 page-limit/oversized counts,`SdfAtlasRun.glyph_failure_reasons` 也按字符位置记录失败原因,且 `ui/text/sdf_fallback.rs` 现在持有 text prepare fallback policy,会把失败原因归并为连续 glyph fallback spans/report span counts；Horizontal LTR/no-wrap/non-justify 失败 span 已局部 native overlay；2026-07-03 `ui/sdf_advances.rs` 首段把 resolved grapheme advances 投影到当前 SDF char-run advances,并让 mixed overlay fallback span 扩展到 whole grapheme；同日 `ui/sdf_char_run.rs` 首段把 ZWJ/zero-width/Bidi format/variation selector 等 invisible format controls 保留在 run index 中但过滤出 atlas slots,且 fallback measurement 对这些 scalar 返回 0 advance；同日 `ui/text_pixel_snap.rs` 让 native glyphon `TextArea` 与 SDF horizontal/vertical draw planning 共用 device-pixel text origin,避免 frame 小数原点在两条路径上分叉。Vertical/RTL/wrap/justify 等不支持原因仍进入 fallback report,并走 whole-batch native fallback。
- `ui/sdf_render.rs` + `ui/sdf_render/{atlas_resources,material,vertices,decorations}.rs` + `shaders/zr_text_sdf.wgsl`:2026-07-14 已完成 R8 SDF / RGBA MSDF-MTSDF texture arrays、group2 dynamic material、fill/outline/derivative shadow/MTSDF glow、straight-alpha、face-derived solid decorations，以及 CPU/fragment-derived 双 `screenPxRange`。vertex flat `decode_mode` 显式选择 SDF `.r` 或 MSDF/MTSDF median RGB，MTSDF alpha 为 true distance；homogeneous clip position 支持旋转/透视产品证明。`UiTextRenderMode::{Msdf,Mtsdf}` 仍只影响 raster batch mode，复用 shared shaping/layout identity；旧 `sdf_text.wgsl` 已删除。
- 当前剩余缺口已收束到 SM-M5 paragraph parity 和长期 atlas/fallback 完整性：真实 alpha bitmap atlas 统一 GPU upload、持久化 glyph cache/residency 完整淘汰闭环、broader glyph-level mixed fallback(Vertical/RTL/justify/wrapped)、independent oversized fallback、native/SDF paragraph bbox/advance/linebreak parity，以及窗口级 editor 字体一致性 QA。MSDF/MTSDF 动态生成、`.zsdf` 离线预生成和 SM-M4 effects/decorations/transformed sampling 已有生产实现与真实 WGPU 证据；SM-M4 仍需完成一次浮点容差修正复验和综合结构/target-client gate 后才能标记 complete。
- 2026-07-02 首个竖排消费切片已让 `sdf_render.rs` 根据 `ScreenSpaceUiTextBatch.writing_mode` 在 `VerticalRl` 下沿 y 轴投影 glyph quads;这只关闭 render-path writing-mode consumption,不替代本计划的 screenPxRange、统一 atlas、MSDF/MTSDF 或离线 bake 工作。

2026-08-03 状态校准：SM-M5 的确定性源码 parity gate 已实现于 `graphics/scene/scene_renderer/ui/render/tests/parity.rs`。`text_paragraph_parity_native_vs_sdf_bbox_advance_linebreak` 逐项比较 native 与 SDF/MSDF 的 source range、行分割、frame、glyph advances 与 batch，覆盖 Latin/CJK/混排/RTL 及 bitmap/SDF 阈值两侧；`text_paragraph_parity_vertical_rl` 覆盖 VerticalRl。该实现不替代真实 WGPU framebuffer/窗口 QA，也不提前关闭 alpha atlas 硬切、residency 淘汰或 broader mixed fallback；这些仍是当前 source/managed-validation 后续项。

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
1. fontsdf SDF 烘焙接 `04` 的 `GlyphAtlasSet`(SDF 页),退出独立固定槽 atlas;fixed bake params/cache key、统一图集页 identity、shared shelf rect、dirty-rect report 数据面、GPU page partial upload、renderer texture-array page consumption、shelf overflow 多页 slot allocation、rebuilt-page full-dirty invalidation、over-cap/oversized failure reporting、per-glyph failure reason mapping、glyph-level fallback span planning、Horizontal LTR/no-wrap/non-justify mixed native overlay、unsupported mixed overlay diagnostics 与 atlas page residency/LRU 首段已完成；alpha/SDF atlas 迁移、持久化 glyph cache/residency 淘汰闭环、broader mixed fallback(Vertical/RTL/justify/wrapped)、actual oversized fallback rendering 与 MSDF/MTSDF 仍待接入。
2. 着色器 `screenPxRange` 正确推导:`screenPxRange = (display_px / bake_em_px) * spread_px`;替换 `sdf_text.wgsl` 硬编码 smoothstep 范围;`fwidth` 抗锯齿。首段已完成。

测试:`text_sdf_screen_px_range_scales_with_font_size`、`render_text_sdf_atlas_unified_with_alpha`。

### SM-M2 MSDF / MTSDF 动态生成

实施切片:
1. `text/sdf/fdsm_gen.rs`:fdsm 隔离层,glyph outline → MSDF/MTSDF;edge coloring + error correction + 轮廓预处理。
2. MSDF 页(RGBA8)进 `04`;着色器 median-of-3 解码;MTSDF 第四通道真距离供 outline/glow。
3. `raster/policy.rs` 增 MSDF 选路(尖角字体/大字号/3D 文本)。

测试:`text_msdf_preserves_sharp_corners`(对比 SDF 圆角)、`text_msdf_median_decode_matches_msdfgen`、`render_text_msdf_3d_space_sharp_at_distance`。

### SM-M3 离线预生成烘焙

实施切片:
1. `tools/zircon_build.py` 增 font-sdf bake target:离线烘焙字形集(可指定码点子集/全 cmap)→ `.zsdf`(图集图 + 元数据)。
2. 运行时装载 `.zsdf` 直灌 `04` atlas(`GlyphAtlasFormat::{Sdf,Msdf}` 预填),命中直取免烘焙;未预生成字形回退动态生成。
3. 产物格式对齐 godot msdfgen 语义(pixel_range/size 元数据)。

测试:`text_sdf_offline_bake_roundtrip`、`text_sdf_offline_glyph_hits_skip_dynamic_gen`、产物逐字节对拍。(2026-07-02 评审收口)验收口径改判:"产物逐字节对拍"仅指 **`.zsdf` 自产物写读 roundtrip 逐字节一致**(同参数两次 bake / 写后读回);与 godot 产物**不做逐字节对拍**,只比 metadata(pixel_range/size)与角点误差指标——与 §7 风险条目的口径统一。

### SM-M4 渲染效果(outline / 阴影 / glow / 装饰线)

实施切片:
1. 着色器扩展:outline(距离阈值偏移 + 描边色)、drop shadow(偏移采样)、glow(MTSDF 真距离软衰减);material 级变体(group2)。
2. 下划线/删除线几何(承接 `editor_layout/17` 装饰)。(2026-07-02 评审收口)装饰线度量定稿:下划线位置/粗细取 face `post` 表 `underlinePosition`/`underlineThickness`,删除线取 `OS/2` `yStrikeoutPosition`/`yStrikeoutSize`(01 已补这两组表解析);face 缺失对应表时按 em 比例合成(下划线位置 ≈ -0.1em、粗细 ≈ 0.05em,删除线位置 ≈ 0.3em)。V1 **不做 skip-ink**(下划线避让降部)。测试断言线位置/粗细来自 face 表而非硬编码常数。
3. (2026-07-02 评审收口)选路策略条款:请求含 outline/shadow/glow 效果的文本**强制走 SDF/MSDF 路径**(bitmap alpha 路径无距离信息,无法实现距离阈值效果)——`raster/policy.rs` 选路输入必须包含效果标志,即使字号在 bitmap 快路径阈值内也改走 SDF。

测试:`render_text_outline_thickness_matches_distance_offset`、`render_text_shadow_offset_correct`、`render_text_decoration_underline_geometry`、`render_text_decoration_metrics_from_face_tables`(2026-07-02 评审收口)、`text_policy_outline_effect_forces_sdf_path`(2026-07-02 评审收口)。

执行状态（2026-07-14，完成）：production graphics 与 target-client compile 均通过；真实 WGPU framebuffer 产品用例两次通过，输出 960×560/5113 colors/SHA256 `D0BD287F65DBABC33E78045942BB38F19A4EB7B5C2D282FC59907C922649BD59`，且 repo/coordinator target 同名副本为 0。broad `render_text_` 先通过 121/122，唯一 exact-zero 浮点断言改为 `1e-5` 后 exact 1/1，组合证明当前组 122/122；production/test file budget 与 UI child-owner split 结构门通过。SM-M4 正式关闭，继续 SM-M5 native/SDF paragraph parity。

### SM-M5 native/SDF paragraph parity 闸门(2026-07-02 评审收口)

各切片状态记录中大量 pending 的 "native/SDF paragraph parity" 在此里程碑统一收束,不再散落。

2026-07-03 首个像素原点收束切片已把 native glyphon placement 与 SDF vertex planning 接到同一 `text_pixel_snap.rs` owner；这只关闭 frame-origin phase drift,不替代本里程碑的 bbox/advance/linebreak 逐项 parity 闸门。

2026-07-03 editor retained-host subpixel spacing threshold 切片把 runtime advance projection 的 per-grapheme 接受窗收紧到 `clamp(host * 6%, 0.35px, 0.60px)`,用于阻断小字号 tab label 在总宽度接近时仍发生的局部 +0.75px/-0.75px 借位；这只关闭 retained-host 小字号 spacing 阈值,不替代 native/SDF paragraph parity。

实施切片:
1. parity 测试夹具:同一字符串、同一 `LayoutConstraints`,分别走 native bitmap 路径与 SDF/MSDF 路径,对两路径的 **bbox、每 glyph advance、换行结果(行数/每行字节区间)** 逐项断言一致(布局恒走 02/03,两路径只在栅格/上屏分叉,布局结果必须逐位相同;渲染像素只做容差抓帧)。
2. 覆盖集:拉丁/CJK/混排/RTL/竖排各至少一条;字号跨 bitmap/SDF 边界两侧。

测试:`text_paragraph_parity_native_vs_sdf_bbox_advance_linebreak`、`text_paragraph_parity_vertical_rl`。闸门:本里程碑通过前,任何声明 "paragraph parity 关闭" 的状态行无效。

## 6. 工程落地细化(实施权威)

### 模块与文件落点

实现层 `zircon_runtime/src/text/sdf/`:

| 文件 | 内容 |
|------|------|
| `mod.rs` | SDF/MSDF 生成调度(薄) |
| `fontsdf_gen.rs` | **fontsdf 隔离层**(承接 `sdf_font_bake.rs`):单通道 SDF |
| `fdsm_gen.rs` | **fdsm 隔离层**:MSDF/MTSDF 生成 + edge coloring + error correction |
| `geometry_preprocess.rs` | glyph outline 去重叠/定向(对照 UE `FontGeometryPreprocessing`) |
| `offline.rs` | `.zsdf` 产物读写(运行时装载)。(2026-07-02 评审收口)产物头必须含:format version、字体资产 GUID、face_index、variation(变量轴实例)hash——装载时四项全匹配才可用,否则视为 stale 走动态 bake;预填页被 LRU 逐出后,后续 miss **优先重读 `.zsdf`** 恢复预生成字形,重读不可用才回退动态 bake |
| `params.rs` | `SdfBakeParams { mode, bake_em_px, spread_px }`(契约,对照 godot msdf_size/pixel_range)。(2026-07-02 评审收口)当前动态路径的 32px bake em / 8px spread 为**过渡值**(见 §8 fixed bake params 切片);SM-M3 离线烘焙落地前统一为 **48/8**(对齐 godot msdf_size 默认),且 CPU 侧 screenPxRange 推导与离线 bake 参数都由 `SdfBakeParams` **单点供给**,禁止 shader 推导代码与 bake 代码各自硬编码 |

着色器 `zircon_runtime/src/text/shaders/zr_text_sdf.wgsl`(`zr_` 前缀,index §8 命名;替换旧 `sdf_text.wgsl`):统一 SDF/MSDF/MTSDF 解码 + AA + 效果分支(变体 define)。

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

(2026-07-02 评审收口)**screenPxRange 供给分两档**:
- **快路径(2D 无变换)**:UI 屏幕空间、无旋转/非均匀缩放时,CPU 按上式 per-vertex 计算 `screen_px_range` 写入顶点(现有 per-vertex 首段即此档),shader 直接消费,省 `fwidth`。
- **通用路径(带变换/3D)**:glyph quad 经任意变换(旋转/透视/3D 空间文本)时,CPU 无法预知屏幕投影尺寸,必须在 shader 内推导:`screenPxRange() = spread * length(fwidth(uv * atlas_size))`(msdfgen 参考实现),按每 fragment 计算。选档依据:批次是否携带非平移/均匀缩放变换。补抓帧用例:旋转 45° 与 3D 缩放下文本边缘仍 1px AA(`render_text_sdf_rotated_screen_px_range_sharp`、`render_text_msdf_3d_space_sharp_at_distance` 覆盖)。

### 与既有路径硬切换

| 现有 | 切换 |
|------|------|
| `sdf_text.wgsl` 硬编码 smoothstep(0.42–0.58) | 删除;`zr_text_sdf.wgsl` 用 screenPxRange 推导 |
| `ui/sdf_atlas.rs` 独立 256 槽 | 并入 `04` `GlyphAtlasSet` SDF 页 |
| `sdf_font_bake.rs` | 迁 `text/sdf/fontsdf_gen.rs`(隔离层);MSDF 走 `fdsm_gen.rs` |
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
| `render_text_decoration_metrics_from_face_tables` | (2026-07-02 评审收口)下划线取 `post.underlinePosition/Thickness`、删除线取 `OS/2 yStrikeout*`;缺表按 em 比例合成 |
| `text_policy_outline_effect_forces_sdf_path` | (2026-07-02 评审收口)含 outline/shadow 效果的请求即使小字号也选 SDF/MSDF 路径 |
| `render_text_sdf_rotated_screen_px_range_sharp` | (2026-07-02 评审收口)旋转/3D 变换下 shader 内 `screenPxRange()` 推导,边缘仍 1px AA(抓帧) |
| `text_paragraph_parity_native_vs_sdf_bbox_advance_linebreak` | (2026-07-02 评审收口,SM-M5)同串同布局两路径 bbox/每 glyph advance/换行逐项一致 |
| `text_paragraph_parity_vertical_rl` | (2026-07-02 评审收口,SM-M5)竖排下 native/SDF parity 同口径断言 |

里程碑命令:`cargo test -p zircon_runtime text_sdf --locked`、`text_msdf --locked`、`render_text_outline --locked`;离线 `python tools/tests/test_zircon_build_font_sdf.py`。

## 7. 风险与回退

- fdsm error correction 与 godot msdfgen 不逐像素一致:以"尖角保真 + 缩放清晰"为验收口径,非逐像素对拍;关键字形对拍角点距离。
- 离线产物体积:全 cmap 烘焙体积大,默认只烘项目用到的码点子集 + 编辑器 UI 字符集;运行时未命中回退动态。

## 8. 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

当前概述（2026-07-14）：SM-M5 layout identity 与真实 Native/SDF 产品像素门已通过；SM2-M1 shared pure-Rust fdsm core 已验收。SM2-M2/M3 已把 authoritative face/glyph id 动态生成、mode-keyed R8/RGBA atlas、format-aware upload、typed per-glyph fallback、flat GPU decode mode、双 storage texture arrays 与显式 `UiTextRenderMode::{Msdf,Mtsdf}` 接到生产路径。当前产品门分别验证两路真实尖角像素、distinct decode 与 MSDF apex 不低于 SDF，bake-space 几何仍由 renderer-neutral fdsm 回归负责；managed current-source compile 与 exact real WGPU exporter 通过，产出 `docs/tests/runtime/text/runtime_text_multilingual_sdf_msdf_product_framebuffer_20260714.png`（1080×1690、321453 bytes、2442 colors、SHA256 `2A033D76EF5C16F99FB6B256AD8F480ACE494FB03537A9E4502DEA293BED866E`，target 同名 0）。SM3-M1/M2/M3 已实现 deterministic embedded-page `.zsdf`、feature-gated `font-sdf` target/CLI、project `.zmeta` UUID identity、runtime prefill 与 dynamic fallback；artifact/renderer exact 6/6、CLI range 1/1、独立三模式 deterministic/decode 2/2、Python 5/5 与既有 build-tool regressions 45/45 均通过。2026-07-14 按 Runtime02 failure 将 build tool 从未裁决 crate-root seat 硬切到 `zircon_runtime::graphics::text::font_sdf_build_tool`，不保留旧路径或兼容转发；feature integration 2/2、CLI check、fresh default Runtime no-run、`generated` 29/29、`core::` 705/705、structure-convention 1304/1304 通过，root audit 恢复 19/19、0 debt、0 risk。Text05 M2 当前产品证明已由协调器验收；整体 Text05 保持 `in_progress`，继续长期 atlas/fallback 完整性和 Text01–09 剩余架构审计。

2026-07-17 Text MVP 稳定性切片已把 generation-sensitive SDF cache 回归拆到 `font_bake/tests/cache_generation.rs`，并用 test-only shared snapshot read guard 隔离并行全局字体发布；生产 SDF 路径不新增锁。根因修复位于 Text01/09 的共享字体发布 owner：等价 FontDatabase 不再推进 generation，因此 renderer 构造顺序不会清空 resident SDF fonts/glyphs。`measure_key(...)` 在 ensure/map 异常不一致时也会继续候选并最终返回既有 fallback metrics，而不是 production panic。旧串行 SDF 20/20 证明功能本体正常；当前源码并行 20 项与真实 framebuffer 门仍在共享 Cargo 队列，故本切片记录为 `implemented / validation_pending`，不宣称 Text05 新里程碑完成。

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

2026-08-01 performance failure implementation advance：Text05 非验收实现与二次审查已收敛 generation-owned parsed source、runtime/offline deterministic batch generator、bounded async scheduler、completion/panic 前向重试、bounded source/offline/baked caches、run-shared `Arc<str>` glyph identity、single `SdfAtlasBake` failure/render artifact、compiled atlas/CPU/vertex-material frame caches、persistent CPU atlas dirty pages、page-local borrowed upload 与 capacity/hash managed vertex buffer。稳定帧跳过 key/slot transition、CPU metrics、failure map、bake metadata、material/draw/vertex build 和 GPU write；pending/deferred/font reload/atlas upload/device recreate 均 fail closed 到重建。二次审查累计前向修复永久 pending、逐 glyph String 深拷贝、无界 resident caches、stable metadata/failure-map 分配、no-fallback Vec 搬移与 renderer generation 双 owner；production owner 全部低于 800 行且禁用模式扫描为 0。两份 2026-07-18 failure 保持 `open / non_validation_implementation_complete / secondary_review_complete / managed_validation_pending`：managed Cargo、1/100/10k 规模 p50/p95/RSS、reload/device-loss、真实 WGPU/RenderDoc 与新截图仍待 coordinator receipt。本轮不等待/轮询协调器，也未向 `target` 或 `docs/tests/runtime/text` 写入伪验证图。

- 迁入记录：[`05/2026-07-09-sdf-msdf-pipeline-output-records.md`](05/2026-07-09-sdf-msdf-pipeline-output-records.md)
- fixed 已修复：[ui-text-distance-field-effects-type-resolution](../../zircon_editor/editor_layout/15/fixed-2026-07-13-ui-text-distance-field-effects-type-resolution.md)
- fixed 已修复：[font-sdf-build-tool-root-surface-drift](../runtime/02/fixed-2026-07-14-font-sdf-build-tool-root-surface-drift.md)
- fixed 已修复：[sdf-font-bake-cjk-loaded-font-count-regression](../../zircon_editor/editor/02/fixed-2026-07-15-sdf-font-bake-cjk-loaded-font-count-regression.md)
- 2026-07-18 SDF upload性能交接：多页command原对每dirty page重建page-key BTreeSet/Vec并逐页累计source offset，GPU write还clone完整dirty report；已改一次有序source-page table+binary lookup并借用report。Text05后续让generation atlas page直接发布offset metadata，消除剩余page()线性投影；stable command/upload=0。见PERF-MVP-249及graphics UI font/SDF upload静态证据。
- 2026-07-18 SDF advance fallback补充交接：grapheme→character advance映射原做整串chars预扫、grapheme Vec物化、逐grapheme再数chars与sanitize二次扫描；已改双iterator单stream并边sanitize边判nonzero。Text05最终直接消费compiled shaping advances，同generation不再重复映射。见PERF-MVP-249及UI root小文件静态证据。
- 2026-07-18 scene UI SDF核心交接：`sdf_atlas` 7/7、`sdf_render` 15/15、`text` 8/8文件确认stable keys仍全量重建glyph key/string、BTree set/map、shelf/slot/run/transition report；generation failure、fallback advance与final renderer重复prepare CPU runs，随后重建material/6-vertex glyph计划和GPU buffer。本轮scalar-count Vec=0、identical material uniform upload=0、report分类5扫→2扫；Text05必须以single generation artifact共享CPU run与persistent key→slot/page allocator，见PERF-MVP-249及UI text/SDF core证据。
