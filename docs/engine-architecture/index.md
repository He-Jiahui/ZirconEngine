---
related_code:
  - zircon_app/src/lib.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_runtime/src/foundation/mod.rs
  - zircon_plugins/mod.rs
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/core/runtime/mod.rs
  - zircon_runtime/src/core/manager/mod.rs
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/core/math/mod.rs
  - zircon_runtime/src/core/resource/mod.rs
  - zircon_runtime/src/engine_module/mod.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly.rs
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - docs/zircon_plugins/first_party_runtime_catalog.md
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/scene/inspection/mod.rs
  - zircon_runtime/src/scene/inspection/snapshot.rs
  - docs/zircon_runtime/scene/inspection.md
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/plugin/export_build_plan/mod.rs
  - zircon_runtime/src/plugin/native_plugin_loader/mod.rs
  - zircon_editor/src/lib.rs
  - zircon_runtime/src/script/mod.rs
implementation_files:
  - zircon_app/src/lib.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_runtime/src/foundation/mod.rs
  - zircon_plugins/mod.rs
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/core/runtime/mod.rs
  - zircon_runtime/src/core/manager/mod.rs
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/core/math/mod.rs
  - zircon_runtime/src/core/resource/mod.rs
  - zircon_runtime/src/engine_module/mod.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly.rs
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - docs/zircon_plugins/first_party_runtime_catalog.md
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/scene/inspection/mod.rs
  - zircon_runtime/src/scene/inspection/snapshot.rs
  - docs/zircon_runtime/scene/inspection.md
  - zircon_runtime/src/graphics/mod.rs
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/plugin/export_build_plan/mod.rs
  - zircon_runtime/src/plugin/native_plugin_loader/mod.rs
  - zircon_editor/src/lib.rs
  - zircon_runtime/src/script/mod.rs
plan_sources:
  - user: 2026-04-13 将架构优先规则保留到 docs 下面用于生产项目 wiki
  - user: 2026-04-15 implement the f64-ready runtime foundation plan with math/scene/asset/graphics boundaries
  - .codex/plans/全系统重构方案.md
tests:
  - docs/engine-architecture/architecture-first-development.md
  - docs/engine-architecture/core-runtime-service-registry.md
  - docs/engine-architecture/runtime-interface-convergence.md
  - docs/engine-architecture/generated-code-boundary.md
  - docs/engine-architecture/non-network-server-naming-m1.md
  - docs/engine-architecture/hard-cutover-migration-smells-m1.md
  - docs/engine-architecture/large-file-ownership-m1.md
  - docs/engine-architecture/native-plugin-boundary.md
  - docs/zircon_runtime/builtin/runtime_modules.md
  - docs/zircon_runtime/scene/inspection.md
  - docs/engine-architecture/runtime-foundation-precision-and-scene-authority.md
  - cargo check --workspace
doc_type: category-index
---

# Engine Architecture

## Purpose

本目录记录 `zirconEngine` 的全局引擎架构约束，而不是某个单独模块的实现细节。这里的文档用于约束跨 crate 设计、系统重构、模块接入和上层功能开发，确保所有工作都优先落在明确的引擎框架结构上，而不是用一次性的直接实现推进主链路。

## Documents

- [Architecture-First Development](./architecture-first-development.md): `zircon_app -> zircon_runtime::core::{runtime, manager, framework, math, resource} -> zircon_runtime modules -> zircon_editor` 主干、ECS 运行时世界、manager contracts、runtime absorption 模块、`LevelManager -> LevelSystem -> World` 分层、VM 插件边界、架构优先设计流程、主流引擎对齐要求和实现红线。
- [Core Runtime Service Registry](./core-runtime-service-registry.md): `zircon_runtime::core::runtime` 的目录化边界，公开导出层、descriptor 子树、`CoreHandle` 行为文件、`PluginFactory + PluginContext` 分流，以及后续继续扩展 service registry 时必须遵守的模块纪律。
- [Runtime Interface Convergence](./runtime-interface-convergence.md): `EngineEntry`、`EngineModule`、`EngineService`、ECS 语义合同、内建 module owner 收敛、`zircon_plugins` 对可选扩展注册面的吸收、结构审计 skill，以及当前 `converged/skeleton/needs-refactor` 诊断基线。
- [Runtime Module Assembly](../zircon_runtime/builtin/runtime_modules.md): `zircon_runtime::builtin::runtime_modules` 的 folder-backed runtime-owned facade，拆分 target/profile identity、manifest baseline、availability diagnostics、extension aggregation、plugin-domain mapping、core module vector construction 和 assembly orchestration。
- [First-Party Runtime Catalog](../zircon_plugins/first_party_runtime_catalog.md): `zircon_first_party_runtime_catalog` 作为 plugin workspace 的 linked-provider catalog，集中一方 runtime provider 的可选 Rust crate fan-out，让 `zircon_app` 只投影 profile/render config 而不直接依赖每个 `zircon_plugin_*_runtime`。
- [Runtime Scene World Inspection](../zircon_runtime/scene/inspection.md): `zircon_runtime::scene::inspection` 的中性 `WorldInspection` hierarchy/field snapshot，替代 runtime 里的 editor-named projection，让 `zircon_editor::scene` 自己生成作者态 `SceneEditModeProjection`、selection、overlay、gizmo 和 viewport state。
- [Generated Code Boundary](./generated-code-boundary.md): generated output 只能作为 leaf data/table/manifest/schema/adapter，禁止持有 runtime bootstrap、plugin registration、native loading、module/service resolution、ECS mutation 或 public architecture 决策；结构审计会点名 export source-template 中的架构敏感行为。
- [Non-Network Server Naming M1 Gate](./non-network-server-naming-m1.md): `server` 只保留给真实 network/target-runtime/service-host/dev-server/external API 语义；结构审计过滤 `observer` 子串噪音，并把剩余 editor asset/resource、graphics render-framework 和 editor scene comment 命名债务分类到明确 owner。
- [Hard-Cutover Migration Smells M1 Gate](./hard-cutover-migration-smells-m1.md): 结构审计扫描生产 Rust 里的 `legacy`、`compat`、`shim`、`bridge` 迁移气味；`compat/shim` 是硬切 blocker，`legacy` 必须按 owner 消债，普通业务 `bridge` 允许但迁移语境 bridge 不允许。
- [Large File Ownership M1 Gate](./large-file-ownership-m1.md): 结构审计把超过 1000 行的生产热点提升为 owner gate；后续拆分必须按 runtime/editor/Hub owner 和行为 family 切分，不做任意行数切块或兼容 facade。
- [Native Plugin Boundary](./native-plugin-boundary.md): native dynamic loading 不是 runtime plugin 主路径；`zircon_runtime::plugin` 应保留 VM/plugin lifecycle、manifest、descriptor、feature registration、profile 和 scene hook 合同，native loader/ABI surface 迁到隔离 namespace、tool/export facade 或测试路径，结构审计用 M4 gate 分类 ABI、loader/discovery、live-host runtime 和 behavior report re-export 债务。
- [Runtime Network Extension](./runtime-network-extension.md): `core::framework::net` 的中性 socket/message-loop 合同、`core::manager` 上的 `NetManager` contract / handle、`zircon_plugin_net_runtime` 的 Tokio TCP/UDP base runtime，以及 `net.http` / `net.websocket` / `net.rpc` / `net.replication` / `net.reliable_udp` / `net.content_download` 可选 feature runtime 边界。
- [Runtime Sound Extension](./runtime-sound-extension.md): `core::framework::sound` 的最小 clip/playback/mix 合同、asset 管线里的 `.wav -> SoundAsset` 支撑、`core::manager` 上的 `SoundManager` contract / handle，以及 `zircon_plugin_sound_runtime` 的 `software-mixer` MVP。
- [Runtime Diagnostics Contract](./runtime-diagnostics-contract.md): `core::diagnostics` 的只读 runtime inspection snapshot、render/physics/animation manager 聚合，以及 editor diagnostics pane 的 `.ui.toml` 接线边界。
- [Runtime/Editor Pluginized Export](./runtime-editor-pluginized-export.md): Runtime/Editor 最小本体、项目插件清单、导出 profile、editor capability gating、独立 `zircon_plugins` workspace 与插件包 runtime/editor crate 分离规则。
- [Plugin Optional Feature Bundles](./plugin-optional-feature-bundles.md): 多插件组合功能的 owner-plugin 子功能模型、all-of capability dependency 规则、feature registration 顺序和导出链接规则。
- [Runtime Foundation Precision And Scene Authority](./runtime-foundation-precision-and-scene-authority.md): `zircon_runtime_interface::math` 精度 seam、`zircon_runtime::core::math` re-export 入口、runtime scene 的 `LocalTransform + WorldMatrix + ActiveSelf/ActiveInHierarchy + RenderLayerMask + Mobility` authority、scene serializer 默认化字段，以及 graphics renderer 的 runtime-to-render downcast 边界。
- [Workspace Ownership Cutover Map](./workspace-ownership-cutover-map.md): workspace hard-cutover 的旧 owner -> 新 owner 权威映射，以及删旧、收根、去兼容层时必须遵守的 owner 依据。
- [Workspace Root Rules And Hard Cutover](./workspace-root-rules-and-hard-cutover.md): 固定三包形态、root file 红线、hard-cutover 删除规则，以及 crate root/public surface 的长期标准。

## Current Scope

当前目录覆盖的系统级约束包括：

- 以 [全系统重构方案](../../.codex/plans/全系统重构方案.md) 为默认权威路线图的全局架构基线
- `EntryRunner`、`CoreRuntime`、模块 descriptor、manager contracts / handles、`zircon_runtime` 吸收的 foundation/input/platform/script 实现目录与 asset/scene/graphics/ui/optional-extensions module-registration surface、`LevelManager -> LevelSystem -> World`、editor host、VM plugin 的职责分层
- `M2` optional extensions 里的 `net` / `sound` 最小可用闭环，其中 `net` 负责 socket/message-loop MVP，`sound` 负责 `.wav` asset import + clip playback + software mix MVP
- `EngineEntry`、`EngineModule`、`EngineService` 与 `RuntimeObject/RuntimeSystem/EntityIdentity/ComponentData` 这组接口家族和语义合同
- `CoreRuntime` service registry 的文件级边界和 `runtime/mod.rs` 只做导出层的结构纪律
- runtime module assembly 的 folder-backed owner 拆分，以及 `zircon_app` optional plugin implementation fan-out 已迁到 plugin workspace 的 `zircon_first_party_runtime_catalog`
- generated code boundary：生成产物只能是 leaf data/table/manifest/schema/adapter，export source-template 中的 runtime bootstrap、plugin registration、native loading 等行为必须迁回手写 owner
- native plugin M4 public-surface gate：native ABI、loader/discovery、live-host runtime 和 behavior report symbol 不允许继续作为 `zircon_runtime::plugin` root-level re-export
- non-network `server` 命名 gate：`observer` 不是 server 命名，真实 server/target/dev-server 语义允许，剩余非网络命名必须在 M6/M7 owner slice 中硬切改名
- hard-cutover migration smell gate：生产 Rust 中的 `legacy`、`compat`、`shim` 和迁移语境 `bridge` 必须按 owner 硬切删除或改成明确版本策略，不能保留兼容层、shim、alias 或 forwarding bridge
- large-file ownership gate：超过 1000 行的 runtime/editor/Hub 生产热点必须先按 owner 分组，再按行为 family 拆分，不能用任意行数切块掩盖混合职责
- `zircon_runtime_interface::math -> zircon_runtime::scene -> runtime scene serializer -> graphics renderer` 这条 runtime foundation 的精度与派生态边界
- workspace hard-cutover 之后 `zircon_app` / `zircon_runtime` / `zircon_editor` 的固定 owner 形态与 root file 纪律
- “先抽象框架，后写功能实现”的工程规则
- “先检查是否和主流引擎模式对齐，过于简单时优先深化架构设计”的设计规则
- 跨 crate 功能接入时对 sibling `zircon_*` crates 的一致性要求

后续如果继续细化 `zircon_runtime::core::runtime` 生命周期、`zircon_runtime::core::manager` contract 族、`zircon_runtime::foundation` 的 clock/config/event/scheduler 内建模块拆分、`zircon_runtime::scene` 的 `LevelSystem` 子系统托管、runtime `f64` 切换过程或 `zircon_runtime::script` VM 热替换协议，可以在本目录继续追加叶子文档。

