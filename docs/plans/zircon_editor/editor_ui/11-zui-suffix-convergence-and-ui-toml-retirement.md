---
related_code:
  - zircon_runtime/src/ui/v2/loader.rs
  - zircon_runtime/src/ui/v2/file_cache.rs
  - zircon_runtime/src/asset/importer/ingest/asset_importer.rs
  - zircon_runtime/src/asset/importer/ingest/import_ui_zui_asset.rs
  - zircon_runtime/src/asset/importer/ingest/ui_v2_document_import.rs
  - zircon_runtime/src/asset/importer/registry.rs
  - zircon_runtime/src/asset/assets/ui.rs
  - zircon_runtime/src/ui/template/asset/hot_reload_plan.rs
  - zircon_runtime/src/plugin/extension_registry/validation/component.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_documents.rs
  - zircon_editor/src/tests/ui/boundary/zui_asset_governance.rs
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
governance:
  - docs/editor-and-tooling/zui-asset-governance.md
  - docs/editor-and-tooling/editor-template-compatibility-migration.md
doc_type: plan
---

# 计划 11：`.zui` 后缀统一与 `.ui.toml` 退役

> 状态：M5 静态实现完成，Cargo 全局验证被当前外部编译/锁文件状态阻塞（2026-06-28）
> 范围：把 UI 资产的 `.ui.toml` 后缀族（`.v2.ui.toml` 生产 view/style root + plain `.ui.toml` v1 遗留）整体收敛到唯一后缀 `.zui`，并逐步删除 `.ui.toml` 的加载/导入/热重载/治理支持。
> 原则（继承全局）：硬切不留兼容层；改 owner 路径时同窗口迁移 live caller 并删旧路径；不新增迁移期 facade/`pub use` 桥；root 接线薄；治理由 guard 测试锁。

## 1. 迁移前基线与当前收口事实

迁移前（计划起点，2026-06 实查）UI 资产存在**三态后缀**：

| 后缀 | 语义 | 起点数量 | 起点加载/校验 owner | 起点约束 |
| --- | --- | --- | --- | --- |
| `.zui` | UI v2 **组件原型**（component prototype） | 194 | `ui/v2/loader.rs::validate_zui_component_profile` | 硬性 `asset.kind = "component"`、无 `[root]`、恰好一个 component |
| `.v2.ui.toml` | UI v2 **view/style root** | 74 | `asset/importer/ingest/import_ui_v2_asset.rs` | 仅允许 `kind = view\|style`；`kind = component` 被拒（须用 `.zui`） |
| `.ui.toml`（plain，v1） | v1 模板文档（带 view `[root]`） | 15 | —— 仅 `zircon_editor/src/tests/fixtures/ui_legacy/**` 测试 fixture | 生产已无 plain v1；仅遗留 fixture |

迁移前证据：tracked suffix inventory 含 89 个旧 UI 后缀文件（74 个 `.v2.ui.toml` + 15 个 plain `.ui.toml`，后者全部位于 `ui_legacy/`）；`.zui` 起点文件数为 194。

当前收口事实（2026-06-28）：production legacy UI suffix file count = 0，覆盖 `zircon_editor/assets`、`zircon_runtime/assets` 与 `zircon_plugins`；`.zui` 是唯一 UI 资产后缀，并按 `asset.kind` 承载 component/view/style/theme token 文档 profile。production `.zui` parse scan = 268（194 component / 69 view / 4 style / 1 theme_tokens），legacy `ui_legacy` fixtures 已迁入 `ui_zui/**/*.zui`；`page_templates.toml`、`shell_regions.toml` 与 `presets.toml` 仍是 typed editor layout metadata assets（不是 UI v2 view/style root），layout metadata `.toml` indexes reference only `.zui` UI assets；Cargo 全局验证仍按状态表记录为外部编译车道阻塞，未在本段声明通过。

迁移前关键事实：

- **组件层已完成 `.zui` 收敛**：治理（`docs/editor-and-tooling/zui-asset-governance.md`）已规定组件文档必须 `.zui`，`.v2.ui.toml`/`.ui.toml` 不得承载 component。
- **迁移起点的 view/style root 仍在 `.v2.ui.toml`**：含 editor host shell（`workbench_shell.zui`、`floating_window_source.zui`、`*_surface_controls.v2.ui.toml` 等 18 个 host 文档）、插件 editor UI（navigation 11、sound 5、editor_build_export_desktop 4、particles 3）、theme（4）、runtime fixtures（5）。
- **迁移起点 plain v1 后缀只存在于测试 fixture**：生产代码不再加载 v1。
- **`editor-template-compatibility-migration.md` 已完成勘误**：迁移起点时该文档仍把 host 文档写成旧后缀；当前已由 `editor_template_compatibility_migration_doc_zui_only_guard_passed` 收束为 `.zui` 文档路径，并由 docs suffix guard 锁定。

目标终态（当前静态实现已达到，Cargo 全局验证未闭合）：`.zui` 成为**唯一** UI v2 资产后缀，按 `asset.kind` 分派 `component | view | style | theme_tokens` profile；`.v2.ui.toml` 与 plain `.ui.toml` 后缀及其加载路径全部退役；guard 测试锁定生产树零 `.ui.toml` 后缀。

## 2. 后缀检测/加载/治理触点清单（改动面）

> 退役 `.ui.toml` 必须穷尽下列触点，避免遗留半支持路径。

| 触点 | 文件:行 | 现行行为 | 终态 |
| --- | --- | --- | --- |
| v2 component profile 校验 | `ui/v2/loader.rs:53-90` | 仅接受 `kind=component` | 按 `kind` 分派：component→组件 profile；view/style→root profile |
| 后缀分类 | `ui/v2/file_cache.rs:576,580` | `.zui`/`.v2.ui.toml` 判 v2，`.zui` 判 component | 统一识别 `.zui`；`.v2.ui.toml` 进废弃诊断 |
| importer descriptor | `asset/importer/ingest/asset_importer.rs:174-175,416-420` | `.zui` → `ui_component` importer | `.zui` 承载 component/view/style 三 kind |
| v2 importer kind 分派 | `asset/importer/ingest/import_ui_v2_asset.rs:16` | 按 `parsed.asset.kind` 分支；拒 `.v2.ui.toml` 的 component | 入口改由 `.zui` 驱动；`.v2.ui.toml` 走废弃路径 |
| zui importer | `asset/importer/ingest/import_ui_zui_asset.rs` | 解析 `.zui` component | 扩展到 view/style |
| source-template registry | `asset/importer/registry.rs:20-23,80` | 拒 `.ui.toml`/`.v2.ui.toml` source-template，要求 `.zui` | 收敛为单一 `.zui` 校验 + 废弃诊断 |
| 错误文案 | `asset/assets/ui.rs:131` | "must use `.zui`, not `.v2.ui.toml`" | 保留并扩展到 view/style |
| 热重载 plan | `ui/template/asset/hot_reload_plan.rs:158` | 同时认 `.zui`/`.v2.ui.toml`/`.ui.toml` | 仅认 `.zui` |
| 扩展点 component 校验 | `plugin/extension_registry/validation/component.rs:55` | `ui_document` 须 `.zui` | 不变（已对齐） |
| editor 扩展文档校验 | `editor/src/core/editor_extension.rs:569,583` | 认 `.zui`/`.v2.ui.toml` | 仅认 `.zui` |
| builtin 模板注册 | `editor/.../template_runtime/builtin/template_documents.rs` | 注册 `.v2.ui.toml` root，禁直接注册 `.zui` | 注册 `.zui` view root |
| 治理 guard | `editor/src/tests/ui/boundary/zui_asset_governance*` | 解析 `.v2.ui.toml` root + `.zui` 组件，限制各自 kind | 扩展为统一 `.zui`；新增"零 `.ui.toml` 后缀"闸口 |

## 3. 里程碑

### M1 — `.zui` 三档 profile 契约（前置，纯加载层）

- T1：`ui/v2/loader.rs` 把 `validate_zui_component_profile` 重构为 `validate_zui_document_profile`，按 `document.asset.kind` 分派：
  - `Component` → 现有组件 profile（无 `[root]`、恰好一个 component）。
  - `View` / `Style` → root profile（允许 `[root]`、允许 imports.widgets/styles，沿用 `.v2.ui.toml` 现行 view/style 校验）。
- T2：`asset/importer/ingest/import_ui_zui_asset.rs` 扩展 kind 分派，复用 `import_ui_v2_asset.rs` 的 view/style 物化路径（不复制逻辑，抽 owner 共享函数）。
- T3：`file_cache.rs` 后缀分类把 `.zui` 视为可承载任意 kind；`.v2.ui.toml` 暂保留为废弃可读路径。
- 测试：`ui/tests/v2_asset/asset_loading.rs` 新增 `zui_view_root_loads`、`zui_style_root_loads`、`zui_component_still_enforced`；锁定三 kind 分派与组件档不回退。
- 验收：`cargo test -p zircon_runtime --lib v2_asset --locked`。

### M2 — 治理统一到 `.zui`（view/style + component 同源）

- T1：`zui_asset_governance/support.rs` 资产扫描改为以 `.zui` 为统一入口，按 kind 决定走 view/style root 规则还是 component 规则；`.v2.ui.toml` 进废弃清单（仍校验但发 deprecation 诊断）。
- T2：`zui_asset_governance.rs` 顶层身份/导入边界规则改为认 `.zui` view root（widget import 仍 `.zui#Component`，style import 仍 fragment-free `.zui`）。
- T3：更新 `docs/editor-and-tooling/zui-asset-governance.md`：`.v2.ui.toml` 段落改为"已退役/迁移中"，view/style root 改述为 `.zui`。
- 测试：`zui_asset_governance` 子套件全绿；新增 `view_style_roots_use_zui_suffix`。
- 验收：`cargo test -p zircon_editor --lib zui_asset_governance --locked`。

### M3 — 生产资产机械迁移（74 个 `.v2.ui.toml` → `.zui`）

> 一次性机械改名 + 引用重写，分批按 owner 提交，避免巨型 diff。

- 批次：
  1. editor host shell（`zircon_editor/assets/ui/editor/host/*.v2.ui.toml`，18）+ `template_documents.rs` builtin 注册重写。
  2. editor 主资产 + theme（`assets/ui/editor/*`、`assets/ui/theme/*`、`assets/ui/editor/windows/*`，~28）。
  3. 插件 editor UI（navigation 11、sound 5、editor_build_export_desktop 4、particles 3）。
  4. runtime fixtures（`zircon_runtime/assets/ui/runtime/fixtures/*`，5）。
- 每批：改名文件 → 全量重写 `res://...v2.ui.toml` 引用（imports.widgets/styles、registry、测试常量、screenshot 期望）→ 跑该 owner focused test。
- 勘误：`editor-template-compatibility-migration.md` 的旧后缀文案与 host 文档名已在状态记录 `editor_template_compatibility_migration_doc_zui_only_guard_passed` 中补齐，并加入 docs suffix guard。
- 测试：`template_assets.rs`、`material_meta_component_contracts.rs`、`workbench_projection_cutover.rs`、host `template_runtime` 套件全绿。

### M4 — 退役 plain v1 `.ui.toml`（`ui_legacy` fixtures）

- T1：逐一评估 15 个 `ui_legacy/**.ui.toml`：仍被引用的转 `.zui` 等价 fixture；已死的删除。
- T2：删除 v1 `.ui.toml` 专属加载/编译路径（若 `ui/template` 仍有 v1-only 分支，硬切删除）。
- T3：`hot_reload_plan.rs:158` 去掉 `.ui.toml`/`.v2.ui.toml` 分支。
- 测试：`asset_hot_reload_plan.rs`、`asset_dependency_index.rs` 改为 `.zui` 期望；新增 fixture 路径校验。

### M5 — 删除 `.ui.toml` 后缀支持 + 闸口锁定

- T1：删除 `file_cache.rs`、`registry.rs`、`import_ui_v2_asset.rs`、`editor_extension.rs` 中 `.v2.ui.toml`/`.ui.toml` 残留分支与错误文案（收敛为单一 `.zui`）。
- T2：新增防回归闸口 `production_ui_assets_use_only_zui_suffix`（`tests/ui/boundary`）：扫描 `zircon_editor/assets`、`zircon_runtime/assets`、`zircon_plugins/*/editor` 下生产 UI 资产，断言无 `.ui.toml`/`.v2.ui.toml` 后缀（`ui_legacy` 已删除）。
- T3：CLI/导出/staged build 若按后缀挑 UI 资产，同步改 `.zui`（核查 `tools/zircon_build*.py`、export packer 的 UI 资产收集）。
- 全局验收：
  ```bash
  cargo test -p zircon_runtime --lib --locked
  cargo test -p zircon_editor --lib --locked
  cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --all-targets --locked
  cargo fmt --all --check
  ```

## 4. 依赖与推进顺序

```
M1（.zui 三档 profile）──→ M2（治理统一）──→ M3（生产资产迁移，分批）
M1 ──→ M4（v1 退役，可与 M3 并行）
M3 + M4 ──→ M5（删后缀支持 + 闸口）
```

- M1 是纯加载层前置，不动资产文件，可独立验收。
- M3 各批次互不依赖，可并行，但都依赖 M1/M2 落地。
- M5 必须在 M3/M4 全部资产清零后，否则闸口会红。

## 5. 风险与硬约束

- **不留双后缀兼容层**：M5 必须真正删除 `.ui.toml`/`.v2.ui.toml` 分支，不得保留"读旧写新"的长期 facade（符合 CLAUDE.md 硬切纪律）。迁移期内 `.v2.ui.toml` 仅允许在 M2–M4 窗口内带 deprecation 诊断地可读。
- **kind 语义不可混淆**：`.zui` 承载 view/style 后，组件 profile 的强约束（无 `[root]`、单 component、closed style scope）必须仍只作用于 `kind=component`；view/style 不得被组件规则误伤。
- **builtin 注册边界**：治理现有"registry 不得直接注册 `.zui`"规则（govern line 160）是针对 component 的；M3 后 view root 本身就是 `.zui`，需把该规则改述为"registry 注册 `.zui` view root，但不直接注册 `.zui` component"。
- **资产 id 唯一性**：改名后 `res://...zui` локator 变化会改派生 id；M3 必须同窗口核对 `zui_asset_governance` 的全局 id 唯一性与 builtin alias 引用，避免 id 漂移。

## 6. 验收清单（每里程碑收口全绿）

- M1：`cargo test -p zircon_runtime --lib v2_asset --locked`
- M2：`cargo test -p zircon_editor --lib zui_asset_governance --locked`
- M3：各 owner focused test + `template_assets`/host `template_runtime` 套件
- M4：`asset_hot_reload_plan` / `asset_dependency_index` 改 `.zui` 后全绿
- M5：新增 `production_ui_assets_use_only_zui_suffix` 闸口 + §3 全局四连绿

## 7. 状态与产出记录

| 日期 | 里程碑 | 状态 | 完成项目 | 验证记录 |
| --- | --- | --- | --- | --- |
| 2026-06-28 | M5 style/theme token scan `.zui` target guard | editor_ui_11_m5_style_theme_token_scan_zui_guard_passed | 扩展 `tools/tests/test_zui_docs_suffix_convergence.py`，新增样式/主题计划未来 token 扫描目标守卫：`docs/plans/zircon_editor/editor_ui/04-style-theme-and-painter-selector.md` M1.S3 不得继续把 `*.v2.ui.toml` 写成裸 hex 扫描目标，必须改为 `.zui` UI 文档与 `paint_template_nodes` 源。Plan 04 §14 已同步记录本切片状态。 | RED：focused unittest 先失败并列出旧 `.zui`/`*.v2.ui.toml` 扫描范围；GREEN：`python -m unittest tools.tests.test_zui_docs_suffix_convergence.ZuiDocsSuffixConvergenceTests.test_style_theme_plan_token_scan_targets_zui_documents_only` 通过 1/1。该切片不改生产代码、不运行 Cargo。 |
| 2026-06-28 | M5 UI asset management `.zui` scope guard | editor_ui_11_m5_ui_asset_management_plan_zui_scope_guard_passed | 扩展 `tools/tests/test_zui_docs_suffix_convergence.py`，新增 UI 资产管理计划当前目标守卫：`docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md` §1 不得继续把 `.v2.ui.toml` 页面模板写成当前资产范围，必须写明 `.zui` UI 文档按 `asset.kind` 承载 component / view / style / theme_tokens，且 `.ui.toml` / `.v2.ui.toml` 后缀已退役。Plan 05 §14 已同步记录本切片状态。 | RED：focused unittest 先失败并列出旧 `.zui` 单组件 + `.v2.ui.toml` 页面模板口径；GREEN：`python -m unittest tools.tests.test_zui_docs_suffix_convergence.ZuiDocsSuffixConvergenceTests.test_ui_asset_management_plan_uses_zui_for_current_asset_scope` 通过 1/1。该切片不改生产代码、不运行 Cargo。 |
| 2026-06-28 | M5 plugin editor integration `.zui` authority guard | editor_ui_11_m5_plugin_editor_integration_zui_authority_guard_passed | 扩展 `tools/tests/test_zui_docs_suffix_convergence.py`，新增插件 editor 集成规范当前口径守卫：`docs/plans/zircon_plugins/10-editor-integration.md` §4/§6 不得继续把 retained host 模板体系或定制 drawer 写成 `.ui.toml`，必须写明 `.zui` 是当前插件 editor view/layout 文档后缀，`.ui.toml` / `.v2.ui.toml` 已退役。该文档正文已改为 `.zui` 模板体系，历史状态表旧后缀行保留为迁移证据。 | RED：focused unittest 先失败并列出旧 `.ui.toml` 模板体系、静态投影和定制 drawer 口径；GREEN：`python -m unittest tools.tests.test_zui_docs_suffix_convergence.ZuiDocsSuffixConvergenceTests.test_plugin_editor_integration_plan_uses_zui_for_current_template_authority` 通过 1/1。该切片不改生产代码、不运行 Cargo。 |
| 2026-06-28 | M5 structure convention descriptor family guard | plan_11_structure_convention_descriptor_family_guard_passed | 扩展 `tools/tests/test_zui_docs_suffix_convergence.py`，新增结构规范 §5 R5.3 守卫：当前 descriptor family 列表不得把 `.ui.toml` 写成活动 UI 家族，必须写明 `.zui` 是唯一 UI asset descriptor 家族、`.ui.toml` / `.v2.ui.toml` 已退役，并保留 `page_templates.toml`、`shell_regions.toml`、`presets.toml` 作为 typed editor layout metadata 而不是 UI asset descriptor。`engine-code-structure-convention.md` 的 R5.3 已同步到该口径。 | RED：focused unittest 先失败并列出 R5.3 仍包含 `.zui` / `.ui.toml` 与缺失当前事实；GREEN：`python -m unittest tools.tests.test_zui_docs_suffix_convergence.ZuiDocsSuffixConvergenceTests.test_structure_convention_descriptor_families_do_not_list_retired_ui_suffix` 通过 1/1，`python -m unittest tools.tests.test_zui_docs_suffix_convergence` 通过 3/3，`python -m unittest tools.tests.test_zui_static_suffix_convergence tools.tests.test_zui_docs_suffix_convergence` 通过 8/8。该切片不改 runtime/editor 生产代码、不运行 Cargo。 |
| 2026-06-28 | M5 layout metadata `.zui` reference guard | plan_11_layout_metadata_zui_reference_guard_passed | 扩展 `tools/tests/test_zui_static_suffix_convergence.py`，新增 editor layout metadata 守卫：使用 `tomllib` 解析 `page_templates.toml`、`shell_regions.toml` 与 `presets.toml`，递归扫描所有字符串，禁止 `.ui.toml` / `.v2.ui.toml` 回流，并要求 `res://ui/...` 引用必须以 `.zui` 结尾。计划书同步明确这些 `.toml` 是 typed editor layout metadata assets，不是 UI v2 view/style root；layout metadata `.toml` indexes reference only `.zui` UI assets。 | RED：`python -m unittest tools.tests.test_zui_docs_suffix_convergence` 先失败并列出 Plan 11 当前事实缺少 layout metadata `.zui` 状态短语；GREEN：`python -m unittest tools.tests.test_zui_static_suffix_convergence tools.tests.test_zui_docs_suffix_convergence` 通过 7/7。该切片不改 runtime/editor 生产代码、不运行 Cargo。 |
| 2026-06-28 | M5 editor-template compatibility 文档勘误守卫 | editor_template_compatibility_migration_doc_zui_only_guard_passed | 扩展 `tools/tests/test_zui_docs_suffix_convergence.py`，把 `docs/editor-and-tooling/editor-template-compatibility-migration.md` 纳入当前 `.zui` authority 文档守卫，禁止该文档继续把 editor host/runtime templates、builtin host documents、removed Slint compatibility surface 或 validation boundary 写成旧后缀当前事实。同步更新该文档头部 `related_code` 资产路径为 `workbench_shell.zui` / `floating_window_source.zui`，并把 Purpose、Current Template Path、Builtin Host Documents、Removed Compatibility Surface 与 Validation 章节改为 `.zui` 当前口径。 | RED：`python -m unittest tools.tests.test_zui_docs_suffix_convergence` 先失败并列出 `editor-template-compatibility-migration.md` 中的旧后缀/Slint 活动口径；GREEN：同命令 2/2 通过。直接扫描该文档的 `ui.toml`、`v2.ui.toml`、`workbench_slint`、`slint_build`、`SlintUiProjection`、`slint_host` 命中为 0。该切片不改生产代码、不运行 Cargo。 |
| 2026-06-28 | M5 Asset Browser user-visible suffix samples | asset_browser_visible_samples_zui_only_guard_passed | 收敛 Asset Browser thumbnail/summary 可见资源名样例：`summary_layout.rs`、`summary_nodes.rs`、`thumbnail_nodes.rs`、`tests.rs` 中的 long-name、locator、extension 测试数据从 `.ui.toml` 改为 `.zui`，避免当前 UI 展示和截图/布局测试继续把旧后缀作为正常资产名。`tools/tests/test_zui_static_suffix_convergence.py` 新增 Asset Browser user-visible sample 守卫，锁定这些 presentation fixture 不再出现 `.ui.toml`/`.v2.ui.toml`；`docs/zircon_editor/ui/layouts/views/asset_browser.md` 同步记录该约束。 | `python -m py_compile tools/tests/test_zui_static_suffix_convergence.py` 通过；`python -m unittest tools.tests.test_zui_static_suffix_convergence` 4/4 通过；Asset Browser scoped 旧后缀扫描清零；`rustfmt --edition 2021 --check` 覆盖 4 个 touched Asset Browser Rust 文件通过。未运行 Cargo，不声明 editor Cargo 验收关闭。 |
| 2026-06-28 | M5 active support `.zui` only static guard | active_support_zui_only_static_guard_passed | 新增 `tools/tests/test_zui_static_suffix_convergence.py`，把 Cargo 之外的 `.zui` only 终态拆成三条静态闸口：生产 UI asset roots（`zircon_editor/assets`、`zircon_runtime/assets`、`zircon_plugins`）不得有 `.ui.toml`/`.v2.ui.toml` 文件；旧 runtime importer 文件 `import_ui_asset.rs` / `import_ui_v2_asset.rs` 不得复活；active editor UI support owners 不得继续接受或生成旧后缀。同步把 widget/style promotion 默认 asset id、promotion conflict suffix、workspace watcher、asset event open、asset editor session route、runtime template host path filter 与 view projection path filter 全部硬切到 `.zui` only；`docs/editor-and-tooling/ui-asset-editor-host-session.md` 中当前路径描述同步改为 `.zui`。 | RED：`python -m unittest tools.tests.test_zui_static_suffix_convergence` 先列出 active support owner 中的 `.ui.toml` / `.v2.ui.toml` 残留；GREEN：同命令 3/3 通过。`py_compile` 通过；touched Rust `rustfmt --edition 2021 --check` 通过。未运行 Cargo，不声明 runtime/editor Cargo 验收关闭。 |
| 2026-06-28 | M5 当前权威计划 11 状态收束 | plan_11_current_status_zui_only_guard_passed | 扩展 `tools/tests/test_zui_docs_suffix_convergence.py`，新增 Plan 11 当前状态块守卫：`> 状态：` 到 §2 之间不得继续把迁移前“三态后缀”“view/style root 仍锁死在 `.v2.ui.toml`”和 `git ls-files '*.ui.toml'` 89 项写成当前事实，并必须列出当前收口事实。Plan 11 §1 已改为“迁移前基线与当前收口事实”，明确当前 production legacy UI suffix file count = 0、`.zui` 是唯一 UI 资产后缀、production `.zui` parse scan = 268，同时保留 Cargo 全局验证仍未闭合。 | RED：`python -m unittest tools.tests.test_zui_docs_suffix_convergence` 先失败并列出 Plan 11 顶部旧“现状基线/三态后缀/74+15 旧后缀”口径；GREEN：同命令 2/2 通过。该切片不改生产代码、不运行 Cargo。 |
| 2026-06-28 | M5 当前权威文档后缀口径收束 | docs_authority_suffix_convergence_guard_passed | 新增 `tools/tests/test_zui_docs_suffix_convergence.py`，只扫描当前权威文档 `docs/plans/zircon_editor/editor_ui/index.md`、`docs/editor-and-tooling/zui-asset-governance.md`、`docs/ui-and-layout/ui-asset-documents-and-editor-protocol.md` 中会把旧后缀描述成生产活动路径的短语，避免历史 `.ui.toml`/`.v2.ui.toml` 再回流成正式 layout 口径。三份文档已改为唯一 `.zui` 资产后缀：editor UI index 写明 `.zui` 按 `asset.kind` 承载 component/view/style/theme token；治理文档移除 active `.v2.ui.toml` root bucket；UI asset protocol 把 tree authority 和 managed asset scope 改为 `.zui`。 | RED：`python -m unittest tools.tests.test_zui_docs_suffix_convergence` 先失败并列出三份文档旧双轨/`.ui.toml` 短语；GREEN：同命令 1/1 通过。该切片不运行 Cargo，不声明 runtime/editor Cargo 验收关闭。 |
| 2026-06-28 | M1 `.zui` 三档 profile 契约 | implemented_static_passed_cargo_blocked_by_external_runtime_state | `ui/v2/loader.rs` 已把 `.zui` 校验改为按 `asset.kind` 分派：component 继续执行严格组件 profile，view/style 使用 root/document profile；`import_ui_zui_asset.rs` 通过共享 `ui_v2_document_import.rs` 物化 component/view/style，避免复制 `.v2.ui.toml` 分支；`.zui` importer descriptor 改为 `ui_document` 并声明 UiLayout/UiStyle 额外输出；`UiV2ComponentAsset::from_zui_str` 仍拒绝非 component kind。新增/更新 loader、file-cache、importer、wrapper 和 typed-error review guard 测试。 | 本轮触碰文件 `rustfmt --edition 2021 --check` 通过；`git diff --check` 仅有仓库 CRLF 提示。`cargo test -p zircon_runtime --lib v2_asset --locked` 编译到 runtime lib-test 后被当前工作区既有 graphics/test 编译错误阻塞：`sdf_font_bake.rs` 缺 `font_database`，camera-loop 测试 closure 参数漂移。随后工作区出现外部 `zircon_runtime/Cargo.toml`/`Cargo.lock` 的 `ttf-parser` 锁文件状态，`--locked` 命令直接拒绝启动；不计 Cargo 通过。 |
| 2026-06-28 | M2 `.zui` 治理统一 | implemented_static_passed_cargo_blocked_external_runtime_font | `zui_asset_governance/support.rs` 新增四个治理扫描桶：全部 `.zui` 文档、component `.zui` 文档、view/style `.zui` root、deprecated `.v2.ui.toml` root；既有 component 规则继续通过 component-only 窄口读取，避免 M3 后 view/style `.zui` 被误套组件目录/单组件规则。`zui_asset_governance.rs` 的 widget/style import、asset id、header、builtin registry 规则改为读取统一 UI root 文档集合；当前 `production_view_style_roots_are_zui_documents` 锁定 `.zui` root 能力与旧 `.v2.ui.toml` 退役。`docs/editor-and-tooling/zui-asset-governance.md` 已把 `.v2.ui.toml` 从正式 root 后缀改为迁移/弃用桶，并明确 builtin registry 后续可注册 `.zui` view/style root、不可注册 `.zui` component。 | 本轮触碰治理测试文件 `rustfmt --edition 2021 --check` 通过；旧 `ui_document_importer.zui_component` / `zircon.builtin.ui_component.zui` 扫描已清零。`cargo test -p zircon_editor --lib production_view_style_roots_are_zui_documents --locked --jobs 1 --target-dir E:\cargo-targets\zircon-zui-m2 --message-format short --color never -- --test-threads=1 --nocapture` 已尝试，编译到 `zircon_runtime` 后被外部 runtime text/font 路径阻塞：`zircon_runtime/src/asset/importer/ingest/import_font_asset/mod.rs:33:71` 报 `E0382 borrow of moved value: asset`；不计 Cargo 通过。 |
| 2026-06-28 | M3 生产资产迁移 / runtime fixtures 批次 | implemented_static_passed_cargo_blocked_external_runtime_state | 完成 M3 批次 4：`zircon_runtime/assets/ui/runtime/fixtures` 下 5 个 runtime view root 从 `.v2.ui.toml` 硬切为 `.zui`（HUD、Inventory、Pause、Quest Log、Settings），未保留旧文件或兼容别名。同步更新 `RuntimeUiFixture` 的 `res://`/相对路径、`runtime_asset_path` 路径测试、runtime fixture 目录 guard、`runtime_ui_asset_root_contains_only_zui_entries`、editor runtime preview include 常量与 route id、runtime UI golden 路径以及相关 runtime UI 文档引用。 | `rustfmt --edition 2021 --check` 覆盖本批 touched Rust 文件通过；5 个 `.zui` fixture 均通过 TOML 解析；旧 5 个 `.v2.ui.toml` 文件名、旧 preview 常量名、旧 `runtime_ui_asset_root_contains_only_v2_ui_toml_entries` 扫描清零；fixture 目录中 `.v2.ui.toml`/`.ui.toml` 为 0、`.zui` 为 5。`cargo test -p zircon_runtime --lib runtime_fixture_assets_live_under_crate_assets --locked --jobs 1 --target-dir E:\cargo-targets\zircon-zui-m2 --message-format short --color never -- --test-threads=1 --nocapture` 已尝试，编译到 `zircon_runtime` lib-test 后被外部 runtime 状态阻塞：typed-error guard 仍读取已被外部拆分的 `asset/importer/ingest/import_font_asset.rs`，`sdf_font_bake.rs` 缺 `font_database`，`ui/tests/text_pipeline.rs` 的 `FontAsset` 初始化缺字段，camera-loop 测试 closure 签名漂移；不计 Cargo 通过。 |
| 2026-06-28 | M3 生产资产迁移 / 插件 editor UI 批次 | implemented_static_passed_cargo_blocked_plugin_workspace_lock | 完成 M3 批次 3：`editor_build_export_desktop`、`navigation`、`particles`、`sound` 四组插件 editor UI 的 23 个 view root 从旧复合 TOML 后缀硬切为 `.zui`；已存在的 `export_profile_drawer.zui` 保持不变。同步更新插件 editor tests/extension ids、editor authoring descriptor 测试、export wizard panel 路径、runtime facade UI locator 测试与插件文档引用；未保留旧后缀文件、compat alias 或读旧写新路径。 | `rustfmt --edition 2021 --check` 覆盖本批 touched Rust 文件通过；23 个迁移后的 `.zui` 插件 editor view root 均通过 TOML 解析并保持 `asset.kind = "view"`；`zircon_plugins/*/editor` 下旧复合 UI 后缀扫描清零，精确旧文件名/引用扫描清零。`cargo test --manifest-path zircon_plugins\Cargo.toml -p zircon_plugin_navigation_editor --lib navigation_editor_plugin_contributes_authoring_extensions --locked --jobs 1 --target-dir E:\cargo-targets\zircon-zui-m2 --message-format short --color never -- --test-threads=1 --nocapture` 在编译前被 `zircon_plugins/Cargo.lock` 需更新且 `--locked` 禁止改写阻塞；根 workspace 不包含该插件包，`cargo test -p zircon_plugin_navigation_editor ... --locked` 未匹配 package；不计 Cargo 通过。 |
| 2026-06-28 | M3 生产资产迁移 / editor host shell 批次 | implemented_static_passed_cargo_timeout | 完成 M3 批次 1：`zircon_editor/assets/ui/editor/host` 下 18 个 host shell view root 从 `.v2.ui.toml` 硬切为 `.zui`，包含 `editor_main_frame`、`workbench_shell`、floating/source、pane body、surface controls、viewport toolbar 和 generated bottom body；`template_documents.rs` builtin host 注册同步改为 `.zui`，相关 host template/runtime tests、integration contracts、runtime dev-root 路径测试、工具原型脚本和文档引用均改指新路径。未保留旧后缀文件、旧 locator alias、读旧写新 facade 或 compat shim；window/main/theme `.v2.ui.toml` 属于 M3 批次 2，仍保留迁移桶。 | 本批 host 目录旧 `.v2.ui.toml`/plain `.ui.toml` 均为 0，host `.zui` 为 19（含既有 `activity_drawer_window.zui`）；18 个迁移 `.zui` 均通过 TOML 解析并保持 `asset.kind = "view"`；18 个旧 host 文件名在 tracked 文件精确扫描清零；命中新 host 路径的 Rust 文件 `rustfmt --edition 2021 --check` 通过；两个 web/native handoff verifier 通过；scoped `git diff --check` 通过，仅有仓库 CRLF 提示。`cargo test -p zircon_editor --lib critical_editor_shells_are_hard_cut_to_v2_assets --locked --jobs 1 --target-dir E:\cargo-targets\zircon-zui-m2 --message-format short --color never -- --test-threads=1 --nocapture` 已尝试，604s 超时无 Rust 诊断和无 test result；属于本命令的残留 `zircon-zui-m2` cargo/rustc 进程已停止；不计 Cargo 通过。 |
| 2026-06-28 | M3 生产资产迁移 / editor 主资产、window、theme 批次 | implemented_static_passed_cargo_timeout_m3_assets_zero | 完成 M3 批次 2：`zircon_editor/assets/ui/editor/*.v2.ui.toml`、`zircon_editor/assets/ui/editor/windows/*.v2.ui.toml`、`zircon_editor/assets/ui/editor/theme/editor_tokens.v2.ui.toml` 与 `zircon_editor/assets/ui/theme/*.v2.ui.toml` 共 28 个 view/style/theme root 物理改名为 `.zui`；同步更新 builtin window/editor 注册、layout `page_templates.toml`/`shell_regions.toml`、bootstrap tests、integration contracts、runtime asset/importer tests、plugin/editor descriptors、工具脚本、`.zui` 组件 style imports 与相关 docs。未保留旧后缀文件、compat alias 或读旧写新路径。M3 四个生产资产批次至此静态收口，生产资产树 `zircon_editor/assets`、`zircon_runtime/assets`、`zircon_plugins` 下 `.v2.ui.toml` 为 0。 | 28 个迁移 `.zui` 均通过 TOML 解析，kind 分布为 23 个 `view`、4 个 `style`、1 个 `theme_tokens`；28 个旧文件名 tracked 精确扫描清零；生产资产树 `.v2.ui.toml` 文件数为 0；命中新路径的 Rust 文件 `rustfmt --edition 2021 --check` 通过；两个 web/native handoff verifier 通过。`cargo test -p zircon_editor --lib critical_editor_shells_are_hard_cut_to_v2_assets --locked --jobs 1 --target-dir E:\cargo-targets\zircon-zui-m3 --message-format short --color never -- --test-threads=1 --nocapture` 已尝试，604s 超时无 Rust 诊断和无 test result；未发现本次 `zircon-zui-m3` target-dir 残留进程；不计 Cargo 通过。 |
| 2026-06-28 | M4 plain v1 `.ui.toml` fixture 退役 | implemented_static_passed | 完成 M4：`zircon_editor/src/tests/fixtures/ui_legacy` 下 15 个 plain `.ui.toml` fixture 硬切到 `ui_zui/**/*.zui`，UI asset editor/support、material meta contracts、fixture locator 和 runtime preview 引用同步直指 `.zui`。`hot_reload_plan.rs` 只把 `.zui` 归入 UI source，`asset_hot_reload_plan.rs` 与 `asset_dependency_index.rs` 的旧后缀期望改为普通 `Other`，未保留 v1 fixture 目录、旧路径 alias 或加载兼容层。 | `rustfmt --edition 2021 --check` 覆盖 M4/M5 触碰 Rust 文件通过；`zircon_editor/src/tests/fixtures` 下旧 `.ui.toml` 文件数为 0，`ui_zui` `.zui` fixture 数为 15；`ui_legacy` scoped 引用扫描清零；15 个 fixture 已在迁移切片通过 TOML 解析。 |
| 2026-06-28 | M5 删除 `.ui.toml` 后缀支持 + 闸口锁定 | implemented_static_passed_cargo_timeout | 完成 M5 静态实现：删除旧 `import_ui_asset.rs` / `import_ui_v2_asset.rs` production importer 文件和 registration 路径；`.zui` importer 通过 `ui_v2_document_import.rs` 统一物化 component/view/style；`registry.rs` 对 `.ui.toml` 与 `.v2.ui.toml` importer descriptor 返回 `DeprecatedUiDocumentSuffixImporter`；`file_cache.rs` 只发现/加载 `.zui`；`editor_extension.rs` 与 watcher/hot-reload/surface-index tests 改为 `.zui`；`tools/zircon_build.py` staging guard 拒绝 `ui/**/*.ui.toml` 与 `ui/**/*.v2.ui.toml`。新增/锁定 `production_ui_assets_use_only_zui_suffix`、`production_view_style_roots_are_zui_documents`、`importer_registry_routes_zui_to_document_backend`、`importer_decodes_zui_view_and_style_assets_from_zui` 与 build-script rejection tests。 | `python -m py_compile tools/zircon_build.py tools/tests/test_zircon_build_plugin_carriers.py tools/zircon_export/cli.py tools/zircon_export/plugin_validate.py tools/zircon_export/tests/test_plugin_validate.py` 通过；`python -m unittest tools.tests.test_zircon_build_plugin_carriers` 9/9；`python -m unittest tools.zircon_export.tests.test_plugin_validate` 7/7；生产资产树 `zircon_editor/assets`、`zircon_runtime/assets`、`zircon_plugins` 旧 `.ui.toml`/`.v2.ui.toml` 文件数为 0；全部生产 `.zui` 268 个可解析（194 component / 69 view / 4 style / 1 theme_tokens）；旧 importer/loader 符号扫描清零；scoped `git diff --check` 通过，仅 CRLF 提示。`cargo test -p zircon_runtime --lib importer_decodes_zui_view_and_style_assets_from_zui --locked --jobs 1 --target-dir E:\cargo-targets\zircon-zui-m5-verify` 已尝试，304s 超时无 Rust 诊断和无 test result；本 target-dir 残留 rustc 已停止，不计 Cargo 通过。 |
| 2026-06-28 | M5 验证刷新 / 状态校准 | verification_refreshed_metadata_passed_compile_timeout | 复核当前工作区后，修正 M2 状态行中的旧测试名为真实存在的 `production_view_style_roots_are_zui_documents`，并确认 M5 文档中真实测试锚点为 `importer_registry_routes_zui_to_document_backend`、`importer_decodes_zui_view_and_style_assets_from_zui`、`production_ui_assets_use_only_zui_suffix`。生产资产旧后缀扫描仍为 0，插件验证入口也通过当前真实命令行 smoke。 | `cargo metadata --locked --format-version 1 --no-default-features` 通过，说明当前根 workspace lockfile 不再是首个阻塞点；`cargo check -p zircon_runtime --lib --no-default-features --locked --jobs 1 --target-dir E:\cargo-targets\zircon-zui-lock-refresh --message-format short --color never` 已尝试，304s 超时无 Rust 诊断和无 test result，且未留下本 target-dir cargo/rustc。`python -m tools.zircon_export plugin validate native_dynamic_fixture --repo-root E:\Git\ZirconEngine --json` 与 `sound_timeline_animation_track` 均 `fatal=false` / diagnostics 为空；`python tools\audit_plugin_structure.py --json` 报告 `dist_capable_plugin_count = 37`、`dist_build_matrix_count = 37`、`dist_dependency_boundary_violations = 0`、`distribution_section_violations = 0`。 |
| 2026-06-28 | M5 Cargo core-min 编译尝试 | verification_attempted_coremin_libtest_timeout | 为缩小 `.zui` importer Cargo 验证范围，本轮按最低 feature 组合尝试只编译 runtime lib-test 中的 `importer_decodes_zui_view_and_style_assets_from_zui`，使用独立 target-dir 和 `CARGO_PROFILE_DEV_DEBUG=0`，避免与默认 target 和其他会话共享输出。该命令未触发 lockfile 错误，也未给出 Rust 编译诊断。 | `cargo test -p zircon_runtime --lib importer_decodes_zui_view_and_style_assets_from_zui --no-run --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-zui-coremin-test --message-format short --color never` 604s 超时无 test binary、无 Rust diagnostics；仅停止了引用该 target-dir 的残留 `rustc.exe`。生产资产旧 UI 后缀文件数仍为 0，旧 loader/importer 符号扫描仍清零；不计 Cargo 通过。 |
