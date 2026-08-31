---
related_code:
  - dev/penpot/plugins/apps/zircon-zui-plugin
  - zircon_runtime/src/ui/v2/loader.rs
  - zircon_runtime/src/ui/v2/compiler.rs
  - zircon_editor/src/ui/workbench/mod.rs
  - zircon_editor/assets/ui/editor/layout/shell_regions.toml
  - zircon_hub/src/tauri_app/view_model.rs
  - zircon_hub/web/src/App.tsx
implementation_files:
  - dev/penpot/plugins/apps/zircon-zui-plugin/src/plugin.ts
  - dev/penpot/plugins/apps/zircon-zui-plugin/src/cli.ts
  - zircon_editor/src/ui/workbench/mod.rs
  - zircon_hub/src/tauri_app/view_model.rs
tests:
  - dev/penpot/plugins/apps/zircon-zui-plugin/src/bridge/penpot-asset.spec.ts
  - zircon_runtime/tests/zui_penpot_bridge_contract.rs
  - zircon_editor/tests/integration_contracts.rs
  - zircon_hub/tests/tauri_react_shell_contract.rs
plan_sources:
  - "user: 2026-08-31 以 dev/penpot 作为界面设计参考，为 ZirconEngine 编写完整界面设计计划"
  - "user: 2026-08-31 先兼容当前 Penpot，实现 .zui 与 Penpot 资产互相转换，再推进 ZirconEngine 自举布局"
  - docs/plans/mvp/index.md
  - docs/plans/zircon_editor/editor_layout/index.md
  - docs/plans/zircon_editor/editor_ui/index.md
  - docs/plans/zircon_editor/editor/06-ui-extension-framework.md
  - docs/plans/zircon_hub/index.md
  - docs/plans/designment/01-penpot-inspired-interface-design.md
  - docs/plans/designment/02-milestone-execution-and-evidence.md
  - dev/penpot/README.md
  - dev/penpot/frontend/src/app/main/ui/workspace.cljs
  - dev/penpot/frontend/src/app/main/ui/workspace/sidebar.cljs
  - dev/penpot/frontend/src/app/main/ui/workspace/top_toolbar.cljs
  - dev/penpot/frontend/src/app/main/ui/ds.cljs
doc_type: category-index
status: in_progress
last_refined: 2026-08-31
---

# Penpot 参考的 ZirconEngine 界面设计计划索引

本目录是 ZirconEngine 的跨产品界面设计编排计划。它把 `dev/penpot` 作为设计思路和交互模式的参考源，把现有 `zircon_editor`、`zircon_runtime`、`zircon_runtime_interface` 与 `zircon_hub` 的架构边界作为实现约束。目标是形成一套可资产化、可测试、可渐进交付的编辑器与 Hub 界面语言，而不是复制 Penpot 的网页实现或视觉像素。

## 使用方式

| 文档 | 用途 | 状态 |
|---|---|---|
| [01-penpot-inspired-interface-design.md](./01-penpot-inspired-interface-design.md) | 唯一设计决策源：模式映射、界面架构、令牌、组件、状态、交互流和里程碑 | design-ready |
| [02-milestone-execution-and-evidence.md](./02-milestone-execution-and-evidence.md) | 里程碑 owner、MVP gate、validation manifest 模板和证据文件登记 | in_progress |

执行时按主计划的 `M0` 至 `M9` 依赖顺序推进。每个里程碑必须先完成设计/契约切片，再进入该里程碑指定的测试阶段；不得以静态截图代替功能验收，也不得绕过 MVP 的 F0-F5 门槛。

## 设计权威

发生冲突时按以下优先级裁决：

1. 当前源码、稳定接口和已通过的聚焦测试。
2. `docs/plans/mvp/index.md` 的产品退出条件。
3. `docs/plans/zircon_editor/editor_layout` 与 `docs/plans/zircon_editor/editor_ui` 的布局、渲染和组件契约。
4. 本目录的跨产品设计编排。
5. `dev/penpot` 的可迁移模式参考。

Penpot 只提供可验证的设计模式：全视口工作区、语义侧栏、令牌优先、组件组合、显式状态、键盘可达、协作反馈和可追踪持久化。它不改变 ZirconEngine 的 Rust 所有权、运行时边界或 Hub 与编辑器的进程边界。

## 现状摘要

- `.zui` v2 与 Penpot semantic asset 的可逆 bridge、CLI 和插件面板已经实现；A0 已验证，A1 仍缺本机真实 Penpot 容器中的 import/edit/export 操作证据，A2 的 Rust loader/compiler 测试正在等待共享 Cargo 通道。
- 编辑器源码已有工作台区域、布局 preset、`.zui` 资产、Taffy 约束布局、GPU command stream、保留式宿主和命令/撤销方向，但相关 owner 计划仍有 open failure，尚未等同于产品验收；本计划负责把它们收敛成一致的产品体验，不重复建设运行时 UI 基础设施。
- Hub 源码已采用 Tauri + React/MUI 和 Rust DTO 单向推送，但 Hub 子计划、Rust gate 和截图仍未全部闭合；本计划只补充 Penpot 风格的信息架构、状态反馈和视觉契约，不把 Hub 改造成编辑器内的 Slint 界面。
- MVP 的核心闭环是创建/打开项目、加载资产、渲染基础场景、选择实体、通过 Inspector 命令修改、保存并重开。高级协作、插件市场和完整动画工作流必须受 MVP 及其 owner 计划的依赖门控制。

## 产出记录

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

A0-A2 的当前输出记录位于 [02 companion](./02-milestone-execution-and-evidence.md) 及其 `evidence/`、`manifests/` 子目录；M0-M9 在进入执行前仍保持空表。
