---
related_code:
  - zircon_runtime/src/ui/text/mod.rs
  - zircon_runtime/src/ui/text/shaper.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/layout_engine/visual_order.rs
  - zircon_runtime/src/graphics/text/mod.rs
  - zircon_runtime/src/graphics/text/layout/mod.rs
  - zircon_runtime/src/graphics/text/layout/measure.rs
  - zircon_runtime/src/ui/text/grapheme.rs
  - zircon_runtime/src/ui/text/hit_test.rs
  - zircon_runtime/src/ui/text/measure_cache.rs
  - zircon_runtime/src/ui/text/font_registry.rs
  - zircon_runtime/src/ui/text/rich_text.rs
  - zircon_runtime/src/ui/text/raster.rs
  - zircon_runtime/src/ui/text/resolved_layout.rs
  - zircon_runtime/src/ui/text/edit_state.rs
  - zircon_runtime/src/ui/surface/input/editable_text/ime_context.rs
  - zircon_runtime/src/asset/assets/font.rs
  - zircon_runtime/src/asset/importer/ingest/import_font_asset/mod.rs
  - zircon_runtime/src/asset/importer/ingest/import_font_asset/parse_sfnt.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/font_asset.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_font_bake.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_atlas.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/sdf_render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/shaders/sdf_text.wgsl
  - docs/zircon_runtime/graphics/text.md
  - zircon_runtime_interface/src/ui/surface/render/mod.rs
  - zircon_runtime_interface/src/ui/surface/render/resolved_style.rs
  - zircon_runtime/Cargo.toml
design_references:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts/FontCache.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts/CompositeFont.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Fonts/SlateTextShaper.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/TextLayout.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/ShapedTextCache.h
  - dev/godot/servers/text/text_server.h
  - dev/godot/modules/text_server_adv/text_server_adv.cpp
  - dev/bevy/crates/bevy_text/src/pipeline.rs
  - dev/bevy/crates/bevy_text/src/font_atlas.rs
  - dev/slint/internal/core/textlayout/sharedparley.rs
plan_sources:
  - docs/plans/zircon_runtime/render/14-2d-stack.md
  - docs/plans/zircon_editor/editor_layout/17-text-rendering-and-typography.md
  - docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/plans/engine-code-structure-convention.md
  - .codex/plans/UI SDF 字体真实 Bake 收束计划.md
status: in_progress
---

# Zircon 文本与字体渲染子系统总体计划

本目录是 `zircon_runtime` **文本与字体渲染子系统的权威深度计划**。它把当前散落在 `ui/text`(启发式布局)、`graphics/scene/scene_renderer/ui`(glyphon bitmap + fontsdf SDF 双绘制后端)与 `asset`(字体资产)中的局部能力,收敛为一条**模块化、可测、高精度、高性能**的文本主链:

```
字体文件/资产 → FontFace/字体库/回退链 → shaping(Unicode/BIDI/竖排) →
换行/度量/布局 → 字形栅格(bitmap / SDF / MSDF) → 字形图集 →
渲染提取(quad/批) → 命中测试/光标/编辑 → IME
                       ↑                              ↑
                  多线程与缓存(贯穿全链)        富文本(HTML/BBCode 预处理)
```

## 0. 为什么单独立这条线

`render/14`(2D 栈)的 `TD-M1` 已经决定"把文本 shaping/字形图集**下沉为 `graphics/text/` 共享服务**,UI 与场景 2D 共用"。但 `render/14` 的篇幅集中在**2D 场景渲染器、sprite 批集成、UI 文本路径硬切换**,并未展开文本子系统**内部**的高精度细节(BIDI/竖排/MSDF/字体回退/富文本/IME/多线程)。`editor_layout/17` 是**编辑器侧排版规范**(度量=绘制、DPI 重栅格、换行自适应),`editor_ui/03` 是**编辑器文本栈定稿**(主链贯通)。三者都**消费**一个尚未深描的运行时文本服务。

本目录就是那个服务的实现权威:**`graphics/text/**` 共享服务内部 + `core/framework/render/text/**` 契约深化**,把用户要求的 15 项能力(glyphon、SDF/MSDF 动态与预生成、UE 风格度量算法、多语言 BIDI/竖排、换行规则、渲染规则、分辨率精度、字体文件处理、图集化、多线程、Unicode、FontFace、回退规则、富文本、多平台 IME、字体回退)逐项落到文件级实施权威。

## 1. 边界与归属(与三份既有计划的勾稽)

固定分工,**不重叠、不矛盾**:

| 计划 | 拥有 | 与本目录关系 |
|------|------|------------|
| 本目录 `text/**` | 文本服务内部:字体库/FontFace/回退、shaping/Unicode/BIDI/竖排、换行/度量算法、栅格/图集、SDF/MSDF、富文本预处理、IME 接口、多线程与缓存 | **实现权威**;`graphics/text/**` + `core/framework/render/text/**` |
| `render/14`(2D 栈) | 场景 `TextRenderer` 组件、glyph quad→sprite 批、UI 文本路径**硬切换**装配、2D 排序 | `TD-M1` 的"共享服务内部"**委托本目录**;`render/14` 持有"如何把 `ShapedGlyphRun` 变成场景顶点/批" |
| `editor_layout/17` | 编辑器排版**规范**:度量=绘制四规则、字形随 `scale_factor` 重栅格、换行自适应两阶段、shrink-to-fit | **消费方**:本目录服务必须满足其度量一致性与 DPI 重栅格契约(本目录 §6.2/04) |
| `editor_ui/03` | 编辑器文本栈**贯通**:Label/Field/Console/树表同一链、CJK 一等公民、编辑链与候选窗实机 | **消费方**:其"shaping 权威未定/栅格策略未书面化/字体注册表缺失/IME 不闭环"四缺口由本目录 01/02/04/06/08 正面补齐 |
| `runtime/15`(结构规范) | owner-module 模式、命名前缀、`module_convention_gate`/`large_file_ownership_gate` | 本目录所有新增文件遵其结构规则;`ui/text` 巨型文件拆分纳入其治理 |

**契约名权威**:`render/14` 已定稿契约层类型(`ShapedGlyphRun`/`ShapedGlyph`/`ShapedLine`/`TextShapingService`/`TextStyle`/`ShapedTextCacheKey`/`GlyphAtlasFormat`/`GlyphAtlasRef`/`RenderTextSnapshot`,见 `render/14` §核心类型与接口)。本目录**沿用并扩展**这些类型,不另造同义类型;扩展项(变量字体轴、竖排朝向、富文本 span、回退命中 face)以本目录各子计划"工程落地细化"为准,扩展后回填 `render/14` 契约定义。

## 2. 现状评审(按代码核实,2026-06-27)

### 2.1 已成立(取其能力,不推倒)

| 能力 | 落点 | 成熟度 |
|------|------|--------|
| glyphon bitmap atlas 绘制后端 | `scene_renderer/ui/text.rs`(FontSystem/SwashCache/TextAtlas/TextRenderer) | 绘制端用真实字形度量(`Shaping::Advanced`) |
| fontsdf SDF 烘焙 + 图集 + 渲染 | `ui/sdf_font_bake.rs`、`ui/sdf_atlas.rs`(LRU 256 槽)、`ui/sdf_render.rs`(R8Unorm)、`shaders/sdf_text.wgsl`(smoothstep) | 单通道 SDF 全链可用,部分上传(脏槽)已设计未启用 |
| 启发式布局/换行/对齐/省略 | `ui/text/layout_engine.rs`(word/glyph wrap、`ellipsize_line`) | **不接触真实字体度量**(等宽 `font_size*0.5`)——`editor_layout/17` G1 根因 |
| 低保真 BiDi | `ui/text/layout_engine/visual_order.rs`(LTR/RTL 检测 + run 重排) | 非 UAX#9 完整算法 |
| grapheme 边界/导航 | `ui/text/grapheme.rs`(`unicode-segmentation`) | grapheme/word/line 边界齐备 |
| 命中测试 | `ui/text/hit_test.rs`(fragment 矩形 + 均匀 advance) | 随真实度量接入需改 cluster 反查 |
| 度量缓存 | `ui/text/measure_cache.rs`(宽度桶) | 在,但喂的是启发式宽度 |
| 字体回退链(硬编码) | `ui/text/font_registry.rs`(Inter/Noto/雅黑/Segoe) | 非脚本感知、非动态 |
| 富文本最小集 | `ui/text/rich_text.rs`(`**bold**`/`*italic*`/`` `code` ``) | 仅 markdown 三标记,无 HTML/BBCode |
| 字体资产/导入 | `asset/assets/font.rs`、`importer/ingest/import_font_asset/{mod.rs,parse_sfnt.rs}` | FR-M2 首段已具备 face_index/family members/变量实例/parsed metadata 与 sfnt/TTC 元数据解析;WOFF2 decode、真实 TTC/变量 fixture 与 SDF 多 face 渲染仍未闭合 |
| IME 上下文/编辑链 | `ui/surface/input/editable_text/ime_context.rs`、`edit_state.rs` | 出入站事件齐备,光标/合成 rect 用硬编码 `font_size*0.6` |
| 渲染 DTO | `iface/ui/surface/render/`(`UiShapedGlyph`/`UiShapedText`/`UiTextPaint*`) | DTO 在,布局未喂真实数据 |

### 2.2 关键缺口(本目录正面补齐)

| 缺口 | 用户需求项 | 承接子计划 |
|------|-----------|-----------|
| shaping 权威未定(glyphon 挂名未接布局、无 GSUB/GPOS) | glyphon 方案、Unicode | 02 |
| 无完整 UAX#9 BIDI、无竖排 | 多语言 BIDI、竖排 | 02 |
| 度量=启发式等宽,非真实字形 | UE 文本长度算法 | 03 |
| 换行非 UAX#14、无 CJK 行首尾禁则、无连字符 | 换行规则 | 03 |
| 栅格器选型未定、图集策略未书面化、无 DPI 重栅格契约 | 图集化、分辨率精度 | 04 |
| 仅单通道 SDF,无 MSDF、无离线预生成 | SDF/MSDF 动态与预生成、渲染规则 | 05 |
| 字体回退硬编码、非脚本感知、单 face | 回退规则、字体回退、FontFace | 01、06 |
| 富文本仅 markdown 三标记 | 富文本 HTML/BBCode | 07 |
| IME 用硬编码度量、候选窗未实机、多平台未抽象 | 多平台 IME 接口 | 08 |
| 全链单线程、无异步栅格、缓存契约缺失 | 多线程处理 | 09 |
| 字体文件仅单 face TOML、无 TTC/WOFF2/变量字体/系统字体发现 | 字体文件处理、FontFace | 01 |

## 3. 子计划地图与执行顺序

| 计划 | 文档 | 主题 | 依赖 |
|------|------|------|------|
| 01 | `01-font-resource-faces-and-database.md` | 字体文件处理 / FontFace / CompositeFont / 字体库 / 系统字体发现 / 资产与导入 | 无(最先) |
| 02 | `02-shaping-unicode-and-bidi.md` | shaping 后端 / Unicode / UAX#9 BIDI / 脚本分段 / 竖排 / cluster 映射 | 01 |
| 03 | `03-line-breaking-measure-and-layout.md` | UAX#14 换行 + CJK 禁则 / UE 风格度量算法 / 对齐与两端对齐 / 竖排布局 / 行高 | 02 |
| 04 | `04-glyph-atlas-and-rasterization.md` | swash 栅格 / shelf 图集 / 页 LRU / 脏矩形上传 / DPI 重栅格 / subpixel / hinting | 02 |
| 05 | `05-sdf-msdf-pipeline.md` | SDF(动态)/ MSDF(动态 fdsm + 离线预生成)/ 渲染规则(着色/阈值/outline/阴影)/ 分辨率无关 | 04 |
| 06 | `06-font-fallback.md` | 脚本感知回退 / Unicode 范围 / CompositeFont 回退链 / 深度限制 / tofu | 01、02 |
| 07 | `07-rich-text-html-bbcode.md` | BBCode + HTML 子集解析 / 装饰器 schema / 内联对象 / 样式 run 合并 | 02、03 |
| 08 | `08-ime-and-text-input.md` | 多平台 IME(TSF/IMM32/macOS/IBus/fcitx/Web)/ preedit/composition / 候选窗定位 / 编辑链 | 03、04 |
| 09 | `09-threading-caching-and-performance.md` | 并行 shaping / 异步栅格 / worker pool / 缓存体系 / 精度与性能预算 / 性能计数 | 01–05 |

**阶段划分**:

- **阶段 A(底座):01 → 02 → 03。** 字体库立起来,shaping 接真实后端,度量=绘制根治错位。这一段即 `render/14 TD-M1 切片 1a` 与 `editor_layout/17 G1` 的共同地基。
- **阶段 B(像素质量):04 → 05。** 栅格与图集随 DPI 重栅格(`editor_layout/17 G2`),SDF 升级 MSDF + 离线预生成。
- **阶段 C(国际化与交互):06 + 07 + 08 并行。** 回退链脚本感知、富文本 HTML/BBCode、IME 多平台实机。
- **阶段 D(性能收敛):09。** 把 A–C 的同步实现并行化、异步化,缓存契约定稿,性能计数进测试。

`render/14 TD-M1 切片 1c`(UI 文本路径硬切换)在阶段 A 末执行——服务接口(`TextShapingService`)稳定后,按 `render/14` 硬切换清单一次性迁移 `ui/text` 调用方并删除启发式路径。

## 4. 参考引擎分工(对齐 zr-reference-engine-routing)

| 引擎 | 主导领域 | 关键源码 |
|------|---------|---------|
| **UnrealEngine / Slate** | 字体缓存/CompositeFont 回退/shaped run 缓存键/HarfBuzz 双向整形/SDF 生成/度量/富文本 marshaller/平台 TextField | `SlateCore/.../Fonts/{FontCache,CompositeFont,FontCacheHarfBuzz,SlateTextShaper,SlateSdfGenerator,FontGeometryPreprocessing,SlateFontRenderer,FontMeasure}`、`Slate/.../Framework/Text/{TextLayout,ShapedTextCache,RichTextLayoutMarshaller,TextDecorators,PlatformTextField}` |
| **godot / TextServerAdvanced** | TextServer API 形态/HarfBuzz+ICU 整形/UAX#9 BIDI/UAX#14 行断/竖排朝向/MSDF 导入(msdfgen)/字体回退 | `servers/text/text_server.{h,cpp}`、`modules/text_server_adv/{text_server_adv.cpp,script_iterator.cpp}`、`editor/import/{resource_importer_dynamic_font,dynamic_font_import_settings}.cpp`、`thirdparty/msdfgen` |
| **bevy / bevy_text** | Rust/wgpu 落地形态(parley 0.8 后端):shaping→图集→quad 全链、`FontAtlasKey`、`PositionedGlyph` | `crates/bevy_text/src/{pipeline,font_atlas,font_atlas_set,parley_context,glyph,font,text_edit,cursor}.rs` |
| **slint / textlayout** | 轻量 Rust 文本布局:shaping trait、Unicode/简单双行断器、glyph cluster | `internal/core/textlayout/{sharedparley,shaping,linebreaker,linebreak_unicode,linebreak_simple,fragments,glyphclusters}.rs` |
| **Fyrox / fyrox-ui** | 极简 Rust 字体/回退(MAX_FALLBACK_DEPTH)/RectPacker 图集/换行 | `fyrox-ui/src/font/{mod,loader}.rs`、`formatted_text.rs`、`formatted_text/textwrapper.rs` |

**纪律(防凭空实现,继承 `render/14` §8.8)**:每个机制动手前先读对应子计划"参考代码"表——UE/godot 提供算法与设计样板(BIDI/行断禁则/MSDF/回退/IME 平台抽象),bevy/slint/Fyrox 提供 Rust/wgpu 落地形态(所有权、缓存键、wgpu 资源)。两类都读,不得只凭记忆。无 Rust 同类参照的机制(如完整竖排、MSDF 离线预生成格式)必须对拍测试先行。

## 5. 选型决策(全目录共享,子计划引用不重定)

| 关注点 | 选型 | 理由 | 备选 |
|--------|------|------|------|
| shaping + Unicode + BIDI + 行断 | **cosmic-text**(承接 `render/14` 既定选型) | 纯 Rust,一库聚合 rustybuzz(整形)+ swash(栅格)+ unicode-bidi(UAX#9)+ unicode-linebreak(UAX#14)+ unicode-script + fontdb(字体库/回退);CJK 与混排开箱;`render/14` 已定 | parley 0.8(bevy 现用,API 更分层但需自管 fontique) |
| 字形栅格(bitmap) | **swash**(cosmic-text 内置,亦可直用) | 彩色 emoji(ColorBitmap/ColorOutline)+ outline alpha + subpixel;bevy 同源 | fontdue(无彩色)、ab_glyph |
| 动态 SDF | **fontsdf**(既有,保留) | 已全链落地,单通道 R8 | swash outline + 自研 SDF |
| MSDF(动态 + 离线) | **fdsm**(纯 Rust MSDF/MTSDF 生成器) | 纯 Rust、无 C++ msdfgen 依赖;离线产物格式对齐 godot msdfgen 语义 | C++ msdfgen FFI(godot 路线,引入构建复杂度) |
| 字体数据库 / 系统字体发现 | **fontdb**(cosmic-text 内置) | 系统字体枚举 + family/weight/style 索引 + 回退候选 | font-kit(更重) |
| 富文本解析 | **自研 BBCode + HTML 子集解析器** | 标签 schema 受控、对齐 godot `RichTextLabel` BBCode 与 UE marshaller | html5ever(过重,安全面大) |
| IME 平台层 | **winit IME 事件 + 平台扩展抽象** | 复用既有 `ime_context` 出入站契约;平台特化经 `zircon_app` 注入 | 直接绑 TSF/IBus(放入平台插件) |

**隔离硬规则**:`cosmic_text`/`fontsdf`/`fdsm`/`fontdb`/`swash` 等第三方类型**只允许**出现在 `graphics/text/` 实现层的指定隔离文件内(见各子计划),出口一律契约层类型(`ShapedGlyphRun` 等 serde 可序列化、无 wgpu/无第三方句柄)。`core/framework/render/text/**` 与 `zircon_editor`/`zircon_app` 不得直接 import 上述库。

## 6. 全局工程约定(各子计划"工程落地细化"共享,不重定)

1. **模块归属**:契约层 `zircon_runtime::core::framework::render::text`(serde、无 wgpu);实现层 `zircon_runtime::graphics::text`(共享服务,持缓存/图集/隔离层);场景渲染器 `graphics/scene/scene_renderer/text`(归 `render/14`);UI 消费方 `ui/text`(硬切换为服务适配器)。不新增 crate。
2. **度量=绘制(继承 `editor_layout/17` 四规则)**:任何布局几何(advance/kerning/ascent/descent/换行点)必须来自 shaping 服务的真实字形度量,与绘制端**同一来源**。禁止任何路径回退到等宽近似。
3. **字形随 DPI 重栅格**:栅格输入 `physical_px = logical_px × scale_factor`;图集 key 含 `scale_factor` 量化桶(接 `editor_layout/17 §3.4`、`render/14`)。SDF/MSDF 因分辨率无关,bake 尺寸固定、运行时按 `font_size` 缩放采样。
4. **缓存键不持引用**:shaped run / measure / atlas 的缓存键一律用 `font_id + size_bits(f32::to_bits) + features_hash + 文本 hash`,不持文本/字体对象引用(改造自 UE `FCachedShapedTextKey`,见 `render/14` §核心类型)。
5. **图集格式**:`R8Unorm`(alpha mask、SDF)/ `Rgba8Unorm`(彩色 emoji、MSDF);格式分组分页(`GlyphAtlasFormat`);页级 LRU,本帧引用页不可逐出(`render/14` §目标架构同款)。
6. **测试命名(继承 `render/14` index §8.6)**:`text_<topic>_*` 单测(布局/度量/回退/富文本/缓存的确定性断言)、`render_text_*` 服务集成、`render_product_text_*` 抓帧对拍、`render_perf_text_*` 性能计数(shape 次数/栅格次数/图集上传字节,确定性计数断言;时间类只观测)。
7. **里程碑优先(milestone-first)**:切片期 `cargo check -p zircon_runtime --lib --locked`;里程碑末 `cargo test -p zircon_runtime <过滤词> --locked`。UI 文本硬切换以 `ui/tests` 全量文本测试为闸门(清单见 `render/14` §UI 文本路径硬切换清单)。
8. **实施权威**:各子计划"## 工程落地细化"是该计划实施权威——文件落点、类型签名、算法、切片步骤、测试清单以该章节为准,与正文概述冲突时以细化章节为新。
9. **结构纪律(遵 `runtime/15`)**:owner-module 模式,root `mod.rs` 留薄 façade;`ui/text/layout_engine.rs`/`graphics/.../ui/text.rs` 等大文件按 owner 叶子拆分;新增文件按命名前缀词表去冗余前缀。

## 7. 能力覆盖矩阵(用户需求 → 承接子计划)

| 用户需求项 | 承接子计划 | 备注 |
|-----------|-----------|------|
| glyphon 方案 | 02、04 | 保留为 bitmap atlas 绘制后端;布局接 cosmic-text(glyphon 同源) |
| SDF 方案(动态/预生成) | 05 | 动态 fontsdf 保留;离线预烘焙产物格式定稿 |
| MSDF 方案(动态/预生成) | 05 | fdsm 动态生成 + 离线预生成;多通道 + 中线通道(MTSDF) |
| 文本长度计算算法(UE 对齐、BIDI、竖排) | 03、02 | `FShapedGlyphSequence::GetMeasuredWidth` 子范围度量对齐 |
| 多语言国际化 BIDI | 02 | UAX#9(cosmic-text/unicode-bidi);run 重排 + 镜像字符 |
| 竖排等模式 | 02、03 | 朝向枚举(对齐 godot `Orientation`);竖排 advance/baseline |
| 换行规则 | 03 | UAX#14 + word/glyph + CJK 行首尾禁则 + 连字符 + 两端对齐 |
| 渲染规则 | 05、04 | bitmap alpha 混合 / SDF/MSDF 阈值 + `fwidth` 抗锯齿 + outline/阴影/下划线 |
| 分辨率精度 | 04 | 物理像素栅格、subpixel 定位、hinting、scale 量化桶 |
| 字体文件处理 | 01 | TTF/OTF/TTC/WOFF2、变量字体轴、face 索引 |
| 图集化生成 | 04 | shelf 分配、多页、脏矩形增量上传、LRU 逐出 |
| 多线程处理 | 09 | 并行 shaping、异步栅格上传、worker pool |
| Unicode 支持 | 02 | grapheme/script/规范化/组合字符/emoji 序列 |
| FontFace | 01 | `FontFace`(单 face)/ `CompositeFont`(family+回退)分层 |
| 回退规则 / 文本字体回退 | 06 | 脚本感知 + Unicode 范围 + 链式 + 深度限 + tofu |
| 富文本(HTML/BBCode) | 07 | BBCode 全集 + HTML 受控子集 + 装饰器 + 内联对象 |
| 多平台 IME 输入法接口 | 08 | TSF/IMM32(Win)、NSTextInputClient(mac)、IBus/fcitx(Linux)、Web |

## 8. 全局验收与测试基线

- 切片期:`cargo check -p zircon_runtime --lib --locked`。
- 里程碑测试:`cargo test -p zircon_runtime text --locked` + UI 文本全量回归(`ui/tests/{text_shaper,text_layout,text_hit_testing,render_text_fields,widget_text_input_pointer,surface_dirty_mui,boundary}`)。
- 产物对拍:`render_product_text_*`(中英混排/CJK/阿拉伯 RTL/竖排/SDF/MSDF/emoji)+ `ZR_RENDERDOC_CAPTURE_NEXT=1` 抓帧对照 UE/godot 同串。
- 多语料黄金集:维护 `text_corpus`(拉丁/CJK/阿拉伯/希伯来/天城文/泰文/emoji/混排)驱动度量与换行对照表,期望值以参考引擎或 Unicode 标准用例标定。
- 文档镜像:每里程碑后更新 `docs/zircon_runtime/ui/text*`、`docs/zircon_runtime/asset/assets/font.md` 镜像,并回填本目录状态表。

## 9. 风险与回退

- **shaping 库 CJK 排版细节不足**:以 `ShapedGlyphRun` 隔离选型,cosmic-text 不足时可换 parley/自研 rustybuzz 绑定而不动调用方(`render/14` 同款风险与对策)。
- **UI 文本硬切换面大**:阶段 A 末以 UI 全量文本测试为闸门,失败修服务不回退双路径。
- **MSDF 离线产物无 Rust 同类参照**:对拍测试先行,产物格式逐字节断言对照 godot msdfgen 语义。
- **竖排为长尾**:V1 落朝向枚举 + 横排正字布局 + 竖排 advance 主轴;旋转字形(`vert`/`vrt2` GSUB)与标点居中为 V2。
- **IME 平台实机**:winit 事件为基线,平台特化(TSF/IBus)落 `zircon_app` 平台层,运行时只持中立 `ime_context` 契约。

## 10. 状态与产出记录

| 日期 | 里程碑/切片 | 状态 | 产出 | 验证与证据 | 后续 |
|------|-------------|------|------|------------|------|
| 2026-06-28 | 阶段 A / SH-M2 RTL mirrored punctuation first slice | runtime_text_sh_m2_rtl_mirrored_punctuation_check_passed | `ui/text/layout_engine/visual_order.rs` 在当前低保真 BiDi visual-order scaffold 中加入单码点镜像表：RTL visual span 反转后对括号、箭头和常见成对符号做 visual glyph text 替换，同时保持 source ranges 不变；新增 `text_bidi_mirrors_paren_in_rtl` 与 `text_bidi_mirrors_arrow_in_rtl` 锁定 visual `(` 来自原 source 14..15、visual `)` 来自原 source 9..10、visual `←` 来自原 source 5..8 | `rustfmt --check` 覆盖 `visual_order.rs` 与 `layout_engine/tests.rs` 通过；`cargo test -p zircon_runtime text_bidi_mirrors --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-bidi-mirror --message-format short --color never -- --nocapture` 通过 2/2；视觉证据 `docs/tests/runtime/text/runtime_text_rtl_mirrored_punctuation_preview_20260628.png` 已检查 | 这只关闭 SH-M2 mirror-table 首段；完整 UAX#9 level/isolate、script segmentation、cosmic/unicode-bidi cutover、真实 shaping 后端镜像与竖排仍 pending |
| 2026-06-28 | 阶段 A / LB-M3 Auto/Mixed first-strong base direction slice | runtime_text_lb_m3_first_strong_base_direction_check_passed | `layout_engine.rs` 将 `Auto` 与现有 `Mixed` request 按首强字符解析为 concrete paragraph base direction，再用该方向计算 logical Start/End；render extract 的 mixed-direction cases 现在断言 resolved layout/line direction 是 concrete LTR/RTL，同时继续验证 run-level visual order 与 source/visual ranges | `rustfmt` 通过；`cargo check -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-first-strong-check` 通过(仅既有 warnings)；直接运行产出的 runtime lib-test binary：`first_strong` 4/4、`start_end` 6/6、`mixed_direction` 1/1、`neutral_separator` 1/1、`rich_directional_ellipsis` 1/1 均通过；视觉证据 `docs/tests/runtime/text/runtime_text_first_strong_direction_preview_20260628.png` 已检查，并确认文本验证图不写 repo `target` | 这只关闭 Auto/Mixed 首强 base direction；完整 UAX#9 level/isolate/mirror、script segmentation、justify、tab stop、shrink/clamp、ellipsis variants、vertical layout 与完整 LB-M3/LB-M4 仍 pending |
| 2026-06-28 | 阶段 A / LB-M3 logical Start/End alignment first slice | runtime_text_lb_m3_rtl_start_end_align_check_passed | `UiTextAlign` 增加并保留 `Start` / `End`，surface parser 不再把逻辑对齐提前降级为 left/right；UI layout 按 resolved line direction 计算 Start/End，screen-space text batch 携带 `text_direction`，native glyphon 与 SDF draw-plan 同步按方向映射逻辑对齐 | `rustfmt --check` 覆盖本切片 interface/layout/render/native/SDF 触达文件通过；`zircon_runtime_interface --lib` check 通过；`zircon_runtime --lib --no-default-features` check 通过(仅既有 warnings)；`cargo test -p zircon_runtime start_end --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-align` 通过 3/3；`render_extract_preserves_logical_start_text_align` 通过 1/1；验证路径暴露并修复结构守卫 `mesh_pipeline_variant_cache_owner.rs` moved-value 支撑阻塞，`runtime_15_non_base_mesh_variant_cache_owner_is_wired` 通过 1/1；视觉证据 `docs/tests/runtime/text/runtime_text_rtl_start_end_alignment_preview_20260628.png`，并确认文本验证图不写 repo `target` | 这只关闭显式 RTL base direction 下的 logical Start/End 首段；`Mixed` UAX#9 base-direction resolution、justify、tab stop、shrink/clamp、ellipsis variants、vertical layout 与完整 LB-M3/LB-M4 仍 pending |
| 2026-06-28 | 阶段 A / LB-M2 CJK open punctuation line-end first slice | runtime_text_lb_m2_cjk_open_punctuation_line_end_check_passed | `graphics/text/layout/kinsoku.rs` 继续作为 CJK 禁则 owner，新增开标点行尾禁止首段；`line_break_chunks(...)` 把 `"中（文"` 规整为 `"中"` / `"（文"`，并把开标点 protected chunk 标为不可 glyph fallback；UI Word wrap 只消费 shared metadata | RED 先证明旧行为拆成 3 行；`rustfmt --check` 通过；`line_break_chunks_keep_cjk_open_punctuation_with_following_text` 通过 1/1；`text_wrap_cjk_kinsoku_no_trailing_open_punctuation` 通过 1/1；既有 line-start kinsoku 与 UAX14 CJK 回归各通过 1/1；runtime lib check 通过(仅既有 warnings)；视觉证据 `docs/tests/runtime/text/runtime_text_cjk_open_punctuation_preview_20260628.png`，并确认文本验证图不写 repo `target` | 这只关闭开标点行尾禁则首段；完整 greedy line breaker、完整 JIS/UAX line-head/line-tail 禁则表、squeeze/overhang、tab/justify/shrink/ellipsis variants、竖排布局与更完整 glue 策略仍 pending |
| 2026-06-28 | 阶段 A / LB-M2 long-word glyph fallback + NBSP glue first slice | runtime_text_lb_m2_long_word_nbsp_check_passed | `graphics/text/layout/line_break.rs` 继续作为 LB-M2 chunk metadata owner；普通过宽 word chunk 保持可 glyph fallback，含 U+00A0 的 chunk 关闭 glyph fallback，使 NBSP glue group 在 Word wrap 下保持不可断并允许 overhang；`ui/text/layout_engine.rs` 只消费 `allow_glyph_fallback`，不持有 NBSP 规则；同步修复下层 review guard `plugin_importer_dx/d13_importer_sdk.rs` 的 stale plan-status include 路径 | `rustfmt --check` 通过；`text_wrap_long_word_falls_back_to_glyph` 通过 1/1；`word_wrap_keeps_non_breaking_space_group_together` 通过 1/1；runtime lib check 通过(仅既有 warnings)；视觉证据 `docs/tests/runtime/text/runtime_text_long_word_nbsp_preview_20260628.png`，并确认文本验证图不写 repo `target` | 这只关闭普通长词 fallback 与 NBSP glue 首段；完整 greedy line breaker、行尾禁则/open punctuation push-out、squeeze/overhang、tab/justify/shrink/ellipsis variants、竖排布局与更完整 glue 策略仍 pending |
| 2026-06-28 | 阶段 A / LB-M2 soft hyphen break suffix first slice | runtime_text_lb_m2_soft_hyphen_break_suffix_check_passed | `graphics/text/layout/line_break.rs` 继续作为 LB-M2 chunk owner，`LineBreakChunk` 增加显式 source range 与可选 break suffix；U+00AD 不进入 visual chunk text，只有真实 width wrap 发生时才在上一行追加普通 `-`；`ui/text/layout_engine.rs` 消费 metadata 并保留 pending suffix，`ui/text/hit_test.rs` 对 source/visual byte length 不一致的 soft-hyphen suffix run 走保守 source offset 映射 | `rustfmt --check` 通过；`cargo test -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never text_wrap_soft_hyphen_inserts_hyphen` 通过 1/1；`text_hit_test_soft_hyphen_break_suffix_maps_to_source_hyphen` 通过 1/1；runtime lib check 通过(仅既有 warnings)；soft-hyphen 改动后 `text_wrap_cjk_kinsoku_no_leading_punctuation` 与 `word_wrap_uses_uax14_cjk_break_opportunities` 回归各通过 1/1；视觉证据 `docs/tests/runtime/text/runtime_text_soft_hyphen_preview_20260628.png`，并确认文本验证图不写 repo `target` | 这只关闭 soft hyphen 断行显示与源映射首段；完整 greedy line breaker、行尾禁则/open punctuation push-out、不可断空白、long-word fallback、justify/shrink/tab、竖排布局与真实 fallback-selected `FontFaceId` 仍 pending |
| 2026-06-28 | 阶段 A / LB-M2 CJK kinsoku line-start punctuation first slice | runtime_text_lb_m2_cjk_kinsoku_line_start_check_passed | 新增 `graphics/text/layout/kinsoku.rs` 作为 CJK 行首禁则首个 owner leaf；`LineBreakChunk` 增加 `allow_glyph_fallback` 元数据，`line_break_chunks(...)` 在共享 layout 层合并或标记行首禁则关闭标点 chunk，`ui/text/layout_engine.rs` 只消费该 metadata，不持有标点表；新增 `text_wrap_cjk_kinsoku_no_leading_punctuation` 锁定 `"中文。"` 窄宽 Word wrap 输出 `"中"` / `"文。"` 且无 line 以 `。` 开头；同步修复 code-review guard 支撑文件 `plugin_importer_dx.rs` 的 child-module path | RED 先证明旧实现会产出 3 行；`rustfmt --check zircon_runtime/src/graphics/text/layout/mod.rs zircon_runtime/src/graphics/text/layout/line_break.rs zircon_runtime/src/graphics/text/layout/kinsoku.rs zircon_runtime/src/ui/text/layout_engine.rs zircon_runtime/src/ui/text/layout_engine/tests.rs zircon_runtime/src/tests/runtime_absorption/code_review_findings/plugin_importer_dx.rs` 通过；`cargo test -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never text_wrap_cjk_kinsoku_no_leading_punctuation` 通过 1/1；`word_wrap_uses_uax14_cjk_break_opportunities` 通过 1/1；`text_shape_` 通过 6/6；`cargo check -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never` 通过(仅既有 warnings)；截图证据 `docs/tests/runtime/text/runtime_text_cjk_kinsoku_preview_20260628.png`，确认文本验证图仍不写 repo `target` | 这只关闭行首禁则的关闭标点首段；完整行尾禁则、开括号追い出し、squeeze/半角挤压、soft hyphen、不可断空白、long-word fallback 策略、justify/shrink/tab 与竖排布局仍 pending |
| 2026-06-28 | 阶段 A / LB-M2 UAX#14 Word-wrap consumption first slice | runtime_text_lb_m2_uax14_word_wrap_cjk_check_passed | 新增 `graphics/text/layout/line_break.rs` 作为首个 LB-M2 layout owner，`line_break_chunks(...)` 消费 shaped-run cluster soft-break flags 并输出 source chunks；`graphics/text/layout/mod.rs` 只做薄导出；`ui/text/layout_engine.rs` 的 Word wrap 不再按 ASCII space `split_inclusive(' ')` 生成唯一分块，而是消费共享 UAX#14 chunks；新增 `word_wrap_uses_uax14_cjk_break_opportunities` 锁定 `"中文"` 在窄宽度下按 CJK UAX#14 机会换成两行 | `rustfmt --check zircon_runtime/src/graphics/text/layout/mod.rs zircon_runtime/src/graphics/text/layout/line_break.rs zircon_runtime/src/ui/text/layout_engine.rs zircon_runtime/src/ui/text/layout_engine/tests.rs` 通过；`cargo test -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never word_wrap_uses_uax14_cjk_break_opportunities` 通过 1/1；`cargo check -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never` 通过(仅既有 warnings)；截图证据仍为 `docs/tests/runtime/text/runtime_text_shared_metrics_preview_20260628.png`，已确认 repo `target` 下无文本验证图 | 这只是 LB-M2 消费首段；完整贪心断行、CJK kinsoku、soft hyphen 插入、long-word glyph fallback、justify/shrink/tab、竖排布局、真实 fallback-selected `FontFaceId` 与 UAX#9 mirror/isolate 仍 pending |
| 2026-06-28 | 阶段 A / SH-M1 support fix + LB-M2 UAX#14 break opportunity projection | runtime_text_sh_m1_uax14_break_flags_focused_tests_passed | 修复 focused lib-test 的下层支撑阻塞：typed-error review guards 改指当前 importer owner，camera-loop 测试闭包补上 `FrameSubmissionSourcePayloads` 参数；新增 `graphics/text/shaping/line_break.rs` 作为 UAX#14 leaf owner，`cosmic.rs` 在 cluster-start glyph 上投影 soft/mandatory break flags，直接依赖 `unicode-linebreak` 并离线同步 lockfile；新增 word-space 与 CJK break opportunity focused tests，synthetic end-of-text mandatory break 不进入内容 glyph | `rustfmt --check zircon_runtime/src/graphics/text/shaping/{mod.rs,cosmic.rs,line_break.rs,tests.rs}` 通过；`cargo metadata --offline --format-version 1 --no-default-features` 通过；`cargo check -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never` 通过(仅既有 warnings)；`cargo test -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check --message-format short --color never text_shape_` 通过 6/6；截图证据仍为 `docs/tests/runtime/text/runtime_text_shared_metrics_preview_20260628.png`，已视觉检查，`target/runtime_text_shared_metrics_preview_20260628.png` 与 `target/runtime_text_shaping_owner_preview_20260628.png` 均不存在 | UAX#14 现在只是 shaped-run cluster flag 数据面；完整贪心断行、CJK kinsoku、soft hyphen、justify/shrink/tab、脚本分段、UAX#9 mirror/isolate、真实 fallback-selected `FontFaceId` 与竖排 metrics 仍 pending |
| 2026-06-28 | SH-M1 owner slice: cosmic-backed `ShapedGlyphRun` contract and isolated shaping owner | runtime_text_sh_m1_shaping_owner_core_check_passed_focused_libtest_blocked | Added neutral `core/framework/render/text/{shaped_run.rs,shaping_service.rs}` contracts and `graphics/text/shaping/{mod.rs,cosmic.rs}` owner; `cosmic.rs` is now the isolated glyphon/cosmic-text Buffer/LayoutGlyph projection point and emits glyph id, source_range, visual_range, advance, baseline, direction, cluster flags, and rotation contract data; `graphics/text/layout/measure.rs` now derives line width/line metrics/per-grapheme advances from `ShapedGlyphRun` instead of importing backend text types directly | scoped rustfmt passed; `cargo check -q -p zircon_runtime --lib --no-default-features --locked --target-dir E:\cargo-targets\zircon-runtime-text-0628-check` passed with existing warnings only; focused `cargo test -q -p zircon_runtime text_shape_ --lib --no-default-features --locked` compiles past old importer include guards but is blocked by unrelated camera-loop lib-test closure signature errors; screenshot evidence remains `docs/tests/runtime/text/runtime_text_shared_metrics_preview_20260628.png` and target image checks are false | Actual fallback-selected `FontFaceId` remains pending; full UAX#9 isolate/mirroring, script segmentation, UAX#14 break data, CJK kinsoku, vertical metrics/rotation policy beyond contract fields, and UI hard cutover from `visual_order.rs` remain future SH/LB slices |
| 2026-06-28 | 阶段 A / SH-M1 + LB-M1 shaped glyph advance DTO projection | runtime_text_sh_lb_m1_shaped_glyph_advances_interface_check_passed_runtime_check_timeout | `UiResolvedTextLine` 新增 `glyph_advances`，由 `ui/text/layout_engine.rs` 从 shared backend grapheme metrics 填充；`UiShapedGlyph` 新增 `font_id`、`cluster_flags` 和 `rotation` 字段；`UiShapedText::from_resolved_layout(...)`、rich text paint runs 与 `UiRenderCommand::text_paint(...)` 的 caret/selection/composition/rich-run geometry 改用 measured advances，避免 render-facing DTO 再把行宽平均切片；旧序列化数据仍通过 default/empty fallback 保持可读 | scoped rustfmt 通过；`cargo check -q -p zircon_runtime_interface --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0628-interface-contract` 通过；`cargo check -q -p zircon_runtime_interface --tests --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0628-interface-contract` 通过；`cargo metadata --locked --format-version 1 --no-default-features` 通过；scoped diff-check 仅 CRLF 提示；截图证据仍在 `docs/tests/runtime/text/runtime_text_shared_metrics_preview_20260628.png`，确认 `target/runtime_text_shared_metrics_preview_20260628.png` 不存在；`cargo check -q -p zircon_runtime --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0628-check` 244s 编译超时无 Rust diagnostics，匹配验证进程已停止 | 这仍是 neutral DTO projection，不是完整 `ShapedGlyphRun`；后续需把实际 fallback-selected `font_id`、cluster source_range、UAX#9/UAX#14、vertical Cw90、full font fallback resolver 和 runtime focused Cargo 绿跑补齐 |
| 2026-06-28 | 阶段 A / FR-M2 FontAsset UI registry schema convergence | runtime_text_fr_m2_font_asset_ui_registry_convergence_core_check_passed_focused_test_timeout | `FontAssetRenderStrategy::effective_render_mode(...)` 与 `FontAsset::effective_render_mode()` 成为 render-mode 优先级/allow_native/allow_sdf clamp 的单一资产 helper;screen-space UI manifest loader 改为调用该 helper,避免重复 schema 分支;`UiFontRegistry::register_font_asset` 现在消费 effective render mode,并把注册 family 与 `fallback_families` 合并进 UI fallback chain,空白过滤、大小写去重;旧 `FontAsset { source,family,render_mode }` test literal 更新到完整 schema,新增 focused registry 测试覆盖 strategy Auto→Sdf clamp 与 asset fallback-family dedupe | scoped `rustfmt --check` 通过;`cargo metadata --locked --format-version 1 --no-default-features` 通过;`cargo check -q -p zircon_runtime --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0628-check` 通过(仅既有 warning);截图证据仍在 `docs/tests/runtime/text/runtime_text_shared_metrics_preview_20260628.png`,确认 `target/runtime_text_shared_metrics_preview_20260628.png` 不存在;focused `text_font_registry_uses_asset_render_strategy_default_mode` 首次在 warm target-dir 命中 Cargo fingerprint path 写入错误,独立 target-dir 重跑 604s 编译超时无 Rust diagnostics,已停止 2 个匹配验证进程,不计 Cargo/test 通过 | UI registry 与 asset schema 首段已收敛;仍需完整 `FallbackResolver`、CompositeFont 资产声明、cluster 同 face/font_id 传出、缺字诊断、WOFF2 decode、变量字体 fixture 绿跑、TTC tests 绿跑和 SDF 非 0 face 真 raster |
| 2026-06-28 | 阶段 A / FR-M2 + FB-M1 cmap-aware fallback candidate filter | runtime_text_fr_m2_fb_m1_cmap_candidate_filter_rustfmt_metadata_passed_focused_test_timeout | 新增 `graphics/text/font/coverage.rs` leaf owner,从可解析 sfnt face 提取 compact cmap ranges;`FontDatabase` 的项目字体记录保存 `FontCoverage`,系统字体和不可解析测试 face 保持 Unknown permissive coverage;`fallback_candidates(codepoint, query, composite)` 现在按请求 codepoint 过滤已知不覆盖的 face,避免 Composite/request/fallback family 候选把必然缺字的 Latin face 排在前面;新增 focused 测试覆盖同一 family 内 Known Latin face 与 Unknown face 的 CJK 过滤差异 | scoped `rustfmt --check` 通过;scoped `git diff --check` 仅 CRLF 提示;`cargo metadata --locked --format-version 1 --no-default-features` 通过;截图证据仍在 `docs/tests/runtime/text/runtime_text_shared_metrics_preview_20260628.png`,确认 `target/runtime_text_shared_metrics_preview_20260628.png` 不存在;focused `text_font_fallback_candidates_filter_known_cmap_coverage` lib-test 编译 244s 超时无 Rust diagnostics,本次独立 target-dir 验证进程已停止,不计 Cargo/test 通过 | 这只是 06 的 cmap 精筛数据面首段;仍需完整 `FallbackResolver`、cluster 同 face、font_id 传出、缺字诊断、深度限制、emoji/color fallback、SDF 非 0 face 真 raster 与 WOFF2/变量 fixture 绿跑 |
| 2026-06-27 | 文本子系统计划集建立 | planned | 建立 `docs/plans/zircon_runtime/text/` 目录:index + 01–09 子计划;定位为文本服务实现权威,与 render/14、editor_layout/17、editor_ui/03、runtime/15 勾稽边界 | 文档创建;未改生产代码 | 阶段 A:01 字体库 → 02 shaping 接入 → 03 度量=绘制;随后 render/14 TD-M1 切片 1c UI 硬切换 |
| 2026-06-28 | 阶段 A / SH-LB-M1 共享字形度量 owner 首段 | runtime_text_shared_metrics_owner_core_check_passed_focused_test_timeout_visual_evidence | 新增 `graphics/text/layout/{mod,measure}.rs` crate-private owner,通过 `UiSharedTextShaper` 让 UI layout/measure/cache/hit-test 消费共享 glyph metrics;移除触达路径中的 fixed half-em 等宽度量与 Native/SDF "not connected" fallback 状态;同步 `docs/zircon_runtime/{ui/text.md,graphics/text.md}` | `rustfmt` 通过;静态扫描确认触达文本路径无 fixed half-em 旧公式/旧 fallback 文案;`cargo check -p zircon_runtime --lib --no-default-features --locked --jobs 1` 通过(仅既有 warning);截图证据 `docs/tests/runtime/text/runtime_text_shared_metrics_preview_20260628.png`;focused Cargo test 超时无 Rust diagnostics,未发现本轮验证残留进程 | 继续 01 FontDatabase/FontFace 与完整 02 `ShapedGlyphRun`/UAX#9、03 UAX#14/CJK 禁则/justify/竖排;后续需在空闲编译通道重跑 `cargo test -p zircon_runtime text --locked` |
| 2026-06-28 | 阶段 A / FR-M1 字体契约与数据库 owner 首段 | runtime_text_fr_m1_font_contract_database_core_check_passed_focused_test_timeout | 新增 `core/framework/render/text/font` 中立 FontFace/FontFamily/CompositeFont/FontQuery DTO 与 `graphics/text/font` crate-private in-memory `FontDatabase`;`ui/text/font_registry.rs` 默认 fallback chain 改由 runtime font owner 提供,移除 UI-local 默认字体字面量 | `rustfmt` 通过;静态扫描确认 UI registry 无默认字体字面量;`cargo check -p zircon_runtime --lib --no-default-features --locked --jobs 1` 通过(仅既有 warning);`text_font` focused Cargo test 编译超时无 Rust diagnostics,匹配验证进程已停止 | 继续 FR-M1 真实 `fontdb`/系统字体发现、项目字体注册、glyphon/SDF 共库;随后 FR-M2 字体文件解析与变量字体 |
| 2026-06-28 | 阶段 A / FR-M1 共享 FontDatabase 接入 native + SDF | runtime_text_fr_m1_shared_font_database_native_sdf_core_check_passed_focused_test_timeout | `graphics/text/font::FontDatabase` 支持真实字体文件注册、source path 去重、共享 face bytes 与 glyphon `FontSystem` 注入;screen-space UI native glyphon 与 SDF bake/render 路径均通过同一个 `FontDatabase` 解析字体资产,fontsdf 由 `FontFaceId` 取得共享 bytes | scoped `rustfmt` 通过;`cargo check -q -p zircon_runtime --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0628-check` 通过(仅既有 warning);沿用截图证据 `docs/tests/runtime/text/runtime_text_shared_metrics_preview_20260628.png`;focused `text_font` 仍为编译超时记录,不计测试通过 | 继续 FR-M1 系统字体发现/CompositeFont 候选枚举;进入 FR-M2 TTC/WOFF2/变量轴/cmap 解析与 02/03 完整 shaping/layout 合同 |
| 2026-06-28 | 阶段 A / FR-M1 系统字体发现与 CompositeFont 候选 | runtime_text_fr_m1_system_fonts_composite_candidates_core_check_passed_focused_test_timeout | `FontDatabase` 通过 glyphon/fontdb 枚举系统字体并纳入 Zircon `FontFaceId`/family 查询表,screen-space UI text system 初始化时加载系统字体索引;新增 CompositeFont sub-font script/range 候选枚举,按 sub-font → default → request → runtime fallback 的数据面顺序产出候选 face | scoped `rustfmt --check` 通过;静态扫描确认 native/SDF render paths 无直接 `load_font_file`/独立 fontsdf path cache 回流;`cargo check -q -p zircon_runtime --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-text-0628-check` 通过(仅既有 warning);截图证据仍在 `docs/tests/runtime/text/runtime_text_shared_metrics_preview_20260628.png`,未写 target | FR-M1 仍需 CompositeFont 资产接线/策略开关;FR-M2/06 继续 TTC/WOFF2/变量轴/cmap 精筛、脚本感知回退深度和 tofu |
| 2026-06-28 | 阶段 A / FR-M2 字体资产元数据 schema 与 sfnt 解析首段 | runtime_text_fr_m2_sfnt_metadata_schema_rustfmt_metadata_passed_cargo_check_timeout | `FontAsset` schema 补齐 `face_index`、family members、variable instances、fallback families、render strategy 与 parsed metadata;导入器硬切为 `import_font_asset/{mod.rs,parse_sfnt.rs}` owner,解析 sfnt/TTC face count、name、OS/2 weight/width/style、`fvar` axes/named instances 与 cmap coverage;runtime manifest 把 face index 传入 shared `FontDatabase` 注册键;`FontDatabase::register_font_file` 读取 selected face 的 family/weight/style/stretch 元数据;测试内 TTC fixture 覆盖多 face metadata 和 database face-index 去重;SDF bake 对 `fontsdf` 不支持的非 0 face 显式 fallback,避免静默用错 face 0 | scoped `rustfmt --check`、scoped `git diff --check` 与 `cargo metadata --locked --format-version 1 --no-default-features` 通过;截图证据仍在 `docs/tests/runtime/text/runtime_text_shared_metrics_preview_20260628.png`,未写 target;focused metadata-ingestion/SDF-face-index/TTC tests 与 wider `text_font` 本轮编译超时无 Rust diagnostics,不计 Cargo/test 通过 | WOFF2 decode、变量字体 fixture、TTC/变量测试绿跑、family_members/fallback_families/render_strategy 消费、SDF face-index-capable raster path、06 cmap/script 精筛仍继续 |
| 2026-06-28 | 阶段 A / FR-M2 render strategy 默认渲染模式接线 | runtime_text_fr_m2_render_strategy_default_mode_rustfmt_metadata_passed_focused_test_timeout | `graphics/scene/scene_renderer/ui/font_asset.rs` 在 manifest 加载边界消费 `FontAsset.render_strategy.default_mode`,旧 `render_mode` 字段继续优先;`allow_native`/`allow_sdf` 只在有效默认模式解析处收窄 Auto/Native/Sdf,不把 schema 细节扩散到 text renderer;新增 focused 单元测试覆盖 strategy 默认、旧字段优先与 disallowed Auto clamp;`sdf_font_bake.rs` 的非 0 face fallback 测试改用 `TemporaryFontManifest` 同时持有 manifest/source 并在 Drop 中清理,避免临时字体源文件泄漏 | scoped `rustfmt --check` 通过;scoped `git diff --check` 仅 CRLF 提示;`cargo metadata --locked --format-version 1 --no-default-features` 通过;截图证据仍在 `docs/tests/runtime/text/runtime_text_shared_metrics_preview_20260628.png`,确认 `target/runtime_text_shared_metrics_preview_20260628.png` 不存在;focused `render_strategy_default_mode_feeds_ui_font_default` runtime lib-test 编译 184s 超时无 Rust diagnostics,本次独立 target-dir 验证进程已停止,不计 Cargo/test 通过 | `render_strategy.default_mode` 首段已消费;`family_members`/`fallback_families` 注入完整 FontDatabase 资产注册、WOFF2 decode、变量字体 fixture、TTC/变量测试绿跑、SDF face-index-capable raster path 与 06 cmap/script 精筛仍继续 |
| 2026-06-28 | 阶段 A / FR-M2 FontAsset family/fallback 数据面接线 | runtime_text_fr_m2_font_asset_registration_rustfmt_metadata_passed_focused_test_timeout | `graphics/text/font/database.rs` 新增 `register_font_asset`,一次读取 source bytes 后按 `FontAsset.family_members` 生成 logical face descriptors,覆盖 family/face_index/weight/style/width_class/variations,并合并 `fallback_families` 到 database fallback chain;`graphics/text/font/asset_registration.rs` 承接 family-member 投影与 asset source key,其 key 包含 path/face/family/style/weight/stretch/variation,避免同一物理 face 的多逻辑声明互相压扁;native glyphon 与 SDF bake 的 UI manifest 路径现在带上完整 `FontAsset`,优先走资产注册入口,直接 `.ttf` 路径仍保留单文件注册;测试 TTC/weight fixture helper 拆到 `graphics/text/font/test_font_fixtures.rs`,避免把 fixture 构造继续堆进 database owner | scoped `rustfmt --check` 通过;scoped `git diff --check` 仅 CRLF 提示;`cargo metadata --locked --format-version 1 --no-default-features` 通过;截图证据仍在 `docs/tests/runtime/text/runtime_text_shared_metrics_preview_20260628.png`,确认 `target/runtime_text_shared_metrics_preview_20260628.png` 不存在;focused `text_font_database_registers_font_asset_family_members_and_fallbacks` logical-face lib-test 编译 244s 超时无 Rust diagnostics,本次独立 target-dir 验证进程已停止,不计 Cargo/test 通过 | family/fallback 首段已进入实际 native/SDF manifest 注册路径;后续仍需 WOFF2 decode、变量字体 fixture 绿跑、TTC tests 绿跑、SDF 非 0 face 真 raster、CompositeFont 资产声明和 plan 06 cmap/script/tofu 精筛 |

