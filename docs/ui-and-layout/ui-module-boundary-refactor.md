---
related_code:
  - zircon_ui/src/binding/mod.rs
  - zircon_ui/src/binding/model/mod.rs
  - zircon_ui/src/binding/model/event_path.rs
  - zircon_ui/src/binding/model/event_kind.rs
  - zircon_ui/src/binding/model/binding_value.rs
  - zircon_ui/src/binding/model/binding_call.rs
  - zircon_ui/src/binding/model/event_binding.rs
  - zircon_ui/src/binding/model/parse_error.rs
  - zircon_ui/src/binding/model/parser.rs
  - zircon_ui/src/event_ui/mod.rs
  - zircon_ui/src/event_ui/manager/mod.rs
  - zircon_ui/src/event_ui/manager/ui_event_manager.rs
  - zircon_ui/src/event_ui/manager/registration.rs
  - zircon_ui/src/event_ui/manager/invocation.rs
  - zircon_ui/src/event_ui/manager/reflection_store.rs
  - zircon_ui/src/event_ui/manager/subscription.rs
  - zircon_ui/src/layout/mod.rs
  - zircon_ui/src/layout/pass/mod.rs
  - zircon_ui/src/layout/pass/layout_tree.rs
  - zircon_ui/src/layout/pass/measure.rs
  - zircon_ui/src/layout/pass/arrange.rs
  - zircon_ui/src/layout/pass/axis.rs
  - zircon_ui/src/layout/pass/child_frame.rs
  - zircon_ui/src/layout/pass/clip.rs
  - zircon_ui/src/template/mod.rs
  - zircon_ui/src/template/bridge/mod.rs
  - zircon_ui/src/template/bridge/tree_builder.rs
  - zircon_ui/src/template/bridge/surface_builder.rs
  - zircon_ui/src/template/bridge/layout_contract.rs
  - zircon_ui/src/template/bridge/parsers.rs
  - zircon_ui/src/tree/mod.rs
  - zircon_ui/src/tree/node/mod.rs
  - zircon_ui/src/tree/node/tree_node.rs
  - zircon_ui/src/tree/node/ui_tree.rs
  - zircon_ui/src/tree/node/tree_access.rs
  - zircon_ui/src/tree/node/layout.rs
  - zircon_ui/src/tree/node/scroll.rs
  - zircon_ui/src/tree/node/interaction.rs
  - zircon_ui/src/tree/node/focus.rs
implementation_files:
  - zircon_ui/src/binding/model/mod.rs
  - zircon_ui/src/event_ui/manager/mod.rs
  - zircon_ui/src/layout/pass/mod.rs
  - zircon_ui/src/template/bridge/mod.rs
  - zircon_ui/src/tree/node/mod.rs
plan_sources:
  - user: 2026-04-16 全部积极拆分并按模块边界持续重构所有脚本
  - .codex/plans/全系统重构方案.md
tests:
  - cargo test -p zircon_ui --offline --verbose
  - cargo test -p zircon_core -p zircon_resource -p zircon_manager -p zircon_ui -p zircon_module -p zircon_input -p zircon_math --offline --verbose
doc_type: module-detail
---

# UI Module Boundary Refactor

## Purpose

这份文档只记录一件事：`zircon_ui` 内部哪些原本混合职责的单文件，已经被强制拆成 folder-backed subtree，以及这些子树现在分别承担什么职责。

## Binding Model

原来的 `binding/model.rs` 同时包含：

- event path
- event kind
- binding value
- binding call
- event binding
- parse error
- parser

现在这些职责被拆到 `binding/model/`：

- 声明类型各自独立文件
- parser 独立到 `parser.rs`
- `binding/mod.rs` 继续只做导出层

这意味着后续新增新的 binding value、event kind 或 parser 规则时，不需要再回到一个 AST+parser 混合文件里追加段落。

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

## Template Bridge

原来的 `template/bridge.rs` 同时承担：

- build error
- tree builder
- surface builder
- interaction inference
- container inference
- layout contract mapping
- TOML 解析 helper

现在 `template/bridge/` 结构明确分层：

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

这一轮之后，`zircon_ui` 相关模块保持以下约束：

- `mod.rs` 只负责 `mod` / `pub use`
- 顶层声明一个文件一个类型
- parser、builder、dispatch、diff、scroll、focus 这类行为族单独成文件
- 新需求优先进入现有子树，不再重新生成新的 umbrella file
