---
related_code:
  - zircon_runtime/src/ui/text/mod.rs
  - zircon_runtime/src/ui/text/shaper.rs
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/edit_state.rs
  - zircon_runtime/src/ui/text/grapheme.rs
  - zircon_runtime/src/ui/text/hit_test.rs
  - zircon_runtime/src/ui/text/rich_text.rs
  - zircon_runtime/src/ui/surface/input/editable_text/ime_context.rs
  - zircon_runtime/src/ui/surface/input/editable_text/mutation.rs
  - zircon_runtime/src/ui/surface/input/text_keyboard/edit_actions.rs
  - zircon_runtime/src/ui/surface/input/text_keyboard/clipboard.rs
  - zircon_runtime_interface/src/ui/surface/render/mod.rs
  - zircon_runtime_interface/src/ui/dispatch/input/effect.rs
  - zircon_runtime/src/asset/assets/font.rs
  - zircon_runtime/src/asset/importer/ingest/import_font_asset.rs
plan_sources:
  - .codex/plans/ZirconEngine 宿主编辑器 UI 基础能力计划.md
  - .codex/plans/UI SDF 字体真实 Bake 收束计划.md
  - .codex/plans/M1 主链收口与文本底座计划.md
  - .codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md
status: planned
---

# 03 文本与字体栈定稿

## 1. 目标

定稿一条文本主链：**字体资产 → shaping/排版 → 字形栅格（SDF/位图）→ 渲染提取 → 命中/编辑/IME**，消除当前依赖声明与实际路径不一致的模糊地带，使 Label、Field、Console 日志、树/表行文本都走同一条链；中文（CJK）输入与显示是一等公民。

## 2. 现状（按代码核实修正）

### 2.1 已存在的设施

| 能力 | 落点 | 证据 |
|------|------|------|
| 文本渲染 DTO | `zircon_runtime_interface/src/ui/surface/render/` | `UiShapedGlyph`、`UiShapedText`、`UiShapedTextCluster`、`UiShapedTextLine`、`UiTextPaint`、`UiTextPaintRun`、`UiTextPaintDecoration(Kind)`、`UiTextRunPaintStyle`（mod.rs:52–53 re-export） |
| 自有 shaping/排版入口 | `zircon_runtime/src/ui/text/shaper.rs` | `layout_text`（:196）；**glyphon 后端未接入**（:117 注释「glyphon native text backend is not connected to layout yet」） |
| 编辑链骨架 | `zircon_runtime/src/ui/text/edit_state.rs` + `ui/surface/input/` | `apply_text_edit_action`（edit_state.rs:8）；`editable_text/{ime_context, mutation, state_transition}.rs`；`text_keyboard/{edit_actions, clipboard, payload}.rs`；grapheme.rs 光标导航 |
| IME 出站契约 | `zircon_runtime_interface/src/ui/dispatch/input/effect.rs` | `UiInputMethodRequest(Kind)`（:190/:205）、`UiInputMethodSurroundingText`（:213）；入站 `UiImeInputEvent`（event.rs:116，含 cursor range，01 已核实） |
| 字体资产 | `zircon_runtime/src/asset/` | `assets/font.rs`（FontAsset）、`importer/ingest/import_font_asset.rs`、`artifact/cache_payload.rs` |
| 依赖现状 | `zircon_runtime/Cargo.toml` | `glyphon = "0.11.0"`（:77，声明未用于布局）、`fontsdf = "0.5.3"`（:78，SDF bake）；**无 cosmic-text/fontdue/swash 直接依赖** |

### 2.2 真实缺口

1. **shaping 权威未定**：现行 `layout_text` 是自有简化路径；glyphon 挂名未接；CJK fallback 链、混排 baseline、BiDi 无系统化能力与测试。
2. **栅格策略未书面化**：fontsdf SDF 与小字号位图 atlas 的选用边界、位图栅格器选型（swash vs fontdue）未定。
3. **字体注册表缺失**：FontAsset 已可导入，但无 family/weight/style 注册与 fallback 链配置；editor 无默认字体包（含 CJK）。
4. **IME 组合链不闭环**：ime_context/出站请求类型齐备，但 preedit 临时 span 注入布局、anchor rect 随光标更新、实机中文输入验证未完成。
5. **测量重复**：文本测量与布局 measure 回调（02）之间无缓存契约，存在同帧重复 shaping 风险。

## 3. 设计

### 3.1 库选型定稿

- **shaping/排版/fallback 权威：cosmic-text**（直接依赖，含 rustybuzz shaping、系统字体发现、fallback、BiDi）。不经 glyphon——Zircon 已有自己的 GPU 文本提交路径（`UiShapedText`/`UiTextPaint` DTO + GPU command stream），从 cosmic-text 布局结果生成自有 `UiResolvedTextLayout`。**glyphon 从 Cargo.toml 移除**（M1，:117 证实未接入布局，删除风险低）。
- **栅格化**：保留 fontsdf SDF 路径用于可缩放/大字号；小字号 UI 文本走位图 atlas，栅格器在 swash（cosmic-text 同族）与 fontdue 间由基准测试定（M1.S1），淘汰者不进依赖。

### 3.2 文本管线

- 字体注册表归资产管线：FontAsset 加载 → family/weight/style 注册进 `UiFontRegistry`（包装 cosmic-text `FontSystem`）→ fallback 链配置（默认链含 CJK 字体）；editor 默认字体包进 `zircon_editor/assets/fonts/`。
- measure 与 layout 解耦：Taffy measure 回调（02 M1 接口）→ `UiTextMeasureCache`（key：内容 hash + 宽度约束桶 + style key）→ shaping 结果复用到 arrange 与 render extract，保证一帧内同一文本只 shape 一次（帧报告记录 shape 计数）。
- 富文本：rich_text 模型对齐 span（颜色/字重/下划线/链接），Console 日志高亮与 Inspector 字段标签共用。

### 3.3 编辑与 IME

- edit_state 定稿：grapheme 光标移动、词跳、选区、双击选词、三击选行、剪贴板 host request（text_keyboard/clipboard.rs 既有路径）。
- IME 全链时序：focus 进入文本节点 → reply 发 `UiDispatchEffect`（InputMethod enable + anchor rect）→ winit Ime 事件经 01 翻译为 `UiImeInputEvent` → preedit 以临时 span 注入 `UiResolvedTextLayout`（不进文档状态，ime_context.rs 持组合态）→ commit 走 mutation.rs 正常插入 → anchor rect 随光标布局更新再上报。
- 命中：hit_test 基于 `UiResolvedTextLayout` 提供 byte-offset ↔ 坐标双向查询，供鼠标定位光标与选区拖拽。

### 3.4 验收用例（编辑器真实场景）

搜索框/重命名框中文输入（含候选窗定位）、Console 万行日志虚拟滚动文本、Inspector 数值字段编辑、树行内重命名。

## 4. 接口与数据结构草案

```rust
// 新增 zircon_runtime/src/ui/text/font_registry.rs
pub struct UiFontRegistry {
    font_system: cosmic_text::FontSystem,       // shaping/fallback 权威
    families: Vec<UiFontFamilyRecord>,
    fallback_chain: Vec<String>,                // 默认链含 CJK family
}
pub struct UiFontFamilyRecord {
    pub family: String,
    pub weight: u16,
    pub style: UiFontStyle,                     // 新增枚举 Normal | Italic
    pub source: UiFontSource,                   // Asset(资产句柄) | System
}
impl UiFontRegistry {
    pub fn register_font_asset(&mut self, asset: &FontAsset) -> Result<UiFontId, UiFontRegistryError>;
    pub fn set_fallback_chain(&mut self, chain: Vec<String>);
}

// 新增 zircon_runtime/src/ui/text/resolved_layout.rs
pub struct UiResolvedTextLayout {
    pub lines: Vec<UiShapedTextLine>,           // 现有 DTO（interface render）
    pub size: UiSize2,
    pub first_baseline: f32,
    pub source_hash: u64,                       // 与 measure cache key 对应
}
pub struct UiTextLayoutRequest {
    pub spans: Vec<UiRichTextSpan>,             // rich_text 现有模型对齐后类型
    pub max_width: Option<f32>,
    pub wrap: UiTextWrap,                       // None | Word | Glyph
    pub style: UiTextStyleKey,                  // 字号/family/行高/字重 的紧凑 key
    pub preedit: Option<UiPreeditSpan>,         // IME 临时 span（不属于文档）
}
pub fn resolve_text_layout(
    registry: &mut UiFontRegistry,
    request: &UiTextLayoutRequest,
) -> UiResolvedTextLayout;

// 新增 zircon_runtime/src/ui/text/measure_cache.rs
pub struct UiTextMeasureCache { /* HashMap<UiTextMeasureKey, UiResolvedTextLayout> + 帧统计 */ }
pub struct UiTextMeasureKey {
    pub content_hash: u64,
    pub width_bucket: UiWidthBucket,            // 以换行等价类划分的宽度桶
    pub style: UiTextStyleKey,
}
impl UiTextMeasureCache {
    pub fn resolve_or_shape(&mut self, registry: &mut UiFontRegistry, request: &UiTextLayoutRequest) -> &UiResolvedTextLayout;
    pub fn frame_shape_count(&self) -> u64;      // 帧报告：零重复 shaping 断言数据源
}

// 新增 zircon_runtime/src/ui/text/raster/{mod,sdf,bitmap}.rs
pub enum UiGlyphRasterPath { Sdf, Bitmap }       // 字号阈值 + 缩放场景决定
pub fn raster_path_for(size_px: f32, scalable: bool) -> UiGlyphRasterPath;
```

## 5. 模块与文件落点

**新增**：`zircon_runtime/src/ui/text/{font_registry.rs, resolved_layout.rs, measure_cache.rs}`、`zircon_runtime/src/ui/text/raster/{mod.rs, sdf.rs, bitmap.rs}`、`zircon_editor/assets/fonts/`（默认字体包，含 CJK）、基准测试 harness（text 模块内 `#[cfg(test)]` + 截图样张）

**修改**：

| 路径 | 改什么 |
|------|--------|
| `zircon_runtime/Cargo.toml` | + cosmic-text；− glyphon（M1.S2）；位图栅格器按基准结论增删 |
| `zircon_runtime/src/ui/text/shaper.rs` | `layout_text` 改走 `resolve_text_layout`；自有简化 shaping 删除（M2） |
| `zircon_runtime/src/ui/text/{rich_text, hit_test, edit_state}.rs` | span 模型对齐、命中改基于 UiResolvedTextLayout、动作矩阵定稿 |
| `zircon_runtime/src/ui/surface/input/editable_text/ime_context.rs` | preedit span 注入与 anchor rect 闭环 |
| `zircon_runtime/src/ui/layout/pass/measure.rs` | 接 UiTextMeasureCache（02 M1 接口） |

**删除（硬切换义务）**：glyphon 依赖与残余引用（M1.S2）；shaper.rs 旧自有 shaping 路径（M2.S3 切换同变更删）；基准淘汰的栅格器不引入。

## 6. 管线时序

```
布局阶段：Taffy measure 回调 → UiTextMeasureCache.resolve_or_shape
  → cosmic-text shape + line break → UiResolvedTextLayout（缓存）
arrange：复用缓存结果定位文本节点
render extract：UiResolvedTextLayout → UiTextPaint runs（现有 DTO）
GPU command stream：glyph atlas（SDF / 位图按 raster_path_for）→ 提交
编辑路径：UiImeInputEvent / 键盘 → edit_state / ime_context → 文档或 preedit 变更
  → 标记文本节点 dirty → 下帧重新 resolve（preedit 作为请求字段进 key）
```

## 7. 里程碑切片化

| # | 切片 | 涉及文件 | 验证命令 | 硬切换 |
|---|------|---------|---------|--------|
| M1.S1 | 栅格基准：样张（拉丁/CJK 混排/数字）× 字号档 11/12/14/16/24/32 × {swash, fontdue}，维度=质量截图对拍/栅格耗时/atlas 占用；结论写 `docs/zircon_runtime/ui/text.md` | 基准 harness | `cargo test -p zircon_runtime --lib text_raster_bench --locked -- --nocapture` | 无删除 |
| M1.S2 | 依赖收口：+cosmic-text、−glyphon、+基准胜者；全工作区 check | Cargo.toml | `cargo check --workspace --locked` | 删 glyphon 及引用 |
| M2.S1 | UiFontRegistry + FontAsset 注册（复用 assets/font.rs 加载链） | font_registry.rs | `cargo test -p zircon_runtime --lib font_registry --locked` | 无删除 |
| M2.S2 | 默认字体包（含 CJK）+ fallback 链配置；editor 启动注册 | zircon_editor/assets/fonts/ | `cargo test -p zircon_editor --lib --locked` | 无删除 |
| M2.S3 | resolve_text_layout 落地，shaper.layout_text 切换；CJK/混排 shaping 测试 | resolved_layout.rs、shaper.rs | `cargo test -p zircon_runtime --lib text --locked` | 删 shaper 旧路径 |
| M3.S1 | UiTextMeasureCache + key 定稿（宽度桶=换行等价类） | measure_cache.rs | `cargo test -p zircon_runtime --lib measure_cache --locked` | 无删除 |
| M3.S2 | pass/measure.rs 接缓存；帧报告记录 shape 计数 | pass/measure.rs | `cargo test -p zircon_runtime --lib measure --locked` | 无删除 |
| M3.S3 | 同帧零重复 shaping 断言（典型 workbench 模板帧） | 测试 | 同上 | 无删除 |
| M4.S1 | edit_state 动作矩阵定稿：grapheme 光标/词跳/选区/双击选词/三击选行（基于 mutation.rs、edit_actions.rs 现有） | edit_state.rs、editable_text/ | `cargo test -p zircon_runtime --lib edit_state --locked` | 无删除 |
| M4.S2 | IME 闭环：preedit span 注入 + anchor rect 随光标上报（依赖 01 M1 事件、01 M3 reply） | ime_context.rs、resolved_layout.rs | `cargo test -p zircon_runtime --lib ime --locked` | 无删除 |
| M4.S3 | 实机中文输入验收：搜索框、重命名框（候选窗定位正确） | 实机 | `cargo run -p zircon_app --no-default-features --features target-editor-host --bin zircon_editor` | 无删除 |
| M5.S1 | rich_text span 模型对齐（颜色/字重/下划线/链接）+ 提取测试 | rich_text.rs | `cargo test -p zircon_runtime --lib rich_text --locked` | 无删除 |
| M5.S2 | Console 日志高亮 + Inspector 字段标签接入（与 09 批次 1 协同） | editor 模块侧 | `cargo test -p zircon_editor --lib --locked` + 实机 | 无删除 |

## 8. 测试矩阵（代表性用例）

- **M1**：`raster_bench_small_size_quality_comparison`（带 --nocapture 输出结论）
- **M2**：`cjk_mixed_run_shapes_with_fallback_chain`、`latin_cjk_baseline_alignment_stable`、`missing_glyph_falls_back_not_tofu`
- **M3**：`measure_cache_hits_same_content_same_width_bucket`、`width_bucket_boundary_reshapes_on_wrap_change`、`frame_report_zero_duplicate_shaping_for_workbench_template`
- **M4**：`grapheme_cursor_moves_over_emoji_cluster`、`double_click_selects_word_triple_click_selects_line`、`preedit_span_injected_without_document_mutation`、`ime_anchor_rect_follows_cursor_line_wrap`、`commit_replaces_preedit_atomically`
- **M5**：`rich_text_decoration_runs_extracted`、`console_log_highlight_spans_render`

落点：`zircon_runtime/src/ui/text/` 模块内 `#[cfg(test)]`（layout_engine/tests.rs 已示范该惯例）。

## 9. 风险与对策

| 风险 | 对策 / 探测信号 |
|------|----------------|
| cosmic-text 切换引发度量回归（行高/基线变化波及全部模板） | M2.S3 切换前对现行 layout_text 输出做数值快照，切换后逐差异审查而非盲改基线 |
| CJK 默认字体包体积大 | 评估 subset/按需加载；staged build 产物体积进验收记录 |
| 宽度桶设计不当（缓存命中低或换行错误） | 桶按换行等价类划分；M3 边界测试覆盖「桶内不换行变化、跨桶必 reshape」 |
| IME 行为平台差异 | 以 Windows 实机为验收基准；其他平台差异显式记录为后续项 |
| glyphon 移除波及未知引用 | :117 已证实未接布局；M1.S2 以 `cargo check --workspace` 全量验证 |

## 10. 里程碑级依赖表

| 里程碑 | 前置 | 被依赖 |
|--------|------|--------|
| M1 | 无 | 03 M2/M3 |
| M2 | 03 M1；05 M1（字体资产类型收口，弱依赖——FontAsset 已可用） | 03 M3/M4/M5、06 M1（TextField/Label） |
| M3 | 03 M1、02 M1（measure 回调接口） | 06 M2（虚拟化行文本） |
| M4 | 03 M2、01 M1（IME 事件）、01 M3（reply/host request） | 06 M1（TextField DoD）、09 M1（重命名/搜索） |
| M5 | 03 M2 | 09 M1（Console/Inspector） |

## 11. 完成定义

- 依赖清单只剩 cosmic-text + fontsdf + 基准胜者；shaping 单实现。
- 实机：搜索框中文输入（候选窗跟随光标）、树行内重命名、Inspector 数值编辑、万行 Console 滚动全部正常。
- 帧报告同帧零重复 shaping；CJK/混排测试全绿。
- 验收命令组：`cargo test -p zircon_runtime --lib --locked`（text/measure/ime 过滤）、`cargo test -p zircon_editor --lib --locked`、实机启动验证。

## 12. 边界约束

- 文本布局引擎留在 `zircon_runtime::ui::text`；接口层只过 `UiShapedText`/`UiTextPaint` 等现有 DTO，`UiResolvedTextLayout` 不出 runtime。
- RTL/BiDi 本期只保证 cosmic-text 给出的正确视觉序与不崩溃，不做镜像布局（记录为后续项）。
- 字形 atlas 归 GPU command stream 资源面，本计划只定生产侧格式（SDF/位图选径函数）。
- preedit 永不进文档状态；组合期间文档 mutation 被拒绝并记录诊断。
