---
related_code:
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
  - zircon_foundation/src/lib.rs
  - zircon_foundation/src/module.rs
  - zircon_foundation/src/runtime/config_manager.rs
  - zircon_foundation/src/runtime/event_manager.rs
  - zircon_scene/src/lib.rs
  - zircon_scene/src/semantics.rs
  - zircon_manager/src/lib.rs
  - zircon_asset/src/lib.rs
  - zircon_input/src/lib.rs
  - zircon_graphics/src/lib.rs
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/builtin.rs
  - zircon_runtime/src/tests.rs
  - zircon_script/src/lib.rs
  - zircon_script/src/vm/backend/backend_registry.rs
  - zircon_script/src/vm/host/constants.rs
  - zircon_script/src/vm/module/module_descriptor.rs
  - zircon_script/src/vm/plugin/vm_plugin_package_discovery.rs
  - zircon_script/src/vm/plugin/vm_plugin_package_source.rs
  - zircon_script/src/vm/runtime/hot_reload_coordinator.rs
  - zircon_script/src/vm/runtime/vm_plugin_slot_record.rs
  - zircon_script/src/vm/runtime/vm_plugin_manager.rs
  - zircon_script/src/vm/tests.rs
  - zircon_scene/src/components/mod.rs
  - zircon_scene/src/components/scene.rs
  - zircon_scene/src/components/viewport.rs
  - zircon_scene/src/components/render_extract.rs
  - zircon_scene/src/components/gizmo.rs
  - zircon_editor/src/lib.rs
  - zircon_animation/src/lib.rs
  - zircon_platform/src/lib.rs
  - zircon_physics/src/lib.rs
  - zircon_sound/src/lib.rs
  - zircon_texture/src/lib.rs
  - zircon_ui/src/lib.rs
  - zircon_net/src/lib.rs
  - zircon_navigation/src/lib.rs
  - zircon_particles/src/lib.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/SKILL.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/references/interface-family.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/references/structural-audit.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
implementation_files:
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
  - zircon_foundation/src/lib.rs
  - zircon_foundation/src/module.rs
  - zircon_foundation/src/runtime/config_manager.rs
  - zircon_foundation/src/runtime/event_manager.rs
  - zircon_scene/src/lib.rs
  - zircon_scene/src/semantics.rs
  - zircon_manager/src/lib.rs
  - zircon_asset/src/lib.rs
  - zircon_input/src/lib.rs
  - zircon_graphics/src/lib.rs
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/builtin.rs
  - zircon_runtime/src/tests.rs
  - zircon_script/src/lib.rs
  - zircon_script/src/vm/backend/backend_registry.rs
  - zircon_script/src/vm/host/constants.rs
  - zircon_script/src/vm/module/module_descriptor.rs
  - zircon_script/src/vm/plugin/vm_plugin_package_discovery.rs
  - zircon_script/src/vm/plugin/vm_plugin_package_source.rs
  - zircon_script/src/vm/runtime/hot_reload_coordinator.rs
  - zircon_script/src/vm/runtime/vm_plugin_slot_record.rs
  - zircon_script/src/vm/runtime/vm_plugin_manager.rs
  - zircon_script/src/vm/tests.rs
  - zircon_scene/src/components/mod.rs
  - zircon_scene/src/components/scene.rs
  - zircon_scene/src/components/viewport.rs
  - zircon_scene/src/components/render_extract.rs
  - zircon_scene/src/components/gizmo.rs
  - zircon_editor/src/lib.rs
  - zircon_animation/src/lib.rs
  - zircon_platform/src/lib.rs
  - zircon_physics/src/lib.rs
  - zircon_sound/src/lib.rs
  - zircon_texture/src/lib.rs
  - zircon_ui/src/lib.rs
  - zircon_net/src/lib.rs
  - zircon_navigation/src/lib.rs
  - zircon_particles/src/lib.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/SKILL.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/references/interface-family.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/references/structural-audit.md
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
plan_sources:
  - user: 2026-04-18 首先抽象接口 IEntry IManager IDriver 等，然后把它设计为 skill，并分析项目是否结构化、是否需要重构
  - user: 2026-04-18 implement the runtime interface family and structural audit skill plan
  - .cursor/plans/基本路线图.md
  - .codex/plans/全系统重构方案.md
tests:
  - cargo check -p zircon_app --locked
  - cargo test -p zircon_runtime --lib --locked
  - cargo test -p zircon_foundation --lib
  - cargo check -p zircon_script --lib
  - cargo test -p zircon_script --lib hot_reload_coordinator_tracks_slot_lifecycle_records -- --nocapture
  - cargo test -p zircon_script --lib vm_plugin_manager_discovers_packages_selects_backends_and_loads_slots -- --nocapture
  - cargo test -p zircon_script --lib core_resolve_plugin_exposes_vm_plugin_runtime_and_manager_facade_shares_it -- --nocapture
  - cargo test -p zircon_app builtin_engine_entry_reports_run_mode_and_owned_modules --locked
  - cargo test -p zircon_app entry_subsystem_is_split_into_builtin_modules_run_modes_and_runtime_app_tree --locked
  - cargo test -p zircon_graphics render_framework_bridge --locked
  - cargo test -p zircon_editor --lib editor_manager_restores_ui_asset_tree_selection_across_undo_and_redo -- --nocapture
  - cargo test -p zircon_editor --lib editor_manager_promotes_selected_ui_asset_component_to_external_widget_asset -- --nocapture
  - cargo test -p zircon_editor --lib editor_manager_ui_asset_sessions_are_split_by_host_orchestration_behaviors -- --nocapture
  - cargo test -p zircon_editor --test native_window_hosts native_window_hosts_remain_empty_after_config_bootstrap -- --exact
  - python .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - python C:/Users/HeJiahui/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence
doc_type: module-detail
---

# Runtime Interface Convergence

## Purpose

这份文档记录本轮“接口家族收敛”的落地结果：不是把现有 runtime 完全推翻，而是在保持 `zircon_core` descriptor 主干不变的前提下，把路线图里的 `IEntry`、`IModule`、`IService`、`IDriver`、`IManager`、`IPlugin`、`IObject`、`ISystem`、`IEntity`、`IComponent` 映射到一组实际可用的 Rust 合同，并把结构诊断沉淀为仓库内 skill。

## Interface Family

当前映射固定为：

- `IEntry -> zircon_app::EngineEntry`
- `IModule -> zircon_module::EngineModule`
- `IService -> zircon_module::EngineService`
- `IDriver -> zircon_module::EngineDriver`
- `IManager -> zircon_module::EngineManager`
- `IPlugin -> zircon_module::EnginePlugin`
- `IObject -> zircon_scene::RuntimeObject`
- `ISystem -> zircon_scene::RuntimeSystem`
- `IEntity -> zircon_scene::EntityIdentity`
- `IComponent -> zircon_scene::ComponentData`

这里最关键的边界是：`EngineService` 是共享 runtime 元数据合同，不是所有具体 service 实例都必须继承的统一业务基类。具体能力仍由 `AssetManager`、`InputManager`、`RenderFramework`、`VmPluginManager` 这类 façade trait 或 handle surface 承担。

## Module Owner Convergence

本轮把内建 module crate 从“只有 `module_descriptor()` 自由函数”推进到了“显式模块拥有者类型 + 兼容自由函数”的状态。

- `zircon_app::BuiltinEngineEntry` 现在直接持有 `Arc<dyn EngineModule>` 集合，而不是再经过独立的 module-set 组合类型缓存 `ModuleDescriptor`
- `zircon_animation`、`zircon_platform`、`zircon_physics`、`zircon_sound`、`zircon_texture`、`zircon_ui`、`zircon_net`、`zircon_navigation`、`zircon_particles`
  - 当前都已导出真实 module owner type；其中 animation/platform/physics/sound/texture/net/navigation/particles 已改成显式 `DriverDescriptor` / `ManagerDescriptor` no-op scaffold，不再继续依赖 `stub_module_descriptor`
  - 这些 crate 仍然属于 `skeleton`，因为服务对象还是空行为壳，不应误判成 `converged`
- `zircon_foundation`、`zircon_asset`、`zircon_input`、`zircon_graphics`、`zircon_scene`、`zircon_script`、`zircon_editor`
  - 当前都已导出真实 module owner type，并保留既有 descriptor 主干
- `zircon_manager`
  - 当前收敛为 support/facade crate，只保留 trait、resolver、handle 和稳定服务名，不再承载 `EngineModule` owner 或 config/event 具体实现

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

当前的 profile 组合改成两段职责：`zircon_runtime::builtin_runtime_modules()` 负责收口内建 runtime 与扩展模块清单，`zircon_app::entry::builtin_modules` 只在 editor profile 下附加 `zircon_editor::EditorModule`。这样入口 crate 不再保留 `module_set/*` 子树，也不再直接依赖 physics/sound/texture/net/navigation/particles/animation 这些扩展 crate。

`src/entry/tests/mod.rs` 也跟着切到新的 `builtin_modules.rs` 树形结构；结构断言现在审计的是 `entry_runner/`、`builtin_modules.rs`、`runtime_entry_app/` 与 runtime presenter，而不是已经删除的 `module_set/` 目录。

## Tree-Backed Follow-Through

2026-04-18 的七目标结构化重构继续把这些合同落到目录树上，而不再停留在概念层：

- `zircon_app/src/entry/`
  - `entry_runner/`、`builtin_modules.rs`、`runtime_entry_app/`、`tests/` 已分离，`BuiltinEngineEntry` 直接持有 profile 对应的 module owner 集合
  - runtime built-in module 清单从 app 侧移回 `zircon_runtime/src/builtin.rs`，`zircon_app` 只负责 editor profile 附加模块与 bootstrap
  - `tests/mod.rs` 现在直接锚定新的 `builtin_modules.rs`、`lib.rs`、`runtime_presenter.rs` 与 `runtime_entry_app/*`，避免结构测试继续引用已经删除的 `module_set/` 路径
- `zircon_script/src/vm/`
  - 已分成 `module/`、`backend/`、`host/`、`plugin/`、`runtime/`，明确区分 module owner、backend 封装和未来 plugin runtime
  - `ScriptModule` 现在通过 `PluginDescriptor` 注册 `ScriptModule.Plugin.VmPluginRuntime`，`VmPluginManager` manager façade 改为依赖并复用该 plugin 实例，`resolve_plugin` 路径已接通
  - `backend/backend_registry.rs`、`plugin/vm_plugin_package_discovery.rs`、`plugin/vm_plugin_package_source.rs`、`runtime/vm_plugin_slot_record.rs` 现已把 package discovery、backend selection 和 slot lifecycle 拉进真实代码路径；当前 plugin runtime 不再只是一个“能 resolve 到 manager façade 的空壳”
- `zircon_editor` 热点链路
  - `host/template_runtime/`、`host/manager/ui_asset_sessions/`、`editing/ui_asset/` 已改成 folder-backed 子树；其中 `ui_asset_sessions/mod.rs` 现已退回接线层，host-side 编辑命令入口挪到 `editing.rs`
- `zircon_scene/src/components/`
  - 已拆成 `scene`、`schedule`、`viewport`、`render_extract`、`gizmo` 五个子域，避免继续让 `components.rs` 同时承载 ECS、viewport packet、overlay 与 gizmo provider 合同

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

## Current Diagnosis

按当前审计规则，仓库状态不是“完全未结构化”，而是“descriptor 主干已成型，但仍未完全收敛”。

已经相对收敛的部分：

- `zircon_core`
  - descriptor/lifecycle/runtime spine 已成立
- `zircon_module`
  - 现在拥有明确的接口家族合同
- `zircon_app`
  - 现在拥有显式 `EngineEntry`
- `zircon_foundation`、`zircon_asset`、`zircon_input`、`zircon_graphics`、`zircon_scene`
  - 当前可以被识别为真实 module crate，而不是单纯的 descriptor 容器
- `zircon_manager`
  - 当前不再被误判成模块实现层，而是明确退回到 manager contract/facade support crate

仍然明确未收敛的部分：

- skeleton module crates 仍然大量存在，不能被误判为完成态
- `zircon_script` 已具备 package discovery、backend registry/default backend 选择、slot load/hot-reload/unload/list 生命周期，但 `PluginContext` 仍然主要停留在 core abstraction，真实 VM backend 也还只有 unavailable/mock 基线
- `zircon_editor` 的 `ui_asset` 会话和 host manager 子树仍然存在明显结构热点
- `zircon_app` 的 production 静态依赖扇出已经明显下降，但 runtime app 和 editor host 仍然直接持有 `scene/input/render_server/editor` 等真运行时依赖；它还不是完全无扇出的纯 profile shell
- 路线图中的 `zircon_server` 与现有 `zircon_framework` 仍然有命名/层次漂移

## Next Convergence Targets

从当前状态继续推进时，优先顺序应保持：

1. 继续替换 skeleton module crate 的 `stub_module_descriptor`
2. 继续降低 `zircon_app` 的剩余直接 runtime/editor host 依赖，避免它重新长回全模块静态组合器
3. 把 `zircon_script` 从当前的 discovery/backend/slot lifecycle 基线继续推进到真实 `PluginContext` 消费面和非 mock VM backend
4. 继续拆解 `zircon_editor` 的热点边界
5. 专门处理 `zircon_server` 与 `zircon_framework` 的层次和命名收敛

不要跳过这些步骤，直接宣布“接口体系已经完成”。当前完成的是抽象骨架和审计能力，不是全仓最终重构终点。

