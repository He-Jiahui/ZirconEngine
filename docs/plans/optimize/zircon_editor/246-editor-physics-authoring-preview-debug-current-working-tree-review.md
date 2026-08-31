---
title: Editor Physics Authoring / Inspector / Cook / Preview / Overlay / Diagnostics 当前工作树复审
category: zircon_editor
report_id: Editor246
review_date: 2026-08-30
baseline_head: working-tree
baseline_epoch: 2026-08-30
verification_head: working-tree
verification_epoch: 2026-08-30
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/227-editor-physics-current-working-tree-authoring-preview-overlay-ragdoll-review.md
  - docs/plans/optimize/zircon_editor/140-editor-physics-material-rigidbody-collider-joint-collision-profile-cook-ragdoll-debug-preview-current-source-review.md
related_runtime_owner:
  - docs/plans/optimize/zircon_runtime/186-runtime-physics-backend-shape-query-event-lifecycle-current-working-tree-review.md
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
  - zircon_plugins/physics/runtime
  - zircon_editor/src/core/asset/type_registry
  - zircon_editor/src/core/document
  - zircon_editor/src/core/editing
  - zircon_editor/src/core/jobs
  - zircon_editor/src/scene/viewport
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/simulation/workbench_extension_physics_collision_workspace.zui
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/PhysicsAssetEditor
  - dev/UnrealEngine/Engine/Source/Editor/StaticMeshEditor/Private/StaticMeshAutomationTests.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/PhysicsEngine/PhysicsAsset.h
  - dev/godot/editor/scene/3d/physics
  - dev/godot/editor/scene/3d/physics/physical_bone_3d_editor_plugin.cpp
  - dev/Fyrox/editor/src/plugins/collider
  - dev/Fyrox/editor/src/plugins/inspector/editors/mod.rs
  - dev/bevy/crates/bevy_time/src/fixed.rs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Runtime/Utilities/EventBinding/Implementation/VFXRigidBodyCollisionEventBinder.cs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
tooling_scope: excluded_by_user_request
---

# Editor246 · Physics Authoring 与 Preview 复审

## 1. 结论

Editor227 已确认 Physics 的四份插件 ZUI 是 Space 壳、Workbench 具有固定反馈、Ragdoll create/debug 只是 OpenView，且没有 provider/document/cook/preview/overlay 产品链。本轮重新扫描 plugin editor registration、overlay/ragdoll source、所有 physics ZUI、Workbench physics collision workspace、Editor callback feedback/navigation/binding、asset type registry、runtime diagnostics projection、first-party catalogs 与 viewport 入口后，判断仍为产品未完成。

当前 Editor 具有可保留的通用底座：Physics Material `ResourceKind`、asset browser filter、plugin descriptor/capability、ZUI navigation/binding 骨架、Scene RigidBody/Collider/Joint reflection、通用 Document/Transaction/Job framework，以及 `build_physics_overlay` 的纯数据转换。这些都不能证明 Physics authoring 成立，因为没有 operation factory、document controller、runtime PreviewWorld、cook artifact、provider overlay registration 或 terminal receipt。

本轮新增的 Editor 证据：

- `zircon_plugins/physics/editor/src/plugin.rs` 仅注册 view/template/command descriptor。Debug toggle 与 Ragdoll create 两个 command 都是 `MenuAction::OpenView`，没有 operation factory、document mutation、job、save、cook 或 preview session。
- 四份 plugin ZUI 共仍为 4 个 view、17 个节点、11 个 `Space`；0 个 Button、0 个显式 event、0 个 route。它们不能 author body/collider/joint/material/profile，也不能显示真实 runtime state。
- `overlay.rs` 只是 `PhysicsWorldSyncState -> Vec<PhysicsOverlayPrimitive>` 的 clone/map，production 没有 `ViewportOverlayProviderRegistration`、scene mode、frame identity、pick/filter 或 stale cleanup。
- `ragdoll_profile_editor.rs` 根据输入字符串骨路径和局部 translation 猜 capsule 半径/长度，返回内存 `RagdollProfile`；无 skeleton asset dependency、PhysicsAsset package、stable bone ID、transaction、preview body、cook artifact 或 reload receipt。
- Workbench physics collision workspace 的 root `visibility = "collapsed"`，内容静态写死 `Body_PlayerCapsule`、`Material_Ice`、`4 manifolds`、`82 kg`、`124 bodies / 32 contacts / 1 warning`；feedback 文件返回同样固定字符串，change/submit route 没有 document mutation。
- Editor runtime diagnostics 可以显示 `Physics: jolt (Ready, 120 Hz)`，但它只投影 runtime diagnostics DTO；没有把 provider qualification、world generation、query precision、event overflow 或 native fault 绑定到 Physics UI。
- first-party Editor catalog 仍只分发 Navigation/Neural；Physics plugin 即使存在 descriptor，也没有默认 Editor Host activation。Runtime catalog 同样没有 Physics provider。

因此本报告不新增 P0，继承 Editor227 的 provider/catalog/operation/document/preview/overlay P0；新增 **28 项 P1、10 项 P2、25 个资格门**。P1 判定 **24 Open、4 Partial、0 Closed**；P2 **10 Open、0 Partial、0 Closed**；资格门 **24 Fail、1 Partial、0 Pass**。Editor 不得用固定 Workbench 文案或 synthetic overlay 宣称 Physics 可用。

## 2. 审查范围

扫描范围包括：Physics plugin editor 9 个 Rust/ZUI/manifest 文件；Workbench physics collision workspace 及 index/host；callback feedback/navigation/template binding；Editor physics asset kind/inspector/viewport/runtime diagnostics；Scene document/transaction/job/extension store 相关 Physics consumer；Runtime Physics contracts 和 Runtime186 owner。Tooling 排除。

参考 Unreal PhysicsAsset Editor/StaticMesh collision automation、Godot physical bone/physics gizmo、Fyrox collider inspector、Bevy fixed clock 与 Unity VFX collision event consumer，重点核对 source document、Undo/Redo、preview runtime、cook artifact、overlay identity 和 event truth。

## 3. 当前 Editor 链路事实

| surface | 当前事实 | 判定 |
|---|---|---|
| Catalog | `first_party_editor_catalog/src/catalog.rs` 只分发 Navigation/Neural，Physics 没有默认 provider | 继承 P0 |
| Plugin | authoring/diagnostics/debug/ragdoll view descriptor 与 capability 存在 | descriptor foundation |
| Commands | debug toggle/ragdoll create 均 `OpenView`，无 factory/executor | P1-003 |
| ZUI | authoring/debug/diagnostics/ragdoll 为 Space 壳，0 controls/events/routes | P1-001 |
| Scene Inspector | Runtime reflection 只暴露部分 RigidBody fields；无 Physics variant/multi-shape/joint profile editor | P1-006 |
| Material | Asset browser 能过滤 PhysicsMaterial；没有 material toolkit/preview/cook validation | P1-007 |
| Overlay | pure mapper 存在，provider registration、frame/pick/stale cleanup 为 0 | P1-011..014 |
| Ragdoll | string bone generator + default capsules，production caller/asset controller 为 0 | P1-015..018 |
| Workbench | physics collision workspace collapsed，静态 body/material/contact/solver/mass 文案 | P1-019..022 |
| Diagnostics | DTO projection 可显示 backend/state/fixed Hz，但无 qualified world/capacity/event identity | P1-023 |

## 4. 继承边界

Editor227 的 ED-P0-01..05 继续 Open：Physics 未进入 Editor Host catalog/provider closure；没有 operation factory/document controller/cook job/PreviewWorld；ZUI/Workbench 以 OpenView 和固定反馈伪造成功；overlay provider 不存在；Ragdoll profile 不是可保存 PhysicsAsset。Runtime186 的 backend/query/event/constraint 缺口是 Editor Preview/Diagnostics 的硬依赖，Editor 不得复制或估算这些事实。

## 5. P1 差异与重构要求

| ID | 状态 | 当前源码证据 | 必须重构 |
|---|---|---|---|
| ED4-P1-001 | Open | 四份 plugin ZUI 共有 11 个 `Space`，没有 Button/event/route | 用 typed Physics authoring widgets：body/collider/joint/material/profile CRUD、validation state、capability-aware disabled state |
| ED4-P1-002 | Open | `physics_collision_workspace.zui` root `visibility = "collapsed"`，workspace 不由 Physics session 控制 | Workspace mount 绑定 Editor extension generation、document/world target 与 first-present receipt |
| ED4-P1-003 | Open | debug toggle 与 ragdoll create command 只 dispatch `OpenView` | 建立 `PhysicsEditorOperationFactory`，每次调用有 preflight、permission、DocumentKey、transaction/job ID 和 terminal receipt |
| ED4-P1-004 | Open | Workbench feedback 返回固定 `Simulation queued 124 bodies 32 contacts` 等字符串 | 删除静态反馈；从 runtime/cook job snapshot 投影 progress, result, artifact, warning, stale disposition |
| ED4-P1-005 | Open | navigation/binding route 只切 tabs/rows 或返回 feedback，未进入 Document/Scene mutation | route -> typed command -> document transaction -> runtime sync；失败不得更新 selected/output row |
| ED4-P1-006 | Partial | Runtime reflection 有 RigidBody mass/velocity/CCD/sleep 部分 read/write，但 body type/locks 与 collider/joint shape 无统一 editor schema | Physics field catalog 覆盖 all source fields、units/ranges/unsupported/restart requirement，multi-selection semantics 明确 |
| ED4-P1-007 | Open | AssetTypeRegistry 只把 PhysicsMaterial 映射到通用 asset presentation；无 material toolkit/import/cook preview | `PhysicsMaterialDocument` + schema validator + dependency/cook job + preview of friction/restitution/combine |
| ED4-P1-008 | Open | Scene component schema 一个 node 一个 collider，compound children 是匿名 tuple | Editor source document 使用 stable shape/subshape IDs、multi-shape list、local TRS/material/filter editing |
| ED4-P1-009 | Open | layer/group/mask/collision matrix 只以 raw integer/strings 出现在 Runtime settings，Editor 没有 matrix authoring/validation | profile document + matrix UI + pair conflict diagnostics + runtime artifact generation |
| ED4-P1-010 | Open | joint editor 没有 local frames/limit/drive/motor/break/projection authoring surface | typed joint inspector with preview constraints and runtime-supported capability matrix |
| ED4-P1-011 | Open | `build_physics_overlay` 只 clone collider shape/transform/color | `PhysicsOverlayProvider` 绑定 viewport/frame/world/provider generation、filter, pick IDs, stale retirement and draw budget |
| ED4-P1-012 | Open | plugin editor registration 没有 scene mode/viewport overlay registration；旧 failure handoff仍Open | register/revoke provider atomically with extension store and test disable/reload/duplicate cleanup |
| ED4-P1-013 | Open | overlay primitive 没有 body/subshape/material/event identity，无法显示 contact/manifold/CCD/sleep states | overlay DTO versioning with stable IDs, debug layer, query/capture cursor and backend precision |
| ED4-P1-014 | Open | overlay 没有 pick/raycast source，Editor 不能把 wireframe selection 绑定 Scene selection | use Runtime query ticket + displayed frame identity; pick result has stale/unsupported/overflow disposition |
| ED4-P1-015 | Open | `generate_initial_ragdoll_profile` 接收 `String` bone path，按 local translation 猜 capsule；无 asset dependency | skeleton asset resolver + stable bone GUID/path migration + PhysicsAsset document and source revision |
| ED4-P1-016 | Open | ragdoll profile is in-memory `RagdollProfile`; creation template points back to `.zui` view | real asset codec/importer/cook/artifact/thumbnail/toolkit; creation command returns AssetId/document receipt |
| ED4-P1-017 | Open | no ragdoll preview world, reset/apply/revert, collision visualization or physical-animation blend | `PhysicsPreviewSession` uses Runtime fixed clock/backend and supports isolated world, reset, apply, cancel, save |
| ED4-P1-018 | Open | no Ragdoll profile delete/rebuild/despawn or skeleton reload semantics | stable mapping migration, transactional rebuild, stale job cancellation and orphan body retirement |
| ED4-P1-019 | Open | Workbench static list names Player Capsule/Ice/Wall and fixed 82 kg/4 manifolds | project live Scene/PhysicsAsset selection; values must carry source/world generation and absent-state text |
| ED4-P1-020 | Open | solver dropdown offers PGS/TGS/XPBD/Debug despite Runtime only builtin/Jolt settings | derive options from provider capability; commit creates validated physics profile, not a string |
| ED4-P1-021 | Open | mass field displays `82 kg`, no unit system/COM/inertia/auto-density provenance | show authored vs resolved mass, density, COM/inertia artifact and backend unsupported diagnostics |
| ED4-P1-022 | Open | simulate/validate buttons have no JobId, cancellation, progress, artifact or terminal state | Background Job owner with bounded work, preview world, result artifact, cancellation and failure receipt |
| ED4-P1-023 | Partial | runtime diagnostics text can display backend/state/fixed Hz but hides feature gate, world generation, capacity and query precision | structured PhysicsDiagnosticSnapshot with provider/artifact/world/step/query/event/fault counters and frame identity |
| ED4-P1-024 | Open | Editor has no physics capture/scrub/replay/state diff route; callback feedback cannot reproduce a frame | versioned capture session with deterministic replay oracle, timeline cursor, world generation and diff navigation |
| ED4-P1-025 | Open | Editor Preview would need to estimate Jolt contacts because Runtime Jolt query/events are empty/fallback | Preview consumes Runtime result only; unsupported/approximate state is visibly typed and cannot be committed as authoritative |
| ED4-P1-026 | Open | scene document/save integration has no PhysicsAsset participant or expected revision/CAS for cook commit | include Physics source/artifact in Document participants, savepoint, recovery, reference repair and CAS commit |
| ED4-P1-027 | Open | no physics-specific inspector multi-selection, prefab override or variant policy | define field-level merge/override for bodies/shapes/material/profile and preserve stable IDs through prefab instantiation |
| ED4-P1-028 | Open | no authoring scale/performance UX: shape count, cook memory, solver budget, preview step cost absent | show bounded budgets/high-water/estimated cook and gate large scenes before preview/cook |

## 6. P2 差异

| ID | 当前问题 | 重构方向 |
|---|---|---|
| ED4-P2-001 | material UI 没有 combine rule explanation/migration | schema versioned material inspector and migration report |
| ED4-P2-002 | collider shape fields 无 units/limits/degenerate geometry preview | shared units/range validator and visual invalid-state overlay |
| ED4-P2-003 | no collision matrix conflict search or pairwise test command | deterministic pair test with query/event receipt |
| ED4-P2-004 | no mass/COM/inertia visualization | preview gizmo tied to cooked artifact |
| ED4-P2-005 | no CCD/sleep/debug policy visualization | runtime policy overlay and unsupported-state display |
| ED4-P2-006 | no world replacement/session close retirement for Physics preview | PreviewSession lease/fence and teardown report |
| ED4-P2-007 | no asset dependency graph for mesh/heightfield Physics cook | dependency panel, stale/rebuild reason and DDC key |
| ED4-P2-008 | no remote/PIE physics inspector with read-only permission boundary | generation-bound remote snapshot and command capability gate |
| ED4-P2-009 | no accessibility/localization keys for Physics-specific fields/status | typed localization/a11y metadata in contribution catalog |
| ED4-P2-010 | no correctness/performance comparison report against reference corpus | capture benchmark report with correctness gate before timings |

## 7. 资格门

| Gate | 验收条件 | 当前 |
|---|---|---|
| ED4-G01 | Physics provider is selected by default Editor Host catalog or surface is unavailable | Fail |
| ED4-G02 | opening Physics view requires live document/world/session receipt | Fail |
| ED4-G03 | all Physics commands have factory, preflight, transaction/job and terminal receipt | Fail |
| ED4-G04 | no Physics workspace publishes fixed sample feedback | Fail |
| ED4-G05 | body/collider/joint/material/profile edits round-trip through document save/reopen | Fail |
| ED4-G06 | stable shape/subshape/bone IDs survive reorder/prefab/reload | Fail |
| ED4-G07 | local transform, scale, filter and material edits are visible in preview runtime | Fail |
| ED4-G08 | collision matrix/solver/CCD/sleep options derive from qualified provider capabilities | Fail |
| ED4-G09 | material/mesh/heightfield cook produces artifact, dependency record and diagnostics | Fail |
| ED4-G10 | Ragdoll creation yields PhysicsAsset artifact, not an in-memory profile or view | Fail |
| ED4-G11 | Ragdoll preview uses Runtime fixed substeps and supports reset/apply/revert/cancel | Fail |
| ED4-G12 | overlay is a registered provider with frame/world generation, pick and stale cleanup | Fail |
| ED4-G13 | overlay/contact display uses Runtime event identity and precision, never Editor estimates | Fail |
| ED4-G14 | simulate/validate jobs are bounded, cancellable and retain terminal receipt | Fail |
| ED4-G15 | diagnostics expose provider/artifact/world/step/query/event/fault identity | Fail |
| ED4-G16 | capture/scrub/replay reproduces a qualified Physics frame | Fail |
| ED4-G17 | Physics document participates in Save All/autosave/recovery/CAS | Fail |
| ED4-G18 | prefab/multi-selection overrides preserve per-field semantics and stable IDs | Fail |
| ED4-G19 | plugin disable/reload revokes views, commands, overlay and jobs without leaks | Fail |
| ED4-G20 | large scene authoring reports shape/cook/solver budgets before work starts | Fail |
| ED4-G21 | unsupported backend/shape/query appears as typed unavailable, not empty success | Fail |
| ED4-G22 | PIE/remote inspector is read-only or capability-gated by world generation | Fail |
| ED4-G23 | accessibility/localization metadata exists for all Physics controls/status | Partial |
| ED4-G24 | editor/runtime Physics artifact and preview output pass correctness corpus | Fail |
| ED4-G25 | benchmark compares same scene/cook/tick/hardware only after correctness passes | Fail |

## 8. 参考引擎差异

- Unreal PhysicsAsset Editor 的核心不是一个 view，而是 package-backed body/constraint/shape authoring、transaction/Modify、mesh invalidation、preview scene、physical animation 与 save/cook。Zircon Ragdoll profile 当前只是 string path generator。
- Godot physics gizmo/physical bone editor 将 scene node、shape/joint、skeleton bone、debug draw 与 Undo/Redo 直接连接，并显式处理 editor/physics-server separation；Zircon 的 four Space templates 没有对应 owner。
- Fyrox collider inspector 将 Rapier collider handle、sensor、friction/restitution/groups 与 scene graph 保存绑定；Zircon Inspector 没有 Physics variant/multi-shape/matrix semantics。
- Bevy fixed clock 说明 Preview 必须调用 Runtime 同一 fixed schedule；不能另建一个只改变 label 的 Editor simulation clock。
- Unity VFX collision event binder 依赖 typed contact data；Editor debug/preview 必须消费 Runtime 的 event identity/precision，而非从 `PhysicsWorldSyncState` 二次估算。

## 9. 重构顺序与 owner

1. 由 Plugins12/Runtime186 先完成 Physics provider/catalog、backend truth、single clock、query/event/constraint contract；Editor 只显示真实 availability。
2. Editor02/03/05/09/59 提供 document/transaction/inspector/job/viewport owner，Physics plugin 只实现 `PhysicsAuthoringDocument`、`PhysicsAssetArtifact`、`PhysicsPreviewSession`、`PhysicsCookJob` 和 `PhysicsOverlayProvider`。
3. 将 Material/CollisionProfile/Shape/Joint/PhysicsAsset/Ragdoll 统一到 source revision + stable IDs + cook artifact；所有 edits 通过 transaction/save/reopen。
4. Preview/overlay/diagnostics/capture 绑定 Runtime world/provider/artifact generation，并覆盖 disable/reload/stale/fault/PIE/remote cases。
5. 完成 large authoring、deterministic replay、correctness-first competitor corpus 后，才允许在 Workbench 显示性能或 solver quality 结论。

Editor246 只写 review 与重构合同，没有修改 Editor/Rust/Cargo/ZUI。Tooling 迁移按用户要求另立报告。
