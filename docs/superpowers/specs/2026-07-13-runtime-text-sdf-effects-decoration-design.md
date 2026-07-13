---
related_code:
  - zircon_runtime_interface/src/ui/surface/render/resolved_style.rs
  - zircon_runtime_interface/src/ui/surface/render/text_effects.rs
  - zircon_runtime_interface/src/ui/surface/render/text_shape.rs
  - zircon_runtime/src/graphics/text/raster/policy.rs
  - zircon_runtime/src/graphics/text/font/decoration_metrics.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/material.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render/vertices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/shaders/zr_text_sdf.wgsl
plan_sources:
  - docs/plans/zircon_runtime/text/05-sdf-msdf-pipeline.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
design_references:
  - dev/godot/servers/rendering/renderer_rd/shaders/canvas.glsl
  - dev/godot/servers/rendering/renderer_rd/renderer_canvas_render_rd.cpp
  - dev/godot/scene/resources/material.cpp
  - dev/godot/scene/3d/label_3d.cpp
  - dev/godot/tests/scene/test_fontfile.cpp
  - dev/bevy/crates/bevy_sprite_render/src/text2d/mod.rs
  - dev/bevy/crates/bevy_text/src/pipeline.rs
  - dev/bevy/crates/bevy_sprite/src/text2d.rs
  - dev/bevy/examples/ui/text/strikethrough_and_underline.rs
  - dev/slint/internal/core/textlayout/sharedparley.rs
  - dev/slint/api/rs/slint/tests/partial_renderer.rs
status: accepted
---

# Runtime Text05 SM-M4 SDF 效果、装饰线与变换采样设计

## 1. 范围与结论

本设计关闭 `05-sdf-msdf-pipeline.md` 的 SM-M4：outline、drop shadow、MTSDF glow、下划线、删除线，以及旋转/透视情况下的 fragment-derived `screenPxRange`。不新增独立字体图集、不复制 shaping/layout、不把编辑态 selection/caret/composition 装饰冒充排版样式。

最终采用以下结构：

1. `zircon_runtime_interface` 提供独立、可序列化、可归一化的 `UiTextDistanceFieldEffects` 与 `UiTextDecorations`；`UiResolvedStyle -> UiTextPaint -> ScreenSpaceUiTextBatch` 只转交值，不包含 GPU 逻辑。
2. `graphics/text/raster/policy.rs` 继续作为选路唯一权威。有效 outline/shadow/glow 强制距离场；glow 需要真距离，因此选择 MTSDF；颜色字形和 subpixel bitmap 仍保持 bitmap 路径。
3. `ui/sdf_render/material.rs` 持有 group 2 材质 ABI、动态 uniform 对齐、draw range。group 0 仍只持有 R8/RGBA atlas；group 1 保留为 view/transform 槽，禁止把 effect 参数塞进 atlas owner 或逐顶点重复。
4. `graphics/text/font/decoration_metrics.rs` 从已解析 face 或 `ttf-parser` face bytes 得到 underline/strikeout font-unit metrics，并按字号缩放。缺表时才使用 -0.1em/0.05em/0.3em 回退；V1 不做 skip-ink。
5. 无旋转屏幕 UI 保留 CPU per-vertex `screen_px_range` 快路径；旋转、非均匀缩放或透视使用 fragment UV derivatives。通用公式采用 Godot/MSDF 实现的 `0.5 * dot(pixel_range / atlas_size, 1 / fwidth(uv))`，它是计划中“由 UV 导数推导”的量纲正确形式。

## 2. 上游证据

### Godot：效果参数是批次/材质状态，变换后范围由导数恢复

- `renderer_canvas_render_rd.cpp` 在 `use_msdf`、pixel range 或 outline 改变时切新 batch，说明效果参数不是 atlas identity，也不应逐字形改变。
- `canvas.glsl` 和 `material.cpp` 使用 median-of-3，并以纹理尺寸及 `1 / fwidth(uv)` 计算屏幕 pixel range；`label_3d.cpp` 把 MSDF pixel range 和 outline size 放入 3D material。
- `test_fontfile.cpp` 同时验证 10px 与 100px 的 underline position/thickness，证明装饰线度量必须随字号缩放，不能固定 1px。

### Bevy：样式、run geometry、渲染抽取分层

- `bevy_text::RunGeometry` 独立保存 underline/strikeout 的 y 与 thickness；渲染抽取按 run 产生装饰矩形。
- `Text2dShadow` 是文本表现属性，抽取阶段生成 shadow draw，不污染 atlas key。
- 示例覆盖嵌套 span、独立颜色、跨字号 underline/strikeout，支持 Zircon 继续使用 run/batch 边界而不是全段硬编码。

### Slint：样式语义与脏区/越界范围必须独立验证

- styled text 将 underline/strikethrough 作为 span style 传给文本布局后端。
- `shadow_redraw_beyond_geometry` 明确验证 shadow offset 与扩展范围，提示 Zircon 的裁剪和产品测试必须覆盖字形原始几何之外的有效效果范围。

## 3. 基础能力充分性

现有基础可复用，但有三个缺口：

- `GlyphRasterEffects` 已能表达选路意图，却没有从公共 style/paint DTO 接入生产渲染。
- `FontAssetFaceMetrics` 已保存 post/OS/2 解析结果，却没有统一的 display-pixel 装饰度量 owner。
- SDF renderer 只有单一 atlas bind group 和单 draw，无法承载 material 级效果。

因此不新增具体字体名、语法名或测试专用分支；只补通用效果 DTO、通用 face decoration metrics、通用 material draw plan。未来 gradient fill、inner shadow、3D text consumer 可沿用这些契约。

## 4. 数据契约

### 4.1 公共表现类型

```rust
pub struct UiTextDistanceFieldEffects {
    pub outline: Option<UiTextOutlineEffect>,
    pub shadow: Option<UiTextShadowEffect>,
    pub glow: Option<UiTextGlowEffect>,
}

pub struct UiTextOutlineEffect { pub width_px: f32, pub color: String }
pub struct UiTextShadowEffect {
    pub offset_x_px: f32,
    pub offset_y_px: f32,
    pub color: String,
}
pub struct UiTextGlowEffect { pub radius_px: f32, pub color: String }

pub struct UiTextDecorations {
    pub underline: bool,
    pub strikethrough: bool,
    pub underline_color: Option<String>,
    pub strikethrough_color: Option<String>,
}
```

所有尺寸进入 renderer 前必须有限、非负并受统一上限约束；无效或透明效果归一化为 disabled。effect presence 由归一化后的值决定，不能只看 `Option::is_some()`。

### 4.2 内部材质与投影模式

`SdfTextMaterial` 是 CPU 语义值；`SdfTextMaterialUniform` 是 16-byte 对齐 GPU ABI。材质包含 fill/outline/shadow/glow colors、outline width、shadow offset、glow radius、effect flags 与 projection mode。每个文本 batch 产生一个 draw range，相邻完全相同材质可合并，但不得跨 atlas decode mode 错并。

`SdfScreenPxRangeMode`：

- `CpuScreenSpace`：消费 vertex `screen_px_range`。
- `FragmentDerived`：消费 vertex atlas pixel range，并从当前 atlas dimensions + UV derivatives 推导。

## 5. Shader 语义

1. fill：SDF `.r` 或 MSDF/MTSDF RGB median。
2. outline：对 fill signed distance 增加 `outline_width_px` 后求 coverage，outline 层为 expanded coverage 减 fill coverage。
3. shadow：在当前 fragment 使用 `uv - dpdx(uv) * offset_x - dpdy(uv) * offset_y` 做屏幕像素偏移采样；这对旋转 quad 仍保持屏幕方向正确。
4. glow：只在 MTSDF alpha true distance 上计算外侧软衰减；非 MTSDF 材质的 glow 必须在 CPU 选路阶段升级，而不是 shader 猜测。
5. 合成顺序：glow -> shadow -> outline -> fill，使用显式 straight-alpha over；输出继续匹配现有 `ALPHA_BLENDING`。

效果宽度和偏移最终还要受当前 glyph 可用 distance range 限制，避免读取 spread 之外的无定义距离。

## 6. 装饰线语义

`TextDecorationMetrics` 保存以 baseline 为原点、字体 y-up 的 position 与 thickness。屏幕 y-down 转换：

- underline center y = `baseline - underline.position_px`；负 position 自然落到 baseline 下方。
- strikeout center y = `baseline - strikeout.position_px`。
- rectangle top = center y - thickness / 2，thickness 至少保持一个可见 device pixel，但测试仍断言原始 face 比例。

同一 run 若含 fallback face，取参与字形 face 中能覆盖完整可见范围的最大 thickness，并采用 primary run face position；这与 Godot shaped-text 聚合语义相容。V1 不做 skip-ink。

## 7. 裁剪、性能与所有权

- effect quad 使用 bake spread 已包含的 padding；clip 仍以 command clip + viewport 为最终边界，不能先按未扩展字形 bbox 丢弃阴影。
- 材质 uniform 使用一个动态 buffer 和一个 group 2 bind group；draw 只切换动态 offset，避免每 batch 创建 buffer/bind group。
- effect 不进入 glyph/atlas cache key；只有 SDF mode 继续进入 bake identity。
- decoration metrics 可按 `(FontFaceId, font_size bits)` 缓存；不在 hot fragment path 解析字体表。
- production module 保持文件夹化：公开 DTO、font metrics、material ABI、vertex planning、shader 各自单责，禁止继续扩大 `render.rs`/`sdf_render.rs`。

## 8. 测试翻译矩阵

| 层 | 必须覆盖 |
| --- | --- |
| DTO | serde default、非有限值/负值归一化、效果/装饰独立于编辑态 decoration |
| Policy | 小字号 outline/shadow 强制 SDF；glow 强制 MTSDF；无效果保持原策略 |
| Font | post/OS/2 face table 缩放、缺表 em 回退、不同字号比例 |
| Draw plan | material 边界产生 draw ranges、动态 offset 对齐、相同材质可合并 |
| Shader | WGSL parse、median/true-distance、outline threshold、shadow derivative offset、fragment range 公式 |
| Runtime | fill/outline/shadow/glow 像素区域与颜色断言、underline/strikeout 几何断言 |
| Product | 真实 WGPU framebuffer 同时展示效果、face-derived 装饰、45° 旋转和透视缩放文本；PNG 仅写 `docs/tests/runtime/text` |

边界覆盖：空文本、零宽效果、透明色、NaN/Infinity、极大 offset/radius、单 glyph/多 run、fallback face、裁剪边界、重复 prepare、SDF/MSDF/MTSDF 三模式。压力近似采用多 batch material buffer 与重复 prepare；当前仓库没有正式 3D text scene API，因此 3D 验证由同一生产 shader/material/vertex ABI 的真实透视 WGPU 产品夹具完成，不伪造新的 scene API。

