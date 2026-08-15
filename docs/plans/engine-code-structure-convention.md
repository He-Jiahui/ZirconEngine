---
related_code:
  - docs/zircon_runtime/structure/module-convention.md
  - docs/plans/zircon_runtime/frameworks/development-conventions.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - tools/tests/test_check_conventions.py
implementation_files:
  - tools/check_conventions.py
  - tools/convention_exemptions.py
  - zircon_runtime/src/tests/runtime_absorption/structure_convention.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/module_convention_gate.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/mod.rs
plan_sources:
  - user: 2026-06-22 优化 docs/plans editor/runtime/plugins 计划，统一代码结构与插件接口开发体验框架
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/engine-architecture/large-file-ownership-m1.md
  - docs/engine-architecture/runtime-interface-convergence.md
tests:
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/code_paths.rs::runtime_15_priority_plan_docs_code_paths_stay_current
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/test_paths.rs::runtime_15_priority_plan_docs_test_paths_stay_current
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/frontmatter_uniqueness.rs::runtime_15_priority_plan_docs_frontmatter_sections_have_unique_entries
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/header_sections.rs::runtime_15_priority_plan_docs_required_header_sections_stay_complete
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/plan_sources.rs::runtime_15_priority_plan_docs_plan_sources_stay_cross_linked
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/listing.rs::runtime_15_priority_plan_docs_guard_tests_stay_listed
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/child_layout.rs::runtime_15_priority_plan_docs_guard_children_are_folder_backed
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/nested_layout.rs::runtime_15_priority_plan_docs_guard_test_children_are_folder_backed
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/nested_layout.rs::runtime_15_priority_plan_docs_guard_test_child_prose_names_full_inventory
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/moved_paths.rs::runtime_15_priority_plan_docs_moved_guard_paths_stay_current
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/moved_paths.rs::runtime_15_priority_plan_docs_moved_mirror_names_full_inventory
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/inventory_sync.rs::runtime_15_priority_plan_docs_guard_inventory_uses_child_row_data_sources
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/priority_plan_docs/guard_tests/inventory_sync.rs::runtime_15_priority_plan_docs_listing_prose_names_full_inventory
  - python tools/check_conventions.py --only docs --json
  - python -B -m unittest tools.tests.test_check_conventions -v
  - .\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -SkipBuild -LibTests -TestFilter structure_convention
doc_type: convention-authority
status: in_progress
---

# 引擎级代码结构与模块接口规范（Engine Code Structure & Module Interface Convention）

> 规范权威：跨域通用规则已统一收敛至 [Zircon 开发规范总纲](zircon_runtime/frameworks/development-conventions.md)；本文保留代码结构主题的细节论证与执行上下文，不再作为并列规则源。

## 产出记录迁移说明

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

具体结构补记、验证与修复记录已迁入 [`_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md`](_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md)。本文件仅保留结构规范、接口约定与当前现状概述。

当前 G7 前置元数据收敛（2026-08-09）：本文件已删除重复枚举的历史细粒度 owner，只保留稳定的规范、守卫入口与 Runtime15 优先文档 guard 清单；没有恢复任何已删除路径。`check_conventions --only docs` 对本文件的悬空路径从 186 项降为 0；`docs/plans/engine-code-review-findings-2026-06.md` 删除 185 条已硬切 owner 并从 185 项降为 0；`docs/zircon_runtime/structure/module-convention.md` 又删除 103 条已硬切 owner（含旧 Runtime RHI、status mirror 与已删除 guard）并从 103 项降为 0；已完成的 Runtime05 计划仅清理 35 条旧 plan-status/审计脚本元数据并从 35 项降为 0，不重开其运行时里程碑。G7 同批把 `tests:` 中整条声明的具体仓库文件引用纳入审计，并排除 Cargo/Python 等命令、glob、模板占位符与 `target`/`build` 产物；这些 owned 明细均为 0，但共享工作树仍为 RED，全局数量会随其他 owner 的并发输入变化，故不作为本 exact scope 的冻结值。规范 Python 契约 26/26 GREEN；Runtime15 managed `priority_plan_docs` gate 仍待在不吸收外来 Runtime guard-owner diff 的不可变副本上执行，因此本文件和 G7 均不标 accepted。

当前 plan-status 结构同步（2026-07-10）：具体状态记录已硬切到 `zircon_runtime/runtime/01/` 至 `15/` 编号归档，父计划和总索引不再复制历史五列表格。测试支持按职责拆为 `plan_status/support/runtime_plan_archives.rs` 与 `plan_status/recent_static_guards/parent_routing.rs`，所有 owner 文件保持各自预算；Python boundary support 84/84、`risks = []`，standalone Rust plan-status 48/48。该结构同步没有恢复旧路径、兼容 facade、shim 或 re-export。

当前 Frameworks 物理边界同步（2026-07-31）：[`Frameworks 01 M0`](zircon_runtime/frameworks/01/2026-07-13-m0-current-structure-and-dependency-baseline.md) 已对 `zircon_runtime/src` 的 9,188 个 Rust 输入完成 pre/post 同指纹 `348fe59c72c798e5e64babb5489910f30dbd00a58441ec11c1e176826519339e` 的原子快照；production graph 为 2,391 refs / 76 edges。公开 `zircon_runtime::*` facade 形态保持不变，内部 physical hard-cut 的基础 prefix 是 `zr_math/zr_resource → zr_contracts → zr_kernel → zr_diagnostics`；完整顺序是 `foundation → platform/input → asset/scene → zr_operation → rhi/...`，随后 `zr_text → graphics text backend/graphics → ui/optional`。`zr_operation` 必须位于 scene 之后、可选 navigation 之前，不得为了伪造 layer-0 纯度把 `CoreHandle`/`scene::World` 依赖藏回 facade；`zr_text` 只承接 backend-neutral shaping/layout/font/atlas policy，GPU atlas upload/draw 与 wgpu/naga/glyphon backend 留在 graphics/RHI owner。历史声明顺序、graphics→scene 与 graphics→ui 生产边已清零；当前 crate-DAG 前置是 `asset→text=2`、`scene→animation=2`、`rhi→rhi_wgpu=1`，以及 source-cubemap projection 的两处 concrete Runtime `TaskPool` 测试先迁到 kernel integration owner。任何移动必须同批迁移消费方，不保留旧内部 owner、alias、shim 或双轨 re-export。依赖治理执行 stable-first；`notify 9.0.0-rc.3`、`winit 0.31.0-beta.2`、`zip 9.0.0-pre2` 仅允许精确 owner/reason/expiry/ticket 例外，过期仍判 RED，不得用 wildcard 或全局禁用规避。四份受管 cold/incremental timings 尚未生成，物理 `zircon_runtime/crates/` 迁移也未开始，因此 M0 与物理 M1 均未完成。

Frameworks 物理边界增量（2026-08-03）：[`Frameworks01 RHI/WGPU failure`](zircon_runtime/frameworks/01/failure-2026-08-02-rhi-wgpu-presenter-and-backend-contract-test-owner.md) 已实现首个 M2 物理 hard cut。`zr_rhi` 只拥有 backend-neutral descriptors/capabilities/handles/device-command/UI-surface contracts，`zr_rhi_wgpu` 单向依赖 `zr_rhi` 并拥有 WGPU presenter、GPU timer/readback 与 deterministic backend tests；Runtime 只保留 `zircon_runtime/src/rhi.rs` curated facade 和唯一 default presenter factory。旧 `zircon_runtime/src/rhi/`、`zircon_runtime/src/rhi_wgpu/` 以及 `mod rhi_wgpu` 已删除，12 个后端测试模块已迁入 `zr_rhi_wgpu/src/tests/`，Editor 仍只消费 `zircon_runtime::rhi`。静态审计已无 Runtime `rhi_wgpu` domain，但 managed Cargo、immutable snapshot 与 fixed return 尚未完成，因此该记录是 `implemented / validation-pending / failure-resolving`，不是 M2 accepted；历史 `rhi_wgpu/command_validation.rs`、`rhi_wgpu/device.rs`、`rhi_wgpu/ui_surface.rs` 等路径只允许在既有 Runtime15 产出记录中作为 former-owner 标识出现。

当前文本计划同步（2026-08-01）：rich/vertical prewarm、cache identity、raster/upload report、backend face-ID authority、fallback span 和 run-language identity 都由对应的 Text 子 owner 管理；renderer root 仍只负责编排。系统 `fontdb` face 的容器字节由 `FontDatabase` 按权威 backend ID 物化，竖排继续复用 `text/shaping/vertical/orientation.rs`，SDF consumer 不另建 Unicode 或字体策略。新的 runtime UI → screen-space text → WGPU readback 产品 harness 已固定输出到 `docs/tests/runtime/text/runtime_text_mvp_foundation_product_framebuffer_20260801.png`，覆盖 CJK/RTL/Emoji/Native/SDF、富文本、表格和 VerticalRl；截至当前该 PNG 尚未由 current-source managed run 生成，旧截图不是接受证据。live typography、scroll 增量和产品像素验收继续由 Text02/Text03/Text04/Text05/Text07/Text09 子计划保持 open；具体证据只在这些 child records 中维护。

## §0 适用范围与背景

适用 `zircon_runtime` / `zircon_editor` / `zircon_plugins` / `zircon_runtime_interface` / `zircon_app` / `zircon_hub`，含 render 子计划覆盖的 `graphics/**`。

引擎已非常庞大，分层方向合理，但模块内细节散乱，直接损害用户 code review / inspect 体验，集中表现在六类结构债：模块布局不统一、命名失序、巨型文件、公共 API 不友好、测试组织三套并行、插件 DX 割裂（证据见各落地子计划"现状缺口表"）。

本规范复用仓库既有的结构治理范式——`audit_runtime_structure.py` + `runtime_structure_audits/*.py` 审计脚本族、`tests/runtime_absorption/**` guard 测试、`docs/**` 镜像文档、`large_file_ownership_gate`（`m1_gate_status` / owner 分类 / 迁移债）——把"统一模块规范 + 插件 DX 框架"做成同形门禁，对**后续开发**与**存量重构**均可机器化强制、可验收、可防回归。

## §1 模块文件布局标准（owner-module 模式）

| 规则 | 定稿 | 反例（实证） |
|---|---|---|
| R1.1 根接线薄 façade | `lib.rs` / `mod.rs` 仅含 `mod` 声明 + 精选 `pub use` + 模块 doc，**零行为** | `plugin/mod.rs`（74 行扁平导出） |
| R1.2 `mod.rs` vs `module.rs` | 目录统一用 `mod.rs` 作 façade；引擎子系统的 `ModuleDescriptor` 单独放 `module.rs`，**且仅注册子系统**才有 `module.rs` | `input/mod.rs` 直塞注册 / `graphics/` 注册分散无 `module.rs` |
| R1.3 行为落具名 owner 叶子 | 行为进 `lifecycle.rs` / `dispatch.rs` / `validation.rs` / `diagnostics.rs` / `conversion.rs` / `extract.rs` 等具名模块，不堆胖单文件 | `dynamic_api/session.rs`（773 行协调 17 子模块） |
| R1.4 行数预算 | 生产文件软上限 800（review 警告）、硬上限 1000（gate 拦截）；测试文件 > 800 必须 folder-backed 拆分；豁免（vendored upstream、fixture、`@generated`）须在 gate `exempt` 字段登记 | `core/framework/tests.rs`（1848） |
| R1.5 嵌套深度 | 同一分类维度 ≤ 3 层；域重叠（如 `asset/assets/scene/animation/`）须拍平 | `asset/assets/scene/` 深嵌 |

**`module.rs` 存在判据**：当且仅当该目录对应一个会向 runtime/editor 注册的 `*Module`（拥有 `module_descriptor()`）。否则不得出现 `module.rs`。

owner 拆分纪律继承 `large-file-ownership-m1.md`：**按 ownership 拆，不按等行数切**；root 文件可作结构 façade，但不得为避免改调用方而保留行为；拆分时不留兼容 wrapper，消费方直接调用新 owner 路径或精选 façade。

## §2 命名规范

- **R2.1 复数 / 单数判定**：目录名是"其下每个文件都是它的一个实例"的**种类** → 复数（`components/`、`assets/`、`importers/`、`systems/`、`effects/`）；目录是"单一内聚子系统 / owner" → 单数（`manager/`、`dispatch/`、`pipeline/`、`backend/`、`layout/`）。判定测问句："该目录名是不是一类东西、其下每个文件是它的一个？"是则复数，否则单数。
- **R2.2 前缀允许词表**（其余前缀视为命名债）：
  - `runtime_`：**仅**当与 authoring/descriptor 孪生对比时（`runtime_asset_path` vs `asset_path`）。**禁止**当通用命名空间标签——已在 runtime crate / `*_runtime_provider` 目录内的模块不得再冠 `runtime_`（`hybrid_gi_runtime_provider/runtime_state.rs` → `state.rs`）。
  - `default_`（默认实现）、`builtin_`（内置目录 / 枚举）、`compiled_`（编译后产物）、`frozen_`（冻结表）。
- **R2.3 禁用无主名**：模块名禁用 `_inner` / `_impl` / `_helper` / `util(s)` / `misc` / `common`；改成描述其 owns 什么的名字（例如旧 `editor_event_runtime_inner.rs` 已按职责硬切为 `editor_event_runtime_state.rs`，旧 `core/runtime/state/runtime_inner.rs` 已按职责硬切为 `core/runtime/state/core_runtime_state.rs`，旧 `scene/ecs/observer/utils.rs` 已按职责硬切为 `scene/ecs/observer/callback_registry.rs`，旧 `scene/ecs/query/query_state/helpers.rs` 已按职责硬切为 `scene/ecs/query/query_state/many_item_array.rs`，旧 `scene/ecs/storage/component_storage/utils.rs` 已按职责硬切为 `scene/ecs/storage/component_storage/component_results.rs`，旧 `asset/watch/drop_impl.rs` 已按职责硬切为 `asset/watch/shutdown_on_drop.rs`，旧 `core/framework/camera_controller/common.rs` 已按职责硬切为 `core/framework/camera_controller/controller_output.rs`，旧 `asset/tests/assets/texture_upload_readiness/common.rs` 已按职责硬切为 `asset/tests/assets/texture_upload_readiness/container_fixtures.rs`）。
- **R2.4 文件名 snake_case**（已普遍满足，纳入审计兜底）。
- **R2.5 构造目录命名**：放构造逻辑的目录用 `construct` / `construction` / `builder`，**禁用 `*_new` 后缀和裸 `new` owner**；具体 hard-cutover 记录由 Runtime 15 产出目录维护。

## §3 公共 API 与"用户友好的模块化接口"

- **R3.1 精选 façade**：子系统 `mod.rs` 只 re-export 小而有意的公共集，**分组 + 每组注释**；façade 的 `pub use` 行数纳入审计（软阈值，超阈值要求改 prelude / 分组），禁止 100 符号扁平 dump。
- **R3.2 禁 glob 出口**：子系统 / crate façade 禁止 `pub use x::*`（隐藏 surface）；owner 组内小范围分组显式 re-export 允许。
- **R3.3 prelude 分层**：**子系统级 prelude 为主**——`<crate>::<subsystem>::prelude`（如 `zircon_runtime::asset::prelude`）是消费者主入口；**crate 级 `prelude` 仅聚合**——只 re-export 各子系统 prelude 的跨子系统高频集，不直接列符号。分工：façade(`mod.rs`)=完整公共面、prelude=高频常用面；prelude 也设符号预算，防退化成第二个 dump。插件 crate 经 `lib.rs` 暴露公共 API，体量大时才加 prelude。
- **R3.4 可见性纪律**：模块非 `pub`（公共 API）即 `pub(crate)` / `pub(super)`（实现）；同一 `mod.rs` 不得无规则混排 `pub mod` / `pub(crate) mod`——若混排，façade 注释须显式标注公共集边界；稳定公共项带 doc 注释。

### 范式：巨型扁平 façade → 分组 façade + prelude（`asset/mod.rs`）
**前**：`pub use assets::{ ...100+ 符号一坨... };`
**后**：
```rust
// asset/mod.rs —— 精选 façade（完整公共面，分组）
pub mod prelude;

// —— 资产类型 ——
pub use assets::{MeshAsset, MaterialAsset, TextureAsset, SceneAsset, ModelAsset,
    UiWidgetAsset, VirtualGeometryAsset /* … */};
// —— 导入 / 校验 ——
pub use assets::{asset_kind_for_imported_asset, validate_sprite_atlas_asset,
    validate_wgsl_captures /* … */};
```
```rust
// asset/prelude.rs —— 高频常用面（设符号预算，不得扩成第二个 dump）
pub use super::{Assets, AssetManager, ProjectAssetManager,
    MeshAsset, MaterialAsset, TextureAsset, SceneAsset};
```
crate 级 `zircon_runtime::prelude` 收窄为 `pub use crate::{asset::prelude::*, scene::prelude::*, ui::prelude::*, ...}`，不再直接列子系统符号。

## §4 测试组织（单一规则）

- **R4.1**：单文件小测（< ~150 行测试）→ 内联 `#[cfg(test)] mod tests`。
- **R4.2**：更大 / 行为测试 → folder-backed `tests/` 镜像源树、按行为族分文件。
- **R4.3**：禁止 > 800 行 `tests.rs`；禁止重复测试树（如 editor `src/tests/**` 镜像 `src/ui/**` 双写）——一个行为一个 owner。
- **R4.4**：跨 crate 集成测试归 crate `tests/`；测试命名按所属子系统过滤词前缀（沿用 `render_*`、`runtime_*` guard 命名惯例），便于 milestone 末按过滤词收窄。

## §5 资源 / 描述 / manifest 放置

- **R5.1**：出货资产 → crate `assets/`（staged build 已合并 editor / runtime 两 `assets/`）；测试 fixture → owner 模块 `tests/fixtures/` 或 `<module>/tests/assets/`，禁散落。
- **R5.2**：插件 manifest → crate 根 `plugin.toml`（强制、统一 schema，见 §6.2）。
- **R5.3**：每个 descriptor 家族（`.zui` / `.zmaterial` / `.zasset` / `plugin.toml`）有唯一 schema owner 文件，reviewer 可定位 schema 权威。`.zui` 是唯一 UI asset descriptor 家族；`.ui.toml` / `.v2.ui.toml` 已退役，不得作为当前 UI layout/asset schema owner 回流。`page_templates.toml`、`shell_regions.toml` 与 `presets.toml` 属 typed editor layout metadata，不是 UI asset descriptor family，只能引用 `.zui` UI 资产。

## §6 统一插件接口开发体验框架（Plugin DX）

### §6.1 唯一插件 crate 骨架（template）
```
<plugin>/
  plugin.toml          # 统一 schema（强制，见 §6.2）
  runtime/
    Cargo.toml
    src/
      lib.rs             # 薄：pub use 公共 API + 导出 Plugin struct + 常量
      plugin.rs          # 唯一注册 owner：impl RuntimePlugin + descriptor()
      capability.rs      # capability id pub const —— 单一来源
      contract/          # 该插件 ABI-safe DTO（纯消费 interface 则省略）
      backend/           # 实际算法 / 协议实现 owner（按 §1 拆叶子）
      systems/           # 注册进调度图的 ECS 系统
      tests/
  editor/                # 镜像同骨架（能力对称：plugin.rs / capability.rs / ...）
    Cargo.toml
    src/{lib.rs, plugin.rs, capability.rs, ...}
```
- 导入器类插件：`backend/` 即 importer 实现，`plugin.rs` 的 `register` 同时 `register_module` + 注册 importer descriptor；退役 `asset_importers/*/registration.rs` 的自由函数分离。
- native-dynamic 插件：`plugin.toml` 显式区分 runtime / editor 两 `[[modules]]` 的 `crate_name`，禁止两 module 指向同名 crate 却不以 `kind` 区分。

### §6.2 统一 `plugin.toml` schema（canonical）
唯一 schema owner：`docs/zircon_plugins/plugin-manifest-schema.md`（含校验器契约）。必选 / 可选段固定形状，使 30 行与 105 行插件共享骨架：

```toml
# —— 必选头 ——
id = "<plugin>"                       # 与 capability 前缀一致
version = "0.1.0"
sdk_api_version = "0.1.0"
display_name = "..."
category = "runtime|asset_importer|editor|..."
description = "..."
supported_targets = ["client_runtime", "editor_host", ...]
supported_platforms = ["windows", "linux", "macos"]
capabilities = ["runtime.plugin.<plugin>", ...]   # 与 capability.rs 单源核对
maturity = "stable|beta|experimental"

# —— 必选模块声明（每个 crate 一条）——
[[modules]]
name = "<plugin>.runtime"             # <plugin>.{runtime|editor}
kind = "runtime|editor"
crate_name = "zircon_plugin_<plugin>_runtime"
target_modes = [...]
capabilities = [...]
system_anchors = [...]                # 与实际注册的 system 源核对

# —— 可选段（按需，schema 固定形状）——
[[capability_statuses]]   capability = "..."  status = "partial|stable"
[[asset_importers]]       id = "..."  source_extensions = [...]  output_kind = "..."  ...
[[optional_features]]     id = "..."
[[dependencies]]          id = "..."
[[options]]               key = "..."  ...
[[event_catalogs]]        ...
```
规则：① 所有插件（含 `asset_importers/*`、native）必须有 `plugin.toml`；② `capabilities` 与 `capability.rs` 常量集双向一致（审计）；③ 可选段缺省即省略，不得改变必选段形状。

### §6.3 唯一注册入口
仅经 `impl RuntimePlugin::register(&self, registry)`（Plugins 01 已硬切运行时插件 trait 到唯一 `register`）；editor 侧经对称 editor plugin trait。自由函数注册收编进 `plugin.rs` 的 `register`。运行时模块与 system 注册优先走 `plugin_sdk::registration::RuntimePluginRegistrationBuilder`，由 SDK 封装 owner token 传递与注册顺序。标准 runtime crate helper 不得逐插件手写复制；新代码使用 `zircon_plugin_sdk::runtime_plugin_exports!(PluginType)`。

当前插件架构同步（2026-07-10，`plugins_01_m2_t2_t4_typed_extension_freeze_runtime_finalize`）：`TypedExtensionPoint` 冻结与 owner 撤销回归放在独立 `extension_registry_typed_points.rs` 测试 owner，生产实现继续由 `extension_registry/` 子模块承接；未向 `plugin/mod.rs` 堆入行为，也未增加兼容 facade、re-export shim 或双轨入口。详细完成项与验证归档到 Plugins 01 和 runtime extension registry 模块文档。插件架构整体状态：进行中。

> 请将产出记录放置在子计划中，子计划中记录超过10条则全部放到子目录，此处仅展示当前现状的概述

历史迁移批次、验证与状态记录已迁入 Runtime 15 产出目录。

- 迁入记录：[`_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md`](_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md)

### §6.4 capability 单源 + 四源一致性
capability id 为 `capability.rs` 的 `pub const`；guard 测试交叉核对四源：`capability.rs` 常量 ↔ `plugin.toml capabilities` ↔ runtime descriptor ↔ workspace member。扩展现有 `declared_system_anchors_are_registered` 同款模式到 capability。

> 请将产出记录放置在子计划中，子计划中记录超过10条则全部放到子目录，此处仅展示当前现状的概述

历史迁移批次、计数与验证记录已迁入 Runtime 15 产出目录。

- 迁入记录：[`_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md`](_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md)

### §6.5 `plugin_sdk` builder（祝福路径）
把 `plugin_sdk_examples` 固化为模板与 builder/test fixture API，新插件 ≈ 一文件声明（descriptor + capability + systems 注册），runtime system 注册通过 `plugin_sdk::registration` 隐藏 owner token 样板，runtime helper exports 通过 `plugin_sdk::runtime_plugin_exports!` 投影 trait-backed manifest/selection/registration，optional feature 能力通过 `PluginFeatureBundleBuilder` 同源投影 feature/module capabilities，editor/runtime 对称通过 `EditorPluginDeclaration::mirrors_runtime(...)` 显式声明，跨插件测试通过 `plugin_sdk::test::TestRuntime::builder()` 复用 runtime/scene/fixed-step 启动样板。

### §6.6 双形态独立构建（发行维扩展）
由 [Plugins 13](zircon_plugins/13-standalone-plugin-build.md) 落地、规范权威 [`docs/zircon_plugins/plugin-standalone-build.md`](../zircon_plugins/plugin-standalone-build.md)。在 §6.1 骨架上扩"发行维"：每个插件一份声明投影两形态——`embed`（`rlib`，静态链接、`impl RuntimePlugin::register`）与 `dist`（`cdylib`，ABI-only、`zircon_native_plugin_descriptor_v3` 导出），二者共享 `backend/` 纯逻辑不复制。**依赖边界铁律**：`dist` 产物依赖闭包禁含 `zircon_runtime`（与 §7.5 E8 同源），`backend/`/`capability.rs` 禁 `use zircon_runtime::*`，触碰 `zircon_runtime` 的代码一律 `#[cfg(feature = "embed")]`；由 `tools/plugin_structure_audits/dependency_boundary.py` 的 `dist_dependency_boundary_violations` 守卫。

> 请将产出记录放置在子计划中，子计划中记录超过10条则全部放到子目录，此处仅展示当前现状的概述

具体 rollout、构建、验证、计数与阻塞记录已迁入 Runtime 15 产出目录；本节仅保留双形态独立构建规范。

- 迁入记录：[`_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md`](_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md)

### 范式：插件 crate 骨架化（`asset_importers/model`）
**前**（无 `plugin.toml`，注册在自由函数）：`src/{lib.rs(re-export), registration.rs(161 行: descriptors + manifest + plugin_registration 自由函数), mesh_importer.rs, cad.rs, tests/}`
**后**：
```
asset_importers/model/runtime/
  plugin.toml          # 新增，统一 schema
  src/
    lib.rs             # 薄：pub use + ModelImporterRuntimePlugin
    plugin.rs          # impl RuntimePlugin::register（自由函数收编于此）
    capability.rs      # pub const RUNTIME_CAPABILITY = "runtime.plugin.asset_importer.model";
    backend/ mesh_importer.rs  cad.rs
    tests/
```

## §7 强制机制（对后续开发与存量重构同时生效）

1. **审计脚本族**：runtime 进现有 `runtime_structure_audits/`（`module_convention_gate.py` + `module_convention_gate_markdown.py`，由 `audit_runtime_structure.py` 聚合）；editor / plugins 各新建 owner 域同级目录 `editor_structure_audits/` / `tools/plugin_structure_audits/`（与 `runtime_structure_audits/` 平级），各带 `audit_editor_structure.py` / `tools/audit_plugin_structure.py` 聚合器。
2. **guard 测试**：runtime `tests/runtime_absorption/structure_convention.rs`、editor `zircon_editor/src/tests/structure_convention/`、plugins workspace guard——断言审计字段与镜像文档计数一致。
3. **owner-class gate**：`module_convention_gate` / `plugin_skeleton_gate` 报告 `m1_gate_status` ∈ {`migration-debt-present`, `classified-and-clear`}，含 `classification_counts` 与 `migration_debt_count`（目标 → 0），`exempt` 字段登记豁免。
4. **镜像文档**：`docs/**/structure/*.md` 计数须与审计一致，由 `*_mirror_docs_match_structure_audit_counts` 守卫锁定。
5. **硬切纪律**：新 owner 路径落地的同一变更内迁移调用方并删除旧路径，不留 re-export / shim / 双轨；grep 旧符号零命中。
6. **milestone-first 验收**：切片期轻量 `cargo check`，里程碑末进测试 + `cargo fmt --all --check` + 运行对应 `audit_*_structure.py --json`。

每条规则都映射到某审计字段（façade 行数→`oversized_facade_files`、前缀→`prefix_vocabulary_violations`、骨架→`skeleton_conformance`、capability 单源→`capability_source_mismatches`…），确保"可验收"非空话。

## §7.5 错误处理与反重复约定（2026-06 审查并入）

来自 [`engine-code-review-findings-2026-06.md`](engine-code-review-findings-2026-06.md) 的规范级结论。具体闭合状态、验证、计数与守卫记录已迁入 Runtime 15 产出目录。

> 请将产出记录放置在子计划中，子计划中记录超过10条则全部放到子目录，此处仅展示当前现状的概述

- 迁入记录：[`_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md`](_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md)

- **E1 typed error 优先**：跨模块公共 API 返回 typed error，不用裸 `String` 或 `format!()` 压扁 source。
- **E2 getter / resolver 命名**：`get_*` 表示 optional lookup，`resolve_*` 留给 fallible/result lookup。
- **E3 builder infallible**：链式 `with_*` 一律返 `Self`；校验移到 `build()` / `finish()`，可失败应用/解析入口使用非 builder 动词。
- **E4 镜像文档**：`docs/**/structure/*.md` 计数须与审计一致，并由镜像文档守卫锁定。
- **E5 反重复样板**：近似复制的并列模块必须抽泛型、宏或 derive；diagnostics 类结构统一 trait + 子域组合。
- **E6 `#[allow(dead_code)]` 限制**：生产代码禁止长期掩盖未接线脚手架或僵尸；要么接线，要么删除。
- **E7 FFI panic 边界**：所有 `extern "C"` 边界必须包裹 panic guard，panic 转状态码，不跨 FFI。
- **E8 边界依赖白名单**：`zircon_runtime_interface` 禁 wgpu/slint/winit/tokio；`zircon_editor`/`zircon_app` 允许窗口/事件循环依赖但禁 graphics backend 泄漏。
- **E9 生产锁 poison 处理**：运行时生产共享状态不得直接 `.lock().unwrap()`；infallible owner API 通过集中 helper 恢复，fallible API 返回 typed error。
- **E10 可失败 render submit 降级**：`submit_frame_extract` production paths must return `RenderFrameworkError`；viewport/provider 缺口不得用裸 `.unwrap(`/`.expect(` 维持不变量。

## §8 各计划集落地索引

| 计划集 | 落地 | 范围 |
|---|---|---|
| Runtime | `zircon_runtime/runtime/15-code-structure-and-module-conventions.md` | runtime 全模块 + graphics（render 子计划引用本文 §1/§5） |
| Editor UI | `zircon_editor/editor_ui/10-code-structure-and-module-conventions.md` | editor `core/scene/ui` |
| Plugins | `zircon_plugins/12-plugin-dx-and-structure-framework.md` | 全插件 DX + manifest/骨架/注册/capability |
| Plugins 发行 | `zircon_plugins/13-standalone-plugin-build.md` | 双形态独立构建 + 依赖边界 + 注册跨 ABI 编组 + per-plugin 动态包 |
| Render | `zircon_runtime/render/index.md`「代码结构规范」节 | graphics 热点纳入 Runtime 15 + `large_file_ownership_gate` |
| Hub | `zircon_hub/index.md`「代码结构规范」节 | Hub 巨型文件 + 前端组件化纳入本文 §1/§3/§4 |
| 审查发现目录 | `engine-code-review-findings-2026-06.md` | F1–F19 + D 系列 DX 发现，分派到各计划 |


## Runtime 15 M3 Review-Guard Row-Data Routing

Runtime 15 M3 review-guard row-data 的具体 cross-doc 与 supplemental anchors 已迁入 [`_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md`](_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md)，共享 current-owner inventory 由 [`zircon_runtime/runtime/15/2026-07-10-priority-plan-doc-current-owner-inventory.md`](zircon_runtime/runtime/15/2026-07-10-priority-plan-doc-current-owner-inventory.md) 持有；本文件继续只持有结构规范、接口约定与当前现状概述。

## 2026-07-13 M4 behavior postprocess tests owner split hard-cut note

- M4 behavior postprocess tests owner split 继续执行 §4 单一规则：`graphics/tests/m4_behavior_layers.rs` 只保留 shared fixture、offline-bake coverage 与 child mounts，`graphics/tests/m4_behavior_layers/postprocess.rs` 持有 bloom/color-grading 产品测试。
- 守卫 `runtime_15_m4_behavior_postprocess_tests_are_child_owner` 锁定父/子 800 行预算；旧测试或 helper 不得回流父文件。具体状态与验证证据只由 Plan09 / Render 编号归档持有，本总览不复制 machine status token。

## 2026-07-10 Runtime Text UAX#9 line-owner hard cut note

- The existing Runtime 15 visual-order child-owner split remains intact: `ui/text/layout_engine/visual_order.rs` is still the narrow adapter owned below `layout_engine.rs` and is now 178 lines.
- Algorithm authority is hard-cut to `text/shaping/bidi.rs`; the UI child no longer owns ASCII/RTL-block classification, neutral-span direction resolution, or a duplicate mirror table.
- `runtime_15_ui_text_layout_engine_visual_order_is_child_owner` now locks the shared `analyze_bidi_line` / `mirrored_bidi_char` calls and the parent call boundary; no compatibility facade or old algorithm path remains.
- Locale-specific cosmic state is isolated under `text/shaping/cosmic/font_system_cache.rs`; `cosmic.rs` remains the backend adapter/orchestrator instead of accumulating cache policy.
- The cache is explicitly bounded to four `FontSystem` instances and reuses one seed database, closing the review concern that arbitrary application-language values could grow persistent backend state without limit.
- The boundary is documented exactly: locale configures cosmic platform fallback selection, while the canonical per-segment RustyBuzz path applies request language and `locl`; cosmic-text remains a whole-request fallback rather than a metadata source for a second projection pass.
- SH-M3 vertical policy is child-owned under `text/shaping/vertical.rs`, `vertical/orientation.rs`, and `vertical/direct.rs`; `cosmic.rs` invokes the direct owner first and only builds a cosmic buffer when that complete request cannot be shaped directly, while `ui/text/layout_engine/vertical.rs` consumes the vertical provider instead of reimplementing Unicode orientation.
- The provider hard cut preserves the existing cache authority: vertical orientation/mode are part of `ShapedRunCacheKey`, and UI wrapping/ellipsis/measurement no longer create horizontal cache entries for VerticalRl content.
- Native `vmtx` advance remains isolated under `text/font/vertical_metrics.rs`. TTB/BTT shaping is split between `shaping/vertical/backend.rs` and `vertical/direct.rs`; shared logical itemization lives in `shaping/itemize.rs`, horizontal DTO construction lives in `horizontal/direct.rs`, and `orientation.rs` owns Unicode rotation policy. The former horizontal/vertical projection owners are deleted, so backend vertical-origin/VORG-side-bearing values reach the renderer without a compatibility wrapper or a second shaping pass.
- V1 normalization policy now has a narrow `text/shaping/normalize.rs` owner. Cosmic/fallback consume its identity view and source projection instead of embedding an unreviewable offset assumption in the backend adapter.
- Text 03 vertical column capacity, right-to-left frame placement, and cross/main axis extents moved to `text/layout/vertical_layout.rs`; the UI child consumes the shared result and retains only CandidateLine/rich/ellipsis/UiResolved DTO projection.
- The production SDF VerticalRl consumer calls the same shaping owner, while `render/text_advances.rs` projects source-cluster advances, `sdf_atlas/text_keys.rs` owns shaped glyph/face key collection, and `sdf_render/vertices.rs` maps vertical origin/rotation into destination frames and UVs. `render.rs` is 712 lines, `sdf_atlas.rs` 611, and no production file crosses the 800-line review warning; no old scalar-only vertical success path or compatibility shim remains.
- Native bitmap atlas now follows the same owner-module rule end to end: the 29-line
  `text/native_bitmap_atlas.rs` root is declaration/re-export wiring only; frame state,
  source-image details, per-frame budget, and frame driver live in
  `text/native_bitmap_atlas/frame.rs`; prepare-report construction and text-area input have
  dedicated `report.rs` and `text_area.rs` children. Mixed-storage partitioning remains
  child-owned by `text/native_bitmap_atlas/storage.rs`. It partitions contiguous storage runs
  rather than globally grouping equal formats, so repeated `R8 -> RGBA -> R8` order survives as
  three renderer passes without reintroducing root-level ordering policy or retaining the former
  glyphon fallback as a supported success path.
- Mixed-BiDi hit source/affinity policy is isolated under `ui/text/hit_test/visual_source.rs`; the parent hit-test owner performs geometry selection and consumes the leaf result, while visual-order no longer merges descending logical clusters into lossy ranges.

## 2026-07-10 Runtime Text backend face-ID owner hard cut note

- Third-party identity stays isolated under `text/font/backend.rs`; `fontdb::ID` does not leak into core/framework DTOs, which continue to expose `FontFaceId`.
- Process sharing is child-owned by `text/font/shared.rs`, while locale cache refresh remains in `shaping/cosmic/font_system_cache.rs`; neither policy is stacked into the text renderer root.
- `shaping/font_id.rs` and its post-shape annotation path were physically deleted. Cosmic and native reporting consume actual `LayoutGlyph.font_id`; the structure guard now rejects the removed bridge symbols instead of preserving a facade or shim.
- The slice introduced no production `allow(dead_code)`: deleting the bridge also deleted its newly orphaned cluster resolver/source helper, restoring the existing 416-warning library baseline.
- Follow-up fallback/diagnostic reporting did not grow the renderer root past its soft budget: `text/prepare_report.rs` now owns prepare/raster/missing-glyph report DTO aggregation, and `text.rs` is back to 777 orchestration lines; the structure guard rejects moving those declarations back into the parent.
- Run language remains a backend-neutral field on `zircon_runtime_interface::UiResolvedStyle`; normalized layout/shaped cache identity stays in their cache owners, while SDF locale identity stays in `sdf_atlas.rs`. `render.rs` and `text.rs` only propagate the value and do not become a second locale-policy owner.

## 2026-07-11 Runtime Text screen-space font initialization note

- System-font policy remains owned by `text/font`: the screen-space renderer invokes the narrow `initialize_screen_space_ui_font_system(...)` boundary and does not duplicate `fontdb` enumeration, family tables, or platform-font constants.
- The fix adds no Editor-only font route, compatibility module, root facade, or test glyph injection. Runtime and Editor consume the same `FontDatabase -> glyphon FontSystem` synchronization path.
- The Windows lower regression remains in the existing folder-backed `scene_renderer/ui/text/tests.rs` owner, so production `text.rs` retains orchestration rather than accumulating test fixtures or platform assertions.
- The real HUD framebuffer gate passes after the same bounded 24-frame async-text settle policy used by the Runtime product test; waiting policy stays in test/product validation rather than becoming a production rendering bypass.

## 2026-07-17 Runtime Text shared font generation stability note

- Process-wide publication remains owned by `text/font/shared.rs`; the semantic comparison is a folder-backed child at `text/font/database/equivalence.rs` rather than new policy in `text.rs`, `render_state.rs`, or the scene renderer.
- The comparison covers only inputs that can change shaping/fallback/raster output: ordered face descriptors and sources, fallback families, CompositeFont, and default UI family. Derived indexes, diagnostics, and runtime caches do not become false invalidation inputs.
- Database replacement and generation advancement share one write-lock critical section, so snapshots cannot observe a new database with an old generation. Shaping/raster hot paths retain the existing atomic generation probe and immutable snapshot refresh boundary; no production test lock or compatibility wrapper was added.
- Shared font bytes take an `Arc::ptr_eq` fast path, making the ordinary clone-and-republish comparison proportional to face count rather than font payload size. Deep byte comparison is reserved for independently materialized equivalent databases on the low-frequency publish path.
- System-font discovery state remains a private `FontDatabase` concern. Because `fontdb 0.23` appends a new backend catalog on each scan, repeated `Discover` calls now return before directory I/O once the policy was applied; renderer clones inherit the state. No scene-renderer singleton, process-global refresh shim, or duplicate backend database owner was added.
- System-policy regressions are folder-backed under `text/font/database/tests/system_policy.rs`; they no longer grow the near-budget `database/tests.rs` root. The split is by discovery/coverage ownership, not equal line counts, and introduces no test facade or duplicate fixture owner.
- `TextRenderState` now constructs cosmic `FontSystem` directly from the shared backend database and a text-owned normalized system-locale helper; it no longer creates a temporary `FontSystem::new()` that independently scans OS fonts before replacement. The raw fallback locale is a single private constant in `text/language.rs`, while `sys-locale` is optional behind the existing `text` feature.
- Async raster work owns font bytes plus a face epoch, not an atlas page. The former fixed `page_generation=0` field and stale-page worker telemetry were hard-cut instead of preserved as a placeholder API; actual page generations remain in the existing allocation/staging/upload children where the page is known. This keeps CPU source caching independent from page residency while retaining fail-closed WGPU upload guards.
- The existing per-page dirty-rect merge stays in `text/atlas/bitmap_run/upload.rs`; its exact performance contract lives in the folder-backed `bitmap_run/tests/dirty_upload.rs` child. Renderer root and texture binding do not gain a second merge policy.
- Generation-sensitive SDF regressions moved from the already-large `font_bake/tests.rs` root into `font_bake/tests/cache_generation.rs`. Production `font_bake.rs` and `font/database.rs` remain below the 1000-line hard limit (the 2026-07-28 current-source audit reports 792 and 553 lines respectively), and the new behavior owners remain narrow.
- Test-only force-publish and read-guard helpers preserve direct stale-handle testing and isolate parallel global-generation fixtures without changing the production API. The implementation is active pending the expanded shared/SDF/default/UI/upward testing stage, so this note records ownership and invariants rather than acceptance.
- The same MVP stability pass removes the production `expect` from `text/parallel/raster_pool.rs::request(...)`. A missing request sender now returns the existing neutral `CoreError::ChannelSend` before touching in-flight state; a test-only disconnect hook owns the unreachable-state regression. No renderer-local fallback, compatibility API, or second queue owner was introduced.
- Primary full-coverage projection in `text/shaping/fallback_spans.rs` now derives its face from the same filtered resolver expression instead of repeating an optional-state `expect`. `text/sdf/font_bake.rs` also treats an unexpected ensure/map mismatch as a rejected candidate and reaches the existing fallback metrics. Both remain inside their existing leaf owners and add no upper-layer success path.

## 2026-07-31 Runtime Text rich layout advance/materialization owner note

- `text/layout/advance_index.rs` is the shared sorted grapheme/prefix-advance storage and range-query owner for plain and rich layout. `rich_advance_index.rs` only compiles style/inline spans into that owner; it does not introduce a second shaping backend or scalar-width approximation.
- `text/layout/rich/materialize.rs` borrows the canonical source, consumes runs with a monotonic cursor, preserves original run indices, and projects item advances from the shared index. Per-line full-source clones and all-run filters were deleted rather than hidden behind a compatibility helper.
- Horizontal inline/run metrics are isolated in `text/layout/rich/metrics.rs`; parser policy remains in `text/rich`, vertical axis policy remains in `rich_vertical.rs`, and renderer/UI roots gain no layout policy.
- Boundary policy is centralized in `text/layout/line_break/boundary_correction.rs`: plain and rich share one tentative metric-range planner, prefix raw advances, an 8-grapheme correction radius, and at most 16-grapheme shape windows. Indexed queries materialize only the leading/trailing 16-unit windows; they do not collect a growing complete line.
- Production owners are bounded at `rich.rs` 279 lines, `rich/materialize.rs` 258, `advance_index.rs` 193, `rich_advance_index.rs` 399, `rich/metrics.rs` 77, `rich_vertical.rs` 232, and `boundary_correction.rs` 358. UI owners remain bounded at `wrapping.rs` 417, `rich_inline.rs` 315, and `rich_inline_vertical.rs` 273. All stay below the current 800-line production review warning. The slice adds no production `allow(dead_code)`, `panic!`, `unwrap()`, or `expect()`.
- `ui/text/layout_engine/wrapping.rs` consumes the shared index with accumulated advances and bounded Word/Glyph boundary queries. The former growing-prefix String/reshape helper and provider-based glyph-fallback admission path were deleted rather than retained as a facade; newline slicing remains borrowed and tests stay in `wrapping/tests.rs`.
- Soft-hyphen ownership remains single-source: the line-break owner exposes suffix/source-range metadata; plain UI retains `CandidateLine` pending suffix semantics, and rich horizontal/VerticalRl projections append the same synthetic `-` run plus measured advance. No renderer-local suffix policy was added.
- The non-validation implementation and post-fix review are complete but not accepted; the review reports P0=0/P1=0. Managed Cargo, run-scale/p50/p95 evidence, Text09 cache regressions, and a fresh product framebuffer remain explicit Text03 follow-ups.

## 2026-08-01 Runtime Text artifact and vertical-provenance owner note

- `UiResolvedTextLayout` carries one opaque, type-erased `Arc` artifact for rich text. The runtime-only resolver downcasts that handle; extract, renderer, texture preparation, and link hit consume the layout-owned artifact instead of re-parsing markup or retaining a global artifact registry. The handle's allocation is released with the last layout/command clone, so idle UI work cannot retain compiled rich payloads in a registry.
- `ui/text/rich_text.rs` only registers a compiled artifact when the command has no resolvable layout artifact. It never overwrites the layout-time `Arc` after cache eviction or parser/decorator generation changes. The extract regression pressures the bounded parser cache before preparation and then asserts pointer identity plus link resolution.
- `text/shaping/vertical/backend.rs` uses two RustyBuzz shapes only for `TransformOrRotate` provenance requests: normal vertical features, then the identical buffer/non-vertical feature set with `vert` and `vrt2` explicitly disabled. Cluster glyph sequences are compared once with indexed maps; rotation is driven by actual output differences, not a GSUB lookup-output superset. Non-provenance vertical segments remain single-shape.
- `text/layout/rich_source.rs` documents the local ordered, non-overlapping run invariant required by `rich_advance_index.rs`. Table/cell projections preserve the parent compiled artifact and stable original run indices; they do not clone source text or style/link/inline metadata.

## 2026-07-11 Runtime Text rich parser owner split note

- Rich-text contracts stay backend-neutral under `core/framework/render/text/rich.rs`; parsing and security policy are not leaked into UI DTO roots or scene-renderer owners.
- `text/rich/parser.rs` remains the orchestration owner, while BBCode, decorator registration, and controlled HTML rules are separate folder children. `html_subset.rs` and `parser.rs` remain below 500 lines each (current parser 434), avoiding another oversized parser root.
- `ui/text/rich_text.rs` remains a narrow Markdown compatibility adapter over the shared parser; it does not retain a second parser or compatibility implementation.
- Grapheme boundary policy is applied once after markup stripping, and the three layout regressions were updated to assert whole-cluster runs rather than preserving a half-cluster legacy shape.
- Image/link parsing stays in the same HTML/BBCode leaf owners and uses the existing neutral `InlineObjectRef`/`LinkRef` contracts; no UI-local duplicate resource parser was added.
- No HTML/CSS crate, script bridge, network loader, root facade, re-export shim, or production `allow(dead_code)` was introduced. `LayoutItem::Inline` remains explicitly open rather than hiding an unused metric helper in production.

## 2026-07-11 Runtime Text rich inline-layout owner note

- `text/layout/rich.rs` is the narrow 03 owner for rich run-to-item projection and inline baseline metrics; parser policy remains under `text/rich`, and the UI/scene renderer roots do not duplicate its ascent/descent rules.
- Backend-neutral `LayoutItem`, `LaidOutLine`, and `LaidOutText` stay under `core/framework/render/text/rich.rs`. The owner records actual emitted item counts, so rejected source ranges cannot leave stale line indices.
- Text run origins are projected from the enlarged rich-line baseline instead of remaining pinned to `y=0`; Baseline/Center/Top/Bottom image modes share one metric conversion path.
- The child owner is under 300 lines and adds no compatibility facade, production `allow(dead_code)`, production panic/unwrap/expect, backend type, or duplicated renderer policy. UI resolved-layout/image-batch/link-hit integration remains explicitly open rather than being represented as a completed render path.
- The next integration cut keeps responsibilities leaf-owned: `ui/text/layout_engine/rich_inline.rs` projects the admitted single-line inline subset, while `scene_renderer/ui/render/rich_text.rs` owns renderer-side range/style/placement interpretation. The public style contract hard-cuts the Markdown-only boolean to `UiRichTextFormat`; no bool-to-format compatibility field or second parser survives.
- The renderer consumes the shared `LaidOutText` placement and never submits U+FFFC as a glyph batch. Image runs now route through folder-backed `scene_renderer/ui/image.rs`; the general color-quad pipeline and text renderer do not absorb texture bindings or WGSL sampling policy.
- UI texture preparation remains under the existing `ResourceStreamer`: `resources/ui_texture.rs` resolves locator-stable IDs against imported UUID-backed records, rejects non-D2/non-single-layer payloads, and returns the existing fallback on failure. No second GPU texture cache, asset loader, renderer-root resource map, or interface-level WGPU type was added.
- The leaf sizes remain bounded (`resources/ui_texture.rs` 139 lines, `scene_renderer/ui/image.rs` 259 lines). Rich run planning and placement now live in `scene_renderer/ui/render/rich_text.rs` (197 lines), which keeps the renderer root at 794 lines rather than the stale 866-line count; the real product framebuffer gate—not a policy diagram—proves both texture sampling and ellipsis-retained inline placement. Concrete evidence remains in the Text07 numbered archive.
- Vertical rich layout is split by responsibility: `text/layout/rich_vertical.rs` owns main/cross-axis metrics and wrap ranges, while `ui/text/layout_engine/rich_inline_vertical.rs` only projects those metrics through the shared VerticalRl column-capacity/placement and ellipsis owners. Object height advances y, object width expands the column; no second Unicode orientation, BiDi, texture loader, or renderer-local layout policy was introduced.
- The vertical addition keeps the production leaves bounded (`rich_vertical.rs` 322 lines, `rich_inline_vertical.rs` 239, `render/rich_text.rs` 236). Renderer rich tests moved to `render/tests/rich_inline.rs` (216 lines), leaving the test root at 779; product command builders moved to a 151-line child, leaving the integration root at 771. The renderer production root remains 794 lines.
- BBCode block alignment stays in the same leaf-owned chain: `text/rich/parser.rs` emits neutral `ParagraphOverride` ranges, while `ui/text/layout_engine.rs` only projects effective per-line alignment. No center/right parser or markup-range policy is duplicated in UI or renderer, and the rich parser remains below the oversized-file threshold.
- Rich-link input stays folder-backed: `ui/text/rich_text/link_hit.rs` owns caret-affinity/range resolution, `ui/surface/input/rich_link.rs` owns pointer admission, and `ui/surface/input/effect/link.rs` owns scheme/owner validation. `pointer.rs` only invokes the leaf after normal routing; the public interface carries neutral effect/host-request DTOs, with no browser backend or network dependency crossing E8.

## 2026-08-01 Runtime Text atlas instance-render owner note

- Text04 的 CPU draw artifact 与 WGPU payload 已硬切为 instance contract：`text/atlas/render_batch.rs` 只按相邻 page/render contract 形成有序 batch，`render_gpu_plan/instance.rs` 只定义一个 68 B packed instance；已删除的两个 `vertex.rs` owner 不保留 alias、facade 或兼容重导出。
- 固定 quad geometry 归 `text/atlas/shaders/glyph_atlas_pipeline.wgsl`，viewport pixel-to-NDC 归 vertex uniform；CPU plan 不拥有 corner vertex 或 NDC projection。`atlas_renderer/instance.rs` 只映射 WGPU layout，`instance_buffer.rs` 只拥有 capacity/growth/write 生命周期，renderer root 只编排资源与 pass。
- current-source owner 尺寸为 `render_gpu_plan.rs` 134 行、`instance.rs` 100、`draw_command.rs` 66、renderer root 769、renderer `state.rs` 148、WGPU `instance.rs` 63、`instance_buffer.rs` 75；production 均低于 800 行 review warning，且本切片的 production panic/unwrap/expect/dead-code allow 与旧 vertex symbol 扫描均为 0。
- 1/100/1k/10k 线性回归仍留在各自 folder-backed test owner：text plan test 锁定 `N` instances、0 CPU quad vertices、`68N` bytes 与 painter-order draw count，renderer test 锁定 resizable buffer 的稳态不扩容。31-sample p50/p95 exporter 为 ignored evidence test，不进入生产 owner。
- 非验收实现与二次静态审查已完成，但结构记录不等同运行时 acceptance；managed Cargo、ignored metrics 与真实 WGPU framebuffer/新 PNG 继续由 open Text04 failure 持有。

## 2026-08-01 Runtime Text native bitmap residency owner note

- `TextRenderState` 是 source/slot/page 失效协调 owner；`native_bitmap_atlas/source_cache.rs` 只拥有 CPU bytes、entry hard cap、indexed LRU 与 `CacheKey <-> GlyphRasterKey` 反向索引，`GlyphAtlasSet` 只拥有 slot/page generation/allocator/shadow，renderer 只返回 upload 成败。三层之间不复制预算或页状态。
- source pressure 先发出已绑定 raster key，atlas owner 定位并原子失效整个 page，再定点回传该页全部 source keys；atlas eviction 通过 `GlyphAtlasBitmapRunPlan.invalidated_raster_keys` 显式交接，禁止依赖跨帧隐藏队列。failed upload 对 page keys 去重后每页只推进一次 generation，并在下一帧 report 中暴露 source invalidation。
- 正常 exact hit、LRU touch/insert/evict 与 key 反查保持 O(1)；近似 lookup 直接构造至多三个 vertical-bin key。预算压力路径只遍历被失效页的 slots，不扫描整个 source cache。
- production owner 当前为 source cache 731 行、frame-report leaf 85、LRU leaf 243、atlas page 527、page residency 208、slot cache 93、render state 396，全部低于 800 行 production review warning。source cache 继续只持缓存、队列回流与驻留状态；frame diagnostics DTO/worker-pool projection 已硬切到 `source_cache/report.rs`，后续诊断字段应进入该 leaf，新的缓存或驻留职责仍须先提取具名 owner。规模/驻留测试按语义拆到 641 行父 owner 与 212 行 `tests/source_cache/residency.rs` 子 owner，均低于 1000 行测试阈值。
- 本切片没有兼容 facade、第二套 cache/slot owner、production `allow(dead_code)` 或 panic/unwrap/expect；非验收实现和二次静态审查已完成，但 managed Cargo、ignored p50/p95、真实 WGPU framebuffer 与新 PNG 仍归 open Text04 failure，本文不提前记为 accepted。

## 2026-08-01 Runtime Text Auto raster route owner note

- `text/raster/policy.rs` 是 threshold/effects/hysteresis 的唯一策略 owner；`scene_renderer/ui/text/resolved_batches/auto_route.rs` 只拥有跨帧 route state、generation cache、容量/idle 生命周期与 telemetry；`render.rs` 只投影稳定 identity；`ScreenSpaceUiTextSystem` 只持有 router。stateless 测试入口与产品 router 共用同一个 effects→policy request helper。
- route identity 明确为 `UiTreeId(Arc<str>) + UiNodeId + source_range`，command/layout generation 每条 command 读取一次并传给普通 line、rich run 与 inline icon。禁止数组位置、text hash 或仅 node-id 的跨帧身份；tree string 每帧只分配一个 Arc，batch/key 只克隆 handle。
- router 的 2048 entry hard cap、300 idle frames 与 tokenized recency queue 是有界状态 owner；generation hit 为 O(1)，只有 recency 超过逻辑容量四倍时才按 live entries 压缩。未使用 Auto 路由时不预分配 2048-entry 容器。
- current-source production owner 为 auto-route 265 行、resolved-batches 180、render 771、rich-render 278、text-system 579、policy 275、interface command 523，全部低于 800 行 warning；测试 owner 453 行。没有 facade、第二策略 owner、production dead-code allow 或 panic/unwrap/expect。
- 非验收实现与二次静态审查已完成，但结构记录不替代 managed Cargo、ignored metrics、stable-route raster-miss 与真实 WGPU framebuffer/新 PNG；Text04 failure 继续保持 open。

## 2026-08-01 Runtime Text SDF generation/residency owner note

- generation source ownership is folder-backed under `text/sdf`: `generation_source.rs` owns parsed-face lifetime and batch generation, `generation_scheduler.rs` owns bounded admission/completion/cancel state, while `font_bake/source_context.rs` owns generation-local identity/LRU. Runtime and offline paths do not retain separate face parsers or font-hash owners.
- `font_bake/offline_source.rs`, `glyph_cache.rs`, and `atlas_pages.rs` separately own offline artifact/bitmap residency, runtime baked-glyph residency, and persistent CPU atlas pages. Entry/byte limits, recency, eviction, and reports stay with those leaves; `font_bake.rs` remains orchestration and report contracts.
- SDF text identity uses run-owned `Arc<str>` for font/family/language. `sdf_atlas/text_keys.rs` normalizes and allocates once per text batch; glyph keys only clone handles. No String compatibility key, facade, flat atlas pixel owner, or deleted `atlas_write.rs` alias remains.
- Compiled frame ownership is split by artifact rather than copied into the renderer root. `font_bake/prepared_atlas.rs` caches an exact slot/atlas bake with shared `Arc<[T]>` metadata; `ui/text/sdf_cpu_frame.rs` caches exact prepared-run and native-decoration inputs; `sdf_render/compiled_frame.rs` owns material/draw/text-range/vertex reuse. Deferred or pending generation bypasses prepared-atlas reuse, and font generation, fallback mutation, atlas resize, upload deltas, or any exact input change invalidates the relevant layer.
- Generation failure projection is single-owner and allocation-stable under `sdf_atlas/generation_failures.rs`: the atlas records the one final bake artifact, validates slot index plus key, and preserves the run failure vector when the same shared failure slice recurs. `ScreenSpaceUiTextSystem` owns CPU preparation and the one allowed whole-batch fallback/rebake; the WGPU renderer no longer owns font-generation refresh, CPU run preparation, or a second generation-failure probe.
- GPU lifecycle remains under renderer children: `sdf_render/atlas_resources.rs` borrows page-local pixels and checked offsets; `sdf_render/vertex_buffer.rs` owns capacity/hash/write; `sdf_render/compiled_frame.rs` controls exact reuse of persistent CPU vectors. Dirty-page merge remains in `sdf_upload.rs`; the renderer root only orchestrates the accepted bake and draw artifact.
- Current production sizes are scheduler 447, generation source 219, `font_bake.rs` 724, atlas build/pages 223/333, async/dynamic 302/200, source/offline/glyph caches 202/364/124, prepared atlas 108, SDF atlas root/failure leaf 732/49, text keys 116, CPU frame 127, renderer root/compiled frame/vertex buffer 479/90/78 lines. All are below the 800-line warning; production panic/unwrap/expect/dead-code allow and removed-owner scans are 0.
- Non-validation implementation and the required second static review are complete for this slice, but the structure note is not acceptance. Managed Cargo, 1/100/10k scale metrics, device-loss and real WGPU/RenderDoc evidence, plus a fresh real-rendered PNG under `docs/tests/runtime/text` remain open Text05 validation work.

## 2026-08-01 Runtime Text shaping model/single-backend ownership note

- `text/model/shaped_run.rs` owns the 4 B `Iso15924Tag`, shared `Arc<str>` run source, range-only shaped line and canonical request scope. Shaped lines remain backend hard-line projections; soft wrapping/final `LaidOutLine` stays under Text03.
- `text/cache/shaped_cache.rs` owns bucket hashes plus exact canonical feature/kerning equality and conservative resident-byte accounting. Its focused tests live in `cache/shaped_cache/tests.rs`; the production owner is 759 lines and the child test owner is 259 lines.
- `text/hard_line.rs` is the mandatory hard-line owner shared by shaping, Text03 layout and UI source segmentation; it retains content plus CR/CRLF/LF/VT/FF/NEL/LS/PS separator ranges. `text/shaping/itemize.rs` owns grapheme/BIDI-level/script/fallback-face-instance/orientation segments. Horizontal, vertical, cosmic and synthetic fallback project separators as zero-advance mandatory virtual glyphs, while Text03 measure/rich/line-break consume the same ranges and prohibit kinsoku/word-smart merges across mandatory boundaries.
- `cosmic.rs` remains the narrow router and whole-request fallback owner, while `cosmic/hard_lines.rs` normalizes raw fallback rows to the shared separator-aware contract. Cosmic raw `line_i` uses the backend LF paragraph offsets, baseline is relative to each row, and shared hard-line data is built once for normalization. No production path consumes cosmic glyphs and then replaces segments with a second backend; actual `LayoutGlyph.font_id` is authoritative, and an unmapped backend face fails closed instead of guessing a preselected span.
- Vertical `Tr` clusters first enter the script-aware TTB/BTT backend with `vert`/`vrt2`; an otherwise identical shape with those two features forced off distinguishes real vertical substitution from `locl`, variation selectors and user features before rotation fallback. Horizontal direct line metrics aggregate scaled ascent/descent/line-gap from actual segment faces. Direct cluster ranges and BIDI line order use single linear cursors/passes, with 0.8em reserved for absent metrics and empty lines.
- Current Text02 production owners remain below the 800-line warning. The non-validation implementation and all findings from the completed independent review passes are forward-fixed; the real Windows WGPU harness now targets `docs/tests/runtime/text/runtime_text_mvp_foundation_product_framebuffer_20260801.png`, but this structure record and path preparation do not substitute for managed validation or an actually generated and inspected product screenshot.
