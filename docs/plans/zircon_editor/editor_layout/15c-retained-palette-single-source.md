---
related_code:
  - zircon_runtime_interface/src/ui/design_tokens.rs
  - zircon_editor/assets/ui/editor/theme/editor_tokens.v2.ui.toml
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme/palette_projection.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme/tokens.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme/model.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_theme.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_chrome/palette.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_button/palette.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/style_selector/workbench_icon_button/palette.rs
design_references:
  - docs/ui-and-layout/editor-workbench-designs/STYLE-NOTES.md
  - docs/ui-and-layout/design-language-contract.md
plan_sources:
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/zircon_editor/editor_layout/01-design-tokens-and-language-contract.md
  - docs/plans/engine-code-structure-convention.md
status: implemented_focused_passed_build_screenshot_passed
---
# 15c 工作台 chrome 色板单源收口(S15.6 深化)

> 本文是 `15` 计划 **S15.6** 切片的实现就绪深化:把 retained-host 的**手写色板**收敛到 `01` 的中央 `EditorDesignTokens::palette`,做到"改一处中央 token,chrome 整体随动"。同时是 `01.S2` 的色彩侧收口。父计划见 `15`,度量侧单源见 `15b`(度量+色彩=双单源)。

## 1. 现状:色彩有"两套半"且已漂移(按代码核实)

| 色源 | 落点 | 字段数 | 性质 |
| --- | --- | --- | --- |
| **中央(权威)** | `zircon_runtime_interface/src/ui/design_tokens.rs::EditorPaletteTokens::workbench_dark()` | 10(`surface[4]`+`accent`+`border`+`text×3`+`success/info/warning/error`) | `01` 的设计 token 单源 |
| **retained `PALETTE`** | `paint_theme/tokens.rs::PALETTE`(`HostMaterialPalette`) | **26** | **手写**,与中央**重叠且漂移** |
| **chrome BG/分隔** | `style_selector/workbench_chrome/palette.rs` | 14 | **手写**,与中央 surface/border 重叠 |
| **按钮/图标态色** | `style_selector/workbench_button/palette.rs`、`workbench_icon_button/palette.rs` | 若干 | **手写**态色 |

### 1.1 已发生的漂移(证明"两套"有害)
| 语义 | 中央 `EditorPaletteTokens` | retained `PALETTE` | 漂移 |
| --- | --- | --- | --- |
| `border` | `[57,65,71]` | `[52,60,66]` | ❌ 不一致 |
| `text` 主 | `text_primary [232,236,238]` | `text [226,230,232]` | ❌ |
| `text` 次 | `text_secondary [164,174,180]` | `text_muted [168,178,183]` | ❌ |
| `text` 禁用 | `text_disabled [101,111,118]` | `text_disabled [98,106,112]` | ❌ |
| `error` | `[235,96,92]` | `[239,53,53]` | ❌ |
| `surface[2]` 面 | `[27,31,35]` | `surface [27,31,35]` | ✅ |
| `accent` teal | `[60,199,214]` | `accent [60,199,214]` | ✅ |

> 结论:这正是 `15` 缺口 G6 说的"色彩事实源有两套"。S15.6 让 retained 侧**从中央投影**(中央赢),消除漂移。

### 1.2 retained 比中央"多出来"的语义色
retained `PALETTE` 有中央**没有**的角色:`surface_inset[15,19,22]`、`surface_hover[52,58,64]`、`surface_selected[10,76,86]`、`surface_disabled[34,39,43]`、`accent_soft[13,78,88]`、`border_disabled`、`track`、`popup`、`focus_ring`、`shadow`、`*_container×4`;chrome 还有 `STRONG/SOFT_SEPARATOR`。收口时每个都要决定:**并入中央** 或 **由中央派生**。

## 2. 收口设计

### 2.1 方案择优:扩展中央(主)vs 派生(辅)
- **主:把缺失角色显式并入 `EditorPaletteTokens`**(新增 `surface_recessed`、`surface_hover`、`surface_selected`、`surface_disabled`、`accent_soft`、`track`、`popup`、`focus_ring`、`separator_strong`、`separator_soft`、`shadow`、`container_*`)。好处:中央成为**完整单源**,retained 侧 1:1 投影,"改一处全局动"字面成立。代价:扩 `zircon_runtime_interface` 的纯数据 DTO(ABI 安全、不违 E8 边界禁令)。
- **辅:纯装饰色由中央派生**(可选,降低 DTO 膨胀):`separator_strong = lighten(border)`、`separator_soft = darken(border)`、`shadow = rgba(0,0,0,α)`、`*_container = darken(role, k)`。派生函数纯函数化、可单测。
- **建议**:语义结构色(inset/hover/selected/disabled/accent_soft/focus_ring/track/popup)走**并入**;纯色调装饰(separators/shadow/containers)走**派生**。最终 retained 侧无手写绝对色。

### 2.2 投影函数(单一事实源 → retained)
落 `paint_theme` 下新 owner 叶子 `palette_projection.rs`:
```rust
pub(in crate::ui::retained_host::host_contract) fn project_host_palette(
    tokens: &EditorDesignTokens,
) -> HostMaterialPalette {
    let p = &tokens.palette;
    HostMaterialPalette {
        shell_background: rgba8(p.surface[0]),
        surface:          rgba8(p.surface[2]),
        surface_inset:    rgba8(p.surface_recessed),     // 并入
        surface_pressed:  rgba8(p.surface[3]),
        accent:           rgba8(p.accent),
        border:           rgba8(p.border),               // 漂移消除:中央赢
        text:             rgba8(p.text_primary),
        text_muted:       rgba8(p.text_secondary),
        // separator_strong: 派生 lighten(p.border) …
        ..
    }
}
```
`paint_theme/tokens.rs::PALETTE` 由 `OnceLock` 缓存 `project_host_palette(&EditorDesignTokens::workbench_dark())`(或在 host 初始化期注入当前 tokens),不再手写字面量。`workbench_chrome/palette.rs` 的 14 个 BG/分隔常量同样改为投影读出(`ROOT_BG=surface[0]`、`PANEL_BG=surface[2]`、`TAB_BG=surface[3]`、`RAIL/INSET=surface_recessed`、`SEPARATOR=border`…),删手写。

### 2.3 资产 `editor_tokens.v2.ui.toml`
新增并入的 token 名(`editor.surface.recessed`/`editor.surface.hover`/`editor.surface.selected`/`editor.accent.soft`/`editor.focus.ring`/…),保持资产=中央 DTO=retained 投影**三处一致**(遵 `13`/`10` 的 chrome 禁裸值、token 三处一致)。

## 3. 结构/边界纪律
- 扩 `EditorPaletteTokens` 属 `zircon_runtime_interface` **纯数据 DTO**,ABI 安全,不引 wgpu/slint(不违 `E8`)。
- 投影 owner `palette_projection.rs` 落叶子;`paint_theme.rs` 仅加 `mod`+`use`(薄根)。
- 硬切换:删 `PALETTE` 手写字面、删 `workbench_chrome/palette.rs` 手写常量、删按钮/图标手写态色,改投影;grep 旧字面零命中(除注释)。
- 无 `unwrap/expect`(`OnceLock::get_or_init` 用 infallible 投影);文件 ≤800。

## 4. 测试矩阵
| 测试 | 断言 |
| --- | --- |
| `host_palette_projects_from_central_tokens` | `project_host_palette(workbench_dark)` 各字段 == 期望(surface/accent/border/text 来自中央) |
| `chrome_backgrounds_track_central_surface_ladder` | `ROOT_BG==surface[0]`、`PANEL_BG==surface[2]`、`TAB_BG==surface[3]` |
| `changing_central_accent_moves_chrome_accent` | 改中央 `accent` → 投影 accent/focus_ring 随动 |
| `no_second_handwritten_palette_remains`(grep 守卫) | retained 侧无手写 `[r,g,b,a]` 色字面(除派生函数输入) |
| 派生纯函数单测 | `lighten/darken/container` 给定输入→期望输出 |

## 5. 验收
- `capture_m3_gui_acceptance_visual_artifacts --ignored` + 组件 atlas 截图刷新 `docs/tests/editor/`。
- 因 surface/accent **本就一致**,这些区域像素不变;`border/text/error` 等**漂移项收敛到中央值**(像素**有意微调**,在状态表注明为"漂移消除,非回归")。
- 人工:改中央 token 重跑截图,chrome 色整体随动(单源生效)。

## 6. 与 01 / 15b 的关系
- `01.S2` 的色彩侧由本切片收口(度量侧由 `15b`),`01` 的中央 `EditorDesignTokens` 成为 chrome 度量+色彩双单源。
- `15b` 的 `HostControlMetrics` 后续亦可由 `EditorControlTokens`/`EditorDensityTokens` 投影,与本文同构。

## 7. 实现顺序
1. 扩 `EditorPaletteTokens` + 资产 token 名(RED:projection 测试先失败)。
2. 建 `palette_projection.rs` + 派生纯函数 + 单测(GREEN)。
3. `PALETTE`/`workbench_chrome`/按钮·图标态色改投影,删手写。
4. 跑色板投影测试 + 截图复验(注明漂移消除项)+ grep 守卫;写状态。

## 8. 边界
不改选择器优先级机制(属 `editor_ui/04`);不引渐变/辉光/阴影(`shadow` 仅既有微透明,沿用);不动 B 层 MUI 移植件色板(那是 MUI design system)。

## 9. 状态与产出记录
| 日期 | 切片 | 状态 | 产出/证据 | 后续项 |
| --- | --- | --- | --- | --- |
| 2026-06-25 | 15c/S15.6 深化立项 | planned | 代码核实色彩"两套半"且**已漂移**(`border [57,65,71]≠[52,60,66]`、`text_primary≠text`、`error [235,96,92]≠[239,53,53]`),并列出 retained 比中央多出的 12+ 语义色;给出"并入(语义结构色)+ 派生(装饰色)"收口方案、`project_host_palette` 投影 owner、chrome 14 常量→surface 阶梯映射、5 条测试矩阵、漂移消除非回归说明、实现顺序。 | 按 §7 实现 S15.6;父计划 `15` 的 S15.6/D6 勾稽随之更新;与 `15b` 合成度量+色彩双单源。 |
| 2026-06-25 | 15c/S15.6 retained palette 单源硬切换 | implemented-focused-passed-build-screenshot-passed | 已扩 `EditorPaletteTokens` 与 `editor_tokens.v2.ui.toml`,新增 `paint_theme/palette_projection.rs` 作为中央 token → retained `HostMaterialPalette` 的唯一投影 owner,`paint_theme/tokens.rs::PALETTE` 改为 `DEFAULT_HOST_PALETTE`;Workbench chrome/button/icon/dropdown/text-field/table/tree/slider/status/alert/toast/tooltip 等 style selector palette 文件已改为消费 `PALETTE` 角色,不再保留第二套 handwritten RGBA。静态验证:`rustfmt --edition 2021 --check` 覆盖触及 Rust 文件;retained palette/style-selector `[r,g,b,255]` 扫描只剩 `#[cfg(test)]` 改 central accent 的测试 fixture;触及生产路径 `unwrap/expect/TODO/FIXME/HACK/allow(dead_code)/Result<_,String>` 扫描零命中;截图/preview fallback 默认路径从 `target` 改到 `docs/tests/editor`。Cargo 验证:`cargo fmt -p zircon_editor -p zircon_runtime_interface --check`;`cargo test -p zircon_runtime_interface editor_design_tokens --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --color never -- --test-threads=1 --nocapture` 5/5;`cargo check -p zircon_editor --lib --tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never`;`host_palette_projects_from_central_tokens` 1/1;`chrome_backgrounds_track_central_surface_ladder` 1/1;`cargo build -p zircon_editor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-components-0625 --message-format short --color never`;`capture_m3_gui_acceptance_visual_artifacts --ignored` 1/1。组件 atlas 的 cargo wrapper 曾 604s 超时且不计通过;随后直接执行已编译测试二进制 `capture_workbench_component_slate_atlas_visual_artifact --ignored --exact` 1/1 通过,刷新 `docs/tests/editor/editor-components-workbench-slate-atlas-900x620.png`。 | S15.6 色板单源关闭;未把完整 S15/goal 标记完成。S15.5 剩余 popup anchor/default threshold token 化、窗口 minimum 与 Ultra 档仍 open;整窗组合观感继续在后续 S15.4/S15.5 复核。 |
