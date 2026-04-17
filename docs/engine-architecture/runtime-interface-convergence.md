---
related_code:
  - zircon_module/src/lib.rs
  - zircon_module/src/engine_module.rs
  - zircon_module/src/engine_service.rs
  - zircon_entry/src/lib.rs
  - zircon_entry/src/entry/mod.rs
  - zircon_entry/src/entry/engine_entry.rs
  - zircon_entry/src/entry/builtin_entry_module_set.rs
  - zircon_entry/src/entry/entry_runner/bootstrap.rs
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
  - zircon_script/src/lib.rs
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
  - zircon_entry/src/lib.rs
  - zircon_entry/src/entry/mod.rs
  - zircon_entry/src/entry/engine_entry.rs
  - zircon_entry/src/entry/builtin_entry_module_set.rs
  - zircon_entry/src/entry/entry_runner/bootstrap.rs
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
  - zircon_script/src/lib.rs
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
  - cargo test -p zircon_manager contract_only_and_does_not_host_builtin_modules
  - cargo test -p zircon_foundation --lib
  - cargo test -p zircon_entry builtin_engine_entry_reports_run_mode_and_owned_modules
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

- `IEntry -> zircon_entry::EngineEntry`
- `IModule -> zircon_module::EngineModule`
- `IService -> zircon_module::EngineService`
- `IDriver -> zircon_module::EngineDriver`
- `IManager -> zircon_module::EngineManager`
- `IPlugin -> zircon_module::EnginePlugin`
- `IObject -> zircon_scene::RuntimeObject`
- `ISystem -> zircon_scene::RuntimeSystem`
- `IEntity -> zircon_scene::EntityIdentity`
- `IComponent -> zircon_scene::ComponentData`

这里最关键的边界是：`EngineService` 是共享 runtime 元数据合同，不是所有具体 service 实例都必须继承的统一业务基类。具体能力仍由 `AssetManager`、`InputManager`、`RenderServer`、`VmPluginManager` 这类 façade trait 或 handle surface 承担。

## Module Owner Convergence

本轮把内建 module crate 从“只有 `module_descriptor()` 自由函数”推进到了“显式模块拥有者类型 + 兼容自由函数”的状态。

- `zircon_entry::BuiltinEntryModuleSet` 现在持有 `Arc<dyn EngineModule>`，而不是仅缓存 `ModuleDescriptor`
- `zircon_animation`、`zircon_platform`、`zircon_physics`、`zircon_sound`、`zircon_texture`、`zircon_ui`、`zircon_net`、`zircon_navigation`、`zircon_particles`
  - 当前都已导出真实 module owner type，但仍然是 `stub_module_descriptor` skeleton
- `zircon_foundation`、`zircon_asset`、`zircon_input`、`zircon_graphics`、`zircon_scene`、`zircon_script`、`zircon_editor`
  - 当前都已导出真实 module owner type，并保留既有 descriptor 主干
- `zircon_manager`
  - 当前收敛为 support/facade crate，只保留 trait、resolver、handle 和稳定服务名，不再承载 `EngineModule` owner 或 config/event 具体实现

这样做的目的不是马上删掉所有自由函数，而是先让“模块拥有者”成为明确公共概念，后续其它入口、审计脚本、架构规则都可以围绕它收敛。

## Entry Convergence

`zircon_entry` 当前增加了三层明确合同：

- `EntryRunMode`
  - 明确 editor/runtime/headless 是入口运行模式，而不是散落在 `EntryRunner` 里的分支语义
- `EngineEntry`
  - 负责 profile、run mode、module owner 集合、bootstrap
- `BuiltinEngineEntry`
  - 当前默认实现，沿用原来的模块集合与 bootstrap 行为

`EntryRunner::bootstrap` 现在只是委托到 `BuiltinEngineEntry`，这意味着后续如果需要更换 profile 组合、拆 editor/runtime boot host、或引入更严格的入口配置，不需要继续把知识堆进一个薄 runner。

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
- `zircon_entry` 的静态依赖扇出
- 生产代码大文件热点
- module crate 分类结果

## Current Diagnosis

按当前审计规则，仓库状态不是“完全未结构化”，而是“descriptor 主干已成型，但仍未完全收敛”。

已经相对收敛的部分：

- `zircon_core`
  - descriptor/lifecycle/runtime spine 已成立
- `zircon_module`
  - 现在拥有明确的接口家族合同
- `zircon_entry`
  - 现在拥有显式 `EngineEntry`
- `zircon_foundation`、`zircon_asset`、`zircon_input`、`zircon_graphics`、`zircon_scene`
  - 当前可以被识别为真实 module crate，而不是单纯的 descriptor 容器
- `zircon_manager`
  - 当前不再被误判成模块实现层，而是明确退回到 manager contract/facade support crate

仍然明确未收敛的部分：

- skeleton module crates 仍然大量存在，不能被误判为完成态
- `zircon_script` 还没有把 plugin runtime 真正贯穿到 `PluginDescriptor` 和 `resolve_plugin`
- `zircon_entry/Cargo.toml` 仍然持有较重的静态依赖扇出
- `zircon_editor` 的 `ui_asset` 会话和 host manager 子树仍然存在明显结构热点
- `zircon_entry` 目前还直接静态依赖 `zircon_foundation`，说明 built-in runtime foundation 还未进一步抽象成更轻的 profile 组合层
- 路线图中的 `zircon_server` 与现有 `zircon_render_server` 仍然有命名/层次漂移

## Next Convergence Targets

从当前状态继续推进时，优先顺序应保持：

1. 继续替换 skeleton module crate 的 `stub_module_descriptor`
2. 打通 `zircon_script` 的真实 plugin lifecycle
3. 降低 `zircon_entry` 的静态依赖扇出
4. 继续拆解 `zircon_editor` 的热点边界
5. 专门处理 `zircon_server` 与 `zircon_render_server` 的层次和命名收敛

不要跳过这些步骤，直接宣布“接口体系已经完成”。当前完成的是抽象骨架和审计能力，不是全仓最终重构终点。
