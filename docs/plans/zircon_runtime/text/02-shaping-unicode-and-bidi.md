---
related_code:
  - zircon_runtime_interface/src/ui/surface/render/resolved_style.rs
  - zircon_runtime/src/text/model/shaped_run.rs
  - zircon_runtime/src/ui/surface/render/resolve.rs
  - zircon_runtime/src/ui/text/resolved_layout.rs
  - zircon_runtime/src/ui/text/shaper.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/layout_engine/visual_order.rs
  - zircon_runtime/src/ui/text/grapheme.rs
  - zircon_runtime/src/text/mod.rs
  - zircon_runtime/src/text/layout/mod.rs
  - zircon_runtime/src/text/layout/measure.rs
  - zircon_runtime/src/text/shaping/mod.rs
  - zircon_runtime/src/text/shaping/bidi.rs
  - zircon_runtime/src/text/shaping/cosmic.rs
  - zircon_runtime/src/text/shaping/cosmic/font_system_cache.rs
  - zircon_runtime/src/text/shaping/horizontal/backend.rs
  - zircon_runtime/src/text/shaping/horizontal/projection.rs
  - zircon_runtime/src/text/shaping/horizontal/tests.rs
  - zircon_runtime/src/text/shaping/vertical.rs
  - zircon_runtime/src/text/shaping/vertical/backend.rs
  - zircon_runtime/src/text/shaping/vertical/orientation.rs
  - zircon_runtime/src/text/shaping/vertical/projection.rs
  - zircon_runtime/src/text/shaping/normalize.rs
  - zircon_runtime/src/text/shaping/script_segment.rs
  - zircon_runtime/src/text/shaping/fallback_spans.rs
  - zircon_runtime/src/text/font/backend.rs
  - zircon_runtime/src/text/shaping/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime_interface/src/ui/surface/render/mod.rs
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime_interface/src/ui/surface/render/text_layout.rs
  - zircon_runtime_interface/src/ui/surface/render/text_shape.rs
  - zircon_runtime_interface/src/tests/render_contracts.rs
  - zircon_runtime/Cargo.toml
design_references:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/SlateTextShaper.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/SlateTextShaper.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/FontCacheHarfBuzz.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts/FontCache.h
  - dev/godot/modules/text_server_adv/text_server_adv.cpp
  - dev/godot/modules/text_server_adv/script_iterator.cpp
  - dev/godot/servers/text/text_server.h
  - dev/bevy/crates/bevy_text/src/pipeline.rs
  - dev/slint/internal/core/textlayout/shaping.rs
  - dev/slint/internal/core/textlayout/glyphclusters.rs
plan_sources:
  - docs/plans/zircon_runtime/text/index.md
  - docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
  - docs/plans/zircon_runtime/render/14-2d-stack.md
status: in_progress
---

# 02 文本整形 / Unicode / BIDI / 竖排

> 本计划把"码点串 + 样式 + face"整形为"定位字形序列 `ShapedGlyphRun`"——文本主链核心。承接 `editor_ui/03 §2.2` 缺口 1(shaping 权威未定)。它产出的真实字形度量是 `03`(换行/度量)与 `editor_layout/17 G1`(度量=绘制)的唯一数据源。

## 1. 目标

1. **整形权威定稿**:cosmic-text(内置 rustybuzz)做 GSUB/GPOS 整形——连字、kerning、上下文替换、标记定位;glyphon 退为 bitmap 绘制后端,不再"挂名未接"。
2. **Unicode 完整支持**:grapheme cluster、script 检测分段、规范化(NFC)、组合字符、变体选择符(VS15/16)、emoji ZWJ 序列、控制字符处理。
3. **BIDI(UAX#9)**:段落级方向解析、level run 切分、视觉序重排、镜像字符(括号/箭头)、`base_direction = Auto|Ltr|Rtl`。
4. **竖排**:朝向枚举(对齐 godot `Orientation::{Horizontal,Vertical}`);竖排主轴 advance、baseline 居中、横排正字(`upright`)/旋转(`mixed`/`sideways`)模式;CJK 标点竖排形。
5. **cluster→source 映射**:每字形携 `source_range`(源文本字节区间),供命中测试/光标/选区/IME 精确反查。

```zircon-workflow
{
  "schema": 1,
  "workflow_id": "zircon-runtime-text-shaping-unicode-bidi",
  "goal": "完成共享文本整形、Unicode/BIDI 与竖排字形数据主链",
  "milestones": [
    {"id": "M1", "title": "整形后端接入与启发式硬切", "depends_on": []},
    {"id": "M2", "title": "Unicode、script segmentation 与 BIDI", "depends_on": []},
    {"id": "M3", "title": "竖排 shaping 与 glyph orientation", "depends_on": []}
  ]
}
```

## 2. 现状与差距

- `text/shaping/cosmic.rs` 已是共享整形 owner，输出真实 glyph id/advance/source range/cluster flags/script；UI measurement/layout 可复用 shaped-run cache，不再以全局等宽近似作为权威度量。
- `text/shaping/bidi.rs` 已用 `unicode-bidi` 落地段落基方向、isolate 与逻辑序 glyph 的 `bidi_level`，并提供断行后 L1/L2 visual index mapping。Text 03 已把旧 `layout_engine/visual_order.rs` 的脚本范围/neutral span/mirror table 硬切到共享 owner，并以完整段落+绝对行 range 保留 wrapped isolate context；hit-test/caret 仍需直接消费 visual index/source cluster，避免长期物化 visual line text。
- grapheme/ZWJ/VS 已进入 cluster/source range 与 script-tag 数据面；V1 NFC 按计划默认关闭，`normalize.rs::ShapingTextView` 已显式承接 identity shaping view 与 shaping→原文 source byte range 投影，cosmic/fallback 都经该 owner。未来启用 NFC 时仍需把 identity 投影升级为完整 pre↔post 双向映射。
- script segment tag 已落地，但 actual per-script shaping/fallback face reconciliation 仍由 Text 02/06 后续完成。
- `TextShapeRequest` 已携 language 与排序去重的 OpenType features，二者进入 shaped cache key；features 已传入 cosmic backend。language 已从可序列化 `UiResolvedStyle.language` 经模板解析、layout/shaped cache、direct/parallel request、native batch 与 SDF atlas/bake fallback 贯通；独立 `cosmic/font_system_cache.rs` 规范化并选择最多四个 locale-specific `FontSystem`。cosmic-text 0.18.2 的 `Attrs` 不暴露 language，因此 folder-backed `shaping/horizontal/` leaf 现按 cosmic 实际 face/cluster 分段交给 RustyBuzz，并在 language 存在时设置 per-run language 触发默认 `locl`；真实 Windows `Calibri` 俄语/塞尔维亚语 glyph 差异 exact 已落代码但尚未执行，不计完成。
- SH-M3 V1 已推进：`vertical.rs` 投影 cluster-head y advance/列中线，`vertical/orientation.rs` 按 Unicode Vertical_Orientation 实现 Mixed/Upright/Sideways，cosmic 请求显式启用 `vert`/`vrt2`，且 UI VerticalRl 的测量/换行 provider 已硬切到 vertical request/cache key。2026-07-10 又修复生产 SDF consumer 并用真实 `竖排布局` CJK 帧验收；随后 `font/vertical_metrics.rs` 从实际 backend face/glyph 读取 `vmtx`，upright 优先原生 advance、sideways 保持横排 advance、缺表回退 em。TTB/BTT backend direction、VORG/side-bearing 已落地；CJK 标点完整黄金图仍待后续。horizontal language leaf 显式拒绝 vertical request，竖排 language 继续由 TTB/BTT backend 直接设置。

## 3. 参考代码

| 文件 | 应重点阅读 |
|------|-----------|
| `dev/UnrealEngine/.../Fonts/SlateTextShaper.cpp` | `ShapeBidirectionalText`(ICU bidi 拆方向)→ `ShapeUnidirectionalText`(按 script/font 分段)→ HarfBuzz `hb_shape`;`FinalizeTextShaping` 建源索引→字形映射;`PerformKerningOnlyTextShaping`(ASCII 快路径)。本计划整形流水主样板 |
| `dev/UnrealEngine/.../Fonts/FontCacheHarfBuzz.cpp` | `hb_font_t` 创建、`hb_buffer` 方向/script/language 设置、cluster 读取——cosmic-text 内部等价,理解 cluster 语义用 |
| `dev/UnrealEngine/.../Fonts/FontCache.h` | `FShapedGlyphEntry { GlyphIndex, SourceIndex, NumCharactersInGlyph, NumGraphemeClustersInGlyph, TextDirection, bIsVisible }`——本计划 `ShapedGlyph.source_range` 即 `SourceIndex + NumCharactersInGlyph` 区间化 |
| `dev/godot/modules/text_server_adv/text_server_adv.cpp` | `shaped_text_shape`:HarfBuzz + ICU bidi(`ubidi_*`)+ script run;竖排 `_shape_run` 的 `Orientation` 分支与 `vertical` advance;`gr_features`(OpenType features)。竖排与 BIDI 权威 |
| `dev/godot/modules/text_server_adv/script_iterator.cpp` | `ScriptIterator`:Unicode script 分段(common/inherited 归并),本计划脚本分段算法直接对照 |
| `dev/godot/servers/text/text_server.h` | `Glyph { flags: GRAPHEME_IS_RTL/VIRTUAL/SPACE/... }`、`Orientation`、`Direction`——字形标志位与朝向枚举样板 |
| `dev/bevy/crates/bevy_text/src/pipeline.rs` | parley `break_and_shape` → `PositionedLayoutItem::GlyphRun` 的 Rust 落地;`source_range`/`text_byte_offset` 取法 |
| `dev/slint/.../textlayout/{shaping,glyphclusters}.rs` | `TextShaper` trait、`Glyph { text_byte_offset }`、glyph cluster 分组——轻量 Rust 形态 |

**Rust/wgpu 落地**:cosmic-text `BufferLine::shape`/`ShapeLine`/`ShapeRun`/`ShapeGlyph`(已内置 bidi + script run + rustybuzz);`unicode-bidi`(若需独立 BIDI)、`unicode-script`、`icu_normalizer`(NFC,可选)。

## 4. 目标架构

归属:契约层 ``text/model/shaped_run.rs` 数据模型 + `core/framework/text/layout_service.rs` 中立服务契约`(已由 `render/14` 定稿,本计划扩展朝向/标志位);实现层 `text/shaping/`(`cosmic_text` 隔离于 `cosmic.rs`)。

```
TextShapeRequest { text, style, base_direction, orientation, language, features }
  └─ normalize(内部视图,可选) → bidi(UAX#9, level runs, 逻辑序) → script segment →
     per-run shape(cosmic-text/rustybuzz, font fallback 交 06) →
     ShapedGlyphRun { glyphs[ShapedGlyph(逻辑序, 携 bidi_level)], base_direction, orientation }
```

**换行归属裁决(D1,2026-07-02 评审收口)**:02 只交付"无宽度约束整形 + 断点机会标注",**不产出行**;行切分/重排归 03 自研贪心断行。cosmic-text `Buffer::set_size` + `layout_runs` 仅作对拍参考,不是主链。shaped cache 键不含 wrap(见 D6/09)。

**BIDI 归属裁决(D2,2026-07-02 评审收口)**:02 输出**逻辑序** glyph + 每 glyph `bidi_level: u8`;UAX#9 L1/L2 行级视觉重排由 03 在断行后调用本计划 `bidi.rs` 的 per-line reorder API 完成(重排必须在行边界确定后进行,这是 UAX#9 的规范要求)。上图不再含 "visual reorder" 阶段。

整形与换行的关系:cosmic-text 一次 `shape` 同时给 cluster 与断点机会;`03` 在 `ShapedGlyphRun` 之上做行切分与对齐(本计划交付**无宽度约束的**整形 + 断点机会标注,`03` 消费)。

## 5. 里程碑

### SH-M1 整形后端接入(替换启发式)

实施切片:
1. `text/shaping/cosmic.rs` 隔离层:`TextShapeRequest → cosmic-text Buffer → ShapedGlyphRun`;cluster→`source_range`;断点机会位标注(交 03)。
2. `ShapedGlyph` 扩展:`source_range`、`cluster_flags`(RTL/space/mandatory-break/whitespace,对齐 godot `GraphemeFlag`)、命中 `font_id`(回退后实际 face)。
3. `ui/text/shaper.rs` 适配:`UiTextShaperStack` 改持 `&dyn TextShapingService`,`shape_text` 投影 `ShapedGlyphRun → UiResolvedTextLayout`;删除 `UiTextBackendIntent`/`active_layout_backend_for_intent`/`fallback_reason_for_backend`(`render/14` 硬切换清单 #1)。

测试:`text_shape_latin_kerning_matches_face_metrics`、`text_shape_ligature_fi_single_glyph`、`text_shape_clusters_map_source_ranges_monotonic`。

### SH-M2 Unicode 与 BIDI

实施切片:
1. script 分段(`ScriptIterator` 对照,common/inherited 归并到邻接 script);per-script run 整形。
2. BIDI:`base_direction = Auto` 走首强字符规则;level run 切分 + 视觉重排 + 镜像字符表(括号/箭头);RTL run 内字形逆序。
3. 规范化(NFC,可选 feature)、变体选择符、emoji ZWJ 序列保簇、控制字符不可见标志。

测试:`text_bidi_mixed_ltr_rtl_visual_order_matches_uax9`、`text_bidi_mirrors_paren_in_rtl`、`text_bidi_mirrors_arrow_in_rtl`、`text_shape_emoji_zwj_sequence_single_cluster`、`text_script_segmentation_arabic_latin_runs`、`text_script_segmentation_keeps_emoji_zwj_sequence_as_emoji_script`。

### SH-M3 竖排

实施切片:
1. `Orientation::Vertical` + `VerticalMode::{Upright,Mixed,Sideways}`;竖排主轴 advance(`vmtx`/合成)、baseline 居中。
2. CJK upright 正字、拉丁 sideways 旋转 90°、标点竖排形(`vert`/`vrt2` GSUB 若 face 提供,否则合成);`ShapedGlyph.rotation` 字段。

测试:`text_vertical_cjk_upright_advances_on_y`、`text_vertical_latin_sideways_rotated`、`text_vertical_punctuation_centered`。

## 6. 工程落地细化(实施权威)

### 模块与文件落点

实现层 `zircon_runtime/src/text/shaping/`:

| 文件 | 内容 |
|------|------|
| `mod.rs` | `SharedTextShapingService::shape` 装配(薄) |
| `cosmic.rs` | **`cosmic_text` 唯一隔离层** —— Buffer/ShapeLine/ShapeRun/ShapeGlyph 只在此;出口 `ShapedGlyphRun` |
| `script_segment.rs` | Unicode script 分段(`unicode-script`,common/inherited 归并;对照 godot `ScriptIterator`) |
| `bidi.rs` | UAX#9 包装(cosmic-text 内置 or `unicode-bidi`):level run、视觉重排、镜像字符表 |
| `vertical.rs` | 竖排:朝向解析、主轴 advance、字形旋转决策、标点居中 |
| `normalize.rs` | NFC 规范化(可选 feature `text-normalize`)。修订(2026-07-02 评审收口):**V1 不启用规范化**(feature 默认关闭,整形直接消费原文);若后续启用,NFC 仅作为整形内部视图,`source_range` **恒指规范化前原文**,本文件须交付 pre↔post offset 双向映射并在投影时换算,禁止把规范化后 offset 泄漏到契约层 |
| `cache.rs` | shaped run 缓存(`ShapedTextCacheKey → Arc<ShapedGlyphRun>`,见 09) |

### 契约扩展(回填 `render/14` 的 `shaped_run.rs`/`shaping_service.rs`)

```rust
// ShapedGlyph 扩展(在 render/14 既定字段上加;2026-07-02 评审收口按 D1/D2/D3/D4 修订)
pub struct ShapedGlyph {
    pub glyph_id: u16,
    pub font_id: FontFaceId,        // D4:整形后端实际选择的 base face；见下"font_id 权威通路"
    pub font_instance_id: Option<InstancedFaceId>, // 2026-07-13:有效变量轴实例 identity；不得替代 base face
    pub source_range: (u32, u32),   // cluster→源字节区间(单调,覆盖完整;恒指规范化前原文)
    pub offset: Vec2,
    pub advance: f32,               // 横排=x 进格;竖排=y 进格
    // D1:删 line_index —— 02 不产出行,行归属由 03 断行后建立
    // D3:删 atlas: GlyphAtlasRef —— 契约不得内嵌 atlas 槽位引用
    pub direction: TextDirection,
    pub bidi_level: u8,             // D2 新增:UAX#9 嵌入层级;03 断行后按行调用 bidi.rs per-line reorder
    pub cluster_flags: ClusterFlags,// 新增
    pub rotation: GlyphRotation,    // 新增:None | Cw90(竖排 sideways);Ccw90 预留(V2)
}
bitflags! { pub struct ClusterFlags: u16 {  // 对齐 godot GraphemeFlag
    const RTL = 1; const SPACE = 2; const WHITESPACE = 4;
    const MANDATORY_BREAK = 8; const SOFT_BREAK = 16; const TAB = 32;
    const VIRTUAL = 64;  // 控制字符/不可见;不进图集
    const CLUSTER_START = 128;
}}
pub enum GlyphRotation { None, Cw90, Ccw90 } // Ccw90 为 V2 预留变体(2026-07-02 评审收口),V1 不产出

// 请求扩展
pub enum TextOrientation { Horizontal, Vertical }
pub enum VerticalMode { Upright, Mixed, Sideways } // 竖排时 latin 处理
// TextShapeRequest 增 orientation: TextOrientation, vertical_mode: VerticalMode
// TextShapeRequest 增 language: Option<LocaleTag>(2026-07-02 评审收口):段落级,默认取应用 locale,进 shaped cache key。
//   用途:`locl` OpenType 特性 Han 消歧(zh-Hans/zh-Hant/ja/ko 字形差异)、传入 cosmic FontSystem locale、
//   06 按语言分派 CJK 回退(联动 01 `SubFontRange.cultures`)。
// TextShapeRequest 增 features: Vec<(OpenTypeTag, u32)>(2026-07-02 评审收口):tnum/smcp/liga 等 OT features,
//   规范化=按 tag 排序后 hash 进 features_hash(index §6 #4 / D6)。
```

**ShapedGlyphRun 相应修订(D1)**:run 删 `lines` 字段——02 输出的 run 是无行结构的逻辑序 glyph 序列;行结构(`LaidOutLine`)由 03 建立。

**atlas 解耦规则(D3,2026-07-02 评审收口)**:atlas 槽位在 render extract/quad 生成阶段按 `GlyphRasterKey` 现查(04 提供查询入口);`GlyphAtlasRef` 只允许帧内短生命周期持有,禁止进入 `ShapedGlyph` 契约或任何跨帧缓存值。

**font_id 权威通路(D4,2026-07-02 评审收口；2026-07-10 已硬切；2026-07-13 实例拆分)**:`ShapedGlyph.font_id` 类型统一为 base `FontFaceId`，其值**提取自整形后端(cosmic)实际选择的 face**，经 `font/backend.rs` 的 fontdb ID↔`FontFaceId` 映射换算；`font_instance_id: Option<InstancedFaceId>` 独立标识该 base face 的有效变量坐标。atlas/raster/offline identity 同时消费两者，禁止用 opaque instance ID 替代可取 bytes/face index 的 base face。`shaping/fallback_spans.rs` 只在 shape 前投影 CompositeFont family，禁止 post-shape 重算；旧 `shaping/font_id.rs` 已删除(见 06)。

### BIDI 算法落点(`bidi.rs`)

优先用 cosmic-text 内置 bidi(`BufferLine` 已按段落解析方向、产出 level)。`base_direction`:
- `Auto`:首个强方向字符(L/R/AL)定段落基方向(UAX#9 P2/P3)。
- 视觉重排:cosmic-text 的 `ShapeLine::layout` 已按 level 重排;本层只在投影时保 `source_range` 不乱。
- 镜像字符:RTL level 内对 `Bidi_Mirrored` 码点(`( ) [ ] { } < >` 等)换镜像字形(查 cosmic-text/face 的 `rtlm` 或镜像表)。

若 cosmic-text bidi 细节不足(嵌套隔离符 LRI/RLI/PDI),回退独立 `unicode-bidi` crate 跑 `BidiInfo`,再喂 per-level run 给整形(UE `ShapeBidirectionalText` 同结构)。

### 竖排落点(`vertical.rs`)

V1 范围(对齐 godot vertical 基线):
- 主轴 = y;`advance` 取 `vmtx` 竖直进格,缺失则 `ascent+descent`。
- `VerticalMode::Mixed`(默认):CJK/全角 upright(`rotation=None`,水平居中到竖列),拉丁/数字 `Cw90`(`rotation=Cw90`)。
- `Upright`:全部正字。`Sideways`:整行旋转(含 CJK)。
- 标点:若 face 有 `vert`/`vrt2` GSUB feature 则启用得竖排形(句读居中);否则合成偏移居中。
- baseline:竖排 baseline 居列中线;`ShapedLine` 在竖排下 `baseline_y` 语义转为"列中线 x"。

V2(本计划不实现,留接口):双向竖排混排、`text-combine-upright`(纵中横)、避头尾竖排禁则交 `03`。

### 与既有路径的硬切换(`render/14` 清单 #1,本计划执行整形侧)

| 现有 | 切换 |
|------|------|
| `shaper.rs::UiTextBackendIntent` 三态 + 回退理由 | 删除;`UiTextShaperStack` 只持 service 适配器 |
| `layout_engine.rs` 等宽 `text_advance`/`measure_width` | 删除(度量迁 03 真实字形);`visual_order.rs` 低保真 BiDi 删除(BIDI 迁本计划 `bidi.rs`) |
| `graphics/.../ui/text.rs` glyphon 既做布局又绘制 | glyphon 仅绘制;布局数据来自 `ShapedGlyphRun` |

### 测试与验收清单

| 测试 | 断言 |
|------|------|
| `text_shape_latin_kerning_matches_face_metrics` | "AV"/"To" 的 advance 含 kerning,与 face `kern`/GPOS 一致 |
| `text_shape_ligature_fi_single_glyph` | "fi" 合字为单字形,`source_range`=两字符区间 |
| `text_shape_clusters_map_source_ranges_monotonic` | 所有 glyph `source_range` 单调、无空洞、并集=全文本 |
| `text_script_segmentation_arabic_latin_runs` | `"abc مرحبا"` 切 Latin/Arabic run,space common 归前一强 script,script 标注正确 |
| `text_script_segmentation_keeps_emoji_zwj_sequence_as_emoji_script` | emoji ZWJ 序列 glyph 标为 `Zsye`,前后 Latin glyph 仍为 `Latn` |
| `text_bidi_mixed_ltr_rtl_visual_order_matches_uax9` | 取 UAX#9 标准用例若干,视觉序与参考一致 |
| `text_bidi_mirrors_paren_in_rtl` | RTL run 内 `(` / `)` 按视觉方向互为镜像且保留 source_range |
| `text_bidi_mirrors_arrow_in_rtl` | RTL run 内 `→` 渲染为 `←` 镜像字形且保留 source_range |
| `text_shape_emoji_zwj_sequence_single_cluster` | 👨‍👩‍👧(ZWJ)为单 cluster,source_range 完整 |
| `text_shape_variation_selector_keeps_cluster` | 基字 + VS16 同簇,emoji 呈现 |
| `text_vertical_cjk_upright_advances_on_y` | 竖排 CJK `advance` 在 y、`rotation=None` |
| `text_vertical_latin_sideways_rotated` | 竖排拉丁 `rotation=Cw90` |
| `text_vertical_punctuation_centered` | 句读竖排居中(有/无 `vert` 两路径) |

里程碑命令:`cargo test -p zircon_runtime text_shape --locked`、`text_bidi --locked`、`text_vertical --locked`。

## 7. 风险与回退

- cosmic-text bidi 隔离符支持不足 → 切 `unicode-bidi` 独立跑,接口不变。
- 竖排是长尾:V1 只保证 CJK 正字 + 拉丁旋转 + 标点居中,旋转字形依赖 face GSUB,缺失则合成,对拍 godot。

## 产出记录与时间

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

当前概述（2026-07-13）：SH-M3 已从“横排 shaping 后投影”硬切出 folder-backed rustybuzz vertical backend：LTR/RTL 分别映射 TTB/BTT，真实 `vert`/`vrt2`、language/features、backend glyph id/face id/instance id、source range、signed y advance、vertical origin offset 与 rotation 进入同一 shaped glyph 数据面。生产 SDF atlas 以实际 glyph id + face/instance id 建 key/烘焙，不再按 Unicode scalar 重查竖排替代字形；SDF quad 消费 backend vertical origin/VORG-side-bearing offset。当前源既有 `text_vertical_` 17/17、SDF vertical 5/5、atlas 23/23、font bake 10/10 与真实 WGPU 两列 CJK 产品帧已通过。新增 horizontal RustyBuzz leaf 现承接 cosmic-text `Attrs` 缺失的 per-run language 与变量轴应用；当前源 `text_horizontal_` 5/5，通过真实 `Bahnschrift` width-axis、真实 `Calibri` 俄语/塞尔维亚语 `locl`、空语言/竖排边界与 screen-space face/instance identity。来源 Editor07 original exact 尚待其非文本门禁复验，因此 Text02 仍为 `in_progress`，不把局部 focused green 扩大为来源计划完成。

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

- 迁入记录：[`02/2026-07-09-shaping-unicode-and-bidi-output-records.md`](02/2026-07-09-shaping-unicode-and-bidi-output-records.md)
- fixed 已修复：[variable-shaping-visibility-compilation](../../zircon_editor/editor/07/fixed-2026-07-14-variable-shaping-visibility-compilation.md)
