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

- 旧基线 `asset/assets/font.rs`:`FontAsset` 仅 `{ source, family, render_mode }`,单 face,无 weight/style/变量轴/回退声明；FR-M2 首段已扩展 schema 并保留 parsed metadata,但 `FontDatabase` 仍需消费这些 metadata 做完整 best-match。
- 旧基线 `importer/ingest/import_font_asset.rs`:仅解析 TOML,无字体文件头解析；当前已硬切到 `import_font_asset/{mod.rs,parse_sfnt.rs}`,可解析 sfnt/TTC metadata、`fvar` 与 cmap,但 WOFF2 decode 和真实 TTC/变量 fixture 仍未关闭。
- `ui/text/font_registry.rs`:回退链**硬编码** `["Inter","Noto Sans","Noto Sans CJK SC","Microsoft YaHei UI","Segoe UI"]`,非数据驱动、无系统字体发现。
- `graphics/.../ui/font_asset.rs`:`res://`/本地路径加载可用,但产出直接喂 glyphon `FontSystem`,无中立 face 句柄。
- **无 `FontDatabase`**:无统一索引,glyphon 内部 `fontdb` 与 SDF bake 的字体对象各自持有,字体二进制重复加载。

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

**资产/导入(`zircon_runtime/src/asset/`)**:`assets/font.rs` 升级 `FontAsset` schema;`importer/ingest/import_font_asset/{mod.rs,parse_sfnt.rs}` 已承接 FR-M2 首段 owner 切分,后续继续补 `woff2` decode 与更细的 variable/cmap leaf(遵 `runtime/15`)。

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
    pub fn instance(&self, face: FontFaceId, vars: &VariationCoords) -> InstancedFaceId;
    pub fn face_bytes(&self, face: FontFaceId) -> Arc<FaceBytes>; // 零拷贝;glyphon/SDF/MSDF 共享
    pub fn unregister_asset(&mut self, asset: &FontAssetSourceKey) -> Vec<FontFaceId>; // 资产卸载/替换;返回被失效的 face(2026-07-02 评审收口)
    pub fn invalidate_face(&mut self, face: FontFaceId);          // 单 face 失效,触发下述失效级联(2026-07-02 评审收口)
}
```

best-match 权重距离用 CSS Fonts L4 算法(weight 优先就近、style Italic>Oblique>Normal、stretch 就近),对照 fontdb `Database::query`。

### 失效级联(2026-07-02 评审收口)

face 失效(`unregister_asset`/`invalidate_face`,来源:资产热重载、字体包卸载、变量实例回收)必须按以下固定顺序级联,任何一级不得跳过:

1. `ShapedRunCache` 按 `font_id`(`FontFaceId`/`InstancedFaceId`)剔除全部命中条目;
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

| 日期 | 里程碑/切片 | 状态 | 产出 | 验证 | 后续 |
|------|-------------|------|------|------|------|
| 2026-07-03 | FR-M1/FR-M2 FontDatabase tests owner split | runtime_text_fr_m1_fr_m2_font_database_tests_owner_split_rustfmt_visual_cargo_deferred | `graphics/text/font/database.rs` 继续只做 FontDatabase production owner、family/source/asset indexes、系统字体注册编排、match/fallback handoff 与 face bytes；既有 10 个私有回归测试机械迁移到 `graphics/text/font/database/tests.rs`，父文件仅保留 `#[cfg(test)] mod tests;`；未改 font matching、registration、fallback resolver、descriptor parsing、glyphon/SDF/render path 行为，也未新增 root facade、compat shim 或旧路径 re-export | scoped `rustfmt --edition 2021 --check zircon_runtime/src/graphics/text/font/database.rs zircon_runtime/src/graphics/text/font/database/tests.rs` 通过；验证图 `docs/tests/runtime/text/runtime_text_font_database_tests_owner_split_preview_20260703.png` 已检查，SHA256 `41A1884B987C0E8E1D37E0E61C5F7B66C861273A94CDBC239FCCCDF9D7429E9D`；验证日志 `docs/tests/runtime/text/runtime_text_font_database_tests_owner_split_validation_20260703.log` SHA256 `6E2CDFDFF5F28D00A0AAC664B1D15AED0B8F544281C64D484B0507DF07A0E770`；repo `target`、`E:\cargo-targets` 与 `D:\cargo-targets` 同名扫描为 0；外部 cargo/rustc lanes 活跃，本切片不启动 focused Cargo，不声明 Cargo green | FR-M1/FR-M2 结构债继续收敛；WOFF2 decode、变量字体 fixture、TTC 绿跑、SDF 非 0 face 真 raster、CompositeFont 资产接线与 plan 06 fallback resolver 仍继续 |
| 2026-06-28 | FR-M2 续段:FontAsset UI registry schema convergence | runtime_text_fr_m2_font_asset_ui_registry_convergence_core_check_passed_focused_test_timeout | `FontAssetRenderStrategy::effective_render_mode(...)` 和 `FontAsset::effective_render_mode()` 集中旧 `render_mode` 优先、`render_strategy.default_mode` 补默认值、`allow_native`/`allow_sdf` clamp 的资产语义;`graphics/scene/scene_renderer/ui/font_asset.rs` 改调用资产 helper,不再复制策略分支;`ui/text/font_registry.rs` 注册资产时使用 effective render mode,并把注册 family 与 `fallback_families` 合并进 UI fallback chain,过滤空白并按大小写归一去重;`ui/tests/text_pipeline.rs` 的旧 `FontAsset` literal 补齐完整 schema,新增 strategy default-mode 与 fallback-chain dedupe 覆盖 | scoped `rustfmt --check` 通过;`cargo metadata --locked --format-version 1 --no-default-features` 通过;`cargo check -q -p zircon_runtime --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0628-check` 通过(仅既有 warning);截图证据仍在 `docs/tests/runtime/text/runtime_text_shared_metrics_preview_20260628.png`,确认 `target/runtime_text_shared_metrics_preview_20260628.png` 不存在;focused `text_font_registry_uses_asset_render_strategy_default_mode` 首次 warm target-dir 失败于 Cargo fingerprint path 写入,独立 target-dir 重跑 604s 编译超时无 Rust diagnostics,已停止匹配验证进程,不计 Cargo/test 通过 | UI registry 资产 schema 消费首段关闭;WOFF2 decode、变量字体 fixture、TTC 测试绿跑、SDF 非 0 face 真 raster、CompositeFont 资产接线与 plan 06 fallback resolver 仍继续 |
| 2026-06-28 | FR-M2 续段:FontAsset family/fallback 数据面注册 | runtime_text_fr_m2_font_asset_registration_rustfmt_metadata_passed_focused_test_timeout | `FontDatabase::register_font_asset` 按完整 `FontAsset` 注册项目字体:一次读取 source bytes,用 `family_members` 覆盖 family/face_index/weight/style/width_class/variation coords,并把 `fallback_families` 合并进 database fallback chain;`graphics/text/font/asset_registration.rs` 承接 family-member → logical face descriptor 投影与 asset source key,重复注册按 path/face/family/style/weight/stretch/variation 去重而不是只按 path+face 压扁;native glyphon 与 SDF bake 的 `LoadedUiFontManifest` 现在携带完整 `FontAsset`,`.font.toml` 路径优先走资产注册入口,直接字体文件路径仍走 `register_font_file`;测试用 patched weight/TTC fixture helper 拆入 `graphics/text/font/test_font_fixtures.rs`,保持 database owner 低于结构预算 | scoped `rustfmt --check` 通过;scoped `git diff --check` 仅 CRLF 提示;`cargo metadata --locked --format-version 1 --no-default-features` 通过;截图证据仍在 `docs/tests/runtime/text/runtime_text_shared_metrics_preview_20260628.png`,确认 `target/runtime_text_shared_metrics_preview_20260628.png` 不存在;focused `text_font_database_registers_font_asset_family_members_and_fallbacks` logical-face lib-test 编译超时且无 Rust diagnostics,本次独立 target-dir 验证进程已停止,不计 Cargo/test 通过 | family/fallback 数据面首段关闭;WOFF2 decode、变量字体 fixture、TTC 测试绿跑、SDF 非 0 face 真 raster、CompositeFont 资产接线与 plan 06 cmap/script/tofu 精筛仍继续 |
| 2026-06-28 | FR-M2 续段:render strategy 默认模式消费 + SDF 测试清理 | runtime_text_fr_m2_render_strategy_default_mode_rustfmt_metadata_passed_focused_test_timeout | `graphics/scene/scene_renderer/ui/font_asset.rs` 在 `.font.toml` manifest 加载边界把 `render_strategy.default_mode` 解析为 UI font 默认渲染模式,并保持旧 `render_mode` 优先;`allow_native`/`allow_sdf` 只在默认模式解析中做最小约束,避免把 asset schema 细节扩散到 renderer routing;新增 focused 单测覆盖 strategy 默认、旧字段优先和 disallowed Auto clamp;`sdf_font_bake.rs` 的 face-index fallback 测试改用 Drop 清理 manifest/source 双路径,修复临时字体源文件可能遗留的问题 | scoped `rustfmt --check` 通过;scoped `git diff --check` 仅 CRLF 提示;`cargo metadata --locked --format-version 1 --no-default-features` 通过;截图证据仍在 `docs/tests/runtime/text/runtime_text_shared_metrics_preview_20260628.png`,未写 target;focused `render_strategy_default_mode_feeds_ui_font_default` lib-test 编译超时且无 Rust diagnostics,本次独立验证进程已停止,不计 Cargo/test 通过 | `render_strategy.default_mode` 消费首段关闭;`family_members`/`fallback_families` 尚未进入完整 `FontDatabase::register_asset` 数据面;WOFF2 decode、变量字体 fixture、TTC 测试绿跑、SDF 非 0 face 真 raster 与 plan 06 cmap/script 精筛仍继续 |
| 2026-06-28 | FR-M2 首段:sfnt/TTC 元数据 schema + 导入器解析 + FontDatabase metadata ingestion | runtime_text_fr_m2_sfnt_metadata_schema_rustfmt_metadata_passed_cargo_check_timeout | `FontAsset` schema 从单 source/family/render_mode 扩展到 `face_index`、family members、variable instances、fallback families、render strategy 与 parsed metadata;字体导入器从单文件硬切到 `import_font_asset/{mod.rs,parse_sfnt.rs}` owner,解析 TTF/OTF sfnt 与 TTC face count、name 表、OS/2 weight/width/style、`fvar` axes + best-effort named instances、cmap 覆盖 range,并把解析结果回填 imported `FontAsset`;runtime UI font manifest 把 `face_index` 传入 `FontDatabase`,native/SDF 注册键包含 face index;`FontDatabase::register_font_file` 现在读取 selected face 的 family/weight/style/stretch 元数据,项目字体 best-match 不再一律当 Regular;测试内组装 TTC fixture 覆盖多 face metadata 与 database face-index 去重;SDF bake 检测到 `fontsdf` 不支持非 0 face 时跳过该 face并回退默认字体,不再静默用错 face 0 | scoped `rustfmt --check` 通过;scoped `git diff --check` 通过;`cargo metadata --locked --format-version 1 --no-default-features` 通过;截图证据仍在 `docs/tests/runtime/text/runtime_text_shared_metrics_preview_20260628.png`,未写 target;focused `text_font_database_reads_file_weight_for_best_match`、`sdf_font_bake_falls_back_when_fontsdf_cannot_open_requested_face_index`、`text_font_` lib-test 本轮编译超时且无 Rust diagnostics,匹配 runtime 验证进程已停止,不计 Cargo/test 通过 | WOFF2 只显式报 unsupported,尚未接 decode;缺变量字体 fixture 与 `text_font_variable_axes_roundtrip` 绿跑;TTC fixture/tests 已加入但未绿跑;FontDatabase 尚未消费 imported family_members/fallback_families/render_strategy;SDF 真实非 0 face raster 仍需替换/扩展 `fontsdf` path |
| 2026-06-28 | FR-M1 第三段:系统字体发现 + CompositeFont 候选数据面 | runtime_text_fr_m1_system_fonts_composite_candidates_core_check_passed_focused_test_timeout | `FontDatabase::load_system_fonts()` 通过 glyphon/fontdb 枚举系统 fonts,把系统 face 编入 Zircon `FontFaceId`/family 查询表,并保留 system `fontdb::Source` 供 glyphon 消费;screen-space UI text system 初始化时加载系统字体索引;新增 `fallback_candidates(codepoint, FontQuery, CompositeFontDescriptor)` 数据面,按 sub-font script/range → composite default → request families → runtime fallback chain 产出有序 `FontFaceId` 候选 | scoped `rustfmt --check` 通过;静态扫描确认 native/SDF render paths 无直接 `load_font_file`/独立 fontsdf path cache 回流;`cargo check -q -p zircon_runtime --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0628-check` 通过(仅既有 warning);截图证据仍在 `docs/tests/runtime/text/runtime_text_shared_metrics_preview_20260628.png`,未写 target | FR-M1 尚余完整系统字体策略开关与 CompositeFont 资产接线;FR-M2 仍需 TTC/WOFF2/变量轴/cmap 解析,06 负责 cmap/script 精筛与 tofu 深度限制 |
| 2026-06-28 | FR-M1 第二段:FontDatabase 接入 glyphon/SDF 渲染路径 | runtime_text_fr_m1_shared_font_database_native_sdf_core_check_passed_focused_test_timeout | `graphics/text/font/database.rs` 从内存 owner 推进到可注册真实字体文件、按 source path 去重、共享 `Arc<[u8]>` face bytes,并能把已注册 face 喂给 glyphon `FontSystem`;`graphics/scene/scene_renderer/ui/text.rs` 持有同一 `FontDatabase` 并让 NativeGlyphon 字体资产加载走数据库;`sdf_font_bake.rs`/`sdf_render.rs` 的 fontsdf 字体对象改由 `FontFaceId -> Arc<[u8]>` 派生,不再绕过共享 owner 各自读盘 | scoped `rustfmt` 通过;`cargo check -q -p zircon_runtime --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0628-check` 通过(仅既有 warning);先前 focused `text_font` lib-test 编译超时无 Rust diagnostics,匹配验证进程已停止 | FR-M1 仍需系统字体发现、CompositeFont 回退候选枚举与完整 fontdb 索引策略;FR-M2 仍需 TTC/WOFF2/变量轴/cmap 解析 |
| 2026-06-28 | FR-M1 首段:中立 font contract + in-memory FontDatabase owner | runtime_text_fr_m1_font_contract_database_core_check_passed_focused_test_timeout | 新增 `core/framework/render/text/font/{face,family,database,composite}.rs` 中立 DTO 与 `graphics/text/font/{database,default_families}.rs` crate-private owner;提供 family/weight/style/stretch 查询、共享 `Arc<[u8]>` face bytes、variation instance id;`ui/text/font_registry.rs` 默认 fallback chain 改由 runtime font database 提供,移除 UI-local 硬编码默认链 | `rustfmt` 通过;静态扫描确认 `ui/text/font_registry.rs` 无默认字体字面量;`cargo check -p zircon_runtime --lib --no-default-features --locked --jobs 1` 通过(仅既有 warning);`text_font` focused Cargo test 编译超时无 diagnostics,匹配验证进程已停止 | FR-M1 后续仍需真实 `fontdb`/系统字体发现、项目字体资产注册到数据库、glyphon/SDF 共用同一数据库;FR-M2 仍需 TTC/WOFF2/变量轴/cmap 解析 |
| 2026-06-27 | 计划建立 | planned | 定义 FontFace/CompositeFont/FontDatabase 分层与导入 schema 升级 | 文档 | FR-M1 中立 face + 共享 fontdb;阻塞 02/06 |
