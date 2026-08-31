---
related_code:
  - zircon_runtime_interface/src/ui/surface/render/resolved_style.rs
  - zircon_runtime/src/text/model/shaped_run.rs
  - zircon_runtime/src/ui/surface/render/resolve.rs
  - zircon_runtime/src/ui/text/resolved_layout.rs
  - zircon_runtime/src/ui/text/shaper.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/layout_engine/wrapping.rs
  - zircon_runtime/src/ui/text/layout_engine/visual_order.rs
  - zircon_runtime/src/ui/text/grapheme.rs
  - zircon_runtime/src/text/mod.rs
  - zircon_runtime/src/text/hard_line.rs
  - zircon_runtime/src/text/layout/mod.rs
  - zircon_runtime/src/text/layout/measure.rs
  - zircon_runtime/src/text/layout/line_break/mod.rs
  - zircon_runtime/src/text/layout/rich.rs
  - zircon_runtime/src/text/shaping/mod.rs
  - zircon_runtime/src/text/shaping/bidi.rs
  - zircon_runtime/src/text/shaping/cosmic.rs
  - zircon_runtime/src/text/shaping/cosmic/font_system_cache.rs
  - zircon_runtime/src/text/shaping/cosmic/hard_lines.rs
  - zircon_runtime/src/text/shaping/itemize.rs
  - zircon_runtime/src/text/shaping/horizontal/backend.rs
  - zircon_runtime/src/text/shaping/horizontal/direct.rs
  - zircon_runtime/src/text/shaping/horizontal/tests.rs
  - zircon_runtime/src/text/shaping/vertical.rs
  - zircon_runtime/src/text/shaping/vertical/backend.rs
  - zircon_runtime/src/text/shaping/vertical/direct.rs
  - zircon_runtime/src/text/shaping/vertical/orientation.rs
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
2. **Unicode 完整支持**:grapheme cluster、script 检测分段、V1 source-preserving 规范化策略（未来 NFC 必须附双向映射）、组合字符、变体选择符(VS15/16)、emoji ZWJ 序列、控制字符处理。
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
- grapheme/ZWJ/VS 已进入 cluster/source range 与 script-tag 数据面；V1 的明确策略为 source-preserving：`normalize.rs::ShapingTextView::source_preserving` 把原始 UTF-8 同时交给 shaping 与 source byte range 投影，组合/分解等价序列保持不同的原文与 cache identity。Cosmic/fallback 都经该 owner；未来启用 NFC 时必须先把 identity 投影升级为完整、版本化的 pre↔post 双向映射。
- script segment tag、per-script shaping 与 fallback face/instance reconciliation 已落地：`fallback_spans.rs` 按 grapheme 解析回退 face/instance，`itemize.rs` 将其与 BIDI level、script 合并为逻辑 segment，horizontal/vertical direct backend 按 segment 直接 RustyBuzz shaping。Arabic mark 同 cluster 同 face、Arabic/Latin script tag 与 Emoji ZWJ script 的回归已在源码中覆盖；受管 Cargo 尚未运行，故此处仅记录静态实现状态。
- `TextShapeRequest` 已携 language 与排序去重的 OpenType features，二者进入 shaped cache key；canonical slice 同时供 cache 与 backend 使用。language 已从可序列化 `UiResolvedStyle.language` 经模板解析、layout/shaped cache、direct/parallel request、native batch 与 SDF atlas/bake fallback 贯通；`text/language.rs` 是唯一 canonicalization owner，`cosmic/font_system_cache.rs` 只消费 canonical identity 并限制最多四个 locale-specific `FontSystem`。由于 cosmic-text 0.18.2 的 `Attrs` 不暴露 language/任意变量实例，horizontal 主链现由 `itemize.rs` 按 grapheme、BIDI level、script、fallback face/instance 分段，并由 `horizontal/direct.rs` 一次 RustyBuzz shape 直接构建逻辑序 `ShapedGlyphRun`；cosmic 只保留整请求失败回退，不再先 shape 后替换 segment。
- SH-M3 V1 已推进为同一单后端架构：`vertical/direct.rs` 对 upright segment 一次执行 TTB/BTT RustyBuzz，对 sideways segment 使用同一 RustyBuzz horizontal backend；face instance、language/features、`vert`/`vrt2`、VORG/side-bearing、source range 与 native y advance 直接进入最终 glyph。`vertical/orientation.rs` 仍单独拥有 Mixed/Upright/Sideways policy，UI VerticalRl 继续消费 vertical request/cache key；旧 `horizontal/projection.rs`、`vertical/projection.rs` 与 overlap projection owner 已硬删除。

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
3. `ui/text/shaper.rs` 适配：`UiSharedTextShaper` 直接实现 `UiTextShaper` 并投影 `ShapedGlyphRun → UiResolvedTextLayout`；删除空的单成员 `UiTextShaperStack` 以及 `UiTextBackendIntent`/`active_layout_backend_for_intent`/`fallback_reason_for_backend`（`render/14` 硬切换清单 #1）。

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
| `cosmic.rs` | backend 路由与 cosmic 整请求回退隔离；禁止消费 cosmic glyph 后再调用 RustyBuzz 替换 segment |
| `../hard_line.rs` | Text02/Text03/UI 共用的 mandatory hard-line owner；保存 content 与 CRLF/Unicode separator range |
| `itemize.rs` | grapheme、BIDI level、script、fallback face/instance、vertical orientation 的共享分段 owner |
| `horizontal/{backend,direct}.rs` | 一次 RustyBuzz horizontal shape 与逻辑序 DTO 构建；承接 locl/features/variable instance |
| `vertical/{backend,direct,orientation}.rs` | 一次 TTB/BTT 或 sideways shape、列定位与 Unicode orientation policy |
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

**ShapedGlyphRun 当前契约修订(D1,2026-08-01；2026-08-27术语硬切同步)**:`ShapedGlyphRun`以`Arc<str>`持有唯一source，并保留backend显式段落/硬换行投影所需的`ShapedHardLine`；hard line只保存range/metrics/glyphs，不再保存owned text。它不是wrap owner，03仍以`CandidateLine`/`UiResolvedTextLine`独占软换行、ellipsis、rich/inline materialization与最终布局语义；不得把02的backend hard line扩张为第二套03布局策略。provider/session入口同批改为`shape_horizontal_range(_with_kerning)`/`shape_vertical_range(_with_kerning)`，因为输入是`text + absolute source_range`且可能产出多个hard lines；这不改变`BackendShapeRequest`、budget或算法。

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
| `shaper.rs::UiTextBackendIntent` 三态 + 回退理由 | 删除；唯一 `UiSharedTextShaper` 直接实现 service 适配器，单成员 `UiTextShaperStack` 同步删除 |
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

当前概述（2026-08-01）：PERF-MVP-233的packed script、single shared source、range-only shaped line、cache exact owner和canonical OpenType identity已完成非验收实现。PERF-MVP-234/235的Text02主链已前向收敛：shared itemization一次解析grapheme/BIDI level/script/fallback face-instance；horizontal与vertical均由RustyBuzz直接构建逻辑序glyph，cosmic只作整请求回退，旧post-shape projection文件已删除。fallback codepoint scratch跨cluster复用，连续同face/instance的family String不再重复分配。首次独立审查的6个P1均已前向修复；后修复复核为P0=0、P1=0。`Tr` 现在以同一 buffer、direction、script、language、非竖排 feature 集的 `vert`/`vrt2` enabled/disabled glyph-sequence 差分确定实际 substitution，并仅在 TransformOrRotate segment 付出第二次 shape；rich extract 保留 layout 所属的 compiled artifact，type-erased `Arc` 随 DTO 生命周期释放，不再有 registry idle retention。ordered/non-overlapping projection run 契约与回归也已加入。真实Windows WGPU产品帧harness已切到`docs/tests/runtime/text/runtime_text_mvp_foundation_product_framebuffer_20260801.png`，不会覆盖旧证据；当前只声明非验收实现与静态复核，managed Cargo、1/100/1k/10k counters、跨Text01 parsed-face指标与该新产品帧的实际生成/像素审查仍待验证。

当前概述（2026-08-15）：Text 工件源视图现在只接受完整原文或与绝对 `source_range` 精确等长的切片，其他输入 fail closed 到既有布局 DTO 路径；构建前还验证每个 line 落在 layout source range 内、每个 run 落在所属 line 内，ellipsis 的边界空 range 保持合法。`glyph_artifact_exact_source_slice_preserves_absolute_glyph_ranges` 现编码 exact slice 的绝对 4..8 glyph range 契约，但尚待受管 Cargo 运行。UI12 M6 的 source-map 测试缺失导入也已经由 `crate::ui::surface` 公共重导出静态闭合。静态复核确认 retained host 对同一 artifact layout 以一个 face batch 投影全部 visual line，任一工件不可用时整套回退，因而不会混用 runtime 与 host fallback 的 glyph identity。截图 harness 已静态复核为 `ProductUiFrameRenderer` 的实际 extract/render/capture，使用背景像素差与 settled raster 状态拒绝纯文本式证据，输出解析到 E 盘工作区的 `docs/tests/runtime/text` 并拒绝 `target`；预期的本轮 PNG 仍不存在。

当前概述（2026-08-30）：Cosmic whole-request fallback 的生产 router 与回归 owner 已完成边界收敛。原 `cosmic.rs` 内联的十个测试不改断言地迁入 `text/shaping/cosmic/tests.rs`，生产 owner 从 875 行降至 712 行，测试 owner 为 161 行；`cosmic.rs` 继续只负责 backend adapter/orchestration，没有引入新 facade 或第二 shaping 状态机。定向静态 owner 契约与 rustfmt 已通过；managed Cargo、性能数据与真实 WGPU 产品帧仍待统一验证，状态为 `cosmic_test_owner_split_complete / production_router_under_review_threshold / behavior_unchanged / managed_validation_pending`。

性能复审的首个待验证结构性候选是 plain layout 与 artifact 的 shape request 不同：前者为完整或视觉行文本采用相对范围和 `Auto` direction，后者为逻辑源切片采用绝对 line `source_range` 与已解析 direction；现有 shaped-cache key 包含 text hash、source range 与 direction，故二者通常不能共用缓存条目。此结论来自静态调用链，不是性能回归结论。为先获得可归因数据，`text.layout/resolve_without_artifact` 现单独报告其 shaped-cache hit/miss，artifact span 继续报告自己的 cache delta、line count、font-handle registration batch、registry lock acquire/wait/hold 和 snapshot publish。后五项 registry 字段来自每个 registration batch 的局部报告并由 artifact 按行累加，不再以全局单调 report 前后差值错误归因其他线程；普通 CPU profiling 在 capture inactive 时，layout 不读取前后 shaped-cache report，artifact 也回落原 batch API，避免 cache copy 或局部 `Instant` 计时污染 idle 路径，Tracy 则保留连续 event 语义。第二个候选仍是每条非空布局行各调用一次 `register_font_handle_batch` 的全局 registry mutex 路径。lane 释放后必须在 1/100/1k/10k 的 Latin、CJK、RTL、ligature 与 wrap 样本上同时采集 layout/artifact span、两阶段 cache delta 和 registry 数据；只有数据证明材料性瓶颈，才执行 hard-cutover：由 Text 持有以源范围索引的不可变 shaped sequence，UI 仅保留 visual subsequence 引用，renderer 不再 shape，synthetic ellipsis/tatweel 等虚拟 run 保持显式 fallback。该方向对齐 Unreal `FShapedGlyphSequence` 的 source-index/subsequence 生命周期，但当前没有运行时或功耗数据、没有功耗对标结论，也没有生成新的 WGPU 产品帧。

`docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md` 已指出当前 CPU recorder 的进程全局 mutex 会造成 observer effect；因此上述 profiler span/counter 只用于定位调用拓扑、cache 和 lock 增量，不能单独支持耗时、p50/p95、功耗或参考引擎对标。最终性能门槛必须在 capture 关闭且预热稳定的独立单调计时入口上运行至少 31 个样本，并以同一输入、机器配置和产品 trace 交叉核对；功耗结论还必须记录采样窗口与平台测量条件。任何一项缺失时只写“待测量”。

`layout_engine/tests/performance.rs` 现已编码该独立入口：它以新 `SharedTextLayoutSession` 的首次 layout/artifact 请求记录 cold 样本，并以同一 session 的第二次完全相同请求记录 warm 样本；四类 Latin、CJK、RTL、ligature 文本各覆盖 1/100/1k/10k grapheme、固定 word-wrap 宽度、31 个样本，输出 cold/warm p50/p95 与本 session shaped-cache hit/miss 增量。测试显式要求 profiling 与 Tracy feature 均未编译，且不设机器相关耗时阈值。它仍是 ignored 的待运行基准，尚无数值、功耗或优化结论；UI12 验证 lane 未释放前不得启动 Cargo。

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

当前概述（2026-08-24）：Text02 的 no-font/error 边界已继续硬化。`shape_text(...)` 现在
返回 typed `TextLayoutError`，服务层会传播缺失 font face；整形失败不会再发布 guessed
advance、FNV glyph id 或 synthetic font identity，glyph projection 也不会在没有实际 face 时
请求 rasterization。接口层把 glyph authority 明确表示为
`UiTextShapeArtifact::Canonical(UiShapedText) | Unavailable`：
`UiShapedText::from_resolved_layout(...)`、样式派生 font/atlas key 与其 synthetic glyph
projection 已删除。仅有完整 grapheme advance 的 resolved layout 可以形成 paint-run geometry；
不完整几何 fail closed，不推断等宽 frame。横排、VerticalRl、缺 advance 与 canonical/unavailable
的回归已迁入 `zircon_runtime_interface/src/tests/render_contracts/text_shape.rs`，避免继续扩张根
contract test owner。该接口 DTO 不取代运行时 text-owned immutable glyph artifact；WGPU 主路径仍
直接消费后者。

本轮完成的是结构和静态契约收敛：受限 `rustfmt --check`、`git diff --check` 与 retired-symbol
扫描通过。状态为 `implementation_complete / resolving_failure / managed_validation_pending`；当前
尚未得到 coordinator 的有效 Cargo 执行授权，因此没有 Cargo green、31-sample p50/p95、功耗结论、
真实 framebuffer PNG、里程碑 acceptance、commit 或 WeCom 通知。

P1-9 状态同步（2026-08-26）：`HardLine` 已只表示 source separator；默认 64 KiB
`TextShapingWorkBudget` 不能被描述为逻辑行、script run 或 cluster 边界。retained session 的
canonical cache miss 与 parallel prewarm 的 unique pending job 现已接入生产规模回执，记录阈值内/
oversized-synchronous 请求数、总输入字节与最大请求字节；cache hit、batch duplicate、invalid
request 不冒充 backend work。超阈值请求仍保留完整 source/context 同步执行，算法未改变。状态为
`production_work_receipt_implemented / source_semantics_preserved / static_checks_complete /
managed_profile_pending`。typed deferred/cancelled work-unit scheduler、CPU/内存/deadline budget、
受管性能测量与产品资格仍保持 open。

P1-18 状态同步（2026-08-26）：horizontal/vertical direct 与两个 RustyBuzz backend adapter 已统一返回
typed `Result`。font access/index/parse、empty output、source range、cluster offset/order 与 finite metric failure
均进入 12-code receipt；只有 horizontal backend/font/glyph capability 可以 alternate Cosmic，vertical 与
source/itemization/BiDi/font-source-budget invariant 均 fail closed。RustyBuzz 0.20.1 的 language parser 只拒绝
空串，而 canonical request 在此之前已过滤空串并拒绝非法 BCP-47，因此 backend 中 optional language
projection 的 `.ok()` 不是可达 fallback signal。本项状态为
`typed_direct_backend_failure_receipt_implemented / policy_scoped_horizontal_alternate_backend /
vertical_and_invariant_fail_closed / static_checks_complete / managed_fault_injection_pending`；没有因本次
current-source 校准新增生产代码、Cargo 或性能/视觉结论。

当前概述（2026-08-26）：对 `RTS-P1-011/012` 的 current-source 重审修正了“shaping 已经
发布最终视觉序”的过度表述。Direct/Cosmic 主链为换行保留 logical glyph、source range 与
resolved bidi level；UAX#9 L1/L2 仍必须在 Text03 确定物理行边界后执行，这与 Unreal Slate
先建 line block、再按 line direction 排列 block 的层级一致。缺陷是 UI 普通行此前重新读取
paragraph text、按 grapheme 重建范围并生成一个临时视觉序，没有把结果作为 canonical line
artifact 保存。

Plain/Horizontal MVP 现由 `CanonicalPhysicalLineFragment` 保存 post-wrap `BidiLineOrder`，并与
同一 final-line shape、metrics、grapheme advances 和 font generation 同寿命；UI 普通物理行
优先消费这份收据，不再依赖 paragraph 文本重新分析。generated virtual/ellipsis 等非同构行
仍走显式 display-owned 路径，rich/vertical/viewport fallback 仍保持原 adapter，因此
`RTS-P1-011/012` 只记为 `partially_implemented / managed_validation_pending`。聚焦测试锁定 mixed
LTR/RTL order、fragment storage 和 receipt 优先路由；静态格式、whitespace、call-site、文件规模
与生产异常扫描通过。新增 `text.layout/resolve_physical_line_bidi_order` 与
`resolve_visual_order_fallback` scope 只用于受管归因；当前没有 Cargo、corpus、p50/p95、功耗、
WGPU 或 PNG 证据，也不声称复用 shaping levels 或获得性能收益。

当前概述（2026-08-26 font-generation owner hard cut）：字体资源在 UI layout/extract 之后发布
新 generation 时，graphics 原先会创建临时 `SharedTextLayoutSession` 重建单行 artifact，并把
结果塞入 renderer `refreshed_line` overlay；这违反了本计划禁止 renderer-local reshaping 的
边界。现在 `UiSurface::rebuild_dirty` 在 clean-frame return 前观测 generation，变化时由 retained
surface owner 执行一次完整 layout/render-extract；graphics 只在 font load 后拒绝 stale artifact
batch 并记录低基数计数。两个 standalone line rebuild API 与 overlay 字段已删除，atlas/CPU/SDF
测试改为消费 text owner 重新发布的不可变 artifact `Arc`。状态为
`non_validation_implementation_complete / static_checks_complete / managed_validation_pending`；受管
`font_generation` batch 在 Cargo 启动前因 `cargo.acquire command_post_timeout` 结束，未轮询或重试。
31-sample profile/allocation/power、真实 WGPU/PNG、commit 与 WeCom 仍保持开放。

当前概述（2026-08-26 script itemization 实施前重审）：当前 `script_segment.rs` 仍只读取单值
`UnicodeScript`，`pending_common_start` 写入后不参与边界决策；fallback 又独立把 cluster 首个非空白
codepoint 填入 `FontScript::Other`，导致 fallback 与 HarfBuzz script owner 分裂并放大 cache identity。
已冻结的修复不是新增字符范围，而是由一次 paragraph script analysis 同时驱动 fallback 与
horizontal/vertical/Cosmic 路径：Script_Extensions 使用现有定长位集做无分配交集，paired bracket
使用 `unicode-bidi` 完整 BidiBrackets 数据栈，显式 language script subtag/邻居只做多候选消歧。
Unreal 提供 face 后再按 script 切 HarfBuzz batch 的结构标准，Godot 提供 nested bracket 继承语义。
目标复杂度为 `O(codepoints + runs + bracket_depth)`；emoji property、likely-subtag、统一 Unicode
snapshot 与官方 corpus 仍保持 open，不能随本切片一起宣称完成。

当前概述（2026-08-26 script itemization 非验收实现）：`script_segments(text, language)` 现在使用
`unicode-script` 的定长 Script_Extensions 交集和 `unicode-bidi` 完整 paired-bracket 数据；opening
context、leading/nested bracket、显式 script subtag 与相邻脚本消歧均由聚焦回归固定。一个 paragraph
只生成一份 script segments，并同时供 fallback、horizontal/vertical direct 与 Cosmic projection 消费。
fallback 的 `FontScript::Other(FontScriptTag)` 改为受检 packed ISO15924，同 script 的不同 codepoint 不再产生
不同 script identity；私有字段、受检构造与自定义反序列化共同拒绝任意 `u32` 和非 canonical-form tag，
同时保持已有 `{"other": <u32>}` 资产序列化外形。未分配字符进入 typed `Unknown`。旧手写 font script 范围、
cluster 首码点伪 script 路径和裸 `Other(u32)` 构造已删除。
触及文件均低于 800 行，scoped formatter、whitespace、调用点与 retired-pattern 扫描通过。状态为
`non_validation_implementation_complete / static_checks_complete / managed_validation_pending`；emoji
property、likely-subtag、统一 Unicode snapshot、官方 corpus、Cargo、性能/功耗、WGPU/PNG、commit 与
WeCom 仍开放。

当前概述（2026-08-26 emoji presentation 实施前重审）：当前 `script_segment.rs` 使用
`0x1F000..=0x1FAFF | 0x2600..=0x27BF` 宽范围同时承担 script 与 emoji presentation 判定，会把
未分配码点、默认 text presentation 符号和 VS15 请求错误路由到 `Zsye`，并漏掉 keycap 等 sequence。
本地依赖 `unicode-properties 0.1.4` 已提供 Unicode 17 Emoji/Component/Presentation/Modifier 状态，故禁止
继续扩私有范围表。实现冻结为 paragraph-owned `ParagraphTextAnalysis`：构造时按 extended grapheme 扫描一次，
保存 script segments 与 emoji-presentation ranges；fallback、itemize、direct 与 Cosmic 只做 source-range 二分
查询。目标复杂度为 `O(codepoints + graphemes + emitted_ranges)`，额外内存为
`O(script_runs + emoji_graphemes)`，glyph 循环不得重复查询 Unicode property。VS15/VS16、默认 presentation 与
keycap 先收敛 MVP；完整 RGI ZWJ/tag/flag sequence 数据、`UnicodeDataSnapshot`、官方 corpus、动态性能/功耗与
WGPU/PNG 继续开放，不随本切片关闭 `RTS-P1-003/006`。

当前概述（2026-08-26 emoji presentation MVP 非验收实现）：`ParagraphTextAnalysis` 现在一次持有
Script_Extensions segments 与按 extended grapheme 生成的 emoji-presentation ranges。后者使用
`unicode-properties 0.1.4` 的 Unicode 17 Emoji/Presentation 状态，覆盖 plain text-default、VS15、VS16、
默认 emoji 与 keycap，并拒绝旧宽范围中的未分配码点；相邻 presentation ranges 会合并。fallback、itemize、
horizontal/vertical direct 与 Cosmic projection 只按 source range 二分查询同一 analysis，glyph loop 不再
读取 Unicode property。旧 emoji 宽范围、`is_emoji_script`、逐 cluster script 重判入口扫描为 0；scoped
formatter 与 whitespace 检查通过。状态为 `mvp_property_presentation_implemented / full_rgi_open /
static_checks_complete / managed_validation_pending`；完整 RGI ZWJ/tag/RI/modifier sequence、统一 Unicode
snapshot、官方 corpus、Cargo、动态性能/功耗、WGPU/PNG、commit 与 WeCom 仍开放。

当前概述（2026-08-26 direct failure receipt 非验收实现）：RustyBuzz horizontal/vertical backend
已用 `BackendShapeError` 保留 font operation、face、原始 `FontDatabaseError`、face parse 与 empty-output
原因；其上 `DirectShapeError` 继续区分 itemization、Bidi、source range、cluster UTF-8 boundary、finite
metrics 与 cluster order。horizontal/vertical direct 现在只返回 required `ShapedGlyphRun` 或 typed error，
不再以 `Option` 表达失败。

`cosmic.rs` 是唯一降级策略边界。`failure_receipt.rs` 现将 direct failure 映射为 12 个稳定 numeric/string code，
并保留 phase、source range、face、dependency、disposition 与 optional budget；低基数 report 由实际
`SharedTextLayoutSession` 逐帧持有，按 code 计数并保留最后一条 receipt，不再经过进程全局锁。旧
`is_bidi_invariant` 第二分类器已删除。

策略已进一步 fail closed：Bidi、非法 source range、itemization/fallback-span 不一致和 font-source admission
budget 不允许进入备用后端；只有 horizontal backend/font-face/glyph capability failure 可尝试 whole-request Cosmic，
vertical 仍终止。这样不会由备用路径掩盖 invariant 或绕过资源预算。静态扫描结果为 backend `Option<Run>` 0、
direct `Option<ShapedGlyphRun>` 0、direct `.ok()?`/`Ok(None)` 0、failure policy owner 1、旧 Bidi classifier 0；
scoped formatter、whitespace、调用点和文件规模检查通过。

随后 `TextShapingOutcome` 的所有非 Ready 分支统一携内部 `TextShapingFailure { error, receipt }`；direct terminal
failure 及 alternate-backend 后最终失败均把本请求 receipt 穿过 stable font-generation 边界、session、layout 与 UI
内部转发链，`map`/`and_then` 不丢失 receipt，Failed/Deferred 仍不可进入 shaped cache。公开
`TextLayoutService` 边界只投影原有 neutral `TextLayoutError`，没有把 Text 私有 `FontFaceId`/`TextRange` 反向引入
`core/framework/text`；session report 仅作诊断，不参与请求 retry/fallback 决策。状态为
`request_outcome_receipt_implemented / public_neutral_error_projection_retained /
run_local_composition_open / static_checks_complete / managed_validation_pending`；fault injection、managed Cargo、
性能/功耗、真实 WGPU/PNG、commit 与 WeCom 仍开放。

2026-08-27 RTS-P2-011 同步：layout fallback 与 shaping failure 的两个 process-global
`OnceLock<Mutex<...>>` owner及公开 getter已删除。`SharedTextLayoutSession` 逐帧持有 layout code、typed shaping
failure 与 direct/whole-alternate/hybrid/terminal backend-route 固定计数；whole Cosmic recovery 以空
`alternate_ranges` receipt 标识，hybrid 继续保留实际 source ranges。parallel prewarm 在 worker 完成项聚合后合并回
同一 session，不让 worker 反向持有 layout owner。cache hit 不计作 backend work，所有 profile 名固定且不记录 raw
text/document id。具体 document drill-down 等待统一有界 document owner；managed Cargo/fault/profile/power/WGPU/PNG
未执行，状态为 `session_owned_diagnostics_implemented / process_global_report_mutexes_removed /
fixed_backend_route_projection_implemented / parallel_prewarm_merge_implemented /
document_drilldown_owner_open / static_checks_complete / managed_validation_pending`。

同一边界复核随后移除了两个隐式缺口：非空 hard-line separator 的非法 source range 现在返回 typed
itemization failure，不再成为 `Ok(None)`；horizontal/vertical RustyBuzz backend 直接消费受检
`Iso15924Tag`，不再把 tag 降为 `&str` 后以 `.ok()?` 重解析。Common/Inherited/Unknown 仍显式保留
backend inference，这属于既定 policy。该补充仍只有静态证据，不改变上述验收状态。

`RTS-P1-013/021` 的下一项前置也已收敛：`DirectShapeError::Backend` 同时保留 itemized segment
`TextRange` 与原始 `BackendShapeError`，horizontal/vertical 三个 backend 调用点均显式附加 range。
这使后续按 analysis run 诊断/组合具备最小上下文，但当前仍是 whole-request policy，没有实现或宣称
partial recovery。

当前概述（2026-08-26 locale canonical identity 与显式 script projection 非验收实现）：语言标签策略已从 Runtime
Interface、shaped-cache 与 script itemizer 的重复 helper 硬切到 `text/language.rs`。该 owner 使用 ICU4X
`Locale::try_from_str` 一次完成非空语法校验和 canonical casing，并从同一结构化 `Locale.id.script` 产出定长
`TextLanguageScriptSubtag`；不再用字符串 split 猜测 script。`BackendShapeRequest::canonicalized` 在 cache/backend 前
返回 typed `InvalidLanguage`，同时保留 canonical tag 与显式 script，私有 canonical 重借用保证
`layout_session -> service -> shape_text -> ParagraphTextAnalysis` 下游不重复解析。

canonical 输入保持 borrowed，shaped cache 改为 exact hash/equality，Cosmic 的四项 `FontSystem` LRU 只在新 locale
插入时分配字符串。fallback span 与 Cosmic paragraph analysis 消费同一 request-owned script identity；旧 interface
normalizer、cache byte-loop、script itemizer 手写拆分与 Cosmic 每请求 normalization 均已删除。

本实现完成 `RTS-P1-002` 的 canonical-tag 基础设施和显式 script 输出，并收紧 `RTS-P1-004` 的输入 owner。对照 Unreal
`FCulture` 的 central canonical name、独立 script/region 和 prioritized parent cultures，未写入标签的 likely
script/region、版本化 locale data 与 fallback decision receipt 仍开放。`shaped_run.rs` 的 194 行内联测试已硬切到
folder-backed `model/shaped_run/tests.rs`，生产 owner 从 801 行回落到 605 行。scoped formatter、whitespace、
retired-helper、手写 split、调用点和文件预算扫描通过；managed Cargo、locale corpus、cache cardinality/p50/p95/p99、
RSS/功耗、WGPU/PNG、commit 与 WeCom 均未完成。状态为
`canonical_tag_and_explicit_script_projection_implemented / likely_subtag_receipt_open / static_checks_complete /
managed_validation_pending`，不得据此关闭 Text02 或 M2。

当前概述（2026-08-26 Unicode provider snapshot 非验收实现）：新增 `text/unicode_data.rs` 单一 owner，逐项记录并
精确锁定十二个 provider role 的实现/数据版本：locale parser `2.2.0`，Normalization `17`，Bidi/Mirroring `16`，
Script/Grapheme/Word/Emoji/GeneralCategory/JoiningType `17`，LineBreak `15`，VerticalOrientation revision `17`。Word 与
Grapheme、Emoji 与 GeneralCategory 虽各自共享当前实现包，仍作为独立语义角色进入 schema v4/generation 4 指纹。它们经稳定 schema hash 形成
16-byte `UnicodeDataSnapshotId { generation, fingerprint }`；完整 descriptor 不复制到热路径。

request 在 analysis 前冻结 identity，canonical reborrow 保持该值；`ParagraphTextAnalysis`、`BidiParagraph`、
`BidiLineSignature`/`BidiLineOrder`、`LineBreakOpportunityMap`、`ShapedGlyphRun`、shaped-cache
exact/direction-alias key 与 `TextLayoutFallbackReport` 均携同一 identity。direct 与 Cosmic production path 在消费
analysis 前断言 request/artifact identity 一致，cache admission 同样断言 key/artifact 一致。旧 serialized shaped
artifact 缺 identity 时直接反序列化失败，不以 current snapshot 回填。以 1024-entry cache 计，key 新增 identity
上限约 16 KiB；没有新增 Unicode analysis、shape call 或 renderer pass。

状态为 `compiled_snapshot_identity_implemented / static_checks_complete / managed_validation_pending`。当前仍是编译期
snapshot；provider hot update、旧 generation lease retirement、完整 analysis/layout/document artifact 贯通、managed
Cargo/corpus、p50/p95/p99、RSS/功耗、WGPU/PNG、commit 与 WeCom 继续开放，因此不关闭 `RTS-P1-003`、
`RTS-GATE-003/012` 或 M1/M2。

当前概述（2026-08-26 locale fallback key 非验收实现）：同一次 ICU4X locale 解析现在还产出
`TextLanguageFallbackKey { language, script, region }`。`BackendShapeRequest` 的 canonical reborrow 保留该 Copy key，
fallback span 直接消费它，避免在 `service -> itemize -> fallback` 链上第二次解析；显式 script 也从该 key 投影，
不再维持并行字段。字体候选缓存身份只哈希这三个会影响 CompositeFont 候选选择的结构化分量；canonical-equivalent
输入共享身份，Unicode extension 不污染字体候选缓存，而完整 canonical tag 仍留在 shaped cache，故不会错误合并
`locl` 等 shaping 语义。

这一步只交付父文化组合所需的显式 request key，不做 likely-subtag 推断。没有脚本/地区的 `zh` 仍保持缺省，
不会被硬编码补成 `Hans/CN`。相关 request retention 与 cache identity 回归已写入源码但未执行；状态为
`canonical_locale_fallback_key_implemented / likely_subtag_receipt_open / static_checks_complete /
managed_validation_pending`，不关闭 `RTS-P1-002`、M2 或任何动态验收 gate。

当前概述（2026-08-26 cluster break-safety receipt 非验收实现）：本地 RustyBuzz 0.20.1 明确将
`GlyphInfo::unsafe_to_break()` 定义为“在该 cluster 起点断开时，两侧必须重新 shaping 才能保持结果”，不是禁止断行。
horizontal/vertical backend 现在逐 glyph 保留该位，direct 在恢复 logical cluster order 后按 cluster 聚合，并只在
cluster 头发布 1-byte `ShapedGlyphBreakSafety::{Safe, RequiresReshape}`；Cosmic、virtual glyph、非 cluster 头与旧
serialized artifact 明确为 `Unknown`。收据随 shaped artifact round-trip，但不投影给 framework/renderer，也不改变
`soft_break`/`mandatory_break`。

本地 Unreal `SlateTextShaper.cpp` 同样以 `HB_BUFFER_CLUSTER_LEVEL_MONOTONE_GRAPHEMES` 生成 shaped sequence 并保存
source/grapheme-cluster count，但未向其 `FShapedGlyphEntry` 投影 HarfBuzz unsafe flag。Zircon 保持 Unreal 的 shaper-owned
sequence 边界，同时利用 RustyBuzz 的公开 flag 补强内部证据；没有把 line-break policy 放进 backend。相邻复核还修正
vertical buffer 仍声明 `script_tag: &str` 的编译边界，horizontal/vertical 现都直接消费受检 `Iso15924Tag`。

当前只完成 `RTS-P1-017/035` 的 provenance 前置。final-line owner 仍需在 `RequiresReshape` 边界精确重塑两侧或使用可证明
context plan，`Unknown` 必须走保守路径；固定 8-grapheme correction 尚未被替换。source-present 回归覆盖 checked-in
kerning pair 的 RustyBuzz receipt、cluster-head rule、1-byte enum、serde round-trip 与 legacy Unknown。状态为
`direct_break_safety_receipt_implemented / final_line_reshape_open / static_checks_complete /
managed_validation_pending`；Cargo/corpus、断行正确性、shape-call 计数、p50/p95/p99、RSS/功耗、WGPU/PNG、commit 与
WeCom 仍开放。

当前概述（2026-08-26 line-break profile/opportunity receipt 非验收实现）：既有
`UnicodeDataSnapshotId` 已记录 `unicode-linebreak 0.1.5 / Unicode 15.0.0`，并随
`ShapedGlyphRun` 发布，因此 `RTS-P1-025` 的数据版本部分不再是空白。缺口是 cluster 只有
`soft_break/mandatory_break` 两个 bool，无法区分 provider allowed、provider mandatory、显式 hard-break
control 或旧 artifact unknown。

`ShapedGlyphClusterFlags` 现在只在 cluster 头保留 2-byte
`ShapedGlyphLineBreakReceipt { profile, opportunity }`；当前 profile 为 `UnicodeDefault`，opportunity 为
`None/ProviderAllowed/ProviderMandatory/MandatoryControl`。horizontal/vertical direct 与 Cosmic 都消费同一
`LineBreakOpportunityMap`，hard-line separator 由 itemizer 明确发布 control receipt。legacy serde 缺字段时为
`Unknown/None`。回执不投影到 framework/renderer，也不改变现有 break bool 或布局策略。

该实现不增加 Unicode pass、shape call 或 opportunity Vec；查询继续对一次性有序机会表做两次
`partition_point`。类型大小回归固定 profile/opportunity 各 1 byte、receipt 2 bytes、完整 flags 11 bytes。
`unicode-linebreak` 当前 API 不公开具体 UAX#14 rule number，locale tailoring profile/data 也尚未实现，故状态为
`line_break_profile_opportunity_receipt_implemented / rule_number_and_locale_tailoring_open /
static_checks_complete / managed_validation_pending`。官方 LineBreakTest、Cargo、性能/功耗、WGPU/PNG、commit 与
WeCom 仍开放，不关闭 `RTS-P1-025/026` 或 Text02。

当前概述（2026-08-26 Unicode word-boundary owner 非验收实现）：`text/word_boundary.rs` 现在是 Runtime Text
唯一的 UAX #29 word owner。`WordBoundaryMap` 借用 source、携 `UnicodeDataSnapshotId`，并通过
`unicode_word_indices()` 提供 previous/next/range/last-complete-prefix 查询；它不物化与段落长度成比例的第二张 Vec。
UI 编辑导航已删除本地 `split_word_bound_indices + is_alphanumeric` policy，EndWord layout 也消费同一 owner。

为使快照按能力而不是按 crate 名建模，compiled snapshot 新增独立 `word` provider role；虽然它与 grapheme 共享当前
implementation/data version，仍分别进入 fingerprint。状态为
`word_boundary_owner_implemented / locale_dictionary_and_wordbreak_corpus_open / static_checks_complete /
managed_validation_pending`。这只完成 `RTS-P1-032` 的 boundary 前置，也为 `RTS-P1-026/028/029` 提供基础设施；locale
dictionary、WordBreakTest、retained paragraph analysis、marker artifact、Cargo/性能/功耗/WGPU/PNG/commit/WeCom 仍开放。

当前概述（2026-08-26 shared cluster geometry 非验收实现）：`text/cluster_geometry.rs` 现在以同一零分配 iterator
聚合 `ShapedGlyph` 与 renderer `TextGlyph` 的 backend cluster；有 cluster-start provenance 时按 backend 起点合并，
legacy 输入只合并相同 source range，混合方向 cluster 明确发布不可用于 caret 的状态。`glyph_artifact.rs` 已删除本地
`GlyphCluster/glyph_cluster_at` owner，caret、hit 与 selection span 改为消费该共享几何。

Text03 measurement 同时从这份 owner 发布 `MeasuredGlyphCluster { source_range, advance, caret_policy }`；覆盖多个
grapheme 且没有 font caret 的 cluster 标记为 `AtomicCluster`。现有 per-grapheme advance 仅保留为兼容投影，不再是
glyph-wrap 边界的唯一真值。当前 Rust 栈仍无 GDEF LigCaretList provider，public caret/selection geometry 也尚未贯通该
typed policy；状态为 `shared_cluster_geometry_receipt_implemented / public_geometry_and_gdef_caret_provider_open /
static_checks_complete / managed_validation_pending`，不关闭 `RTS-P1-034`。

当前概述（2026-08-26 WordSmart Unicode policy 非验收实现）：`line_break/smart.rs` 已删除收尾标点与闭合符号的
手写字符表。候选必须紧邻 `WordBoundaryMap` 发布的 UAX #29 word end；样式层只以 snapshot-bound Unicode
`General_Category` 选择 `OtherPunctuation` trigger，并允许 `ClosePunctuation/FinalPunctuation` 延伸同一 protected run。
open punctuation、dash、symbol/emoji 与 separator 不会被误当作尾标点策略。

WordSmart 与 EndWord/UI navigation 共用零拷贝 word ranges，`WordEndCursor` 对有序 chunk 做单调扫描；复杂度为
`O(text + chunks + word ranges)`，不新增 paragraph-sized boundary Vec。chunk 仅在 text/visual/source 长度同构且相邻
双范围连续时切分或合并，软连字符、virtual text 与其它非同构映射会保守跳过策略。compiled snapshot 中
GeneralCategory 与 JoiningType 都是独立 capability，当前为 12 roles、schema 4、generation 4。状态为
`word_smart_uax29_context_and_general_category_policy_implemented / locale_dictionary_and_tailoring_open /
static_checks_complete / managed_validation_pending`；这只部分推进 `RTS-P1-028`，官方 WordBreakTest、locale dictionary、
style matrix、Cargo、性能/功耗、WGPU/PNG、commit 与 WeCom 仍开放。

当前概述（2026-08-26 Unicode Joining_Type 与 Arabic Kashida 候选策略非验收实现）：`text/joining_type.rs` 现为
`icu_properties 2.2.0 / Unicode 17` Joining_Type 的单一编译期 trie owner；ICU 原始类型不泄漏到 layout/UI/renderer，
布局层只消费 `TextJoiningTypeMap` 的中性连接语义。`layout/align.rs` 已删除 Arabic 字母与 non-left-joining 手写范围，
候选对按逻辑序使用 Left/Right/Dual/JoinCausing/Transparent 属性，并额外要求 grapheme base 属于 Arabic script；Tatweel、
ZWJ、ZWNJ 与透明 mark 的行为由显式 policy 保持。属性表以一个 `OnceLock` 复用，扫描仍为 `O(graphemes)`，不产生
per-character 分配。

码点属性候选之后，`layout/arabic_justification.rs` 现消费一次完整 candidate shape，只有独立 Tatweel cluster 具有非零
glyph、正 advance、RTL 左右邻接，且 Tatweel/左右 cluster 的 face/instance 一致时才发布成功收据；混合 generated/source
cluster、tofu、fallback-face 断裂、非增长或 source identity 漂移都会 fail closed。共享 cluster iterator 同时给出 glyph
span，扫描为 `O(glyph clusters + inserted tatweels)`；插入范围临时数组受现有 32 上限约束。UI 只消费收据的 width/count，
不再把“候选宽度未超目标”当作安全证明。

font/language justification API、最多 32 个候选与最多 5 次重 shape 的 profile/预算重构仍开放；在 1/100/1k/10k Arabic
workload 与真实渲染证据前不得调整探测算法或宣称 Unreal parity。本地 Unreal 源码未找到可直接复用的
Tatweel/Kashida 实现，因此这里只遵循其 shaper-owned artifact/validation 边界，不声称算法等价。状态为
`unicode_joining_type_and_backend_tatweel_safety_receipt_implemented /
language_font_justification_and_probe_strategy_open / static_checks_complete / managed_validation_pending`；不关闭
`RTS-P1-036`、M2 或任何动态验收 gate。

2026-08-26 profiling 基础设施增量：`layout_engine/line_box/profile.rs` 现对每条真正进入 Tatweel candidate fit 的物理行
聚合 requested/probe/safe/accepted count、candidate input bytes 与最后一个稳定拒绝码；循环中不发布 span/counter，
行结束时才各发布一次。`ArabicTatweelCandidateRejection::profile_code` 明确映射 1..13，0 表示无安全拒绝，14 表示理论上
不应发生的 candidate-count/receipt-count 不一致。普通 build 的 profile owner 为零字段，不增加全局状态或改变 shaping、
cache、fallback 路由。状态为 `arabic_tatweel_probe_instrumentation_implemented / algorithm_unchanged /
static_checks_complete / managed_profile_pending`；32/5 策略仍冻结，不能据此关闭 `RTS-P1-036`。

当前概述（2026-08-26 TransformOrRotate comparison pre-optimization receipt）：Text02 已重新审查
`vertical/orientation -> itemize -> vertical/direct -> vertical/backend` 完整调用链。Unicode VO 先按 grapheme
产生 Upright/Sideways/TransformOrRotate，再由 itemizer 按连续相同 face、instance、BiDi direction/level、script
与 orientation 合并；只有 Tr logical segment 会在正常 TTB/BTT shape 后，以相同 context 和关闭 `vert/vrt2`
的 features 再 shape。因此 comparison work 为 `O(Tr segments)`，最坏 `O(graphemes)`。

本地 Unreal Slate 以一次 HarfBuzz shape 构造 retained `FShapedGlyphSequence`，但该路径只有 LTR/RTL、
`kern/liga`，没有 vertical substitution provenance；Godot Advanced TextServer 使用 TTB/BTT 和单次 feature-aware
shape，也没有发布 Tr cluster 是否实际替代的 receipt。两者都不能支持直接删除 Zircon comparison。RustyBuzz
当前 API 不提供 lookup execution trace，所以 enabled/disabled glyph-sequence 差分仍是当前唯一可证明 decision source。

原 direct request backend-call TLS 已下沉到 `cosmic/direct_profile.rs`。profiling/Tracy 路径现固定发布 8 个
低基数 counter，其中新增 comparison call、input byte、disabled-output glyph 与 changed-cluster 四项；backend
segment 只更新 request-local TLS，成功 direct run 才一次性发布，失败报告丢弃。没有 per-segment event、source
label、局部计时或全局锁。managed harness 新增 `vertical_tr` 1/100/1k/10k、31-sample workload，但尚未执行。

`cosmic.rs` 从 797 降至 722 行，shaping test 根从 774 降至 523 行，profile/test 叶子分别为 129/293 行。
Rust 2024 rustfmt、scoped diff-check、cfg/counter-name/file-budget 静态检查通过。状态为
`vertical_substitution_comparison_receipt_implemented / request_local_capture_only_aggregation_implemented /
algorithm_unchanged / static_checks_complete / managed_profile_pending`；Cargo、counter/timing 数值、allocation/RSS、
功耗、WGPU/PNG 未执行，不关闭 `RTS-P1-019/020`，也不声明算法已最优或接近 Unreal 耗时。

当前概述（2026-08-26 Common/Inherited script-run current-source 校准）：旧优化项所述
`pending_common_start/end` 已不在当前源码。paragraph analysis 以 `unicode-script 0.5.8` 的无堆分配
`ScriptExtension` 位集做前向交集；依赖定义 Common/Inherited 与所有 specific script 兼容，因此前导中性字符
跟随首个 specific script，行内/尾随中性字符跟随前一个兼容 run，纯 Common 文本保持 `Zyyy`。paired bracket
继续由完整 BiDi bracket stack 覆盖，fallback/direct/Cosmic 仍消费同一 paragraph analysis。

本地 Godot script iterator 的 `same_script` 与首个 specific script 回填语义支持相同策略。新增回归固定前导
Common+Inherited、跨脚本中间标点、尾随标点和纯 Common 文本；没有新增 Common 状态、分配、生产分支或复杂度。
状态为 `stale_pending_common_finding_corrected / script_extension_policy_regressions_added /
production_algorithm_unchanged / static_checks_complete / managed_text_test_pending`。

当前概述（2026-08-26 unified vertical cluster decision receipt）：cluster-head shaped flags 现保留
orientation、effective `vert/vrt2` set、substitution proof 与 typed fallback reason。完整
`VerticalGlyphDecision` 通过该 basis 与 glyph 已有 rotation、selected face/instance 零分配组合，neutral
`TextGlyph` 投影继续保留 basis 和 generation-qualified handles；layout/renderer 不重新推断 Unicode VO 或字体选择。

direct Tr 使用既有 enabled/disabled output comparison 发布 `Observed/NotObserved`；no-substitution、Unicode
sideways、forced sideways、backend provenance unavailable 和 non-rendering control 均有独立原因。RustyBuzz 不公开
具体 lookup trace，所以 `vert+vrt2` 同开只报告可证明的 effective set，不猜单一 tag。shape-call 数与算法未改，
新增路径不分配。状态为 `vertical_cluster_decision_basis_implemented /
direct_feature_set_and_substitution_provenance_retained / neutral_projection_preserved /
compatibility_unknown_explicit / static_checks_complete / managed_validation_pending`。

当前概述（2026-08-26 horizontal run-local alternate composition 非验收实现）：direct horizontal 现在保留所有成功
logical segment，并把 backend capability failure 形成 source-sorted hole；Bidi、source、itemization 与 budget 仍由
唯一 failure owner fail closed。Cosmic 只生成一次完整候选，组合器以 run/line identity、单调 source order、完整
hole containment 与非空 coverage 资格化；跨 hole cluster、direct overlap、非单调输出或拓扑不相容均使用原完整候选。

成功 hybrid run 只采用 hole 内 Cosmic glyph，并从最终 selected face IDs 重建行 envelope/raw metrics/span sidecar；
direction、BiDi level、script、font 与 cluster receipt 均来自各自 backend。artifact 保存绝对 alternate ranges 与首因
failure receipt，renderer/UI 不重建或猜测该决策。静态实现与 focused pure regressions 已完成，managed Cargo、fault
injection、Unicode corpus、profile/power 与 WGPU/PNG 仍待验收；whole Cosmic candidate 成本仍存在，不声明性能改善。
状态为 `direct_partial_attempt_implemented / source_ordered_hybrid_composition_implemented /
selected_face_metric_rebuild_implemented / hybrid_artifact_receipt_and_profile_implemented /
fail_closed_whole_candidate_retained / static_checks_complete / managed_validation_pending`。

当前概述（2026-08-26 shaped source lifetime pre-optimization）：`ShapedGlyphRun` 仍持 exact `Arc<str>`，同步 request
无 owner 时在 artifact 最终化分配，parallel hard-line prewarm 则先为每段物化 Arc 再由 run 复用。现已在唯一
materialization 边界聚合 allocation/reuse/bytes，并在 batch owner 区分 logical leases、unique Arc owners、leased bytes
与 unique-owner bytes；算法、source storage 与 cache admission 未改变。

本地 Unreal `FShapedGlyphSequence` 不持源文本，只持 source range/index reverse map，文本由外部 owner 保寿命。
Zircon 的后续方案必须是 immutable document revision snapshot + validated range lease + absolute origin，并保持 exact
collision guard 与 wire slice；cache 对共享 owner 只计一次。managed profile 未执行，因此状态为
`source_lifetime_architecture_research_complete / unreal_external_text_owner_confirmed /
source_materialization_and_batch_owner_instrumentation_implemented / algorithm_unchanged /
static_checks_complete / managed_profile_pending`，不授权 lease 或 glyph SoA 硬切。

当前概述（2026-08-26 cache/artifact identity类型硬切）：current-source确认 shaped、parallel pending、rich、measure/
layout和physical-line hash只做进程内桶定位，完整key与exact source仍决定命中；`TextDocumentKey` 的owner+revision也只用于
当前surface/session，不是持久内容摘要。本地Unreal的`FCachedShapedTextKey::GetTypeHash`采用相同的临时map hash边界。

生产缓存字段已统一为不可序列化且无字节导出的`EphemeralCacheHash`，`DefaultHasher`只存在于唯一builder owner。
`.zsdf` generation/offline identity则统一为`StableContentDigest`，继续逐字节写入原BLAKE3 variation/source槽；v1 codec
负责格式版本，public inspection仍投影`[u8;32]`。没有每viewport全文hash、cache算法、shape-call或SDF格式变化。
状态为`ephemeral_cache_hash_type_implemented / stable_artifact_digest_type_implemented /
default_hasher_isolated / sdf_v1_bytes_unchanged / algorithm_unchanged / static_checks_complete /
managed_validation_pending`；Cargo、golden/collision、profile/power与WGPU/PNG仍开放。

当前概述（2026-08-26 paragraph/lifetime 架构审查）：`SharedTextLayoutSession` 已由 UI measure owner 长期持有，内部
`ShapedRunCache`、`HardLineIndexCache` 与 layout/measure cache 分层；shaped key 命中不会重做 Bidi/script analysis，带
document revision 的 plain viewport 复用共享 hard-line source owner。未证明的结构性重复位于 direct→Cosmic fallback 的
line-break/hard-line materialization，以及 rich advance-index/physical-line/viewport 各自投影。

本轮不新增 retained paragraph artifact；先按 plain/rich、direct-success/partial-fallback/terminal、cold/warm、
1/100/1k/10k hard lines、scroll/edit 采集 analysis 构造、hard-line/line-break bytes、cache miss、DTO current/peak、
allocation/RSS/p50/p95/p99/功耗。只有 profile 证明重复分析主导时，才设计 document-revision artifact 与 dirty-range
依赖；source lease、glyph SoA、renderer artifact 分开立项。状态为`paragraph_lifetime_architecture_review_complete /
duplicate_analysis_instrumentation_deferred / algorithm_unchanged / static_checks_complete / managed_profile_pending`。

- 迁入记录：[`02/2026-07-09-shaping-unicode-and-bidi-output-records.md`](02/2026-07-09-shaping-unicode-and-bidi-output-records.md)
- fixed 已修复：[variable-shaping-visibility-compilation](../../zircon_editor/editor/07/fixed-2026-07-14-variable-shaping-visibility-compilation.md)

## 2026-08-26 TextLayoutError diagnostic catalog boundary

`core/framework/text::TextLayoutError` now exposes a stable `diagnostic_code()` (`ZR-TEXT-LAYOUT-*`)
and a stable `message_key()` (`text.layout.*`). The enum remains backend-neutral: Runtime Text does
not leak face/range payloads into the core contract, and `Display` remains a compatibility/debug
projection rather than a machine-readable protocol. Editor, telemetry, and localization owners can
consume the code/key pair without parsing English text. Static formatting and focused behavior tests
are complete; managed Cargo and integration evidence remain pending.

## 2026-08-26 UI shaper facade hard cut

`UiTextShaperStack` was a one-field forwarding wrapper with no capability ordering or backend
receipt. It is removed. All UI public/provider/viewport/measurement entrypoints now call the sole
`UiSharedTextShaper` adapter directly; actual direct/Cosmic backend composition remains below the UI
boundary in Runtime Text shaping. Local Unreal likewise gives `FSlateFontCache` one concrete
`FSlateTextShaper` and keeps method/script/face decisions inside it. Status:
`empty_ui_shaper_stack_removed / sole_shared_adapter_preserved / source_guard_updated /
static_checks_complete / managed_validation_pending`.

## 2026-08-26 DTO residency pre-migration boundary

Serializable layout/shaped DTOs remain owned serde values. Existing layout-cache accounting covers
line/run text and advances; renderer prepare now adds final post-Auto native/SDF batch count, text
bytes, and advance bytes as a no-raw-text lower-bound receipt. Intermediate paint copies and actual
serialization materialization remain open; no `String`/`Arc`/range/lease migration is authorized
before managed profile evidence. Status:
`layout_dto_and_renderer_batch_residency_receipts_implemented / intermediate_paint_copy_open /
algorithm_unchanged / static_checks_complete / managed_profile_pending`.

## 2026-08-27 Runtime budget ownership and audit receipts

The 8-grapheme boundary context and Arabic 32-tatweel/five-measurement limits are algorithm-local
bounds, not one shared tuning profile. Their owners now expose immutable snapshots and publish
`text.runtime_budget.*` values beside the existing boundary-safety and tatweel candidate receipts.
The default values and shaping/break/justify decisions are unchanged. Managed corpus and 31-sample
profiling must prove a correctness or cost issue before changing them. Status:
`owner_local_budget_snapshots_implemented / runtime_budget_profile_projection_implemented /
algorithm_defaults_unchanged / static_checks_complete / managed_profile_pending`.

## 2026-08-27 Non-ready font capability cause boundary

The request-owned shaping failure catalog now has 14 stable codes: the existing 12 direct/backend
causes plus `FontPrimaryUnavailable` and `FontGenerationChanged`. Missing primary selection is a
terminal font-resolution cause; generation retry, stale cache, and stale worker results are deferred
causes. Session and parallel diagnostics count deferred and terminal runs separately, while the
public layout service continues to expose only neutral `TextLayoutError` values. Candidate ordering,
coverage, backend calls, cache admission, and shaping algorithms are unchanged. Candidate, pending,
policy, and combined backend-capability traces remain open pending the font-runtime owner work.

## 2026-08-27 Request-owned font-resolution work receipt

The synchronous resolver now returns a fixed request report beside the shaped run. It distinguishes
resolution/candidate cache hit-miss, actual coverage probes, complete/partial candidate visits,
coverage rejection, final selection class, shaping attempts, and generation restarts. The report is
carried by a transient completion envelope and is merged by the retained session or parallel batch;
it is not serialized or stored in `ShapedGlyphRun`, so cache hits do not replay one miss's work.

No second coverage pass or candidate allocation was added. Generation retries merge discarded
attempt costs into the final Ready/terminal/deferred outcome. This is the observable synchronous
subset of Runtime Font 80 `RFF-P1-032`, not the full `FontResolveOutcome`: exact candidate identity,
pending dependencies, collection generation, policy rejection, budget exhaustion, and backend
capability remain owned by Runtime Font M3/M5. Static checks pass; managed Cargo, corpus timing,
RSS/power, WGPU, and PNG validation remain pending.

## 2026-08-29 FontObject-scoped shaping selection

Canonical shaping now treats `TextStyle.font` as a registered font-object owner and
`font_family` as its optional typeface selector. The query no longer inserts an asset URI into the
family list. Primary selection first searches that owner's ordered faces; fallback uses the same
owner's generation-compiled CompositeFont and fallback list, then the database's base/platform
fallbacks. Another loaded asset's fallback declaration cannot enter this request.

Cosmic rich spans still receive the resolver-selected physical family, so backend face IDs remain
authoritative. The shaped-run key stores asset identity and family independently, and owner changes
advance font generation. Regressions cover asset-before-project-default selection, asset-local CJK
composite selection, cross-owner fallback isolation, and distinct FontObjects sharing a typeface
name. Rustfmt/static checks pass; no Cargo, multilingual product render, WGPU/PNG, profile, RSS or
power evidence was produced. Status: `font_object_scoped_shaping_static_implemented /
cross_owner_fallback_isolated / managed_validation_pending`.

An explicit but unavailable FontObject now suppresses its owner-local typeface from global matching.
The request falls through to the project/runtime default chain with the same weight/style/stretch; a
registered owner keeps the borrowed query unchanged. A regression registers a global homonym and proves
that an unavailable asset still selects the runtime primary. This request-level correction adds no work to
the grapheme loop and remains under the same managed validation gate.

Candidate-family provenance is retained through deduplication. Request typefaces are owner-local only;
CompositeFont, asset fallback, and base/platform candidates may fall through to global faces. A duplicate
normalized family is upgraded to external authority only when one of those authored fallback sources declares
it. Regressions prove both that a missing local CJK typeface cannot select a global homonym and that an explicit
asset fallback family still can.

The owner face set is now a generation-local `Arc<[FontFaceId]>` built by the registration transaction.
Primary selection and cluster fallback borrow that slice instead of rebuilding it from source keys for each
request or candidate family. Matching order and coverage semantics are unchanged; measured allocation and
timing evidence remains pending.

After the authored fallback chain is exhausted, itemization now selects the packaged
`runtime_last_resort_face` rather than shaping a missing cluster through an arbitrary FontObject primary.
The shaped glyph therefore carries an engine-owned real face/instance lineage; missing diagnostics remain
attached and glyph 0 is not replaced by a hash or codepoint-derived synthetic ID. A source regression binds
an unknown scalar's neutral handle to the last-resort face. Cargo and real raster validation remain pending.

Canonical shaping now acquires a `FontCollectionSnapshot` containing one exact generation and an
`Arc<FontDatabase>`, then passes that same snapshot through direct/fallback/cosmic shaping. The cosmic
thread-local locale cache no longer probes global font state inside the attempt, so one attempt cannot pair a
generation sampled before publication with a database selected afterward. Neutral and artifact projection now
register through the snapshot's collection-qualified registry, and the renderer-facing artifact view retains the
matching registry publication with the database Arc. The bounded retry remains the publication-race fence for new
work; it is no longer the lifetime mechanism for already-issued work. Source regressions are written but not
dynamically executed; status is `collection_bound_projection_and_inflight_lease_static_implemented /
managed_validation_pending`. Session-owned layout/shaping service injection remains open.

The 2026-08-29 owner-ready continuation now carries a `FontCollectionRevision` through the
`TextShapeRunProvider` contract. Physical-line fragments, logical virtual fragments, layout publication
fences, intrinsic/viewport measurement, plain/rich/secure glyph artifact projection, and shaped-run cache
admission compare the exact `(collection_id, generation)` instead of consulting the process generation.
`SharedTextLayoutSession` exposes the matching immutable collection snapshot to consumers that need font
coverage or handle projection, so one layout request cannot shape against one collection and certify metrics
or build an artifact against another. The default provider remains a process-collection adapter for existing
callers. Static format/diff/global-probe scans pass; managed Cargo, mixed-script corpus, WGPU/PNG and profile/
power evidence remain pending. Status is
`collection_revision_provider_contract_static_implemented / managed_validation_pending`.

2026-08-29 locale cache cold-start correction：Cosmic thread-local cache 的 backing state 现在以
`Option<LocaleFontSystemCache>` 延迟创建，首次 `with_font_system` 直接绑定调用方的
`FontCollectionSnapshot`。此前无参数构造会先读取共享 DB，再在同一次请求中刷新为 session DB，
既增加整库复制也让 session isolation 依赖后置修正；本切片已移除该隐式 probe，并补静态 ownership
guard。真实多集合 shaping、Cargo/WGPU/PNG 与 profile/RSS/power 仍待执行，Text02 保持
`managed_validation_pending`。

2026-08-30 rich bidi security owner boundary：shaping 继续只对 admitted logical Unicode 执行 UAX#9、
保留 bidi level 并投影视觉顺序，不在 glyph/layout 层 strip、replace 或插入 isolate。富文本 raw scalar、
HTML entity 与 BBCode control tag 的 content trust/source diagnostic 由 Text07 parser owner 统一承担；
当前四类 source-ranged bounded diagnostic、typed trust/cache identity 与显式栈平衡 gate 已静态完成：
默认 untrusted 允许 mark/balanced isolate 并拒绝 legacy embedding/override，trusted authoring 仍要求平衡。
managed copy/a11y/render 验证待办。该边界防止安全策略破坏逻辑 range、hit test 与 accessibility offset。

2026-08-30 Cosmic hard-line projection fail-closed：`line_from_layout_run` 现在先验证 line/glyph
shaping range 的 checked 加法、UTF-8 边界和行内上界；`normalize_cosmic_hard_lines` 对无法回投到
当前 hard line 的 source range 返回 `InvalidSourceRange`，不再用空串或 `continue` 丢弃 glyph。
这样 alternate backend 的坏 cluster 不能发布缺字的成功 shaped run，错误仍沿现有 typed fallback
receipt 路径传播。静态 18/18、定向 rustfmt 与 diff-check 通过；Cargo 已尝试但在 E 盘依赖 target
写入阶段失败，未进入源码检查；真实多语种 WGPU/PNG、profile/RSS/power 尚未执行，Text02 状态仍为
`managed_validation_pending`。
