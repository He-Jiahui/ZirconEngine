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
  - zircon_runtime/src/ui/component/state_reducer/text_input.rs
  - zircon_runtime/src/ui/component/catalog/material_foundation/shared.rs
  - zircon_runtime/src/ui/tests/component_catalog/component_state/text_input_validation.rs
  - zircon_runtime_interface/src/ui/surface/render/mod.rs
  - zircon_runtime_interface/src/ui/dispatch/input/effect.rs
  - zircon_runtime/src/asset/assets/font.rs
  - zircon_runtime/src/asset/importer/ingest/import_font_asset/mod.rs
plan_sources:
  - .codex/plans/ZirconEngine 宿主编辑器 UI 基础能力计划.md
  - .codex/plans/UI SDF 字体真实 Bake 收束计划.md
  - .codex/plans/M1 主链收口与文本底座计划.md
  - .codex/plans/ZirconEngine UITextInputA11y 缺口收束计划.md
  - docs/plans/zircon_runtime/text/
status: planned
---

# 03 文本与字体栈定稿

## 1. 目标

定稿一条文本主链：**字体资产 → shaping/排版 → 字形栅格（SDF/位图）→ 渲染提取 → 命中/编辑/IME**，消除当前依赖声明与实际路径不一致的模糊地带，使 Label、Field、Console 日志、树/表行文本都走同一条链；中文（CJK）输入与显示是一等公民。

## 与 zircon_runtime/text 的分工（2026-07-02 评审收口）

文本服务实现权威 = `docs/plans/zircon_runtime/text/`（9 子计划，2026-06-27 建立）。本计划的 M1（栅格基准）、M2（字体注册表）、M3（measure cache 实现）主体让渡给 text/01–04/09；03 保留**编辑器侧接入切片与验收**：搜索框中文输入、树行内重命名、Console 万行滚动、preedit 注入。以下正文中与 text/ 子计划冲突的实现指令，一律以 text/ 定稿为准，03 只承接消费与验收。

## 2. 现状（按代码核实修正）

### 2.1 已存在的设施

| 能力 | 落点 | 证据 |
|------|------|------|
| 文本渲染 DTO | `zircon_runtime_interface/src/ui/surface/render/` | `UiShapedGlyph`、`UiShapedText`、`UiShapedTextCluster`、`UiShapedTextLine`、`UiTextPaint`、`UiTextPaintRun`、`UiTextPaintDecoration(Kind)`、`UiTextRunPaintStyle`（mod.rs:52–53 re-export） |
| 自有 shaping/排版入口 | `zircon_runtime/src/ui/text/shaper.rs` | `layout_text`（:196）；**glyphon 后端未接入**（:117 注释「glyphon native text backend is not connected to layout yet」）（2026-07-02 勘误：行号已漂移，`layout_text` 现位于 :176；shaping 已经 `UiTextShaperStack`→`SharedTextShapingService`→`graphics/text/shaping/cosmic.rs` 接入真实整形，「未接入」表述已过时） |
| 编辑链骨架 | `zircon_runtime/src/ui/text/edit_state.rs` + `ui/surface/input/` | `apply_text_edit_action`（edit_state.rs:8）；`editable_text/{ime_context, mutation, state_transition}.rs`；`text_keyboard/{edit_actions, clipboard, payload}.rs`；grapheme.rs 光标导航 |
| IME 出站契约 | `zircon_runtime_interface/src/ui/dispatch/input/effect.rs` | `UiInputMethodRequest(Kind)`（:190/:205）、`UiInputMethodSurroundingText`（:213）；入站 `UiImeInputEvent`（event.rs:116，含 cursor range，01 已核实） |
| 字体资产 | `zircon_runtime/src/asset/` | `assets/font.rs`（FontAsset）、`importer/ingest/import_font_asset.rs`、`artifact/cache_payload.rs` |
| 依赖现状 | `zircon_runtime/Cargo.toml` | `glyphon = "0.11.0"`（:77，声明未用于布局）、`fontsdf = "0.5.3"`（:78，SDF bake）；**无 cosmic-text/fontdue/swash 直接依赖**（2026-07-02 勘误：行号已漂移至 :90/:91；「无 cosmic-text/fontdue/swash 直接依赖」对 editor 已失效——editor 直依 fontdb/fontdue/swash，属 text/index 隔离规则违例，收束归 retained-host 迁移） |

（2026-07-02 勘误：本表撰写后代码已推进——`font_registry.rs`/`resolved_layout.rs`/`measure_cache.rs`/`raster/` 已全部存在于 `zircon_runtime/src/ui/text/`；`graphics/text/{font, shaping, layout, atlas}` 服务层已建。§4/§5 中标注「新增」的落点应按此读作「已存在，待接真实度量 / 待切服务适配器」。）

### 2.2 真实缺口

1. **shaping 权威未定**：现行 `layout_text` 是自有简化路径；glyphon 挂名未接；CJK fallback 链、混排 baseline、BiDi 无系统化能力与测试。
2. **栅格策略未书面化**：fontsdf SDF 与小字号位图 atlas 的选用边界、位图栅格器选型（swash vs fontdue）未定。
3. **字体注册表缺失**：FontAsset 已可导入，但无 family/weight/style 注册与 fallback 链配置；editor 无默认字体包（含 CJK）。
4. **IME 组合链不闭环**：ime_context/出站请求类型齐备；retained-state preedit/commit/cancel、ABI disabled -> Cancel、`wrap`/`multiline` opt-in 的 soft-wrap cursor/composition rect 基线、以及 TextField render extract 的 `UiResolvedTextLayout` preedit span 注入已落，但平台候选窗实机定位与中文输入验收仍未完成。
5. **测量重复**：文本测量与布局 measure 回调（02）之间无缓存契约，存在同帧重复 shaping 风险。

## 3. 设计

### 3.1 库选型定稿

- **shaping/排版/fallback 权威：cosmic-text**（含 rustybuzz shaping、系统字体发现、fallback、BiDi）。Zircon 已有自己的 GPU 文本提交路径（`UiShapedText`/`UiTextPaint` DTO + GPU command stream），从 cosmic-text 布局结果生成自有 `UiResolvedTextLayout`。（2026-07-02 评审收口）**glyphon 保留为 bitmap atlas 绘制后端**（text/index §7 定稿），不再移除；布局权威 = cosmic-text（经 glyphon re-export）。原「glyphon 从 Cargo.toml 移除」义务作废，glyphon/cosmic-text 的定位以 text/02（shaping）与 text/04（atlas）为准。
- **栅格化**：保留 fontsdf SDF 路径用于可缩放/大字号；小字号 UI 文本走位图 atlas。（2026-07-02 评审收口）字形栅格定稿 **swash**（text/index §5），fontdue 仅备选；原「swash vs fontdue 由基准测试定」的选型切片作废，降级为验证性对拍（见 M1.S1）。

### 3.2 文本管线

- 字体注册表归资产管线：FontAsset 加载 → family/weight/style 注册进 text/01 `FontDatabase`（`FontFaceId`/`InstancedFaceId` 契约；2026-07-02 评审收口，原「UiFontRegistry 包装 cosmic-text FontSystem」表述作废）→ fallback 链配置（默认链含 CJK 字体）；editor 默认字体包进 `zircon_editor/assets/fonts/`。
- measure 与 layout 解耦：Taffy measure 回调（02 M1 接口）→ `UiTextMeasureCache`（key：内容 hash + 宽度约束桶 + style key）→ shaping 结果复用到 arrange 与 render extract，保证一帧内同一文本只 shape 一次（帧报告记录 shape 计数）。
- 富文本：（2026-07-02 评审收口）span/装饰器 schema 权威 = text/07（BBCode+HTML 子集）；rich_text 模型对齐其契约，本计划 M5 收窄为 Console 日志高亮与 Inspector 字段标签的编辑器侧接入。

### 3.3 编辑与 IME

- edit_state 定稿：grapheme 光标移动、词跳、选区、双击选词、三击选行、剪贴板 host request（text_keyboard/clipboard.rs 既有路径）。
- IME 全链时序：focus 进入文本节点 → reply 发 `UiDispatchEffect`（InputMethod enable + anchor rect）→ winit Ime 事件经 01 翻译为 `UiImeInputEvent` → preedit 以临时 span 注入 `UiResolvedTextLayout`（不进文档状态，ime_context.rs 持组合态）→ commit 走 mutation.rs 正常插入 → anchor rect 随光标布局更新再上报。
- （2026-07-02 评审收口）IME 职责表正文在 text/08，此处仅引用：winit 基线入站翻译归 zircon_runtime `ui/platform_input`（01 拥有）；平台特化（TSF/IMM32/IBus/fcitx）与出站 host request 应用归 zircon_app 平台层（text/08 IM-M2）；iface dispatch DTO 变更由 01 与 text/08 协同一次合并。focus→IME enable/disable 生命周期次序：焦点进入可编辑节点→enable+anchor rect，离开→commit preedit→disable；popup 抢焦期间 Esc 先取消组合再关 popup。
- 命中：hit_test 基于 `UiResolvedTextLayout` 提供 byte-offset ↔ 坐标双向查询，供鼠标定位光标与选区拖拽。

### 3.4 验收用例（编辑器真实场景）

搜索框/重命名框中文输入（含候选窗定位）、Console 万行日志虚拟滚动文本、Inspector 数值字段编辑、树行内重命名。

## 4. 接口与数据结构草案

（2026-07-02 评审收口）原 `UiFontRegistry { cosmic_text::FontSystem }` 草案**作废**——直持 `cosmic_text::FontSystem` 违反第三方类型隔离硬规则（fontdb 等第三方类型隔离在 graphics/text 内）。字体注册表权威 = text/01 的 `FontDatabase` + `FontFaceId`/`InstancedFaceId` 契约。03 只保留消费切片：**编辑器默认字体包（含 CJK）注册进 `FontDatabase`**（见 M2.S2）。原草案留档如下仅作历史对照，不再实现：

```rust
// （作废草案，权威见 text/01 FontDatabase）
// pub struct UiFontRegistry { font_system: cosmic_text::FontSystem, ... }

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

（2026-07-02 评审收口）`UiTextMeasureKey`/`UiTextMeasureCache` 与 text/09 的两级缓存统一：**ShapedRunCache**（无 wrap，键=内容+style，存 run 级 shaping 结果）+ **LayoutCache**（含宽度约束，键=ShapedRun+宽度桶，存换行布局结果）。本计划的 measure cache 即 LayoutCache 的编辑器侧消费面，不另建第三级缓存。另按 editor_layout/13 §3.2b 要求，measure 回调需支持 **min-content / max-content / preferred 三值语义**：min-content=最长不可断片段宽度、max-content=不换行整段宽度、preferred=给定宽度约束下的换行布局结果；三值共享 ShapedRunCache 的 shaping 结果，仅 preferred 需键入 LayoutCache 宽度桶。

## 5. 模块与文件落点

**已存在，待接真实度量/待切服务适配器**（2026-07-02 勘误：原标注「新增」，现已全部在码）：`zircon_runtime/src/ui/text/{font_registry.rs, resolved_layout.rs, measure_cache.rs}`、`zircon_runtime/src/ui/text/raster/{mod.rs, sdf.rs, bitmap.rs}`。仍为新增：`zircon_editor/assets/fonts/`（默认字体包，含 CJK）、验证性对拍 harness（text 模块内 `#[cfg(test)]` + 截图样张）。

**修改**：

| 路径 | 改什么 |
|------|--------|
| `zircon_runtime/Cargo.toml` | （2026-07-02 评审收口）glyphon 保留（U2 定稿为 bitmap atlas 绘制后端）；依赖收口以 text/index §5/§7 定稿清单为准 |
| `zircon_runtime/src/ui/text/shaper.rs` | `layout_text` 改走 `resolve_text_layout`；自有简化 shaping 删除（M2；前置=text/02 shaping、text/03 布局里程碑交付） |
| `zircon_runtime/src/ui/text/{rich_text, hit_test, edit_state}.rs` | span 模型对齐（schema 权威=text/07）、命中改基于 UiResolvedTextLayout、动作矩阵定稿 |
| `zircon_runtime/src/ui/surface/input/editable_text/ime_context.rs` | preedit span 注入与 anchor rect 闭环 |
| `zircon_runtime/src/ui/layout/pass/measure.rs` | 接 UiTextMeasureCache（02 M1 接口；缓存实现前置=text/09 两级缓存里程碑） |

**删除（硬切换义务）**：（2026-07-02 评审收口）原「glyphon 依赖与残余引用（M1.S2）」删除义务**作废**（U2）；shaper.rs 旧自有 shaping 路径（M2.S3 切换同变更删）仍有效。M2/M3 的实现主体已让渡 text/01/03/09，本表相应行读作「待对应 text/ 里程碑交付后的编辑器侧接入」。

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
| M1.S1 | （2026-07-02 评审收口：选型切片作废，降级为**验证性对拍**——栅格定稿 swash，见 U3/text/index §5）样张（拉丁/CJK 混排/数字）× 字号档 11/12/14/16/24/32，对拍 swash 输出质量/栅格耗时/atlas 占用；结论写 `docs/zircon_runtime/ui/text.md` | 对拍 harness | `cargo test -p zircon_runtime --lib text_raster_bench --locked -- --nocapture` | 无删除 |
| M1.S2 | （2026-07-02 评审收口：「−glyphon」义务**作废**，glyphon 保留为 bitmap atlas 绘制后端）依赖收口按 text/02（shaping/cosmic-text 定位）与 text/04（atlas/glyphon 定位）执行；workspace 全量 check 义务已随 U2 裁决一并作废，包级 focused check 留给对应 text/ 里程碑的最小批次（policy §4 波次收口执行全量） | Cargo.toml | `cargo check -p zircon_runtime --lib --locked` | 无删除 |
| M2.S1 | （实现主体让渡 text/01 `FontDatabase`）编辑器侧接入 FontAsset→FontDatabase 注册链（复用 assets/font.rs 加载链；前置=text/01 里程碑） | font_registry.rs | `cargo test -p zircon_runtime --lib font_registry --locked` | 无删除 |
| M2.S2 | 默认字体包（含 CJK）注册进 FontDatabase + fallback 链配置；editor 启动注册 | zircon_editor/assets/fonts/ | `cargo test -p zircon_editor --lib --locked` | 无删除 |
| M2.S3 | resolve_text_layout 落地，shaper.layout_text 切换；CJK/混排 shaping 测试（前置=text/02/03 里程碑） | resolved_layout.rs、shaper.rs | `cargo test -p zircon_runtime --lib text --locked` | 删 shaper 旧路径 |
| M3.S1 | （实现主体让渡 text/09 两级缓存）UiTextMeasureCache + key 定稿（宽度桶=换行等价类；前置=text/09 里程碑） | measure_cache.rs | `cargo test -p zircon_runtime --lib measure_cache --locked` | 无删除 |
| M3.S2 | pass/measure.rs 接缓存；帧报告记录 shape 计数 | pass/measure.rs | `cargo test -p zircon_runtime --lib measure --locked` | 无删除 |
| M3.S3 | 同帧零重复 shaping 断言（典型 workbench 模板帧） | 测试 | 同上 | 无删除 |
| M4.S1 | edit_state 动作矩阵定稿：grapheme 光标/词跳/选区/双击选词/三击选行（基于 mutation.rs、edit_actions.rs 现有） | edit_state.rs、editable_text/ | `cargo test -p zircon_runtime --lib edit_state --locked` | 无删除 |
| M4.S2 | IME 闭环：preedit span 注入 + anchor rect 随光标上报（依赖 01 M1 事件、01 M3 reply） | ime_context.rs、resolved_layout.rs | `cargo test -p zircon_runtime --lib ime --locked` | 无删除 |
| M4.S3 | 实机中文输入验收：搜索框、重命名框（候选窗定位正确）。（2026-07-02 评审收口）验收项与 text/08 IM-M3 checklist 合并，直接引用其 checklist，不在此重复定义 | 实机 | `cargo run -p zircon_app --no-default-features --features target-editor-host --bin zircon_editor` | 无删除 |
| M5.S1 | （2026-07-02 评审收口：span/装饰器 schema 权威=text/07（BBCode+HTML 子集），本切片收窄为消费对齐）rich_text 模型对齐 text/07 span 契约 + 提取测试 | rich_text.rs | `cargo test -p zircon_runtime --lib rich_text --locked` | 无删除 |
| M5.S2 | Console 日志高亮 + Inspector 字段标签接入（与 09 批次 1 协同；M5 收窄为 Console/Inspector 接入，见 U9） | editor 模块侧 | `cargo test -p zircon_editor --lib --locked` + 实机 | 无删除 |

### 7.1 当前执行状态（2026-06-13）

- M4.S1 的宿主物理键覆盖已向 `zircon_runtime/src/ui/tests/widget_text_input_mui.rs` 扩展：MUI `TextField` 覆盖 Ctrl+Backspace 词删除、Ctrl+A 全选后替换、Shift/Ctrl+Shift 选区扩展与 Delete 选区折叠；`TextareaAutosize` 覆盖多行 ArrowUp/ArrowDown/Home/End/Enter retained edit-state 写回。指针路径已在 `zircon_runtime/src/ui/surface/input/text_pointer.rs` 明确区分双击选词与三击选行，三击使用共享 `line_start_boundary`/`line_end_boundary`，不再落入双击词选择分支。
- 这些用例复用既有 `text_keyboard/edit_actions.rs`、`ui/text/grapheme.rs`、`editable_text/state_transition.rs` 路径，没有引入新的兼容分支或替代编辑器路径。
- M4.S2 的输入法取消链路已补到底层窗口适配器：`zircon_runtime_interface/src/ui/window/runtime_event_adapter.rs` 将 ABI `ime_disabled` 映射为 `UiImeInputEventKind::Cancel`，让 surface 的 cancel 分支能从窗口输入泵触发；MUI `TextField` 新增 `mui_text_field_ime_preedit_commit_and_cancel_use_retained_composition_state`，覆盖 preedit 选区替换、commit 原子提交与 cancel 恢复组合前文本并清理 `input_method_owner`。
- M4.S2 的候选窗锚点基线已补到 `zircon_runtime/src/ui/surface/input/editable_text/ime_context.rs`：光标 rect 与 composition rect 现在共享可见行列计算，保留硬换行拆分，并在节点声明 `wrap != "none"` 或 `multiline = true` 时按当前 text frame 宽度与简化字体度量拆 soft-wrap 行。`widget_text_input_ime_context.rs` 新增 `text_input_ime_preedit_rects_follow_soft_wrapped_composition_range`，覆盖 preedit 组合范围跨 soft-wrap 行时返回多段 `composition_rects`，并让 `cursor_rect` 落到换行后的可见位置。
- M4.S2 的 resolved-layout preedit 注入基线已补到 render extract：`zircon_runtime/src/ui/surface/render/extract.rs` 和 `text_fields.rs` 改走 `UiTextLayoutRequest` / `resolve_text_layout`，TextField composition metadata 会转换成 `UiPreeditSpan` 后只影响临时布局文本；`text_layout.rs` 新增 `render_extract_injects_preedit_span_without_document_value_mutation`，断言 render command 的 retained value 仍是原文，而 `UiResolvedTextLayout` 行文本显示 preedit 替换结果。
- M4.S1 的 TextInput 验证时机基线已补到组件 reducer：`zircon_runtime/src/ui/component/state_reducer/text_input.rs` 从主键盘 reducer 拆出 TextInput 写入与验证逻辑，并按 `validation_timing = change|commit|blur` 管理 required/min/max-length 校验、dirty/touched 标记、`validation_level` 与 `validation_message` 写回。`shared.rs` 让 TextField/Input/Textarea/SearchField/FieldEditor/SourceEditor 共享验证 props，query-backed SearchField 也走同一套时机规则。
- 当前验证：`rustfmt --edition 2021 --check zircon_runtime\src\ui\tests\widget_text_input_mui.rs` 通过；`cargo test -p zircon_runtime --lib --no-run --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-text-edit-actions-0613-coremin --message-format short --color never mui_text_field_keyboard_word_delete_and_select_all_replace_use_retained_state` 在 1204s 后仍停于 Windows lib-test 编译/链接且无 Rust 诊断，已停止该 target-dir 残留进程，目标目录未产出 `zircon_runtime-*.exe` 测试二进制。
- 当前验证：`rustfmt --edition 2021 --check zircon_runtime\src\ui\surface\input\text_pointer.rs zircon_runtime\src\ui\tests\widget_text_input_pointer.rs` 通过；`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-typeahead-host-0613-coremin --message-format short --color never` 通过，仅有既有 warning。
- 当前验证：`rustfmt --edition 2021 --check zircon_runtime_interface\src\ui\window\runtime_event_adapter.rs zircon_runtime_interface\src\tests\window_runtime_event_adapter_contracts.rs zircon_runtime\src\ui\tests\widget_text_input_mui.rs` 通过；`cargo check -p zircon_runtime_interface --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-interface-ime-cancel-0613 --message-format short --color never` 通过；`cargo test -p zircon_runtime_interface --lib runtime_event_adapter_maps_keyboard_ime_drag_gamepad_and_accessibility_inputs --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-interface-ime-cancel-0613 --message-format short --color never` 通过 1/1。MUI runtime 新用例尚未执行，仍受 Windows `zircon_runtime` lib-test 编译/链接耗时阻塞。
- 当前验证：`rustfmt --edition 2021 --check zircon_runtime\src\ui\surface\input\editable_text\ime_context.rs zircon_runtime\src\ui\tests\widget_text_input_ime_context.rs` 通过；`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-ime-anchor-0613-coremin --message-format short --color never` 通过，仅有既有 warning；`cargo test -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-ime-anchor-0613-coremin text_input_ime_preedit_rects_follow_soft_wrapped_composition_range --message-format short --color never -- --exact --nocapture` 在 1204s 后仍停于 Windows lib-test 编译/链接且无 Rust 诊断，目标目录未产出 `zircon_runtime-*.exe` 测试二进制，残留 cargo/rustc 进程已停止。
- 当前验证：`rustfmt --edition 2021 --check zircon_runtime\src\ui\text\mod.rs zircon_runtime\src\ui\surface\render\extract.rs zircon_runtime\src\ui\surface\render\text_fields.rs zircon_runtime\src\ui\tests\text_layout.rs` 通过；`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-preedit-layout-0613-coremin --message-format short --color never` 通过，仅有既有 warning；`cargo test -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-preedit-layout-0613-coremin ui::tests::text_layout::render_extract_injects_preedit_span_without_document_value_mutation --message-format short --color never -- --exact --nocapture` 在 1204s 后仍停于 Windows lib-test 编译/链接且无 Rust 诊断，目标目录未产出 `zircon_runtime-*.exe` 测试二进制，残留 cargo/rustc 进程已停止。
- 当前验证：`rustfmt --edition 2021 --check zircon_runtime\src\ui\component\state_reducer.rs zircon_runtime\src\ui\component\state_reducer\keyboard.rs zircon_runtime\src\ui\component\state_reducer\text_input.rs zircon_runtime\src\ui\component\catalog\material_foundation\shared.rs zircon_runtime\src\ui\tests\component_catalog\component_state.rs zircon_runtime\src\ui\tests\component_catalog\component_state\text_input_validation.rs zircon_runtime\src\ui\tests\component_catalog\material_foundation\mod.rs` 通过；`cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-text-validation-0614-coremin --message-format short --color never` 通过，仅有既有 warning；`cargo test -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-text-validation-0614-coremin text_input_ --message-format short --color never -- --nocapture` 在 1204s 后仍停于 Windows lib-test 编译/链接且无 Rust 诊断，目标目录未产出 `zircon_runtime-*.exe` 测试二进制，残留 cargo/rustc 进程已停止。
- M4.S1 行为代码已经覆盖键盘 grapheme/word/selection、多行 caret、pointer hit-test、拖拽选区、双击选词与三击选行，以及 TextInput change/commit/blur 验证时机；M4.S2 已覆盖 ABI disabled -> Cancel 输入链路、MUI preedit/commit/cancel retained composition 用例、retained IME context 的 hard-line/soft-wrap cursor 与 composition rect 基线，以及 TextField render extract 的 resolved-layout preedit 注入基线。剩余风险是新的 runtime lib-test 执行仍被 Windows 测试二进制编译/链接耗时阻塞，完整通过信号需要在里程碑测试阶段复跑；M4.S3 仍需真实中文输入验收和平台候选窗位置验证。

## 8. 测试矩阵（代表性用例）

- **M1**：`raster_bench_small_size_quality_comparison`（带 --nocapture 输出结论）
- **M2**：`cjk_mixed_run_shapes_with_fallback_chain`、`latin_cjk_baseline_alignment_stable`、`missing_glyph_falls_back_not_tofu`
- **M3**：`measure_cache_hits_same_content_same_width_bucket`、`width_bucket_boundary_reshapes_on_wrap_change`、`frame_report_zero_duplicate_shaping_for_workbench_template`
- **M4**：`mui_text_field_keyboard_word_delete_and_select_all_replace_use_retained_state`、`mui_text_field_keyboard_shift_extends_selection_and_delete_collapses_it`、`mui_textarea_keyboard_multiline_navigation_and_enter_update_retained_state`、`text_input_triple_click_selects_line_from_pointer_hit`、`runtime_event_adapter_maps_keyboard_ime_drag_gamepad_and_accessibility_inputs`、`mui_text_field_ime_preedit_commit_and_cancel_use_retained_composition_state`、`text_input_ime_preedit_rects_follow_soft_wrapped_composition_range`、`render_extract_injects_preedit_span_without_document_value_mutation`、`text_input_commit_timing_defers_validation_until_commit`、`text_input_blur_timing_validates_on_focus_loss`、`text_input_change_timing_validates_max_length_live`、`grapheme_cursor_moves_over_emoji_cluster`、`double_click_selects_word_triple_click_selects_line`、`ime_anchor_rect_follows_cursor_line_wrap`、`commit_replaces_preedit_atomically`
- **M5**：`rich_text_decoration_runs_extracted`、`console_log_highlight_spans_render`

落点：`zircon_runtime/src/ui/text/` 模块内 `#[cfg(test)]`（layout_engine/tests.rs 已示范该惯例）。

## 9. 风险与对策

| 风险 | 对策 / 探测信号 |
|------|----------------|
| cosmic-text 切换引发度量回归（行高/基线变化波及全部模板） | M2.S3 切换前对现行 layout_text 输出做数值快照，切换后逐差异审查而非盲改基线 |
| CJK 默认字体包体积大 | 评估 subset/按需加载；staged build 产物体积进验收记录 |
| 宽度桶设计不当（缓存命中低或换行错误） | 桶按换行等价类划分；M3 边界测试覆盖「桶内不换行变化、跨桶必 reshape」 |
| IME 行为平台差异 | 以 Windows 实机为验收基准；其他平台差异显式记录为后续项 |
| glyphon 移除波及未知引用 | （2026-07-02 评审收口：glyphon 保留，本风险随 U2 裁决作废）~~:117 已证实未接布局；M1.S2 workspace 全量 check 义务已随裁决作废，参 policy §4 波次收口~~ |

## 10. 里程碑级依赖表

| 里程碑 | 前置 | 被依赖 |
|--------|------|--------|
| M1 | 无 | 03 M2/M3 |
| M2 | 03 M1；05 M1（字体资产类型收口，弱依赖——FontAsset 已可用） | 03 M3/M4/M5、06 M1（TextField/Label） |
| M3 | 03 M1、02 M1（measure 回调接口） | 06 M2（虚拟化行文本） |
| M4 | 03 M2、01 M1（IME 事件）、01 M3（reply/host request） | 06 M1（TextField DoD）、09 M1（重命名/搜索） |
| M5 | 03 M2 | 09 M1（Console/Inspector） |

## 11. 完成定义

- 依赖清单以 text/index 定稿为准：cosmic-text（布局权威）+ swash（栅格）+ glyphon（bitmap atlas 绘制后端）+ fontsdf；shaping 单实现。（2026-07-02 评审收口，原「只剩 cosmic-text + fontsdf + 基准胜者」表述按 U2/U3 更新）
- 实机：搜索框中文输入（候选窗跟随光标）、树行内重命名、Inspector 数值编辑、万行 Console 滚动全部正常。
- 帧报告同帧零重复 shaping；CJK/混排测试全绿。
- 验收命令组：`cargo test -p zircon_runtime --lib --locked`（text/measure/ime 过滤）、`cargo test -p zircon_editor --lib --locked`、实机启动验证。

## 12. 边界约束

- （2026-07-02 评审收口，按 U5 改写）文本实现层归 `graphics/text` 共享服务；`ui::text` **硬切换为服务适配器**（render/14 TD-M1 + text/index §6 定稿），不再作为布局引擎权威落点。接口层只过 `UiShapedText`/`UiTextPaint` 等现有 DTO，`UiResolvedTextLayout` 不出 runtime。原「布局引擎留在 ui::text」表述作废。
- RTL/BiDi 本期只保证 cosmic-text 给出的正确视觉序与不崩溃，不做镜像布局（记录为后续项）。
- 字形 atlas 归 GPU command stream 资源面，本计划只定生产侧格式（SDF/位图选径函数）。
- preedit 永不进文档状态；组合期间文档 mutation 被拒绝并记录诊断。

## 13. 参考实现对照（dev/ 源码锚点）

实现切片前先读对应锚点，不确定的行为语义以参考实现为准（在 PR 说明中注明出处）；禁止凭印象实现、禁止引用未核实路径。

| 设计点 | 主参考 | 次参考 | 参考什么 |
|--------|--------|--------|---------|
| cosmic-text + swash 消费样板 | `dev/bevy/crates/bevy_text` | — | bevy_text 以 swash 0.2 做栅格（Cargo.toml:43）；其字形 atlas、measure 与 ECS 集成是消费级样板（恰与 M1 基准候选一致） |
| 换行/字簇/排版 | `dev/slint/internal/core/textlayout/{linebreaker.rs, linebreak_unicode.rs, glyphclusters.rs, fragments.rs}` | — | Unicode 换行、grapheme cluster 切分、行片段模型的 Rust 实现 |
| 字体缓存/atlas 架构 | `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Fonts` | `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Textures` | FontCache 的 family/fallback/atlas 组织与失效 |
| 富文本与编辑模型 | `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text` | `dev/Fyrox/fyrox-ui/src/formatted_text.rs` | run/span 模型、文本编辑器的 layout-marshaller 分层 |
| 文本编辑控件行为 | `dev/godot/scene/gui/{text_edit.cpp, line_edit.cpp}` | `dev/godot/scene/gui/rich_text_label.cpp` | 光标/选区/双击选词/IME（Godot 对 IME preedit 的处理是成熟参考）、富文本标签 |
| shaping 服务化/BiDi | `dev/godot/servers/text` | — | TextServer 的 shaping 接口边界、BiDi 与字簇 API 形态（架构参考，不复制实现） |
| 字体资产 | `dev/Fyrox/fyrox-ui/src/font/` | — | Rust 引擎字体资产加载与度量缓存 |

## 产出记录与时间

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

- 当前失败交接（`open / 待修复`）：[`03/failure-2026-07-11-retained-text-family-and-subpixel-contracts.md`](03/failure-2026-07-11-retained-text-family-and-subpixel-contracts.md)
- fixed 已修复：[rich-table-runtime-export-and-layout-boxes](../editor/15/fixed-2026-07-12-rich-table-runtime-export-and-layout-boxes.md)
- fixed 已修复：[runtime-rich-table-layout-recursion](../../zircon_runtime/runtime/09/fixed-2026-07-12-runtime-rich-table-layout-recursion.md)
