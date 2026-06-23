---
related_code:
  - zircon_runtime/src/ui/layout/mod.rs
  - zircon_runtime/src/ui/layout/taffy_bridge.rs
  - zircon_runtime_interface/src/ui/layout/mod.rs
  - zircon_runtime_interface/src/ui/layout/constraints.rs
  - zircon_runtime_interface/src/ui/layout/slot.rs
  - zircon_editor/src/ui/workbench/autolayout/mod.rs
  - zircon_editor/src/ui/workbench/autolayout/workbench_shell_geometry.rs
  - zircon_editor/src/ui/workbench/autolayout/axis_constraint_override.rs
design_references:
  - docs/ui-and-layout/editor-workbench-designs/main-tabs-layout-spec.png
  - docs/ui-and-layout/editor-workbench-designs/tool-drawers-layout-spec.png
  - docs/ui-and-layout/ai-workbench-style/prototype/README.md
plan_sources:
  - docs/plans/zircon_editor/editor_layout/01-design-tokens-and-language-contract.md
  - docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md
status: in_progress
---
# 02 声明式布局接口(区域语义 / 槽位 / 约束 Token)

## 1. 目标

给编辑器作者(及后续插件作者)一套**用户友好的设计接口**:用区域语义(region) + 槽位(slot) + 约束 token(constraint token)声明界面布局,而不是手写 Taffy 节点树或绝对坐标。接口面向"我要一个左抽屉、宽度用 `--left-drawer-width` token、放工程树"这种意图表达,底层翻译成壳 autolayout + Taffy 求解。本计划只做**声明层接口**,布局引擎本身在 `editor_ui/02`。

## 2. 现状(按代码核实)

### 2.1 已存在的设施

| 能力 | 落点 | 证据 |
| --- | --- | --- |
| Taffy 后端 | `taffy_bridge.rs` | Flex/Grid/Block/Wrap 求解 |
| 约束 DTO | `layout/constraints.rs` | 尺寸/min/max 约束 |
| Slot 模型 | `layout/slot.rs` | 子节点放置语义 |
| 壳 autolayout | `workbench_shell_geometry.rs` / `axis_constraint_override.rs` | 壳级区域几何 + 轴约束覆盖 |
| 尺寸 token 概念 | `ai-workbench-style/prototype/README.md` | `--left-drawer-width`/`--right-drawer-width`/`--bottom-output-height` |

### 2.2 真实缺口

- 缺面向作者的区域语义命名(region:left-top / left-bottom / right-top / right-bottom / bottom / center)。
- 缺约束 token 的声明形态(尺寸用 token 名而非裸像素)。
- 缺槽位→区域的声明式绑定接口。

## 3. 设计

### 3.1 区域语义(对应抽屉区职责)

按 `index.md §1.1` 的抽屉区职责命名区域,作者按语义而非坐标声明:

| 区域名 | 职责(见 index §1.1) |
| --- | --- |
| `region.left-top` | 放置/预制体工具 |
| `region.left-bottom` | 文件/工程树 |
| `region.right-top` | 层级/结构 |
| `region.right-bottom` | 属性/动画/细节 |
| `region.bottom` | 控制台/诊断/时间轴 |
| `region.center` | 活动视口/文档表面 |

### 3.2 约束 token(尺寸单源)

尺寸用 token 名声明,token 值集中在设计 token 资产(承接 01):`--left-drawer-width`/`--right-drawer-width`/`--bottom-output-height`,以及 `gap.*`。作者写 token 名,改一处全局生效。

### 3.3 声明式绑定接口

作者声明"哪个 `.zui` 面板放进哪个区域、用哪个约束 token",接口翻译成壳 autolayout 输入。区域为固定语义槽,面板入槽前校验职责匹配(见 §1.1)。

## 4. 接口与数据结构草案(Rust)

```rust
pub enum EditorRegion { LeftTop, LeftBottom, RightTop, RightBottom, Bottom, Center }
pub struct RegionBinding {
    pub region: EditorRegion,
    pub panel_asset: AssetRef,        // .zui 面板
    pub size_token: Option<ConstraintTokenName>, // 如 "--left-drawer-width"
}
pub fn build_shell_layout(bindings: &[RegionBinding]) -> WorkbenchShellGeometry;
```

## 5. 模块与文件落点

| 动作 | 文件 | 说明 |
| --- | --- | --- |
| 新增 | `zircon_editor/src/ui/workbench/autolayout/region_binding.rs` | 区域语义 + 绑定接口 |
| 修改 | `workbench_shell_geometry.rs` | 接受声明式绑定输入 |
| 新增 | `zircon_editor/assets/ui/editor/layout/shell_regions.v2.ui.toml` | 区域→面板声明资产 |

## 6. 里程碑切片化

| # | 切片 | 涉及文件 | 验证命令 | 硬切换 |
| -- | --- | --- | --- | --- |
| S1 | 区域语义 + 约束 token 接口草案 | region_binding.rs | `cargo check -p zircon_editor --lib --locked` | 新建 |
| S2 | 声明落到壳 autolayout + 区域资产 | workbench_shell_geometry.rs / shell_regions.v2.ui.toml | `cargo test -p zircon_editor --lib --locked` | 移除壳代码内联尺寸 |

## 7. 测试矩阵

- 区域绑定可声明并产出正确壳几何。
- 约束 token 名解析为设计 token 资产中的值。
- 面板入槽时职责不匹配报错(如把属性面板放进 left-bottom)。

## 8. 风险与对策

- 风险:区域语义太死,挡住合理的自定义布局。对策:保留 `region.center` 为自由区,职责校验只对四角抽屉强制。

## 9. 完成定义

作者用区域 + 槽位 + token 声明布局,壳几何由声明产出,无裸像素手写。

## 10. 边界约束

不改 Taffy 求解(属 `editor_ui/02`);约束 token 值来自 01;不手写绝对坐标。

## 11. 参考实现对照(dev/ 源码锚点)

- `dev/bevy/crates/bevy_ui/src/layout/convert.rs`:声明式样式→taffy 转换样板。
- `dev/Fyrox/fyrox-ui/src/dock`:docking 区域语义参考。

## 12. 状态与产出记录

| 日期 | 切片 | 状态 | 产出/证据 | 后续项 |
| --- | --- | --- | --- | --- |
| 2026-06-23 | 02.S1 区域语义 + 约束 token 接口草案 | implemented-static-passed-editor-cargo-blocked | 已新增 `zircon_editor/src/ui/workbench/autolayout/region_binding/` owner 树,包含 `EditorRegion`、`EditorRegionRole`、`WorkbenchConstraintTokenName`、`RegionBinding` 与 `RegionBindingError`;`zircon_editor/assets/ui/editor/layout/shell_regions.v2.ui.toml` 记录六区域声明;`zircon_editor/src/tests/workbench/layout/editor_layout_contracts.rs` 覆盖职责匹配与区域映射。scoped rustfmt、`git diff --check`、新模块债务扫描通过。 | 02.S2:把声明资产加载接入壳 autolayout,替换 `workbench_main_band.zui` 内裸尺寸为 `--left-drawer-width` / `--right-drawer-width` / `--bottom-output-height` token。`zircon_editor` Cargo gate 当前在下层 `zircon_runtime` render mesh import 编译漂移处阻塞,未到 editor 测试代码。 |
| 2026-06-23 | 02.S2a 壳声明 token extents + 抽屉资产尺寸 token 化 | implemented-focused-passed | `WorkbenchSkeleton::preferred_region_extents_from_tokens(...)` 已把区域绑定里的 `size_token` 投影成 `BTreeMap<ShellRegionId, f32>`,并通过既有 `compute_workbench_shell_geometry(..., transient_region_preferred)` normal path 喂入壳 autolayout。`workbench_main_band.zui`、`workbench_scene_tree_panel.zui`、`workbench_inspector_panel.zui` 已导入 `editor_tokens.v2.ui.toml`,左/右抽屉宽度从内联 `332.0`/`404.0` 改为 `$--left-drawer-width`/`$--right-drawer-width`。验证:`editor_layout_contracts` 新增 2 个断言,覆盖 token extents 改变壳几何与抽屉资产不保留旧内联宽度;`cargo check -p zircon_editor --lib --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-editor-0623 --message-format short --color never` 通过;`cargo test -p zircon_editor --lib editor_layout_contracts --no-run --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-editor-0623 --message-format short --color never` 通过;直接运行测试二进制 `editor_layout_contracts --test-threads=1 --nocapture` 8/8 通过;scoped rustfmt、diff check、token 资产扫描和尾随空白扫描通过。 | 02.S2 还剩 authored `shell_regions.v2.ui.toml` 到 DTO 的加载入口与完整区域资产 ingestion;本轮只关闭内置 skeleton 声明投影、壳 autolayout token feed 和抽屉壳资产裸宽度替换。 |
| 2026-06-23 | 02.S2b authored shell_regions 资产加载入口 | implemented-focused-passed | 新增 `zircon_editor/src/ui/workbench/autolayout/shell_regions_asset.rs` owner,负责 `shell_regions.v2.ui.toml` 的 header schema、TOML 解析、区域完整性/重复区域校验、职责错配 typed error,并把验证后的 `RegionBinding` 列表投影到 `WorkbenchSkeleton::from_shell_regions_asset(...)` / `from_shell_regions_asset_str(...)`。`autolayout/mod.rs` 只新增模块声明和 re-export。`editor_layout_contracts` 新增真实资产加载断言和职责错配拒绝断言,并证明真实资产生成的 skeleton extents 能继续喂入 `compute_workbench_shell_geometry(..., transient_region_preferred)`。验证:`cargo test -p zircon_editor --lib editor_layout_contracts --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-editor-0623 --message-format short --color never -- --test-threads=1 --nocapture` 10/10 通过;scoped `rustfmt --check`、生产 debt scan、尾随空白扫描、`Cargo.lock` 无内容 diff 和备份清理检查通过。 | 02.S2 的声明资产到 DTO/shell autolayout 路径已具备 focused 证据;后续可进入 03.S2 停靠状态运行时或继续补 01.S2 旧 shell/module 资产 token hard cutover。 |
