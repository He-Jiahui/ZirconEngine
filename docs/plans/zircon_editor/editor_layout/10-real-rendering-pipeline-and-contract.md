---
related_code:
  - zircon_runtime/src/ui/surface/render/extract.rs
  - zircon_runtime/src/ui/surface/render/resolve.rs
  - zircon_runtime/src/ui/surface/render/painter_state.rs
  - zircon_runtime/src/ui/surface/render/node_visual_data.rs
  - zircon_runtime/src/ui/surface/render/cache.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime_interface/src/ui/surface/render/mod.rs
  - zircon_runtime_interface/src/ui/surface/render/command.rs
  - zircon_runtime_interface/src/ui/surface/render/command_kind.rs
  - zircon_runtime_interface/src/ui/surface/render/brush.rs
  - zircon_runtime_interface/src/ui/surface/render/batch.rs
  - zircon_runtime_interface/src/ui/surface/render/resolved_style.rs
  - zircon_runtime_interface/src/ui/surface/render/text_shape.rs
  - zircon_runtime_interface/src/ui/surface/render/cache.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector
design_references:
  - docs/ui-and-layout/editor-workbench-designs/STYLE-NOTES.md
  - docs/ui-and-layout/ai-workbench-style/ai-workbench-web-framework.png
plan_sources:
  - docs/plans/zircon_editor/editor_layout/01-design-tokens-and-language-contract.md
  - docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md
  - docs/plans/zircon_editor/editor_ui/index.md
status: planned
---
# 10 真实渲染管线与渲染规范(style+layout → draw command → 上屏)

## 1. 目标

把"布局/样式如何变成真正上屏的绘制"沉淀为一份**确定性渲染规范**:从已解析样式(01 token)+ 已排布几何(13 Taffy 约束)出发,经**渲染提取(extract)→ 绘制命令(draw command)→ 批次(batch)→ 上屏**的固定管线,产出可缓存、可增量、可验收的绘制结果。本计划只定**渲染契约与提取规范**,不重写底层 `wgpu`/render framework(那是 `zircon_runtime` 渲染框架职责),也不改样式选择器机制(`editor_ui/04` + 本目录 01)。

设计目标:让"编辑器作者只声明 token + 布局 + 组件",绘制结果由确定性管线产出,**像素无歧义、改 token 全局生效、脏区只重绘脏部分**,与 STYLE-NOTES 的扁平无阴影语言一致。

## 2. 现状(按代码核实)

### 2.1 已存在的设施(渲染管线主干已成立,不重做)

| 能力 | 落点 | 证据 |
| --- | --- | --- |
| 渲染提取入口 | `surface/render/extract.rs` | `extract_ui_render_tree(tree, arranged_tree)` → `UiRenderExtract`(含 `draw_order: Vec<UiNodeId>`) |
| 已解析渲染态 | `surface/render/resolve.rs` / `painter_state.rs` | 样式选择器结果落为节点视觉态 |
| 节点视觉数据 | `surface/render/node_visual_data.rs` | 每节点提取后的视觉负载 |
| 绘制命令 DTO | `iface ui/surface/render/command.rs` / `command_kind.rs` | `UiRenderCommand`、`UiRenderCommandKind` |
| 画刷集 | `iface .../render/brush.rs` | `UiBrushPayload`、`UiBrushSet`(Solid/Gradient/Image/Material/Border/Vector/Rounded) |
| 文本绘制 | `iface .../render/text_shape.rs` / `text_layout.rs` | `UiTextPaint`、`UiShapedText`、`UiShapedGlyph` |
| 批次规划 | `iface .../render/batch.rs` | `UiBatch`、`UiBatchPlan`、`UiBatchKey` |
| 渲染缓存 | `surface/render/cache.rs` / `iface .../render/cache.rs` | `UiRenderCachePlan`、`UiRenderStats`、`UiRenderDebugSnapshot` |
| 已解析样式 | `iface .../render/resolved_style.rs` | `UiResolvedStyle` |

### 2.2 真实缺口与隐患

- 缺**面向编辑器布局的渲染契约文档**:管线已存在,但没有把"扁平/无阴影/1px 边框/低圆角"这套设计语言固化为渲染层可验收的硬约束(目前靠资产自觉)。
- 缺**token→画刷的单源喂入规范**:`UiBrushPayload::Solid` 等可接受任意色值,缺"编辑器面板的 surface/accent/border 必须来自 01 token 解析"的渲染层 guard。
- 缺**禁用视觉的渲染层拦截**:STYLE-NOTES 禁渐变/辉光/阴影/模糊,但 `UiBrushSet` 含 `Gradient`,缺"编辑器 chrome 不得提取出 Gradient/Shadow 命令"的提取期校验。
- 缺**脏区→重提取的增量契约**:`extract` 当前按 `draw_order` 全树走;需明确"只有脏视图/脏节点重提取,其余复用 `UiRenderCachePlan`",与 09 的 `ViewDirtySet` 对齐。
- 缺**渲染验收基线**:命令计数、批次数、缓存命中率作为可回归指标(`UiRenderStats`/`UiRenderDebugSnapshot` 已有载体,缺契约阈值)。

## 3. 设计

### 3.1 确定性渲染管线(五段固定)

```
(01 token 解析样式 UiResolvedStyle)  +  (13 Taffy 排布几何 UiArrangedTree)
        │                                        │
        └──────────────┬─────────────────────────┘
                       ▼
   ① RESOLVE   resolve.rs / painter_state.rs
                每节点合成视觉态(色/边框/圆角/文本)= UiNodeVisualData
                       ▼
   ② EXTRACT   extract.rs：按 draw_order 遍历排布树,产出有序绘制意图
                       ▼
   ③ COMMAND   command.rs：视觉意图 → UiRenderCommand(矩形/边框/文本/图像/裁剪)
                       ▼
   ④ BATCH     batch.rs：按 UiBatchKey(画刷/材质/裁剪层)合并为 UiBatchPlan
                       ▼
   ⑤ PRESENT   交给运行时 render framework 上屏(本计划不改其内部)
```

每段输入/输出类型固定;脏区只重跑必要段(见 §3.4)。

**坐标与 scale 职责条款(2026-07-02 评审收口)**:全管线(SOURCE→STYLE→LAYOUT→COMMAND,即 ①—④)一律**逻辑坐标**;逻辑→物理换算**单点发生在 `21` 顶点装配阶段**(乘 `scale_factor` + 像素吸附),①—④ 段不得出现乘 scale 的代码。唯一例外是**文本字形**:字形栅格按物理像素进行,由 runtime `text/04` 的 `GlyphRasterKey { px_size_bucket }` 承担(`px_size_bucket = logical_px × scale_factor` 量化)。与 `16` §3.4、`21` 同条款互为引用。

**opacity 语义条款(2026-07-02 评审收口)**:透明度为**逐命令 α 直乘**(命令色值的 alpha 通道,COMMAND 段直乘);**子树组透明(group opacity)V1 禁止**——正确组透明需离屏合成,直乘会导致重叠子元素双重混合。如弹层淡入淡出确需整树 α,允许受限形态:整棵子树同一 α **且子元素互不重叠**;离屏合成登记为 V2 项。

**多窗口条款(2026-07-02 评审收口)**:管线以**渲染根(窗口)为单位实例化**——每窗口独立走 ①—⑤,持各自的 `scale_factor` 与 extract/present 节奏;`UiRenderStats` 按窗口分账,跨窗口不合批、不共享脏集(与 `21` 多窗口条款一致)。

### 3.2 渲染契约(把设计语言固化到渲染层)

把 01 设计语言契约的视觉规则下沉为**提取期硬约束**,编辑器 chrome 渲染必须满足:

| 渲染契约 | 规则 | 拦截点 |
| --- | --- | --- |
| 画刷单源 | 编辑器面板 `Solid`/`Border` 色值必须解析自 01 token,不得为裸 `UiRgbaColor` 字面量 | EXTRACT 期校验节点视觉数据来源 |
| 禁用视觉 | 编辑器 chrome 不得提取出 `UiBrushSet::Gradient` 命令(现存 DTO 中唯一违规 kind);阴影/辉光/模糊 kind 当前 DTO **不存在**,登记为"未来新增 kind 默认禁用"预防条款(2026-07-02 评审收口) | COMMAND 期拒绝违规 kind |
| 边框规格 | 边框宽=1px、圆角=低圆角 token,扁平态 | RESOLVE 期取 `control.radius`/`border.width` token |
| 文本 | 文本色取 `text.*` token,字号取密度 token,无英雄字号 | RESOLVE 期取 typography token |
| 状态视觉 | 激活/选中/焦点用 accent token,且仅这些态用 accent | RESOLVE 期按状态优先级(01)解析 |

契约只对**编辑器 chrome 资产**强制(`components/workbench/**`、`floating/**`、`layout/**`);用户内容视口(center 自由区)不强制,可用任意画刷。

**15c `shadow` token 豁免行(2026-07-02 评审收口)**:`15c` §8 登记的 popup `shadow` 角色为 **1px 分隔式微透明实线**(非高斯阴影/辉光),属 `Solid`/`Border` 画刷形态,不触发禁用视觉拦截——契约 guard 对该 token 溯源的用法放行,与 `15c` §8 对齐。

### 3.3 token → 画刷喂入规范

```
01 EditorDesignTokens
   → resolve_painter_style(family, state)  (01.S2 已落地)
   → UiResolvedStyle{ surface, foreground, border, radius, height }
   → RESOLVE 把 UiResolvedStyle 映射为 UiNodeVisualData{ fill: UiBrushPayload::Solid(token_color), border: ... }
   → EXTRACT/COMMAND 透传
```

组件 `.zui` 写 token 名(`$editor.surface.1` / `$--left-drawer-width`),**不写裸 hex**;渲染层若发现编辑器 chrome 节点视觉色无 token 溯源 → 提取期报告违规(配合 01 的资产扫描契约,形成"资产层 + 渲染层"双闸)。

### 3.4 增量渲染(对齐 09 的视图脏集)

| 变更类型 | 重跑段 | 复用段 |
| --- | --- | --- |
| paint-only(色/态变,几何不变) | ④ 起(命令可复用、仅刷脏节点) | ①②③ 缓存(`record_paint_only_invalidation` 快路径) |
| 视图局部脏(09 `ViewDirtySet`) | 仅脏视图子树走 ①→④ | 其余视图复用 `UiRenderCachePlan` |
| 布局脏(13 排布变) | ②→④ | ① 部分复用 |
| 结构脏(增删节点) | ①→④ 该子树 | 兄弟子树缓存 |

**cache_clear owner 条款(2026-07-02 评审收口)**:布局脏对应的 taffy 子树 `cache_clear` 的执行 owner = **`09` 帧末 drain 的布局脏处理段**(脏节点入队后由 09 统一在帧末清缓存并触发重排,本表"布局脏"行的前置动作);`13` §3.8 同步引用此条款。

硬约束:**不存在"全树无条件重提取"的常规入口**;全量重建只作 09 §8 的调试兜底命令。渲染统计(`UiRenderStats`)须能证明"改一个抽屉只重提取该抽屉子树"。

### 3.5 渲染验收指标(可回归)

把 `UiRenderDebugSnapshot` / `UiRenderStats` 作为验收载体,契约阈值:
- 编辑器 chrome 单帧违规视觉命令数 = 0(无 Gradient/Shadow/Glow/Blur)。
- 改单一抽屉 token,重提取节点数 ≤ 该抽屉子树节点数(不波及兄弟)。
- 批次合并率:同画刷相邻矩形应合批(`UiBatchPlan` 批数随合批下降)。

## 4. 接口与数据结构草案(Rust)

```rust
// zircon_runtime/src/ui/surface/render/extract.rs 旁,编辑器渲染契约 guard
pub struct EditorRenderContract;
impl EditorRenderContract {
    /// 提取期:校验 chrome 子树视觉数据全部 token 溯源、无禁用视觉
    pub fn validate_chrome_extract(extract: &UiRenderExtract, scope: ChromeScope) -> Result<(), EditorRenderViolation>;
}
pub enum EditorRenderViolation {
    UntokenizedColor { node: UiNodeId },
    ForbiddenBrush { node: UiNodeId, brush: UiBrushSetKind }, // 现存 DTO 仅 Gradient 违规;未来新增 kind 默认禁用(2026-07-02 评审收口)
    NonFlatChrome { node: UiNodeId },
}
// 增量提取:只对脏视图子树重提取,对齐 09 ViewDirtySet
pub fn extract_dirty_views(
    surface: &UiSurface,
    dirty: &ViewDirtySet,
    cache: &mut UiRenderCachePlan,
) -> UiRenderExtractDelta;
```

## 5. 模块与文件落点

| 动作 | 文件 | 说明 |
| --- | --- | --- |
| 新增 | `zircon_runtime/src/ui/surface/render/editor_render_contract.rs` | chrome 渲染契约 guard(提取期校验) |
| 修改 | `surface/render/extract.rs` | 暴露脏视图增量提取入口,接 09 `ViewDirtySet` |
| 新增 | `docs/ui-and-layout/render-pipeline-contract.md` | 五段管线 + 渲染契约 + 验收指标文档 |
| 修改 | `paint_template_nodes/style_selector` | 确认 token→画刷喂入,无裸色回退 |

## 6. 里程碑切片化

| # | 切片 | 涉及文件 | 验证命令 | 硬切换 |
| -- | --- | --- | --- | --- |
| S1 | 渲染契约 guard + 管线文档 + 禁用视觉拦截 | editor_render_contract.rs / render-pipeline-contract.md | `cargo test -p zircon_runtime --lib editor_render_contract --locked` | 新建,无旧路径 |
| S2 | 脏视图增量提取接 09 + 验收指标 | extract.rs / cache.rs | `cargo test -p zircon_runtime --lib --locked` | 删除编辑器路径的全树无条件重提取调用 |

## 7. 测试矩阵

- chrome 子树含裸 hex 或 Gradient/Shadow 命令时,契约 guard 报违规。
- token 改动后,RESOLVE 产出的画刷色与 01 token 一致。
- 单视图脏只重提取该子树,`UiRenderStats` 重提取计数受界。
- paint-only 变更走快路径,不重布局不重命令生成。
- 相邻同画刷矩形合批,批数下降。

## 8. 风险与对策

- 风险:渲染契约误伤 center 自由区用户内容。对策:契约只对 chrome scope 生效,center/viewport 不校验。
- 风险:增量提取脏区遗漏致漏绘。对策:复用 09 的显式全量兜底命令 + 渲染统计回归。
- 风险:与底层 render framework 重构耦合。对策:本计划只到 `UiBatchPlan` 边界,PRESENT 段不碰。

## 9. 完成定义

五段渲染管线契约成文;编辑器 chrome 渲染 token 单源、无禁用视觉;脏视图增量提取接 09;渲染验收指标可回归。

## 10. 边界约束

不重写 `wgpu`/render framework(运行时渲染职责);不改样式选择器优先级(01 + `editor_ui/04`);渲染契约只约束编辑器 chrome,不约束用户内容视口;不内嵌设计 PNG。

## 11. 参考实现对照(dev/ 源码锚点)

- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Rendering`:`FSlateDrawElement`/`FSlateWindowElementList` — draw element → batch → render 的提取/批次样板。
- `dev/UnrealEngine/.../SlateCore/Public/FastUpdate`:invalidation panel 的增量重绘,取"只重提取脏 widget"理念。
- `dev/bevy/crates/bevy_ui/src/render`:`ExtractedUiNodes`/`UiBatch` — ECS 式提取→批次参考。
- `dev/slint/internal/core/items`(item rendering)与 `internal/renderers`:retained item → 渲染原语映射参考。

## 12. 状态与产出记录

planned。后续项:S1 渲染契约 guard + 管线文档 + 禁用视觉拦截。
