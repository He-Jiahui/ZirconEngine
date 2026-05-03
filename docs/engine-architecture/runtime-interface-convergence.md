---
related_code:
  - Cargo.toml
  - zircon_module/src/lib.rs
  - zircon_module/src/engine_module.rs
  - zircon_module/src/engine_service.rs
  - zircon_app/src/lib.rs
  - zircon_app/src/entry/mod.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/entry_runner/bootstrap.rs
  - zircon_app/src/entry/tests/mod.rs
  - zircon_app/src/entry/tests/builtin_engine_entry.rs
  - zircon_app/src/runtime_presenter.rs
  - zircon_runtime/src/foundation/mod.rs
  - zircon_runtime/src/foundation/module.rs
  - zircon_runtime/src/foundation/runtime/config_manager.rs
  - zircon_runtime/src/foundation/runtime/event_manager.rs
  - zircon_runtime/src/engine_module/mod.rs
  - zircon_runtime/src/engine_module/tests.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/module.rs
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/scene/module.rs
  - zircon_runtime/src/scene/world/property_access/mod.rs
  - zircon_runtime/src/scene/world/property_access/entries.rs
  - zircon_runtime/src/scene/world/property_access/write.rs
  - zircon_plugins/Cargo.toml
  - zircon_plugins/animation/runtime/src/lib.rs
  - zircon_plugins/animation/runtime/src/module.rs
  - zircon_plugins/animation/runtime/src/manager.rs
  - zircon_plugins/animation/runtime/src/sequence.rs
  - zircon_plugins/animation/runtime/src/scene_hook.rs
  - zircon_plugins/navigation/runtime/src/lib.rs
  - zircon_plugins/net/runtime/src/lib.rs
  - zircon_plugins/particles/runtime/src/lib.rs
  - zircon_plugins/physics/runtime/src/lib.rs
  - zircon_plugins/physics/runtime/src/module.rs
  - zircon_plugins/physics/runtime/src/manager.rs
  - zircon_plugins/physics/runtime/src/query_contact.rs
  - zircon_plugins/physics/runtime/src/scene_hook.rs
  - zircon_plugins/sound/runtime/src/lib.rs
  - zircon_plugins/texture/runtime/src/lib.rs
  - zircon_scene/src/lib.rs
  - zircon_scene/src/semantics.rs
  - zircon_manager/src/lib.rs
  - zircon_asset/src/lib.rs
  - zircon_runtime/src/input/mod.rs
  - zircon_graphics/src/lib.rs
  - zircon_runtime/Cargo.toml
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/builtin/mod.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/graphics/host/mod.rs
  - zircon_runtime/src/graphics/host/module_host/mod.rs
  - zircon_runtime/src/graphics/host/module_host/module_registration/mod.rs
  - zircon_runtime/src/graphics/host/module_host/module_registration/module_descriptor.rs
  - zircon_runtime/src/graphics/host/module_host/module_registration/service_names.rs
  - zircon_runtime/src/graphics/host/module_host/create/create_render_framework.rs
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/ui/module.rs
  - zircon_runtime/src/ui/runtime_ui/mod.rs
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_fixture.rs
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_manager.rs
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_manager_error.rs
  - zircon_runtime/src/asset/tests/editor/manager.rs
  - zircon_runtime/src/scene/tests/component_structure.rs
  - zircon_runtime/src/scene/tests/property_paths.rs
  - zircon_runtime/src/ui/tests/asset.rs
  - zircon_runtime/src/script/mod.rs
  - zircon_runtime/src/script/vm/backend/backend_registry.rs
  - zircon_runtime/src/script/vm/host/constants.rs
  - zircon_runtime/src/script/vm/module/module_descriptor.rs
  - zircon_runtime/src/script/vm/plugin/vm_plugin_package_discovery.rs
  - zircon_runtime/src/script/vm/plugin/vm_plugin_package_source.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator.rs
  - zircon_runtime/src/script/vm/runtime/vm_plugin_slot_record.rs
  - zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs
  - zircon_runtime/src/script/vm/tests.rs
  - zircon_runtime/src/tests/mod.rs
  - zircon_scene/src/components/mod.rs
  - zircon_scene/src/components/scene.rs
  - zircon_framework/src/render/camera.rs
  - zircon_framework/src/render/scene_extract.rs
  - zircon_editor/src/scene/viewport/render_packet.rs
  - zircon_editor/src/lib.rs
  - zircon_runtime/src/platform.rs
  - zircon_plugins/sound/runtime/src/module.rs
  - zircon_plugins/texture/runtime/src/lib.rs
  - zircon_ui/src/lib.rs
  - zircon_plugins/net/runtime/src/module.rs
  - zircon_plugins/navigation/runtime/src/lib.rs
  - zircon_plugins/particles/runtime/src/lib.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/SKILL.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/references/interface-family.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/references/structural-audit.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
implementation_files:
  - Cargo.toml
  - zircon_module/src/lib.rs
  - zircon_module/src/engine_module.rs
  - zircon_module/src/engine_service.rs
  - zircon_app/src/lib.rs
  - zircon_app/src/entry/mod.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/entry_runner/bootstrap.rs
  - zircon_app/src/entry/tests/mod.rs
  - zircon_app/src/entry/tests/builtin_engine_entry.rs
  - zircon_app/src/runtime_presenter.rs
  - zircon_runtime/src/foundation/mod.rs
  - zircon_runtime/src/foundation/module.rs
  - zircon_runtime/src/foundation/runtime/config_manager.rs
  - zircon_runtime/src/foundation/runtime/event_manager.rs
  - zircon_runtime/src/engine_module/mod.rs
  - zircon_runtime/src/engine_module/tests.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/module.rs
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/scene/module.rs
  - zircon_runtime/src/scene/world/property_access/mod.rs
  - zircon_runtime/src/scene/world/property_access/entries.rs
  - zircon_runtime/src/scene/world/property_access/write.rs
  - zircon_plugins/Cargo.toml
  - zircon_plugins/animation/runtime/src/lib.rs
  - zircon_plugins/animation/runtime/src/module.rs
  - zircon_plugins/animation/runtime/src/manager.rs
  - zircon_plugins/animation/runtime/src/sequence.rs
  - zircon_plugins/animation/runtime/src/scene_hook.rs
  - zircon_plugins/navigation/runtime/src/lib.rs
  - zircon_plugins/net/runtime/src/lib.rs
  - zircon_plugins/particles/runtime/src/lib.rs
  - zircon_plugins/physics/runtime/src/lib.rs
  - zircon_plugins/physics/runtime/src/module.rs
  - zircon_plugins/physics/runtime/src/manager.rs
  - zircon_plugins/physics/runtime/src/query_contact.rs
  - zircon_plugins/physics/runtime/src/scene_hook.rs
  - zircon_plugins/sound/runtime/src/lib.rs
  - zircon_plugins/texture/runtime/src/lib.rs
  - zircon_scene/src/lib.rs
  - zircon_scene/src/semantics.rs
  - zircon_manager/src/lib.rs
  - zircon_asset/src/lib.rs
  - zircon_runtime/src/input/mod.rs
  - zircon_graphics/src/lib.rs
  - zircon_runtime/Cargo.toml
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/builtin/mod.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/graphics/host/mod.rs
  - zircon_runtime/src/graphics/host/module_host/mod.rs
  - zircon_runtime/src/graphics/host/module_host/module_registration/mod.rs
  - zircon_runtime/src/graphics/host/module_host/module_registration/module_descriptor.rs
  - zircon_runtime/src/graphics/host/module_host/module_registration/service_names.rs
  - zircon_runtime/src/graphics/host/module_host/create/create_render_framework.rs
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/ui/module.rs
  - zircon_runtime/src/asset/tests/editor/manager.rs
  - zircon_runtime/src/scene/tests/component_structure.rs
  - zircon_runtime/src/scene/tests/property_paths.rs
  - zircon_runtime/src/ui/tests/asset.rs
  - zircon_runtime/src/script/mod.rs
  - zircon_runtime/src/script/vm/backend/backend_registry.rs
  - zircon_runtime/src/script/vm/host/constants.rs
  - zircon_runtime/src/script/vm/module/module_descriptor.rs
  - zircon_runtime/src/script/vm/plugin/vm_plugin_package_discovery.rs
  - zircon_runtime/src/script/vm/plugin/vm_plugin_package_source.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator.rs
  - zircon_runtime/src/script/vm/runtime/vm_plugin_slot_record.rs
  - zircon_runtime/src/script/vm/runtime/vm_plugin_manager.rs
  - zircon_runtime/src/script/vm/tests.rs
  - zircon_runtime/src/tests/mod.rs
  - zircon_scene/src/components/mod.rs
  - zircon_scene/src/components/scene.rs
  - zircon_framework/src/render/camera.rs
  - zircon_framework/src/render/scene_extract.rs
  - zircon_editor/src/scene/viewport/render_packet.rs
  - zircon_editor/src/lib.rs
  - zircon_runtime/src/platform.rs
  - zircon_plugins/sound/runtime/src/module.rs
  - zircon_plugins/texture/runtime/src/lib.rs
  - zircon_ui/src/lib.rs
  - zircon_plugins/net/runtime/src/module.rs
  - zircon_plugins/navigation/runtime/src/lib.rs
  - zircon_plugins/particles/runtime/src/lib.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/SKILL.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/references/interface-family.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/references/structural-audit.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
plan_sources:
  - user: 2026-04-19 继续把更多模块压入zircon_runtime
  - user: 2026-04-20 请将physics和animation 吸收进runtime，并按最终架构直接切换不保留 shim
  - user: 2026-04-19 先根据Runtime吸收层 Editor吸收的规则迁移，外部目录干净化
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - user: 2026-04-18 首先抽象接口 IEntry IManager IDriver 等，然后把它设计为 skill，并分析项目是否结构化、是否需要重构
  - user: 2026-04-18 implement the runtime interface family and structural audit skill plan
  - .cursor/plans/基本路线图.md
  - .codex/plans/全系统重构方案.md
tests:
  - cargo test -p zircon_runtime --locked --offline --target-dir target/codex-shared-b optional_extension_module_registration_is_absorbed_into_runtime_extensions_surface -- --nocapture
  - cargo test -p zircon_runtime script_subsystem_is_physically_absorbed_into_runtime_crate --offline --target-dir target/codex-shared-b -- --nocapture
  - cargo test -p zircon_runtime graphics_module_host_is_absorbed_into_runtime_graphics_surface --offline --target-dir target/codex-shared-b -- --nocapture
  - cargo test -p zircon_runtime graphics_runtime_host_no_longer_owns_legacy_preview_or_render_service_wiring --offline --target-dir target/codex-shared-b -- --nocapture
  - cargo test -p zircon_runtime graphics_runtime_surface_re_exports_module_descriptor_and_owner_type --offline --target-dir target/codex-shared-b -- --nocapture
  - cargo test -p zircon_runtime ui_module_registration_is_absorbed_into_runtime_ui_surface --locked --offline --target-dir target/codex-shared-b -- --nocapture
  - cargo test -p zircon_runtime --locked --offline --target-dir target/codex-shared-b -- --nocapture
  - cargo check --manifest-path zircon_plugins/physics/runtime/Cargo.toml --tests --locked --target-dir target-codex-runtime-check
  - cargo check --manifest-path zircon_plugins/animation/runtime/Cargo.toml --tests --locked --target-dir target-codex-runtime-check
  - cargo test -p zircon_runtime optional_extension_module_registration_is_absorbed_into_runtime_extensions_surface --offline --target-dir target/codex-shared-b -- --nocapture
  - cargo check --workspace --locked --offline --target-dir target/codex-shared-b
  - cargo test -p zircon_asset --locked --offline --target-dir target/codex-shared-b asset_project_api_moves_under_project_module_namespace -- --nocapture
  - cargo test -p zircon_asset --locked --offline --target-dir target/codex-shared-b asset_watch_api_moves_under_watch_module_namespace -- --nocapture
  - cargo test -p zircon_ui --locked --offline --target-dir target/codex-shared-b legacy_template_compat_api_moves_under_template_namespace -- --nocapture
  - cargo test -p zircon_ui --locked --offline --target-dir target/codex-shared-b template_selector_api_moves_under_template_namespace -- --nocapture
  - cargo test -p zircon_ui --locked --offline --target-dir target/codex-shared-b template_binding_model_api_moves_under_template_namespace -- --nocapture
  - cargo test -p zircon_runtime --locked --offline --target-dir target/codex-shared-b runtime_asset_surface_keeps_project_and_watch_under_namespaces -- --nocapture
  - cargo test -p zircon_runtime --locked --offline --target-dir target/codex-shared-b runtime_ui_surface_keeps_template_and_layout_specialists_under_namespaces -- --nocapture
  - cargo test -p zircon_runtime --lib --offline
  - cargo test -p zircon_editor --lib --offline
  - cargo test -p zircon_asset --locked --offline --target-dir target/codex-shared-b -- --nocapture
  - cargo test -p zircon_ui --locked --offline --target-dir target/codex-shared-b -- --nocapture
  - cargo test -p zircon_scene --locked --offline --target-dir target/codex-shared-b -- --nocapture
  - cargo test -p zircon_graphics --lib --no-run --locked --offline --target-dir target/codex-shared-b
  - cargo test -p zircon_graphics render_framework_bridge --locked
  - cargo test -p zircon_editor --lib editor_manager_restores_ui_asset_tree_selection_across_undo_and_redo -- --nocapture
  - cargo test -p zircon_editor --lib editor_manager_promotes_selected_ui_asset_component_to_external_widget_asset -- --nocapture
  - cargo test -p zircon_editor --lib editor_manager_ui_asset_sessions_are_split_by_host_orchestration_behaviors -- --nocapture
  - cargo test -p zircon_editor --lib --no-run --locked --offline --target-dir target/codex-shared-b
  - cargo test -p zircon_editor --test native_window_hosts native_window_hosts_remain_empty_after_config_bootstrap -- --exact
  - cargo test -p zircon_runtime --locked engine_module --message-format short --target-dir target/codex-runtime-stub-cleanup -- --nocapture
  - cargo test -p zircon_runtime --locked --offline runtime_scene_property_reflection_stays_internal --message-format short --target-dir D:/cargo-targets/zircon-runtime-property-access-split -- --nocapture
  - cargo test -p zircon_runtime --locked --offline world_resolves_entity_paths_and_mutates_component_properties --message-format short --target-dir D:/cargo-targets/zircon-runtime-property-access-split -- --nocapture
  - cargo test -p zircon_runtime --locked --offline scene::tests --message-format short --target-dir D:/cargo-targets/zircon-runtime-property-access-split -- --nocapture
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - python C:/Users/HeJiahui/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence
doc_type: module-detail
---

# Runtime Interface Convergence

## Purpose

这份文档记录本轮“接口家族收敛”的落地结果：不是把现有 runtime 完全推翻，而是在保持 `zircon_runtime::core::runtime` descriptor 主干不变的前提下，把路线图里的 `IEntry`、`IModule`、`IService`、`IDriver`、`IManager`、`IPlugin`、`IObject`、`ISystem`、`IEntity`、`IComponent` 映射到一组实际可用的 Rust 合同，并把结构诊断沉淀为仓库内 skill。

## Interface Family

当前映射固定为：

- `IEntry -> zircon_app::EngineEntry`
- `IModule -> zircon_runtime::engine_module::EngineModule`
- `IService -> zircon_runtime::engine_module::EngineService`
- `IDriver -> zircon_runtime::engine_module::EngineDriver`
- `IManager -> zircon_runtime::engine_module::EngineManager`
- `IPlugin -> zircon_runtime::engine_module::EnginePlugin`
- `IObject -> zircon_runtime::scene::RuntimeObject`
- `ISystem -> zircon_runtime::scene::RuntimeSystem`
- `IEntity -> zircon_runtime::scene::semantics::EntityIdentity`
- `IComponent -> zircon_runtime::scene::semantics::ComponentData`

这里最关键的边界是：`EngineService` 是共享 runtime 元数据合同，不是所有具体 service 实例都必须继承的统一业务基类。具体能力仍由 `AssetManager`、`InputManager`、`RenderFramework`、`VmPluginManager` 这类 contract trait 或 handle surface 承担。

## Module Owner Convergence

本轮把内建 module crate 从“只有 `module_descriptor()` 自由函数”推进到了“显式模块拥有者类型 + 兼容自由函数”的状态。

- `zircon_app::BuiltinEngineEntry` 现在直接持有 `Arc<dyn EngineModule>` 集合，而不是再经过独立的 module-set 组合类型缓存 `ModuleDescriptor`
  - `zircon_runtime`
  - 当前已稳定导出 `FoundationModule` / `PlatformModule` / `InputModule` / `ScriptModule` / `UiModule` / `AssetModule` / `SceneModule`；脚本 VM 实现目录位于 `zircon_runtime/src/script/`，UI/asset/scene module-registration surface 分别位于 `zircon_runtime/src/ui/`、`zircon_runtime/src/asset/`、`zircon_runtime/src/scene/`
  - `zircon_runtime::graphics` 现在已经重新成为 graphics module owner：`GraphicsModule` 保留在 runtime graphics surface，`zircon_runtime::builtin_runtime_modules()` 也直接把 graphics 放在 asset 之后、scene 之前；`zircon_app` 不再手工维护这段 runtime module 顺序
  - `physics` 与 `animation` 已经硬切到 `zircon_plugins/{physics,animation}/runtime`，由插件 module owner、manager-backed runtime path 和 scene hook 直接承载；`zircon_runtime` 只保留 framework contract、manager resolver/service name、scene ECS authority 和 hook dispatcher
  - 旧可选扩展 module-owner paths 不再作为 runtime 本体实现入口；`builtin_runtime_modules()` 不再通过 legacy crate root 间接取得可选服务实现
- `zircon_asset`
  - 当前退回到 asset domain 和 editor-asset protocol crate；`AssetManager` / `resolve_asset_manager` 继续挂在 `zircon_asset::pipeline::manager::*`，editor asset records / resolver / handle 挂在 `zircon_asset::editor::*`，但 root 不再拥有 `AssetModule`、`module_descriptor()` 或根级 module-registration service names
  - `ProjectManager` / `ProjectManifest` / `ProjectPaths` / `AssetMetaDocument` / `PreviewState` 现已显式收口到 `zircon_asset::project::*`，`AssetChange` / `AssetChangeKind` / `AssetWatchEvent` / `AssetWatcher` 则收口到 `zircon_asset::watch::*`
  - asset root 只保留 asset-named alias 和结构性 namespace 入口，不再继续平铺 asset/editor/importer/pipeline/project/watch 子域 surface
- `zircon_scene`
  - 当前退回到 runtime world / scene-domain implementation crate；root 继续暴露 `components` / `semantics` / `serializer` / `world` 这四个 namespace 与 `Scene` 别名，但 `LevelSystem`、`DefaultLevelManager`、`SceneModule`、`module_descriptor()`、`create_default_level()` 与 `load_level_asset()` 都已经收进 `zircon_runtime::scene`
- `zircon_editor`
  - 继续保留 `EditorModule` 作为 editor host owner，并承接作者态 scene/ui 逻辑
- `zircon_graphics`
  - graphics 行为与 renderer implementation 继续留在 runtime-owned graphics 子树里；`zircon_app` 的 bootstrap 已不再知道 `GraphicsModule` 的具体接线顺序，入口侧对 graphics owner 的知识已经被 runtime 吸收
- `zircon_ui`
  - 当前已退回到共享 UI 实现与 DTO crate；`UiModule`、`UiConfig`、`UI_MODULE_NAME` 与 `module_descriptor()` 不再由它持有，crate 根继续暴露的是 layout/tree/template/binding/event-ui 这些实现和数据表面
  - 历史 template fixture conversion 只保留在 runtime/editor test support；production template/runtime 链路只接受 tree-shaped `UiAssetDocument`
  - template compiler 链现已明确挂在 `zircon_ui::template::{UiCompiledDocument, UiDocumentCompiler, UiStyleResolver}`，layout solver 链则挂在 `zircon_ui::layout::{compute_layout_tree, compute_virtual_list_window, solve_axis_constraints}`；crate 根不再继续平铺这些特化 surface
  - template asset component-schema / reflection 链现也明确挂在 `zircon_ui::template::{UiComponentDefinition, UiComponentParamSchema, UiNamedSlotSchema, UiStyleScope}`；`zircon_runtime::ui` 同步保持 namespace-first，不再把这组 asset-side schema surface 重新拍平到吸收层根入口
  - template asset selector/parser DTO 链已经转为 `zircon_runtime_interface::ui::template::{UiSelector, UiSelectorToken}` 的 neutral contract；runtime stylesheet 解析、component-contract validation 与 selector path matching 直接消费 interface DTO，`zircon_runtime::ui::template` 不再重新导出这组 parser model
  - template document/binding model 也已明确挂在 `zircon_ui::template::{UiActionRef, UiBindingRef, UiComponentTemplate, UiSlotTemplate}`；`zircon_editor` 的 binding inspector / document diff / template adapter 与 `zircon_ui` 内部 tree/asset model 都已切到 `template::*` 路径，`zircon_runtime::ui` 也不再把这组 template model 重新拍平到根入口
  - binding 协议链现已明确挂在 `zircon_ui::binding::{UiBindingValue, UiBindingCall, UiEventKind, UiEventPath, UiEventBinding, UiEventRouter}`，event/reflection/invocation 协议链则挂在 `zircon_ui::event_ui::{UiControlRequest, UiControlResponse, UiReflectionSnapshot, UiNodeDescriptor, UiPropertyDescriptor, UiRouteId, UiTreeId, UiEventManager}`；crate 根不再继续平铺这两簇 DTO
- `zircon_manager`
  - 当前收敛为 support/contract crate，只保留 trait、resolver、handle 和稳定服务名，不再承载 `EngineModule` owner 或 config/event 具体实现

这样做的目的不是马上删掉所有自由函数，而是先让“模块拥有者”成为明确公共概念，后续其它入口、审计脚本、架构规则都可以围绕它收敛。

同时，runtime built-in module 清单已经继续收束进 `zircon_runtime::builtin_runtime_modules()`。`zircon_app` 只保留 profile 选择与 editor 附加模块接线，不再自己拥有 `feature_modules` 这类 runtime 扩展组合知识。

## Entry Convergence

`zircon_app` 当前增加了三层明确合同：

- `EntryRunMode`
  - 明确 editor/runtime/headless 是入口运行模式，而不是散落在 `EntryRunner` 里的分支语义
- `EngineEntry`
  - 负责 profile、run mode、module owner 集合、bootstrap
- `BuiltinEngineEntry`
  - 当前默认实现，沿用原来的模块集合与 bootstrap 行为

`EntryRunner::bootstrap` 现在只是委托到 `BuiltinEngineEntry`，这意味着后续如果需要更换 profile 组合、拆 editor/runtime boot host、或引入更严格的入口配置，不需要继续把知识堆进一个薄 runner。

当前的 profile 组合改成两段职责：`zircon_runtime::builtin_runtime_modules()` 负责收口内建 runtime、graphics 与扩展模块清单，并保持 `asset -> graphics -> scene` 这段 runtime-owned 顺序；`zircon_app::entry::builtin_modules` 只在 editor profile 下附加 `zircon_editor::EditorModule`。这样入口 crate 不再保留旧的模块集合子树，也不再直接依赖 physics/sound/texture/net/navigation/particles/animation 这些扩展 crate，连 graphics 的模块插位知识也一起收回 runtime。

`src/entry/tests/mod.rs` 也跟着切到新的 `builtin_modules.rs` 树形结构；结构断言现在审计的是 `entry_runner/`、`builtin_modules.rs`、`runtime_entry_app/` 与 runtime presenter，而不是已经删除的旧模块集合目录。

## Tree-Backed Follow-Through

2026-04-18 的七目标结构化重构继续把这些合同落到目录树上，而不再停留在概念层：

- `zircon_app/src/entry/`
  - `entry_runner/`、`builtin_modules.rs`、`runtime_entry_app/`、`tests/` 已分离，`BuiltinEngineEntry` 直接持有 profile 对应的 module owner 集合
  - runtime built-in module 清单从 app 侧移回 `zircon_runtime/src/builtin/runtime_modules.rs`，而 `zircon_runtime/src/builtin/mod.rs` 只保留结构性窄导出；`zircon_app` 只负责 editor profile 附加模块与 bootstrap
  - `tests/mod.rs` 现在直接锚定新的 `builtin_modules.rs`、`lib.rs`、`runtime_presenter.rs` 与 `runtime_entry_app/*`，避免结构测试继续引用已经删除的旧模块集合路径
- `zircon_runtime/src/script/vm/`
  - 原 `zircon_script` VM 子树已整体并入 runtime，standalone `zircon_script` workspace member 与 Cargo package 已删除；目录继续分成 `module/`、`backend/`、`host/`、`plugin/`、`runtime/`
  - `ScriptModule` 现在作为 `zircon_runtime::script::ScriptModule` 暴露，并通过 `PluginDescriptor` 注册 `ScriptModule.Plugin.VmPluginRuntime`；`VmPluginManager` manager contract 继续依赖并复用该 plugin 实例，`resolve_plugin` 路径已接通
  - `backend/backend_registry.rs`、`plugin/vm_plugin_package_discovery.rs`、`plugin/vm_plugin_package_source.rs`、`runtime/vm_plugin_slot_record.rs` 现已把 package discovery、backend selection 和 slot lifecycle 拉进真实代码路径；当前 plugin runtime 不再只是一个“能 resolve 到 manager contract 的空壳”
- `zircon_runtime/src/graphics/`
  - runtime tree 现在重新包含 `src/graphics/mod.rs + host/module_host/**`，boundary tests 也重新以这组路径作为 graphics module-host owner surface 的验收目标
  - 这次收束先修复了 owner-tree 漂移和外部目录净化回归，随后又把 `GraphicsModule` 的 builtin registration 顺序一起收回 `zircon_runtime::builtin_runtime_modules()`；入口侧不再保留 graphics-side helper path 或手工 `insert(...)` 逻辑
  - graphics host 这轮又补了一条导出边界修正：`host/module_host/mod.rs` 现在把 `module_descriptor` 从其他 `module_registration` 符号里拆成单独一条 `pub use`，避免 grouped re-export 在继续收窄 public surface 时重新撞上 `module_registration` 内部的私有同名子模块
- `zircon_runtime/src/ui/`
  - 原先 `zircon_runtime/src/ui.rs -> pub use zircon_ui::*` 的 shim 已改成 folder-backed `mod.rs + module.rs`；`UiModule`、`UiConfig`、`UI_MODULE_NAME` 与 `module_descriptor()` 现在都由 runtime 侧持有
  - `runtime_ui/` 子树现在也由 runtime 侧持有 `RuntimeUiFixture`、`RuntimeUiManager`、`RuntimeUiManagerError` 与四个 builtin fixture 资产；runtime UI host helper 不再由 graphics 子树代持
  - `zircon_runtime::ui` 现在保留 `layout` / `surface` / `template` / `tree` / `binding` / `event_ui` 作为显式 namespace，而不再重新平铺 `UiDocumentCompiler` / `UiTemplateSurfaceBuilder` / `UiTemplateInstance` / `UiTemplateNode` / `UiActionRef` / `UiBindingRef` / `UiComponentTemplate` / `UiSlotTemplate`、binding/event DTO、layout solver、`UiRenderExtract` / `UiRenderCommandKind` 或 `UiTreeError` / `UiTemplateNodeMetadata` 这类子域特化 surface
  - `zircon_runtime::ui` 保留共享实现 surface，template/layout/surface/tree/binding/event-ui 子域不经 runtime root 重新拍平
- `zircon_runtime/src/asset/`
  - `src/asset/mod.rs + module.rs` 现在持有 `AssetModule`、`ASSET_MODULE_NAME`、`ASSET_MANAGER_NAME`、`RESOURCE_MANAGER_NAME`、`PROJECT_ASSET_MANAGER_NAME`、`EDITOR_ASSET_MANAGER_NAME` 与 `module_descriptor()`；`zircon_asset` root 不再代持 module owner 或 registration helper，只保留 asset/editor-asset API
  - `zircon_runtime::asset` 现在保留 `project` / `watch` 作为显式 namespace，而不再重新平铺 `ProjectManager`、`AssetWatcher` 一类子域特化 surface
- `zircon_runtime/src/scene/`
  - `src/scene/mod.rs + module.rs` 现在持有 `SceneModule`、`SCENE_MODULE_NAME`、`DEFAULT_LEVEL_MANAGER_NAME`、`LEVEL_MANAGER_NAME`、`create_default_level()`、`load_level_asset()` 与 `module_descriptor()`；`zircon_scene` root 退回 runtime world / scene domain surface，不再根级公开 module-registration helper
- `zircon_plugins/`
  - `zircon_plugins/<plugin>/runtime/src/lib.rs` 现在成为外置可选扩展注册面的统一 owner；`sound`、`net`、`navigation`、`particles`、`texture` 在各自 plugin crate 内持有对应 module/manager/config/service-name contracts 或最小 manager-backed activation point
  - `physics` 与 `animation` 重新属于 plugin workspace；对应 concrete manager、fallback behavior、sequence writeback 和 scene hook 已收束到 `zircon_plugins/{physics,animation}/runtime`
  - 旧 legacy extension crate root 不再持有 `*Module` 或 `module_descriptor()`，runtime built-in 清单不再绕回旧扩展 crate root
- `zircon_editor` 热点链路
  - `host/template_runtime/`、`host/manager/ui_asset_sessions/`、`editing/ui_asset/` 已改成 folder-backed 子树；其中 `ui_asset_sessions/mod.rs` 现已退回接线层，host-side 编辑命令入口挪到 `editing.rs`
- `zircon_scene/src/components/`
  - 当前只保留 `scene`、`schedule` 与 scene-domain `Mobility` re-export，避免继续让 scene crate 滞留 framework/editor-owned viewport packet、overlay 与 gizmo contract

## ECS Semantics

`zircon_scene` 本轮只补了语义合同，不引入新的对象层级：

- `RuntimeObject`
  - 表达“这是运行时对象角色”
- `RuntimeSystem`
  - 表达“这是系统级运行时对象”
- `EntityIdentity`
  - 表达 ECS entity 的身份语义
- `ComponentData`
  - 表达 ECS component 的数据语义

这组合同的边界非常刻意：`Entity` 和 `Component` 仍然是 ECS 语义，不是面向对象继承入口。这样可以把路线图里的对象术语保留下来，同时避免把 `zircon_scene` 再退回 scene-node OO 模型。

## Structural Audit Skill

本轮新增仓库内 skill：

- `.codex/skills/zircon-project-skills/zr-runtime-interface-convergence/SKILL.md`
- `references/interface-family.md`
- `references/structural-audit.md`
- `scripts/audit_runtime_structure.py`

这个 skill 的目标不是生成更多抽象名词，而是强制执行一个固定流程：

1. 先读路线图
2. 再把 `I*` 术语映射到当前 Rust 合同
3. 再跑审计脚本
4. 最后才决定某个 crate 是 `converged`、`skeleton` 还是 `needs-refactor`

当前脚本会输出：

- `module_descriptor()` 分布
- `stub_module_descriptor` 使用点
- `EngineModule` owner 覆盖情况
- plugin runtime gaps
- `zircon_app` 的静态依赖扇出
- 生产代码大文件热点
- module crate 分类结果

最新一轮又把一个误导性的诊断源收掉了：`stub_module_descriptor`、`stub_driver_descriptor`、`stub_manager_descriptor`、`stub_plugin_descriptor` 与对应 stub service object 已经从 `zircon_runtime` 的生产目录移出，只保留在 [tests.rs](/E:/Git/ZirconEngine/zircon_runtime/src/engine_module/tests.rs) 里的本地 test helper。结构审计里的 “Stub Module Descriptor Usage” 现在为空，`zircon_runtime` 不再因为测试辅助 builder 仍驻留生产 surface 而被判成 `skeleton`。

## Current Diagnosis

## 2026-04-19 Cleanup Snapshot

这一轮按 `Runtime 吸收层与 Editor_Scene 边界收束计划` 回到了“先 owner cutover、再做上层功能”的顺序，重点不是继续堆功能，而是把 runtime/editor 吸收后的外部目录残留清干净。

- `zircon_runtime/src/asset/tests/editor/manager.rs`、`zircon_runtime/src/scene/tests/component_structure.rs`、`zircon_runtime/src/ui/tests/asset.rs` 已修正到吸收后的真实 folder-backed 根路径，避免边界测试继续锚到旧 `zircon_asset` / `zircon_scene` / `zircon_ui` 目录
- `zircon_runtime/src/ui/mod.rs` 继续保持 namespace-first surface，并把 runtime UI host 的 public re-export 文本重新对齐到 `RuntimeUiFixture / RuntimeUiManager / RuntimeUiManagerError`
- 代码级扫描已经找不到新的 `zircon_asset::`、`zircon_scene::`、`zircon_ui::` 实际消费点；残留主要在 `docs/` 的历史描述，而不是编译链路
- `cargo test -p zircon_runtime --lib --offline` 当前结果是 `175 passed / 6 failed`；剩余失败全部集中在 animation binary asset 的 bincode 解析，不再是 Runtime/Editor 吸收边界回归
- `cargo test -p zircon_editor --lib --offline` 当前结果是 `596 passed / 0 failed`，说明 editor 侧已经稳定消费 absorbed runtime asset/scene/ui surface

真正还没收口完的是 graphics public owner cutover：

- runtime 目录中的 graphics owner tree 已恢复并通过 boundary tests
- `zircon_app/src/entry/builtin_modules.rs` 已经不再手工插入 `GraphicsModule`；`asset -> graphics -> scene` 这段 builtin module 顺序现在完全由 `zircon_runtime::builtin_runtime_modules()` 持有
- 当前继续需要观察的是更深层 graphics 内部 helper/public surface 是否还有历史泄漏，但入口侧 owner cutover 本身已经闭环

## Current Diagnosis

按当前审计规则，仓库状态不是“完全未结构化”，而是“descriptor 主干已成型，但仍未完全收敛”。

已经相对收敛的部分：

- `zircon_runtime::core::runtime`
  - descriptor/lifecycle/runtime spine 已成立
- `zircon_runtime::engine_module`
  - 现在拥有明确的接口家族合同
- `zircon_app`
  - 现在拥有显式 `EngineEntry`
- `zircon_runtime`
  - 当前已经成为主 runtime module-owner surface；除 foundation/input/platform/script/graphics-host/ui-registration 之外，asset/scene module-registration 与 optional extension registration 也已继续收口到 runtime
  - `engine_module` 的 stub descriptor helper 现在只存在于单元测试本地 helper，不再驻留生产目录或 crate root test re-export；当前 runtime 结构审计里剩余的主因已经收敛到大型生产文件热点，而不是假 owner/stub module wiring
- `zircon_runtime::asset`
  - 当前持有 asset domain、asset module registration、project/watch/importer/pipeline 子域和 manager-facing protocol
- `zircon_runtime::scene`
  - 当前持有 runtime world、level implementation、serialization authority 和 scene module registration
- `zircon_runtime::graphics`
  - 当前持有 renderer implementation、graphics module owner surface 和 runtime-facing graphics contracts
- `zircon_runtime::core::manager`
  - 当前不再被误判成模块实现层，而是明确退回到 manager contract support namespace

仍然明确未收敛的部分：

- 可选 runtime plugin crates 已完成 registration-owner 迁移，但不能被误判为领域功能终点
  - `navigation`、`particles`、`texture` 已有最小 manager-backed activation point；后续仍需要真实寻路、粒子模拟与纹理导入/运行时能力，而不是再回到 legacy crate root
- `zircon_plugins::{physics,animation}`
  - 当前已经完成实现所有权吸收与 runtime-facing interface 收束，但服务实现仍然只是基线 stub；真实物理 backend、动画轨道/图/状态机栈仍是后续功能层工作，不应误判成领域功能已经完成
- `zircon_runtime::script` 现在已经收口到真实 plugin-runtime contract：`PluginDescriptor` 通过 `PluginFactory` 显式接收 `PluginContext`，`VmPluginManager` 保存 base plugin context，load/hot-reload 路径会派生 `VmPluginHostContext`，backend 选择也已经从实例 map 收束到 `VmBackendFamily` registry；当前剩余限制只在 backend breadth，内建仍是 `builtin:{mock,unavailable}` 基线
- `zircon_editor` 的 `ui_asset` 会话和 host manager 子树仍然存在明显结构热点
- `zircon_app` 的 production 静态依赖扇出已经明显下降，但 runtime app 和 editor host 仍然直接持有 `scene/input/render_server/editor` 等真运行时依赖；它还不是完全无扇出的纯 profile shell
- 路线图中的 `zircon_server` 与现有 `zircon_framework` 仍然有命名/层次漂移

## Next Convergence Targets

从当前状态继续推进时，优先顺序应保持：

1. 继续收口当前审计仍然点名的大型生产文件热点；`scene::world::property_access` 已经切成 folder-backed subtree，下一刀应优先继续压缩 [write.rs](/E:/Git/ZirconEngine/zircon_runtime/src/scene/world/property_access/write.rs) 这一类剩余 runtime world owner 热点，而不是再往 root surface 堆结构性 helper
2. 继续降低 `zircon_app` 的剩余直接 runtime/editor host 依赖，避免它重新长回全模块静态组合器
3. 在已经收口的 `zircon_runtime::script` 合同之上继续补真实 backend family 成员，而不是再回到 `CoreHandle` 反查或 backend 实例 map；并继续把 `zircon_graphics` 里剩余高层 runtime-facing surface 下沉到 `zircon_runtime::graphics`
4. 继续拆解 `zircon_editor` 的热点边界，尤其是审计仍然点名的大型 `ui_asset`/animation/slint host 文件
5. 专门处理 `zircon_server` 与 `zircon_framework` 的层次和命名收敛

不要跳过这些步骤，直接宣布“接口体系已经完成”。当前完成的是抽象骨架和审计能力，不是全仓最终重构终点。

