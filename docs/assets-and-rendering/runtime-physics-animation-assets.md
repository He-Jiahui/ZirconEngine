---
related_code:
  - zircon_runtime/src/core/framework/scene/mod.rs
  - zircon_runtime/src/core/framework/scene/property_value.rs
  - zircon_runtime/src/core/framework/animation/mod.rs
  - zircon_runtime/src/core/framework/animation/manager.rs
  - zircon_runtime/src/core/framework/animation/parameter_map.rs
  - zircon_runtime/src/core/framework/animation/parameter_value.rs
  - zircon_runtime/src/core/framework/animation/playback_settings.rs
  - zircon_runtime/src/core/framework/animation/track_path.rs
  - zircon_runtime/src/core/framework/physics/mod.rs
  - zircon_runtime/src/core/framework/physics/manager.rs
  - zircon_runtime/src/core/framework/physics/material_metadata.rs
  - zircon_runtime/src/core/framework/physics/settings.rs
  - zircon_runtime/src/core/framework/physics/world_sync_state.rs
  - zircon_runtime/src/animation/animation_interface.rs
  - zircon_runtime/src/asset/assets/animation.rs
  - zircon_runtime/src/asset/assets/physics_material.rs
  - zircon_runtime/src/asset/assets/scene.rs
  - zircon_runtime/src/physics/physics_interface.rs
  - zircon_runtime/src/scene/components/scene.rs
  - zircon_runtime/src/scene/components/schedule.rs
  - zircon_runtime/src/scene/world/component_access.rs
  - zircon_runtime/src/scene/world/property_access/mod.rs
  - zircon_runtime/src/scene/world/records.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/asset/project/manager/collect_files.rs
  - zircon_runtime/src/animation/service_types.rs
  - zircon_runtime/src/animation/sequence_runtime.rs
  - zircon_runtime/src/physics/service_types.rs
  - zircon_runtime/src/asset/tests/assets/animation.rs
  - zircon_runtime/src/asset/tests/assets/physics_material.rs
  - zircon_runtime/src/asset/tests/assets/scene.rs
  - zircon_runtime/src/animation/tests.rs
  - zircon_runtime/src/physics/tests.rs
  - zircon_runtime/src/scene/tests/asset_scene.rs
  - zircon_runtime/src/scene/tests/property_paths.rs
  - zircon_runtime/src/scene/tests/physics_animation_components.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/reference_analysis.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/catalog_snapshot.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/asset_details.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/record_to_facade.rs
  - zircon_editor/src/ui/workbench/reflection/activity_actions/inspector_actions.rs
  - zircon_editor/src/ui/workbench/reflection/route_registration/action_route.rs
  - zircon_editor/src/ui/workbench/reflection/animation_route.rs
  - zircon_editor/src/scene/viewport/render_packet.rs
  - zircon_editor/src/ui/slint_host/viewport/test_render_framework.rs
  - zircon_editor/src/ui/slint_host/viewport/tests/fake_render_framework.rs
  - zircon_editor/ui/workbench/assets.slint
  - zircon_editor/ui/workbench/chrome.slint
  - zircon_editor/src/tests/host/asset_references.rs
  - zircon_editor/src/tests/host/slint_asset_pointer.rs
  - zircon_editor/src/tests/workbench/reflection/action_dispatch.rs
  - zircon_editor/src/ui/slint_host/app/assets.rs
  - zircon_editor/src/ui/slint_host/app/helpers.rs
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_manager.rs
implementation_files:
  - zircon_runtime/src/core/framework/scene/mod.rs
  - zircon_runtime/src/core/framework/scene/property_value.rs
  - zircon_runtime/src/core/framework/animation/mod.rs
  - zircon_runtime/src/core/framework/animation/manager.rs
  - zircon_runtime/src/core/framework/animation/parameter_map.rs
  - zircon_runtime/src/core/framework/animation/parameter_value.rs
  - zircon_runtime/src/core/framework/animation/playback_settings.rs
  - zircon_runtime/src/core/framework/animation/track_path.rs
  - zircon_runtime/src/core/framework/physics/mod.rs
  - zircon_runtime/src/core/framework/physics/manager.rs
  - zircon_runtime/src/core/framework/physics/material_metadata.rs
  - zircon_runtime/src/core/framework/physics/settings.rs
  - zircon_runtime/src/core/framework/physics/world_sync_state.rs
  - zircon_runtime/src/animation/animation_interface.rs
  - zircon_runtime/src/asset/assets/animation.rs
  - zircon_runtime/src/asset/assets/physics_material.rs
  - zircon_runtime/src/asset/assets/scene.rs
  - zircon_runtime/src/physics/physics_interface.rs
  - zircon_runtime/src/scene/components/scene.rs
  - zircon_runtime/src/scene/components/schedule.rs
  - zircon_runtime/src/scene/world/component_access.rs
  - zircon_runtime/src/scene/world/property_access/mod.rs
  - zircon_runtime/src/scene/world/records.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/asset/project/manager/collect_files.rs
  - zircon_runtime/src/animation/service_types.rs
  - zircon_runtime/src/animation/sequence_runtime.rs
  - zircon_runtime/src/physics/service_types.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/reference_analysis.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/catalog_snapshot.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/asset_details.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/record_to_facade.rs
  - zircon_editor/src/ui/workbench/reflection/activity_actions/inspector_actions.rs
  - zircon_editor/src/ui/workbench/reflection/route_registration/action_route.rs
  - zircon_editor/src/ui/workbench/reflection/animation_route.rs
  - zircon_editor/src/scene/viewport/render_packet.rs
  - zircon_editor/src/ui/slint_host/viewport/test_render_framework.rs
  - zircon_editor/src/ui/slint_host/viewport/tests/fake_render_framework.rs
  - zircon_editor/ui/workbench/assets.slint
  - zircon_editor/ui/workbench/chrome.slint
  - zircon_editor/src/tests/host/asset_references.rs
  - zircon_editor/src/tests/host/slint_asset_pointer.rs
  - zircon_editor/src/tests/workbench/reflection/action_dispatch.rs
  - zircon_editor/src/ui/slint_host/app/assets.rs
  - zircon_editor/src/ui/slint_host/app/helpers.rs
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_manager.rs
plan_sources:
  - user: 2026-04-20 继续正在runtime/editor/framework实现完整的物理和动画系统
  - user: 2026-04-20 physics和animation吸收进runtime
  - .codex/plans/Physics + Full Animation Support 新计划.md
  - .codex/plans/Physics  Full Animation Support Plan.md
  - .codex/plans/Runtime Core Fold-In And Compile Recovery.md
tests:
  - zircon_runtime/src/asset/tests/assets/animation.rs
  - zircon_runtime/src/asset/tests/assets/physics_material.rs
  - zircon_runtime/src/asset/tests/assets/scene.rs
  - zircon_runtime/src/animation/tests.rs
  - zircon_runtime/src/core/framework/tests.rs
  - zircon_runtime/src/physics/tests.rs
  - zircon_runtime/src/scene/tests/asset_scene.rs
  - zircon_runtime/src/scene/tests/property_paths.rs
  - zircon_runtime/src/scene/tests/physics_animation_components.rs
  - cargo test -p zircon_editor --locked --lib ui::slint_host::app::helpers::tests::derive_animation_assets_from_model_source_preserves_project_asset_ids_across_reimport_with_gltf_buffer_sidecars -- --exact --nocapture
  - cargo test -p zircon_editor --locked --lib ui::slint_host::app::helpers::tests::derive_animation_assets_from_model_source_writes_stable_sibling_skeleton_and_clip_files -- --exact --nocapture
  - cargo test -p zircon_editor --locked tests::host::asset_references::editor_asset_manager_tracks_scene_animation_and_physics_references -- --exact --nocapture
  - cargo test -p zircon_editor --locked slint_asset_pointer --target-dir target/codex-asset-filter-validation -- --nocapture
  - cargo test -p zircon_editor --locked asset_surface_templates_map_no_preview_physics_and_animation_assets_to_specific_icons --target-dir target/codex-asset-icon-validation-b -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --locked binding_dispatch --target-dir target/codex-asset-icon-validation-b -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --locked editor_event::runtime --target-dir target/codex-asset-icon-validation-b -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --locked slint_asset_pointer --target-dir target/codex-asset-icon-validation-b -- --nocapture --test-threads=1
  - cargo test -p zircon_editor --locked workbench_reflection_ -- --nocapture
  - zircon_runtime/src/scene/tests/world_basics.rs
  - cargo test -p zircon_runtime --locked --offline --target-dir target/codex-shared-b scene_asset_toml_roundtrip_preserves_physics_and_animation_components -- --nocapture
  - cargo test -p zircon_runtime --locked --offline --target-dir target/codex-shared-b scene_assets_roundtrip_asset_bound_physics_and_animation_components -- --nocapture
  - cargo test -p zircon_runtime --locked --offline --target-dir target/codex-shared-b world_project_roundtrip_preserves_physics_and_animation_components -- --nocapture
  - cargo test -p zircon_runtime --locked --offline --target-dir target/codex-shared-b apply_sequence_to_world_resolves_track_paths_and_updates_scene_properties -- --nocapture
  - cargo test -p zircon_runtime --locked --offline --target-dir target/codex-shared-b animation_manager_persists_playback_settings_to_runtime_config -- --nocapture
  - cargo test -p zircon_runtime --locked --target-dir target/manual-physics-animation physics_manager_syncs_world_snapshot_and_exposes_queries_and_contacts --lib
  - cargo test -p zircon_runtime --locked --target-dir target/manual-physics-animation animation_manager_evaluates_graphs_and_parameter_overrides --lib
  - cargo test -p zircon_runtime --locked --target-dir target/manual-physics-animation animation_manager_resolves_state_machine_transitions_from_parameters --lib
  - cargo test -p zircon_runtime --locked --target-dir target/manual-physics-animation animation_manager_samples_clip_pose_against_skeleton --lib
  - cargo test -p zircon_runtime --locked --target-dir target/manual-physics-animation
  - cargo test -p zircon_runtime --locked --offline --target-dir target/codex-shared-b physics_manager_persists_settings_to_runtime_config -- --nocapture
  - cargo test -p zircon_runtime --locked --offline --target-dir target/codex-shared-b physics_manager_tracks_fixed_step_accumulator_per_world -- --nocapture
  - cargo test -p zircon_runtime --locked --offline --target-dir target/codex-shared-b -- --nocapture
  - cargo test -p zircon_runtime --locked --offline core::framework::tests --target-dir D:/cargo-targets/zircon-workspace-hard-cutover -- --nocapture
  - cargo test -p zircon_runtime --locked --offline physics_and_animation_managers_resolve_through_framework_facades --target-dir D:/cargo-targets/zircon-workspace-hard-cutover -- --nocapture
doc_type: module-detail
---

# Runtime Physics And Animation Assets

## Purpose

这份文档记录已经真正吸收进 `zircon_runtime` 的物理/动画底层能力。当前范围已经不止资产与 scene 组件，还包括第一波执行层：

- 共享属性路径与 scene property runtime access
- sequence property application
- animation / physics manager 的 runtime config 持久化与 fixed-step bookkeeping

当前完成的是三条底层真源：

- `zircon_runtime::core::framework::{scene, physics, animation}` 提供共享 DTO、路径契约与 manager-facing settings
- `zircon_runtime::{asset, scene}` 负责 physics material、五类动画资产、scene 组件、以及 `SceneAsset <-> World` 的 typed roundtrip
- `zircon_runtime::{animation, physics}` 提供 sequence/property runtime 与 manager state/runtime config 接口

同时，这两块 framework root 也已经完成结构收口：

- `zircon_runtime/src/core/framework/physics/mod.rs`
- `zircon_runtime/src/core/framework/animation/mod.rs`

现在都只保留 `mod` 声明和受控 `pub use`；原先堆在 root `mod.rs` 里的 DTO、trait、default 行为和路径 helper 都已经下沉到 folder-backed 子文件。

这仍然不等于完整 physics backend、Jolt 驱动、skeleton graph runtime 或 editor sequence UI 都已落地，但 v1 已经不再只是“资产壳 + DTO 壳”。

## Ownership

当前物理和动画的稳定 ownership 固定为：

- `zircon_runtime::core::framework::scene`
  - `EntityPath`
  - `ComponentPropertyPath`
  - `ScenePropertyValue`
  - `ScenePropertyEntry`
- `zircon_runtime::core::framework::physics`
  - `PhysicsCombineRule`
  - `PhysicsMaterialMetadata`
  - `PhysicsSettings`
  - `PhysicsBackendStatus`
  - `PhysicsWorldSyncState`
  - `PhysicsRayCastQuery`
  - `PhysicsContactEvent`
- `zircon_runtime::core::framework::animation`
  - `AnimationTrackPath`
  - `AnimationParameterValue`
  - `AnimationPlaybackSettings`
  - `AnimationGraphEvaluation`
  - `AnimationStateMachineEvaluation`
  - `AnimationPoseOutput`
- `zircon_runtime::asset::assets`
  - `PhysicsMaterialAsset`
  - `AnimationSkeletonAsset`
  - `AnimationClipAsset`
  - `AnimationSequenceAsset`
  - `AnimationGraphAsset`
  - `AnimationStateMachineAsset`
  - `SceneAsset` 上的 physics/animation entity component DTO
- `zircon_runtime::scene`
  - 运行时权威 `World`
  - scene 组件存储、typed getter/setter、project JSON roundtrip
- `zircon_runtime::animation`
  - sequence track sampling
  - property-path based scene mutation
  - animation playback settings manager surface
- `zircon_runtime::physics`
  - physics settings manager surface
  - per-world fixed-step accumulator bookkeeping
  - backend status / world sync / ray-cast / contact fallback contract

这里的 structure 规则也一起固定下来：

- `core::framework::physics` 与 `core::framework::animation` 内部现在都和 `input/`、`render/`、`scene/` 一样采用 folder-backed subtree
- 顶层 `zircon_runtime/src/physics/mod.rs` 与 `zircon_runtime/src/animation/mod.rs` 也降成结构入口
- `PhysicsConfig` / `AnimationConfig` 进入 `config.rs`，module registration 与 service-name 常量进入 `module.rs`
- root `mod.rs` 不再允许重新吸收 DTO、trait、default impl 或 parse helper
- 后续新增 physics/animation contract 时，应该继续进入子文件，而不是回到 umbrella root

这里已经不再走旧的独立 `zircon_framework / zircon_manager / zircon_scene` crate 路径。

## Asset Kinds

当前 runtime 认可的 physics/animation 资产种类是：

- `PhysicsMaterial`
- `AnimationSkeleton`
- `AnimationClip`
- `AnimationSequence`
- `AnimationGraph`
- `AnimationStateMachine`

格式约束保持不变：

- physics material: `*.physics_material.toml`
- animation binary assets: `*.skeleton.zranim` / `*.clip.zranim` / `*.sequence.zranim` / `*.graph.zranim` / `*.state_machine.zranim`

动画资产仍然是版本化二进制 envelope，`AnimationSequenceAsset` 继续通过 `EntityPath + ComponentPropertyPath` 派生 canonical `AnimationTrackPath`。

## Framework Contract Subtrees

这轮之后，framework contract 形状已经显式固定：

- `core/framework/physics/`
  - `combine_rule.rs`
  - `material_metadata.rs`
  - `simulation_mode.rs`
  - `settings.rs`
  - `backend_state.rs`
  - `backend_status.rs`
  - `world_step_plan.rs`
  - `body_type.rs`
  - `collider_shape.rs`
  - `joint_type.rs`
  - `body_sync_state.rs`
  - `collider_sync_state.rs`
  - `joint_sync_state.rs`
  - `material_sync_state.rs`
  - `world_sync_state.rs`
  - `ray_cast_query.rs`
  - `ray_cast_hit.rs`
  - `contact_event.rs`
  - `manager.rs`
- `core/framework/animation/`
  - `parameter_value.rs`
  - `parameter_map.rs`
  - `track_path.rs`
  - `track_path_error.rs`
  - `playback_settings.rs`
  - `graph_clip_instance.rs`
  - `graph_evaluation.rs`
  - `state_machine_evaluation.rs`
  - `pose_source.rs`
  - `pose_bone.rs`
  - `pose_output.rs`
  - `manager.rs`

这里的目标不是把文件拆散，而是让 physics/animation framework contract 真正变成可持续增长的 owner subtree：

- `AnimationTrackPath` 的 parse/split 逻辑与 error 类型进入显式 owner file
- physics world sync / query / contact contract 不再和 backend settings/enum 声明共住一个 root
- root `mod.rs` 只负责结构声明和公开面，不再承载行为实现

## Model Import Derived Animation Assets

当前模型导入链没有把 runtime importer 改成 multi-output。相反，editor 在 `ImportModel` 流程里对 `gltf/glb` 做派生输出：

- 同目录 sibling skeleton: `*.skeleton.zranim`
- 同目录 sibling clip: `*.<clip>.clip.zranim`

关键约束如下：

- 生成路径稳定，重复导入同一模型时 locator 保持不变
- 生成文件落在 project `assets/` 内，再走现有单输出 `import_asset(...)` 链
- clip 内部对 skeleton 的引用直接指向派生 sibling locator，而不是额外的临时 ID
- `ProjectManager::scan_and_import()` 会忽略 glTF 外部 buffer sidecar `.bin`
  - 这些文件是模型源辅助物，不是独立 runtime 资产；否则项目重导入会在真正处理派生 `.zranim` 前就因为 unsupported format 失败
- 现有 editor/runtime 联合回归测试已经钉住：重复执行 `derive_animation_assets_from_model_source(...)` 再调用 `scan_and_import()` 时，派生 skeleton/clip 的 `.meta.toml` `asset_uuid` 和 registry `AssetId` 都保持稳定

这让 scene / graph / state-machine 对 clip 和 skeleton 的引用可以继续依赖现有 sidecar meta 稳定 UUID 规则，而不需要改写 `ImportedAsset` 或 `ProjectManager::scan_and_import()`。

## Scene Component Surface

`zircon_runtime::scene::components` 现在已经有第一波 physics/animation 组件：

- physics
  - `RigidBodyComponent`
  - `ColliderComponent`
  - `JointComponent`
- animation
  - `AnimationSkeletonComponent`
  - `AnimationPlayerComponent`
  - `AnimationSequencePlayerComponent`
  - `AnimationGraphPlayerComponent`
  - `AnimationStateMachinePlayerComponent`

这些组件都进入了三层权威数据面：

- `World` 内部 `HashMap<EntityId, Component>`
- `SceneNode` cache
- `NodeRecord`

因此它们现在能通过统一的 runtime world 查询、项目 JSON roundtrip、和 scene asset roundtrip 被观察到，而不是只停在 asset DTO。

## Scene Asset Mapping

`zircon_runtime/src/asset/assets/scene.rs` 现在对 entity 暴露了这些新字段：

- `rigid_body`
- `collider`
- `joint`
- `animation_skeleton`
- `animation_player`
- `animation_sequence_player`
- `animation_graph_player`
- `animation_state_machine_player`

设计约束如下：

- 所有新字段都带 `#[serde(default)]`
  - 旧 scene TOML 不会因为缺少这些字段而失效
- `TransformAsset` 现在有 identity `Default`
  - collider local transform 可以稳定省略
- collider 允许两种材质来源
  - `material: AssetReference`
  - `material_override: PhysicsMaterialMetadata`
- graph/state-machine 参数统一走 `BTreeMap<String, AnimationParameterValue>`
  - 这样序列化顺序稳定，editor/runtime roundtrip 可比对

## World Access And Mutation

`zircon_runtime::scene::world::component_access` 新增了 typed API：

- getters
  - `rigid_body(...)`
  - `collider(...)`
  - `joint(...)`
  - `animation_skeleton(...)`
  - `animation_player(...)`
  - `animation_sequence_player(...)`
  - `animation_graph_player(...)`
  - `animation_state_machine_player(...)`
- setters
  - `set_rigid_body(...)`
  - `set_collider(...)`
  - `set_joint(...)`
  - `set_animation_skeleton(...)`
  - `set_animation_player(...)`
  - `set_animation_sequence_player(...)`
  - `set_animation_graph_player(...)`
  - `set_animation_state_machine_player(...)`

这些 setter 只做最小 runtime 校验：

- entity 必须存在
- joint 不能连回自己

其余 physics/backend 级校验还没有下沉到这里。

## Shared Property Paths And Runtime Access

`zircon_runtime::core::framework::scene` 现在不只提供 `EntityPath + ComponentPropertyPath` DTO，还补齐了 runtime property surface：

- `ScenePropertyValue`
- `ScenePropertyEntry`

`zircon_runtime::scene::world::property_access` 则把这组 DTO 接到运行时世界：

- `entity_path(entity)` 从 hierarchy/name 反推稳定 `EntityPath`
- `resolve_entity_path(path)` 走 canonical 路径查回实体
- `property(entity, path)` 读取统一属性值
- `set_property(entity, path, value)` 通过路径回写组件属性
- `property_entries(entity)` 枚举 inspector / sequence 共用的属性条目

当前第一波 runtime 覆盖面已经包含：

- scene 基础组件
  - `Name`
  - `Hierarchy.parent`
  - `Transform.translation/rotation/scale`
  - `Active.enabled`
  - `RenderLayer.mask`
  - `Mobility.kind`
  - `Camera.*`
  - `MeshRenderer.*`
  - `DirectionalLight.*`
- physics 组件
  - `RigidBody.*`
  - `Collider.*`
  - `Joint.*`
- animation 组件
  - `AnimationSkeleton.skeleton`
  - `AnimationPlayer.*`
  - sequence/graph/state-machine player 的 clip、speed、time、weight、looping、playing 以及参数字典

这组路径 runtime access 是 editor inspector、asset track path 和 runtime sequence apply 的共同基础，而不是 animation 自己再走一条私有反射链。

## Sequence Runtime

`zircon_runtime::animation::sequence_runtime` 现在提供了第一波 sequence property runtime：

- `apply_sequence_to_world(world, sequence, time_seconds)`
- `AnimationSequenceApplyReport`
- `AnimationChannelAsset::sample(time_seconds)`

当前行为约束：

- binding 先通过 `EntityPath` 找到 world entity
- track 再通过 `ComponentPropertyPath` 走 `World::set_property(...)`
- Step 通道返回前一个 key
- Hermite 通道使用 cubic Hermite 标量/向量采样
- quaternion 通道当前走归一化 `slerp`

因此 `AnimationSequenceAsset` 已经可以直接驱动第一波 scene/physics/animation 组件属性，而不是停留在只可保存不可执行的资产格式。

## Runtime Manager State

runtime-owned physics / animation 模块现在也有了真正的 manager 状态，而不是只有 facade trait：

- `DefaultAnimationManager`
  - 从 `ANIMATION_PLAYBACK_CONFIG_KEY` 读取 `AnimationPlaybackSettings`
  - `store_playback_settings(...)` 会同时更新 runtime 内存态和 foundation config
  - `parameter_defaults(...)` / `parameter_value(...)` / `set_parameter(...)` 统一 graph/state-machine 参数面
  - `evaluate_graph(...)` 返回合并默认参数后的 clip 输出列表
  - `evaluate_state_machine(...)` 按当前状态和条件计算迁移与目标 graph
  - `sample_clip_pose(...)` 按 skeleton + clip 生成局部骨骼 pose
- `DefaultPhysicsManager`
  - 从 `PHYSICS_SETTINGS_CONFIG_KEY` 读取 `PhysicsSettings`
  - `store_settings(...)` 会同时更新 runtime 内存态和 foundation config
  - `backend_status()` 统一报告 requested backend、active backend、feature gate 和降级原因
  - `advance_clock(world, delta_seconds)` 维护每个 `WorldHandle` 的 fixed-step accumulator
  - `PhysicsTickPlan` 返回本帧 `steps / step_seconds / remaining_seconds`
  - `sync_world(...)` / `synchronized_world(...)` 保存当前 world 的中性 physics snapshot
  - `ray_cast(...)` / `drain_contacts(...)` 提供 contract-level query/contact 输出

`advance_clock(...)` 当前已经做了浮点边界保护，避免 `fixed_hz` 离散步进在临界帧长下少算 step。这样 physics module 至少已经有了后续 pre-sync / simulate / writeback 调度真正需要的 runtime bookkeeping。

## Scene Schedule

scene 默认阶段顺序已经按当前计划固定为：

- `PreUpdate`
- `FixedUpdate`
- `Update`
- `LateUpdate`
- `RenderExtract`

也就是说，physics fixed-step bookkeeping 先于 update / late-update 的 animation 参数写入阶段，而 render extract 始终在这两类运行时写入之后。

## Roundtrip Guarantees

当前已经通过测试钉住的行为：

- scene TOML roundtrip
  - physics/animation component DTO 不丢字段
- `SceneAsset <-> World`
  - asset reference 正确解析成 typed `ResourceHandle`
  - 再保存时还能回写到正确的 `res://...` locator
- `World` project JSON roundtrip
  - component maps、`SceneNode` cache、以及 getter API 都能恢复同一份数据
- shared property path
  - `EntityPath` / `ComponentPropertyPath` 可以跨 world 查询、路径写回和序列应用保持同一 canonical target
- runtime manager state
  - physics / animation settings 会同步进入 runtime config store
  - physics fixed-step accumulator 按 `WorldHandle` 分桶追踪
  - physics backend unavailable 会显式降级成 status，而不是隐式失败
  - physics world sync / ray cast / contact DTO 可以在无真实 backend 时继续跑 contract 验证
  - animation graph / state-machine / clip pose evaluator 能在 runtime 内直接执行

这意味着 physics/animation 现在已经进入 runtime 的 scene authority 与最小执行链，而不是停留在“只有 asset kind，没有实体组件”的空壳状态。

## Editor Asset Reference Analysis

editor asset manager 现在会把 scene 里的 physics/animation 组件资产一起纳入 direct-reference 分析，而不是只识别 mesh：

- `SceneColliderAsset.material`
- `SceneAnimationSkeletonAsset.skeleton`
- `SceneAnimationPlayerAsset.clip`
- `SceneAnimationSequencePlayerAsset.sequence`
- `SceneAnimationGraphPlayerAsset.graph`
- `SceneAnimationStateMachinePlayerAsset.state_machine`

这些引用全部走 `zircon_editor::ui::host::editor_asset_manager::manager::reference_analysis` 的 `ImportedAsset::Scene` 分支，进入与 graph/state-machine 相同的一套 reference graph。结果是：

- scene 资产的 `direct_reference_uuids` / `direct_references` 会同时展示 physics material、skeleton、clip、sequence、graph、state machine
- `catalog_snapshot.direct_reference_uuids` 会优先规范化成项目 catalog 内真实 `asset_uuid`
  - 不再直接暴露 `AssetReference::from_locator(...)` 生成的 locator-derived UUID
- `AnimationClip` 的 `referenced_by` 不再只来自 `AnimationGraph`，scene 直接播放器也会回指它
- `AnimationGraph` 的 `referenced_by` 不再只来自 state machine，scene graph player 也会回指它
- `AnimationStateMachine` 现在会在 scene state-machine player 挂载时被 editor 反向引用图捕获

这一步保证了 editor catalog、asset details 和后续 authoring 入口都建立在 runtime 同源的 scene 组件资产引用上，而不是另维护一套 editor-only 的补丁引用表。

## Inspector Animation Track Route

workbench reflection 的 inspector activity 现在也暴露动画轨道创建动作，但没有新增 inspector-only payload：

- action id: `create_animation_track`
- binding symbol: `AnimationCommand.CreateTrack`
- route registration: `zircon_editor::ui::workbench::reflection::animation_route`

这条路径的约束是刻意和现有 animation authoring pipeline 对齐的：

- inspector remote-call 只传一个字符串参数 `track_path`
- route stub 直接构造 `AnimationCommand::CreateTrack`
- binding normalization / dispatch 继续复用现有 animation binding 分支
- `track_path` 仍然走共享 `zircon_runtime::core::framework::animation::AnimationTrackPath::parse(...)`

因此 inspector 发起的“加轨道”不会再引入第二套 inspector mapping，也不会绕过 runtime 已接受的 canonical property path 模型。现有 workbench reflection 回归已经钉住：

- inspector 节点会公开 `create_animation_track`
- 该 action 可被远程调用并带有 route id
- 在 sequence editor 已打开时，通过 inspector 调这个 action 会真正把 canonical `AnimationTrackPath` 加入动画会话

## Asset Browser Kind Filters

physics / animation 资产虽然不会各自打开第三个专用 editor，但现在 generic asset surfaces 已经正式暴露这些 kind filter，而不是只能靠搜索字符串碰运气：

- `AssetsActivityPane` 和 `AssetBrowserPane` 都保留统一 `SetKindFilter` generic control route
- 两个 surface 现在都各自补齐了：
  - `PhysicsMaterial`
  - `AnimationSkeleton`
  - `AnimationClip`
  - `AnimationSequence`
  - `AnimationGraph`
  - `AnimationStateMachine`
- 这些 chip 直接回传 runtime/editor 已认可的 canonical kind 字符串
  - 没有再引入 UI-only alias 或第二套 animation/physics 过滤枚举
- 现有 host Slint 回归已经钉住：上面六类过滤入口在 activity 和 browser 两个 surface 中都必须各出现一次

这让计划里“`AnimationClip`、`AnimationSkeleton`、`PhysicsMaterial` 继续走现有 asset browser/details/preview，不单独做第三个动画 view 或 physics material 专用 editor”这一条不再只是路由层成立，而是连现有通用浏览面板的作者入口也真正可用。

这一轮又把无预览资产的图标回退链补齐到了同一完成线：

- `assets.slint` 的 activity/browser item thumbnail、list fallback、selection preview、details preview 现在都会显式识别 `PhysicsMaterial`
- 同一条回退链也覆盖 `AnimationSkeleton`、`AnimationClip`、`AnimationSequence`、`AnimationGraph`、`AnimationStateMachine`
- `chrome.slint` 新增了 `physics-material`、`animation-sequence`、`animation-graph` 这组 icon key 到 ionicon SVG 的固定映射

因此 physics / animation 资产即便没有 preview artifact，也不会再退回通用 `console` 占位图标，而是落到稳定、可识别的 kind-specific 视觉语义。

为了让这条 editor 回归真正能执行到断言，本轮还补齐了共享 render 契约扩展后的默认值路径：

- `zircon_runtime::ui::runtime_ui::runtime_ui_manager::empty_scene_snapshot(...)` 现在会给 `RenderSceneSnapshot.virtual_geometry_debug` 明确填 `None`
- `zircon_editor::scene::viewport::render_packet::build_render_packet(...)` 现在会给 `SceneViewportExtractRequest.virtual_geometry_debug` 明确填 `None`

这不是 physics / animation 资产新能力本身，但它避免了上层 asset/browser Slint 测试在编译阶段被共享 render DTO 漏补字段直接截断。

同一轮 editor acceptance sweep 里，`RenderFramework` 新增的 `query_virtual_geometry_debug_snapshot()` 也已经补到了 Slint viewport 的测试替身上：

- `zircon_editor::ui::slint_host::viewport::test_render_framework::TestRenderFramework`
- `zircon_editor::ui::slint_host::viewport::tests::fake_render_framework::FakeRenderFramework`

两个测试框架都显式返回 `Ok(None)`，这样 `binding_dispatch`、`editor_event::runtime` 和相关 viewport host 测试会继续通过同一条共享 render framework trait 路径，而不是因为测试替身落后于契约扩展而在编译期断掉。

## Not Done Yet

这份文档刻意不把未实现层包装成已完成：

- physics backend 驱动、fixed-step simulate/writeback、query/event、Jolt 接入
- Householder-based tangent fitting / key reduction editor math
- skeleton pose solve、graph/state-machine evaluator
- inspector canonical property model 与 sequence editor 的统一 authoring surface

当前仍未完成的是完整 backend 驱动接入、scene writeback 到真实刚体世界、以及 graph/state-machine/pose 解算真正挂进 render extract 消费链。底层共享合同、fallback query/contact、graph/state-machine evaluator 和 clip pose sampling 已经进入 runtime 主干，不再属于未落地空壳。
