---
related_code:
  - zircon_editor/src/ui/workbench/floating_window.rs
  - zircon_editor/src/ui/workbench/mod.rs
  - zircon_editor/tests/integration_contracts.rs
  - zircon_editor/tests/integration_contracts/floating_window_design_parity.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/layouts/windows
  - zircon_runtime_interface/src/ui/surface/mod.rs
  - zircon_editor/assets/ui/editor/components/workbench/primitives/feedback
  - zircon_editor/assets/ui/editor/components/workbench/floating/command_palette.zui
  - zircon_editor/assets/ui/editor/components/workbench/floating/preferences.zui
design_references:
  - docs/ui-and-layout/editor-workbench-designs/command-palette-window-spec.png
  - docs/ui-and-layout/editor-workbench-designs/preferences-window-workbench.png
  - docs/ui-and-layout/editor-workbench-designs/STYLE-NOTES.md
plan_sources:
  - docs/plans/zircon_editor/editor_layout/03-jetbrains-docking-workbench.md
  - docs/plans/zircon_editor/editor_layout/05-page-layout-templates.md
status: implemented-focused-passed-visual-host-pending
---
# 06 浮动窗口与设计对齐验收

## 1. 目标

定义工作台之上的**浮动窗口浮层规则**(命令面板、偏好、独立编辑器),并对整套布局/设计语言做**与设计图对齐的验收**收口。承接 `command-palette-window-spec.png` / `preferences-window-workbench.png` 等浮窗设计图。这是本目录的收口子计划:验证 01-05 已定的语言与骨架在浮窗层依然一致。

## 2. 现状(按代码核实)

### 2.1 已存在的设施

- Rust-owned retained host + window registry(承接 `editor_ui/08`)。
- 壳浮层/弹出原语(feedback primitives)存在。

### 2.2 真实缺口

- 缺浮动窗口浮层规则(浮层级、模态/非模态、相对工作台定位)。
- 缺设计对齐验收基线(浮窗用同一 token/控件规则)。

## 3. 设计

### 3.1 浮动窗口类型

| 窗口 | 浮层规则 | 设计图 |
| --- | --- | --- |
| 命令面板 | 顶层居中浮层,非阻塞工作台,键盘驱动 | `command-palette-window-spec` |
| 偏好 | 模态浮层,工作台之上,分类导航 + 内容区 | `preferences-window-workbench` |
| 独立编辑器 | 可分离为独立窗口,复用页面模板 | 各 `floating-window` |

### 3.2 浮层与设计一致性

- 浮窗 chrome 复用 01 的 token、控件规则(圆角矩形/1px 边框/扁平态/无阴影辉光)。
- 浮窗内部布局复用 02 的区域语义(偏好窗 = 左导航 + 右内容,套用 left/center)。
- 命令面板键盘语义与组件原型一致。

### 3.3 设计对齐验收

收口验收:逐页面/逐浮窗对照设计图,核对(a)区域职责符合 §1.1,(b)色值=token,(c)控件规格符合 STYLE-NOTES,(d)状态优先级一致。验收清单写进设计语言契约文档(01)的附录。

## 4. 接口与数据结构草案(Rust)

```rust
pub enum FloatingWindowKind { CommandPalette, Preferences, DetachedEditor }
pub struct FloatingWindow {
    pub kind: FloatingWindowKind,
    pub modal: bool,
    pub layer: FloatingLayer,
    pub content: AssetRef, // 复用页面模板/组件
}
```

## 5. 模块与文件落点

| 动作 | 文件 | 说明 |
| --- | --- | --- |
| 新增 | `zircon_editor/assets/ui/editor/components/workbench/floating/command_palette.zui` | 命令面板 |
| 新增 | `zircon_editor/assets/ui/editor/components/workbench/floating/preferences.zui` | 偏好窗 |
| 修改 | `docs/ui-and-layout/design-language-contract.md` | 追加设计对齐验收清单 |

## 6. 里程碑切片化

| # | 切片 | 涉及文件 | 验证命令 | 硬切换 |
| -- | --- | --- | --- | --- |
| S1 | 浮动窗口浮层规则 + 命令面板/偏好 | command_palette.zui / preferences.zui | `cargo test -p zircon_editor --lib --locked` | — |
| S2 | 设计对齐验收 + 收口 | design-language-contract.md | `cargo test -p zircon_editor --test integration_contracts --features integration-contracts --locked` | — |

## 7. 测试矩阵

- 命令面板顶层居中、键盘可驱动、不阻塞工作台。
- 偏好窗模态、左导航右内容、复用区域语义。
- 浮窗 chrome 全部用 token,无裸色值,无禁用视觉。
- 验收清单逐项通过。

## 8. 风险与对策

- 风险:浮窗各自为政破坏一致性。对策:浮窗强制复用 01 token + 02 区域语义,验收清单作为门禁。

## 9. 完成定义

浮动窗口浮层规则成文,命令面板/偏好落地,设计对齐验收清单通过,01-06 语言闭环。

## 10. 边界约束

不实现窗口分离运行时(属 `editor_ui/08`);浮窗复用既有 token/区域/组件;不内嵌设计 PNG。

## 11. 参考实现对照(dev/ 源码锚点)

- `dev/UnrealEngine/.../Slate/Public/Framework/Application`:命令面板/弹出层参考。
- `dev/Fyrox/fyrox-ui/src/popup`:浮层参考。

## 12. 状态与产出记录

| 日期 | 切片 | 状态 | 产出/证据 | 后续项 |
| --- | --- | --- | --- | --- |
| 2026-06-23 | 06.S1 浮动窗口浮层规则 + 命令面板/偏好 | implemented-static-passed-editor-cargo-blocked | 已新增 `zircon_editor/src/ui/workbench/floating_window.rs`、`zircon_editor/assets/ui/editor/components/workbench/floating/command_palette.zui` 与 `zircon_editor/assets/ui/editor/components/workbench/floating/preferences.zui`;命令面板、偏好和独立编辑器窗口声明复用 01 token 与 02 区域语义,设计对齐清单同步进 `docs/ui-and-layout/design-language-contract.md`。scoped rustfmt、`git diff --check`、新模块债务扫描通过。 | 06.S2:运行设计对齐验收,补齐截图/像素/交互证据并收口 01-06。`zircon_editor` Cargo gate 当前在下层 `zircon_runtime` render mesh import 编译漂移处阻塞,未到 editor 测试代码。 |
| 2026-06-23 | 06.S2 设计对齐验收 + 收口 | implemented-focused-passed-visual-host-pending | `FloatingWindowDesignContract` 明确命令面板、偏好和独立编辑器的 layer、modal、placement、content layout 与 interaction mode;`floating_window_design_parity.rs` 直接解析真实 `command_palette.zui` / `preferences.zui` 资产,验证 tokenized flat chrome、无裸 hex/gradient/shadow/glow/blur、1px 低圆角边框、命令面板顶层键盘布局与偏好窗左导航/右内容结构。验证:`cargo check -p zircon_runtime --lib --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-runtime-state-reducer-0623 --message-format short --color never` 通过;`cargo test -p zircon_editor --test integration_contracts --features integration-contracts floating_window_design_parity --offline --jobs 1 --target-dir E:\cargo-targets\zircon-editor-layout-runtime-state-reducer-0623 --message-format short --color never -- --test-threads=1 --nocapture` 4/4 通过;scoped rustfmt/diff/尾随空白/生产债务扫描通过。 | 真实 retained-host 截图/像素比对仍待有稳定窗口 harness 后补;01.S2 历史 shell/module 资产裸色 hard cutover 与 03.S2 更宽 Cargo 复验债继续保留。 |
