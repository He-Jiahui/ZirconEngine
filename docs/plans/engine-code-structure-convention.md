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

当前文本计划同步（2026-08-01）：rich/vertical prewarm、cache identity、raster/upload report、backend face-ID authority、fallback span 和 run-language identity 都由对应的 Text 子 owner 管理；renderer root 仍只负责编排。系统 `fontdb` face 的容器字节由 `FontDatabase` 按权威 backend ID 物化，竖排继续复用 `text/shaping/vertical/orientation.rs`，SDF consumer 不另建 Unicode 或字体策略。新的 runtime UI → screen-space text → WGPU readback 产品 harness 已固定输出到 `docs/tests/runtime/text/runtime_text_mvp_foundation_product_framebuffer_20260831.png`，覆盖 CJK/RTL/Emoji/Native/SDF、富文本、表格和 VerticalRl；截至当前该 PNG 尚未由 current-source managed run 生成，旧截图不是接受证据。live typography、scroll 增量和产品像素验收继续由 Text02/Text03/Text04/Text05/Text07/Text09 子计划保持 open；具体证据只在这些 child records 中维护。

当前文本 collection owner 校准（2026-08-29；2026-08-30 Core 注入补强）：`text/font/shared.rs` 的进程集合仅保留为默认适配器；shaping/layout/cache/artifact 的内部身份必须使用 `(collection_id, generation)` revision，并由长期 `SharedTextLayoutSession` 持有 exact `FontCollectionService`。覆盖/metrics、physical/logical fragment、artifact projection 与 renderer line lease 必须消费该 owner 的 immutable snapshot，禁止在下层以裸 generation 重抓进程集合。Graphics 创建链、screen-space `TextRenderState`/text system/renderer、动态 Runtime project surfaces 以及 HUD/menu 回退 extract cache 均从 Core manager 接收显式 collection；回退 cache key 读取自身 layout session generation，不能读取裸全局 generation。window/PIE 与 project/no-project 产品运行、current-source managed WGPU/PNG/profile/power 仍开放；权威状态见 `zircon_runtime/text/01`、`02`、`09` 与 optimize Runtime80/81。

2026-08-29 owner-module follow-up：`text/module.rs` 现在是唯一 TextModule descriptor 与 Core Services
registration owner，`text/font/runtime_asset.rs` 是项目字体 admission/retire owner，UI surface 依赖扫描
位于 `ui/surface/render/font_dependencies.rs`，动态 UI 聚合/计数编排位于
`dynamic_api/session/runtime_ui/font_admission.rs`；renderer `text/font_assets.rs` 只保留资源缓存和消费适配。
动态 UI 首次布局采用 build → dependency admission → layout 三阶段，避免在 renderer 或 surface builder
各自维护字体注册算法。新增生产文件均低于 800 行；本轮仅做 rustfmt/scoped diff 静态检查，managed
Cargo、WGPU/PNG 与性能/功耗数据仍待受管执行。

owner scope 的回收合同仍开放：不得让 `runtime_ui.rs` 按自身 surface 集合删除 collection owner，也不得让
renderer cache 独占 project/session 生命周期真值。跨 HUD、菜单、surface 与 renderer 的 claim/release 必须
由 Text Core collection owner 聚合，并以一次 generation publication 更新 active set。

当前 Render F16 owner 同步（2026-08-30）：compiled-scene `render.rs`只保留frame阶段编排，foundation、mesh submission preparation、graph-frame preparation与accepted-frame success commit分别由具名child承接；root为481行。资源绑定root把SSAO params绑定迁入58行child后为153行；scene submit root把HZB readback投影与137行folder-backed tests迁出后为627行。既有F16守卫继续执行严格`render < 500`、binding `< 160`、submit `< 650`，没有把预算放宽到通用800行warning。该切片同时承接history clear并入scene packet的source ownership，但managed Cargo、WGPU、PNG/RDC、profile与功耗仍pending，不计结构或渲染accepted milestone。

当前 frame submission contract owner 同步（2026-08-30）：逐帧logical/physical submission统计进入独立93行`frame_submission_metrics.rs`，typed physical boundary进入独立12行`frame_submission_boundary_reason.rs`，producer/resource/reason/ticket组合与合法配对进入90行`frame_submission_producer_record.rs`。`frame_submission_receipt.rs`只保留回执身份、顺序验证和统计附着合同，当前334行；完整receipt测试为203行folder-backed child。transaction root/tests进一步拆为174/228行；Runtime09A结构守卫root/child为351/122行。backend映射和compiled/legacy terminal frame owner只消费backend-neutral DTO，不把`WgpuSubmissionMetricsSnapshot`泄漏到core contract；纹理具体reason只在resource streamer赋值，错误producer配对在transaction修改前fail closed。动态Cargo/WGPU/profile仍pending，最新状态为`runtime09a_texture_mip_preservation_typed_boundary_source_implemented_static_checks_passed_dynamic_validation_pending`。

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

- The existing Runtime 15 visual-order child-owner split remains intact: `ui/text/layout_engine/visual_order.rs` is still the narrow adapter owned below `layout_engine.rs` and remains below the 800-line production review budget.
- Algorithm authority is hard-cut to `text/shaping/bidi.rs`; the UI child no longer owns ASCII/RTL-block classification, neutral-span direction resolution, or a duplicate mirror table.
- `runtime_15_ui_text_layout_engine_visual_order_is_child_owner` now locks the shared `analyze_bidi_line` / `mirrored_bidi_char` calls and the parent call boundary; no compatibility facade or old algorithm path remains.
- Secure display substitution uses the same boundary: `text/shaping/bidi.rs` owns source-free `BidiLineSignature`, which captures resolved scalar levels plus L1-relevant class metadata and resolves the final physical line after wrapping. `visual_order.rs` only consumes the resulting `BidiLineOrder`; it must not infer direction from neutral password-mask glyphs or reimplement UAX#9.
- Secure glyph projection stays under `text/glyph_artifact/visual_projection.rs`: after the presentation owner has supplied contiguous one-grapheme visual runs, this leaf validates them once and maps physical-order glyph ranges with one monotonic run cursor. `ui/text/presentation.rs` is the matching display-range and atomic boundary lookup owner: it binary-locates the source hard line, complete mask clusters, and source/display caret boundaries, while the layout adapter only consumes that validated slice. These owners reject unordered, cross-run, incomplete, or unmapped data rather than scanning all runs per glyph or deriving any source range from the mask. The parent artifact builder retains only display text; layout, renderer, and cache owners do not receive the secure source.
- Secure glyph projection stays under `text/glyph_artifact/visual_projection.rs`: after the presentation owner has supplied contiguous one-grapheme visual runs, this leaf validates them once and maps physical-order glyph ranges with one monotonic run cursor. It rejects unordered, cross-run, incomplete, or unmapped data rather than scanning all runs per glyph or deriving any source range from the mask. The parent artifact builder retains only display text; layout, renderer, and cache owners do not receive the secure source.
- Locale-specific cosmic state is isolated under `text/shaping/cosmic/font_system_cache.rs`; `cosmic.rs` remains the backend adapter/orchestrator instead of accumulating cache policy.
- The cache is explicitly bounded to four `FontSystem` instances and reuses one seed database, closing the review concern that arbitrary application-language values could grow persistent backend state without limit.
- The boundary is documented exactly: locale configures cosmic platform fallback selection, while the canonical per-segment RustyBuzz path applies request language and `locl`; cosmic-text remains a whole-request fallback rather than a metadata source for a second projection pass.
- SH-M3 vertical policy is child-owned under `text/shaping/vertical.rs`, `vertical/orientation.rs`, and `vertical/direct.rs`; `cosmic.rs` invokes the direct owner first and only builds a cosmic buffer when that complete request cannot be shaped directly, while `ui/text/layout_engine/vertical.rs` consumes the vertical provider instead of reimplementing Unicode orientation.
- The provider hard cut preserves the existing cache authority: vertical orientation/mode are part of `ShapedRunCacheKey`, and UI wrapping/ellipsis/measurement no longer create horizontal cache entries for VerticalRl content.
- Native `vmtx` advance remains isolated under `text/font/vertical_metrics.rs`. TTB/BTT shaping is split between `shaping/vertical/backend.rs` and `vertical/direct.rs`; shared logical itemization lives in `shaping/itemize.rs`, horizontal DTO construction lives in `horizontal/direct.rs`, and `orientation.rs` owns Unicode rotation policy. The former horizontal/vertical projection owners are deleted, so backend vertical-origin/VORG-side-bearing values reach the renderer without a compatibility wrapper or a second shaping pass.
- V1 normalization policy now has a narrow `text/shaping/normalize.rs` owner. Cosmic/fallback consume its identity view and source projection instead of embedding an unreviewable offset assumption in the backend adapter.
- Text 03 vertical column capacity, right-to-left frame placement, and cross/main axis extents moved to `text/layout/vertical_layout.rs`; the UI child consumes the shared result and retains only CandidateLine/rich/ellipsis/UiResolved DTO projection.
- The production SDF VerticalRl consumer calls the same shaping owner, while `render/text_advances.rs` projects source-cluster advances, `sdf_atlas/text_keys.rs` owns shaped glyph/face key collection, and `sdf_render/vertices.rs` maps vertical origin/rotation into destination frames and UVs. `render.rs` is 712 lines, `sdf_atlas.rs` 611, and no production file crosses the 800-line review warning; no old scalar-only vertical success path or compatibility shim remains.
- Native bitmap atlas follows the same owner-module rule end to end: the
  `text/native_bitmap_atlas.rs` root is declaration/re-export wiring only; frame state,
  source-image details, per-frame budget, frame driver, and renderer-facing glyph DTO live in
  `frame.rs`, `source_image.rs`, and `glyph_run.rs`. The former `text_area.rs` input is deleted.
  `storage.rs` owns resource-format accounting, while one canonical frame submission retains
  painter order as ordered resource segments. Repeated `R8 -> RGBA -> R8` therefore switches
  resources while replaying one draw plan, rather than cloning an atlas or creating a second
  renderer route. Glyphon is not a supported native atlas success fallback.
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
- The fix adds no Editor-only font route, compatibility module, root facade, or test glyph injection. Runtime and Editor consume the same `FontDatabase` face identity; any retained-host glyphon `FontSystem` synchronization is local to that owner and must not be reintroduced into the screen-space native atlas.
- The Windows lower regression remains in the existing folder-backed `scene_renderer/ui/text/tests.rs` owner, so production `text.rs` retains orchestration rather than accumulating test fixtures or platform assertions.
- The real HUD framebuffer gate passes after the same bounded 24-frame async-text settle policy used by the Runtime product test; waiting policy stays in test/product validation rather than becoming a production rendering bypass.

## 2026-07-17 Runtime Text shared font generation stability note (2026-08-29 collection-owner corrected)

- `text/font/shared.rs` owns `FontCollectionService`; the process-wide instance is only the compatibility/default adapter. Semantic comparison remains a folder-backed child at `text/font/database/equivalence.rs`, not policy in `text.rs`, `render_state.rs`, or the scene renderer.
- The comparison covers only inputs that can change shaping/fallback/raster output: ordered face descriptors and sources, fallback families, CompositeFont, default UI family, runtime primary and last-resort inputs. Derived indexes, diagnostics, and runtime caches do not become false invalidation inputs.
- Database replacement and generation advancement share one write-lock critical section, so snapshots cannot observe a new database with an old generation. Current shaping/layout/artifact hot paths retain an immutable collection snapshot and compare `FontCollectionRevision(collection_id, generation)`; a bare atomic generation probe is not a valid cross-collection identity. Default adapters add no production test lock or compatibility algorithm path.
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

## 2026-08-26 Runtime Text virtual visual-run artifact owner note

- `text/glyph_artifact.rs` remains the lifecycle and source-congruent-fragment selection owner. The new `text/glyph_artifact/projection.rs` leaf owns canonical shape outcome handling, font-handle registration, generation checks, and the local physical visual-line shape required by zero-width virtual runs. `visual_projection.rs` owns only visual-run validation and source/anchor projection; renderer batches do not invent source ranges.
- A horizontal final-LTR ellipsis line is admitted only when its resolved visual runs are contiguous, complete, and all LTR. Zero-width source anchors become `virtual_glyph` ranges; RTL/tatweel or mixed-direction text, mixed source/virtual backend clusters, multiple virtual anchors in one cluster, incomplete runs, and vertical lines fail closed to the existing renderer fallback. First construction and generation rebuild use the same leaf owner.
- The visual path advances run-to-cluster and glyph-to-cluster cursors monotonically, preserving `O(G + R)` work without a per-glyph scan of every run. It is a distinct visual-shape counter, not an excuse to weaken source-fragment cache identity or describe unmeasured timing as a win.
- The artifact root is 799 lines, `projection.rs` 395, and `visual_projection.rs` 427; the layout root is 798 after moving virtual-fragment sequencing and final virtual-fragment metric ownership into its child. The `logical_virtual_line.rs` leaf is 572 lines and its adapter is 508 lines. Scoped formatting and diff checks are static evidence only; managed Cargo, profiling, and a fresh product WGPU framebuffer under `docs/tests/runtime/text` remain open and are not implied by this structure record.

## 2026-08-26 Runtime Text logical virtual-sequence owner follow-up

- `text/layout/logical_virtual_line.rs` is the backend-neutral private owner for horizontal plain virtual display text, local grapheme ranges, source anchors, captured UAX#9 levels, and visual indices. `ui/text/layout_engine/virtual_fragment_sequence.rs` is the only `CandidateLine` adapter. This state is request-local through artifact construction and retained only inside the runtime artifact for font-generation rebuild; it does not enter a public UI DTO or cache key.
- `CanonicalLogicalVirtualLineFragment` now sits inside that sequence rather than in the layout root. It retains one current-generation logical shaped run, exact metrics, and grapheme advances before UAX#9; the child applies its metrics to the final line and artifact construction projects the same run. A missing or generation-stale fragment alone may re-shape preserved logical input. RTL tatweel and mixed horizontal virtual content therefore never uses physical text as a fresh LTR shaping input. Projection remains `O(G + C)` by monotonic glyph/cluster cursors. Cross ordinary/virtual, distinct-anchor, direction-boundary, non-isomorphic, and non-monotonic backend clusters reject the artifact and retain the renderer fallback; rich and VerticalRl remain outside this owner.
- This is static implementation evidence only. Managed Cargo, profiler and power samples, and a genuine WGPU framebuffer saved only under `docs/tests/runtime/text` remain open.

## 2026-08-26 Runtime Text horizontal-fragment and rejected-virtual owner note

- `text/layout/horizontal_line_fragment.rs` is the shared private geometry owner for canonical physical and logical-virtual Horizontal Plain fragments. It owns one retained shaped-run `Arc`, current metrics, and grapheme advances; `physical_line_fragment.rs` and `logical_virtual_line.rs` retain only their distinct source/anchor and generation semantics. This removes duplicated leaf policy without growing the layout orchestration root or adding a public compatibility facade.
- `LogicalVirtualLineSequence` owns the explicit artifact-projection rejection state after a virtual BiDi invariant fails. `virtual_fragment_sequence.rs` clears only private fragment/advance state, while `glyph_artifact.rs` and `glyph_artifact/projection.rs` both decline every artifact route for the marked line, including rebuild and final-LTR visual projection. The resolved candidate remains under the established renderer fallback instead of turning that private route failure into a new UI DTO or renderer-local mapper.
- Current source sizes are bounded: the shared geometry leaf is 40 lines, the physical fragment leaf 121, the logical virtual owner 572, its UI adapter 508, artifact root 799, and projection leaf 395. Scoped formatter parsing and source audits are static-only evidence; managed Cargo, profiling/power samples, and a product WGPU framebuffer under `docs/tests/runtime/text` remain open.

## 2026-08-26 Runtime Text selected-face metric-span owner note

- `text/model/shaped_run.rs` owns the crate-private `HorizontalGlyphMetricSpan` sidecar and its complete-coverage admission rule. `text/shaping/horizontal/direct.rs` alone produces spans while it already owns direct itemization and selected-face metric resolution; `text/font/line_metrics.rs` returns the existing scaled face metrics from that same access. Cache capacity accounting remains in `text/cache/shaped_cache/memory.rs`.
- The model, shaper, cache, UI layout, and renderer retain separate responsibilities: model retains immutable provenance, direct shaping records it, a future named line-policy leaf aggregates it, UI publishes a resolved frame, and artifact projection consumes only a prepared optional offset sidecar. No public re-export, DTO field, renderer mapper, or compatibility owner was introduced.
- Production leaf sizes after this foundation remain below the 800-line warning: `shaped_run.rs` 658, direct horizontal shaping 261, line-metrics 439, cache memory 78, and the shared geometry leaf 40. Static parser/source checks only; managed Cargo, profiling/power evidence, and a real WGPU framebuffer remain open.

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

- `TextRenderState` 是 source/slot/page 失效协调 owner；`native_bitmap_atlas/source_cache.rs` 只拥有 CPU bytes、entry hard cap、indexed LRU 与 requested/actual `GlyphRasterKey` 反向索引，`GlyphAtlasSet` 只拥有 slot/page generation/allocator/shadow，renderer 只返回 upload 成败。三层之间不复制预算或页状态。
- source pressure 先发出已绑定 raster key，atlas owner 定位并原子失效整个 page，再定点回传该页全部 source keys；atlas eviction 通过 `GlyphAtlasBitmapRunPlan.invalidated_raster_keys` 显式交接，禁止依赖跨帧隐藏队列。failed upload 对 page keys 去重后每页只推进一次 generation，并在下一帧 report 中暴露 source invalidation。
- 正常 exact hit、LRU touch/insert/evict 与 key 反查保持 O(1)；近似 lookup 直接构造至多三个 vertical-bin key。预算压力路径只遍历被失效页的 slots，不扫描整个 source cache。
- production owner 保持 folder-backed：source cache 只持缓存、队列回流与驻留状态，frame diagnostics 与 worker-pool projection 保持在具名 child owner；后续诊断字段不得回流 renderer 或 atlas root，新的缓存或驻留职责仍须先提取具名 owner。规模/驻留测试按语义拆分，生产与测试 owner 继续受当前文件预算门禁约束。
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
- The inline cosmic adapter regressions are child-owned by `text/shaping/cosmic/tests.rs`; `cosmic.rs` is 712 lines and the child test owner is 161 lines after the 2026-08-30 owner split. The move preserves the existing ten test bodies and keeps the production router below the 800-line review threshold without changing backend behavior.
- Vertical `Tr` clusters first enter the script-aware TTB/BTT backend with `vert`/`vrt2`; an otherwise identical shape with those two features forced off distinguishes real vertical substitution from `locl`, variation selectors and user features before rotation fallback. Horizontal direct line metrics aggregate scaled ascent/descent/line-gap from actual segment faces. Direct cluster ranges and BIDI line order use single linear cursors/passes, with 0.8em reserved for absent metrics and empty lines.
- Current Text02 production owners remain below the 800-line warning. The non-validation implementation and all findings from the completed independent review passes are forward-fixed; the real Windows WGPU harness now targets `docs/tests/runtime/text/runtime_text_mvp_foundation_product_framebuffer_20260831.png`, but this structure record and path preparation do not substitute for managed validation or an actually generated and inspected product screenshot.

## 2026-08-26 Runtime Text Plain baseline-policy owner note

- `text/layout/horizontal_line_policy.rs` is the sole Plain horizontal composite-baseline policy leaf. It accepts complete model-owned selected-face spans and the raw envelope, returns only finite `TextLineMetrics`, and fails closed to the shaped-line metrics when provenance is incomplete or inconsistent. `horizontal_line_fragment.rs` remains the sole physical/logical-virtual geometry consumer.
- Renderer coordinate policy remains outside that leaf: Native and SDF artifact vertices consume `layout_baseline + TextGlyph.offset[1]`, so Plain fallback spans share the resolved alphabetic baseline and do not allocate a duplicate per-glyph y-offset vector. The pre-existing projection sidecar stays dormant for a future rich/inline block-origin adapter.
- Current source sizes are bounded: policy leaf 171 lines, shared fragment geometry 51, physical fragment 105, logical virtual owner 530, virtual adapter 477, artifact root 766, and projection leaf 375. Static-only evidence does not replace managed Cargo, profiling, or a current WGPU screenshot under `docs/tests/runtime/text`.

## 2026-08-26 Runtime Text rich baseline handoff owner note

- `text/layout/rich/materialize.rs` owns rich text/inline line extents and the resolved baseline; `UiResolvedTextLine` transports that result; `render/text_decorations.rs` maps each source-owned paint run back to the correct line; Native/SDF raster consumers share the resulting absolute baseline. No renderer-local metric aggregation or second baseline DTO exists.
- The rich planning regression is kept in the existing folder-backed `render/tests/rich_inline.rs` owner. It proves a font-size override preserves the same resolved line baseline rather than testing a source-only strategy image.

## 2026-08-29 Runtime Text physical-line geometry owner note

- `UiResolvedTextLine.frame` owns absolute natural content geometry; its main-axis extent agrees with resolved advances and alignment changes only its origin. The required `placement_frame` owns the paragraph or rich-cell slot used for line admission and nearest-line selection. Renderer, caret, selection and IME geometry consume the content frame; rich activation uses `hit_frame()`, while slot gaps may still choose the nearest caret without activating content.
- Plain/rich horizontal and `VerticalRl` layout producers are the only geometry publishers. Rich-table projection must call `UiResolvedTextLine::translate` so content and placement geometry move atomically. Renderer-local alignment policy, default production placement frames and compatibility serde fallbacks are prohibited.
- The DTO cost is one 16-byte `UiFrame` per published physical line. This correction adds no shaping, wrapping, allocation or search loop and therefore carries no timing or power claim until the managed cold/warm, scroll/edit, product WGPU and power matrices run.

## 2026-08-26 Runtime Text tab fragment ownership note

- `ui/text/layout_engine/physical_line_metrics.rs` owns the distinction between a source-congruent canonical fragment and fragment advances that are safe to reuse as final layout advances. A tab line retains the former for metric/artifact identity but the named accessor rejects the latter because tab-stop placement is pen-position-dependent.
- `layout_engine.rs` consumes that accessor before visual ordering; the existing line-width owner remains the sole tab x-placement policy. This prevents a Text03 metric fallback without adding a renderer-local tab branch, a second cache, or a public UI contract.

## 2026-08-26 Runtime Text viewport metric-certificate owner note

- `ui/text/layout_engine/measurement.rs` owns the stable-font-generation primary-face coverage certificate; `viewport.rs` owns fixed-height hard-line window selection; `physical_line_metrics.rs` remains the owner of actual selected-face per-line metrics. A fallback-chain envelope cannot cross these boundaries as a synthetic baseline.
- An uncertified Plain document uses complete physical-line layout until a session-owned prefix-metrics cache can provide cumulative heights and baselines. Any such cache must be generation-aware, bounded, profiler-justified, and remain below `ui/text` rather than moving policy into a renderer or public DTO.
- The current production owners remain within the review budget: `layout_engine.rs` 779 lines, `measurement.rs` 187, and `viewport.rs` 304. The new coverage scan has its own profiling span so later optimization can be evidence-led without obscuring visible-window selection cost.

## 2026-08-26 Runtime Text font-generation fence owner note

- The canonical shaping service produces the actual stable font generation; `layout_session.rs` retains it only in a crate-private tagged result until synchronous cache admission, and `parallel/shape_pool.rs` applies the same three-way generation check at worker completion. The untagged helper is test-only; public shaped DTOs and cache keys are not widened.
- `ui/text/layout_engine/artifact.rs` owns the request-scoped publication fence across metrics, physical-line resolution, and artifact attachment. `layout_result.rs` owns only the private retained fragment sidecars; it cannot escape into the serializable UI layout or layout cache.
- Current production sizes are `layout_session.rs` 653, parallel shape pool 396, `layout_engine.rs` 779, artifact owner 101, and layout-result owner 22. Generation mismatch remains a typed Deferred outcome; no compatibility facade, renderer-local font epoch, production panic, or second cache owner was added.

## 2026-08-26 Runtime Text artifact-identity owner note

- `zircon_runtime_interface/ui/surface/render/text_layout.rs` owns only opaque payload storage, payload-kind separation, and identity type erasure. It does not know rich parser fields, glyph projection fields, font generations, or cache internals.
- `text/rich/artifact_handle.rs` defines compiled-rich identity over the complete immutable compiled artifact, including source, parser generations, parsed runs, and projection indexes while excluding residency-only byte estimates. `text/glyph_artifact/identity.rs` defines resolved-glyph render/rebuild identity and deliberately excludes the regenerable logical shaped-fragment cache.
- Rich table/cell projections remain `UiParsedText` range/index views that own an `Arc` to the one parent `CompiledRichText`; no projected compiled-artifact subtype or default-generation compatibility constructor survives.
- `text/glyph_artifact/snapshot.rs` owns source/effective-style/generation/writing-mode/layout-line compatibility. UI preparation may rebuild a mismatch; graphics may only reject it and must not reconstruct identity or reshape locally. The check is fused into the existing line traversal and adds no digest/cache facade.
- The glyph identity and snapshot leaves are 53/66 lines; `glyph_artifact.rs` remains a 797-line orchestration root, below the 800-line warning. Interface layout, rich handle, compiled artifact, and logical virtual owners remain folder-backed. No compatibility constructor, public identity DTO, second registry, or renderer-owned comparison policy was added.
- Artifact absence is qualified by the same owner boundary: `glyph_artifact.rs` rejects an incompatible source/layout owner or out-of-owner line/run range as typed `LayoutFailed`, and `glyph_artifact/projection.rs` rejects a non-sliceable UTF-8 range. `Ready(None)` remains reserved for valid visual-only DTO routes. Invariant failures live in the 168-line `glyph_artifact/tests/invariant_failures.rs` child, while ligature/RTL/multiglyph/stale-generation geometry lives in the 318-line `glyph_artifact/tests/cluster_geometry.rs` child. The test root is 673 lines, production roots are 601/431 lines, and no renderer-local repair policy was introduced.
- Identity is mandatory for marker payloads as well as data-bearing payloads. The secure-text no-source marker owns a versioned stable identity in `ui/text/presentation.rs`; it must not restore the deleted identity-free constructor merely because its payload is zero-sized. Its layout child imports the registration function directly from that sibling owner rather than adding a facade re-export. The owner remains below budget at 793 lines after its equality/type-recognition regression.

## 2026-08-26 Runtime Text retained-document source owner note

- `ui/text/measure_cache.rs` remains the layout-cache orchestrator; parsed Plain document capacity, byte accounting, LRU, exact source qualification, and alias diagnostics live in the 141-line folder-backed `measure_cache/retained_document.rs` owner. The orchestration root is 755 lines and does not retain the extracted eviction implementation as a facade.
- `text/cache/hard_line_index.rs` owns canonical hard-line offsets plus their exact immutable source receipt. It shares the parsed document's `Arc<str>`, uses pointer identity before exact comparison, accounts the source lifetime in its bounded byte estimate, and exposes low-cardinality pointer/exact/alias counters. UI viewport selection passes the compiled source snapshot; renderer and public DTO layers do not acquire cache policy.
- Surface extraction still owns fresh `String` materialization, so a pointer-stable source snapshot and `text_layout_revision` are not yet one atomic owner object. This keeps RTS-P1-044 open and forbids calling the retained-document path `O(visible)` until managed 10k profiling proves the source scan is gone. Static checks do not replace managed Cargo, power, WGPU, or a real framebuffer PNG under `docs/tests/runtime/text`.

## 2026-08-28 Runtime Text retained-document edit owner note

- `text/document/storage.rs` owns exactly one immutable original source, one append-only addition source, and logical piece ranges. Prepared replacements may own only their not-yet-committed replacement; commit appends after expected-revision validation. An immutable addition allocation per edit, a compatibility chunk facade, or a second storage owner must not be reintroduced.
- `text/document/hard_line_model.rs` distinguishes separator-neutral local content edits from separator-changing reparse. The local path may update only one retained line's content length and stable ID; CR/LF insertion/removal/split/join, including CRLF-interior edits, must use the structural path. Layout/renderer owners must consume dirty receipts rather than infer source changes or create another line model.
- The architecture was selected only after a 17-scenario, 31-sample baseline and was rerun with the same matrix. The 10k tail lane improved from 1,710.706 to 4.508 milliseconds p50 and from 8.127 GB to 3.643 MB counted allocation; million-character local edits no longer copy the whole line. This evidence closes those two document-edit hotspots only. Surface snapshot/revision ownership (RTS-P1-044), product limits, sampled CPU stacks, power, matched Unreal runtime, managed module-graph validation, WGPU and a real PNG under `docs/tests/runtime/text` remain open.

## 2026-08-26 Runtime Text resolved-glyph route-report owner note

- `graphics/scene/scene_renderer/ui/render/resolved_layout.rs` owns the Plain resolved-glyph consumption decision and its command-level receipt. Missing, stale, and incomplete are distinct rejection reasons; artifact, valid visual-only, source-isomorphic fallback, and rejected are distinct route dispositions. The renderer root only records the receipt and does not own glyph reconstruction or fallback policy.
- `PreparedScreenSpaceUi` transports the aggregate through `ScreenSpaceUiTextSystem`; `text/prepare_report.rs` owns frame reporting, while `text/prepare_report/profile/artifact_routes.rs` owns the seven low-cardinality profiler projections. The report type and metric prefix explicitly say `resolved_glyph_artifact`, reserving a separate owner for future compiled-rich route diagnostics.
- Text-decoration vertex emission moved from `render.rs` to the existing `render/text_decorations.rs` leaf. Current owners are `render.rs` 782 lines, `resolved_layout.rs` 579, decoration leaf 188, prepare-report profile root 797, and profile leaf 47; all remain below the 800-line warning without a compatibility facade or second shape truth. The post-layout stale-artifact rejection projection lives with the resolved-artifact route counters in that child rather than pushing the profile root over budget.
- Static formatting, whitespace, call-site, file-budget, and production exception scans pass. This owner note is not acceptance: compiled-rich parity, managed Cargo, performance/power data, current WGPU rendering, and a validated real-rendered PNG under `docs/tests/runtime/text` remain open.

## 2026-08-26 Runtime Text physical-line BiDi receipt owner note

- `text/layout/physical_line_fragment.rs` owns the Plain/Horizontal post-wrap `BidiLineOrder` because the final line boundary, retained shape, metrics, advances, and font generation meet there. Shaping glyph storage remains logical/source ordered; the renderer and Runtime Interface do not acquire a second visual-order contract.
- `ui/text/layout_engine/virtual_fragment_sequence.rs` only selects between a borrowed physical-line receipt, the explicit logical-virtual display artifact, and the existing typed fallback analysis. It does not own UAX#9 policy. `visual_order.rs` remains the projection leaf until rich/vertical consumers can receive the same canonical cluster/source receipt.
- Current owners are physical fragment 186 lines, physical-line metrics 355, virtual adapter 567, and layout root 780, all below the 800-line warning. The two profiling scopes distinguish receipt generation from fallback analysis without adding a cache or claiming a speedup.
- This is a current-state owner note, not an accepted output record. Managed tests/corpus, profiling/power, real WGPU rendering, and a validated PNG under `docs/tests/runtime/text` remain open.

## 2026-08-26 Runtime Text locale identity owner note

- Runtime Interface may serialize an optional language string, but it must not own locale parsing, canonicalization, likely-subtag or fallback policy. Those policies belong to `zircon_runtime/src/text/language.rs`; UI and graphics consumers only propagate or call that owner.
- A backend request must be canonicalized before cache/backend use. Invalid non-empty input is a typed `InvalidLanguage` failure; a canonical reborrow carries a private invariant so nested service boundaries do not parse again.
- Canonical tag, explicit script, and the font-fallback `language/script/region` key are one parse result. `text/language.rs` reads ICU4X's structured locale ID into packed typed values; shaping/itemization/font fallback must consume the request-owned projections and must not split BCP-47 strings again. This rule does not authorize hardcoded language-to-script inference.
- `FontCultureTag` is an opaque authored asset value. `CompositeFontIndex` compiles its selectors at cache-miss/font-generation publication, while candidate queries only compare typed components. Invalid restricted selectors remain restricted and never become an unrestricted sub-font. Culture-eligible families precede generic families, preserving declaration order inside each bucket, matching Unreal's `CachedPriorityFontRanges` ownership.
- Cache-hit preconditions must not parse locale strings. Explicit CompositeFont descriptor identity may scan/hash authored bytes before an index lookup; project CompositeFont queries reuse the generation-owned identity/index. Full canonical tags remain in shaping identity, while font-candidate identity contains only the typed components that affect candidate selection.
- Shaped/SDF/FontSystem caches compare the canonical tag exactly. Cache leaves must not reintroduce lowercase/separator loops, permissive invalid-tag aliases or a second normalizer. Locale string allocation is allowed when canonical output changes or when a new bounded FontSystem entry is inserted, not for each glyph or canonical cache hit.
- Canonical tag plus explicit fallback components are not a complete fallback receipt. Explicit parent-combination matching is implemented, but likely script/region, locale-data generation and an observable prioritized fallback decision receipt must be added by the same owner before `RTS-P1-002` can close.

## 2026-08-26 Runtime Text Unicode-data identity owner note

- `zircon_runtime/src/text/unicode_data.rs` is the sole owner of Unicode/locale provider implementation versions, data versions, snapshot schema and stable fingerprint. Provider dependencies are exact-version pinned; dependency upgrades and snapshot generation changes are one atomic change.
- A request captures one `UnicodeDataSnapshotId` before analysis. Script/emoji analysis, Bidi paragraph/line artifacts, line-break opportunity maps, shaping artifacts, cache keys and diagnostics propagate that exact identity; a nested owner must not query a newer global identity or infer one from text.
- Cache and artifact identity is `generation + fingerprint`, not a generic `unicode_version` string. Mixed provider data versions remain explicit because Bidi, line breaking and emoji/script data do not currently share one Unicode release.
- Missing identity is an incompatible serialized shaped artifact, not permission to substitute the current snapshot. Future hot update must retain old descriptors/artifact leases until their consumers retire; it must not reinterpret old ranges or clusters with new data.

## 2026-08-26 Runtime Text failure-policy owner note

- `zircon_runtime/src/text/shaping/failure_receipt.rs` is the sole direct-shaping failure classifier. Stable code, phase, source range, face, dependency, disposition and budget fields are generated together; backend adapters and Cosmic must not maintain parallel boolean classifiers.
- Only a receipt explicitly marked `AlternateBackend` may enter Cosmic fallback. Source/Bidi/itemization invariants and admission-budget failures are terminal. A fallback backend must not be used to conceal malformed ranges or evade resource limits.
- Low-cardinality counters use the stable code enum and fixed storage. Exact paths, strings and cause chains must not become metric labels. Text layout/shaping diagnostics belong to the retained or operation-local session; a process-global failure-report mutex is prohibited even on the error path.
- The session report is diagnostic only. Internal non-Ready `TextShapingOutcome` values carry `TextShapingFailure { error, receipt }`, and transforms/session owners must forward that envelope intact; consumers must not read a shared “last error” to decide layout or retry policy.
- The public `core/framework/text::TextLayoutError` remains backend-neutral. Implementation-owned face/range receipts are projected away only when crossing `TextLayoutService`; they must not create a `core/framework/text -> text` reverse dependency or a renderer-owned failure classifier.
- `TextLayoutError::diagnostic_code()` and `TextLayoutError::message_key()` are the stable machine
  boundary for text failures. Callers must not parse `Display` strings; localized messages belong to
  the editor/host catalog, while the core enum remains free of Runtime Text implementation types.
- A UI text “stack” must own a real capability-ordered backend set and typed selection receipt. A
  one-member forwarding wrapper is prohibited; the sole adapter implements `UiTextShaper` directly,
  while direct/Cosmic composition remains in the Runtime Text shaping owner.
- Serializable text DTOs and process-local renderer artifacts are different ownership domains.
  Internal `Arc`/range/lease types must not leak into serde contracts, and owned `String` fields must
  not be replaced before layout-cache plus final-batch residency receipts prove duplicate storage is
  dominant. Materialize owned strings only at an explicit versioned boundary.

## Runtime Text Operation-Owner Convention (2026-08-26)

- A Runtime UI text operation resolves its owner once at the entry boundary. `UiSurface` uses its
  retained `UiTextMeasureCache`; a standalone layout/extract operation owns one bounded local
  cache for the complete operation.
- Nested layout, extraction, component measurement, and artifact-preparation phases receive the
  required owner/provider. `Option<&mut UiTextMeasureCache>` is permitted only for an explicit
  deferred phase whose admission contract proves that the product path cannot use it; it is never
  permission to construct a second production session.
- Surface projections and Editor materialization consume the published `UiSurface::render_extract`
  when available. They must not re-extract the same tree merely to obtain text commands.
- One-shot helpers remain explicit tool/test boundaries and must retain a bounded ownership and
  allocation contract. Do not replace missing ownership with a process-global mutex, registry, or
  renderer-local shaper.
- Font/resource generation recovery belongs to the retained layout/document owner. A renderer may
  reject a generation-qualified artifact after a late resource publication, but it must not
  reshape that artifact, attach a mutable replacement line, or create a layout session. The owner
  republishes one immutable artifact snapshot on its next invalidation/rebuild boundary.
- Session diagnostics reset at the frame/operation boundary and may expose only fixed code/route
  dimensions. Parallel workers aggregate fixed value reports and merge them after join; they do not
  lock or retain the session. Cache hits are not backend-work receipts. A document identifier is not
  a metric label; exact document drill-down requires a separately bounded document owner.

## 2026-08-26 Runtime Text script-analysis owner note

- One paragraph/request owns one immutable script analysis. Font fallback, horizontal/vertical
  shaping, and backend projection consume that same segment set; a cluster consumer must not derive
  a second script identity from its first codepoint.
- Unicode script identity is data driven. `Script_Extensions` uses the selected Unicode provider's
  fixed-size set operations; paired brackets use the selected BidiBrackets provider. Production
  code must not add private script ranges or a partial bracket table beside those providers.
- Common/Inherited context and bracket nesting remain analysis policy below `text/shaping`.
  Renderers, Runtime Interface DTOs, and font cache keys receive resolved identities only. Unknown
  is a typed state; other known scripts use a stable ISO15924 identity, never a source codepoint.
- A serialized script identity must preserve that invariant at the schema boundary. Opaque packed
  tags use a private, validated value type with checked deserialization; a public enum variant must
  not expose a raw integer that lets authoring code or persisted data manufacture codepoint identities.
- Emoji presentation is a paragraph-analysis concern separate from Unicode script identity. The
  analysis layer may read the selected Unicode emoji property/sequence provider once and publish
  immutable source ranges; fallback, shapers, renderers, and cache keys consume resolved ranges and
  must not add broad codepoint ranges or repeat property lookup in a glyph loop.

## 2026-08-26 Runtime Text fallback-receipt owner note

- The font fallback owner publishes one immutable resolution receipt containing the selected face,
  completeness, and resolution source. Shaping spans retain that receipt as their only selected-face
  truth; they must not collapse it to a face ID or merge complete and missing decisions on the same face.
- Successful itemization has a required primary face. Missing primary ownership is a typed error and
  must not be encoded as empty spans or forwarded to an implicit backend fallback policy. Partial
  coverage may retain a real face and `.notdef`, but its missing receipt and bounded diagnostics remain.
- This owner note is a non-acceptance contract. Candidate/capability traces, managed Cargo, real tofu
  raster, WGPU/PNG, and performance/power evidence remain required before closing Runtime Text gates.

## 2026-08-26 Runtime Text backend-error owner note

- `text/shaping/backend_error.rs` owns font-access, face-parse, and empty-output failures shared by the
  horizontal and vertical RustyBuzz adapters. A backend must retain the original font database source;
  it must not collapse these causes to `Option` or a synthetic empty run.
- `text/shaping/direct_error.rs` owns the direct receipt across itemization, Bidi, backend, source-range,
  cluster-boundary, finite-metric, and cluster-order failures. Horizontal and vertical adapters return a
  required shaped run or that receipt; they must not encode failure as `Option`, `Ok(None)`, or `.ok()?`.
- Cosmic is the sole downgrade owner and has one explicit policy boundary. Bidi invariant failures fail
  closed; other direct failures may follow the existing orientation-specific whole-request fallback
  policy. Consumers below or beside Cosmic must not independently reinterpret the receipt or construct a
  synthetic success run.
- Script identity remains typed through the backend boundary. Horizontal and vertical RustyBuzz adapters
  consume `Iso15924Tag`, not a reparsed string. Common/Inherited/Unknown may deliberately defer to backend
  inference, but invalid tag parsing must not be encoded as an absent script after the analysis owner has
  already validated the tag. A non-empty hard-line separator likewise either produces its virtual glyph or
  a typed source-range failure; only an actually empty separator may return `Ok(None)`.
- A backend failure crossing into direct shaping must be qualified by the itemized source range at that
  boundary. Face/cause without range is insufficient for diagnostics or future run-local recovery; range
  without the original backend source is insufficient for policy. This receipt does not itself authorize
  partial recovery, which remains owned by the shaping router.
- Typed errors add no claim of speed or acceptance. Managed fault tests, profiling/power, current WGPU
  output, and a validated PNG under `docs/tests/runtime/text` remain open.
- Scratch work is request-local and bounded by source-derived output: extension intersection is
  allocation-free, bracket entries are pushed/popped once, and no global registry or per-character
  heap set is permitted. Static complexity evidence does not replace corpus, profiling, power, or
  WGPU acceptance.

## 2026-08-26 Runtime Text cluster break-safety owner note

- RustyBuzz backend adapters own raw `GlyphInfo::unsafe_to_break()` capture. `ShapedGlyphClusterFlags`
  owns the typed `Unknown/Safe/RequiresReshape` projection, and only a cluster head may carry a known
  value. Cosmic, synthetic/virtual glyphs, and legacy artifacts must remain explicitly `Unknown`.
- Break safety is provenance, not UAX#14 policy. Backend/direct code must not clear `soft_break` or
  reshape. The post-wrap final-line owner must join the selected boundary to the following cluster's
  receipt and then reuse, reshape both sides, or take a conservative unknown path.
- The receipt stays in Runtime Text shaped artifacts. Runtime Interface and renderers do not receive
  a parallel break classifier. Horizontal and vertical buffer construction consume validated
  `Iso15924Tag`; no backend leaf may degrade it to a reparsed string.
- Current owners are shaped model 647 lines, Cosmic 784, horizontal backend/direct 130/323, and
  vertical backend/direct 307/371, all below the 800-line warning. This is static infrastructure;
  exact final-line reshape, corpus, managed Cargo, performance/power, and real WGPU/PNG remain open.

## 2026-08-26 Runtime Text line-break analysis owner note

- `text/shaping/line_break.rs` is the only provider-opportunity owner. It builds one ordered map for a
  request-bound Unicode snapshot and projects cluster-head receipts; horizontal, vertical, Cosmic,
  layout, UI, and renderer code must not call `unicode_linebreak::linebreaks` independently.
- `ShapedGlyphRun::unicode_data_snapshot` versions the provider. `ShapedGlyphClusterFlags::line_break`
  stores only the compact tailoring profile and selected opportunity. Legacy/invalid/non-head input is
  typed unknown, never silently upgraded to the current profile.
- Break provenance is not layout policy. `soft_break/mandatory_break` remain the current compatibility
  decisions; Runtime Interface and renderers do not receive a parallel classifier. Locale tailoring and
  UAX rule traces belong in this analysis owner before layout/Editor consumers may use them.
- `shaping/tests.rs` delegates line-break and script receipt contracts to folder-backed child modules
  and remains below the 800-line warning. Production owners are shaped model 691, line-break 248, and
  Cosmic 797 lines. Static size/format/source scans do not replace managed tests or profiling.

## 2026-08-26 Runtime Text soft-hyphen virtual-fragment owner note

- A discretionary hyphen is generated display content. Its owner must publish a typed decision with
  consumed source range, marker mode, and zero-width visual anchor. Plain canonical artifact paths must
  attach marker bytes to that anchor rather than infer a source range from their UTF-8 length.
- Marker glyph selection and shaping use the current resolved run's font, language, script, features,
  writing mode, and generation. Measurement, final-line metrics, glyph artifact, hit testing, and
  accessibility consume the same canonical virtual fragment and must not rediscover style in UI or the
  renderer.
- `line_break/soft_hyphen.rs` may detect U+00AD and emit the decision, but it must not own font choice or
  final glyphs. `candidate_line.rs` materializes one display-owned zero-anchor for Plain, horizontal
  rich, and VerticalRl rich; the typed `DiscretionaryHyphen` role and replaced-source receipt survive
  logical UAX#9, style shaping, glyph publication, hit testing, and renderer routing.

## 2026-08-26 Runtime Text word-boundary owner note

- UAX #29 word segmentation belongs to `text/word_boundary.rs`. Layout, UI navigation, rich text,
  accessibility, and future dictionary-tailoring consumers must not maintain local whitespace,
  punctuation, or `is_alphanumeric` boundary policies.
- A boundary view carries the exact `UnicodeDataSnapshotId`. Word and Grapheme are distinct snapshot
  roles even when one compiled library implements both; cache/artifact identity follows semantic
  capability, not dependency package count.
- One-shot queries remain zero-copy and stop at the requested prefix. A retained document analysis may
  materialize ranges once, but leaf callers must not allocate competing paragraph-sized maps.
- EndWord selection consumes completed word ranges only. Ellipsis marker shaping/source/accessibility
  remains owned by the final-line virtual artifact and must not be inferred by this boundary owner.

## 2026-08-26 Runtime Text ligature-caret owner note

- Backend cluster/source coverage is the common contract for measurement, wrap, caret, hit testing,
  selection, accessibility, and paint. A grapheme-width compatibility index must not claim precise
  interior carets for a multi-grapheme cluster.
- Font-derived interior carets require typed provider provenance such as OpenType GDEF LigCaretList or
  an equivalent backend result. When unavailable, the artifact publishes an explicit atomic-cluster
  fallback; it must not divide advance evenly and call the result font-derived.
- The current Rust stack lacks that provider, so this note freezes the intended hard cut rather than
  authorizing a partial algorithm change. Managed corpus and product validation remain required.
- `text/cluster_geometry.rs` is the sole internal grouping owner for `ShapedGlyph` and renderer
  `TextGlyph`. Measurement, retained artifacts, wrap correction, caret, hit, and selection consumers
  must consume its source range and aggregate advance rather than reconstructing clusters locally.
- A grapheme advance array may remain only as an explicitly named compatibility projection. Before a
  glyph-wrap range is committed, its endpoints must be coalesced against typed atomic clusters with a
  monotonic cursor; downstream loops must not reopen the range and make per-grapheme break decisions.
- A resolved-line advance projection is exact only when it has one finite non-negative value per
  visual grapheme. An interface/DTO owner without a font database or shaped artifact must expose only
  known aggregate endpoints when that invariant fails; it must not divide total line width to invent
  interior caret, selection, decoration, or IME geometry.
- Runtime recovery may shape only with an exact command/source owner, its complete resolved style,
  and an explicit source-congruence qualification. Layout-level default style is not a substitute for
  missing run/font/tab/justification/BiDi context; without those inputs hit testing stays endpoint-only.
- Cross-run grapheme continuation is not permission to infer a backend cluster from independently
  shaped style fragments. That case remains conservative until paragraph-owned shaping publishes one
  source/style-aware cluster artifact.

## 2026-08-26 Runtime Text WordSmart policy owner note

- UAX #29 word ranges remain owned by `text/word_boundary.rs`; `layout/line_break/smart.rs` may apply
  wrap style to those ranges but must not own another word or punctuation segmentation table.
- Unicode GeneralCategory is a versioned capability in `UnicodeDataSnapshot`, distinct from Emoji even
  when one crate supplies both. WordSmart may select category classes, but raw dependency calls must not
  spread into UI, rich text, renderer, or Editor consumers.
- WordSmart scans ordered word ranges and chunks monotonically. It must not allocate another full
  boundary vector or repeatedly scan text prefixes. Split/merge requires source-isomorphic chunks;
  discontinuous hidden-source or virtual mappings fail closed and stay with their canonical owner.
- Locale dictionary segmentation, tailoring, and hyphenation remain separate versioned providers. The
  current GeneralCategory policy is not authorization to encode language-specific exceptions in code.

## 2026-08-26 Runtime Text rich profiling owner note

- Rich shaping attribution belongs in `ui/text/layout_engine/rich_layout/profile.rs`; layout,
  advance-index, provider/session, and renderer owners must not each publish competing phase names.
- Profile capture is aggregate and bounded: one scope plus request/input-byte counters per logical
  phase. Per-run, per-line, per-grapheme, and per-glyph spans/counters are prohibited because profiler
  cardinality would become part of the measured algorithm.
- Non-profiling builds retain direct provider forwarding and no mutable diagnostic registry. Profiling
  must not clone shape results, change cache admission, or select a different layout route.
- Attribution is evidence collection, not optimization. Artifact/cache/document lifetime changes begin
  only after matched cold/warm scale data identifies a dominant owner and preserves source/cluster
  correctness.

## 2026-08-26 Runtime Text joining-property owner note

- Unicode Joining_Type belongs to `text/joining_type.rs`. It owns the compiled provider trie and
  projects a backend-neutral `TextJoiningTypeMap`; layout, UI, renderer, and Editor code must not add
  Arabic letter/non-left ranges or import raw ICU joining types.
- Script membership and joining direction are separate checks. Arabic Kashida candidates require an
  Arabic-script grapheme base plus compatible preceding/following Joining_Type; transparent marks and
  explicit join controls remain visible policy, not hidden table exceptions.
- A joining candidate is not a safe insertion receipt. The shaping owner must validate the selected
  face, language, features, and resulting clusters before a virtual Tatweel becomes canonical layout
  content. Renderer code must not probe or materialize Tatweel independently.
- `text/layout/arabic_justification.rs` is the current backend-safety owner. It consumes the retained
  candidate `MeasuredTextLine` and the shared cluster iterator; only an independent nonzero Tatweel
  cluster with positive advance, RTL neighbors, and one face/instance may cross into UI virtual-source
  materialization. Width-only checks and renderer-local fallback-face inference are prohibited.
- Candidate scans are monotonic `O(graphemes)` over one immutable process-lifetime map. Changes to the
  32-candidate/5-probe budget require measured Arabic scale data and a written performance decision;
  this static owner note is not an acceptance or performance claim.
- Arabic fit attribution belongs in the 108-line
  `ui/text/layout_engine/line_box/profile.rs` child. It retains six profiling-only saturating values,
  while `line_box.rs` owns candidate construction and typed outcome routing. A probe iteration may
  update those local values but must not emit a profiler event or acquire global/TLS diagnostic state.
- Each physical line that reaches candidate fitting may publish one scope and one sample for each of
  requested/probe/candidate-byte/safe/accepted/last-rejection. Stable rejection codes are explicit
  mappings rather than enum discriminants. Ordinary builds retain a zero-field profile owner and the
  same shaping/cache/fallback behavior.

## 2026-08-26 Runtime Text rich runtime-artifact owner note

- A rich resolved layout has one private runtime-artifact owner. That owner may compose compiled rich
  metadata, immutable glyph data, exact layout snapshot, and run-to-glyph projection while preserving
  the existing neutral public handle. Consumers must not repurpose the handle as mutually exclusive
  payload storage.
- Glyph storage is physical-line owned. Paint runs carry stable ranges into the immutable line rather
  than cloned glyph vectors or independently shaped fragments. Cross-run ligatures have one glyph
  owner; continuation runs publish an empty range, not a fallback-shaping request.
- Renderer extraction validates source, origin, font generation, full style, writing mode and exact
  layout-line snapshot before accepting a slice. It may consume a typed negative receipt but must not
  reconstruct rich layout, compiled markup or paragraph shaping policy.
- Rich artifact construction belongs under `text/glyph_artifact/`; composite registration and payload
  multiplexing belong under `text/runtime_artifact.rs`; UI layout/prepare only coordinate publication;
  renderer modules only project screen-space slices. New public DTO fields require a separate runtime
  interface review.
- Performance instrumentation is owner-aggregate and profiling-only. Per-glyph/per-grapheme events,
  renderer-local shaping caches, and changes to cache budgets without matched scale data are prohibited.
- A horizontal rich line with generated display content reuses `LogicalVirtualLineSequence` as its
  display-BiDi owner. Style-aware shaping derives coalesced style spans by monotonic source-cluster
  mapping; the renderer must not infer style or logical order from the already-visual line. Vertical
  writing uses the canonical vertical provider for ordinary styled spans, external blocks, and the
  explicitly admitted U+2026 ellipsis marker; unsupported generated markers remain negative artifacts.
- A zero-width generated source anchor is selection provenance, not a style identity. Each generated
  rich cluster must retain a non-empty source range owned by exactly one compiled style run; marker
  measurement, artifact shaping, glyph-slice publication, and renderer presentation must consume that
  same receipt. Boundary heuristics in the renderer are prohibited.
- Generated-cluster provenance contains the non-empty style-source owner and may contain one
  `replaced_source_range`. Only the final-line owner may derive an ellipsis replacement, and only as
  the single non-empty complement of retained source ranges. Multiple gaps, overlap, or ranges outside
  the exact layout line fail closed; consumers must not invent source coverage from a zero-width anchor.
- The immutable logical/glyph artifact is the owner of replaced-source identity. Caret affinity,
  marker hit-test, and selection geometry consume it in the existing monotonic cluster/glyph walk;
  public source maps, renderer presentation, and leaf widgets must not reconstruct a second mapping.
- Visual overflow text is not accessibility source. Accessibility name/value extraction continues to
  use original template metadata, component state, or widget value; visual-line text and ellipsis
  markers must never overwrite that semantic source.
- Virtual-source geometry must remain `O(clusters + glyphs)` per query with no persistent
  paragraph-sized lookup table. Tests for replacement geometry belong in a focused child module once
  the parent test file approaches the repository size threshold.
- Rich ellipsis is generated before display UAX#9. End/EndWord/Middle inherit the preceding current
  run and Start inherits the following run. Horizontal and VerticalRl U+2026 consume that same owner;
  unsupported vertical markers must fail closed to their explicit compatibility receipt and must not
  manufacture source coverage or reuse base style as if it were canonical.
- A compiled rich inline image/widget is an external layout block, not a font glyph. Its exact compiled
  source range is the only admission key; a literal U+FFFC remains ordinary text. The logical display
  sidecar keeps the external cluster for UAX#9 and final advances, while style-span shaping and glyph
  projection skip it. Text runs on either side are shaped independently, and the inline paint run owns
  an explicit empty glyph slice. Inline-only horizontal lines may publish an accepted zero-glyph text
  artifact only when every logical cluster is proven external; other empty shaping results fail closed.
- Caret, hit-test, and selection geometry consume external-block and replaced-source receipts through
  the same visual-cluster directory and final advance array. Rejected sidecars are never eligible for
  geometry. The implementation remains split between `glyph_artifact.rs` (538 lines),
  `glyph_artifact/geometry.rs` (361), `glyph_artifact/rich.rs` (523),
  `layout/logical_virtual_line.rs` (762), its 50-line `fragment.rs`, 152-line `glyph_projection.rs`,
  and 59-line `input_validation.rs` leaves, all below the 800-line review warning. Rich builder tests live in
  `glyph_artifact/tests/rich_builder.rs` (327 lines), rather than consuming the production leaf's
  remaining budget.
- Rich artifact shaping dispatches from the layout writing mode: ordinary styled VerticalRl spans and
  external-only sidecars use the canonical vertical provider with `VerticalMode::Mixed`; horizontal
  spans keep horizontal shaping. A VerticalRl sidecar admits only typed `Ellipsis` and
  `DiscretionaryHyphen` roles and validates their exact U+2026/ASCII-hyphen display grapheme;
  discretionary hyphen additionally requires a non-empty replaced U+00AD range. Untyped markers and
  `Justification` remain negative artifacts and cannot borrow another role's rotation or source
  semantics.
- Rich renderer publication resolves the flattened paint-run sequence in one monotonic
  `layout.lines -> line.runs` pass. `runtime_artifact.rs` exposes an exact indexed directory resolver;
  the renderer must not scan all layout lines and all artifact runs for each paint run. Each run
  receives one typed `Artifact`, `VisualOnly`, or `Rejected(Missing|Stale|Incomplete)` route before
  planning. Intentional marker compatibility is `VisualOnly`, not an undifferentiated
  missing artifact. A rejected text run may invoke renderer shaping only when the exact resolved line
  proves it source-isomorphic; non-isomorphic rejected runs produce no guessed text batch.
- The renderer root delegates rich planning to `render/rich_text.rs` and is 754 lines; the rich leaf
is 485 lines, `resolved_layout.rs` is 727, and `runtime_artifact.rs` is 339. Route tests live in the
384-line `render/tests/rich_artifact_routes.rs`, leaving the test root at 651 lines. No compatibility
  facade or duplicate run lookup survives.
- A rich inline widget is a real direct UI child, never a renderer primitive. The UI tree owns its
  lifetime, input, focus, accessibility, and ordinary render extraction; the text owner may publish
  only its canonical absolute placement frame. A graphics renderer must not resolve a bare widget ID,
  reparse markup, clone another node's draw list, or draw a stable placeholder as product output.
- The MVP `[widget=id|widthxheight]` contract uses the authored finite size as an explicit layout size.
  The ID is valid only for one direct child of the rich-text owner. Duplicate IDs, missing/cross-parent
  nodes, and overflow-omitted runs fail closed and cannot leave a hit-testable child at stale geometry.
- Inline-widget binding and arrangement must be a monotonic `O(rich runs + direct children)` pass per
  owner, with whole-tree candidate discovery bounded by the existing layout traversal. Do not add a
  process-global widget registry or per-child full-run scan without measured evidence and a separate
  surface-generation/lifetime design.
- The fixed-size MVP publishes widget frames from compiled source ranges and canonical resolved
  lines/runs. Full layout scans all layout roots; incremental layout scans only its arrangement-root
  subtrees. Duplicate, missing, cross-parent, and omitted bindings must clear subtree geometry, and
  the graphics rich renderer must emit no widget placeholder. Desired-size invalidation and
  surface/owner/generation identity remain a later qualification boundary, not an implicit renderer
  responsibility.
- A count-bounded cache that retains variable-size text source or layout DTOs must publish current and
  peak byte-residency evidence before its byte budget, admission, or eviction policy changes. The
  receipt name must state whether it is a lower bound; source/DTO heap, shared artifacts, hash-table
  capacity, and allocator/RSS evidence must not be presented as interchangeable quantities.
- Shared text artifacts are attributed only by their unique residency owner. Per-entry accounting must
  not duplicate the full bytes of an `Arc` that may survive entry eviction, and `Arc::strong_count`
  must not become an admission policy. Cache profiling stays aggregate and low-cardinality; it cannot
  expose source text or create per-entry telemetry.
- A text work-budget threshold classifies complete backend work; it is never a source-line, script-run,
  grapheme, or backend-cluster boundary. Until a typed deferred/cancelled work-unit contract exists,
  oversized requests preserve their full source/context and synchronous result semantics.
- Work-size profiling is charged only where backend work is committed: a canonical session cache miss
  or one unique parallel pending job. Cache hits, batch duplicates, and invalid requests cannot inflate
  backend-work counters. Reports remain per-frame aggregates of counts/bytes/maxima and never include
  source text.
- Direct shaping/backend adapters return typed failures. Alternate-backend selection consumes the
  failure receipt and orientation policy; `Option` cannot select a backend. A proven infallible
  optional dependency projection may remain `Option` only when its owner has already validated the
  input and the dependency's accepted domain is documented.
- Cluster break-safety provenance must survive shaped artifact -> measured cluster -> final-line
  advance index. Missing or compatibility provenance is `Unknown`; a consumer cannot reinterpret it
  as `Safe`, delete a legal UAX#14 opportunity, or infer safety from grapheme boundaries alone.
- Final-line candidate profiling uses one monotonic aggregate pass over sorted measured clusters.
  Do not add per-boundary events, source labels, a boundary vector, repeated binary searches, or
  profiling-only shape calls when an `O(boundaries + clusters)` receipt is sufficient.
- A correctness comparison shape is charged separately from the primary backend call at the lowest
  backend owner, then aggregated once at the completed request boundary. Segment loops update only
  request-local integers; they cannot emit profiler events, acquire profiler locks, or attach source.
- A reference engine that omits vertical-substitution provenance is not evidence that Zircon can drop
  its comparison. Replace enabled/disabled output comparison only with an equally provable backend
  trace, feature-plan receipt, or validated cache, and only after managed cost/hit-rate measurement.
- Paragraph script-run segmentation uses the Unicode Script_Extensions compatibility set as its state.
  Common/Inherited are wildcard inputs: leading neutral text resolves with the first specific script,
  intermediate/trailing neutral text remains with the previous compatible run, and all-neutral text
  remains Common. Paired-bracket context may refine this rule. Do not add a delayed Common state or a
  second analysis owner when the set intersection already expresses the policy.
- A vertical cluster decision is authored once at the shaping owner and retained on the cluster head.
  Its compact basis records Unicode orientation, the effective `vert/vrt2` set, substitution proof,
  and typed fallback reason; the complete view composes the co-located rotation and selected
  face/instance instead of duplicating font identity in per-glyph metadata.
- Compatibility shaping must mark unavailable vertical-substitution provenance as unknown. Layout,
  renderer, and diagnostics cannot infer a specific GSUB lookup, rerun Unicode orientation, or query a
  font database to reconstruct a decision already owned by shaping.
- Alternate-backend subrun recovery is owned entirely by shaping. Direct publishes ordered typed
  holes; one alternate candidate is qualified by identity, line topology, monotonic source ranges,
  exact hole containment, and coverage before any merge. UI and renderer cannot splice glyphs or
  trigger a local reshape.
- A hybrid shaped artifact retains alternate ranges and its first typed failure at the run owner.
  Ordinary runs pay only the optional pointer; hybrid heap residency participates in shaped-cache byte
  admission. Failure classification remains a single owner and artifact projection cannot reclassify
  a terminal error.
- Hole qualification and merge use cross-line monotonic cursors with `O(lines + holes + glyphs)`
  complexity. Failed qualification keeps the already-built whole alternate candidate. Profiling is one
  request-level low-cardinality aggregate and cannot add per-glyph events or source labels.
- A shaped source lease must name an immutable owner snapshot, a validated owner-local range, and the
  absolute source origin. Shaped/cache/layout code consumes the lease view; renderer code cannot own or
  reconstruct source lifetime. Wire serialization may emit the exact leased slice but cannot silently
  restore a mutable/current document revision.
- Shared source residency is charged once by the cache/document owner across insert, update, eviction,
  and clear. Per-run full-document charging and `Arc::strong_count` admission are invalid. Source
  lifetime and glyph SoA migrations require separate measurements and must not be bundled into one
  unreviewable storage rewrite.
- A retained paragraph artifact is not justified by naming alone. Before merging Bidi/script,
  line-break, hard-line, shaped-run, and layout-line lifetimes, measure duplicate construction on
  direct-success and fallback paths across cold/warm and document sizes. A document-revision artifact
  may own dirty-range dependencies only after that evidence; source leases, glyph SoA, and renderer
  artifacts remain separate ownership decisions.
- A process-local cache fingerprint uses `EphemeralCacheHash`, has no serialization/byte-export
  contract, and only narrows lookup. Full key equality and exact source qualification remain the
  admission authority; renaming a transient value to `content_hash` does not make it stable.
- Artifact/replay identities use a deterministic digest type plus an explicit format/domain version.
  Runtime cache hashes cannot cross that boundary. Conversely, do not add an `O(document bytes)`
  stable digest to viewport work when a retained owner+revision identity already provides `O(1)`
  invalidation.
- A runtime budget stays with the owner that enforces it. Correctness context, per-request work,
  cache residency, async completion, and renderer shadow memory cannot be combined into one mutable
  cross-domain profile merely because their defaults are numeric constants.
- Every variable-size resident or queued text budget must expose the effective limit beside current
  usage and pressure/rejection state. A central profiler may project those immutable snapshots under
  stable low-cardinality names, but it cannot become a second admission or policy owner.
- Correctness bounds such as boundary-shaping context are not runtime tuning knobs. Change them only
  with corpus evidence and an explicit safe fallback; profiling may observe the bound but cannot
  select a different algorithm.
- Runtime Text terminology separates source range, shaped hard line, and final visual line.
  `ShapedHardLine` is a pre-wrap shaping projection; `CandidateLine`/`UiResolvedTextLine` own wrap,
  overflow, and placement. Provider/session shaping entrypoints are range-named because one request
  may contain multiple hard lines. Do not restore line-named shaping APIs or compatibility aliases.
- Runtime Text non-ready causes remain request-owned until the public neutral projection. Missing
  primary face is a terminal font-resolution receipt; font-generation instability is deferred.
  Session and parallel diagnostics must not count deferred work as terminal or recover either cause
  from empty glyph output.
- Capability diagnostics use stable enum codes and fixed aggregate profile names. Source text,
  document identity, pointer values, candidate-family names, and other dynamic labels cannot enter
  profiler dimensions; exact candidate traces require a bounded request/document owner.
- Font-resolution work receipts are request history, not shaped artifact truth. Carry them beside the
  shaped value in a transient completion envelope and merge them at the session/operation owner. Do
  not serialize them, charge them as shaped-cache residency, or replay miss-time candidate work on a
  cache hit.
- Resolution counters must be incremented inside the existing decision loops. A coverage-work metric
  counts actual coverage probes, including candidate compilation and partial ranking; it cannot run a
  second diagnostic coverage pass. Generation retries accumulate discarded attempt work before the
  final Ready, terminal, or deferred publication.
- Shared-cache lock timing must preserve the same attribution rule. Global cumulative cache statistics
  may support an isolated benchmark, but overlapping global before/after snapshots cannot be assigned
  to concurrent shaping requests. Profiling builds aggregate lock acquire/wait/hold at the cache owner
  in request-scoped TLS and publish one fixed completion value; ordinary builds must not pay timing or
  profiler-lock cost per cache access.
- Paragraph-analysis optimization is lifetime-sensitive. Count and time Bidi, script/emoji, and
  line-break construction at the canonical shape-request owner before hoisting a value across direct,
  alternate, rich, layout, or document boundaries. A profiling-only request TLS may aggregate fixed
  values; it cannot become production retained state. Source lease, glyph storage, dirty dependency,
  and renderer artifact migrations remain independent decisions.
- Backend-cluster geometry has one authority. Wrap, caret, hit-test, selection, IME, accessibility,
  and renderer projection consume the canonical glyph artifact or the same typed measured-cluster
  index; UI code cannot reconstruct an interior ligature caret from per-grapheme DTO advances. A
  missing-artifact source fallback must prove exact source/visual/style/direction/writing-mode
  congruence, shape the complete final physical line, and otherwise fail closed to the DTO contract.
- Without a font-derived caret provider, a multi-grapheme backend cluster is atomic. Source caret
  affinity selects one cluster edge, physical hit uses the cluster midpoint, and any intersecting
  selection covers the whole cluster. Do not publish equally spaced or proportional interior carets
  as font truth; adding GDEF LigCaretList is an explicit capability upgrade.
- Retained text mutation must be revision-qualified at the owner boundary. A source-changing edit
  accepts an expected document key and must reject stale or exhausted revisions before changing
  pieces, length, indexes, or cache identity. `saturating_add` is not valid for document/layout
  generations because it can publish changed source under an existing key.
- Exact source equality is a typed document outcome, not a changed receipt with an invented dirty
  range. An unchanged replacement does not advance revision, append source chunks, invalidate line/
  grapheme state, or enter history. Piece-backed equality may scan the requested range but must not
  flatten or hash the whole document merely to detect a no-op.
- A serialized surface revision without a typed error return reserves an explicit unpublishable
  exhaustion state. Reaching that state may continue exact-source layout, prewarm and editable
  geometry, but it cannot wrap or form a retained cache key. Missing retained identity disables reuse,
  never the underlying text-layout operation.
- A stable mutable document authority is not a cloneable value. Cloning owner plus revision permits
  divergent sources to publish the same identity; clone only immutable snapshots or source pieces,
  and let the document service allocate a distinct owner for a distinct mutable authority.
- A contiguous document snapshot is an explicit revision-bound lease, not an unqualified `String`
  clone. Piece-backed storage may lazily flatten at most once for a requested revision and share that
  immutable source by `Arc`; a changed revision receives a new slot while old leases remain stable,
  and a typed no-op preserves the current lease identity. Index, layout and host consumers borrow the
  lease instead of making another full-source copy. Lease/debug/report output never exposes content.
  Snapshot byte/age/count budgets and secure retention/zeroization belong to the document service,
  not to a cloneable Surface cache.
- A piece table is storage, not an incremental layout session. Paragraph dirtiness must come from a
  retained separator-aware hard-line owner with stable identity; taking full old/new snapshots inside
  every edit is an `O(N)` rebuild and cannot be described as incremental reflow.
- Text layout admission distinguishes an empty source from whitespace content. Do not use `trim()`
  to decide whether a layout exists: spaces, tabs, and hard separators retain physical advance,
  line boxes, and editing geometry. Trimming belongs to explicit wrap, justification, validation,
  query, or presentation policy.
- Retained hard-line identity is separator-aware and independent from wrapped visual-line identity.
  A local edit may materialize its affected hard-line envelope plus bounded separator context, but it
  cannot snapshot the complete old and new document to manufacture a dirty-line receipt. Unchanged
  source lines retain identity; a split creates identities and a merge keeps the left affected owner.
- Hard-line, grapheme and visual-line indexes must not silently duplicate authority. If a retained
  hard-line model exists, a revision-bound grapheme index owns grapheme boundaries only; layout owns
  wrapped visual lines and consumes explicit document line IDs/deltas.
- Intrinsic line size, display/clip slot and content display offset are distinct text-layout concepts.
  Do not change overflow alignment from a clamped frame observation alone: first compare the natural
  advance, viewport width and reference-engine justification rule across renderer and interaction
  consumers. Any DTO split must migrate serde, native/SDF, hit, caret, selection and IME together.
- Text-input constraint transformation has one product owner and returns a typed, low-cardinality
  receipt. Keyboard, text, IME, clipboard and accessibility routes must not infer filtering or
  truncation by comparing strings or invent route-local notes. Receipts contain counts/reasons, never
  raw input, document content or dynamic labels, and remain observable if a later mutation rejects.
- Single-line admission consumes Runtime Text's canonical hard-line separator model, including CRLF
  as one separator. Component defaults and reducers must share the same limit semantics; in
  particular, a zero catalog maximum means unbounded unless the public schema explicitly replaces
  that contract. When sanitization changes preedit text, cursor and clause endpoints use the same
  monotonic edit map; they cannot be applied as old offsets to the new string or silently discarded.
- A one-pass replacement filter does not make validation incremental while every edit still scans
  retained prefix/suffix graphemes. Structural optimization starts by wiring the revision-bound
  document/grapheme owner, then profiles edit scale and index repair; do not add a second cache or
  tune scalar predicates before the retained authority exists.
- Editable-text commands are resolved before keyboard text-payload fallback. Unmodified Enter inserts
  a hard line only for multiline input; writable single-line input consumes it as Submit without
  rewriting unchanged text properties or inventing a newline-rejection receipt. Key repeat remains
  consumed but cannot emit repeated submit events.
- An editable state projection publishes value, caret, selection and composition as one semantic
  transaction. All fallible node/metadata/property-domain/range validation happens before the first
  write; rejection cannot leave later fields committed. The commit batches retained metadata,
  component/style/binding projection and one dirty registration. A semantic value change always
  owns text/layout invalidation regardless of the authored value-property name; caret-only changes
  remain render-only. Keyboard, IME, clipboard and accessibility entry routes preserve their typed
  source kind through both retained and component-state binding updates; an accessibility action
  cannot be relabeled as generic runtime state. Composition clearing includes clauses as well as
  range/text/restore state. Generic mutation of the editable value must enter the same transaction:
  an actual display-text change preserves a still-valid caret or clamps it to a grapheme boundary,
  clears selection/composition, and commits once; an identical value is a no-op. Generic writes to
  derived caret/selection/composition properties fail closed. Keep stored `UiValue` separate from
  display text so a numeric field's external `Float` cannot be silently converted to `String`.
  Do not clone the complete UI tree per key event to obtain this atomicity.
- Raw text input has one editing authority. Keyboard text, IME commit, paste and accessibility edit
  intent enter the Surface/document transaction; an independently callable component reducer must
  not rebuild text/caret/selection/composition or apply edit actions. Editable component descriptors
  expose semantic `ValueChanged`/`Commit`/`Focus` projection only. `KeyboardText` remains valid for
  non-editable typeahead owners such as menus and command palettes, not as a second text editor.
- Editable value-property selection is canonical and resolved outside the character/glyph hot path.
  An explicit widget override wins; otherwise descriptor/schema inference uses `query`, `value`,
  `value_text`, then `text`. Surface state, semantic component projection, validation and render must
  consume that same property identity. Component aliases and roles classify through the shared
  `UiWidgetBehavior` table instead of route-local editable-name lists.
- A focused editable's bound model value and active edit buffer are distinct authorities. A property
  source label or downstream binding source kind is not a refresh policy. Requests must distinguish
  external bound refresh, explicit SetText/LoadText replacement and edit projection with typed origin
  and expected revision. Ordinary bound refresh cannot replace focused editable text; it retains at
  most one bounded pending value per product document/edit session. Explicit replacement may force
  review through the same transaction. Blur/commit must apply a compatible pending revision or emit
  a typed conflict/rebase receipt; it cannot silently discard either side or report a deferred update
  as accepted. Secure pending values remain in the secure owner and are cleared on detach, policy
  change and teardown. Do not create a second Surface cache before the product document owner exists.
- Editable property transactions preserve the retained value kind. A generic text edit cannot replace
  a `Float`, `Int`, `Bool`, collection or other typed property with `String`; value-kind mismatch is a
  typed preflight rejection before caret, selection, component, binding or dirty publication. A
  numeric field therefore owns a separate transient edit string and committed numeric value. The edit
  buffer may retain intermediate input such as a sign, decimal separator or exponent, while only the
  numeric type interface parses, clamps and publishes the typed value on its declared change/commit
  policy. Formatting/cache, cancel/blur and external refresh all consume the same product edit session;
  do not parse locale-sensitive numbers in the generic text transaction or store the buffer in the
  numeric `value` property.
- Public text edit notification carries a versioned, content-free document receipt, never a raw edit
  action or complete before/after editable-state snapshots. The receipt owns a non-nil document
  identity, strictly consecutive checked revisions, typed source/kind, old/new changed byte ranges and
  final byte selection with focus affinity. Public deserialization rejects unknown schema versions,
  nil identities, revision jumps/exhaustion and reversed ranges before consumption. The non-cloneable
  document authority signs its UUID once; receipt projection obtains that identity and both document
  lengths from the changed receipt, never from caller-supplied values. It additionally rejects owner
  changes, inconsistent revision/delta, offset narrowing, range bounds and selection bounds.
  Document-service integration still validates grapheme policy and source equality. Publication stays
  `O(1)` in document length; snapshots, source hashes and dynamic labels are obtained separately from
  a revision-bound lease and are not embedded in the event.
- Clipboard is an asynchronous text transaction, not a fire-and-forget effect. Every request carries
  a transfer identity, intent and expected edit/document revision; cut commits deletion only after a
  matching successful write result, while paste applies a matching read result through the shared
  constraint/edit owner. Failure, duplicate, owner/focus/policy/revision mismatch, clone, serde and
  teardown fail closed. Pending state is bounded per editable owner and must not clone a full document.
- Secure text classification has one typed policy owner shared by render, input, accessibility,
  clipboard and host-session projection. Product schema such as `input_kind=password` cannot be
  reinterpreted by route-local TOML helpers; conflicting or malformed secure signals fail closed.
  The keyboard command owner receives that classification explicitly: for secure text, Control plus
  Left/Right or Backspace/Delete uses hard-line boundaries and never queries the source word-boundary
  map. This follows Slate's password command policy and keeps ordinary Unicode word navigation in its
  existing shared owner.
- Secure Change/Submit publication uses a surface-owned opaque value reference, never a masked or
  raw generic `UiValue`. Only the latest node/property token may resolve against the same live
  surface and non-exhausted text revision; clone, serde, cross-surface and stale references fail
  closed. Input dispatch/direct-reply projection must redact original input, binding previous/value,
  effect/host/component payload and template action values through one owner.
- The classifier and event projection are still not a complete secure pipeline: retained state,
  history, logs/crash/plugin/export, zeroization and trusted host consumption need the same data
  classification, and disabling IME is not a secure-input session implementation.
- When inline tests push a production route over the owner budget, move the tests to a folder-backed
  child and keep the production route thin. Structure guards must then read the actual route, worker,
  and test owners separately; concatenating production and test text or continuing to scan only the
  former parent file is a false-positive contract. The shader-prewarm reference split is 142 lines for
  the route, 251 for the worker, 667 for 11 primary tests, and 159 for 2 combined-validation tests;
  status `runtime_07_15_shader_prewarm_test_owner_and_source_guard_routing_static_passed_cargo_deferred`.
- A shader registry's dependency-closure tests belong to a folder-backed child once they push the
  production owner over budget. Keep source-relative `include_str!` paths explicit after relocation,
  register production and test files as separate budget owners, and preserve dependency-order
  assertions byte-for-byte apart from that path adjustment. The current module-registry split is
  656 production lines plus a 214-line child containing 11 tests; status
  `runtime_08_15_shader_module_registry_test_owner_split_static_passed_cargo_deferred`.
- A behavior test root that already has domain children should move another cohesive behavior family
  into its own sibling rather than create a generic overflow file. Preserve concurrently added child
  routes and keep shared fixtures in the parent when the child can consume them through `super`.
  Runtime resolution currently owns a 631-line root plus dependency-cycle/exact-dependency/
  factory-panic children of 115/217/258 lines; status
  `runtime_02_15_resolution_exact_dependency_test_owner_split_static_passed_cargo_deferred`.
- UI behavior tests should group overlay-instance authority separately from generic dismissal and
  activation behavior while sharing the canonical surface fixture through the parent. Do not clone
  fixtures into the child or move a concurrent binding-contract change merely to make the split
  self-contained. The widget-menu root is currently 625 lines / 11 tests and the control-anchored
  overlay child is 239 lines / 5 tests; status
  `runtime_09_15_widget_menu_control_anchored_test_owner_split_static_passed_cargo_deferred`.
- A release-only benchmark and its deterministic ordinary regression may share a dedicated child
  with their private sampling helpers. Keep the lifecycle/transaction fixture in the parent and lock
  sample count, concurrency and latency budgets in the structure guard; moving the benchmark is not
  new performance evidence. Activation currently owns a 756-line root / 11 tests and a 132-line
  contention child / 2 tests; status
  `runtime_02_15_activation_contention_test_owner_split_static_passed_cargo_profile_deferred`.
- A compile-contract child that grows beyond the owner budget should split by graph responsibility,
  not by an arbitrary test count. Plugin-provided scene-velocity and hybrid-GI inputs form one narrow
  child because they share feature descriptors and extract fixtures; built-in LUT/Bloom/exposure/
  terminal/HZB routes remain in the parent. The current postprocess route split is 622 lines / 13
  tests plus a 262-line / 6-test `postprocess_routes/plugin_inputs.rs` owner; status
  `runtime_01_15_render_pipeline_postprocess_plugin_input_owner_split_static_passed_cargo_deferred`.
- Scene extract tests should mirror the published frame-domain boundary. Local volumetric fog belongs
  in an advanced-lighting child because it is filtered by camera render layers and published through
  `AdvancedLightingExtract.fog_volumes`; ordinary post-process volumes remain filtered by camera
  volume mask in the parent. The current split is 677 parent lines / 11 tests plus a 149-line / 3-test
  `render_post_process_extract/volumetric_fog.rs` child; status
  `runtime_07_15_scene_post_process_volumetric_fog_test_owner_split_static_passed_cargo_deferred`.
- A typed error with a substantial enum, Display mapping, and source-chain policy is a production
  responsibility of its own once the orchestration owner exceeds budget. Keep one internal re-export
  only when it preserves the existing sibling-module path; do not add a public facade or duplicate
  formatting branch. Native registration replay is now 785 orchestration lines plus a 178-line
  `registration_replay/error.rs` owner; status
  `runtime_06_15_native_registration_replay_error_owner_split_static_passed_cargo_deferred`.
- A frame profiler must separate current-frame assembly/publication from delayed GPU-result
  resolution once their combined production owner reaches the review budget. Keep the bounded
  pending ring and publication state in the parent; move timer/statistics merge, pass occurrence
  matching and subsystem budget projection together, because they mutate one late-result profile.
  Do not use the split to tune matching, copy-on-write, budgets or ring capacity without product
  profiles. `frame_profiler.rs` is now 796 lines and `frame_profiler/gpu_resolution.rs` 153 lines;
  status `runtime_17_15_frame_profiler_gpu_resolution_owner_split_static_passed_cargo_profile_deferred`.
- A dynamic ABI event adapter should keep event-kind routing and shared session dispatch in its root,
  while input families with their own payload validation and dual runtime/UI projection use named
  child owners. Keyboard and IME belong together because they share text payload and composition
  semantics; gamepad connection/button/axis plus UI navigation mapping form a second owner. Do not
  introduce per-child queues or managers. The current split is 734 root lines, 223 keyboard/IME
  lines, and 115 gamepad lines; status
  `runtime_10_12_15_dynamic_event_keyboard_ime_gamepad_owner_split_static_passed_cargo_deferred`.
- Retained document memory reporting belongs to a content-free document owner, separate from edit,
  storage, layout, and product admission policy. Reports may publish byte/count/capacity totals and a
  documented lower bound, but never source text, hashes usable as content identifiers, allocator-
  dependent guesses presented as exact bytes, or old external lease ownership they cannot observe.
  A diagnostic estimate is not a memory cap. Repeated-edit chunk/piece growth must be encoded as a
  profile hypothesis, and compaction/batching/container thresholds may be frozen only after matched
  operation-stream allocation/RSS/latency/power data identifies the dominant cost.
- A mutable text document authority follows the product editing-session lifecycle. Do not embed a
  non-cloneable authority in clone/serde `UiSurface`, and do not register a process-global manager
  whose node keys can alias across surfaces. Use prepare/commit when admission needs exact next
  topology: prepare performs every fallible structural calculation without mutation; commit rechecks
  expected identity/revision and then publishes once. Stores must require explicit limits rather than
  carry an unqualified `Default`. Managed snapshot leases are non-cloneable unless clone itself can
  fail admission, and their count/byte ownership must release on `Drop`.
- A retained UI surface root should own full rebuild/publication and dirty mutation, while a cohesive
  incremental transaction that coordinates layout, arranged, hit, render and navigation patches may
  live in one folder-backed child. The child must mutate the same surface and must not create a facade,
  second tree, cache or invalidation state. Move its fallback budget and report-patch policy with it so
  the responsibility is not split across hidden root helpers. Current shape is 500 root lines and 711
  incremental lines; four moved items are normalized-equivalent to the pre-split source. Status:
  `runtime_09_15_ui_surface_incremental_rebuild_owner_split_static_passed_cargo_profile_deferred`.
- A surface-level property transaction is a cohesive owner when one accepted mutation must update the
  canonical tree, component state, runtime style, focus/popup, editable text, clipboard revision and
  typed invalidation together. Keep those projections in one folder-backed child that mutates the same
  surface; do not scatter them into per-domain partial transactions or duplicate state stores. The
  current `surface.rs` / `surface/property_transaction.rs` split is 483 / 485 lines and all 12 moved
  items are normalized-equivalent to the pre-split source. Status:
  `runtime_09_15_ui_surface_property_transaction_owner_split_static_passed_cargo_profile_deferred`.
- Pointer event envelopes, declarative template-action projection, and pointer-driven retained-state
  invalidation are separate responsibilities inside one folder-backed event owner. Keep route-derived
  component/focus events and damage in the root; keep compiled binding handle validation, action/route
  selection, missing-value policy, and payload expression/property resolution in one action child;
  keep hover/pressed/focus state, pseudo-style propagation, and render dirty in one state child. All
  three mutate or read the same surface authorities and must not create another event router, action
  registry, state cache, tree, or binding store. Current event/action/state owners are 426 / 262 / 226
  lines; nine action methods and seven state items are normalized-equivalent to their pre-split source.
  Status: `runtime_09_15_ui_pointer_template_action_owner_split_static_passed_cargo_profile_deferred`
  with the earlier
  `runtime_09_15_ui_pointer_component_state_owner_split_static_passed_cargo_profile_deferred` retained.
- Dynamic Runtime UI action admission is a distinct folder-backed child of the surface-set owner.
  Keep `runtime_ui.rs` responsible for multi-surface routing/capture/render aggregation, and keep
  row/byte/depth budgets, secure Change supersession and secure rejection receipts in
  `runtime_ui/action_requests.rs`. The ABI DTO belongs in `runtime_api/host/ui_action.rs`; the App
  fallback diagnostic belongs in `runtime_entry_app/host_requests/ui_action.rs`. These owners must
  not grow a second action registry, execute product routes in engine core, or serialize secure
  plaintext. Current production roots remain below the 800-line review threshold; managed Cargo
  and product route validation remain pending.
- Exact text edit ranges belong to the action application owner. A final-state consumer must not infer
  old/new ranges with a whole-document prefix/suffix diff. Carry a fixed-size content-free intent and
  borrow replacement bytes from the resulting state. Multi-action key mappings must either contain at
  most one committed edit or fail as a typed transaction; state-only selection/caret/preedit changes do
  not advance document revision.
- Surface editable properties and the session-owned document form one product edit transaction. Run
  both fallible prepares and public receipt validation before mutation, then enter an infallible dual
  commit or use an explicit rollback journal. Only the document authority may sign identity/revision;
  the property transaction must not fabricate a public edit receipt.
- Focused editable model refresh belongs beside the session-owned document in a folder-backed input-
  manager child. Keep versioned request/receipt DTOs in `zircon_runtime_interface::ui::text`, pending
  identity/revision/origin and terminal receipts in `input_manager/bound_text_model_updates`, and
  secure pending plaintext in the Surface secure store. That store wraps each retained pending value
  in `Zeroizing<String>`: replacement, rejection, policy change, Surface switch, clear and teardown
  erase the allocation; successful application moves the allocation into the existing persistent
  state with `mem::take` instead of cloning it. This is a pending-store boundary only. Component
  state, retained document/history, layout intermediates, platform input and crash handling still
  require one product secure-document policy and must not be reported as zeroized. Do not add a
  second document/binding cache or
  overload generic property source kinds to mean bound refresh. A bound refresh defers while focused;
  explicit Set/Load applies immediately; blur performs document-key CAS before the existing dual
transaction. Keep pending/terminal lifecycle in the 535-line queue owner, committed-base projection
and dual commit in the 282-line transaction child, and fixed-label counters in the 137-line profile
  child; the 793-line manager root only exposes orchestration. All remain below the 800-line review
  threshold. Status: `runtime_09_15_focused_bound_model_update_owner_implemented_static_unvalidated /
  secure_pending_drop_zeroization_implemented_unvalidated /
  persistent_secure_document_zeroization_open`.
- A UI asset surface index should own forward/reverse graph edges and hot-reload target selection,
  while tolerant retained node-resource registration belongs in a named folder-backed child. Keep
  strict compile-time schema validation and diagnostics in `resource_ref/collect.rs`; do not merge it
  with the runtime metadata scan because ordinary retained metadata tables are valid input there. The
  current surface-index/resource-registration split is 758 / 175 lines and all 11 moved items are
  normalized-equivalent to their pre-split source. Status:
  `runtime_09_15_ui_asset_surface_node_resource_owner_split_static_passed_cargo_profile_deferred`.
- Scene property enumeration should mirror the component-domain topology already used by property
  writes. Keep basic entity and dynamic-metadata orchestration in the root; keep camera, mesh,
  lighting, animation and physics enumeration plus each domain's capacity budget in named children.
  Every child must operate on the same `World` and must not create another reflection registry,
  editor cache, property DTO or mutation path. Current root/camera/mesh/lighting/animation owners are
  210/49/122/153/175 lines; ten moved projection/capacity/helper blocks are normalized-equivalent to
  `HEAD`. Status:
  `runtime_08_15_scene_property_entry_component_owner_split_static_passed_cargo_profile_deferred`.
- A platform window-event adapter should keep the single host-event dispatch in its root and place
  keyboard, pointer, IME, and window metadata/metrics translations in named domain children. Those
  children must emit the same input-pump contract directly; they must not own queues, window state,
  dispatch authorities, or duplicate DTOs. Keep the public modifier adapter path stable and preserve
  ordering, synthetic flags, touch identities, scroll units, IME byte clamping, metadata and
  normalization. The current root/keyboard/pointer/IME/window owners are 530/40/161/52/51 lines,
  including about 430 retained root test lines; 17 moved function bodies are normalized-equivalent
  to `HEAD`. Status:
  `runtime_09_15_winit_translation_domain_owner_split_static_passed_cargo_product_profile_deferred`.
- Plugin package and plugin module descriptors are distinct manifest layers even when Rust exposes
  their constructors as inherent methods. Keep package identity, content, feature, shader, packaging
  and distribution construction in the package owner; keep module kind, initialization phase,
  dependencies, target modes and runtime module projection in the module owner. A folder-backed root
  may wire both without adding a builder facade or changing public method paths. Current root/module/
  package owners are 4/169/331 lines and the two impl blocks plus two helpers are normalized-equivalent
  to `HEAD`. Status:
  `runtime_06_15_plugin_manifest_constructor_owner_split_static_passed_cargo_deferred`.
- A native system-access contract has three distinct responsibilities: the declaration plan parses
  and deterministically compiles access, the plugin authority checks ownership/capability grants,
  and typed errors describe parse/authorize/resolve failures. Keep those owners folder-backed while
  preserving one plan and one World access projection; never add an authorization cache or second
  access registry as part of the split. Current root/authority/error owners are 318/89/123 lines and
  12 moved definition/implementation blocks are normalized-equivalent to `HEAD`. Status:
  `runtime_06_15_native_system_access_owner_split_static_passed_cargo_profile_deferred`.
- Artifact-cache wire types must be grouped by the asset schema they serialize. Material authoring
  state and Shader source/pipeline state may share a parent enum but must not share one declaration
  file; the parent remains navigation-only and re-exports the private wire types without changing
  bincode variant or field order. Current root/material/shader owners are 5/161/487 lines, and both
  moved definition blocks are normalized SHA-256 equivalent to `HEAD`. Status:
  `runtime_04_15_asset_artifact_material_shader_owner_split_static_passed_cargo_deferred`.
- An ECS component registry should keep stable component identity, descriptors and table layouts in
  its base owner while transferred-descriptor preflight/import logs and atomic publication live in a
  transaction child. The child must mutate the same registry and must not introduce another ID
  authority, descriptor cache or compatibility facade. Current root/transaction/test owners are
  154/259/174 lines; 16 moved type, method, predicate and test blocks are normalized SHA-256
  equivalent to `HEAD`. Runtime08 inventories 76 production files. Status:
  `runtime_08_15_component_registry_transfer_owner_split_static_passed_cargo_deferred`.
- Runtime plugin availability has three owners: provider membership/projection construction,
  category/reason evaluation, and profile/manifest selection merging. Keep these folder-backed while
  preserving one descriptor index and one generation builder; never add a second provider registry,
  availability cache or compatibility facade as part of the split. Current root/evaluation/selection
  owners are 291/282/91 lines and 14 moved blocks are normalized SHA-256 equivalent to `HEAD`.
  Status:
  `runtime_06_15_plugin_availability_evaluation_selection_owner_split_static_passed_cargo_deferred`.
- Asset-management public DTO declarations, family status/issue behavior, and record-set
  aggregation are separate owners. Keep the declaration root stable while family classification
  and aggregate construction live in folder-backed children; do not create another asset read
  model, cache, or facade as part of structural cleanup. Current root/family/record-set owners are
  164/176/254 lines and 21 declaration/implementation blocks are normalized SHA-256 equivalent to
    `HEAD`. `ProjectAssetManager` now owns the immutable asset-only generation projection and
    refresh-before-broadcast publication; Cargo/product/profile validation remains deferred. Status:
  `runtime_04_15_asset_management_generation_static_implemented_cargo_deferred`.
- A viewport output target declaration owns target identity, resolved size, and format only.
  Render-graph import planning and final texture writeback planning are separate consumption-stage
  owners, even though both use the same target and format labels; tests belong in a third child.
  Current root/writeback/graph-import/test owners are 83/217/217/268 lines and 26 production/
  behavior blocks are normalized SHA-256 equivalent to `HEAD`. Do not create a second target
  authority or fold retained camera-plan optimization into a structural split. Status:
  `runtime_render_09_15_viewport_output_planner_owner_split_static_passed_cargo_deferred`.
- An ECS archetype index must not use a constant equality implementation to bypass erased component
  values. Keep structural identity owner-local and allocation-free: compare the signature lookup,
  component inverted index, ordered archetype IDs/signatures and entity rows; exclude diagnostics
  counters and membership revision history. Preserve the existing `World` equality call surface and
  do not add a public snapshot DTO, second topology store or cache. Status:
  `runtime_08_15_archetype_topology_equality_receipt_static_passed_cargo_deferred`.
- A retained text advance index owns production grapheme metrics, prefix advances, backend-cluster
  geometry and break-safety queries; its behavior regressions belong in a folder-backed test child.
  Keep `text/layout/advance_index.rs` free of an inline test owner so profiling-gated final-line or
  geometry work cannot push the production root across the review threshold. The current root/test
  owners are 521/192 lines, with no algorithm, public contract, cache or renderer change. Status:
  `runtime_text_03_advance_index_test_owner_split_static_passed_cargo_deferred`.
- Rich-text dialect parsing and native bitmap source-cache worker execution are separate owners
  from their shared state roots. Keep Markdown token walking in `rich/parser/markdown.rs` while the
  parser root retains the shared builder and HTML/BBCode dispatch; keep raster request/completion/
  cancellation state transitions in `native_bitmap_atlas/source_cache/worker.rs` while the cache
  root retains residency, LRU, readiness and invalidation state. Do not duplicate either builder or
  cache state. Current owners are 771/77 and 597/253 lines. Status:
  `runtime_text_04_07_parser_and_source_cache_worker_owner_split_static_passed_cargo_deferred`.
- The UI text layout root owns entry routing, rich/table selection and renderer-artifact attachment;
  plain physical-line materialization owns viewport/full-line selection, paragraph constraints,
  ellipsis/justify, physical fragments and resolved-line assembly. Keep the latter in
  `ui/text/layout_engine/plain_layout.rs` and mutate the same request-local layout result. Current
  root/plain owners are 404/400 lines; no second layout engine, cache or output DTO is permitted.
  Status: `runtime_text_03_plain_layout_owner_split_static_passed_cargo_deferred`.
- UI text measure-cache production behavior and its cache-generation/retained-document regressions
  are separate owners. Keep key construction, prewarm, measure/layout resolution and retained-cache
  access in `ui/text/measure_cache.rs`; keep the four private cache contract tests in
  `ui/text/measure_cache/generation_key_tests.rs`. Current root/test owners are 678/117 lines, with
  no cache policy, threshold, public contract or algorithm change. Status:
  `runtime_text_09_measure_cache_test_owner_split_static_passed_cargo_deferred`.
- Runtime UI dispatch output collection follows the reply/host boundary: action and generic host
  queues retain their independent typed contracts, while IME/clipboard/action/generic drain wiring
  belongs in `dynamic_api/session/runtime_ui/host_request_drain.rs`. Keep `runtime_ui.rs` as the
  SurfaceSet coordinator and do not reintroduce platform-request matching there. Current root/drain
  owners are 762/50 lines. Status:
  `runtime_text_08_dynamic_ui_host_drain_owner_split_static_passed_cargo_deferred`.
- Retained node lifetime identity belongs to the tree insertion owner, not to each input/cache
  consumer. `UiTreeNodes::insert` is the only mutable-map insertion API and assigns the monotonic
  insertion serial used by paint order; `clear()` must retain the retired high-water mark.
  `UiTree::node_incarnation` exposes that serial read-only. Consumers may combine it with their own
  session identity, but must not invalidate every owner for an unrelated topology generation or add
  a parallel incarnation registry. NumberField model UUIDs use this contract while layout generation
  only gates detached-owner pruning. Current tree/gateway/test owners are 691/293/528 lines. Status:
  `runtime_text_08_node_incarnation_authority_static_implemented_managed_validation_pending`.
- SDF atlas key collection has one iterator owner in
  `graphics/scene/scene_renderer/ui/sdf_atlas/text_keys.rs`. Retained segment preparation passes its
  flattened iterator directly; slice-based cache discard and standalone planning pass `texts.iter()`
  directly. The slice helper remains `cfg(test)` and must not be called from production or restored
  as a compatibility facade. This keeps segmented frame assembly from rebuilding a temporary batch
  vector and prevents test-only symbols from entering default Runtime code. Current atlas/key owners
  are 793/281 lines. Status:
  `runtime_text_04_sdf_atlas_iterator_owner_static_passed_cargo_wgpu_deferred`.
- A packaged runtime font baseline and a project font override are separate ownership layers. Keep the
  engine-owned default `CompositeFont`/UI family alive independently from project asset attachment, and
  resolve composites in the fixed order explicit request, project override, runtime baseline. Clearing a
  project projection must restore the runtime baseline rather than host generic defaults. Keep the runtime
  primary face in the same database generation so SDF/default metrics do not reopen loose assets. Rebuild both
  compiled indexes through one fallback-cache generation owner; do not parse the manifest, query the
  filesystem, discover system fonts, acquire a lock or allocate in the per-glyph fallback loop. Status:
  `runtime_text_01_default_composite_baseline_owner_static_implemented_managed_validation_pending`.
- An unspecified text family is a request for the effective default font, not a request to enumerate
  platform fallback first. Keep primary selection ordered as explicit family, project default, exact
  runtime primary, runtime family, then platform/asset fallback. Runtime/project default mutations must
  detach face-match and fallback caches. Do not place the private packaged primary in the public fallback
  family list or let process-global test history supply the default face implicitly. Status:
  `runtime_text_01_clean_process_default_face_owner_static_implemented_managed_validation_pending`.
- A text font object and a family/typeface selector are different identities. Keep the asset URI in the
  FontDatabase owner lookup and the optional family in owner-local face matching; never alias the URI into
  the global family index. The same owner scope must feed shaping, fallback, fixed-height line-metric
  certification, shaped-cache keys and fallback-cache keys. Compile each owner CompositeFont when the font
  generation is published, and retain the derived index through `Arc`; do not hash its authored range table
  per request. Owner attach/remove is a render-input change even when physical face bytes remain shared.
  Owner fallback may add base/platform families but must not consume another asset's fallback declaration.
  If an explicit owner is unavailable, discard its owner-local family before entering the project/runtime
  default chain; never reinterpret that typeface as a global family. Registered owners must keep the borrowed
  query path allocation-free, and SDF stale-handle recovery must use the same constraint while discarding the
  stale glyph ID. For a registered owner, preserve family provenance through deduplication: request typefaces
  are owner-local only, while CompositeFont/asset/base fallback declarations may explicitly authorize an
  owner-then-global search. Never infer external authority solely because the owner lacks a same-name face.
  Retain the owner's ordered face set as a generation-local immutable slice and share it across primary,
  fallback, metric, and recovery queries; do not reconstruct that owner set from source indexes per request.
  Status:
  `runtime_text_01_02_03_05_06_09_font_object_owner_scope_static_implemented_managed_validation_pending`.
- Missing-glyph publication must retain a real generation-owned face lineage. After the authored fallback chain
  is exhausted, use the packaged runtime last-resort face; never synthesize a glyph ID from source text or shape
  through an arbitrary FontObject primary as the global missing-glyph policy. The fixed-height metric envelope and
  Native/SDF source lookup must consume the same face. A zero fallback-depth request remains fail-closed and does
  not bypass its budget. Status:
  `runtime_text_01_02_03_05_06_09_engine_last_resort_face_static_implemented_product_validation_pending`.
- Font publication identities must own their resource lifetime. A shaping or raster work item may not retain a
  bare generation number and later reacquire a process-global database. Acquire one immutable
  `FontCollectionSnapshot` containing generation plus `Arc<FontDatabase>` and pass it through the complete work
  attempt. Generation remains the render-input invalidation key; Arc identity is lifetime only, so
  diagnostic-only equivalent publications must not rebuild locale/font caches. The process-global accessor is
  bootstrap debt, not the final manager boundary. Status:
  `runtime_text_font_collection_snapshot_static_implemented_session_manager_cutover_pending`.
- A font handle is meaningful only inside the collection and generation that issued it. Registry state, immutable
  resolver snapshots, and metrics belong to `FontCollectionService`; shaping, artifact projection, SDF, and raster
  consumers must receive that owner explicitly. An in-flight raster artifact retains both its database snapshot and
  handle-resolver snapshot. It may finish after a new publication, but a new consumer may not acquire an old lease or
  resolve through the process-default registry. Status:
  `runtime_text_collection_qualified_registry_and_inflight_lease_static_implemented_managed_validation_pending`.
- Scene/UI renderer children must not recover process font state after their owner has been selected. Inject one
  `FontCollectionService` through `ScreenSpaceUiRenderer -> ScreenSpaceUiTextSystem -> TextRenderState`; use the
  complete revision in plan/segment cache keys and compare artifact leases before raster admission. Process task-
  pool sizing may be reused, but it must not implicitly choose the font collection. The process collection accessor
  belongs to the single-process Editor host and explicit bootstrap/test adapters; Runtime renderer, project UI, and
  HUD/menu fallback paths resolve their collection from Core. Window/PIE product execution remains pending. Status:
  `runtime_text_core_collection_boundary_static_implemented_product_wiring_validation_pending`.
- Runtime font asset URI ownership must be collection-scoped and aggregate, never a renderer-local delete list.
   A dynamic UI or renderer consumer acquires one non-Clone `RuntimeFontAssetClaimScope`; shared owners remain
   resident until the last scope releases them. Text Core batches all newly unclaimed owner retirement into one
   database mutation/publication and restores the packaged default projection in that transaction. Stable renderer
   reconciliation may only perform dependency-count/hash membership checks; it must not take the claim mutex,
   allocate strings/Arcs, clone the database, or publish a generation. Consumer-local ready/missing/error caches
  must prune released identities so re-admission after project/session changes retries the owner. Status:
  `runtime_text_font_owner_claim_scope_static_implemented_managed_validation_pending`.
- Font publication copy boundaries must remain explicit. The outer collection mutation clone protects
  immutable snapshot leases; an owner registration staging clone protects direct database error atomicity;
  an owned database return clone is permitted only for legacy mutable consumers. Receipt-only or claim/
  admission callers must use a published `Arc<FontDatabase>`/snapshot result so they do not copy the full
  database after publication. Each boundary requires a fixed profiling span/counter before any in-place
  registration API is attempted; do not infer zero-copy or performance gains from source shape alone.
  Status: `runtime_text_font_publication_clone_boundaries_profile_plan_written_managed_validation_pending`.
- Renderer font admission must have one production batch owner. Legacy single-asset load/resolve and
  standalone admit/retire wrappers are not retained. A test-only ensure helper may exist only when it
  calls the production batch refresh with an isolated collection and a real claim scope; it must not
  implement per-asset collection publication/global lookup semantics.
- SDF/native raster consumers are lookup-only with respect to runtime font assets. They consume the
  exact collection database adopted before shaping/raster preparation and may not parse a runtime font
  manifest, register/remove an asset owner, or publish a collection generation on a raster cache miss.
  Offline cooked-artifact lookup remains a separate owner; fixture registration helpers must be
  `cfg(test)`. Status: `runtime_text_single_font_admission_owner_and_sdf_lookup_only_static_implemented`.

## 2026-08-29 Shader Prewarm Inventory Owner Note

- `bin/zircon_shader_prewarm/manifest/asset_inventory.rs` must own only inventory state, bounded
  payload collection and orchestration. Persisted warm snapshot/index lifecycle belongs to
  `asset_inventory/snapshot.rs`; deterministic path discovery and link/reparse rejection belong to
  `asset_inventory/traversal.rs`. The split is source-complete at 224/431/159 lines with a focused
  owner guard and isolated production metadata compile; managed Cargo/product validation remains
  pending. Status:
   `runtime_15_shader_prewarm_asset_inventory_owner_split_static_metadata_passed_cargo_product_deferred`.

## 2026-08-29 Light Grid Projection Correctness Note

- `light_grid_builder.rs` now consumes the shared `ViewProjectionMatrixPair` projection owner,
  uses the canonical orthographic half-height and keeps
  perspective spheres that cross the near plane. Camera-inside spheres conservatively cover all
  tiles; a sphere wholly behind the camera remains depth-culled. Inline regressions and an E-drive
  isolated rustc harness cover shared projection equality, canonical orthographic scale,
  camera-inside, behind-center near-crossing and fully-behind cases (5/5). This is a
  correctness-only P0-3 repair; tile budgets,
  z-bin layout and scheduling are unchanged. Managed Cargo/WGPU/product validation is pending.
  Status: `runtime_render05_light_grid_projection_correctness_isolated_tests_passed_managed_validation_pending`.

## 2026-08-30 Runtime Text range-invariant owner note

- `text/layout/line_break/mod.rs` remains the single hard-line/glyph cluster boundary owner. Invalid
  source slices and malformed non-virtual cluster ranges now return typed `TextShapingOutcome::Failed`
  instead of being skipped, while legal zero-width virtual anchors remain representable.
- `ui/text/layout_engine/wrapping.rs` keeps corrected range materialization in the wrapping owner and
  returns `LayoutFailed` when a shared metric cannot be projected to source UTF-8. The production path
  therefore cannot publish a partially materialized line after a range invariant violation.
- `text/shaping/cosmic.rs` and `text/shaping/cosmic/hard_lines.rs` own alternate-backend line/glyph
  projection. Checked range arithmetic, UTF-8 boundaries and hard-line containment are validated before
  normalization; invalid Cosmic output returns the existing typed fallback error rather than fabricating
  an empty cluster or silently dropping the glyph.
- The change is a non-validation correctness hardening; the 16-case static suite, Python compile check
  and scoped diff check pass. Managed Cargo, product WGPU/PNG, profiling, RSS/power and matched Unreal
  comparison remain pending. Status: `runtime_text_range_invariant_fail_closed_static_implemented /
  partial_layout_suppression_removed / managed_validation_pending`.

## 2026-08-30 Runtime UI text render-extract owner note

- `ui/surface/render/extract.rs` is the render-command orchestration owner; it no longer embeds popup
  anchor traversal/tests or owner-text prewarm collection/suppression policy. Those responsibilities are
  child-owned by `extract/popup_anchor.rs` and `extract/owner_text_prewarm.rs`.
- Current owner sizes are 632/208/158 lines. Popup frame resolution, two popup regressions, prewarm
  overlap admission, partial-viewport rejection, suppression predicates and prewarm request construction
  moved without changing thresholds, algorithms or render-command order.

## 2026-08-30 Runtime Text input geometry owner note

- Once a retained `UiSurface` selects a `FontCollectionService`, every derived text geometry path
  belongs to that same owner. Missing-artifact caret, selection, IME rectangle and pointer hit-test
  recovery must accept the Surface's immutable `FontCollectionSnapshot`; it may not instantiate a
  process-default shaper after layout publication.
- A neutral layout DTO without a collection revision must not be combined with an arbitrary current
  snapshot. Gate source-metric recovery on the Surface's observed layout generation; on mismatch,
  consume the published artifact/glyph advances and let the normal layout invalidation path rebuild.
- A collection-bound direct provider is the low-level compatibility mechanism, not a second shaping
  engine. It must publish the injected collection revision and use the canonical shaping backend.
  Editor-host/standalone helpers may acquire the process snapshot only because that process collection
  is their declared owner.
- Capture one snapshot lease per input query/context update and share it across its geometry calls.
  Do not add per-glyph service resolution or a second metrics cache. This mirrors Unreal
  `FSlateFontMeasure` retaining one concrete `FSlateFontCache`. Status:
  `runtime_text_surface_input_geometry_collection_owner_static_implemented /
  managed_validation_pending`.

## 2026-08-30 Runtime Text one-shot provider owner note

- A compatibility text provider is an operation owner, not a stateless convenience value. It must
  capture one immutable font snapshot before the first request and reuse it for all metrics, source
  ranges and vertical/horizontal shaping in that operation.
- Publication during an operation must not silently switch collection or generation. The provider
  reports the captured revision and delegates to the canonical collection-bound backend; Runtime
  retained paths should prefer their long-lived Surface/Session owner.

## 2026-08-30 Runtime Text retained-index preflight note

- Incremental grapheme-index admission is allowed only after the retained document owner proves
  exact revision/boundary identity and ASCII/no-CRLF context. The preflight walks the existing
  pieces without flattening or allocating a context string, checks every checked source slice, and
  rejects incomplete coverage. Unicode, combining/emoji/ZWJ/RI, CRLF, stale revisions and all
  arithmetic failures retain the canonical full-rebuild fallback.
- This is an ownership and allocation-boundary correction, not a measured latency or power result.
  The source-index splice remains managed-profile gated and must not create a second grapheme or
  line-break authority.
- This rule follows Unreal `FSlateFontMeasure` retaining one concrete `FSlateFontCache` and prevents
  cross-generation geometry without adding a per-glyph lookup or a second cache. Status:
  `runtime_text_one_shot_provider_snapshot_owner_static_implemented / managed_validation_pending`.
## 2026-08-30 Runtime Text rich-source contract note

- `text/layout/rich_source.rs` is the sole validator for the borrowed `RichTextLayoutSource`
  contract. It rejects missing runs, non-sentinel non-increasing parent source indices,
  overlapping/empty/out-of-bounds ranges, integer conversion failures, and non-UTF-8 boundaries
  before rich index, artifact, materialization, or prewarm consumers can run.
- Legal uncovered source gaps remain owned by `RichAdvanceIndex`, which fills them with the base
  style. No renderer/UI fallback or second source validator was introduced; malformed inputs use
  the existing `TextLayoutError::LayoutFailed` path.
- `ui/text/layout_engine/rich_table/layout.rs` owns table/cell source-range admission. It must use
  checked absolute-to-projection conversion, preserve legal empty cells, and reject reversed,
  cross-projection, parent-outside, non-UTF-8, or overlapping/descending cell ranges before table
  sizing or cell slicing. `rich_table/source_slice.rs` rejects malformed local segment ranges before
  projection, and delimiter trimming consumes `text::hard_line::is_hard_line_separator` rather than
  maintaining a private LF-only rule. `UiParsedText::project_range` is the single checked projection
  owner and returns typed `TextLayoutError`; adapters only propagate it. Legal gaps between cells
  remain allowed.
- This is a static correctness slice. Managed Cargo, real WGPU/PNG, profiling, RSS/power, and
  Unreal-matched product validation remain open.

## 2026-08-30 Runtime Text measurement source admission

- `text/layout/measure.rs` owns shaped-geometry source admission before grapheme advance
  projection. It must validate exact source snapshot identity, absolute source span, hard-line and
  glyph containment, checked arithmetic, and UTF-8 boundaries; callers must propagate the existing
  `TextShapingOutcome` instead of clamping an invalid range or manufacturing partial geometry.
- `source_grapheme_span` is a projection helper, not an authority: it performs no `min/max` range
  recovery. Legitimate zero-width virtual anchors remain legal only at a source boundary, while a
  malformed cached/compatibility run fails the measurement owner closed.
- The single-shape and physical-line metric provider regressions are owned by
  `text/layout/measure/measured_line_contract_tests.rs`, not an inline production block. Current
  production/general-test/focused-contract sizes are 746/437/116 lines, restoring the 800-line
  review budget without changing the measurement algorithm or public API.
- This slice does not claim runtime performance, power, WGPU, or screenshot acceptance; those
  measurements remain managed-validation gated.

## 2026-08-30 VerticalRl rich range admission

- The VerticalRl rich column owner must use one checked conversion helper for absolute `(u32, u32)`
  ranges and one checked publication helper for generated columns. Failed conversions may not use
  `usize::MAX`, `u32::MAX`, the original start, or another sentinel as a successful range.
- Forced lines, line-break chunks, fallback segments, leading-space trimming, and final column
  publication all share this admission boundary before `RichAdvanceIndex` measurement. Malformed
  source input returns the existing `TextLayoutError::LayoutFailed`; legal empty ranges remain
  representable.

## 2026-08-30 Rich advance-index outcome ownership

- Rich advance-index append stages return `TextLayoutOutcome`; retryable font-generation changes
  must remain `Deferred` until the publication owner applies its retry policy. Structural source and
  range errors use the existing `Failed` path.
- Horizontal and VerticalRl forced-line/range conversion must consume the `rich_source.rs` checked
  source-range owner; no renderer/layout module may recover with integer sentinels or unchecked
  chunk offsets.
- Rich forced-line extraction should use the canonical hard-line visitor with output preallocation;
  a bounded capacity scan is acceptable, but callers must not reintroduce a second separator scanner
  or an intermediate document-sized line vector.
- Rich source validation and span projection should consume `for_each_validated_rich_run`; compact
  `(u32, u32)` publication belongs to `checked_source_range_to_u32` in `rich_source.rs`.

- Text service projection helpers used across sibling text modules must have one crate-local owner;
  optional hash branches must remain unit-typed, family identity maps must declare their key/value
  types, SDF lookup-only caches must receive only the database authority they consume, and Cosmic
  caches must clone the immutable font snapshot while snapshot identity accessors remain runtime
  methods when they dereference `Arc` state.

## 2026-08-30 Runtime Text parser admission owner

- Indexed rich-text source and output require an explicit request budget. Source admission belongs
  before cache lookup/copy; cache residency limits are not parser authorization.
- Byte offsets, collection counts and projection indices must use one checked artifact-index owner.
  `u32::MAX` saturation, truncating casts, skipped entries, or empty-slice recovery are not valid
  identities.
- Parser capacity failure is a typed public error. UI may project it into a stable low-cardinality
  layout diagnostic, but must not publish a partial artifact or silently fall back to unbounded parse.
- Inline style/link depth belongs to the shared request-local ActiveTag owner used by HTML and BBCode.
  A parser must reject depth `max + 1` before stack growth; a cache hit, benchmark corpus, or syntax
  branch may not bypass that budget. Block/table/general nesting remain separate open admission work.
- An unrepresentable paragraph/list projection index is a typed failure, not a `u32::MAX` alias.
  Delta-style or metadata-clone optimization requires release allocation/profile evidence before code
  changes; bounded depth alone is a correctness and resource-safety contract, not a speed claim.
- Tokenizer admission is narrower than parser/cache residency: total recognized token count belongs
  to the request builder; per-token encoded bytes and per-token attribute count/bytes belong to one
  `RichTokenizerBudget` consumed by HTML and BBCode before tag-name/key/value allocation. BBCode
  `tag=value` may not bypass the attribute-byte budget. Markdown paired delimiters consume two token
  units before style dispatch. Node/span/time/decorator budgets remain open and must not be inferred.
- Parser builder/admission state is a cohesive child owner when it would push the semantic parser root
  over the 800-line review budget. The child mutates the same request-local result and must not create
  a second parser, cache, or format dispatch.

## 2026-08-30 Runtime Text table projection owner

- Table-cell projection must not rescan the complete run, paragraph, and table arrays once per cell.
  The compiled owner may build a request-local interval structure over checked canonical ranges;
  subtree `max_end` pruning must reject non-intersecting branches before output allocation.
- The interval structure is construction-local and is dropped before the compiled artifact is
  published. Retained state remains only the compact checked cell projection consumed by layout;
  no global interval cache or second source-range authority is allowed.
- Producers must emit source-index order without duplicates. Private UI projection must not sort and
  deduplicate the same indices again. At 4,096 objects the removed scan measured 50,331,648
  comparisons with p50 60,544 us over 31 samples. The final isolated interval path enters 215,046
  nodes and measures p50/p95/p99 3,337/4,467/5,611 us; the p50 improvement is 18.14x. Canonical
  source order must take a linear check path; defensive malformed/out-of-order constructor input may
  sort before tree construction. The 4,096-object first-sample working-set delta rose from 208,896 to
  360,448 bytes, so end-to-end allocation/RSS/power claims still require a matched managed profile.
  Status: `runtime_text_table_projection_interval_owner_static /
  quadratic_rescan_removed_isolated_profiled / managed_validation_pending`.

## 2026-08-30 Runtime Text rich cluster ownership

- A compiled rich-markup artifact owns stripped text, source/run ranges, semantic metadata and
  resource/projection indices actually consumed by downstream systems. It must not materialize or
  retain a document-wide grapheme/cluster vector solely for prospective use.
- Grapheme and glyph-cluster computation belongs to shaping/layout owners with a request, viewport,
  direction, font generation and actual consumer. Parser/compiler code must not import segmentation
  only to duplicate that derived index, nor expose a compatibility accessor or lazy second cache.
- Structural removal follows an owner profile, not source intuition. The removed implementation's
  1/8/32 MiB ASCII vector payload measured 8/64/256 MiB and p50 65,236/736,093/3,074,179 us over 31
  release samples. After hard cutover the isolated owner has exact payload zero and no build stage;
  broader latency/power claims still require managed end-to-end validation. Status:
  `runtime_text_rich_cluster_owner_converged_static / managed_product_validation_pending`.

## 2026-08-30 Runtime Text representation count admission

- Parser admission must bound run, paragraph, table and table-cell materialization, and compiled
  projection must bound the total retained cell-index output. Each owner returns a typed failure before
  its vector grows; a faster interval query is not authorization for unbounded hostile input.
- The request-local interval structure may be dropped after compiled construction. Retained state is
  only the compact checked projection consumed by layout, with source-index order and no duplicate
  cleanup pass in UI. Status: `runtime_text_representation_count_and_projection_admission_static /
  managed_validation_pending`.
- Block/table nesting is a separate typed admission boundary. A parser may not silently suppress
  owners beyond a budget or saturate multiple table depths into one compact identity. Grapheme-run
  normalization is a cohesive parser child owner when keeping it in the semantic parser root would
  cross the 800-line file budget; current root/builder/alignment sizes are 715/162/100 lines.

## 2026-08-30 Runtime Text exact-tag decorator dispatch owner

- When a provider contract assigns one unique normalized tag at registration, the parser-local keyed
  registry is the sole dispatch owner. A token must not scan unrelated providers, and a keyed index
  must not coexist with a mirror vector or global fallback registry.
- Registration performs reserved/duplicate admission and insertion as one owner operation, then the
  parser advances its existing generation. Immutable compiled artifacts keep the generation in their
  cache identity; dispatch optimization must not rewrite old artifacts or bypass invalidation.
- Unreal's ordered decorator scan is tied to arbitrary `ITextDecorator::Supports` predicates. That
  scan is not a requirement for Zircon's stronger exact-tag contract. The owner/lifetime boundary is
  the reference behavior to align, while lookup follows Zircon's declared identity semantics.
- The isolated 4,096-dispatch/4,096-decorator lane improved p50 from 116,314 to 139 us after the hard
  cut. This closes only decorator-count-dependent lookup work. Callback isolation, provider leases,
  retained registry memory, product power and render acceptance remain separate gates.
- A custom callback unwind is a typed parser/provider failure, not a process-wide parser contract and
  not a byte-budget diagnostic. Accepted decorator metadata must pass a per-call bound before entering
  the active tag stack; each non-merged materialized run must also charge dynamic family/link/icon-font/
  feature bytes against a request-retained budget before publication.
- Per-call and retained-output admission cannot bound allocations privately performed and released
  inside a synchronous callback. Deadline/cancellation, allocator quota and process/plugin isolation
  remain explicit provider-lifecycle work; do not infer them from `catch_unwind` or retained-byte caps.

## 2026-08-30 Runtime Text immutable rich artifact ownership

- A production rich-text parse has one canonical immutable owner. When
  `Arc<CompiledRichText>` already owns source, parser generation, visible text, ranges, semantic
  metadata, and projections, public consumers retain that artifact and borrow parsed views from it.
- Do not expose a convenience parse API that deep-clones runs, paragraphs, tables, or dynamic metadata
  from a compiled/cache artifact. Such a clone is a second owner with incomplete identity, not a view.
  Do not preserve the old contract as an alias, facade, lazy snapshot, or second cache during hard
  cutover.
- Owned parse results may exist under `cfg(test)` for parser corpus assertions, but production builds
  must have only the canonical `compile() -> Arc<CompiledRichText>` materialization path. External
  compatibility must be migrated explicitly rather than paid as an allocation on every cache hit.
- Before this cutover, a 131,072-run isolated clone performed 395,267 allocations, requested
  32,473,088 bytes, and measured p50/p95/p99 111,366/232,754/331,802 us over 31 release samples. The
  removed production stage now has exact allocation/byte cost zero. End-to-end latency, RSS, power,
  WGPU output, and external compatibility remain separate managed validation gates.

## 2026-08-30 Runtime Text cache generation exhaustion

- Parser/provider/cache identities must never use numeric wrap, saturation, `max(1)`, or another
  fallback that maps exhaustion onto an identity previously published by a live or evicted artifact.
  Identity allocation is monotonic and nonzero; exhaustion is an explicit terminal state.
- Any mutation whose result changes a cache key computes and admits the next unique generation before
  changing the registry or provider owner. If generation is exhausted, the operation returns a typed
  error and leaves both registry contents and current generation unchanged.
- A parser without an allocatable identity may remain a constructible value when `Default` cannot
  return `Result`, but compile must fail before source copy, cache lookup, single-flight creation, or
  provider execution. UI maps this owner-lifecycle fault to generic layout failure, not a byte/count
  budget diagnostic.
- `u64::MAX - 1` may advance once to `u64::MAX`; no subsequent mutation may return zero or one. Tests
  exercise the terminal boundary with local state and must not advance the process-global allocator.
  These rules do not substitute for runtime-context service ownership, provider leases/revoke, or
  targeted generation retirement.

## 2026-08-30 Runtime Text compiled-rich owner boundary

- A compiled-rich cache belongs to the parser whose decorators, shortcode providers, generation, and
  admission budget define artifact identity. Production code must not route parser instances through
  an independent process-global cache, free compile/lookup facade, shared report, or service locator.
- Retained Runtime UI routes rich compilation through the existing Surface text session. Layout,
  measurement, prewarm, retained-document resolution, render preparation, profiling, clear, and
  teardown must observe that same owner. Independent Surface sessions must not share residency,
  counters, LRU pressure, failure cells, or clear effects.
- Cloning a retained session may retain the same parser through `Arc`; constructing a new session
  creates a new parser/cache owner. Session equality and serialization do not include transient cache
  contents. A cfg-gated static parser is permitted only for deterministic corpus tests and must not be
  reachable from production code.
- Cache ownership convergence is not a performance result. Multi-Surface aggregate quota,
  cancellation, targeted retirement, latency, allocation/RSS/contention, package power, and rendered
  WGPU evidence require separate measured gates before tuning or acceptance.

## 2026-08-30 Runtime Text provider-generation retirement

- A successful provider mutation must retire derived compiled residency owned by that parser after
  the next non-reusing generation is committed. Advancing only the lookup key while leaving old
  entries to incidental LRU eviction is not a complete lifecycle transition.
- A rejected duplicate, invalid, over-budget, or exhausted registration is transactional: it changes
  neither registry nor generation and must not evict healthy current-generation residency.
- Already-issued immutable compiled artifacts use their `Arc` lifetime as the current last-use lease;
  retirement removes cache ownership, not consumer ownership. This rule does not claim the later
  project/session/plugin snapshot, unregister/revoke fence, or concurrent publication contract.

## 2026-08-30 Runtime Text shaping-style identity

- Every rich style property that can change glyph selection, substitution, positioning, or advance
  belongs to the canonical shaping style and shaped-cache identity. Retaining italic/features only in
  parser metadata while font query or backend request uses defaults is invalid.
- Immutable OpenType features are retained once per resolved style and normalized at the backend
  request boundary. One canonical value is retained per four-byte tag using last-declaration
  precedence, then emitted in stable tag order; cache identity and backend execution must consume that
  same slice. They are not copied into glyph artifacts or reapplied by the renderer. Italic selects a
  font face through the shared query and must distinguish shaped-cache entries.
- Letter spacing is inline-axis cluster geometry, not paint decoration. Direct and fallback backends
  must converge through one neutral policy before measurement, wrap, caret, hit-test, artifact, and
  renderer consumers. Do not use a backend shortcut whose trailing-gap or RTL behavior differs from
  the canonical contract.
- Nonzero tracking must define its OpenType `liga` precedence, unit, cache bits, line/span trailing-gap
  rule, RTL/mixed/vertical behavior, negative-advance admission, and virtual/inline-object handling
  before implementation. Under the current last-declaration rule, forced `liga=0` must be appended
  after user features before canonicalization. A rich-only width correction or renderer offset pass
  is forbidden.
- Status: `runtime_text_italic_feature_identity_static_complete /
  letter_spacing_architecture_review_complete / managed_product_validation_pending`.

## 2026-08-30 Runtime Text intrinsic geometry ownership

- Intrinsic/no-wrap mode and final allotted geometry are separate contracts. A near-maximum finite
  `UiFrame` or a source-byte-derived square is not an intrinsic measurement protocol or a budget.
- Logical geometry limits belong to a runtime/session-owned, unit-explicit budget. Parser byte/count
  limits and shaping scheduling thresholds must not be reused as pixel extent policy.
- An unbounded axis is request metadata only. Measured lines, cell tracks, prefix sums, boxes, and
  published artifacts must be finite and admitted; overflow returns a typed `GeometryTooLarge`
  outcome instead of clamp-to-max or sanitize-to-zero behavior. The stable public diagnostic is
  `ZR-TEXT-LAYOUT-013` / `text.layout.geometry_too_large`; defining it does not authorize producers
  to reject geometry before the runtime/session budget and rejecting-owner receipt are available.
- Preferred cell measurement consumes natural glyph/inline-object geometry. Final table arrange uses
  resolved tracks; renderer, hit-test, and spatial-index layers may not repair invalid layout output.
- Current implementation ownership is `text/layout_geometry.rs` for unit-safe budget/constraint
  primitives, `SharedTextLayoutSession` for the immutable snapshot and rejection diagnostics,
  `layout_engine/measurement.rs` for bounded/unbounded frame projection,
  `layout_engine/geometry_admission.rs` for neutral resolved-DTO publication, and
  `rich_table/{sizing,cell_layout,geometry,layout}.rs` for checked solving and table context. Positive infinity may only originate
  from `TextLayoutAxisConstraint::Unbounded`; production geometry never uses source byte length.
- Status: `runtime_text_intrinsic_geometry_owner_and_table_cutover_static_complete /
  managed_compile_render_and_profile_pending`.

## 2026-08-30 Runtime Text compiled projection indices

- A compiled rich artifact admits byte and collection lengths before storing `u32` identity. Every UI
  projection derived from that artifact remains fallible and uses checked conversion and lookup.
- `as u32`, saturating sentinels, and `filter_map`-based silent loss are forbidden for semantic run,
  paragraph, table, or source-range identity. Invalid projection input terminates the view and prevents
  partial artifact publication.

## 2026-08-30 Runtime Text rich format version identity

- Public format names must state the implemented grammar, not a broader web/document standard.
  The current profiles are `MarkdownInlineV1`, `BbCodeV1`, and `HtmlSubsetV1`; `Markdown` and `Html`
  are forbidden until those standards' documented support matrices are actually implemented.
- Grammar version is artifact/cache identity. Runtime and interface enums use the same explicit wire
  values, parser/UI conversions remain exhaustive, and cache keys own the typed format directly.
  Parallel manual numeric tags or serde aliases for retired unversioned values are prohibited.
- Runtime parsing does not perform legacy-format migration. Authored assets/projects migrate at their
  versioned document boundary. HTML-subset recovery diagnostics are bounded canonical artifact data,
  separate from fatal parser admission/security errors. They retain typed code/severity, source-markup
  byte range and recovery only; dynamic messages and tag strings are reconstructed/localized by the
  authoring consumer. Diagnostic count has an independent request budget and truncation receipt, and
  retained cache accounting includes its capacity. Tokenizer and value projection accumulate compact
  issue flags during their existing pass; a second attribute/declaration scan is prohibited. The
  diagnostic-construction owner lives in `parser/html_diagnostics.rs`, not the shared parser root.
  Malformed tag and unterminated quoted attribute recovery preserves the exact source token as visible
  text. Malformed/unrecognized entities are diagnosed while the existing decoder preserves unknown
  source. Diagnostics are emitted in source order, use the same independent cap, and must not diagnose
  ordinary less-than text. The shared active-tag stack lives in `parser/active_tags.rs`; parser-format
  dispatch must not reacquire its indexing/depth-admission implementation.
- The HTML parse/entity-projection state machine is child-owned by `parser/html.rs`; the parser root is
  558 lines and the HTML owner is 259 lines after the 2026-08-30 split. The root retains format dispatch,
  BBCode behavior and shared metadata helpers; tokenizer, budget, diagnostic and run-alignment behavior
  is unchanged.
- Rich parser scale and release-evidence tests are child-owned by `rich/tests/parser_performance.rs`.
  The 2026-08-30 move preserves the existing corpus sizes, ignored release benchmarks, legacy comparison
  helpers, and thresholds; the rich test root is 758 lines and the performance child is 238 lines.

## 2026-08-30 Runtime Interface reflection type identity admission owner

- `reflect/type_path.rs` owns only the private DTO, fallible constructors/builders, borrowed accessors,
  and custom serde coordination. The pure Rust/VM path grammar and four byte limits live in the named
  `reflect/type_path/validation.rs` child; current owner sizes are 107/213 lines.
- Rust `::` identity and VM `.` identity are explicit grammar families. VM namespace tokens retain the
  canonical plugin-key ability to use a numeric first byte or `-`, but the terminal type remains an
  identifier. Mixed separators, generic syntax in this wire revision, invalid leaf/module/plugin text,
  and unknown serde fields fail before registry publication.
- All four stored strings/options are private. More than 70 direct read projections were hard-cut to
  borrowed accessors, both direct plugin-projection test writes now use the fallible builder, external
  struct literals are zero, and `ReflectTypeRegistration` now writes only the canonical nested plugin
  owner after validation. Its retired top-level duplicate is removed and rejected on decode. No
  compatibility constructor, second owner field, or second parser remains.
- The production validation owner passes a direct F-drive release `rustc` harness 21/21. Custom serde
  and cross-crate consumer compilation still require the managed Cargo lane, so status is
  `runtime_interface_reflect_type_path_invariant_source_complete / managed_validation_pending`.

## 2026-08-30 Runtime Interface reflection registration state ownership

- `ReflectTypeRegistration` stores one `ReflectTypeRole::{Value, Component, Resource}` field. Parallel
  component/resource booleans are forbidden because they create four wire states for a three-state
  domain and force every registry/adapter consumer to repeat exclusivity checks.
- Plugin ownership is not a registration flag. `ReflectTypePath::plugin_id` is the canonical owner and
  `is_plugin_owned()` is a derived query; registration-level owner/ownership fields, builders, and
  derive attributes are retired and rejected instead of synchronized.
- Serialization strategy, persistence eligibility, editor visibility, remote visibility, and script
  visibility remain orthogonal policy. They must not be collapsed into an enum merely to reduce field
  count; a typed flag set is permitted later only if it preserves all currently valid combinations.
- The hard cut migrates registry, scene capture/spawn, inspection, reflection, VM projection, derive,
  and focused tests. Twenty-seven affected Rust files pass parser-only validation; legacy registration
  fields/builders have zero source consumers. Status is
  `runtime_interface_reflect_registration_role_source_complete / managed_validation_pending`.

## 2026-08-30 Runtime Text table layout work receipt ownership

- Parser admission, geometry admission, and layout-work observation are separate unit domains. A parser
  table/cell/token limit cannot be reused as a layout time or line/box threshold, and source bytes cannot
  substitute for generated layout work.
- `SharedTextLayoutSession` owns the frame-scoped table work report. The report resets with `begin_frame`,
  publishes fixed-name aggregate counters at `finish_frame`, and contains no source text, table identity,
  pointer, or dynamic profiler label. Saturation is telemetry overflow handling, not layout admission.
- Instrumentation belongs at semantic execution boundaries: after checked table source admission,
  immediately before each real preferred/final cell layout, after both track owners resolve, and after
  aggregate geometry admission for published output. Render, hit-test, and adapters must not synthesize
  these counts from resolved DTOs.
- A work receipt cannot introduce skipping, partial table publication, deferred layout, a retained
  intrinsic cache, or a product threshold. Those require the documented E-drive profile matrix and an
  explicit policy owner. Current status is
  `runtime_text_table_layout_work_receipt_static_complete / managed_profile_decision_pending`.

## 2026-08-30 Runtime Text paint projection profile ownership

- Serializable UI DTO construction remains in `zircon_runtime_interface`; runtime profiling remains in
  the renderer owner. Do not add runtime diagnostics dependencies or profiler macros to the interface
  crate to time `UiRenderCommand::text_paint`.
- `render/paint_projection.rs` owns the transient text-paint profiling boundary and content-free work
  report. Counters are fixed-name aggregate values; raw text, tree/node identity, font/family value, and
  dynamic style labels are forbidden.
- Cached plans may retain prepared products but cannot replay historical work as current-frame work.
  Rebuilt segments contribute actual projection counts; exact plan-cache hits publish zero projection.
- Payload string lengths are a materialization lower bound, not allocation/capacity/RSS evidence. They
  cannot authorize a DTO owner migration without matched phase timing and allocator traces. Current
  status is `runtime_text_paint_projection_profile_static_complete /
  managed_baseline_and_owner_decision_pending`.

## 2026-08-30 Runtime Text rich semantic projection ownership

- Rich accessibility text is a projection of the current `CompiledRichText` artifact. The
  accessibility module must not parse markup, strip tags ad hoc, or concatenate resolved layout lines.
- `text/semantic_projection.rs` owns source/format validation and retains the compiled generation.
  `ui/accessibility/semantic_text.rs` owns accessibility fallback policy. Surface exposes only the
  current per-node render-command slice through its existing render-cache index.
- Plain template text remains source-owned. Rich template text with a published command range may become
  an accessible name only when command source, versioned format, artifact payload, and parser generation
  agree. Missing, stale, or ambiguous published artifacts fail closed to explicit alt/tooltip policy;
  raw markup is forbidden as a recovery value.
- Per-node lookup may scan that node's commands but may not scan the complete render extract. Candidate
  generation comparison is constant time. A semantic cache requires measured repeated-extraction cost
  and explicit source/format/provider/render invalidation before introduction.
- A rich node without any published command range may request a visibility-independent projection only
  through the Surface-retained `UiTextMeasureCache -> SharedTextLayoutSession` owner. This path reuses
  parser generation, admission budgets, and compiled cache; an accessibility parser, second semantic
  cache, or eager all-hidden-node pass is forbidden.
- Typed link/action, inline object alternative, and list/table structure remain under RRT-P1-040. They
  require a qualified semantic identity/action route backed by compiled-run or real UI-child ownership;
  synthetic byte-offset `UiNodeId` values are forbidden. Current status is
  `RRT-P1-039_visibility_independent_surface_semantic_owner_static_complete /
  RRT-P1-040_typed_children_and_managed_validation_pending`.

## 2026-08-30 Runtime Text rich list semantic ownership

- A list marker's rendered string or glyphs are geometry, not semantic identity. Consumers must not infer
  ordered/unordered kind, ordinal, marker style, or nesting level from visible prefix text.
- `RichListItem` is the compiled semantic owner. Ordered kind stores its ordinal and marker enum together;
  semantic level is one-based and independent of visual indentation. Ordinal advance must be checked and
  may not saturate into duplicate item identities.
- `ParagraphOverride` may carry the typed list item. UI layout derives only the exact marker range needed
  for hanging-indent measurement. Physical-paragraph overlap uses a private layout projection and must not
  rebuild, replace, or erase the semantic item.
- BBCode is the only list authoring surface covered by this slice. A marker-range-only DTO is permitted
  inside layout, but it must not escape as a second semantic model. HTML list support and full typed block
  tree require their own parser and product contracts.
- Accessibility publication remains blocked on the qualified semantic identity/action route. Byte offsets,
  ordinals, or marker ranges must not be encoded as synthetic `UiNodeId` values. Current status is
  `RRT-P1-037_typed_list_item_metadata_static_complete /
  RRT-P1-040_qualified_publication_and_managed_validation_pending`.

## 2026-08-30 Runtime Text inline object semantic fallback ownership

- Inline object replacement text is a compiled run concern. Accessibility, copy, render, and platform
  adapters must not parse markup, inspect image locators, or infer meaning from U+FFFC.
- Image alternative text distinguishes absent from explicitly empty. Explicit alt always wins; empty alt is
  decorative. Tooltip may be a secondary fallback only when alt is absent. The strings participate in the
  same retained metadata quota and cache residency accounting as their run.
- Semantic output has a dedicated request-local byte budget and typed terminal failure. It is built once by
  `CompiledRichText` from the ordered inline index. A source without inline objects shares the visible-text
  Arc; a source with inline objects owns one accounted semantic Arc. Per-snapshot run walking or a second
  semantic cache is forbidden without measured evidence.
- Every compiled inline range must be non-empty, ordered, UTF-8-valid, and contain only admitted replacement
  characters. Invalid ranges fail artifact publication; clamping, partial output, and raw-placeholder
  fallback are forbidden.
- Image semantic text does not create an accessibility child identity. Icon/widget alternatives, image
  resource outcome, and actions remain separate contracts. Current status is
  `RRT-P1-029_inline_image_semantic_fallback_static_complete /
  RRT-P1-040_qualified_inline_children_and_managed_validation_pending`.

## 2026-08-30 Runtime Interface reflected field admission ownership

- A reflection DTO constructor may collect authoring metadata, but only the Runtime `TypeRegistry`
  admits a complete field schema. Native, derived, dynamic-component, and VM publication paths must
  converge through that owner before registry or generation mutation.
- Declared value syntax has one bounded parser owner with explicit general-native and strict-VM
  policies. A second parser in an adapter, VM catalog, derive macro, or editor is forbidden. General
  policy may retain explicit heterogeneous native `List`/`Map`; strict VM policy requires typed
  containers and cannot inherit that permissiveness.
- Field and enum collection budgets are local admission policy and stay beside the registry validator.
  Parser byte/depth limits stay beside the declaration parser. Neither is promoted to a shared magic
  constants module without a second production owner using the same unit and rejection semantics.
- Schema validation is transactional and reports type/field context. Runtime instance writes use
  value mismatch errors; malformed defaults and metadata are registration errors. Adapters must not
  retain duplicate weak validation or a fallible projection after the central owner makes it
  infallible.
- VM schema preparation returns one validated descriptor for all atomic registry projections. World
  code must not reconstruct or reparse the same descriptor. Identical upsert is a no-op and must not
  invalidate catalog-bound plans or field caches.
- Derive inference for a homogeneous container preserves its element type (`Vec<T>` to `List<T>`).
  Unknown elements require an explicit declaration; degrading to an untyped container is forbidden.
  Dense vector position is not a stable field ID.
- Current status is `runtime_interface_reflect_field_admission_source_complete /
  managed_validation_pending`.

## 2026-08-30 Runtime Interface reflected value budget ownership

- `ReflectedValue` remains a runtime-neutral tagged DTO. RuntimeInterface owns only caller-supplied
  `ReflectValueBudget`, structured validation errors, and one traversal algorithm; product thresholds
  cannot be embedded in the neutral value type or copied into adapters.
- Runtime `scene::reflect::value_admission` is the single product-policy owner. Schema defaults,
  World reflection, world-query inspection, dynamic component admission, dynamic-scene capture/spawn,
  reflected JSON persistence, and VM reflected state/schema must reuse it before publication,
  mutation, or serialization.
- Value traversal must cover embedded `serde_json::Value`, map/object keys, cumulative UTF-8 bytes,
  per-container entries, total nodes, depth, and every floating component. Recursive adapter-local
  finite walkers and unchecked/saturating state counters are forbidden.
- The current validator uses one flat work stack: `O(nodes + string bytes)` time with no value/string
  clone and no dependence on the call stack. Container admission occurs before child references enter
  the worklist.
- A per-value budget is not an outer response or snapshot budget. Encoded bytes, total DTO items,
  processing time, paging/bulk lifetime, cancellation, and backpressure stay with their transport or
  persistence owner.
- Current status is `runtime_interface_reflect_value_budget_source_complete /
  managed_and_product_validation_pending`.

## 2026-08-30 Runtime Text rich-link target ownership

- A rich-link destination is admitted once into `UiRichLinkTarget`. Parser, compiled run, hit projection,
  input effect, transaction, and host request must carry that typed value; converting it back to a string
  before the serialization boundary is forbidden.
- The target's locator field remains private. Checked constructors and deserialization own canonical path,
  label, and `res/lib/package/builtin` policy. Input application validates node ownership but must not
  maintain a second scheme/path parser or allowlist.
- Repeated run/effect/request copies share canonical locator storage. Wire compatibility may retain the
  scalar `href` field, but that serialization name is not an invitation to restore a stringly Rust API.
- Link tooltip metadata belongs to the compiled run and is shared as `Arc<str>` through hit projection.
  It participates in decorator quota and compiled residency. The surface tooltip state owns overlay IDs
  and timers, so a rich-run string must not mutate that state before a qualified hover/action owner exists.
- Compiled residency accounting lives in `text/rich/compiled/memory.rs`; `compiled.rs` owns artifact
  construction/index orchestration and delegates the estimate. The current owners are 76 and 730 lines.
- Typed destination and tooltip are not a qualified semantic child or authorization principal. Link action
  kind, visited/disabled state, navigation policy, trust/principal, and accessibility identity remain
  explicit follow-up contracts. Current status is
  `RRT-P1-030_typed_target_and_tooltip_metadata_static_complete /
  RRT-P1-040_qualified_link_child_and_managed_validation_pending`.

## 2026-08-30 Runtime Text rich-icon asset ownership review

- A family-only icon glyph is not an admitted inline resource and has been removed from the production
  contract. `RichIconAssetId` is the strong image-icon identity; the run owns explicit size, baseline, and
  alternative/decorative text.
- Unreal-aligned rich icons use a style/icon asset that resolves to a brush/ImageRun before measure. Zircon's
  current primary route now publishes `RichTextDependency::IconAsset`, uses the same stored geometry in
  horizontal/VerticalRl layout and paint, and emits an image batch without renderer-local text shaping.
- A font-backed icon is a separate typed asset contract. It must name the font asset/face/glyph/fallback and
  capture the layout session's collection revision; canonical shaping/layout/glyph artifact and paint consume
  the same result. Renderer-local family lookup or reshape is forbidden.
- Resource generation/readiness, intrinsic metric resolution, missing/error fallback, and qualified semantic
  child identity remain separate owners; explicit authored size and fallback text do not close them. Authored-size
  icons reuse the shared UI image renderer's `ResourceManagementGeneration` cache and fallback texture. The compiled
  text artifact must not pin or duplicate that render generation; only a future intrinsic metric snapshot may bind
  layout invalidation to a qualified texture revision.
- Current status is `RRT-P1-028_typed_image_icon_asset_hard_cut_static_complete /
  intrinsic_metric_revision_readiness_font_icon_and_managed_validation_pending`.

## 2026-08-30 Runtime Text compiled dependency ownership

- `CompiledRichText` publishes resource requirements only through `RichTextDependency`. A raw
  `ResourceId` slice with an implied resource kind is forbidden because render collectors must not guess
  whether an id names a texture, font, icon asset, widget resource, or provider artifact.
- Dependency collection occurs once from canonical admitted runs before artifact publication. The retained
  slice is sorted and deduplicated; cache residency accounts the typed elements. Frame/render consumers read
  the slice without parsing markup or walking all runs.
- The current admitted variants are `ImageTexture(ResourceId)` and `IconAsset(RichIconAssetId)`. Texture/icon
  collection must explicitly match the kind. Adding generation, font, widget, or provider dependencies still
  requires a real lease identity and an owning lifecycle consumer.
- Font family strings, bare widget ids, and parser/decorator generation numbers are not resource leases and
  must not be inserted to make the closure appear complete. Current status is
  `RRT-P1-020_typed_image_and_icon_dependency_foundation_static_complete /
  generation_font_widget_decorator_lease_and_managed_validation_pending`.

## 2026-08-30 Runtime Text compiled cache telemetry ownership

- Compiled rich cache events belong to the same private cache owner that mutates the cache. A UI/frame
  adapter must not reconstruct intervals by subtracting process-lifetime cumulative counters.
- Snapshot and reset of hit, miss, parse, eviction, admission-bypass, and candidate-probe events occur while
  holding the cache mutex. Residency entries/bytes and configured bounds are gauges and survive the reset.
- The parser owner stamps parser identity plus decorator/emoji generations onto that snapshot. Checked
  overflow emits a saturation receipt; it must not freeze all later intervals at `u64::MAX`.
- Profile names remain fixed and low-cardinality. Markup, pointers, resource ids, and dynamic project/parser
  labels are forbidden. The outer Surface/profile session owns project/surface correlation.
- Current status is `RRT-P1-022_parser_provider_qualified_reset_snapshot_static_complete /
  project_surface_correlation_and_managed_profile_pending`.

## 2026-08-30 Runtime Text single-flight measurement boundary

- `CompiledRichTextCacheOwner` remains the only compiled-artifact admission and single-flight owner. Contention
  telemetry is recorded at its `OnceLock` call boundary; UI and profiler adapters do not infer waits from hits.
- `compile_requests_in_flight` is a point-in-time gauge. Completed non-initializer calls contribute interval
  wait count, total nanoseconds, and maximum nanoseconds; reset snapshots retain the gauge and clear the three
  interval fields.
- A call-local `Cell` marks the actual initializer. An RAII request guard decrements the gauge on return or
  unwind. Already-complete artifacts return before `Instant` acquisition.
- Removing single-flight, duplicating parse work, or adding a timeout without a bounded worker/cancellation
  owner is forbidden before the documented same-key contention and fault profile matrix is collected.
- Tests are folder-backed under `text/cache/rich_cache/tests.rs`; current production/tests/profile owners are
  541/340/739 lines. Status: `RRT-P1-014_contention_measurement_static_complete /
  bounded_worker_cancellation_and_managed_profile_pending`.

## 2026-08-30 Runtime Interface stable reflected field identity ownership

- Persistent field identity, current/display names, migration aliases, and dense execution slots are
  separate concepts. `ReflectFieldId` owns the non-nil 128-bit identity; a vector position must never be
  serialized or renamed as an ID, and runtime must not hash a current field name as a fallback.
- Native/script derive are the initial identity-key generation owners. Explicit keys must be non-empty
  and already trimmed; a field rename retains the old key. RuntimeInterface `ReflectSchemaCatalog` is the
  final identity, alias, path, dependency, and fingerprint admission authority.
- The neutral catalog is the only ID-to-slot index owner. Small admitted schemas use a sorted compact
  array, large schemas use one immutable hash index, and Runtime `TypeRegistry` mutates that catalog
  before its runtime-only adapter projection. Component/resource single-field writes consume dense slots
  after catalog ID lookup.
- Catalog entry, admission, field index, and fingerprint are separate leaf owners at 26, 336, 70, and
  369 lines. Runtime `type_registry.rs` is 689 lines after deleting its duplicate short/ambiguous/field
  indexes, below the 800-line production review warning; no compatibility module, re-export shim,
  production `allow(dead_code)`, or second identity owner was added.
- Current status is `RRI-P1-043_field_identity_slot_index_source_foundation_complete /
  public_dto_persistence_vm_managed_product_validation_pending`.

## 2026-08-30 Runtime Interface reflection catalog ownership

- A vector of `ReflectTypeRegistration` is not an admitted catalog. The neutral owner must retain full
  and short path indexes, explicit short-name ambiguity, global field-ID ownership, scoped legacy aliases,
  dependency closure/order, and one versioned registration-set fingerprint.
- Runtime adapters, ECS storage, function pointers, and world mutation stay outside RuntimeInterface.
  Runtime `TypeRegistry` contains the catalog and a runtime-only adapter projection; editor, remote schema,
  scene migration, and script must consume catalog products rather than build local name/redirect maps.
- Fingerprint input uses explicit domain/version/tags/lengths and an iterative value/JSON walker. It is
  lazy per catalog generation, so ordinary field access and incremental registration do not rehash the
  full schema. Snapshot decode must re-admit entries and verify fingerprint plus derived projections.
- Legacy aliases are scoped migration input only. Normal field access must not search aliases, hash current
  names into IDs, apply wildcard redirects, or tolerate dependency cycles/missing plugin schemas.
- Current status is `RRI-P1-045_catalog_and_runtime_projection_source_foundation_complete /
  generated_dependency_persistence_managed_product_validation_pending`.

## 2026-08-30 Runtime Text bidi content-policy boundary

- Rich parser/admission owns content trust, source-range diagnostics, and the eventual disposition of raw,
  entity-decoded, or markup-synthesized direction controls. Shaping/layout only consumes admitted logical
  Unicode and must not silently strip, replace, or inject isolation characters.
- Bidirectional mark, embedding/pop, override, and isolate are distinct diagnostic classes. All formats use
  one bounded diagnostic owner and one truncation receipt; HTML entity decode may observe source ranges in
  its existing loop but must not retain a second unbounded source map.
- Visual reordering is a projection over logical identity. Copy, hit testing, selection, accessibility, and
  paint must not each invent a direction-control policy.
- The trusted-authoring gate is typed and included in compiled/cache identity. Default untrusted content permits
  marks and balanced isolates, rejects legacy embeddings/overrides, and applies the same exact-range policy to raw
  scalars, entities, and control tags. Trusted authoring remains explicit and balance checked; warnings alone are
  never authorization.
- Current status is `RRT-P1-041_trust_gate_and_balanced_isolation_static_complete /
  managed_copy_a11y_render_and_profile_pending`.

## 2026-08-30 Runtime WGPU standalone UI owner boundary

- Standalone native UI no longer builds a raw device/queue context without an RHI owner. Shared and local
  contexts both carry one `Arc<WgpuRenderDevice>`; a typed `External/Local` value defines which frame entry
  may advance completion.
- Initial WGPU profile construction is owned by `device_profile.rs`; offscreen backend and standalone UI
  provide adapter facts, native device, and negotiation receipt instead of copying generation/limit/budget
  defaults.
- Native present recording, submission, completion collection, and result publication remain separate
  focused owners. The legacy staging queue may allocate/copy/map but production collection occurs only after
  the render-device owner poll; direct native polling is test-only.
- UI image in-flight pins are packet payload, not a second callback owner. The submission service retains
  them in a ticket-keyed bounded owner and releases them after its sole completion callback or generation
  terminalization, outside the submission-state lock.
- Current owner sizes are submission root 763, queued work 58, UI retirement 53, native recording 475,
  device root 798, fault terminal 51, UI root 715, surface setup 206, external-image copy 129,
  presentation 477, readback queue 749, and timeline tests 113 physical lines, all below the existing
  800-line review bound.
- Current status is `runtime90_standalone_ui_sui_0_through_sui_3_source_implemented_static_checks_passed_dynamic_validation_pending`.

## 2026-08-30 Runtime product raw queue parameter boundary

- Product preparation APIs may receive a raw WGPU queue only when they perform a still-owned native
  queue operation with an explicit deletion plan. Passing queue through resource/material preparation
  to an unused parameter is forbidden; frame uploads belong to the typed submission transaction.
- Immutable queue-derived device facts are captured once by `WgpuRenderDevice`. Product diagnostic
  helpers consume those facts or ticket-qualified deliveries and do not receive queue authority merely
  to call an accessor.
- Legacy test fixtures and standalone UI query-set construction remain named exceptions, not a generic
  queue facade. Shared UI query migration requires device-level consumer routing before deleting that
  exception.
- PFO-4d4b owner sizes are device 798, timer 615, scene resource prepare 726, material 945, direct frame
  459, and compiled frame submission owner 392 physical lines. Status:
  `runtime90_pfo_4d4b_source_implemented_static_checks_passed_dynamic_validation_pending`.

## 2026-08-31 Runtime Text rich paint projection publication boundary

> 请将产出记录放置在子计划中，此处仅展示当前现状的概述

Runtime Text paint projection, shared plain/rich command outcome, safe-empty-layout fail-closed routing,
and renderer batch ownership are statically implemented and remain
pending managed validation. Canonical evidence is maintained by the Text03 cardinality record and the
Text09 geometry/profile plan:

- `zircon_runtime/text/03/2026-08-31-rich-paint-run-cardinality-fail-closed.md`
- `zircon_runtime/text/09/2026-08-31-rich-paint-block-geometry-owner-and-profile-plan.md`
