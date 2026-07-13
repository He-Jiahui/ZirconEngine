---
related_code:
  - zircon_runtime/src/asset/assets/font.rs
  - zircon_runtime/src/asset/importer/ingest/import_font_asset/mod.rs
  - zircon_runtime/src/asset/importer/ingest/import_font_asset/parse_sfnt.rs
  - zircon_runtime/src/ui/text/font_registry.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/font_asset.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_font_bake.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs
  - zircon_runtime/src/core/framework/render/text/font/mod.rs
  - zircon_runtime/src/graphics/text/font/database.rs
  - zircon_runtime/Cargo.toml
design_references:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts/CompositeFont.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/CompositeFont.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts/FontFaceInterface.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts/SlateFontInfo.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/FontCacheFreeType.cpp
  - dev/godot/scene/resources/font.h
  - dev/godot/editor/import/resource_importer_dynamic_font.cpp
  - dev/bevy/crates/bevy_text/src/font.rs
  - dev/bevy/crates/bevy_text/src/font_loader.rs
  - dev/Fyrox/fyrox-ui/src/font/loader.rs
plan_sources:
  - docs/plans/zircon_runtime/text/index.md
  - docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md
  - docs/plans/engine-code-structure-convention.md
status: in_progress
---

# 01 字体资源 / FontFace / CompositeFont / 字体库

> 本计划是文本主链的最上游:把"一个字体文件"变成"shaping 可消费的 face + family + 回退链 + 变量实例"。承接 `editor_ui/03 §2.2` 缺口 3(字体注册表缺失)。回退**算法**在 `06`,本计划只定义回退链**数据结构与字体库索引**。

## 1. 目标

1. **字体文件处理**:支持 TTF / OTF / TTC(集合)/ WOFF2;face 索引(一个文件多 face);变量字体(`fvar` 轴 + 命名实例);字体二进制按 `Arc<[u8]>` 零拷贝共享。
2. **FontFace 分层**:`FontFace`(单一物理 face,绑定字体数据 + face index)/ `FontFamily`(同族不同 weight/style/stretch 的 face 集)/ `CompositeFont`(family + 按脚本/范围的回退链,对齐 UE `FCompositeFont`)。
3. **字体库与系统字体发现**:进程级 `FontDatabase`(fontdb)索引项目字体资产 + 系统字体;按 `(family, weight, style, stretch)` 查询 best-match;按 codepoint/script 枚举回退候选(喂给 `06`)。
4. **资产与导入升级**:`FontAsset` 从"单 source + family + render_mode"升级为可声明 family 成员、变量实例、回退链、render 策略。

## 2. 现状与差距

- `FontAsset` 已扩展 face index、family members、变量实例、fallback family、render strategy、parsed metadata 与中立 `CompositeFontDescriptor`；FR-M2 补齐 face 级 typographic/Windows/装饰线 metrics，FR-M3 补齐按 Unicode range/script/culture 的复合字体资产 schema。
- 字体导入已硬切到 `import_font_asset/{mod.rs,parse_sfnt.rs,parse_sfnt/tests/}`；`asset/assets/font_source.rs` 统一处理 WOFF2→SFNT 解码与 TTC 选定 face 的 standalone SFNT 提取。真实 transformed-glyf WOFF2、合成 `fvar` 命名实例、TTC face 1 SDF raster focused tests 已落代码；library check 已绿，focused lib-test 当前被活动 plugin-extension 会话的非文本 E0282 编译错误阻断，断言尚未执行。
- UI 不再自持默认 family 字面量；默认链由 `graphics/text/font/default_families.rs` 统一提供，项目默认字体 manifest 声明 Fira Mono default 与 Noto CJK/system family culture routes。`SystemFontPolicy` 默认 `Disabled`，只有 screen-space renderer 显式选择 `Discover`。
- native glyphon 与 SDF 均通过 `FontDatabase` 的中立 `FontFaceId`/共享 `Arc<[u8]>` 消费项目字体；SDF 对 TTC 非零 face 通过 standalone face bytes 构造 `fontsdf::Font`，不再静默使用 face 0。
- `FontDatabase` 已具项目/系统 face 索引、best-match、coverage/fallback 候选、variation instance id、共享 bytes 与 active project composite 投影；剩余工作归 02/06 的真实 locale/per-script fallback 贯穿和 09 的完整 cache invalidation 闭环。

## 3. 参考代码

| 文件 | 应重点阅读 |
|------|-----------|
| `dev/UnrealEngine/.../Fonts/CompositeFont.h` + `CompositeFont.cpp` | `FCompositeFont`/`FTypeface`/`FTypefaceEntry`/`FCompositeSubFont`:default typeface + 按 Unicode block range 的 sub-font 回退;`FFontData` 的 inline/bulk 数据持有。本计划 `CompositeFont`/`FontFace` 数据结构主样板 |
| `dev/UnrealEngine/.../Fonts/FontFaceInterface.h` + `SlateFontInfo.h` | `IFontFaceInterface`/`FFontFaceData`/`FSlateFontInfo`(`FontObject + TypefaceFontName + Size + OutlineSettings`):字体信息如何打包成 shaping 请求键 |
| `dev/godot/scene/resources/font.h` | `Font`/`FontFile`:`fallbacks` 链、`face_index`、`variation_coordinates`(变量轴坐标 map)、`opentype_features`、动态/位图/imagefont 三类 face。本计划变量字体与 face 索引样板 |
| `dev/godot/editor/import/resource_importer_dynamic_font.cpp` | 导入选项:`fonts/fallbacks`、`fonts/face_index`、`variation_*`、`fonts/generate_mipmaps`、`msdf` 开关——本计划导入 schema 对照 |
| `dev/bevy/crates/bevy_text/src/font.rs` + `font_loader.rs` | Rust 落地:`Font` 包 face 数据 + `FontLoader` 解析;`FontAtlasKey { variations_hash }` 把变量轴坐标进缓存键 |
| `dev/Fyrox/fyrox-ui/src/font/loader.rs` | 极简 Rust 字体加载(`FontImportOptions`),所有权组织参照 |

**Rust/wgpu 落地**:`fontdb::Database`(系统字体发现 + family 查询 + face 枚举)、`ttf-parser`(cosmic-text 内部,读 `fvar`/`cmap`/`name`/`os2`)。变量实例化坐标进缓存键(bevy `variations_hash` 同法)。

## 4. 目标架构

归属:契约层 `core/framework/render/text/font/`(纯数据,serde);实现层 `graphics/text/font/`(持 `Arc<[u8]>` + fontdb 索引,`fontdb`/`ttf-parser` 隔离于此)。

```
FontAsset(磁盘 TOML + 字体文件)
  └─importer→ FontResource { faces: Vec<FontFace>, family_decl, fallback_decl, variations }
        └─register→ FontDatabase(进程级,fontdb 索引 + ZirconEngine face 句柄表)
              ├─query(family, weight, style, stretch) → FontFaceId(best-match)
              ├─query_fallback(codepoint/script) → Vec<FontFaceId>(喂 06)
              └─instance(FontFaceId, variation_coords) → InstancedFaceId(进 shaping)
```

## 5. 里程碑

### FR-M1 中立 FontFace 与 FontDatabase

实施切片:
1. 契约层 `font/face.rs`、`font/database.rs` 类型定稿;实现层 `graphics/text/font/` 接 `fontdb`,索引项目字体 + 可选系统字体发现;`FontFaceId`/`InstancedFaceId` 句柄。
2. `ui/text/font_registry.rs` 硬编码链改为查 `FontDatabase`(数据驱动);glyphon `FontSystem` 与 SDF bake 改为消费同一 `FontDatabase`(字体二进制单次加载、`Arc` 共享)。

测试:`text_font_database_query_best_match_*`、`text_font_face_shares_arc_bytes`;`cargo check -p zircon_runtime --lib`。

### FR-M2 字体文件解析与变量字体

实施切片:
1. 导入器解析字体头:face 数(TTC)、`fvar` 轴 + 命名实例、`cmap` 覆盖码点位集(喂 `06` 回退命中预筛)、`OS/2` weight/width/style;另补(2026-07-02 评审收口)`post.underlinePosition`/`post.underlineThickness` 与 `OS/2 yStrikeoutPosition`/`yStrikeoutSize`(装饰线度量,供 05 SM-M4 下划线/删除线消费)。
2. `FontAsset` schema 升级:family 成员声明、变量实例、回退链、render 策略;WOFF2 解压接入(`woff2` decode → TTF)。
3. 变量轴坐标进 `InstancedFaceId` 与 shaping/atlas 缓存键(`variations_hash`)。

测试:`text_font_parse_ttc_enumerates_faces`、`text_font_variable_axes_roundtrip`、`text_font_woff2_decodes_to_sfnt`、`text_font_cmap_coverage_bitset`。

### FR-M3 CompositeFont 与回退链数据面

实施切片:
1. `CompositeFont`(default family + 按 script/Unicode-range 的 sub-font 列表,对齐 UE `FCompositeFont`);编辑器默认字体包(含 CJK Noto/思源)声明为 `CompositeFont` 资产。
2. 回退链查询 API 定稿(算法实现归 `06`,本里程碑只交付数据结构 + 候选枚举)。

测试:`text_composite_font_resolves_default_and_subfont_ranges`;编辑器默认字体包加载回归。

## 6. 工程落地细化(实施权威)

### 模块与文件落点

**契约层 `zircon_runtime/src/core/framework/render/text/font/`**(serde,无第三方句柄):

| 文件 | 内容 |
|------|------|
| `mod.rs` | 薄声明 + re-export |
| `face.rs` | `FontFaceDescriptor`、`FontWeight`、`FontStyle`、`FontStretch`、`FaceIndex`、`VariationCoords` |
| `family.rs` | `FontFamilyName`、`FontFamilyDescriptor`(family→face 列表) |
| `composite.rs` | `CompositeFontDescriptor`、`SubFontRange`(script/UnicodeRange→family) |
| `database.rs` | `FontFaceId`、`InstancedFaceId`、`FontQuery`、`FontMatch`(纯查询请求/结果 DTO) |

**实现层 `zircon_runtime/src/graphics/text/font/`**:

| 文件 | 内容 |
|------|------|
| `mod.rs` | `FontDatabase` 装配(薄) |
| `fontdb_index.rs` | **`fontdb`/`ttf-parser` 隔离层** —— 系统/项目字体索引、best-match、回退候选枚举;出口只给 `FontFaceId`/`FontMatch` |
| `face_store.rs` | `FontFaceId → Arc<FaceBytes> + face_index + 解析元数据(轴/cmap 位集/metrics)`。度量优先级规则(2026-07-02 评审收口):OS/2 `fsSelection.USE_TYPO_METRICS` 置位取 `sTypoAscender/sTypoDescender/sTypoLineGap`,否则取 `hhea` 度量,`usWinAscent/usWinDescent` 仅作裁剪参考;baseline 统一 alphabetic(D7) |
| `instance.rs` | `InstancedFaceId` 缓存(face + variation coords 量化);`variations_hash` |
| `composite_resolve.rs` | `CompositeFont` → 有序回退 `FontFaceId` 列表的解析(供 `06` 消费) |

**资产/导入(`zircon_runtime/src/asset/`)**:`assets/font.rs` 已升级 `FontAsset` schema 并承载 `CompositeFontDescriptor`；`assets/font_source.rs` 独占 WOFF2 解码/TTC standalone face 提取；`importer/ingest/import_font_asset/{mod.rs,parse_sfnt.rs,parse_sfnt/tests/}` 承接 metadata 与 folder-backed fixture tests（遵 `runtime/15`）。

### 核心类型(契约层)

```rust
// face.rs
pub struct FontFaceDescriptor {
    pub family: FontFamilyName,
    pub weight: FontWeight,   // 100..=900(OS/2 usWeightClass)
    pub style: FontStyle,     // Normal | Italic | Oblique(angle)
    pub stretch: FontStretch, // UltraCondensed..=UltraExpanded
    pub face_index: u32,      // TTC 内索引
    pub variations: VariationCoords, // 变量轴 tag→value(空=默认实例)
}
pub struct VariationCoords(pub Vec<(/*tag*/ u32, /*value*/ f32)>); // 'wght'/'wdth'/'slnt'/'ital'/'opsz'/自定义
#[derive(Clone, Copy)] pub struct FontFaceId(pub u64);      // FontDatabase 内稳定 id
#[derive(Clone, Copy)] pub struct InstancedFaceId(pub u64); // face + 量化变量坐标

// database.rs
pub struct FontQuery<'a> { pub families: &'a [FontFamilyName], pub weight: FontWeight,
    pub style: FontStyle, pub stretch: FontStretch }
pub struct FontMatch { pub face: FontFaceId, pub synthetic_bold: bool, pub synthetic_oblique: bool }
// synthetic_bold/oblique 注(2026-07-02 评审收口):消费方为 04 `GlyphRasterKey.synthetic: SyntheticFlags`
// (bold=swash embolden,oblique=quad shear);合成标志经栅格键显式携带,不进缓存键即污染,见 04。

// composite.rs(对齐 UE FCompositeFont)
pub struct CompositeFontDescriptor {
    pub default_family: FontFamilyName,
    pub sub_fonts: Vec<SubFontRange>, // 按优先序;命中则用其 family
}
pub struct SubFontRange { pub family: FontFamilyName,
    pub scripts: Vec<Script>, pub ranges: Vec<(u32, u32)>, // Unicode block range,对齐 UE UnicodeBlockRange
    pub cultures: Vec<LocaleTag> } // 可选(2026-07-02 评审收口):与 02 `TextShapeRequest.language` 字段联动,
                                   // 用于 Han 消歧(zh-Hans/zh-Hant/ja/ko 分派不同 CJK sub-font),对齐 UE FCompositeSubFont::Cultures
```

### `FontDatabase` 接口(实现层)

```rust
impl FontDatabase {
    pub fn register_asset(&mut self, font: &FontResource) -> Vec<FontFaceId>;
    pub fn load_system_fonts(&mut self);                       // fontdb 系统枚举(可关,headless 不调)
    pub fn match_face(&self, q: &FontQuery) -> Option<FontMatch>; // best-match(weight 距离 + style/stretch)
    pub fn fallback_candidates(&self, cp: char, base: &FontQuery) -> Vec<FontFaceId>; // cmap 命中 + script 过滤(喂 06)
    pub fn instance(&mut self, face: FontFaceId, vars: &VariationCoords) -> Result<InstancedFaceId, FontDatabaseError>; // fvar clamp/default-drop/F2DOT14 normalized quantization 后登记
    pub fn face_bytes(&self, face: FontFaceId) -> Arc<FaceBytes>; // 零拷贝;glyphon/SDF/MSDF 共享
    pub fn unregister_asset(&mut self, asset: &FontAssetSourceKey) -> Vec<FontFaceId>; // 资产卸载/替换;返回被失效的 face(2026-07-02 评审收口)
    pub fn invalidate_face(&mut self, face: FontFaceId);          // 单 face 失效,触发下述失效级联(2026-07-02 评审收口)
}
```

best-match 权重距离用 CSS Fonts L4 算法(weight 优先就近、style Italic>Oblique>Normal、stretch 就近),对照 fontdb `Database::query`。

### 失效级联(2026-07-02 评审收口)

face 失效(`unregister_asset`/`invalidate_face`,来源:资产热重载、字体包卸载、变量实例回收)必须按以下固定顺序级联,任何一级不得跳过:

1. `ShapedRunCache` 按 base `font_id: FontFaceId` 与 `font_instance_id: InstancedFaceId` 剔除全部命中条目；base face 失效必须覆盖其所有 instance；
2. `LayoutCache` 连带剔除(其键含 shaped key,见 D6/09);
3. `GlyphRasterKey` 索引按 face 剔除,对应 atlas slot 标脏回收;
4. SDF bake cache(含离线 `.zsdf` 预填页的内存驻留项)按 face 剔除。

09 缓存契约表为每级缓存持"失效来源"列,本级联是其中"字体失效"来源的权威定义(与 09 修订呼应)。

### 与既有路径的硬切换

| 现有 | 切换 |
|------|------|
| `ui/text/font_registry.rs` 硬编码链 | 删除硬编码常量,改注入 `FontDatabase` + `CompositeFontDescriptor`(默认包资产);保留函数名供调用方不变 |
| glyphon `FontSystem::new()` 自建 fontdb | 改 `FontSystem::new_with_locale_and_db(locale, shared_db)`,共享 `FontDatabase` 的 fontdb 实例 |
| `sdf_font_bake.rs` 各自加载字体 | 改 `face_bytes(FontFaceId)` 取 `Arc`,不再重复读盘 |

### 测试与验收清单

| 测试 | 断言 |
|------|------|
| `text_font_database_query_best_match_weight_distance` | (Inter, 700) 命中 Bold face;无 700 时就近(600/800) |
| `text_font_face_shares_arc_bytes_across_backends` | 同 face 在 glyphon/SDF 路径 `Arc::ptr_eq` |
| `text_font_parse_ttc_enumerates_faces` | TTC 文件枚举出 ≥2 face,face_index 正确 |
| `text_font_variable_axes_roundtrip` | `fvar` 轴 tag/min/max/default 解析正确;命名实例坐标可还原 |
| `text_font_variations_hash_stable` | 同坐标 `variations_hash` 稳定、不同坐标不碰撞 |
| `text_font_woff2_decodes_to_sfnt` | WOFF2 解压后 sfnt 头有效、face 可解析 |
| `text_font_cmap_coverage_bitset_matches_face` | cmap 覆盖位集与逐码点 `glyph_index` 一致 |
| `text_composite_font_resolves_default_and_subfont_ranges` | CJK 码点解析到 Noto CJK sub-font,拉丁解析到 default |

里程碑命令:`cargo test -p zircon_runtime text_font --locked`、`text_composite --locked`。

## 7. 状态与产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

当前概述（2026-07-13）：系统 `fontdb` face 已能从权威 backend ID 物化容器字节，并按记录的 collection face index 提取独立 SFNT；`Microsoft YaHei UI` 已通过同一 `FontDatabase` 进入 production SDF CJK 竖排帧。Screen-space renderer 构造现在对共享快照重新应用 `SystemFontPolicy::Discover` 后再同步 glyphon `FontSystem`，默认空项目快照不再清空系统 face；Editor HUD 使用与 Runtime 产品门禁一致的 24 帧异步 glyph 收敛窗口后，真实帧缓冲回归为 1/1。项目字体、系统字体、native shaping 与 SDF 继续共享一个 face-ID/字节 lineage，没有新增 Editor 旁路数据库、硬编码平台字体或兼容层。FR-M2 变量轴已从仅有哈希推进到可反查实例、horizontal/vertical RustyBuzz、native Swash、dynamic SDF/MSDF/MTSDF 与 atlas/offline identity 的同一坐标链；horizontal leaf 同时承接 cosmic-text `Attrs` 缺失的 per-run language→`locl`，并显式拒绝竖排请求。当前源 Windows exact `text_horizontal_` 已 5/5，通过真实 `Bahnschrift` width-axis、真实 `Calibri` 俄语/塞尔维亚语 `locl`、空语言/竖排边界和 renderer face/instance identity；F2DOT14 数据库量化、dynamic SDF 像素、atlas instance separation 与产品帧仍待继续验收，因此本计划保持 `in_progress`。跨平台 CJK fixture 继续保持 open。

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

- 迁入记录：[`01/2026-07-10-font-resource-faces-and-database-output-records.md`](01/2026-07-10-font-resource-faces-and-database-output-records.md)
- 已修复交接（`fixed / 2026-07-11`）：[`Editor 01/fixed-2026-07-11-editor-m1-font-discovery.md`](../../zircon_editor/editor/01/fixed-2026-07-11-editor-m1-font-discovery.md)；当前 HUD glyph capture 上层 exact 为 1/1，功能计划保留回链与摘要，不保留重复真相。
- fixed 已修复：[reflection-probe-product-type-inference](01/fixed-2026-07-12-reflection-probe-product-type-inference.md)
- fixed 已修复：[font-decoration-display-size-argument](../../zircon_editor/editor/09/fixed-2026-07-13-font-decoration-display-size-argument.md)
