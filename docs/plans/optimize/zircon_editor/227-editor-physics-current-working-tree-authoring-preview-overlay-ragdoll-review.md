---
title: Editor Physics 当前工作树 Authoring / Preview / Overlay / Ragdoll / Collision Workbench 复审
category: zircon_editor
report_id: Editor227
review_date: 2026-08-30
baseline_head: working-tree
baseline_epoch: 2026-08-30
verification_head: working-tree
verification_epoch: 2026-08-30
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/140-editor-physics-material-rigidbody-collider-joint-collision-profile-cook-ragdoll-debug-preview-current-source-review.md
  - docs/plans/optimize/zircon_editor/94-editor-physics-material-rigidbody-collider-joint-collision-profile-cook-ragdoll-debug-preview-product-integration-current-source-review.md
related_runtime_owner:
  - docs/plans/optimize/zircon_runtime/167-runtime-physics-current-working-tree-world-sync-jolt-fixed-step-query-event-ragdoll-review.md
related_plugin_owner:
  - docs/plans/optimize/zircon_plugins/12-first-party-physics-source-runtime-editor-dist-catalog-simulation-collision-joint-ragdoll-product-integration-review.md
related_editor_owners:
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/59-editor-scene-viewport-interaction-controller-input-picking-selection-highlight-gizmo-transaction-cancel-generation-product-integration-current-source-review.md
related_failure:
  - docs/plans/zircon_plugins/03/failure-2026-08-01-physics-debug-overlay-provider-missing.md
related_code:
  - zircon_plugins/physics/editor
  - zircon_editor/src/core/editor_extension/viewport_overlay_provider.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_overlay_providers.rs
  - zircon_editor/src/core/asset/type_registry
  - zircon_editor/src/core/document
  - zircon_editor/src/core/editing
  - zircon_editor/src/core/jobs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback/simulation_physics.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/simulation_physics.rs
  - zircon_editor/src/ui/retained_host/workbench_preview_actions/extensions.rs
  - zircon_plugins/first_party_editor_catalog/src/catalog.rs
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/PhysicsAssetEditor
  - dev/UnrealEngine/Engine/Source/Editor/StaticMeshEditor/Private/StaticMeshAutomationTests.cpp
  - dev/Fyrox/editor/src/plugins/collider
  - dev/Fyrox/editor/src/plugins/inspector/editors/mod.rs
  - dev/godot/editor/scene/3d/gizmos/physics
  - dev/godot/editor/scene/3d/physics/physical_bone_3d_editor_plugin.cpp
  - dev/bevy/crates/bevy_time/src/fixed.rs
  - dev/Graphics/Packages/com.unity.visualeffectgraph
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
tooling_scope: excluded_by_user_request
---

# Editor227 · Physics 当前工作树复审

## 1. 结论

当前 Editor 的 Physics surface 仍是 descriptor 和静态 preview 的组合，不是工程级 authoring 产品。Physics 插件注册了 authoring、diagnostics、debug overlay 和 ragdoll profile view，但 `authoring.zui`、`debug_overlay.zui`、`diagnostics.zui`、`ragdoll_profile.zui` 四个文件总计 12 个 plugin-editor 文件之外的实际业务 UI 仍由 `Space` 占位；插件源代码只有 `build_physics_overlay`（snapshot -> primitive）和 `generate_initial_ragdoll_profile`（字符串骨骼 -> capsule profile）两个纯函数，均没有 production controller、document、job、PreviewWorld、cook 或 runtime receipt。

Editor 核心已经拥有真正的 `ViewportOverlayProviderRegistration`、toggle command、provider retirement/cleanup 和 generation-aware viewport controller，但 Physics 没有注册 provider，Physics debug toggle 在 `plugin.rs:76-102` 仍通过 `WorkbenchMenu(OpenView(...))` 导航。Ragdoll create 在 `plugin.rs:123-131` 同样只是打开 view。`first_party_editor_catalog/src/catalog.rs:41-54` 没有 Physics provider，runtime catalog `src/lib.rs:34-99` 也没有 Physics registration；Editor 的 Physics 资产类型能被通用 Asset Browser 过滤，却没有专属 toolkit executor。

Physics Collision 与 Collision Proxy Workbench 的 route/feedback 依然是 UI-only projection：`simulation_physics.rs` 中的 open/select/edit/commit 只更改 tab、row、control 或固定状态文本；simulate/validate/bake 没有 JobId、source revision、artifact digest、PreviewWorld、cancel 或 terminal receipt。它们将固定的 body/contact/material 示例当作运行反馈，不能作为真实 Jolt/Runtime 证据。

因此 Editor140/94 的 Physics authoring 结论只需刷新 currentness，不关闭任何问题：5 项 P0、60 项 P1、12 项 P2 全部 Open，32 项资格门全部 Fail。Runtime167 拥有 solver/world/query/event；Plugins12 拥有 package/catalog/dist；Editor227 只持有 authoring、preview、overlay、inspector、Workbench controller 和 Editor -> Runtime 的正式边界。没有运行 Editor、Jolt preview、资产 cook/save/reopen 或产品 bootstrap，本报告不作性能或功能完成声明。

## 2. 冻结范围与方法

本轮只统计 Physics Editor/plugin、Editor physics consumer 与正式 viewport/document/asset/job boundary；当前工作树包含其他 session/用户改动，读取但不覆盖。统计按 repository-relative lowercase path + file SHA-256 生成 fingerprint；测试数量只说明合同覆盖，不代表产品可达。

| 范围 | 文件 / 行 / bytes / tests / ignored | fingerprint |
|---|---:|---|---|
| Physics editor plugin、editor manifest 与四份 ZUI | 12 / 629 / 22,052 / 4 / 0 | `4ff4ad3d0dcd49992a17a8a565a74b5d09e61dd9ebdfae8efddf090e0386a56d` |
| Runtime/Physics plugin 与 catalog downstream（引用 Runtime167） | 见 Runtime167 | 见 Runtime167 |
| Editor formal overlay/controller boundary（结构证据） | `ViewportOverlayProvider` registration、toggle、retirement、cleanup 已存在；Physics registration 为 0 | n/a |
| 参考 Unreal/Fyrox/Godot/Bevy/Unity Graphics | 以 Editor140 选集为基础，本轮增补当前 provider/controller 调用点 | n/a |

按用户要求没有查询、轮询、等待或实时跟踪协调器；Tooling 不进入本轮。未运行 Cargo、Editor、真实窗口、Jolt native preview、asset cook/reopen、fault、scale、soak 或 benchmark。

## 3. 当前 Editor 调用链

```text
PhysicsEditorPlugin
  -> register authoring drawer + World surface
  -> register debug view + UI template + command(OpenView)
  -> register diagnostics template + surface
  -> register ragdoll template + asset type/toolkit + command(OpenView)

ZUI / Workbench callback
  -> navigation spec / fixed feedback
  -> no Physics document/controller/job/PreviewWorld

build_physics_overlay(sync)
  -> Vec<PhysicsOverlayPrimitive>
  -> no ViewportOverlayProviderRegistration caller

generate_initial_ragdoll_profile(skeleton)
  -> path strings + translation-length capsule defaults
  -> no asset transaction/cook/preview/runtime artifact
```

Editor 核心的正式 provider boundary 存在于 `zircon_editor/src/core/editor_extension/viewport_overlay_provider.rs` 与 `scene/viewport/controller/scene_viewport_controller_overlay_providers.rs`，其中已有 unknown provider、quarantined、disabled capabilities、retirement cleanup 等错误语义。Physics 未消费该边界，因此旧 failure handoff 仍保持 Open。

## 4. P0：Editor 伪可达

| ID | 状态 | 当前源码证据 | 必须重构 |
|---|---|---|---|
| PHY3E-P0-01 | Open | Editor catalog 没有 Physics provider；Ragdoll asset type 只在 plugin load 后出现 | 同一 project selection 生成 editor/runtime/resource/provider closure；缺项显示 unavailable，不显示完整工具 |
| PHY3E-P0-02 | Open | 四份 ZUI 的业务节点均为 `Space`，无 Button/event/route binding | 真实 retained controller 投影 document、selection、validation、job progress 与 error；未实现能力隐藏或 disabled |
| PHY3E-P0-03 | Open | Ragdoll create command 只有 `OpenView` event，generator production caller 为 0 | skeleton identity -> transactional PhysicsAsset/Ragdoll source -> cook artifact -> isolated PreviewWorld -> save/spawn/despawn receipt |
| PHY3E-P0-04 | Open | debug toggle 只有 `OpenView`，Physics provider registration 为 0 | 注册 Physics-owned `ViewportOverlayProvider`，消费 Runtime generation-bound snapshot，正式调用 `ViewportCommand::ToggleOverlayProvider` 并支持 retirement cleanup |
| PHY3E-P0-05 | Open | Workbench simulate/validate/bake feedback 使用固定 queued/sample 文本，不产生 JobId/artifact | typed operation -> job admission/progress/cancel/terminal result；没有真实 executor 时明确 Unsupported，删除伪成功反馈 |

## 5. P1：Asset、Document、Inspector、Gizmo 与 Cook

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| PHY3E-P1-001 | Open | Physics Material 只有通用 builtin presentation/Asset Browser filter，没有专用 toolkit | source document、history、typed field editor、reference graph、save/reopen/reimport、readonly/LKG 状态 |
| PHY3E-P1-002 | Open | RigidBody/Collider/Joint 只有通用 property paths，缺 schema-driven inspector 与 multi-edit | Runtime schema/validator 单一来源，body/shape/joint/material/filter 全字段、mixed value、atomic transaction |
| PHY3E-P1-003 | Open | Compound/multi-shape 没有 child/subshape stable ID、tree、add/remove/reorder/duplicate | stable element identity、hierarchical selection、local frame/scale/mirror policy、undo/redo |
| PHY3E-P1-004 | Open | Joint UI 未提供双 local frame、limits/drives/motors/break/collision/backend support | typed body picker、joint-kind schema、capability-gated fields 与 preview diagnostic |
| PHY3E-P1-005 | Open | Collision Profile、layer/group/mask、query-vs-solver response 没有 versioned asset/matrix | profile artifact、reference impact scan、rename/delete migration、backend limit/error projection |
| PHY3E-P1-006 | Open | Collision Proxy Bake route 没有 source hash/settings/backend/platform revision ack | job snapshot、deterministic cook、cancel、stale result rejection、artifact digest/DDC/residency |
| PHY3E-P1-007 | Open | Convex/mesh/heightfield/compound authoring 仍以 UI route 表示，未绑定 cook executor | geometry validation、weld/degenerate/material slots、heightfield tile revision、optimized native artifact |
| PHY3E-P1-008 | Open | 质量、COM、principal axes、inertia tensor、contact feature 不在 viewport preview | Runtime compiler 结果回投，显示 source/derived/backend 值和误差，不在 Editor 重新估算 |
| PHY3E-P1-009 | Open | play/edit/preview 期间字段权限和 apply-back policy 未定义 | isolated `PhysicsPreviewSession`，world generation、pause/step/reset、apply/revert transaction |
| PHY3E-P1-010 | Open | plugin disable/unload 没有 Physics preview/job/provider drain contract | revoke admission -> cancel jobs -> stop preview -> clear overlay -> reject late commit -> unload |

## 6. P1：Overlay、Diagnostics 与 Workbench 真实性

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| PHY3E-P1-011 | Open | `build_physics_overlay` 仅复制 collider DTO，缺 world/provider generation、body/joint/contact/COM/sleep/capacity | provider 消费 immutable Runtime publication，提供 filter/pick/capture/overflow/stale age |
| PHY3E-P1-012 | Open | Debug view filters 与 wireframes 是 `Space`，toggle 不走 viewport controller | filters 生成 typed provider state；overlay command 与 UI state 分离 |
| PHY3E-P1-013 | Open | Diagnostics 的 step history、collision matrix、world statistics 没有真实 source | 使用 Runtime per-world diagnostic snapshot，显示 step debt、islands/pairs/contacts/query/capacity/fault/cook |
| PHY3E-P1-014 | Open | Workbench `extension_module_feedback/simulation_physics.rs` 的 simulate/validate/bake 只是常量映射 | action payload 必须含 target identity/source revision/backend；terminal projection 区分 queued/running/applied/rejected/stale |
| PHY3E-P1-015 | Open | Workbench navigation actions 没有 target identity/selection persistence，edit/commit 不是 document transaction | callback -> controller -> transaction/job -> result；纯 tab/select route 单独标为 UI-only |
| PHY3E-P1-016 | Open | 固定 Player Capsule、Ice、124 bodies、32 contacts、82 kg 等文本制造成功表象 | 无 selection/runtime 时显示 Unavailable；所有值来自 qualified asset/scene/runtime generation |
| PHY3E-P1-017 | Open | debug overlay failure handoff 仍无 provider registration、共享 extract 与 Cargo green evidence | 真实 provider、stale cleanup、capture、受管验证和产品 bootstrap 通过后再关闭 handoff |

## 7. P1/P2：Ragdoll 与高级 authoring

| ID | 状态 | 当前差距 | 重构要求 |
|---|---|---|---|
| PHY3E-P1-018 | Open | `RagdollSkeletonBone` 使用 path 字符串；profile 只有默认 mass、capsule、Generic6Dof metadata | skeleton/rig signature、stable bone IDs、per-bone shape/material/filter/body/constraint source、reimport remap |
| PHY3E-P1-019 | Open | generator 按 local translation 长度猜 capsule，不读取 mesh/skin/pose/orientation，也无左右对称/预算诊断 | geometry fit/cook preview、explicit overrides、quality/error/triangle/hull budget |
| PHY3E-P1-020 | Open | ragdoll view 没有 skeleton tree、preview、bone property controller；spawn/despawn 无 Editor owner | document selection、PreviewWorld native lifecycle、partial-spawn rollback、save/reopen/cook/runtime parity |
| PHY3E-P1-021 | Open | physical animation、blend/recovery、authority handoff 没有 authoring schema | drive profiles、strength/mode/transition、network/save receipt 和 Runtime167 的 generation contract |
| PHY3E-P2-001 | Open | character controller authoring 没有 step/slope/snap/slide/platform 场景与 trace | dedicated controller asset、query/solver preview和golden movement corpus |
| PHY3E-P2-002 | Open | vehicle、soft body、cloth、rope、destruction 没有 Editor owner | 每类独立 source/artifact/provider/capability、cook、preview、debugger和资格门 |
| PHY3E-P2-003 | Open | physics capture/scrub/state diff、remote inspector、constraint profile library 缺失 | versioned capture、replay oracle、timeline、batch retarget和multi-user semantic diff |

## 8. 参考引擎差异

- Unreal Physics Asset Editor 以真实 PhysicsAsset package、Body/Shape/Constraint CRUD、`Modify()`/事务、mesh invalidation 和 preview/editor mode 管理为中心；当前 Ragdoll `OpenView` + pure generator 没有对应资产生命周期。
- Unreal StaticMesh collision automation 把 collision geometry/cook 作为可验证 artifact，包含输入 mesh、质量设置和自动化断言；当前 Collision Proxy Bake 只有固定 feedback。
- Godot physics gizmo/physical bone editor 将 scene node、shape/joint、skeleton bone、debug draw 与 Undo/Redo 直接连接，并能显示编辑态/运行态差异；Physics ZUI 仍是 Space，不能承载该合同。
- Fyrox collider/inspector plugin 直接编辑 Rapier collider shape、sensor、friction、restitution、groups、handles，并由 scene graph owner 保存；Zircon 通用 Inspector 尚未提供 Physics variant/multi-shape semantics。
- Bevy fixed clock 仅用于 Editor preview 的时间语义参考：preview step 必须消费和 Runtime 相同的 fixed substep，不得另建 Editor clock。
- Unity Graphics VFX collision binder 说明 contact point/normal 是 typed event consumer；Editor preview 必须使用 Runtime 的 contact identity/precision，而不是重新从 DTO 估算。

## 9. 目标 owner 与 hard cutover

Editor 只拥有 Physics source document、transaction、PreviewSession、UI/viewport projection、cook orchestration 和 diagnostic display；Runtime167 拥有 world/solver/query/event truth；Plugins12 拥有 provider/package/catalog/dist closure。必须删除：Physics 未就绪时显示完整 Space surface、OpenView 假命令、固定 sample feedback、Editor 私自重算 contact/overlap/mass、String bone identity、没有 source revision 的 bake commit。

建议类型：`PhysicsAuthoringDocument`、`PhysicsAssetArtifact`、`PhysicsPreviewSession`、`PhysicsCookJob`、`PhysicsOverlayProvider`、`PhysicsDiagnosticSnapshot`、`PhysicsEditorOperationReceipt`。所有类型必须携带 source revision、provider/backend generation、target identity、permission、progress 和 terminal disposition；不在插件里复制通用 Document/Job/Viewport authority。

## 10. 依赖顺序与资格门

1. **E0 truth**：Editor catalog/provider closure、false-ready、OpenView 命令、Space placeholder 和 fixed Workbench feedback RED tests。
2. **E1 source**：Material/CollisionProfile/Shape/PhysicsAsset/Ragdoll document、typed inspector、stable IDs、transaction/save/reopen。
3. **E2 cook/preview**：Runtime artifact compiler、DDC/residency、PreviewSession、0..N fixed substep、reset/apply/revert。
4. **E3 overlay/diagnostics**：真实 `ViewportOverlayProvider`、generation-bound draw/pick/filter/capture 和 per-world metrics。
5. **E4 qualification**：plugin disable/reload、stale job、fault、large authoring、cross-platform and correctness-first competitor corpus。

当前 Editor Physics 资格门全部 Fail：没有 provider closure、document/controller、cook artifact、PreviewWorld、overlay provider 或真实 runtime capture。只有 Editor core 的通用 overlay retirement/cleanup 与 Document/Job 基础可作为 Partial foundation，不能转化为 Physics 已完成。

## 11. 本轮边界

本轮只新增 review 文档及索引/覆盖记录，不修改 Editor/Rust/Cargo/manifest/ABI/ZUI。Plugin Editor 当前没有本轮归属的 production 修改；仓库其他 Editor/App 文件存在大量 working-tree 变更，实施前必须针对 Physics selected set 重新取 fingerprint，并由 Editor02/03/05/09/59 与 Runtime167/Plugins12 先完成底层 owner。Tooling 按用户要求排除，未来 Rust 迁移另立报告。
