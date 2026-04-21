---
related_code:
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/ui/module.rs
  - zircon_runtime/src/ui/template/mod.rs
  - zircon_runtime/src/ui/template/asset/mod.rs
  - zircon_runtime/src/ui/template/asset/document.rs
  - zircon_runtime/src/ui/template/asset/compiler/mod.rs
  - zircon_runtime/src/ui/template/asset/legacy.rs
  - zircon_runtime/src/ui/layout/constraints.rs
  - zircon_runtime/src/ui/layout/geometry.rs
  - zircon_runtime/src/ui/layout/pass/mod.rs
  - zircon_runtime/src/ui/layout/scroll.rs
  - zircon_runtime/src/ui/layout/virtualization.rs
  - zircon_runtime/src/ui/dispatch/mod.rs
  - zircon_runtime/src/ui/tree/node/mod.rs
  - zircon_runtime/src/ui/tree/hit_test.rs
  - zircon_runtime/src/ui/surface/mod.rs
  - zircon_editor/src/ui/asset_editor/mod.rs
  - zircon_editor/src/ui/workbench/autolayout/mod.rs
  - zircon_editor/src/ui/slint_host/tab_drag.rs
  - zircon_editor/ui/workbench.slint
implementation_files:
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/ui/module.rs
  - zircon_runtime/src/ui/template/mod.rs
  - zircon_runtime/src/ui/template/asset/mod.rs
  - zircon_runtime/src/ui/template/asset/document.rs
  - zircon_runtime/src/ui/template/asset/compiler/mod.rs
  - zircon_runtime/src/ui/template/asset/legacy.rs
  - zircon_runtime/src/ui/layout/constraints.rs
  - zircon_runtime/src/ui/layout/geometry.rs
  - zircon_runtime/src/ui/layout/pass/mod.rs
  - zircon_runtime/src/ui/layout/scroll.rs
  - zircon_runtime/src/ui/layout/virtualization.rs
  - zircon_runtime/src/ui/dispatch/mod.rs
  - zircon_runtime/src/ui/tree/node/mod.rs
  - zircon_runtime/src/ui/tree/hit_test.rs
  - zircon_runtime/src/ui/surface/mod.rs
  - zircon_editor/src/ui/asset_editor/mod.rs
  - zircon_editor/src/ui/workbench/autolayout/mod.rs
  - zircon_editor/src/ui/slint_host/tab_drag.rs
  - zircon_editor/ui/workbench.slint
plan_sources:
  - user: 2026-04-14 实现运行时/编辑器共享 UI 布局与事件系统架构计划
  - user: 2026-04-15 继续实现 ScrollableBox、scroll state、visible range invalidation 和 pointer dispatcher
  - user: 2026-04-15 继续把更完整的 editor shell pointer hit-test / dock target route 往 shared core 迁移
  - user: 2026-04-20 zircon_editor UI 回迁 + 树形 TOML cutover 继续清理 root 出口与旧 UI crate 文档路径
  - .codex/plans/布局系统.md
  - .codex/plans/Zircon 运行时编辑器共享 UI 布局与事件系统架构计划.md
  - .codex/plans/全系统重构方案.md
tests:
  - zircon_runtime/src/ui/tests/shared_core.rs
  - zircon_runtime/src/ui/tests/asset.rs
  - zircon_editor/src/tests/ui/activity/mod.rs
  - zircon_editor/src/tests/ui/activity/window_descriptor.rs
  - zircon_editor/src/tests/ui/activity/route.rs
  - cargo test -p zircon_editor --lib --locked
  - cargo test -p zircon_runtime --locked
  - cargo check --workspace --locked
doc_type: category-index
---

# UI And Layout

## Purpose

本目录记录运行时 UI 与 editor shell 共用的布局、树结构、命中和 surface 权威模型，重点回答三件事：

- 哪些布局/几何/命中语义已经属于 `zircon_runtime::ui`
- editor workbench 还保留哪些 editor-only 壳体职责
- Slint 和未来其他宿主如何只做适配层，而不是再次成为布局真源

## Documents

- [Shared UI Core Foundation](./shared-ui-core-foundation.md): `zircon_runtime::ui` 新增的共享约束类型、measure/arrange pass、`HorizontalBox`/`VerticalBox`/`ScrollableBox` 容器、retained tree、dirty/invalidation、scroll state、命中索引、pointer/focus/navigation route、统一 pointer dispatcher、虚拟化窗口工具，以及 `zircon_editor` workbench autolayout 的复用边界。
  这一轮还补上了显式 `Container` / `Overlay` / `Space` 共享容器名、pointer button payload、capture 后移出命中范围仍保持派发的底层语义、第一条 Slint viewport pointer/scroll -> shared `UiSurface + UiPointerDispatcher` 的宿主接线，以及 editor shell drag target hit-test / dock target route 的 host-owned shared bridge。
- [Shared UI Template Runtime](./shared-ui-template-runtime.md): `zircon_runtime::ui::template` 的 TOML 文档模型、component/slot 节点语义、模板校验规则、运行时实例展开、shared `UiTree` / `UiSurface` 桥接，以及 `attributes.layout` 到 shared `BoxConstraints` / `Anchor` / `Pivot` / `Position` / container config 的显式映射；它是后续 shared layout 求解和 Slint host projection 之前的模板真源层。
- [UI Asset Documents And Editor Protocol](./ui-asset-documents-and-editor-protocol.md): `zircon_runtime::ui::template::asset` 当前 tree-shaped `.ui.toml` authority、flat-to-tree 一次性迁移器、shared loader/compiler/surface builder，以及 editor/runtime 如何继续共用同一条资产消费链路。
- [UI Module Boundary Refactor](./ui-module-boundary-refactor.md): `binding/model`、`event_ui/manager`、`layout/pass`、`template/build`、`tree/node` 从混合单文件重构成 folder-backed subtree 后的职责边界，约束后续 shared UI runtime 继续演化时的文件级纪律。
- [Editor Host Final Cleanup](./editor-host-final-cleanup.md): `Final cleanup` 阶段对 editor host 剩余 legacy seam 的删除记录，覆盖 drawer extent root binding、menu button frame host setter/binding、floating-window drag/document-tab 生产路径里的 geometry outer-frame fallback，以及 root-shell projection / callback sizing helper 对 legacy geometry 的最后兜底。

## Related Files

- `zircon_runtime/src/ui/mod.rs`
- `zircon_runtime/src/ui/layout/constraints.rs`
- `zircon_runtime/src/ui/layout/geometry.rs`
- `zircon_runtime/src/ui/layout/pass/mod.rs`
- `zircon_runtime/src/ui/layout/scroll.rs`
- `zircon_runtime/src/ui/layout/virtualization.rs`
- `zircon_runtime/src/ui/template/asset/mod.rs`
- `zircon_runtime/src/ui/dispatch/mod.rs`
- `zircon_runtime/src/ui/tree/node/mod.rs`
- `zircon_runtime/src/ui/tree/hit_test.rs`
- `zircon_runtime/src/ui/surface/mod.rs`
- `zircon_editor/src/ui/workbench/autolayout/mod.rs`

## Current Scope

当前文档只覆盖这次共享 UI core 计划已经落地的第一段：

- 共享约束和几何类型进入 `zircon_runtime::ui`
- retained tree、dirty 标记、clip 链命中、scroll state 与 surface/render extract scaffolding 进入 `zircon_runtime::ui`
- 基础 measure/arrange pass、`Container` / `Overlay` / `Space` / `HorizontalBox` / `VerticalBox` / `ScrollableBox` 共享容器、pointer/focus/navigation route、统一 pointer dispatcher 和虚拟化窗口计算进入 `zircon_runtime::ui`
- `binding/model`、`event_ui/manager`、`layout/pass`、`template/build`、`tree/node` 已经按 folder-backed subtree 重新分层，入口 `mod.rs` 保持导出层，不再承载混合实现
- `template/asset` 已经承接正式 `layout/widget/style` 资产 AST、tree-shaped `.ui.toml` authority、一次性 flat-to-tree migration adapter，以及编译到 `UiTemplateInstance` / `UiSurface` 的 shared 真源链路
- `zircon_runtime/src/ui/mod.rs` 进一步收缩为 crate 导航层，`UiConfig`、`UI_MODULE_NAME` 和 `module_descriptor` 已下沉到 `zircon_runtime/src/ui/module.rs`
- shared pointer button payload、capture 持续派发语义，以及第一条 editor host viewport pointer/scroll shared bridge 已经接到 `zircon_runtime::ui`
- editor shell drag target route 已经开始通过 host-owned `WorkbenchDragTargetBridge` 复用 shared `UiSurface + UiPointerDispatcher`
- editor workbench layout 数学改为直接复用共享 solver 和共享 frame/size/constraint 类型
- `zircon_editor::ui::asset_editor` 已经承接 `editor.ui_asset` 的窗口协议、session 和 source/preview authoring 入口，为后续更深的 tree-native editor refactor 保留稳定 id、mode 和 route 载荷
- `WorkbenchLayout` 继续只做 editor 拓扑与持久化，不再承担底层基础类型的定义权

后续如果继续落地更完整的容器族、grid/flow 虚拟化、editor/runtime 宿主接线、world-space UI 和 runtime ECS bridge，可以在本目录继续追加更细分的实现文档。
