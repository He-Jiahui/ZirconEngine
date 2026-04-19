---
related_code:
  - zircon_ui/src/lib.rs
  - zircon_ui/src/module/mod.rs
  - zircon_ui/src/module/ui_config.rs
  - zircon_ui/src/module/ui_module_descriptor.rs
  - zircon_ui/src/module/ui_module_name.rs
  - zircon_ui/src/template/mod.rs
  - zircon_ui/src/template/asset/mod.rs
  - zircon_ui/src/template/asset/document.rs
  - zircon_ui/src/template/asset/compiler.rs
  - zircon_ui/src/template/asset/legacy.rs
  - zircon_ui/src/layout/constraints.rs
  - zircon_ui/src/layout/geometry.rs
  - zircon_ui/src/layout/pass/mod.rs
  - zircon_ui/src/layout/scroll.rs
  - zircon_ui/src/layout/virtualization.rs
  - zircon_ui/src/dispatch/mod.rs
  - zircon_ui/src/tree/node/mod.rs
  - zircon_ui/src/tree/hit_test.rs
  - zircon_ui/src/surface/mod.rs
  - zircon_editor/src/ui/ui_asset_editor.rs
  - zircon_editor/src/ui/workbench/autolayout/mod.rs
  - zircon_editor/src/ui/slint_host/tab_drag.rs
  - zircon_editor/ui/workbench.slint
implementation_files:
  - zircon_ui/src/lib.rs
  - zircon_ui/src/module/mod.rs
  - zircon_ui/src/module/ui_config.rs
  - zircon_ui/src/module/ui_module_descriptor.rs
  - zircon_ui/src/module/ui_module_name.rs
  - zircon_ui/src/template/mod.rs
  - zircon_ui/src/template/asset/mod.rs
  - zircon_ui/src/template/asset/document.rs
  - zircon_ui/src/template/asset/compiler.rs
  - zircon_ui/src/template/asset/legacy.rs
  - zircon_ui/src/layout/constraints.rs
  - zircon_ui/src/layout/geometry.rs
  - zircon_ui/src/layout/pass/mod.rs
  - zircon_ui/src/layout/scroll.rs
  - zircon_ui/src/layout/virtualization.rs
  - zircon_ui/src/dispatch/mod.rs
  - zircon_ui/src/tree/node/mod.rs
  - zircon_ui/src/tree/hit_test.rs
  - zircon_ui/src/surface/mod.rs
  - zircon_editor/src/ui/ui_asset_editor.rs
  - zircon_editor/src/ui/workbench/autolayout/mod.rs
  - zircon_editor/src/ui/slint_host/tab_drag.rs
  - zircon_editor/ui/workbench.slint
plan_sources:
  - user: 2026-04-14 实现运行时/编辑器共享 UI 布局与事件系统架构计划
  - user: 2026-04-15 继续实现 ScrollableBox、scroll state、visible range invalidation 和 pointer dispatcher
  - user: 2026-04-15 继续把更完整的 editor shell pointer hit-test / dock target route 往 shared core 迁移
  - user: 2026-04-16 全仓库模块边界拆分与根入口去逻辑化
  - .codex/plans/布局系统.md
  - .codex/plans/Zircon 运行时编辑器共享 UI 布局与事件系统架构计划.md
  - .codex/plans/全系统重构方案.md
tests:
  - zircon_ui/src/tests/shared_core.rs
  - zircon_ui/src/tests/asset.rs
  - zircon_editor/src/tests/ui/activity.rs
  - zircon_editor/tests/workbench_autolayout.rs
  - cargo test -p zircon_ui shared_core -- --nocapture
  - cargo test -p zircon_ui --lib --locked tests::asset
  - cargo test -p zircon_editor --lib --locked tests::ui::activity
  - cargo test -p zircon_ui --offline --verbose
  - cargo test -p zircon_ui --offline
  - rustc --edition=2024 --test zircon_editor/tests/workbench_slint_shell.rs -o <temp> && <temp> --nocapture
  - rustc --edition=2024 --test <temp-workbench-drag-target-prefix-test.rs> --extern zircon_ui=<target/debug/deps/libzircon_ui-*.rlib> -L dependency=target/debug/deps -o <temp> && <temp> --nocapture
  - cargo test -p zircon_editor --test workbench_autolayout -- --nocapture
  - cargo test --workspace --locked --verbose
doc_type: category-index
---

# UI And Layout

## Purpose

本目录记录运行时 UI 与 editor shell 共用的布局、树结构、命中和 surface 权威模型，重点回答三件事：

- 哪些布局/几何/命中语义已经属于 `zircon_ui`
- editor workbench 还保留哪些 editor-only 壳体职责
- Slint 和未来其他宿主如何只做适配层，而不是再次成为布局真源

## Documents

- [Shared UI Core Foundation](./shared-ui-core-foundation.md): `zircon_ui` 新增的共享约束类型、measure/arrange pass、`HorizontalBox`/`VerticalBox`/`ScrollableBox` 容器、retained tree、dirty/invalidation、scroll state、命中索引、pointer/focus/navigation route、统一 pointer dispatcher、虚拟化窗口工具，以及 `zircon_editor` workbench autolayout 的复用边界。
  这一轮还补上了显式 `Container` / `Overlay` / `Space` 共享容器名、pointer button payload、capture 后移出命中范围仍保持派发的底层语义、第一条 Slint viewport pointer/scroll -> shared `UiSurface + UiPointerDispatcher` 的宿主接线，以及 editor shell drag target hit-test / dock target route 的 host-owned shared bridge。
- [Shared UI Template Runtime](./shared-ui-template-runtime.md): `zircon_ui::template` 的 TOML 文档模型、component/slot 节点语义、模板校验规则、运行时实例展开、shared `UiTree` / `UiSurface` 桥接，以及 `attributes.layout` 到 shared `BoxConstraints` / `Anchor` / `Pivot` / `Position` / container config 的显式映射；它是后续 shared layout 求解和 Slint host projection 之前的模板真源层。
- [UI Asset Documents And Editor Protocol](./ui-asset-documents-and-editor-protocol.md): `zircon_ui::template::asset` 的 `layout/widget/style` TOML AST、selector stylesheet、component/reference/slot 编译器、legacy template adapter、slot-aware shared layout bridge，以及 shared asset model 如何移交给 `zircon_asset` 正式资产注册和 `zircon_editor` 宿主会话。
- [UI Module Boundary Refactor](./ui-module-boundary-refactor.md): `binding/model`、`event_ui/manager`、`layout/pass`、`template/bridge`、`tree/node` 从混合单文件重构成 folder-backed subtree 后的职责边界，约束后续 `zircon_ui` 继续演化时的文件级纪律。
- [Editor Host Final Cleanup](./editor-host-final-cleanup.md): `Final cleanup` 阶段对 editor host 剩余 legacy seam 的删除记录，覆盖 drawer extent root binding、menu button frame host setter/binding、floating-window drag/document-tab 生产路径里的 geometry outer-frame fallback，以及 root-shell projection / callback sizing helper 对 legacy geometry 的最后兜底。

## Related Files

- `zircon_ui/src/lib.rs`
- `zircon_ui/src/layout/constraints.rs`
- `zircon_ui/src/layout/geometry.rs`
- `zircon_ui/src/layout/pass/mod.rs`
- `zircon_ui/src/layout/scroll.rs`
- `zircon_ui/src/layout/virtualization.rs`
- `zircon_ui/src/template/asset/mod.rs`
- `zircon_ui/src/dispatch/mod.rs`
- `zircon_ui/src/tree/node/mod.rs`
- `zircon_ui/src/tree/hit_test.rs`
- `zircon_ui/src/surface/mod.rs`
- `zircon_editor/src/ui/workbench/autolayout/mod.rs`

## Current Scope

当前文档只覆盖这次共享 UI core 计划已经落地的第一段：

- 共享约束和几何类型进入 `zircon_ui`
- retained tree、dirty 标记、clip 链命中、scroll state 与 surface/render extract scaffolding 进入 `zircon_ui`
- 基础 measure/arrange pass、`Container` / `Overlay` / `Space` / `HorizontalBox` / `VerticalBox` / `ScrollableBox` 共享容器、pointer/focus/navigation route、统一 pointer dispatcher 和虚拟化窗口计算进入 `zircon_ui`
- `binding/model`、`event_ui/manager`、`layout/pass`、`template/bridge`、`tree/node` 已经按 folder-backed subtree 重新分层，入口 `mod.rs` 保持导出层，不再承载混合实现
- `template/asset` 已经开始承接正式 `layout/widget/style` 资产 AST、import registry、selector stylesheet、legacy adapter 和编译到 `UiTemplateInstance` / `UiSurface` 的 shared 真源链路
- `zircon_ui/src/lib.rs` 进一步收缩为 crate 导航层，`UiConfig`、`UI_MODULE_NAME` 和 `module_descriptor` 已下沉到 `zircon_ui/src/module/`
- shared pointer button payload、capture 持续派发语义，以及第一条 editor host viewport pointer/scroll shared bridge 已经接到 `zircon_ui`
- editor shell drag target route 已经开始通过 host-owned `WorkbenchDragTargetBridge` 复用 shared `UiSurface + UiPointerDispatcher`
- editor workbench layout 数学改为直接复用共享 solver 和共享 frame/size/constraint 类型
- `zircon_editor::ui` 已经新增 `editor.ui_asset` 窗口协议入口，为后续 `UI Asset Editor` activity window 和 session/preview/source 六区宿主接线预留稳定 id、mode 和 route 载荷
- `WorkbenchLayout` 继续只做 editor 拓扑与持久化，不再承担底层基础类型的定义权

后续如果继续落地更完整的容器族、grid/flow 虚拟化、editor/runtime 宿主接线、world-space UI 和 runtime ECS bridge，可以在本目录继续追加更细分的实现文档。
