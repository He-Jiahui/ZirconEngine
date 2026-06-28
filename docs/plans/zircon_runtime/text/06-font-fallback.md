---
related_code:
  - zircon_runtime/src/ui/text/font_registry.rs
  - zircon_runtime/src/ui/text/shaper.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/asset/assets/font.rs
design_references:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts/CompositeFont.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/FontCacheCompositeFont.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/FontCacheCompositeFont.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts/UnicodeBlockRange.h
  - dev/godot/scene/resources/font.h
  - dev/godot/modules/text_server_adv/text_server_adv.cpp
  - dev/Fyrox/fyrox-ui/src/font/mod.rs
plan_sources:
  - docs/plans/zircon_runtime/text/index.md
  - docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
  - docs/plans/zircon_runtime/text/02-shaping-unicode-and-bidi.md
status: planned
---

# 06 字体回退规则 / 文本字体回退

> 本计划定义"当首选 face 无某码点字形时,如何选回退 face"的**算法**(数据结构在 `01`)。承接 `editor_ui/03 §2.2` 缺口 3,根治当前 `font_registry.rs` 硬编码链与 CJK/emoji 缺字显示豆腐块(tofu)。

## 1. 目标

1. **脚本感知回退**:按 cluster 的 script(`02` 已分段)选最优回退 family,而非固定线性链。
2. **Unicode 范围回退**:`CompositeFont` 的 sub-font 按 Unicode block range 命中(对齐 UE `FCompositeFont` + `UnicodeBlockRange`)。
3. **链式回退 + 深度限制**:默认链 → CompositeFont sub-font → 系统字体发现(fontdb 按码点查)→ last-resort tofu;深度上限防循环(Fyrox `MAX_FALLBACK_DEPTH=10`)。
4. **回退一致性**:同一 cluster 的所有字形落同一 face(避免半簇换字);命中 face 写入 `ShapedGlyph.font_id` 供图集/SDF 正确取 face。
5. **缺字处理**:无任何 face 命中 → `.notdef`(tofu)或可配占位;记录缺字诊断。

## 2. 现状与差距

- `font_registry.rs`:线性硬编码链 `[Inter, Noto Sans, Noto Sans CJK SC, Microsoft YaHei UI, Segoe UI]`,非脚本感知、不查系统字体、emoji 无回退。
- glyphon 内部 fontdb 有自己的回退,但与 ZirconEngine `font_registry` 双轨、不一致。
- 缺口:无 script→family 映射、无 Unicode range 命中、无深度限制、无 cluster 级一致性保证、无缺字诊断。

## 3. 参考代码

| 文件 | 应重点阅读 |
|------|-----------|
| `dev/UnrealEngine/.../Fonts/CompositeFont.h` | `FCompositeFont { DefaultTypeface, SubTypefaces[FCompositeSubFont{ Typeface, CharRanges, Cultures, ScriptName }] }`——**脚本/范围/文化三维回退主样板** |
| `dev/UnrealEngine/.../Fonts/FontCacheCompositeFont.cpp/.h` | `FCompositeFontCache::GetFontData`/`GetDefaultFontData`:按 codepoint 解析 sub-font、`GetCompositeFontDataForCodepoint` 的命中顺序与缓存 |
| `dev/UnrealEngine/.../Fonts/UnicodeBlockRange.h` + `.inl` | Unicode block range 全表(`EUnicodeBlockRange`)——本计划 range 命中表直接对照 |
| `dev/godot/scene/resources/font.h` | `Font::fallbacks` 链 + `find_variation`;`TextServerAdvanced` 按 script/codepoint 自动选 fallback |
| `dev/godot/modules/text_server_adv/text_server_adv.cpp` | `_font_get_glyph_index` 失败 → 遍历 fallback;系统字体按 `font_get_supported_chars` 命中 |
| `dev/Fyrox/fyrox-ui/src/font/mod.rs` | `MAX_FALLBACK_DEPTH=10` 深度限;`glyph_index` 失败遍历 `fallbacks` 链的 Rust 实现 |

**Rust/wgpu 落地**:fontdb `Database::query` + 按码点过滤(cosmic-text `FontSystem` 已内置 script-aware fallback via fontdb,可优先复用其结果再补 CompositeFont 覆盖)。cosmic-text shaping 时会自动对缺字 run 换 face——本计划主要是**配置回退源 + 统一诊断 + 保证 font_id 正确传出**。

## 4. 目标架构

```
cluster(script from 02, codepoints) →
  1. 首选 face(用户 style.font)有字形? → 用之
  2. CompositeFont sub-font(script/range 命中,01 composite_resolve) →
  3. fontdb 系统发现(按码点 supported) →
  4. last-resort(.notdef / 占位)+ 缺字诊断
  → 命中 face 写 ShapedGlyph.font_id;cluster 内所有字形同 face
```

回退发生在 `02` 整形阶段(per-run);cosmic-text 内部已做大部分,本计划提供回退源配置 + 覆盖 + 诊断。

## 5. 里程碑

### FB-M1 脚本/范围感知回退链

实施切片:
1. `graphics/text/font/fallback.rs`:回退解析器——首选→CompositeFont(script/range,查 `01` `composite_resolve`)→fontdb(码点)→last-resort;深度上限 10。
2. `font_registry.rs` 硬编码链改为数据驱动 `CompositeFontDescriptor`(默认包含 latin/CJK/emoji/阿拉伯等 sub-font)。
3. cosmic-text fallback 与本链对齐:配置 fontdb 回退源 = `FontDatabase`(`01`),消除双轨。

测试:`text_fallback_cjk_resolves_to_cjk_font`、`text_fallback_emoji_resolves_to_color_font`、`text_fallback_depth_limited`。

### FB-M2 cluster 一致性 + font_id 传出 + 诊断

实施切片:
1. 保证同 cluster 字形同 face(整形 run 切分尊重回退边界);`ShapedGlyph.font_id` = 实际命中 face(`02` 已留字段)。
2. 缺字诊断:记录未命中码点(script/codepoint/上下文),供编辑器字体缺失提示;tofu 渲染 `.notdef` 或可配占位字形。

测试:`text_fallback_cluster_stays_single_face`、`text_fallback_glyph_carries_resolved_font_id`、`text_fallback_missing_codepoint_reports_diagnostic`。

## 6. 工程落地细化(实施权威)

### 模块与文件落点

实现层 `zircon_runtime/src/graphics/text/font/fallback.rs`(承接 `01` 的 `composite_resolve.rs`):

```rust
pub struct FallbackResolver<'a> { db: &'a FontDatabase, composite: &'a CompositeFontDescriptor,
    max_depth: u8 /*=10*/ }
impl FallbackResolver<'_> {
    /// 为一个 cluster(同 script,codepoints)解析命中 face
    pub fn resolve(&self, primary: FontFaceId, script: Script, cps: &[char])
        -> FallbackResolution; // { face: FontFaceId, missing: bool }
    pub fn diagnostics(&self) -> &MissingGlyphLog;
}
```

解析顺序(对齐 UE `GetCompositeFontDataForCodepoint`):
1. `primary` face cmap 命中所有 cps → 用 primary。
2. CompositeFont sub-font:按 script 命中(优先)或 Unicode range 命中(`UnicodeBlockRange` 对照),取首个覆盖全 cps 的 family → `db.match_face`。
3. `db.fallback_candidates(cp, query)`:fontdb 按码点枚举系统/项目 face,取首个覆盖。
4. 都不命中 → last-resort face(`.notdef`),`missing=true`,记诊断。

深度限制:CompositeFont sub-font 自身可声明 fallback,递归深度上限 `max_depth=10`(Fyrox 对照),超限停在 last-resort。

### 默认 CompositeFont 包(编辑器/运行时默认)

| sub-font | script/range | family(默认) |
|----------|-------------|-------------|
| default | Latin/Common | Inter / Segoe UI |
| CJK | Han/Hiragana/Katakana/Hangul | Noto Sans CJK SC / Microsoft YaHei UI / 思源黑体 |
| Arabic | Arabic | Noto Sans Arabic |
| Hebrew | Hebrew | Noto Sans Hebrew |
| Emoji | Emoji presentation | Noto Color Emoji / Segoe UI Emoji |
| Symbols | Misc symbols | Noto Sans Symbols |

声明为 `CompositeFontDescriptor` 资产(`01` FR-M3),非硬编码常量。

### 与既有路径硬切换

| 现有 | 切换 |
|------|------|
| `font_registry.rs` 硬编码 5 字体链 | 删除常量;改默认 `CompositeFontDescriptor` 资产 + `FallbackResolver` |
| glyphon 内部 fontdb 独立回退 | 配置 glyphon `FontSystem` 用共享 `FontDatabase`(01),回退源一致 |

### 测试与验收清单

| 测试 | 断言 |
|------|------|
| `text_fallback_cjk_resolves_to_cjk_font` | "中文" 用 latin 首选时回退到 CJK face,font_id 正确 |
| `text_fallback_emoji_resolves_to_color_font` | emoji 回退到 color emoji face,落 RGBA 页 |
| `text_fallback_arabic_resolves_and_shapes_rtl` | 阿拉伯回退 face + RTL 连写正确(联动 02) |
| `text_fallback_cluster_stays_single_face` | 组合簇所有字形同 face,无半簇换字 |
| `text_fallback_glyph_carries_resolved_font_id` | `ShapedGlyph.font_id` = 实际命中 face,非首选 |
| `text_fallback_depth_limited` | 构造循环 fallback,深度 10 停,无栈溢出 |
| `text_fallback_missing_codepoint_reports_diagnostic` | 无 face 覆盖 → tofu + 诊断记录 script/codepoint |

里程碑命令:`cargo test -p zircon_runtime text_fallback --locked`。

## 7. 风险与回退

- cosmic-text 已自带 fallback,本计划重在**配置回退源一致 + 覆盖 CompositeFont + 诊断**,避免重造其内部逻辑;若其 font_id 传出不足,在隔离层补映射。
- last-resort 字体打包:编辑器默认包需含 CJK/emoji(体积),运行时项目可裁剪。

## 8. 状态与产出记录

| 日期 | 里程碑/切片 | 状态 | 产出 | 验证 | 后续 |
|------|-------------|------|------|------|------|
| 2026-06-28 | FB-M1 首段:cmap-aware fallback candidate filter | runtime_text_fr_m2_fb_m1_cmap_candidate_filter_rustfmt_metadata_passed_focused_test_timeout | `graphics/text/font/coverage.rs` 作为回退候选 coverage leaf,为可解析 sfnt project faces 保存 compact cmap ranges;`FontDatabase::fallback_candidates` 在 CompositeFont/request/default/fallback family 排序后按 codepoint 剔除 Known coverage 不覆盖的 face,Unknown coverage 对系统字体和 synthetic tests 维持 permissive,避免误删不可判定候选;新增 focused database 测试锁定 Latin known face 不应覆盖 CJK codepoint、Unknown face 保留的预筛行为 | scoped `rustfmt --check` 通过;scoped `git diff --check` 仅 CRLF 提示;`cargo metadata --locked --format-version 1 --no-default-features` 通过;截图证据仍在 `docs/tests/runtime/text/runtime_text_shared_metrics_preview_20260628.png`,未写 target;focused `text_font_fallback_candidates_filter_known_cmap_coverage` lib-test 编译超时无 Rust diagnostics,本次独立 target-dir 验证进程已停止,不计 Cargo/test 通过 | 后续实现完整 `FallbackResolver`、cluster 级一致性、`ShapedGlyph.font_id` 实际命中 face、缺字诊断、深度限制、emoji/color font 路由 |
| 2026-06-27 | 计划建立 | planned | 脚本/范围感知回退 + cluster 一致性 + font_id 传出 + 缺字诊断路线 | 文档 | FB-M1 数据驱动回退链;依赖 01 CompositeFont、02 script 分段 |
