---
related_code:
  - zircon_runtime/src/asset/assets/font.rs
  - zircon_runtime/src/asset/artifact/cache_payload/font.rs
  - zircon_runtime/src/asset/importer/ingest/import_font_asset/mod.rs
  - zircon_runtime/src/asset/importer/ingest/import_font_asset/parse_sfnt.rs
  - zircon_runtime/src/text/font/asset_registration.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/font_asset.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/text/sdf/font_bake.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs
  - zircon_runtime/src/text/model/font/mod.rs
  - zircon_runtime/src/text/font/database.rs
  - zircon_runtime/src/text/font/database/equivalence.rs
  - zircon_runtime/src/text/font/database/fallback_queries.rs
  - zircon_runtime/src/text/font/database/tests/system_policy.rs
  - zircon_runtime/src/text/font/shared.rs
  - zircon_runtime/src/text/font/source_manifest.rs
  - zircon_runtime/src/text/render_state.rs
  - zircon_runtime/src/text/language.rs
  - zircon_runtime/Cargo.toml
design_references:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts/CompositeFont.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/CompositeFont.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/LegacySlateFontInfoCache.cpp
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

## Cross-Plan Failure Status

- fixed 已修复：[milestone-finalize-session-relative-owned-scope](01/fixed-2026-07-15-milestone-finalize-session-relative-owned-scope.md)
- fixed 已修复：[milestone-session-relative-line-ending-drift](01/fixed-2026-07-15-milestone-session-relative-line-ending-drift.md)
- fixed 已修复：[goal-closeout-counts-terminal-failed-intents](01/fixed-2026-07-18-goal-closeout-counts-terminal-failed-intents.md)
- current MVP product gate 已完成：2026-07-28 当前源码的 managed Windows WGPU framebuffer 运行 `785e1f2f52fd44778f489e01c4bcd2b8` 已通过（`1 passed / 0 failed`），并将真实 1080x2000 文本/布局截图写入 `docs/tests/runtime/text/`；详情见 [M2 manifest](01/2026-07-26-text-mvp-foundation-f1-milestone-manifest.md)。Deferred 与 Editor 的外部返回仍按各自 owner 跟踪，本计划状态保持 `in_progress`。
- current-source FontDatabase gate 已完成：2026-07-28 managed `text::font` job `4bf5b5da066b48e3b3fe4e56664351a3` / run `ed80cc6a895c40c48d57a2b8b91d618c` 在 Runtime11 child-module routing 修复后通过（`83 passed / 0 failed / 2 ignored`）。这证明 Runtime11 E0583 不再遮蔽 Text01；Editor 上行回传仍由其 owner 独立跟踪，本计划继续 `in_progress`。

## 1. 目标

1. **字体文件处理**:支持 TTF / OTF / TTC(集合)/ WOFF2;face 索引(一个文件多 face);变量字体(`fvar` 轴 + 命名实例);字体二进制按 `Arc<[u8]>` 零拷贝共享。
2. **FontFace 分层**:`FontFace`(单一物理 face,绑定字体数据 + face index)/ `FontFamily`(同族不同 weight/style/stretch 的 face 集)/ `CompositeFont`(family + 按脚本/范围的回退链,对齐 UE `FCompositeFont`)。
3. **字体库与系统字体发现**:进程级 `FontDatabase`(fontdb)索引项目字体资产 + 系统字体;按 `(family, weight, style, stretch)` 查询 best-match;按 codepoint/script 枚举回退候选(喂给 `06`)。
4. **资产与导入升级**:`FontAsset` 从"单 source + family + render_mode"升级为可声明 family 成员、变量实例、回退链、render 策略。

```zircon-workflow
{
  "schema": 1,
  "workflow_id": "zircon-runtime-font-resource-faces-and-database",
  "goal": "在既有 FR-M1 中立 FontFace 与 FontDatabase 基线上，完成变量字体及 CompositeFont 数据面",
  "milestones": [
    {"id": "M2", "title": "字体文件解析与变量字体", "depends_on": []},
    {"id": "M3", "title": "CompositeFont 与回退链数据面", "depends_on": ["M2"]}
  ]
}
```

## 2. 现状与差距

- `FontAsset` 已扩展 face index、family members、变量实例、fallback family、render strategy、parsed metadata 与中立 `CompositeFontDescriptor`；FR-M2 补齐 face 级 typographic/Windows/装饰线 metrics，FR-M3 补齐按 Unicode range/script/culture 的复合字体资产 schema。
- 字体导入已硬切到 `import_font_asset/{mod.rs,parse_sfnt.rs,parse_sfnt/tests/}`；`asset/assets/font_source.rs` 统一处理 WOFF2→SFNT 解码与 TTC 选定 face 的 standalone SFNT 提取。真实 transformed-glyf WOFF2、合成 `fvar` 命名实例、TTC face 1 SDF raster focused tests 已落代码；library check 已绿。Text01 的 fresh `text_font_asset` focused lib-test 已运行两次：test-owner child relocation 的三个本地诊断已修复，第二次在 Runtime11-owned `AssetWorkerCompletionTicket: Debug` 共享支持层错误前停止。Runtime11 回传后必须重跑作为故障回传前的当前源码证据；不再归因为 Editor11 共享 serialization。
- UI 不再自持默认 family 字面量；默认链由 `text/font/default_families.rs` 统一提供，项目默认字体 manifest 声明 Fira Mono default 与 Noto CJK/system family culture routes。`SystemFontPolicy` 默认 `Disabled`，只有 screen-space renderer 显式选择 `Discover`。
- native glyphon 与 SDF 均通过 `FontDatabase` 的中立 `FontFaceId`/共享 `Arc<[u8]>` 消费项目字体；SDF 对 TTC 非零 face 通过 standalone face bytes 构造 `fontsdf::Font`，不再静默使用 face 0。
- `FontDatabase` 已具项目/系统 face 索引、best-match、coverage/fallback 候选、variation instance id、共享 bytes 与 active project composite 投影。2026-07-28 当前源码的 managed Windows WGPU 产品门已通过；`text_font_asset` focused 回归已证明 Text01 child-owner 编译诊断消失，但在 Runtime11 共享 asset-worker regression 前停止，待其下层修复回传后重跑。剩余工作归 02/06 的真实 locale/per-script fallback 贯穿和 09 的完整 cache invalidation 闭环。

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

归属:契约层 `text/model/font/`(纯数据,serde);实现层 `text/font/`(持 `Arc<[u8]>` + fontdb 索引,`fontdb`/`ttf-parser` 隔离于此)。

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
1. 契约层 `font/face.rs`、`font/database.rs` 类型定稿;实现层 `text/font/` 接 `fontdb`,索引项目字体 + 可选系统字体发现;`FontFaceId`/`InstancedFaceId` 句柄。
2. 删除 UI-local 字体注册表，FontAsset 生命周期直接更新 `FontDatabase`；glyphon `FontSystem` 与 SDF bake 消费同一 `FontDatabase`（字体二进制单次加载、`Arc` 共享）。

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

**契约层 `zircon_runtime/src/text/model/font/`**(serde,无第三方句柄):

| 文件 | 内容 |
|------|------|
| `mod.rs` | 薄声明 + re-export |
| `face.rs` | `FontFaceDescriptor`、`FontWeight`、`FontStyle`、`FontStretch`、`FaceIndex`、`VariationCoords` |
| `family.rs` | `FontFamilyName`、`FontFamilyDescriptor`(family→face 列表) |
| `composite.rs` | `CompositeFontDescriptor`、`SubFontRange`(script/UnicodeRange→family) |
| `database.rs` | `FontFaceId`、`InstancedFaceId`、`FontQuery`、`FontMatch`(纯查询请求/结果 DTO) |

**实现层 `zircon_runtime/src/text/font/`**:

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
| UI-local fallback registry | 已硬删除；`FontDatabase` + `CompositeFontDescriptor`（默认包资产）是唯一 owner，不保留兼容函数或 facade |
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

当前概述（2026-07-14）：系统 `fontdb` face、项目字体、native shaping 与 SDF 继续共享一个 face-ID/字节 lineage；FR-M2 变量轴已贯通可反查实例、horizontal/vertical RustyBuzz、native Swash、dynamic SDF/MSDF/MTSDF 与 atlas/offline identity。Screen-space preparation 在 atlas 前按每帧唯一资产 URI 加载正式项目字体，失败保持可重试，成功加载与真实 face-count delta 分离；raw VerticalRl 批次清除内部旧 advances 后重塑形，resolved layout-line 保留其权威 advances。fallback span 同时携带 logical family、`FontFaceId` 与 `InstancedFaceId`，cosmic 投影使用有序 span 的 `partition_point`，避免同一物理 family 的逻辑变量实例合并且不引入 O(glyph×spans) 扫描。整个产品 exporter 仅在 Windows 编译，正式临时项目只导入一个 Bahnschrift face，并用真实 `wdth` min/max 形成两个逻辑实例。最终 post-review managed GPU job `d80d6dabac754907b50aa3ae2c1c1056` 为 1/1，PNG 目视与像素验收得到 narrow 256px/3187px、wide 346px/3747px、差异 4984px；focused jobs `61aaa263af684ab7b028956c772e0a20`、`deb789dcbdbe43c3b17fea6a234c9079` 同时取得 `text_font` 41/41、`text_horizontal_` 6/6、dynamic SDF/atlas/Swash 各 1/1。独立复审为 `Accept`，无 Critical/Important 遗留，FR-M2 已完成。FR-M3 CompositeFont 默认包与跨平台 CJK WGPU fixture 已由 [FR-M3 child acceptance](01/2026-07-14-fr-m3-composite-font-default-package-acceptance.md) 接受；Text01 仍因后续共享 FontDatabase failure recovery 与当前源 managed gates 保持 `in_progress`，该历史验收不替代新的回归验证。

2026-07-17 Text MVP 基础设施切片已实现共享字体库的 render-input 语义等价判定：fallback、CompositeFont、默认 UI family 与有序 face/source 任一真实变化才推进 generation；等价的 screen-space renderer 重建不再让 shaping/SDF cache 全量失效。数据库替换与 generation 递增在同一写锁临界区，snapshot 不会取得“新库 + 旧代际”；常规重复发布优先走共享字节 `Arc::ptr_eq`，不把全字体字节扫描放进热路径。`apply_system_font_policy(Discover)` 也在 FontDatabase owner 内变为幂等；renderer clone 继承已发现状态，不再重复扫描系统字体目录或让 `fontdb` backend catalog 追加重复 face。`TextRenderState::new` 现按 text-owned system locale 直接从共享 backend DB 构造 cosmic `FontSystem`，删除先 `FontSystem::new()` 扫描再覆盖的第二次 OS 字体 I/O；`sys-locale` 仅随 `text` feature 启用。当前 shared-publication 旧 focused batch 为 2/2，新 locale/idempotent-discovery/default-family guard 与并行 SDF 回归仍待协调器测试阶段执行，因此本切片状态为 `implemented / validation_pending`，Text01 仍保持 `in_progress`。

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

- 迁入记录：[`01/2026-07-10-font-resource-faces-and-database-output-records.md`](01/2026-07-10-font-resource-faces-and-database-output-records.md)
- 已修复交接（`fixed / 2026-07-11`）：[`Editor 01/fixed-2026-07-11-editor-m1-font-discovery.md`](../../zircon_editor/editor/01/fixed-2026-07-11-editor-m1-font-discovery.md)；当前 HUD glyph capture 上层 exact 为 1/1，功能计划保留回链与摘要，不保留重复真相。
- fixed 已修复：[reflection-probe-product-type-inference](01/fixed-2026-07-12-reflection-probe-product-type-inference.md)
- fixed 已修复：[font-decoration-display-size-argument](../../zircon_editor/editor/09/fixed-2026-07-13-font-decoration-display-size-argument.md)
- fixed 已修复：[runtime-text-ui-system-constructor-drift](../../zircon_editor/editor_layout/15/fixed-2026-07-14-runtime-text-ui-system-constructor-drift.md)
- fixed 已修复：[ui-text-module-split-import-drift](../../zircon_editor/editor/02/fixed-2026-07-14-ui-text-module-split-import-drift.md)
- fixed 已修复：[dynamic-scene-format-version-root-export-drift](01/fixed-2026-07-14-dynamic-scene-format-version-root-export-drift.md)
- fixed 已修复：[font-database-render-input-equivalence-visibility](../../zircon_editor/editor/01/fixed-2026-07-17-font-database-render-input-equivalence-visibility.md)
- fixed 已修复（Runtime15）：[screen-space-ui-text-font-id-report-mount-drift](01/fixed-2026-07-17-screen-space-ui-text-font-id-report-mount-drift.md)；Runtime15 已补齐真实生产调用的 child-owner 守卫并完成 failure return，Text01 不再保留过期的 open 状态或失效链接。
- 2026-07-19 PERF-MVP-250 UI font manifest cache 已完成实现：`font_assets` 由仅缓存成功值切为 `(resource revision, resource state)` owned `Ready / Missing / Error` 三态记录；同 identity 的 missing/error ensure 直接命中负缓存，revision 或 Error/Reloading/Ready state 变化后重新加载并覆盖旧状态。Framework `ResourceManager` 提供只含 revision/state 的 `ResourceCacheIdentity`，ProjectAssetManager 在一次 registry 读锁内投影两个 Copy 字段，不克隆完整 ResourceRecord；每帧 refresh 对显式 font asset 与 code-style 派生 default asset 按唯一资产执行一次，native 每文本 family resolve 只读已准备 cache。默认字体初始化、Auto render-mode 路由和 native family resolve 统一消费该 cache；默认资产每次 refresh 同步更新或清除 default family/composite projection，native atlas 失效改按真实 face-count delta 判定，负记录插入不再误报 face change。行为测试已覆盖 Missing/Error 重复命中、同 revision Error -> Ready、Missing -> Ready project revision、code-style 派生 default 预取守卫，以及默认 family/composite A -> B -> None；Rust 1.94.1 rustfmt、scoped diff-check 已通过，fresh managed compile/focused tests 仍按 CPU FIFO 等待，因此当前状态为 `implemented / managed_validation_pending`，不得标记 fixed。
- 2026-07-22 Text MVP 字体所有权闭环已收敛到单一共享 `FontDatabase`：生产侧 replace/remove/default/composite 只在 Text-owned 写锁内更新权威库，不再 clone 后发布第二份真值；generation 只在 face/source/fallback/composite/default family 等渲染输入真实变化时推进。字体资产以稳定 asset reference 作为 owner，同一物理 face 可由多个资产共同持有；当时由独立 `asset_mapping_changed` 驱动 owner 的 `Missing -> Ready`、`Ready -> Missing` UI reshape/bitmap/SDF 失效，并允许共享 face 时数据库 generation 不变。该历史 generation 规则已被 2026-08-29 FontObject 可选择性语义取代：owner attach/remove/composite/fallback 映射本身属于 render input，必须推进 generation；物理 face/source 仍保持去重。renderer 已删除构造期重复 publish、face-count 推断失效和 graphics 对完整字体库的读取，只通过 Text-owned `font_face_id` 窄查询解析 backend id。最低层与 UI 多 owner 生命周期测试、19 项静态边界守卫以及两轮独立 review 均为 `0 Critical / 0 Important / 0 Minor`；当前源码 WGPU 产品门与 Runtime15 窄边界结构守卫已通过，测试根文件已按资产缓存、渲染、报表和共享 fixture 硬切到 folder-backed child owner。fresh managed `text_font_asset` focused test 已越过本次 child-owner 编译修复，但在 Runtime11 的共享 asset-worker regression 前停止；其修复回传和 Editor 上行回归仍待闭合，因此状态保持 `implemented / review_green / managed_validation_pending`，Text01 仍为 `in_progress`。
- 2026-07-22 TTC extraction性能补充：PERF-MVP-524已把standalone face每table的临时Vec+二次copy改为直接写最终SFNT并原位清`head.checkSumAdjustment`；Text01验收新增table count/bytes/scratch与checksum counter，要求scratch=0、face bytes owner=1。更高层face/source generation继续服从既有唯一共享FontDatabase，不建立第二份提取cache。
- 2026-08-24 Text01 P1-6 字体 source failure contract 已开始硬切：`text/font/source_manifest.rs` 的 loader、UI manifest adapter 与 `TextRenderState::replace_font_source` 均改为 typed `Result`，不再把 URI、manifest、canonicalize、安全越界和注册失败压成 `Option`。`FontLoadError` 保存稳定的 project/manifest/source phase 与 `NotFound`/`PermissionDenied`/`Other` IO cause；直接字体字节读取和解码失败也在 UI/SDF adapter 映射为稳定分类，不再退化为通用注册失败。resource-backed UI negative cache、SDF face cache、offline manifest cache 保留该 `Result` failure，`Error -> Ready` 会清除旧分类。`ScreenSpaceUiTextPrepareReport` 现以一帧一次的有界 snapshot 暴露 Ready/Missing/Error、source contract/IO/decode 与 registration 分类，不保留路径或原始错误。越界、绝对路径、TOML parse、missing source、直接读取错误映射和 UI resource recovery 回归已写入。此切片只完成当前 source-boundary contract：asset UUID/artifact generation/cook phase receipt、Editor reimport、fail/last-good/tofu policy 与 cooked artifact-only runtime 仍未实现；状态为 `implementation_in_progress / managed_validation_pending`，没有 Cargo、WGPU 或产品截图结论。
- 2026-08-24 Text01 P1-7A 字体输入 admission budget 已落地在 `asset/assets/font_source/budget.rs` 的唯一策略 owner：Importer 读盘前、统一 decoder 的 WOFF2 声明/实际解压、metadata/TTC extraction 目录枚举以及 `FontDatabase` 直接读盘都复用同一组 source/decoded bytes、face/table、standalone materialization 与 `fvar` cardinality 限额；TTC extraction 在任何 `Vec` 扩容前限制 duplicate table 引发的总 copy bytes。UI/SDF 缓存保留独立 budget failure，prepare report 只做有界聚合。静态回归覆盖 WOFF2 expansion、TTC face、table、`fvar`、duplicate-table materialization 与 UI mapping。cmap/glyph、color-layer、parse time、第三方 corpus/fuzzer、阈值实测校准、cook receipt 和所有 managed Cargo/WGPU 验收仍未完成，因此该项为 `implementation_in_progress / managed_validation_pending`，不得宣称性能或产品资格。
- 2026-08-24 Text01 P1-7B：`parse_sfnt` 已将逐码点 `BTreeSet` 硬切为固定 Unicode-scalar bitmap，只生成既有 ranges/count artifact，并以每 face 65,536 range budget 在 overflow 时发出包含 face index 的 typed error，防止 cmap 交替码点放大 artifact。glyph semantic、color-layer、parse time、第三方 corpus/fuzzer、阈值实测校准、cook receipt 和所有 managed Cargo/WGPU 验收仍未完成；状态保持 `implementation_in_progress / managed_validation_pending`。
- 2026-08-29 Text01 P0-1 current-source 校准与默认复合字体基线：Importer 已把解码后字体写入 versioned `FontBlobArtifact`（schema、source format、BLAKE3 content hash、`Arc<[u8]>`），artifact cache payload 持有 `FontAssetMetadata.cooked_blob`；project loader 只发布 `cooked-font/<asset-uri>` 逻辑 key，`FontDatabase::replace_font_asset_blob` 从 artifact bytes 注册，不重新打开 `FontAsset::source`。Runtime 启动内嵌 default manifest 与 `ZirconDefaultComposite-subset.ttc`，固定注册 2 个 face 且不发现系统字体。对照 Unreal `FStandaloneCompositeFont` / `FCompositeFont` 的 engine-owned default、fallback、sub-font 生命周期，`FontDatabase` 现把 runtime baseline 与 project override 分层：解析优先级为 request explicit > project > runtime；项目 default asset 清除后恢复 `Fira Mono` 与内置 zh-Hans CJK face，不再回落宿主 `fontdb` generic。fallback cache 重建同时重编译 runtime/project 两个复合索引；共享 generation 等价判定包含两层 descriptor/family，但 glyph 热循环没有新增查询、分配或锁。source-deleted restart、hash 与 owner attach 的 focused regressions及本轮 baseline/override 回归已落源码；Rustfmt、scoped diff-check 与静态引用检查通过。clean package、shipping direct-path policy、系统字体隔离、多语言真实 shape/raster、Cargo/WGPU/PNG、profile/RSS/power 仍未执行，因此状态为 `artifact_byte_chain_and_runtime_default_composite_baseline_static_implemented / managed_product_validation_pending`，不关闭 Text01 总计划。
- 同一切片补齐 SDF 最终消费边界：`FontDatabase` 显式保存 runtime default primary face。SDF 解析先使用已挂载的项目 asset owner；默认 owner 不存在时直接使用该内置 face，不再从 `res://fonts/default.font.toml` 重开 manifest/source。自定义字体与 offline cook 路径保持原合同。focused regression 已落源码但未运行，状态并入上述 managed product validation pending。
- 2026-08-29 Text01 clean-process default-face MVP 修正：`TextFontRequest::default()` 的 family 列表为空，而旧 `FontDatabase::match_face` 只搜索显式 family 与 platform/asset fallback；内置 Fira/CJK 不属于该列表，系统字体禁用时会在 CompositeFont itemization 前错误返回 `FontUnavailable`。当前匹配顺序硬切为 explicit family -> project default family -> runtime default primary face -> runtime default family -> platform/asset fallback；默认 primary/family 的任一变化均重建 face-match/fallback cache，防止空 query 保留旧项目 face。没有把私有内置 face 混入公共 fallback 列表，也不改变显式 family 的首选语义。新增 fresh packaged database `A界` 回归要求 neutral glyph handle 可反查到 Fira Mono 与内置 zh-Hans face；匹配算法只在 request-level cache miss 增加至多两个 family lookup 与一次 O(1) face 检查，不进入 glyph 循环。Rustfmt 与静态检查完成，focused test/Cargo/WGPU/PNG/profile/power 尚未执行，状态为 `clean_process_default_face_admission_static_implemented / managed_validation_pending`。
- 2026-08-29 Text01 FontObject/Typeface owner 硬切：`TextStyle.font` 现只表示字体资产 owner，`font_family` 只表示该 owner 内的 typeface/family 选择，不再把 `res://...font.toml` URI 当作 family 查询。`FontAssetOwnerState` 保存有序 faces、owner-local fallback 与 CompositeFont descriptor；generation 发布时建立 `owner -> Arc<CompositeFontIndex>` 派生索引，整形请求只做 owner lookup。owner attach/remove/composite/fallback 变化现在属于 render input 并推进共享 generation，即使物理 face 仍由另一 owner 共享；shaped cache 同时保存 asset 与 family，避免两个 FontObject 的同名 typeface 别名。owner 请求只消费自身 fallback 加基础/platform fallback，不读取其他已加载资产的 fallback 并集。该切片保留 face/source 去重与 bounded cache，不新增逐 glyph URI 查找、I/O、descriptor hash 或锁；旧 generation 在途 lease、session-owned collection、Cargo/WGPU/PNG/profile/power 仍开放。状态为 `font_object_owner_scope_static_implemented / generation_and_cache_identity_corrected / managed_product_validation_pending`，Text01 不关闭。
- 同日 unavailable FontObject 恢复语义修正：显式 `font` owner 未注册或加载失败时，其 owner-local `font_family` 不得退化为全局 family 查询。数据库以 borrowed-or-owned query view 统一约束 shaping、line-metric certificate 与 SDF scalar recovery；registered owner 与无 family 请求零分配，只有 unknown owner + non-empty family 在 request 级克隆并清空小型 family 列表。owner 后续挂载会推进 generation 并恢复 owner-local typeface 语义。全局同名 face 与 metrics 泄漏回归已落源码，动态门仍并入上述 pending 状态。
- Registered owner 的候选来源也已显式化：request typeface 标记为 `OwnerLocalOnly`，只有 CompositeFont、FontAsset fallback 或 base/platform fallback 标记为 `OwnerThenGlobal`。family identity 仍做 O(n) 规范化去重；同名项仅在 authored fallback/composite 明确授权时升级外部搜索。这样 owner 中缺失的 typeface 不会被全局同名 face 冒充，同时正式外部 fallback 仍可命中。
- 二次热路径复核移除了 owner face 的重复物化：注册事务已取得的有序 face 列表现在以 generation-local `Arc<[FontFaceId]>` 保存在 `FontAssetOwnerState`，primary、fallback 与 line metrics 只借用该切片，不再按每个请求/候选 family 遍历 source keys、查询 `asset_source_index` 并先构造临时 face `Vec`。注册/替换阶段仍一次性构造报告和 immutable slice；动态分配与耗时收益等待 profile，不写入已验收结论。
- Runtime last-resort MVP 静态接线：packaged bootstrap 将内嵌 Fira Mono face 同时发布为独立 `runtime_last_resort_face` render input。fallback 全链耗尽时不再借用任意 custom FontObject primary 的 glyph 0，而是切到 engine-owned face；该 identity 进入数据库等价判定并在变更时清理 matching/fallback/metric cache。源码回归要求 unknown scalar 投影为该 face 的真实 handle，SDF 回归要求 packaged glyph 0 具有可生成的轮廓；尚未执行，专用全码点 LastResort 字体仍开放。
- 2026-08-29 publication clone-boundary correction：`FontCollectionService::mutate_published_snapshot` 现在返回精确已发布 `Arc<FontDatabase>` 的 `FontCollectionSnapshot`，claim/admission/retire 等 receipt-only 路径不再在发布后立即复制整库；legacy mutable `TextRenderState` 仍保留 owned `FontDatabase`，因为 native/SDF lazy instance 与其当前可变缓存 API 需要 `&mut`，不会用未经 profile 的 Arc 替换掩盖接口重构。外层 generation clone、owner-registration staging clone、legacy result clone 已各自有固定 profile span/counter；owner staging 的 in-place API 必须等待 31 样本 CPU/allocation/RSS/power 与错误原子性证据，当前状态为 `published_arc_receipt_path_static_implemented / renderer_mutable_owner_profile_gated / managed_validation_pending`。
- render-input 等价不等于整个候选数据库可丢弃：generic mutation 仍可能更新 generation-excluded instance/cache/diagnostic state。因而本轮不以 `has_same_render_inputs` 直接复用旧 Arc；后续必须先把 publication API 收窄为 typed render-input transaction，才能证明 no-op candidate discard 不会孤立已返回的 instance identity。
- 2026-08-29 generation snapshot 与 collection registry 基础设施：共享发布对象由锁内完整 `FontDatabase` 改为 `Arc<FontDatabase>`，`FontCollectionSnapshot` 在同一读锁下取得 exact generation 与 Arc，并允许已开始的 shaping attempt 在新代发布后继续持有旧数据库字节/backend catalog。handle registry/snapshot/metrics 已归入各自 `FontCollectionService`，`TextFontFaceHandle` 同时携带 collection、slot 与 generation；相同代数和 backend ID 的另一集合也不能解析。canonical/artifact projection、renderer SDF bake 和 SDF font-asset cache 都消费显式集合；UI 在途 raster view 同时持有 database 与 resolver snapshot，新代发布后只允许既有租约完成。旧 renderer mutable consumer 的 owned clone 仍由独立 `shared_owned_snapshot_clone` profiler scope 计量。foreign collection、SDF isolation 与 old-generation resolution 源码回归已落，rustfmt/diff/static guard 通过；真实 manager/session owner、backend face slot generational reclaim、Cargo/WGPU/profile/power 仍开放。状态为 `collection_registry_and_inflight_lease_static_implemented / managed_validation_pending`，Text01 不关闭。
- 同日 owner-ready 补强不再把裸 `u64 generation` 当成跨集合身份：新增不可混淆的 `FontCollectionRevision(collection_id, generation)`，`SharedTextLayoutSession`、`UiTextMeasureCache` 与 crate 内 `UiSurface::new_with_font_collection` 共享一个 `Arc<FontCollectionService>`。resolved glyph artifact 还在所有 handle 注册完成后捕获 collection/database 与 resolver publication 两个 Arc lease，后续 raster line acquire 不读取当前进程集合；旧代在途帧和另一同代集合均不会被重新解释。该切片没有建立真实 Core manager，也没有把 screen-space `TextRenderState` 构造切到同一 product owner；`RFF-P1-013/017` 仍开放。静态格式、diff、旧 global probe 与旧 fragment API 扫描通过，Cargo/WGPU/PNG/profile/power 未执行，状态为 `layout_surface_artifact_collection_revision_static_implemented / managed_validation_pending`。
- 2026-08-29 FontObject runtime admission lifetime correction：对照 Unreal `FCompositeFontCache` 的 owner flush 与 `FSlateFontCache::FlushObject`，并参考 Bevy live asset IDs、Fyrox `Resource<Font>` handle 生命周期，`FontCollectionService` 现持有 `HashMap<Arc<str>, usize>` 聚合认领账本。`RuntimeFontAssetClaimScope` 为不可 Clone 的 RAII scope；动态 Runtime UI 在首个 layout 前认领完整依赖，screen-space renderer 在刷新 collection 前按当前依赖做无分配稳定路径 reconciliation。一个 consumer 释放不会删除共享 owner；最后一个 scope 释放时，所有变为 unclaimed 的 asset URI 在一次数据库 mutation/publication 中批量移除，并恢复 packaged runtime default/composite 投影。renderer 本地 ready/missing/error cache 会随释放裁剪，后续重新认领会重新加载，避免已退休 owner 的负缓存污染新 project/session fallback。

该切片只宣称结构复杂度：稳定 renderer 帧为依赖数 `D` 的长度检查加 `HashSet` membership probe，不获取 claim mutex、不 clone `Arc`、不 clone `FontDatabase`、不发布 generation，也不做 glyph 工作；依赖变化路径为 `O(D)` 的 Arc/hash diff，最后释放与所有待接纳 owner 在一次 collection mutation/publication 中完成，随后只刷新一次 collection snapshot。单个 asset registration 的 owner-local staging clone 仍存在，后续需独立 profile 与数据库 API 评估，不能把本切片描述成零 clone。两项 Rust 生命周期回归已写入但未执行；rustfmt、scoped diff-check、source ownership guard 与 Text01 静态测试守卫通过。Cargo/WGPU/PNG、project-switch/hot-reload residency、31-sample allocation/RSS/power 与 Unreal same-load 仍待 managed validation。状态：`font_object_claim_scope_static_implemented / renderer_stable_path_lock_free_by_source / release_plus_admission_single_publication_static_implemented / managed_product_validation_pending`，Text01 总计划保持 `in_progress`。

2026-08-29 loader convergence follow-up：screen-space renderer 的 single-asset load/resolve 与 standalone
admit/retire 包装已删除；测试 helper 只通过正式 batch refresh + 独立 collection claim scope 观察 cache 状态。
SDF raster face cache 同时改为 lookup-only，只读取 `TextRenderState` 已采用的 collection database，不在
shaping 之后再次解析 runtime manifest 或向 renderer 私有 database 注册/删除 face。上游 batch owner 继续负责
source/decode/budget 诊断，offline `.zsdf` artifact cache 保持独立。静态契约 9/9、rustfmt、scoped diff-check
通过；managed build 在 Cargo 启动前返回 `cargo_reuse_pool_busy`，故 Cargo/WGPU/PNG/profile/power 未执行。
状态：`single_font_admission_owner_static_implemented / sdf_face_lookup_only_static_implemented /
managed_product_validation_pending`，Text01 保持 `in_progress`。
