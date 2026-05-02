---
related_code:
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/ui/module.rs
  - zircon_runtime/src/ui/dispatch/mod.rs
  - zircon_runtime/src/ui/dispatch/pointer/mod.rs
  - zircon_runtime/src/ui/dispatch/navigation/mod.rs
  - zircon_runtime/src/ui/binding/mod.rs
  - zircon_runtime_interface/src/ui/binding/mod.rs
  - zircon_runtime_interface/src/ui/binding/model/mod.rs
  - zircon_runtime_interface/src/ui/binding/model/event_path.rs
  - zircon_runtime_interface/src/ui/binding/model/event_kind.rs
  - zircon_runtime_interface/src/ui/binding/model/binding_value.rs
  - zircon_runtime_interface/src/ui/binding/model/binding_call.rs
  - zircon_runtime_interface/src/ui/binding/model/event_binding.rs
  - zircon_runtime_interface/src/ui/binding/model/parse_error.rs
  - zircon_runtime_interface/src/ui/binding/model/parser.rs
  - zircon_runtime/src/ui/event_ui/mod.rs
  - zircon_runtime/src/ui/event_ui/manager/mod.rs
  - zircon_runtime/src/ui/event_ui/manager/ui_event_manager.rs
  - zircon_runtime/src/ui/event_ui/manager/registration.rs
  - zircon_runtime/src/ui/event_ui/manager/invocation.rs
  - zircon_runtime/src/ui/event_ui/manager/reflection_store.rs
  - zircon_runtime/src/ui/event_ui/manager/subscription.rs
  - zircon_runtime/src/ui/layout/mod.rs
  - zircon_runtime/src/ui/layout/pass/mod.rs
  - zircon_runtime/src/ui/layout/pass/layout_tree.rs
  - zircon_runtime/src/ui/layout/pass/measure.rs
  - zircon_runtime/src/ui/layout/pass/arrange.rs
  - zircon_runtime/src/ui/layout/pass/axis.rs
  - zircon_runtime/src/ui/layout/pass/child_frame.rs
  - zircon_runtime/src/ui/layout/pass/clip.rs
  - zircon_runtime/src/ui/surface/mod.rs
  - zircon_runtime/src/ui/surface/pointer/mod.rs
  - zircon_runtime/src/ui/surface/navigation/mod.rs
  - zircon_runtime/src/ui/surface/render/mod.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/template/mod.rs
  - zircon_runtime/src/ui/template/build/mod.rs
  - zircon_runtime/src/ui/template/build/tree_builder.rs
  - zircon_runtime/src/ui/template/build/surface_builder.rs
  - zircon_runtime/src/ui/template/build/layout_contract.rs
  - zircon_runtime/src/ui/template/build/parsers.rs
  - zircon_runtime/src/ui/tree/mod.rs
  - zircon_runtime/src/ui/tree/node/mod.rs
  - zircon_runtime/src/ui/tree/node/tree_node.rs
  - zircon_runtime/src/ui/tree/node/ui_tree.rs
  - zircon_runtime/src/ui/tree/node/tree_access.rs
  - zircon_runtime/src/ui/tree/node/layout.rs
  - zircon_runtime/src/ui/tree/node/scroll.rs
  - zircon_runtime/src/ui/tree/node/interaction.rs
  - zircon_runtime/src/ui/tree/node/focus.rs
implementation_files:
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/ui/module.rs
  - zircon_runtime/src/ui/dispatch/mod.rs
  - zircon_runtime/src/ui/surface/mod.rs
  - zircon_runtime/src/ui/binding/mod.rs
  - zircon_runtime_interface/src/ui/binding/model/mod.rs
  - zircon_runtime/src/ui/event_ui/manager/mod.rs
  - zircon_runtime/src/ui/layout/pass/mod.rs
  - zircon_runtime/src/ui/template/build/mod.rs
  - zircon_runtime/src/ui/tree/node/mod.rs
  - zircon_runtime/src/ui/tests/boundary.rs
plan_sources:
  - user: 2026-04-16 全部积极拆分并按模块边界持续重构所有脚本
  - .codex/plans/全系统重构方案.md
  - user: 2026-04-20 implement the workspace hard cutover and standardize the result
tests:
  - cargo test -p zircon_runtime ui_module_registration_is_absorbed_into_runtime_ui_surface --locked --offline --target-dir target/codex-shared-b -- --nocapture
  - cargo test -p zircon_runtime --locked --offline --target-dir target/codex-shared-b -- --nocapture
  - cargo check --workspace --locked
  - cargo test -p zircon_runtime --locked
  - cargo test -p zircon_editor --lib --locked
  - cargo test -p zircon_runtime --locked --offline ui::tests --target-dir D:/cargo-targets/zircon-workspace-hard-cutover -- --nocapture
  - cargo check --workspace --locked --offline --message-format short --target-dir D:/cargo-targets/zircon-workspace-hard-cutover
  - cargo test --workspace --locked --offline --target-dir D:/cargo-targets/zircon-workspace-hard-cutover
doc_type: module-detail
---

# UI Module Boundary Refactor

## Purpose

这份文档记录两件边界变化：

- `zircon_runtime::ui` 内部哪些原本混合职责的单文件，已经被强制拆成 folder-backed subtree，以及这些子树现在分别承担什么职责。
- `UiModule` / `UiConfig` / `module_descriptor()` 这层 runtime 模块注册面已经固定在 `zircon_runtime::ui`，不再依赖已经删除的独立 UI crate root。

## Root Ownership

当前 crate 边界已经调整为：

- `zircon_runtime/src/ui/mod.rs` + `zircon_runtime/src/ui/module.rs`
  - 持有 `UiModule`、`UiConfig`、`UI_MODULE_NAME` 与 `module_descriptor()`
  - `zircon_runtime::builtin_runtime_modules()` 通过这里注册 runtime UI 模块
- `zircon_runtime/src/ui/mod.rs`
  - 继续暴露 binding、event_ui、layout、template、tree 等共享 UI 实现与 DTO surface
- `zircon_runtime/src/ui/dispatch/mod.rs` 与 `zircon_runtime/src/ui/surface/mod.rs`
  - 都降回结构入口，只声明子模块并导出受控 surface；不再承载 pointer/navigation/render/surface 行为实现
- 旧独立 UI crate 的 `src/module/*.rs`
  - 已删除；这层 runtime module wiring 不再留在独立 UI crate 内

这样处理的目的不是再保留一层独立 UI crate，而是把共享 UI 实现与 runtime 模块注册一起收束到 `zircon_runtime::ui`，让 `editor/graphics/asset` 直接消费当前 owner path。

## Dispatch

原来的 `dispatch/mod.rs` 同时混着：

- pointer event DTO
- pointer dispatch effect/context/result
- pointer dispatcher handler orchestration
- navigation dispatch effect/context/result
- navigation dispatcher handler orchestration

现在 `dispatch/` 继续保留原 owner path，但内部强制拆成两个 folder-backed 子树：

- `pointer/`
  - `event.rs`、`effect.rs`、`context.rs`、`invocation.rs`、`result.rs`、`dispatcher.rs`
  - pointer event payload、dispatch DTO 和 bubbling/capture orchestration 全部在这里
- `navigation/`
  - `effect.rs`、`context.rs`、`invocation.rs`、`result.rs`、`dispatcher.rs`
  - navigation dispatch DTO 和 focused-route/root-fallback orchestration 全部在这里
- `dispatch/mod.rs`
  - 只保留 `mod` 声明和显式 `pub use`

这意味着以后再扩 pointer 或 navigation 行为时，不需要再回到同一个 root `mod.rs` 里叠段落。

## Binding Model

原来的 `binding/model.rs` 同时包含：

- event path
- event kind
- binding value
- binding call
- event binding
- parse error
- parser

这些 neutral DTO/parser 职责先被拆到 `binding/model/`，并在 2026-05-02 UI runtime-interface hard-cutover 中整体迁到 `zircon_runtime_interface::ui::binding`：

- 声明类型各自独立文件
- parser 独立到 `parser.rs`
- `zircon_runtime/src/ui/binding/mod.rs` 继续保留 `UiEventRouter` behavior，并 re-export interface DTOs for runtime consumers

这意味着后续新增新的 binding value、event kind 或 parser 规则时，应修改 interface DTO owner，而不是在 runtime 重新引入本地 DTO/parser shadow module。

## Event Manager

原来的 `event_ui/manager.rs` 混合了：

- route 注册
- invocation
- reflection tree 存取
- diff 计算
- subscription 广播

现在 `event_ui/manager/` 被切成：

- `ui_event_manager.rs`
  - 只定义 `UiEventManager` 状态
- `registration.rs`
  - route 注册相关行为
- `invocation.rs`
  - binding / route / action 调用与 control request 处理
- `reflection_store.rs`
  - tree snapshot、property mutation、node index rebuild
- `subscription.rs`
  - subscribe / unsubscribe / broadcast
- `diff.rs`
  - reflection diff 纯函数

## Layout Pass

原来的 `layout/pass.rs` 现在拆成：

- `layout_tree.rs`
  - 顶层 `compute_layout_tree(...)`
- `measure.rs`
  - 反向 measure
- `arrange.rs`
  - 正向 arrange 与 scrollable child 排布
- `axis.rs`
  - 轴向求解和 extent helper
- `child_frame.rs`
  - child frame 计算
- `clip.rs`
  - clip frame 求交

因此 `layout/pass/mod.rs` 现在只是入口，不再承载所有布局算法细节。

## Surface

原来的 `surface/mod.rs` 同时混着：

- focus/navigation state DTO
- pointer / navigation route DTO
- render DTO、extract 组装、样式/视觉属性解析
- `UiSurface` 的 layout、hit-test、focus、capture、pointer/navigation dispatch orchestration

现在 `surface/` 被收成显式子树：

- `focus_state.rs`、`navigation_state.rs`
  - 只定义 shared focus/navigation 状态
- `pointer/`
  - `button.rs`、`event_kind.rs`、`route.rs`
- `navigation/`
  - `event_kind.rs`、`route.rs`
- `render/`
  - interface owns `command.rs`、`command_kind.rs`、`list.rs`、`resolved_style.rs`、`visual_asset_ref.rs` DTO declarations under `zircon_runtime_interface::ui::surface::render`
  - runtime keeps `extract.rs` for `extract_ui_render_tree(&UiTree)` behavior
  - `node_visual_data.rs` 与 `resolve.rs` 负责 render extract 的 metadata 解析
- `surface.rs`
  - `UiSurface` 自身的 rebuild / compute_layout / route / dispatch 行为
- `surface/mod.rs`
  - 只保留 `mod` 声明和受控导出

这里的关键不是“文件拆小了”，而是把 render extraction 和 interaction orchestration 从 root wiring 文件里赶出去，明确回到各自 owner path。

## Template Build

原来的模板 builder 单文件版本同时承担：

- build error
- tree builder
- surface builder
- interaction inference
- container inference
- layout contract mapping
- TOML 解析 helper

现在 `template/build/` 结构明确分层：

- builder 相关：`tree_builder.rs`、`surface_builder.rs`
- contract 相关：`layout_contract.rs`
- parser 相关：`parsers.rs`
- 轻量推断：`interaction.rs`、`container_inference.rs`
- 纯声明：`build_error.rs`

## Tree Runtime

原来的 `tree/node.rs` 既定义数据，又实现 layout dirty、scroll、route、focus、clip-chain 和 draw order 行为。

现在 `tree/node/` 被分成两层：

- 声明层
  - `dirty_flags.rs`
  - `input_policy.rs`
  - `layout_cache.rs`
  - `template_node_metadata.rs`
  - `tree_error.rs`
  - `tree_node.rs`
  - `ui_tree.rs`
- 行为层
  - `tree_access.rs`
  - `layout.rs`
  - `routing.rs`
  - `scroll.rs`
  - `render_order.rs`
  - `interaction.rs`
  - `focus.rs`

这样之后继续扩展 retained tree，不需要再让一个文件同时承载声明、scroll 语义、focus 几何和 pointer support 判断。

## Structural Rule Going Forward

这一轮之后，`zircon_runtime::ui` 相关模块保持以下约束：

- runtime module registration 只放在 `zircon_runtime::ui`
- `mod.rs` 只负责 `mod` / `pub use`
- `dispatch/mod.rs` 只能导出 `pointer/` 与 `navigation/`，不能再回流 handler type alias、dispatcher impl 或 mixed DTO
- `surface/mod.rs` 只能导出 state / route / render / `UiSurface` surface，不能再回流 render parsing 或 interaction orchestration
- 顶层声明一个文件一个类型
- parser、builder、dispatch、diff、scroll、focus 这类行为族单独成文件
- 新需求优先进入现有子树，不再重新生成新的 umbrella file
