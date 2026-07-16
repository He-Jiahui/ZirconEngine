---
related_code:
  - zircon_runtime/src/ui/v2/loader.rs
  - zircon_runtime/src/ui/v2/file_cache.rs
  - zircon_runtime/src/asset/importer/ingest/asset_importer.rs
  - zircon_plugins/ui_document_importer/runtime/src/lib.rs
  - zircon_runtime/src/asset/importer/registry.rs
  - zircon_runtime/src/asset/assets/ui.rs
  - zircon_runtime/src/ui/template/asset/hot_reload_plan.rs
  - zircon_runtime/src/plugin/extension_registry/validation/component.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_documents.rs
  - zircon_editor/src/tests/ui/boundary/zui_asset_governance.rs
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
  - tools/tests/test_zui_docs_suffix_convergence.py
  - tools/tests/test_zui_docs_suffix_status_guards.py
  - tools/tests/test_zui_docs_suffix_convergence_test_owner_boundaries.py
  - tools/tests/test_zui_docs_current_status_suffix_test_owner_budget.py
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
| zui importer | `zircon_plugins/ui_document_importer/runtime/src/lib.rs` | 解析 `.zui` component/view/style | 当前 plugin-owned 单一入口，无 Runtime 内置 importer 兼容路径 |
| source-template registry | `asset/importer/registry.rs:20-23,80` | 拒 `.ui.toml`/`.v2.ui.toml` source-template，要求 `.zui` | 收敛为单一 `.zui` 校验 + 废弃诊断 |
| 错误文案 | `asset/assets/ui.rs:131` | "must use `.zui`, not `.v2.ui.toml`" | 保留并扩展到 view/style |
| 热重载 plan | `ui/template/asset/hot_reload_plan.rs:158` | 同时认 `.zui`/`.v2.ui.toml`/`.ui.toml` | 仅认 `.zui` |
| 扩展点 component 校验 | `plugin/extension_registry/validation/component.rs:55` | `ui_document` 须 `.zui` | 不变（已对齐） |
| editor 扩展文档校验 | `editor/src/core/editor_extension.rs:569,583` | 认 `.zui`/`.v2.ui.toml` | 仅认 `.zui` |
| builtin 模板注册 | `editor/.../template_runtime/builtin/template_documents.rs` | 注册 `.v2.ui.toml` root，禁直接注册 `.zui` | 注册 `.zui` view root |
| 治理 guard | `editor/src/tests/ui/boundary/zui_asset_governance*` | 解析 `.v2.ui.toml` root + `.zui` 组件，限制各自 kind | 扩展为统一 `.zui`；新增"零 `.ui.toml` 后缀"闸口 |

## 3. 里程碑

### M1 — `.zui` 四档 profile 契约（前置，纯加载层）（2026-07-02 评审收口：原「三档」补 theme_tokens 为第四档）

- T1：`ui/v2/loader.rs` 把 `validate_zui_component_profile` 重构为 `validate_zui_document_profile`，按 `document.asset.kind` 分派：
  - `Component` → 现有组件 profile（无 `[root]`、恰好一个 component）。
  - `View` / `Style` → root profile（允许 `[root]`、允许 imports.widgets/styles，沿用 `.v2.ui.toml` 现行 view/style 校验）。
- T2：`zircon_plugins/ui_document_importer/runtime/src/lib.rs` 统一按 kind 分派 component/view/style；Runtime 只提供 current document loader/DTO，不复制 plugin importer，也不保留旧 ingest owner。
- T3：`file_cache.rs` 后缀分类把 `.zui` 视为可承载任意 kind；`.v2.ui.toml` 暂保留为废弃可读路径。
- T4（2026-07-02 评审收口，U6 联动）：**theme_tokens profile 校验/物化切片**——`validate_zui_document_profile` 补 `ThemeTokens` 分支：仅允许 `[asset]` + token 表（palette/typography/spacing 等 token 组表），禁 `[root]`、禁 component、禁 imports.widgets；物化由 importer 走 theme 资产路径，**消费方 = 计划 04 ThemeRegistry/loader**（04/05 已按 U6 改为 `.zui` theme_tokens profile，生产已有 `editor_tokens.zui`），本计划只持有后缀/profile 校验层，不定义 token 语义。
- 测试：`ui/tests/v2_asset/asset_loading.rs` 新增 `zui_view_root_loads`、`zui_style_root_loads`、`zui_component_still_enforced`；锁定三 kind 分派与组件档不回退。（2026-07-02 评审收口）补 `zui_theme_tokens_profile_enforced`（theme_tokens 档拒 `[root]`/component 混入），四 kind 分派齐备。
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
- 全局验收（policy §3 最小批次）：
  ```bash
  cargo test -p zircon_runtime --lib --locked zui suffix ui_toml
  cargo test -p zircon_editor --lib --locked zui_asset_governance production_ui_assets suffix
  cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --all-targets --locked
  cargo fmt --all --check
  ```
  全量 runtime/editor lib 回归留给波次收口（policy §4）。

## 4. 依赖与推进顺序

```
M1（.zui 四档 profile）──→ M2（治理统一）──→ M3（生产资产迁移，分批）
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

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

本子计划产出记录已超过 10 条，具体记录已迁入编号子目录。

- 迁入记录：[`11/2026-07-09-zui-suffix-convergence-and-ui-toml-retirement-output-records.md`](11/2026-07-09-zui-suffix-convergence-and-ui-toml-retirement-output-records.md)
- open 待修复：[plan-output-archive-notice](11/failure-2026-07-13-plan-output-archive-notice.md)
