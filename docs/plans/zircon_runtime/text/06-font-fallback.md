---
related_code:
  - zircon_runtime_interface/src/ui/surface/render/resolved_style.rs
  - zircon_runtime/src/text/model/shaped_run.rs
  - zircon_runtime/src/ui/surface/render/resolve.rs
  - zircon_runtime/src/ui/text/resolved_layout.rs
  - zircon_runtime/src/ui/text/font_registry.rs
  - zircon_runtime/src/ui/text/shaper.rs
  - zircon_runtime/src/text/font/mod.rs
  - zircon_runtime/src/text/font/backend.rs
  - zircon_runtime/src/text/font/database.rs
  - zircon_runtime/src/text/font/shared.rs
  - zircon_runtime/src/text/font/fallback.rs
  - zircon_runtime/src/text/font/fallback/tests.rs
  - zircon_runtime/src/text/font/coverage.rs
  - zircon_runtime/src/text/shaping/cosmic.rs
  - zircon_runtime/src/text/shaping/cosmic/font_system_cache.rs
  - zircon_runtime/src/text/shaping/fallback_spans.rs
  - zircon_runtime/src/text/shaping/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/prepare_report.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas.rs
  - zircon_runtime/src/text/sdf/font_bake.rs
  - zircon_runtime/tests/runtime_text_multilingual_product_framebuffer.rs
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
status: in_progress
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

- `font_registry.rs` 的默认 family 已改为消费 runtime font database；`FallbackResolver` 已承接 script/range/locale 候选与深度上限。
- 2026-07-10 D4 硬切后，`FontDatabase` 持有权威 `fontdb::Database` lineage 与双向 ID map；shared locale shaping cache 和 native renderer 通过 generation snapshot 消费同一 lineage，`ShapedGlyph.font_id`/native report 均来自实际 `LayoutGlyph.font_id`。旧 `shaping/font_id.rs` post-shape 重算桥已删除且无 shim。
- 2026-07-10 locale 数据面已贯通：可序列化的 `UiResolvedStyle.language` 从模板 `[font].language` 进入 layout/shaped cache key、direct/parallel `TextShapeRequest`、native rich spans 与 SDF atlas/bake fallback；`zh-Hans`/`ja` 等同码点不会跨 locale 复用缓存或 SDF 槽。
- 2026-07-10 真实 WGPU 产品 framebuffer 已覆盖 Latin/CJK/Arabic/Hebrew/emoji/mixed BiDi/native/SDF、zh-Hans/ja 同码点与 VerticalRl SDF 十项；逐项 background delta、地区字体相对像素差与人工原图检查通过，证据只写入 `docs/tests/runtime/text`。
- 后端直证已锁定实际选中的 `Segoe UI Emoji` face 产生 `SwashContent::Color` 且字节数严格等于 `width * height * 4` RGBA；同一窄层可执行验收还证明 `نَ` 的 base+fatha 形成两个实际 glyph 且都保留同一 `Segoe UI` backend face。生产映射继续由 `GlyphAtlasFormat::Color` / `Rgba8Unorm` 合同 owner 持有；per-run OpenType `locl` 已由 Text 02 horizontal RustyBuzz leaf 实现并等待真实语言 exact，竖排像素等由对应后续计划继续承接。

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

(2026-07-02 评审收口,D4；2026-07-10 已硬切；2026-07-13 实例拆分)**font_id 权威通路**:`ShapedGlyph.font_id` = backend 实际选择的 base `FontFaceId`，`font_instance_id` 独立承载该 face 的有效 `InstancedFaceId`；二者不得相互替代。当前 `text/font/backend.rs` 隔离 fontdb ID↔`FontFaceId`，`cosmic.rs` 直接投影 `LayoutGlyph.font_id` 并派生有效 instance，native report 直接遍历实际 `Buffer.layout_runs()`；禁止 post-shape 按 script/codepoint 重算，旧 bridge 已物理删除。

## 5. 里程碑

### FB-M1 脚本/范围感知回退链

实施切片:
1. `text/font/fallback.rs`:回退解析器——首选→CompositeFont(script/range,查 `01` `composite_resolve`)→fontdb(码点)→last-resort;深度上限 10。
2. `font_registry.rs` 硬编码链改为数据驱动 `CompositeFontDescriptor`(默认包含 latin/CJK/emoji/阿拉伯等 sub-font)。
3. ✅ cosmic-text fallback 与本链对齐：shared locale cache 与 native renderer 消费 process-shared `FontDatabase` generation snapshot，消除 backend database 双轨。
4. ✅ (2026-07-02 评审收口,D4；2026-07-10 完成)建立 fontdb ID↔`FontFaceId` 双向映射，`ShapedGlyph.font_id` 从整形后端实际选择的 face 直出；删除 `shaping/font_id.rs` post-shape annotation 过渡路径。
5. ✅ (2026-07-10)run language/locale 从公共样式进入 layout/shaped/SDF 三类缓存键以及 native/SDF fallback 查询；空 tag 归一为无标注，缓存键对 tag 大小写归一。

测试:`text_fallback_cjk_resolves_to_cjk_font`、`text_fallback_emoji_resolves_to_color_font`、`text_fallback_depth_limited`。

### FB-M2 cluster 一致性 + font_id 传出 + 诊断

实施切片:
1. 已完成：grapheme cluster 在 shape 前解析为一个 CompositeFont family span，`ShapedGlyph.font_id` 来自实际 backend；`text_fallback_arabic_mark_cluster_stays_on_one_actual_backend_face` 以阿拉伯 base+fatha 的多 glyph cluster 断言 span 内所有 glyph 保持同一 backend face。受管 Cargo/WGPU 验证仍延后。
2. ✅ 缺字诊断：按 `(face,codepoint)` 去重并记录 script/reason/occurrence；1024 容量、overflow/dropped 已落，`FontDatabase` poison-recovering store 在 prepare 后排到 frame report，不在 shaping 热路径 IO/格式化。

测试:`text_fallback_cluster_stays_single_face`、`text_fallback_glyph_carries_resolved_font_id`、`text_fallback_missing_codepoint_reports_diagnostic`。

## 6. 工程落地细化(实施权威)

### 模块与文件落点

实现层 `zircon_runtime/src/text/font/fallback.rs`(承接 `01` 的 `composite_resolve.rs`):

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

(2026-07-02 评审收口；2026-07-10 已实现)`MissingGlyphLog` 契约:按 **(face, codepoint) 去重**(同一缺字每 face 只记一条,重复命中只累加计数);**容量上限**(默认 1024 条,超限丢弃新条目并置 overflow 标志,防恶意/超长文本撑爆内存);导出走**帧外** `ScreenSpaceUiTextPrepareReport.missing_glyphs`，不在整形热路径上做 IO 或格式化。

解析顺序(对齐 UE `GetCompositeFontDataForCodepoint`):
1. `primary` face cmap 命中所有 cps → 用 primary。
2. CompositeFont sub-font:按 script 命中(优先)或 Unicode range 命中(`UnicodeBlockRange` 对照),取首个覆盖全 cps 的 family → `db.match_face`。
3. `db.fallback_candidates(cp, query)`:fontdb 按码点枚举系统/项目 face,取首个覆盖。
4. 都不命中 → last-resort face(`.notdef`),`missing=true`,记诊断。

(2026-07-02 评审收口；2026-07-10 已实现 resolver/span 侧规则)**partial cluster coverage 规则**:无任何候选 face 覆盖 cluster 全部码点时(常见:base 字符有覆盖但个别 combining mark 无),不再继续深链搜索"全覆盖" face——用 canonical combining class 找基字，选**覆盖基字且 cluster 覆盖数最多**的 face,未覆盖 mark 渲 `.notdef` 并按 (face, codepoint) 记入缺字诊断。真实 backend cluster 单 face 仍由端到端断言验收。

(2026-07-02 评审收口)解析入口的 `FontQuery` 必须带请求的 weight/style 进 `db.match_face`(回退 family 同样按 weight/style 匹配变体);family 命中但无对应 weight/style 变体时,取最近变体并置 `SyntheticFlags`(bold=embolden / oblique=shear,进 04 `GlyphRasterKey`)。

(2026-07-02 评审收口)`ShapedGlyph.font_id` 的传出通路见 §4 D4 条款:必须来自后端实际选择的 face,禁止在本解析器结果之外 post-shape 重算。

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

(2026-07-02 评审收口)**locale 维度**:CJK sub-font 需按 locale 分行——Han 统一表意字在 SC/TC/JP/KR 下字形规范不同(UE `FCompositeSubFont.Cultures` 同款维度):

| sub-font | script × locale | family(默认) |
|----------|----------------|-------------|
| CJK-SC | Han + `zh-Hans` | Noto Sans CJK SC / Microsoft YaHei UI |
| CJK-TC | Han + `zh-Hant` | Noto Sans CJK TC / Microsoft JhengHei UI |
| CJK-JP | Han/Kana + `ja` | Noto Sans CJK JP / Yu Gothic UI |
| CJK-KR | Han/Hangul + `ko` | Noto Sans CJK KR / Malgun Gothic |

locale 取 run 的 `UiResolvedStyle.language` 标注(02 shaped key 含 language)，模板入口支持 `[font].language`/`text_language`/`language`；无标注时回退项目默认 locale。命中回退 face 后的混排行度量按 03 §6"混 face 行度量"(D7:行 ascent/descent 取 max、line_gap 取主 face)。

声明为 `CompositeFontDescriptor` 资产(`01` FR-M3),非硬编码常量。

### 与既有路径硬切换

| 现有 | 切换 |
|------|------|
| `font_registry.rs` 硬编码 5 字体链 | 删除常量;改默认 `CompositeFontDescriptor` 资产 + `FallbackResolver` |
| glyphon 内部 fontdb 独立回退 | 已硬切：`FontDatabase` 持有 backend DB；renderer 与 locale shaping cache 通过 snapshot/generation 共用同一 ID lineage |

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
| `text_fallback_partial_cluster_coverage_keeps_base_face` | (2026-07-02 评审收口)base 有覆盖、mark 部分缺失时选 mark 覆盖数最多的 face,缺失 mark 渲 .notdef 并记诊断,cluster 单 face 不破 |

里程碑命令:`cargo test -p zircon_runtime text_fallback --locked`。

## 7. 风险与回退

- cosmic-text 已自带 fallback,本计划重在**配置回退源一致 + 覆盖 CompositeFont + 诊断**,避免重造其内部逻辑;若其 font_id 传出不足,在隔离层补映射。
- last-resort 字体打包:编辑器默认包需含 CJK/emoji(体积),运行时项目可裁剪。

## 8. 状态与产出记录

| 日期 | 里程碑/切片 | 状态 | 产出 | 验证 | 后续 |
|------|-------------|------|------|------|------|
| 2026-07-10 | FB-M2 multilingual fallback product framebuffer | runtime_text_fb_m2_multilingual_wgpu_framebuffer_color_locale_cluster_cjk_vertical_passed | 独立 ignored integration target 经 runtime UI batching、native glyphon/cosmic fallback、native bitmap/color atlas、SDF atlas/render 与 WGPU readback导出十项产品帧；逐项 background-only 门禁，直证 `Segoe UI Emoji`→`SwashContent::Color`→RGBA、`نَ` 两个 glyph 同一 actual face、zh-Hans/ja 同码字形差异，并把 VerticalRl proof 从 sideways Latin 升级为 `Microsoft YaHei UI`/`zh-Hans` 的 `竖排布局`。修复 `StoredFontSource::FontDb` 系统 face 字节不可用后，SDF 与 native 继续共享同一 backend face-ID/bytes lineage。 | 当前源 build 以 `--no-default-features --features target-client --locked` 通过（15m16s，418 条既有 warnings）；final exact exporter 1/1（95.04s）；changed pixels 为 3751/3983/1820/2843/3321/3643/4823/1473/1179，CJK VerticalRl=1789/bbox 31×118，zh-Hans/ja 相对差 1613；PNG 1080×620、92720 bytes、1010 colors，SHA256 `352FBD3A31126E862D1BDFEDAD2F7109A6F3E94BD877BBE38D4879CF2BBF1A25`；原图目视通过；target/cargo-target 同名扫描均 0。 | FB-M2 产品可见 fallback/color/locale/complex-cluster/CJK VerticalRl 单列首段关闭；后续转 Text 02/03 CJK 多列/标点、editing geometry 与 per-run `locl` 能力。 |
| 2026-07-10 | FB-M1 locale/language end-to-end data path | runtime_text_fb_m1_locale_language_pipeline_interface_test_check_passed_runtime_focus_interrupted | 新增可序列化 `UiResolvedStyle.language`；模板 `[font].language`/aliases 解析后进入 `UiTextStyleKey`、`ShapedRunCacheKey`、direct/parallel `TextShapeRequest`、screen-space native rich spans、SDF atlas key/measure/fallback resolver；SDF 同码点按 locale 分槽。 | TDD red 明确捕获旧接口缺字段；独立 interface 回归 1/1；locked no-default production `cargo check` 通过（203.3s，415 既有 warnings）；scoped rustfmt/diff check 通过；聚合 runtime language test 在自动续接时中断且不计通过。日志 `docs/tests/runtime/text/runtime_text_fb_m1_locale_language_pipeline_validation_20260710.log`，SHA256 `B6E147D42E3DD83C0CDA01A68EFB32BEBAE5612F1B6A34E670B4ABF7D5FFEA77`。无策略文字截图。 | 等并发 lib-test lanes 释放后重跑 runtime language focused tests；再做真实 SC/JP 字形 framebuffer 对比与 cluster 单 face backend 断言。 |
| 2026-07-10 | FB-M1/FB-M2 cluster fallback spans + bounded frame-out diagnostics | runtime_text_fb_m1_cluster_fallback_diagnostics_exact_harness_passed_production_check_recovered | `shaping/fallback_spans.rs` 以 grapheme cluster 复用 CompositeFont script/range/locale resolver，并同时喂 shared cosmic/native rich spans；partial cluster 选择覆盖基字且覆盖数最多 face；`MissingGlyphLog` 按 `(face,codepoint)` 去重、1024 容量、overflow/dropped/occurrence 计数，经共享 `FontDatabase` poison recovery 后排到 `text/prepare_report.rs`；系统 face 改读真实 cmap；default manifest 补 Arabic/Hebrew/emoji/symbols；shaped cache key 加 font database generation；prepare report child split 后 `text.rs` 777 行。 | 实际 `fallback.rs` 窄层 harness 2/2、glyphon native/shared rich-span API harness 2/2、rustfmt/diff check 通过；首两次 production Cargo 被并发 Render 08 阻断，外部 owner 恢复后同一工作树 locked no-default library check 于后续 locale 切片通过（203.3s，415 既有 warnings；证据见紧邻记录）。原始日志 `docs/tests/runtime/text/runtime_text_fb_m1_cluster_fallback_diagnostics_validation_20260710.log`，SHA256 `3BCCDD5BC02E4BEC268185A28B2FFDC3D5EC2E41A8C47EE07DC1768B0B6E9E70`；无策略文字截图。 | 重跑 focused runtime tests；再做真实 CJK/Arabic/emoji fallback raster framebuffer 与 cluster single-face backend 断言。 |
| 2026-07-10 | FB-M1 D4 backend face-ID authority hard cut | runtime_text_fb_m1_backend_face_id_reconciliation_check_passed_exact_harness_passed_lib_test_build_timeout | `font/backend.rs` + `font/database.rs` 建立权威 backend DB 与双向 ID map；`font/shared.rs` 以 generation snapshot 统一 shared locale shaping/native renderer lineage；cosmic/native 直接消费实际 `LayoutGlyph.font_id`；删除 `shaping/font_id.rs` post-shape 重算桥与无消费者 API。 | production `cargo check` 通过（1m09s，416 既有 warnings，无新增）；真实 Fira Mono 窄层 harness 2/2；focused lib-test 604.2s 与 `--tests` check 304.4s 构建超时、无诊断且不计通过；日志 `docs/tests/runtime/text/runtime_text_fb_m1_backend_face_id_reconciliation_validation_20260710.log`，SHA256 `6311EDC9D779096061CD97D9F92F10C71809A0B87E4BEA61F4964A4608BFD28D`。无策略文字截图。 | D4 完成；emoji/color、partial cluster/.notdef bounded diagnostics、真实 fallback raster/framebuffer 仍 pending。 |
| 2026-07-03 | FB-M1 fallback resolver tests owner split | runtime_text_font_fallback_tests_owner_split_rustfmt_visual_cargo_deferred | 按结构规范把 `zircon_runtime/src/text/font/fallback.rs` 从 production + private regressions mixed owner 收敛为 235 行 fallback resolver leaf + `#[cfg(test)] mod tests;`;新增 `zircon_runtime/src/text/font/fallback/tests.rs` 124 行承接 4 个 primary coverage、CJK fallback、depth-limit 与 missing-codepoint diagnostic 回归。该切片只移动私有测试,不改 fallback candidate order、diagnostic、CompositeFont script/range 或 SDF/native consumer 行为。 | `rustfmt --edition 2021 --check zircon_runtime/src/text/font/fallback.rs zircon_runtime/src/text/font/fallback/tests.rs` 通过；`git diff --check -- zircon_runtime/src/text/font/fallback.rs zircon_runtime/src/text/font/fallback/tests.rs` 通过。验证图 `docs/tests/runtime/text/runtime_text_font_fallback_tests_owner_split_preview_20260703.png`,SHA256 `9D38C2B6187BBB7C647997E48EBE8D9B7518449F8D91AAE3CC1115DE69A90BE9`；验证日志 `docs/tests/runtime/text/runtime_text_font_fallback_tests_owner_split_validation_20260703.log`,SHA256 `C0F56396A1A0372D3DFA5CA5100667FBCFF63A902AFBCF31B927D329F729B403`；repo `target`、`E:\cargo-targets` 与 `D:\cargo-targets` 同名扫描 0。外部 cargo/rustc lanes 活跃,本切片不启动 focused Cargo,不声明 Cargo green。 | fallback resolver production/test owner 漂移首段关闭；emoji/color fallback、partial cluster coverage、backend-native face-id reconciliation、完整 tofu/raster 缺字路径、真实 editor typography QA 与空闲 Cargo 绿跑仍 pending。 |
| 2026-06-29 | FB-M2 shaped glyph `font_id` annotation bridge | runtime_text_fb_m2_shaped_font_id_bridge_check_passed | `FontDatabase::resolve_fallback_face_for_cluster(...)` 暴露 crate 内 cluster resolver API;`text/shaping/font_id.rs` 将 shaped glyph source range/script/codepoints 映射回 resolver-selected face 并写入 `ShapedGlyph.font_id`;native screen-space text prepare 在现有 batch loop 内汇总 `ScreenSpaceUiTextFontIdReport`,让运行时消费该桥接而不是留下 test-only/dead-code 数据面 | `rustfmt --edition 2021 --check` 覆盖 `text/font/{database.rs,fallback.rs}`、`text/shaping/{mod.rs,font_id.rs,tests.rs}`、`scene_renderer/ui/text.rs` 和本轮支撑修复文件通过;`cargo check -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0629-shaped-fontid-check --message-format short --color never` 通过(既有 warnings);`cargo test -p zircon_runtime text_fallback_glyph_carries_resolved_font_id --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0629-shaped-fontid-check --message-format short --color never -- --nocapture` 通过 1/1;视觉证据 `docs/tests/runtime/text/runtime_text_shaped_font_id_bridge_preview_20260629.png` 已检查,repo `target` 与 `E:\cargo-targets` 下同名匹配为 0 | 这只关闭 shaped output 的 post-annotation bridge 与 native prepare 统计消费;真实 glyphon/cosmic backend face-id reconciliation、per-script fallback shaping、emoji/color fallback、tofu/raster 缺字路径、SDF 非 0 face 真 raster 和完整端到端 FB-M2 断言仍 pending |
| 2026-06-29 | FB-M2 SDF fallback bridge first slice | runtime_text_fb_m2_sdf_fallback_bridge_check_passed_test_compile_timeout | `FontDatabase` 新增 `resolve_fallback_face_for_codepoint(...)`,由 `FallbackResolver::resolve_codepoint(...)` 复用 FB-M1 的 primary coverage、script/range、fallback chain 和 last-resort 规则;`graphics/scene/scene_renderer/ui/sdf_font_bake.rs` 在每个 `SdfAtlasGlyphKey` bake/measure 前按 glyph + `font_family` 构造 `FontQuery`,优先尝试 resolver 选出的 face,再保留 requested/default face fallback order。这样 SDF glyph bake 首次消费统一 fallback resolver,避免继续只按请求 font asset 烘焙缺字 face;实现仍保持 font owner 与 SDF bake owner 分层,不把 fallback 规则写进 UI/layout/render facade。 | `rustfmt --edition 2021 --check zircon_runtime/src/text/font/database.rs zircon_runtime/src/text/font/fallback.rs zircon_runtime/src/text/sdf/font_bake.rs` 通过;`cargo check -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0629-fallback-sdf-check --message-format short --color never` 通过(既有/no-default warnings);focused `cargo test -p zircon_runtime text_font_database_resolves_fallback_face_for_codepoint --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0629-fallback-sdf-check --message-format short --color never -- --nocapture` 15 分钟编译/链接超时无 Rust diagnostics,匹配进程已停止,不计 test 通过;`cargo check -p zircon_runtime --lib --tests ...` 失败于无关既有 test targets:`zircon_host_reflection_docs` 缺 `args/error/run` 模块和 `virtual_geometry_debug_snapshot_contract` 调用已移除 `RenderLayerSet::from_legacy_mask`;视觉证据 `docs/tests/runtime/text/runtime_text_sdf_fallback_bridge_preview_20260629.png` 已检查,并确认 repo `target` 下同名匹配为 0。 | 这只关闭 FB-M2 的 SDF bake face-selection bridge;cosmic/glyphon shaping 的稳定 runtime `FontFaceId` 映射、`ShapedGlyph.font_id` 实际 fallback-selected face、cluster 同 face shaped output、emoji/color fallback、tofu/raster 缺字路径和完整端到端断言仍 pending。 |
| 2026-06-29 | FB-M1 fallback resolver data-plane | runtime_text_fb_m1_fallback_resolver_data_plane_check_passed_test_compile_timeout | 新增 `text/font/fallback.rs` 作为回退解析器 leaf owner,承接 `FontDatabase`/`CompositeFontDescriptor`/`FontQuery`;按 primary coverage → CompositeFont script/range family → request/default/fallback families → last-resort 顺序解析 cluster face;加入 `DEFAULT_FALLBACK_MAX_DEPTH=10`、`FallbackResolutionSource`、`MissingGlyphLog`/`MissingGlyphDiagnostic`;`FontDatabase::fallback_candidates(...)` 改为委托 resolver 的候选顺序,并保留 cmap-aware coverage 预筛。实现前对照 UE `FCompositeFont` sub-typeface/range 分派与 Fyrox `MAX_FALLBACK_DEPTH=10`,未把规则散落到 UI 或 render facade。 | scoped `rustfmt --edition 2021 --check` 通过;`cargo check -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0629-fallback-resolver-check2 --message-format short --color never` 通过,但仍有既有/no-default warning 噪声,并包含本轮 resolver 在 FB-M2 真实 shaping bridge 接线前的未使用数据面 warnings;focused `cargo test -p zircon_runtime text_fallback --lib --no-default-features --locked ...` 编译超时无 Rust diagnostics,匹配验证进程已停止,不计测试通过;视觉证据 `docs/tests/runtime/text/runtime_text_fallback_resolver_preview_20260629.png` 已检查,不写 repo `target`。 | 继续 FB-M2:把 resolver 命中的 face 接入 cosmic/glyphon/SDF shaping bridge,让 `ShapedGlyph.font_id` 携带实际 fallback-selected `FontFaceId`;补 emoji/color fallback、cluster 同 face 断言、真实缺字 tofu/raster 诊断和完整 per-script fallback run。 |
| 2026-06-28 | FB-M1 首段:cmap-aware fallback candidate filter | runtime_text_fr_m2_fb_m1_cmap_candidate_filter_rustfmt_metadata_passed_focused_test_timeout | `text/font/coverage.rs` 作为回退候选 coverage leaf,为可解析 sfnt project faces 保存 compact cmap ranges;`FontDatabase::fallback_candidates` 在 CompositeFont/request/default/fallback family 排序后按 codepoint 剔除 Known coverage 不覆盖的 face,Unknown coverage 对系统字体和 synthetic tests 维持 permissive,避免误删不可判定候选;新增 focused database 测试锁定 Latin known face 不应覆盖 CJK codepoint、Unknown face 保留的预筛行为 | scoped `rustfmt --check` 通过;scoped `git diff --check` 仅 CRLF 提示;`cargo metadata --locked --format-version 1 --no-default-features` 通过;截图证据仍在 `docs/tests/runtime/text/runtime_text_shared_metrics_preview_20260628.png`,未写 target;focused `text_font_fallback_candidates_filter_known_cmap_coverage` lib-test 编译超时无 Rust diagnostics,本次独立 target-dir 验证进程已停止,不计 Cargo/test 通过 | 后续实现完整 `FallbackResolver`、cluster 级一致性、`ShapedGlyph.font_id` 实际命中 face、缺字诊断、深度限制、emoji/color font 路由 |
| 2026-06-27 | 计划建立 | planned | 脚本/范围感知回退 + cluster 一致性 + font_id 传出 + 缺字诊断路线 | 文档 | FB-M1 数据驱动回退链;依赖 01 CompositeFont、02 script 分段 |
