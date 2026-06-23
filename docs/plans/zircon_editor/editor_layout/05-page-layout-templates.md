---
related_code:
  - zircon_editor/src/ui/workbench/page_layout_template.rs
  - zircon_editor/src/tests/workbench/layout/page_layout_templates.rs
  - zircon_editor/src/ui/host/module.rs
  - zircon_editor/assets/ui/editor/layout/page_templates.v2.ui.toml
  - zircon_editor/assets/ui/editor/components/workbench/modules
design_references:
  - docs/ui-and-layout/editor-workbench-designs/scene-drawer-layout-spec.png
  - docs/ui-and-layout/editor-workbench-designs/material-drawer-layout-spec.png
  - docs/ui-and-layout/editor-workbench-designs/inspector-drawer-content-spec.png
  - docs/ui-and-layout/editor-workbench-designs/drawer-expanded-state-spec.png
  - docs/ui-and-layout/ai-workbench-style/STYLE-NOTES.md
plan_sources:
  - docs/plans/zircon_editor/editor_layout/03-jetbrains-docking-workbench.md
  - docs/plans/zircon_editor/editor_ui/09-editor-modules-and-design-parity.md
status: implemented-focused-passed
---
# 05 编辑器页面布局模板与状态规范

## 1. 目标

为每个编辑器页面(主文档标签)定义**它在同一壳骨架内如何填充六区域的布局模板**,并规范激活/展开/折叠/分屏等状态,使所有页面共享一套语言而非各造一套。承接 `editor-workbench-designs` 的各 `*-editor-page`/`*-layout-spec` 设计图。本计划只做**页面布局模板**,模块数据接线在 `editor_ui/09`。

## 2. 现状(按代码核实)

### 2.1 已存在的设施

- 11 个 core module workspace 存在(`host/module.rs`)。
- 骨架六区域契约(承接 03)、区域绑定接口(承接 02)。

### 2.2 真实缺口

- 05.S2 已补齐 13 个页面的区域填充模板(哪个区域放什么面板)。
- 05.S2 已补齐页面级状态规范(默认抽屉状态、默认分屏)。
- 模块数据接线仍按本计划边界留给 `editor_ui/09`,不在页面模板层重做。

## 3. 设计

### 3.1 页面布局模板(按区域填充)

每个页面声明六区域填充,职责遵守 `index.md §1.1`:

| 页面 | left-top | left-bottom | center | right-top | right-bottom | bottom | 设计图 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Scene | 放置工具 | 工程树 | 视口 | 场景树 | Inspector | 控制台 | `scene-drawer-layout-spec` |
| Material | 节点工具 | 资产树 | 材质图 | 大纲 | 参数细节 | 编译输出 | `material-drawer-layout-spec` |
| Inspector(详情) | — | — | 详情主体 | 对象头 | 字段/资源行 | — | `inspector-drawer-content-spec` |
| 其余页面 | 同壳复用,按各 `*-editor-page` 填充 | | | | | | |

### 3.2 页面级状态规范

每页面声明默认布局预设(承接 04)、默认抽屉状态、是否默认分屏。激活态用 teal accent(STYLE-NOTES)。

### 3.3 复用而非重造

页面模板只声明区域填充与默认状态,壳骨架、抽屉行为、token 全部复用 01-04;模块内部内容(数据接线)留给 `editor_ui/09`。

## 4. 接口与数据结构草案(Rust)

```rust
pub struct PageLayoutTemplate {
    pub page: DocumentId,
    pub region_fills: Vec<RegionBinding>, // 复用 02
    pub default_preset: PresetName,        // 复用 04
    pub default_drawer_states: [DrawerState; 5],
}
pub fn instantiate_page(skeleton: &WorkbenchSkeleton, template: &PageLayoutTemplate) -> WorkbenchSkeleton;
```

## 5. 模块与文件落点

| 动作 | 文件 | 说明 |
| --- | --- | --- |
| 新增 | `zircon_editor/assets/ui/editor/layout/page_templates.v2.ui.toml` | 各页面区域填充声明 |
| 修改 | `host/module.rs` | 模块工作区按页面模板入 center |

## 6. 里程碑切片化

| # | 切片 | 涉及文件 | 验证命令 | 硬切换 |
| -- | --- | --- | --- | --- |
| S1 | 核心页面模板(Scene/Material/Inspector) | page_templates.v2.ui.toml / host/module.rs | `cargo test -p zircon_editor --lib --locked` | — |
| S2 | 其余页面 + 状态规范 | page_templates.v2.ui.toml | `cargo test -p zircon_editor --test integration_contracts --features integration-contracts --locked` | — |

## 7. 测试矩阵

- 每页面模板可实例化为正确的区域填充。
- 页面切换时默认预设与抽屉状态正确应用。
- 区域填充职责不违反 `index.md §1.1`。

## 8. 风险与对策

- 风险:页面模板与模块工作区耦合过深。对策:模板只声明"哪个面板入哪个区域",面板内容由模块自身提供(09)。

## 9. 完成定义

13 页面均有区域填充模板与默认状态规范,共享同一骨架与语言。

## 10. 边界约束

不做模块数据接线(属 `editor_ui/09`);区域职责按 `index.md §1.1`;不内嵌设计 PNG。

## 11. 参考实现对照(dev/ 源码锚点)

- `dev/Fyrox/editor`:各编辑器面板入壳参考。
- `dev/theatre/packages/studio`:时间轴页面区域布局参考(bottom 区)。

## 12. 状态与产出记录

| 日期 | 切片 | 状态 | 产出/证据 | 后续项 |
| --- | --- | --- | --- | --- |
| 2026-06-23 | 05.S1 核心页面模板(Scene/Material/Inspector) | implemented-static-passed-editor-cargo-blocked | 已新增 `zircon_editor/src/ui/workbench/page_layout_template.rs` 与 `zircon_editor/assets/ui/editor/layout/page_templates.v2.ui.toml`;Scene/Material/Inspector 三个核心页面复用 02 区域绑定与 04 默认预设,并通过职责角色约束防止面板错槽。scoped rustfmt、`git diff --check`、新模块债务扫描通过。 | 05.S2:补齐其余页面模板与页面级状态规范,再接入 `host/module.rs` 的模块工作区实例化路径。`zircon_editor` Cargo gate 当前在下层 `zircon_runtime` render mesh import 编译漂移处阻塞,未到 editor 测试代码。 |
| 2026-06-23 | 05.S2 其余页面 + 状态规范 | implemented-focused-passed | `PageLayoutTemplate::builtin_templates()` 与 `page_templates.v2.ui.toml` 已覆盖 13 个编辑器页面:Scene/Game/Material/MaterialPreview/Inspector/Prefab/UIDesigner/UISource/AnimationTimeline/AnimationGraph/AssetBrowser/Console/RuntimeDiagnostics。每页声明六区域填充、默认预设、抽屉状态和中心分屏形状;新增 `page_layout_templates.rs` 覆盖页面集合、区域职责、状态 profile 与资产字段。验证:`cargo test -p zircon_editor --lib page_layout_templates --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-editor-0623-clean-2309 --message-format short --color never -- --test-threads=1 --nocapture` 4/4 通过。Cargo 过程中按 support-first 最小修复下层 runtime UI surface split 漂移:拖拽组件事件 helper 可见性、scrollable candidates trait 导入、timer metadata import。 | 05 计划 focused path 已关闭;继续 06.S2 浮动窗口设计对齐验收、01.S2 旧 shell/module token hard cutover 与 03.S2 更宽 editor-layout Cargo 复验债。 |
