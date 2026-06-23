---
related_code:
  - zircon_editor/src/ui/workbench/mod.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/host/module.rs
  - zircon_editor/assets/ui/editor/components/workbench/shell
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_main_band.zui
design_references:
  - docs/ui-and-layout/editor-workbench-designs/main-tabs-layout-spec.png
  - docs/ui-and-layout/editor-workbench-designs/tool-drawers-layout-spec.png
  - docs/ui-and-layout/editor-workbench-designs/scene-drawer-layout-spec.png
  - docs/ui-and-layout/editor-workbench-designs/drawer-expanded-state-spec.png
  - docs/ui-and-layout/editor-workbench-designs/split-editor-state-spec.png
  - docs/ui-and-layout/ai-workbench-style/prototype/README.md
plan_sources:
  - docs/plans/zircon_editor/editor_layout/02-declarative-layout-interface.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
status: in_progress
---
# 03 类 JetBrains 停靠工作台架构

## 1. 目标

把"类 JetBrains 的设计架构"落成一套完整的工作台骨架规范:**主文档标签条 + 活动栏 + 四角停靠抽屉 + 底部输出 + 状态栏 + 浮动窗口**,带停靠语义(展开/折叠/分屏/激活)。本计划定义**架构结构与区域职责**,运行时 docking 机制在 `editor_ui/08`,本目录只定结构契约与抽屉行为规范。

## 2. 现状(按代码核实)

### 2.1 已存在的设施

| 能力 | 落点 | 证据 |
| --- | --- | --- |
| Rust-owned retained host | `retained_host/app.rs` | 壳承载 |
| 壳区域 `.zui`(8 件) | `assets/.../workbench/shell` | top_toolbar/main_band/activity_rail/status_bar/component_drawer/scene_tree_panel/inspector_panel/viewport_panel |
| 主带组合 | `workbench_main_band.zui` | 场景树 + 视口 + 模块工作区 |
| 11 个 core module workspace | `host/module.rs` | 各模块壳 |

### 2.2 真实缺口

- 缺工作台骨架的结构契约(各区域固定职责、嵌套关系)。
- 缺停靠抽屉行为规范(展开/折叠/分屏/激活态),对应 `drawer-expanded-state-spec`/`split-editor-state-spec`。
- 缺主文档标签 + 活动栏的 JetBrains 式语义(标签 = 文档,活动栏 = 抽屉切换)。

## 3. 设计

### 3.1 工作台骨架(类 JetBrains)

```
┌─────────────────────────────────────────────┐
│ 壳命令条 (top toolbar)                        │
├──┬────────────────────────────────────────┬──┤
│活│ 主文档标签条 (main document tabs)        │活│
│动├──────────┬───────────────┬──────────────┤动│
│栏│ left-top │               │  right-top   │栏│
│左│ left-bot │  center 视口   │  right-bot   │右│
├──┴──────────┴───────────────┴──────────────┴──┤
│ bottom 输出 (console/diagnostics/timeline)     │
├──────────────────────────────────────────────┤
│ 状态栏 (status bar)                            │
└──────────────────────────────────────────────┘
```

- **主文档标签**:Scene/Material/Montage/UI Asset/Asset Browser/Diagnostics/Project 等 = 可切换文档,对应 `main-tabs-layout-spec.png`。
- **活动栏**:左右垂直命令栏,点击切换抽屉显隐,对应 `tool-drawers-layout-spec.png`。
- **四角抽屉**:职责见 `index.md §1.1`,固定语义。
- **底部输出 / 状态栏**:控制台/诊断/时间轴 + 全局状态。

### 3.2 停靠抽屉行为规范

| 行为 | 语义 | 对应设计图 |
| --- | --- | --- |
| 展开/折叠 | 抽屉宽/高在 token 值与 0 之间切换,活动栏图标显激活态 | `drawer-expanded-state-spec.png` |
| 分屏 | center 区可水平/垂直分屏,各持文档 | `split-editor-state-spec.png` |
| 激活态 | 当前焦点面板边框/标签用 teal accent | STYLE-NOTES 状态规则 |

### 3.3 与运行时 docking 的边界

本计划定结构契约;实际拖拽停靠、窗口注册、承载切换在 `editor_ui/08`。本目录产出的是骨架资产(壳区域组合 `.zui`)与抽屉行为规范文档,喂给 08 的运行时实现。

## 4. 接口与数据结构草案(Rust)

```rust
pub struct WorkbenchSkeleton {
    pub top_toolbar: AssetRef,
    pub activity_rail_left: AssetRef,
    pub activity_rail_right: AssetRef,
    pub document_tabs: Vec<DocumentTab>,
    pub regions: [RegionBinding; 5], // 四角 + center,见 02
    pub bottom_output: AssetRef,
    pub status_bar: AssetRef,
}
pub enum DrawerState { Expanded, Collapsed }
```

## 5. 模块与文件落点

| 动作 | 文件 | 说明 |
| --- | --- | --- |
| 新增 | `zircon_editor/assets/ui/editor/components/workbench/shell/workbench_skeleton.zui` | 骨架组合 |
| 修改 | `workbench_main_band.zui` | 接入四角抽屉 + 分屏 center |
| 新增 | `docs/ui-and-layout/workbench-skeleton-contract.md` | 骨架与抽屉行为规范 |

## 6. 里程碑切片化

| # | 切片 | 涉及文件 | 验证命令 | 硬切换 |
| -- | --- | --- | --- | --- |
| S1 | 工作台骨架 + 抽屉区职责落地 | workbench_skeleton.zui / workbench-skeleton-contract.md | `cargo test -p zircon_editor --lib --locked` | 新建骨架,旧 main_band 接入 |
| S2 | 停靠语义(展开/折叠/分屏/激活) | workbench_main_band.zui | `cargo test -p zircon_editor --test integration_contracts --features integration-contracts --locked` | — |

## 7. 测试矩阵

- 骨架组合可加载,六区域齐全。
- 抽屉展开/折叠状态切换正确,活动栏激活态联动。
- center 分屏可水平/垂直,各持文档。

## 8. 风险与对策

- 风险:骨架与现有 11 module workspace 冲突。对策:骨架只定壳结构,模块工作区作为 center 文档接入,不改模块内部。

## 9. 完成定义

工作台六区域骨架契约落地,抽屉行为规范成文,主文档标签 + 活动栏语义对齐 JetBrains。

## 10. 边界约束

不实现运行时拖拽 docking(属 `editor_ui/08`);区域职责按 `index.md §1.1` 固定;不内嵌设计 PNG。

## 11. 参考实现对照(dev/ 源码锚点)

- `dev/Fyrox/editor`:编辑器壳(world viewer/inspector/asset browser 接线)。
- `dev/UnrealEngine/.../Slate/Public/Framework/Docking`:停靠语义参考。

## 12. 状态与产出记录

| 日期 | 切片 | 状态 | 产出/证据 | 后续项 |
| --- | --- | --- | --- | --- |
| 2026-06-23 | 03.S1 工作台骨架 + 抽屉区职责落地 | implemented-static-passed-editor-cargo-blocked | 已新增 `zircon_editor/src/ui/workbench/autolayout/workbench_skeleton.rs`、`zircon_editor/assets/ui/editor/components/workbench/shell/workbench_skeleton.zui` 与 `docs/ui-and-layout/workbench-skeleton-contract.md`;骨架固定 left-top/left-bottom/right-top/right-bottom/bottom/center 六区域和抽屉默认状态,并通过 02 的职责校验复用区域语义。scoped rustfmt、`git diff --check`、新模块债务扫描通过。 | 03.S2:将展开/折叠/分屏/激活态落到运行时 docking 与壳资产。`zircon_editor` Cargo gate 当前在下层 `zircon_runtime` render mesh import 编译漂移处阻塞,未到 editor 测试代码。 |
| 2026-06-23 | 03.S2 停靠语义(展开/折叠/分屏/激活) | implemented-static-passed-lower-ui-support-repaired-cargo-timeout | 已新增 `zircon_editor/src/ui/workbench/layout/layout_command_error.rs`,并把 `LayoutManager::apply(...)` / `attach_instance(...)` 的停靠命令失败从裸字符串收束为 `LayoutCommandError`;抽屉折叠清空 active view、活动栏激活抽屉 tab、center 文档分屏与 focus active-tab 语义由 `editor_layout_contracts.rs` 覆盖。scoped rustfmt、`git diff --check`、触及生产文件债务扫描、尾随空白扫描通过。 | 先前 focused `cargo test` / `cargo check` 均在约 604s 超时;23:06 复跑 focused test 因旧 target-dir `.fingerprint` 路径缺失失败;23:40 干净 target-dir 复跑暴露下层 `zircon_runtime::ui::template::asset::compiler::style_apply` split 漂移(`mui_slot_name` 未重新暴露、slot helper import 私有),已最小修复为 `style_apply` 父模块 re-export + `slot_contract` owner 保持实现。随后 `cargo check -p zircon_runtime --lib --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-editor-0623-clean-2309` 606s 超时无诊断,仍未取得 Cargo 通过;后续继续从该下层支撑验证向上复跑。 |
