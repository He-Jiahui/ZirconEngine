---
related_code:
  - zircon_editor/src/ui/workbench/mod.rs
  - zircon_editor/src/ui/workbench/autolayout/mod.rs
  - zircon_editor/src/ui/workbench/layout_preset.rs
  - zircon_editor/src/ui/host/layout_persistence.rs
  - zircon_editor/src/ui/host/layout_commands.rs
  - zircon_editor/src/tests/workbench/layout/layout_preset_persistence.rs
  - zircon_editor/src/ui/host/module.rs
design_references:
  - docs/ui-and-layout/ai-workbench-style/prototype/README.md
  - docs/ui-and-layout/editor-workbench-designs/drawer-expanded-state-spec.png
plan_sources:
  - docs/plans/zircon_editor/editor_layout/03-jetbrains-docking-workbench.md
status: in_progress
---
# 04 布局预设与持久化

## 1. 目标

把工作台布局做成**可保存、可恢复、可按场景切换的布局档案(layout preset)**:作者/用户在 Authoring / Review / Focus / Debug 之间一键切换,且每个页面/每个用户的布局状态(抽屉宽、展开态、分屏)被持久化。承接 `prototype/README.md` 的布局预设映射。

## 2. 现状(按代码核实)

### 2.1 已存在的设施

- 壳 autolayout 模块存在(承接 02/03)。
- web 原型已确立布局预设概念(Authoring/Review/Focus/Debug → 保留布局档案)。

### 2.2 真实缺口

- S1 已补布局档案数据形态与内置 Authoring/Review/Focus/Debug 预设。
- S2 已补页面/用户维度持久化 store、版本校验/Authoring 回退、host 保存/恢复接口与主页面切换默认用户自动保存/恢复路径。

## 3. 设计

### 3.1 布局预设档案

| 预设 | 语义 | 区域配置 |
| --- | --- | --- |
| Authoring | 默认创作 | 四角抽屉全开,center 单文档 |
| Review | 审阅 | 右侧抽屉为主,bottom 输出展开 |
| Focus | 专注 | 抽屉全折叠,center 全宽 |
| Debug | 调试 | bottom 诊断/时间轴最大化 |

每个预设记录:各区域抽屉状态(展开/折叠)、约束 token 覆盖值、center 分屏布局。

### 3.2 持久化

- 按页面(每个主文档标签可有独立布局)。
- 按用户(Hub TOML 配置层之外的编辑器布局状态)。
- 切换页面时恢复该页面上次布局;切换用户/工程时恢复对应档案。

## 4. 接口与数据结构草案(Rust)

```rust
pub struct LayoutPreset {
    pub name: String,                    // Authoring/Review/Focus/Debug
    pub drawer_states: [DrawerState; 5], // 四角 + bottom
    pub size_overrides: Vec<(ConstraintTokenName, f32)>,
    pub center_split: CenterSplitLayout,
}
pub fn apply_preset(skeleton: &mut WorkbenchSkeleton, preset: &LayoutPreset);
pub fn persist_layout(page: DocumentId, layout: &LayoutPreset);
pub fn restore_layout(page: DocumentId) -> Option<LayoutPreset>;
```

## 5. 模块与文件落点

| 动作 | 文件 | 说明 |
| --- | --- | --- |
| 新增 | `zircon_editor/src/ui/workbench/layout_preset.rs` | 预设 + 持久化接口 |
| 新增 | `zircon_editor/assets/ui/editor/layout/presets.v2.ui.toml` | 内置四预设 |

## 6. 里程碑切片化

| # | 切片 | 涉及文件 | 验证命令 | 硬切换 |
| -- | --- | --- | --- | --- |
| S1 | 布局预设档案 + 切换 | layout_preset.rs / presets.v2.ui.toml | `cargo test -p zircon_editor --lib --locked` | 新建 |
| S2 | 持久化(按页面/用户) | layout_preset.rs | `cargo test -p zircon_editor --lib --locked` | — |

## 7. 测试矩阵

- 四预设可加载并正确应用到骨架。
- 切换预设后区域状态符合预设定义。
- 持久化后重载恢复上次布局。

## 8. 风险与对策

- 风险:持久化布局与新版骨架不兼容。对策:档案带版本号,加载时校验,失配回退默认 Authoring。

## 9. 完成定义

四预设可一键切换,布局按页面/用户持久化恢复。

## 10. 边界约束

不改骨架结构(属 03);预设只覆盖抽屉状态与 token 值;不持久化模块内部数据。

## 11. 参考实现对照(dev/ 源码锚点)

- `dev/UnrealEngine/.../Slate/Public/Framework/Docking`:布局保存/恢复参考。

## 12. 状态与产出记录

| 日期 | 切片 | 状态 | 产出/证据 | 后续项 |
| --- | --- | --- | --- | --- |
| 2026-06-23 | 04.S1 布局预设档案 + 切换声明 | implemented-static-passed-editor-cargo-blocked | 已新增 `zircon_editor/src/ui/workbench/layout_preset.rs` 与 `zircon_editor/assets/ui/editor/layout/presets.v2.ui.toml`;内置 Authoring/Review/Focus/Debug 四档案,记录抽屉状态、尺寸 token 覆盖和 center 分屏语义。scoped rustfmt、`git diff --check`、新模块债务扫描通过。 | 04.S2:补页面/用户持久化、版本校验与失配回退。`zircon_editor` Cargo gate 当前在下层 `zircon_runtime` render mesh import 编译漂移处阻塞,未到 editor 测试代码。 |
| 2026-06-23 | 04.S2 持久化(按页面/用户) | implemented-focused-passed | `LayoutPresetPersistenceStore` 按 `(user_id,page_id)` 保存 `LayoutPreset`,持久化文档带 `LAYOUT_PRESET_PERSISTENCE_VERSION`;缺失或版本不匹配时回退 Authoring。`LayoutPreset::capture_from_layout(...)` 只采集抽屉 mode、抽屉 extent/token 覆盖与 center split 形状,不写入视图实例 ID 或模块 payload;`apply_to_layout(...)` 恢复抽屉状态、尺寸和 split 形状。`EditorManager`/`EditorUiHost` 新增页面布局保存/恢复接口,`ActivateMainPage` 在 host 边界用 default 用户保存旧页并恢复目标页。验证:`cargo test -p zircon_editor --lib layout_preset_persistence --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-editor-0623-clean-2309 --message-format short --color never -- --test-threads=1 --nocapture` 2/2 通过;scoped rustfmt、diff check、尾随空白扫描和 editor layout 生产债务扫描通过。复跑过程中暴露下层 `zircon_runtime::ui::component::catalog::editor_showcase` helper split 后漏导 `numeric`;已最小补导入,不改 catalog 行为。 | 04.S2 focused path 关闭;后续进入 05.S2 其余页面模板、06.S2 设计对齐验收,并保留 03.S2 全量 editor-layout Cargo 与旧 shell/module token hard cutover 的验证债。 |
