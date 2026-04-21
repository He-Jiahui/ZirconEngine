---
related_code:
  - zircon_runtime/src/ui/template/mod.rs
  - zircon_runtime/src/ui/template/asset/mod.rs
  - zircon_runtime/src/ui/template/asset/document.rs
  - zircon_runtime/src/ui/template/asset/legacy.rs
  - zircon_runtime/src/ui/template/asset/loader.rs
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_fixture.rs
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_manager.rs
  - zircon_runtime/src/ui/tests/asset.rs
  - zircon_runtime/src/tests/ui_boundary/mod.rs
  - zircon_editor/src/ui/asset_editor/mod.rs
  - zircon_editor/src/ui/asset_editor/preview/preview_mock.rs
  - zircon_editor/src/ui/asset_editor/session/mod.rs
  - zircon_editor/src/tests/support.rs
  - zircon_editor/src/tests/editing/ui_asset_preview_binding_authoring.rs
  - zircon_editor/src/tests/ui/boundary/mod.rs
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/mod.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/bootstrap_assets.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/editor_layouts.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/runtime_previews.rs
  - zircon_editor/tests/workbench_slint_shell.rs
  - zircon_editor/assets/ui/editor/ui_asset_editor.ui.toml
  - zircon_runtime/assets/ui/runtime/fixtures/hud_overlay.ui.toml
implementation_files:
  - zircon_runtime/src/ui/template/mod.rs
  - zircon_runtime/src/ui/template/asset/mod.rs
  - zircon_runtime/src/ui/template/asset/document.rs
  - zircon_runtime/src/ui/template/asset/legacy.rs
  - zircon_runtime/src/ui/template/asset/loader.rs
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_fixture.rs
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_manager.rs
  - zircon_runtime/src/ui/tests/asset.rs
  - zircon_runtime/src/tests/ui_boundary/mod.rs
  - zircon_editor/src/ui/asset_editor/mod.rs
  - zircon_editor/src/ui/asset_editor/preview/preview_mock.rs
  - zircon_editor/src/ui/asset_editor/session/mod.rs
  - zircon_editor/src/tests/editing/ui_asset_preview_binding_authoring.rs
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/bootstrap_assets.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/editor_layouts.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/runtime_previews.rs
  - zircon_editor/tests/workbench_slint_shell.rs
  - zircon_editor/assets/ui/editor/ui_asset_editor.ui.toml
  - zircon_runtime/assets/ui/runtime/fixtures/hud_overlay.ui.toml
plan_sources:
  - user: 2026-04-20 目前zircon_editor有两套ui相关代码 一套在core里面需要迁移回ui
  - user: 2026-04-20 要求加载入口不允许放入src
  - user: 2026-04-20 PLEASE IMPLEMENT THIS PLAN
  - .codex/plans/Runtime Core Fold-In And Compile Recovery.md
  - .codex/plans/Zircon UI 资产化 Widget Editor 与共享 Layout.md
  - .codex/plans/编辑器 .slint 去真源 Runtime UI 可用 Cutover 路线图.md
tests:
  - zircon_runtime/src/ui/tests/asset.rs
  - zircon_runtime/src/tests/ui_boundary/mod.rs
  - zircon_editor/src/tests/editing/ui_asset_preview_binding_authoring.rs
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/bootstrap_assets.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/editor_layouts.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/runtime_previews.rs
  - zircon_editor/tests/workbench_slint_shell.rs
  - cargo test -p zircon_runtime ui_document_compiler_expands_imported_widget_references_and_applies_stylesheets --locked
  - cargo test -p zircon_runtime ui_flat_asset_migration_adapter_converts_flat_assets_into_tree_authority_source --locked
  - cargo test -p zircon_runtime ui_asset_loader_rejects_flat_asset_documents_on_formal_path --locked
  - cargo test -p zircon_runtime ui_legacy_template_adapter_emits_canonical_asset_source_that_roundtrips --locked
  - cargo test -p zircon_editor --locked --offline tests::ui::boundary::editor_test_support_migrates_flat_ui_asset_documents_for_editor_consumers -- --exact
  - cargo test -p zircon_editor --lib editor_production_ui_modules_keep_flat_asset_migration_in_test_support_only --locked
  - cargo test -p zircon_editor --lib ui_asset_editor_bootstrap_assets_parse_and_compile_with_imports --locked
  - cargo test -p zircon_editor --lib tests::ui::ui_asset_editor --locked --offline --message-format short
  - cargo test -p zircon_editor --locked --offline --test workbench_slint_shell
doc_type: module-detail
---

# UI Asset Documents And Editor Protocol

## Purpose

这份文档记录当前 shared UI 资产协议的三个硬结论：

- 生产序列化权威已经切到 tree-shaped `.ui.toml`
- formal loader 和 formal public template surface 都只接受 tree authority，不再公开 flat migration adapter
- editor/runtime 继续共用同一条 `UiAssetLoader -> UiDocumentCompiler -> UiTemplateSurfaceBuilder` 链路

本篇只描述“资产文档格式”和“editor/runtime 如何消费这份格式”。editor workbench owner 收口见 [`UI Asset Editor Host Session`](../editor-and-tooling/ui-asset-editor-host-session.md)。

## Serialized Authority Is Tree TOML

当前磁盘上的正式格式不再是：

- 顶层 `root = { node = ... }`
- 顶层 `nodes = { ... }` 作为长期权威注册表
- `components.*.root = "node_id"` 的根节点字符串引用

正式 authority 现在改成递归树：

- 顶层 `root` 直接内嵌根节点定义
- 每个节点显式带稳定 `node_id`
- 子节点通过 `children[].node` 递归内嵌
- 每条父子边的 `mount/slot` 元数据跟随 `children[]` 保存
- `components.*.root` 也直接内嵌子树，而不是再回指某个全局节点表 id

这意味着 `.ui.toml` 的 diff、迁移和 source roundtrip 已经围绕“真实树结构”展开，而不是继续把持久化权威建立在一张平面节点注册表上。

## Stable `node_id` Remains Mandatory

虽然磁盘格式改成了树，但 identity 没有改成“按位置寻址”。

当前仍然要求：

- 每个节点显式声明稳定 `node_id`
- source、hierarchy、preview、undo/replay 都以 `node_id` 作为跨表示层锚点
- 迁移器和 serializer 必须保住 `node_id`，而不是在输出时重新编号

这样 editor 侧的 selection、source cursor remap、document diff 和 preview projection 仍然能围绕稳定身份工作，而不会因为树形序列化就退化成位置敏感协议。

## Formal Loader Is Tree-Only

[`zircon_runtime/src/ui/template/asset/document.rs`](../../zircon_runtime/src/ui/template/asset/document.rs) 现在通过自定义 `Serialize`/`Deserialize` 把 `UiAssetDocument` 的磁盘格式固定成 tree TOML。

当前正式入口有两个关键点：

- `toml::from_str::<UiAssetDocument>(...)` 仍然只按 tree wire format 反序列化
- [`UiAssetLoader`](../../zircon_runtime/src/ui/template/asset/loader.rs) 现在直接执行 tree parse，并在 parse 通过后校验 tree authority；它不会再探测 flat 文档形态，也不会在 formal path 上自动迁移旧平面输入

也就是说，正式运行链路的 authority 只有 tree，而且 formal load path 会直接拒绝 flat `root + nodes` 文档。

## Flat Migration Is Crate-Internal Or Test-Only

为了完成这次一次性 cutover，并兼容 editor 侧仍会接触到历史 flat 夹具/测试文档，shared 模板层保留了 [`UiFlatAssetMigrationAdapter`](../../zircon_runtime/src/ui/template/asset/legacy.rs)。

它的职责仍然被故意压到最小：

- 读取旧平面 TOML
- 组装 `UiAssetDocument`
- 输出 canonical tree TOML

但它现在只保留两类受控使用点：

- runtime test-only 迁移 helper：通过 `#[cfg(test)] pub(crate)` 重导出给 `zircon_runtime` 自己的单元测试使用
- editor test support：[`zircon_editor/src/tests/support.rs`](../../zircon_editor/src/tests/support.rs) 保留本地 flat fixture 迁移 helper，用来把历史测试夹具转成 tree 后再喂回正式 loader

这仍然保持了真正的 authority 边界不变:

- 正式解析后的 `UiAssetDocument` 只来自 tree wire format
- canonical save 仍然只会输出 tree TOML
- flat migration adapter 不是正式 public API，也不会挂在 production loader 上继续兜底

## Internal Compatibility Layer

当前还有一个刻意保留的实现折中：

- 正式磁盘 authority 仍然是 tree TOML
- runtime 内部的 flat migration 只保留在 `#[cfg(test)]` 单元测试路径
- editor 对历史 flat fixture 的兼容只保留在 `src/tests/support.rs`
- editor/runtime 的生产 consumer 已经直接走递归树 helper，而不是继续依赖 `document.nodes` 作为工作态真源

因此这轮的真实边界是：

- 磁盘权威：tree TOML
- 正式 loader：tree only，flat 输入直接报错
- legacy 兼容：只保留在 runtime test-only helper 和 editor test support
- 生产内存访问：tree-native helper API

## Editor Protocol Consequences

对 `zircon_editor::ui::asset_editor` 来说，这次 cutover 带来三条直接后果：

- canonical save 现在总是输出 tree TOML
- source roundtrip 仍然围绕稳定 `node_id`，不依赖旧平面节点表继续存在
- preview / undo / replay / style inspection / tree edit 已切到 `UiAssetDocument` 递归 helper，不再直接围绕 `document.nodes` 做权威更新
- editor production consumer 现在只接受 tree authority；只有 editor test support 还会对历史 flat fixture 做本地 canonicalize，再把 tree 结果投进同一条正式编译链

这也是为什么 editor 侧可以在不回滚 `UiAssetEditorSession` 既有 authoring 流程的前提下，完成 shared 资产真源切换，同时把 authoring 内部逻辑也收敛到 tree-shaped helper surface。

## Managed Asset Scope

这次被正式迁成 tree authority 的受管资产包括两类：

- editor/project 资产：`zircon_editor/assets/ui/**/*.ui.toml`
- runtime builtin fixture：`zircon_runtime/assets/ui/runtime/fixtures/*.ui.toml`

其中 runtime fixture 还额外完成了“加载入口不能放在 `src/` 下”的目录规范收口，详见 [`Runtime UI Graphics Integration`](../assets-and-rendering/runtime-ui-graphics-integration.md)。

## Shared Consumption Path

无论 editor 还是 runtime，现在都走同一条 shared 资产消费路径：

1. `UiAssetLoader` 读取 tree TOML
2. `UiDocumentCompiler` 展开 import/component/slot/stylesheet
3. `UiTemplateSurfaceBuilder` 把编译结果落成 `UiSurface`

区别只在最后的宿主投影：

- editor 把 `UiSurface` 投影到 Slint host
- runtime 把 `UiSurface.render_extract` 打包进 runtime frame / graphics pass

资产真源、编译器和 surface builder 没有 editor/runtime 分叉实现。

## Acceptance Evidence

当前与这次协议切换直接对应的验证包括：

- `cargo test -p zircon_runtime ui_document_compiler_expands_imported_widget_references_and_applies_stylesheets --locked`
  - 证明 tree authority 仍能经 shared compiler 展开 import 和 stylesheet
- `cargo test -p zircon_runtime ui_flat_asset_migration_adapter_converts_flat_assets_into_tree_authority_source --locked`
  - 证明 flat-to-tree 一次性迁移器能生成新 authority
- `cargo test -p zircon_runtime ui_asset_loader_rejects_flat_asset_documents_on_formal_path --locked`
  - 证明 formal loader 现在会直接拒绝 flat authority，而不是在 production path 上继续兜底迁移
- `cargo test -p zircon_runtime ui_legacy_template_adapter_emits_canonical_asset_source_that_roundtrips --locked`
  - 证明 legacy template adapter 生成的新源码能够 roundtrip
- `cargo test -p zircon_editor --locked --offline tests::ui::boundary::editor_test_support_migrates_flat_ui_asset_documents_for_editor_consumers -- --exact`
  - 证明 editor test support 仍能把历史 flat fixture 本地迁成 tree，再复用正式 shared compiler 链路
- `cargo test -p zircon_editor --lib editor_production_ui_modules_keep_flat_asset_migration_in_test_support_only --locked`
  - 证明 editor 生产 UI 模块不再持有 flat migration helper，legacy 兼容只留在 `src/tests/support.rs`
- `cargo test -p zircon_editor --lib ui_asset_editor_bootstrap_assets_parse_and_compile_with_imports --locked`
  - 证明 editor bootstrap 资产已经能按新 tree format 打开和编译

这几条证据组合起来，已经覆盖 formal loader tree-only 边界、crate-internal/test-only flat migration、legacy adapter 和 editor/runtime 共用消费链路。
