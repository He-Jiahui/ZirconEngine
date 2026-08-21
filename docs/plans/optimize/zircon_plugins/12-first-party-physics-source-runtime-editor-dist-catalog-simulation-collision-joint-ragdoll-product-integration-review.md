---
title: First-Party Physics Source、Runtime、Editor、Dist、Catalog、Simulation、Collision、Joint、Ragdoll 与 Product Integration 工程化差距
category: zircon_plugins
report_id: Plugins12
review_date: 2026-08-19
baseline_head: 25e09a23178000f2e783ce2143cf70a8b118d404
baseline_epoch: 333
related_code:
  - zircon_plugins/physics/plugin.toml
  - zircon_plugins/physics/runtime/Cargo.toml
  - zircon_plugins/physics/runtime/src
  - zircon_plugins/physics/runtime/tests
  - zircon_plugins/physics/editor/Cargo.toml
  - zircon_plugins/physics/editor/src
  - zircon_plugins/physics/editor/authoring.zui
  - zircon_plugins/physics/editor/debug_overlay.zui
  - zircon_plugins/physics/editor/diagnostics.zui
  - zircon_plugins/physics/editor/ragdoll_profile.zui
  - zircon_plugins/physics/dist/Cargo.toml
  - zircon_plugins/physics/dist/src
  - zircon_plugins/first_party_runtime_catalog/Cargo.toml
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/first_party_editor_catalog/Cargo.toml
  - zircon_plugins/first_party_editor_catalog/src
  - zircon_app/Cargo.toml
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/first_party_editor_plugins.rs
  - zircon_app/src/entry/export_bootstrap.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/simulation_physics.rs
tests:
  - zircon_plugins/physics/runtime/src/tests.rs
  - zircon_plugins/physics/runtime/src/backend/tests
  - zircon_plugins/physics/runtime/src/manager/tests.rs
  - zircon_plugins/physics/runtime/src/skeletal/tests.rs
  - zircon_plugins/physics/runtime/tests/physics_manager_runtime_contract.rs
  - zircon_plugins/physics/runtime/tests/physics_manager_runtime_contract
  - zircon_plugins/physics/editor/src/tests.rs
  - zircon_plugins/physics/dist/src/lib.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
  - docs/plans/optimize/zircon_runtime/08a-physics-runtime-review.md
  - docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/42-builtin-runtime-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-review.md
  - docs/plans/optimize/zircon_editor/18-physics-material-rigidbody-collider-joint-collision-profile-cook-ragdoll-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/25-runtime-diagnostics-performance-timeline-console-telemetry-observability-authoring-review.md
  - docs/plans/optimize/zircon_editor/50-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-product-integration-review.md
  - docs/plans/zircon_plugins/03-physics.md
  - docs/plans/zircon_plugins/03/failure-2026-08-01-physics-debug-overlay-provider-missing.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/PhysicsEngine/BodyInstance.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/CollisionQueryParams.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/WorldCollision.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Collision/WorldCollision.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Collision/WorldCollisionAsync.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Experimental/Chaos/Public/PBDRigidsSolver.h
  - dev/UnrealEngine/Engine/Source/Editor/PhysicsAssetEditor
  - dev/UnrealEngine/Engine/Source/Developer/PhysicsUtilities/Private/PhysicsAssetUtils.cpp
  - dev/UnrealEngine/Engine/Source/Programs/HeadlessChaos/Private/HeadlessChaosTestRaycast.cpp
  - dev/UnrealEngine/Engine/Source/Programs/HeadlessChaos/Private/HeadlessChaosTestConstraints.cpp
  - dev/UnrealEngine/Engine/Source/Programs/HeadlessChaos/Private/HeadlessChaosTestPerf.cpp
  - dev/godot/servers/physics_3d/physics_server_3d.h
  - dev/godot/servers/physics_3d/direct_states/physics_direct_space_state_3d.h
  - dev/godot/servers/physics_3d/physics_server_3d_wrap_mt.h
  - dev/Fyrox/fyrox-impl/src/scene/graph/physics/mod.rs
  - dev/Fyrox/fyrox-impl/src/scene/collider.rs
  - dev/Fyrox/fyrox-impl/src/scene/rigidbody.rs
  - dev/Fyrox/fyrox-impl/src/scene/joint.rs
  - dev/bevy/crates/bevy_time/src/fixed.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/package.json
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 12 · First-Party Physics Source、Runtime、Editor、Dist、Catalog、Simulation、Collision、Joint、Ragdoll 与 Product Integration 工程化差距

## 1. 结论

`zircon_plugins/physics` 不是空壳。它已经具备中立 body/collider/joint/material/query/event DTO、fixed schedule system、按 world 的 manager、generation handle、命令容量限制、builtin 参考后端、可选 Jolt FFI 后端、场景同步、Ragdoll profile 与 skeletal pose feed。manifest 将成熟度标为 `experimental`，七项能力标为 `partial`，比把原型能力误报为完成更诚实。这些合同、输入校验、world replacement epoch 与局部 Jolt body lifecycle 可以保留。

但它还不是普通 Zircon 产品中的工程级物理系统。默认 runtime feature 不启用 Jolt；普通 App 与 first-party runtime catalog 不链接 Physics provider；first-party editor catalog 也不链接 Physics Editor。generated source export 可直接链接 Physics registration，却仍使用无 Jolt 的默认 feature；NativeDynamic dist 则是 `is_stateless`、无 command/event/state/lifecycle/bridge 的 metadata shell。普通 App、Editor Host、source export 与 NativeDynamic 因而分别得到“无 provider”“无 editor”“Disabled provider”“只有注册 metadata”四种不同产品事实。

显式启用 Jolt 也没有闭合 solver 语义。产品 fixed system 每次调用只推进一次 scheduler delta，公开的 Physics accumulator/fixed_hz/max_substeps 是另一套未接入时钟；每 tick 扫描所有 Scene node并深拷贝完整同步状态。Jolt native object layer 只有 moving/non-moving，查询方法为空，contact/trigger 重新走 builtin 的近似 pair scan，constraint 只存 DTO并在 native step 后用 Rust 投影修改 body。builtin 更只有积分、近似 overlap 和少量位置投影，没有碰撞响应、摩擦、堆叠、岛、warm start、CCD 或完整 joint solver，却可以进入 Ready 状态。

Editor 同样停留在注册层。四份 ZUI 的业务区域都是 `Space`，overlay 只把同步 collider 转成颜色 DTO，没有 viewport provider；toggle command只打开 view；ragdoll create command也只打开 view，没有生成、保存或重新打开资产。开放 failure 已明确记录 Physics debug overlay provider 缺失。Workbench 又维护一套 `PlayerCapsule`、`IceMaterial`、`WallContact`、`CcdWarning` 等演示 binding，不能成为运行时观测或 authoring truth。

Physics runtime 本体由 Runtime08A 管理，authoring/cook/debug由 Editor18 管理，catalog/native 通用缺陷由 Plugins01/06 管理。本篇不重复累计父报告的最高优先级问题，登记 **0 项新增 P0、48 项 P1、12 项 P2**。本篇唯一拥有的是 Physics 单包从 manifest、source runtime、editor、dist、catalog/profile、普通 App 到 export 的纵向交付合同，以及这些载体之间的 provider、backend、capability、lifecycle 和行为 parity。

## 2. 审查边界、规模与 currentness

### 2.1 物理冻结

| 范围 | 文件 / 行 / bytes | 冻结事实 |
|---|---:|---|
| `zircon_plugins/physics` 全包 | 93 / 13,207 / 462,852 | 85个Rust、4个TOML、4个ZUI；包内无tracked working-tree差异 |
| runtime | 78 / 12,383 / 433,244 | builtin、Jolt、manager、constraint、skeletal、系统和integration tests |
| editor | 12 / 635 / 22,371 | registration、overlay DTO、ragdoll generator、4份ZUI和4项test |
| dist | 2 / 115 / 4,298 | Native ABI v3 descriptor与registration manifest projection |
| test-path inventory | 13 / 4,091 | 全包共82项test attribute；没有Criterion、bench、property、Loom、sanitizer或soak证据 |
| package fingerprint | `ebcef0796246eb827b0d5f7a9f2bcfc2cbabfe73bd992ad93109ae7fb04015c7` | tracked path排序，以小写path加文件SHA-256的LF串、无末尾LF再计算SHA-256 |
| 产品装配 | ordinary runtime 0、editor 0、generated source 1、native behavior 0 | Runtime builtin catalog仍广告Physics row/capability，实际provider closure不一致 |

源 revision 为 `25e09a23178000f2e783ce2143cf70a8b118d404`，coordinator baseline epoch 为333。Physics包自身冻结时干净；App、Editor Workbench与共享计划存在其他会话或用户改动，所以本文按当前工作树读取并保留 `source_recheck_required`。实施前必须在同一 generation 重算 Physics package、App feature、runtime/editor catalog、profiles、builtin row、export bootstrap 与 native dist。

### 2.2 测试库存不等于产品资格

82项测试覆盖DTO校验、handle失效、命令容量、builtin primitive query、Jolt body创建、近似constraint投影、fixed plan helper、场景同步、ragdoll profile和registration shape。这是有价值的局部底座，但不能证明产品闭环。

Jolt关节测试调用 `create_constraint()` 后验证投影结果，而该方法没有创建native Jolt constraint；因此测试证明的是插件侧projection，不是Jolt joint语义。Jolt query trait的三个方法为空，产品query仍克隆manager同步状态后运行builtin geometry。dist测试只检查descriptor和registration manifest非空；Editor测试只检查ID、menu/view注册和纯overlay DTO。仓内没有native filter/listener/manifold、source/native parity、普通App provider、Editor executable factory、mesh cook、scale、100k body、multi-world、soak、fault injection或统计性能资格。

对照的 Unreal `HeadlessChaos` 单独覆盖raycast、constraint、CCD、sleep、rewind、serialization、large-scale与performance workload；Fyrox collider测试至少让sensor/non-sensor通过真实PhysicsWorld执行；Bevy fixed time测试直接验证accumulator与overstep。Zircon不能用82个局部test attribute替代这些行为层级。

### 2.3 本轮纵向追踪

1. `plugin.toml`、Cargo feature、runtime/editor/dist registration 与能力声明。
2. fixed system、manager settings、per-world state、scene sync、builtin/Jolt backend、query/event/constraint、mesh与ragdoll路径。
3. runtime/editor catalog、App feature、profile、builtin capability row、generated export与NativeDynamic行为。
4. Physics Editor command、view/drawer/template、四份ZUI、overlay、ragdoll generator和Workbench binding。
5. Runtime08A、Editor18、Plugins01/06、Runtime22/24/42及开放failure的唯一owner边界。
6. Unreal、Godot、Fyrox、Bevy适用源代码与测试；Unity Graphics只用于包边界排除说明。

本轮为E3静态源码审查。没有修改production或tests，没有运行Cargo、Jolt native build、Editor、App、export、NativeDynamic、真实场景、soak或性能测试。测试数量是源码库存，不是本轮通过数。

## 3. 当前真实产品链与断点

~~~text
ordinary zircon_app client / editor host
  -> compiles Physics contracts
  -> does not link Physics runtime provider
  -> editor catalog does not link Physics Editor

runtime builtin catalog / profiles
  -> advertise Physics row and partial capabilities
  -> client3d/editor/dev optional lists omit Physics
  -> server lists it optional, but first-party runtime catalog has no provider branch

generated source export
  -> directly links zircon_plugin_physics_runtime::plugin_registration
  -> generated dependency does not enable backend-jolt
  -> provider exists, effective backend remains unconfigured/Disabled

NativeDynamic dist
  -> exports ABI v3 descriptor and registration manifest
  -> is_stateless, empty command/event, no save/restore/unload/bridge/host-ready
  -> explicitly says world/query remain hosted by source runtime

source Physics runtime when explicitly constructed
  -> FixedUpdate scans and deep-clones Scene physics state
  -> scheduler invocation becomes exactly one physics step
  -> separate manager accumulator is not used by product system
  -> builtin is approximation; Jolt is partial native body integration

Physics Editor
  -> registration IDs exist
  -> command factories/controllers/product provider do not
  -> four business surfaces are Space placeholders
  -> debug overlay and ragdoll commands open views without executing domain work
~~~

目标不是把Jolt直接暴露成全引擎API，也不是把所有职责塞进一个全局manager。目标是由同一 `PhysicsActivationPlan` 选择backend artifact并创建 `PhysicsRuntimeInstance`；每个World拥有独立 `PhysicsWorldSlot`，scene delta、solver、query、event和debug observation都绑定同一simulation generation。Editor和NativeDynamic通过typed bridge消费同一能力事实，不再各自解释manifest字符串。

## 4. 可保留基础

| 基础 | 当前价值 | 重构约束 |
|---|---|---|
| 中立Physics合同 | body/collider/joint/material/query/event不泄漏Jolt FFI类型 | 继续由Runtime吸收层拥有；backend能力用typed receipt表达 |
| experimental/partial声明 | 没有把原型功能标成stable/complete | 所有资格门通过前保持partial/default-off |
| generation handle | shape/body/constraint handle具备generation，优于裸index | 扩展到world/backend/query/event/artifact generation和stale拒绝 |
| command预算 | 每world body command有4,096项上限和有限数校验 | 改为可配置bytes/items/time预算并给出overflow receipt |
| world replacement epoch | LevelSystem能拒绝旧world结果覆盖新world | 纳入PhysicsWorldSlot publication transaction |
| Jolt body lifecycle | primitive body创建、active state读取、部分命令与错误路径可复用 | 由真实filter/query/listener/constraint/cook补全，不保留第二投影solver |
| Ragdoll profile校验 | TOML profile、拓扑/数值校验与spawn rollback已有雏形 | 接入真实asset/import/cook/editor/runtime owner，不再生成Empty node原型 |
| Editor contribution词汇 | view/drawer/menu/template和overlay DTO有可扩展边界 | 只有真实factory/provider/document存在时才发布可见能力 |

## 5. 参考实现给出的工程边界

### 5.1 Unreal Engine / Chaos

`FBodyInstance` 将body生命周期、mass/material、CCD、sleep、DOF、collision profile、weld与async physics语义放在长期持有对象上；`FCollisionQueryParams` 明确complex、initial overlap、face index、mobility、ignore mask、trace/owner tag和stat identity，`WorldCollision`又区分trace/sweep/overlap以及sync/async buffer。Chaos solver拥有明确的advance、buffer、rewind、constraint和event边界，而不是每tick从场景重建solver输入。

PhysicsAssetEditor和PhysicsUtilities形成独立资产编辑、生成、选择、仿真、viewport interaction和detail customization体系。`HeadlessChaos`测试覆盖raycast边界、joint/constraint演进、CCD、sleep、rewind、serialization、large scale和perf。这些事实要求Zircon把solver、query、asset、editor和qualification分owner，但通过同一shape/filter/generation合同闭合。

Zircon不需要复制Unreal类数量，也不能仅因使用Jolt就假设达到Chaos工程性。只有在同一workload下先通过正确性、failure、soak和资源预算，才可比较CPU、延迟、内存或表现。

### 5.2 Godot PhysicsServer3D

Godot以RID管理shape、space、area、body、joint和soft body，明确shape local transform、collision layer/mask、CCD、force/impulse、axis lock、contact上限与callback。direct space state提供ray/point/shape/motion/collide/rest查询并由caller给出max results；MT wrapper把sync、step、flush_queries和finish的线程边界显式化。

适用结论不是复制Godot singleton，而是Physics service必须有稳定资源身份、明确thread phase、同一backend query和有界结果。Zircon当前从manager snapshot运行近似query，并不能证明与solver空间一致。

### 5.3 Fyrox PhysicsWorld

Fyrox `PhysicsWorld` 长期持有Rapier sets、pipeline、query pipeline、integration parameters与event collector；collider/rigidbody通过dirty标记同步，而不是每tick深拷贝整个scene。ray cast允许 `Vec` 或 `ArrayVec` 实现caller-owned `QueryResultsStorage`，可排序且可避免运行时分配；intersection还保留collider handle、normal、position、feature和TOI。其collider测试让sensor与solid通过真实world更新后验证contact/intersection差异。

Fyrox规模小于Unreal，但给出了工程下界：长期world、dirty sync、真实query、caller容量和场景生命周期必须先成立。Zircon当前“DTO更多、产品执行更少”不构成领先。

### 5.4 Bevy Fixed Time

Bevy `Time<Fixed>` 的timestep、accumulator和overstep是Fixed schedule的单一时钟权威；测试直接验证accumulate、expend、discard和timestep变化。Zircon Runtime scheduler已经能按fixed plan运行零次或多次，Physics manager不应再持有第二套game-time accumulator。

PhysicsSettings可以保留solver substep/iteration/quality policy；若未来需要异步physics frequency，必须建立独立clock domain、simulation generation和interpolation bridge，而不是让同一个API同时表示scheduler tick与solver substep。

### 5.5 Unity Graphics参考边界

本地 `dev/Graphics` 是Unity Scriptable Render Pipeline与Graphics package镜像，不含Unity Physics或完整Editor物理源码。它只可证明runtime/editor package边界应显式，不能为Physics solver、query、joint、ragdoll或authoring提供完成证据。为满足引擎名单而从Graphics推导Physics结论会制造错误类比，因此本篇明确排除。

## 6. P0归属：本文不新增最高优先级finding

| 已证实现象 | Canonical owner | 本篇责任 |
|---|---|---|
| fixed clock、scene full sync、backend/query/event/constraint/cook/ragdoll运行时本体 | Runtime08A、Runtime22/24 | 记录它们如何阻断Physics package纵向闭环，不复制P0 |
| Physics material/body/collider/joint/profile/cook/ragdoll/debug authoring | Editor18、Editor25/50 | 记录本包command/ZUI/provider/catalog断点 |
| debug overlay无可执行provider | `docs/plans/zircon_plugins/03/failure-2026-08-01-physics-debug-overlay-provider-missing.md` | 保持open并纳入G27，不用假provider关闭 |
| source/dist/native ABI与lifecycle parity | Plugins01 | 定义Physics parity gate，不重造通用loader/ABI P0 |
| first-party catalog/profile/capability closure | Plugins06、Runtime42 | 记录ordinary runtime/editor为0的单包影响 |
| App target/provider composition | App与O00/O01/O02/O11/O14 | 要求Physics消费共享activation/evidence合同 |

只要Physics保持experimental/partial、生产profile不把builtin approximation当完整solver、缺provider/无Jolt/无editor时所有入口fail-close，本篇不因功能量差距新增P0。任何release将Physics标为stable/complete/required/default-enabled前，必须先关闭父owner硬阻塞并通过本篇G01-G32。

## 7. P1：Package、Catalog、Capability 与 Distribution闭环

| ID | 当前差距 | 需要重构 |
|---|---|---|
| NPHY-P1-001 | ordinary App不链接Physics runtime provider，合同可存在但service不可解析 | 由project/profile生成PhysicsActivationPlan；required provider缺失时build/startup fail-close |
| NPHY-P1-002 | Editor Host不链接Physics runtime或Physics Editor，preview/authoring无产品owner | Editor与game/export消费同一activation closure，仅允许显式target policy差异 |
| NPHY-P1-003 | first-party runtime catalog没有Physics dependency和registration branch | 用生成provider graph替代手写遗漏，并验证manifest package到linked registration一一对应 |
| NPHY-P1-004 | first-party editor catalog只链接少量包，对Physics为0 | package selection同时解析runtime/editor closure；缺provider时隐藏入口并返回typed原因 |
| NPHY-P1-005 | Runtime builtin catalog广告Physics row/capability，实际linked provider可能为0 | capability发布必须消费ActivationReceipt，区分declared、linked、admitted、active、degraded |
| NPHY-P1-006 | client3d/editor/dev optional列表省略Physics，server虽列optional但catalog仍无provider | profile只引用可解析package；CI重建每target的provider closure和缺失原因 |
| NPHY-P1-007 | generated export直接链接registration，形成强于ordinary host的第二composition authority | ordinary/source/generated/library/native共用ProviderResolver与相同selection lock |
| NPHY-P1-008 | generated依赖未启用`backend-jolt`，显式provider默认仍是unconfigured/Disabled | backend选择进入target build plan与artifact identity，不允许运行时静默得到Disabled |
| NPHY-P1-009 | runtime default feature为空，editor/dist也不传播Jolt feature | 建立client/server/editor/export/native target-backend matrix和unsupported-target admission |
| NPHY-P1-010 | manifest options、PhysicsSettings与实际backend artifact没有一个effective config receipt | 生成ValidatedPhysicsConfig，记录source、override、range、backend build和applied generation |
| NPHY-P1-011 | NativeDynamic dist是stateless metadata shell，world/query仍要求source runtime | 实现等价service bridge、state/lifecycle/quiesce，或撤销NativeDynamic行为可用声明 |
| NPHY-P1-012 | dist command/event/state/save/restore/unload/host-ready/bridge均为空 | 为source可观察行为定义ABI projection；不能表达的能力显式Unsupported并从capability移除 |
| NPHY-P1-013 | native registration只有module，systems/events为空，无法安装fixed execution和event流 | projection必须携带可执行system/service/event factory及generation receipt |
| NPHY-P1-014 | root/raycast/overlap/shape_cast/trigger/constraint/skeletal七项partial没有backend级支持矩阵 | 每项声明native/reference/approximate/unsupported和query/event/shape限制 |
| NPHY-P1-015 | source、export、native、editor可以对同一项目给出不同Ready/Unavailable结论 | 建立跨载体scenario corpus，逐项比较registration、backend、capability、error和lifecycle |
| NPHY-P1-016 | package diagnostics只显示requested/active/state/fixed_hz，无法证明真实world与provider | 输出build/backend/world/generation/object/query/event/capacity/cook和degraded reason |

## 8. P1：World、Solver、Query、Event 与 Constraint闭环

| ID | 当前差距 | 需要重构 |
|---|---|---|
| NPHY-P1-017 | Runtime scheduler与manager accumulator形成两套fixed clock authority | 由Runtime fixed schedule拥有game tick；PhysicsSettings只拥有solver substep/iteration，旧clock硬切 |
| NPHY-P1-018 | product `tick_scene_world`每次只推进一步完整scheduler delta，忽略fixed_hz/max_substeps | 将scheduler plan或显式PhysicsClock plan作为唯一输入，记录dropped/overstep/quality receipt |
| NPHY-P1-019 | settings先改内存再持久化，失败非原子；非backend变化不更新既有Jolt world | validate-prepare-persist-publish-retire，world记录desired/applied config generation |
| NPHY-P1-020 | 每fixed tick扫描全部node并深拷贝body/collider/joint/material | 建立per-world增量scene delta、stable handle table和dirty hierarchy frontier |
| NPHY-P1-021 | manager用多把全局`Mutex<HashMap<WorldHandle,...>>`发布不同代状态 | 每world单一PhysicsWorldSlot拥有lifecycle/solver/query/event/config，step后原子发布immutable view |
| NPHY-P1-022 | Jolt world全局map锁覆盖sync和native update，不相关world串行 | slot级并行调度，global registry只做短时lookup，world step有deadline和health状态 |
| NPHY-P1-023 | poison recovery取inner继续运行，不能证明native/world/event一致 | poison/native fault将slot标记Faulted，保留诊断并走显式recreate/LKG流程 |
| NPHY-P1-024 | builtin只有积分与近似pair scan，却可报告Ready | 重命名为ReferenceApproximation/TestFallback；production profile拒绝其完整solver资格 |
| NPHY-P1-025 | builtin没有collision impulse、penetration、friction/restitution、stack、island、sleep或CCD | 不扩成第二生产solver；仅保留有界oracle能力并精确降级声明 |
| NPHY-P1-026 | Jolt native filter只有moving/non-moving，忽略layer/group/mask/matrix | 编译CollisionFilterGeneration并接入object/broadphase/filter callback、query、event和debug |
| NPHY-P1-027 | Jolt backend的ray/shape cast/overlap实现为空，manager绕回builtin线性扫描 | backend提供persistent broad/narrow query view；fallback结果必须标记Approximate |
| NPHY-P1-028 | query每次clone整份world并返回新Vec，filter排除列表线性contains | 提供caller-owned/reused buffer、compiled filter、batch/async ticket、bytes/items/time预算 |
| NPHY-P1-029 | query `First`依赖遍历顺序，缺Any/Closest/AllSorted、tie-break、overflow和generation | 定义query mode、stable ordering、snapshot generation、max results与overflow receipt |
| NPHY-P1-030 | primitive query忽略旋转/scale，cylinder/convex/shape cast多用AABB proxy，mesh类不支持 | solver/query/debug共享cooked shape与subshape identity，建立transform/scale oracle |
| NPHY-P1-031 | contact/trigger由同步DTO做近似pair scan，不来自Jolt listener/manifold | 采集native contact/sensor callback，映射stable entity/shape/subshape/material identity |
| NPHY-P1-032 | contact缺Begin/Persist/End、impulse、penetration、relative velocity；event无容量/overflow | 建立有界双缓冲、pair lifecycle、排序、coalesce、overflow telemetry和consumer cursor |
| NPHY-P1-033 | Jolt `create_constraint`只存descriptor，step后运行Rust projection并写回body | 创建真实native Jolt constraint；删除生产projection solver，unsupported参数admission拒绝 |
| NPHY-P1-034 | joint anchor用同一translation构造两帧，orientation、collide_connected、break和motor语义不完整 | authoring/cook产生body-local frame A/B，backend实现limit/motor/break/projection完整映射 |
| NPHY-P1-035 | mesh/heightfield按triangle塞入static compound，无cook、DDC、streaming或production注册caller | 建立CollisionArtifact compiler、platform cook、validation、cache、residency和last-good |
| NPHY-P1-036 | native capacity/temp allocator/thread pool为硬编码单world预算，scale和失败策略不可配置 | target/workload配置budget，pre-admission容量，暴露high-water/OOM/update error并做scale qualification |

## 9. P1：Scene、Ragdoll、Editor 与 Qualification闭环

| ID | 当前差距 | 需要重构 |
|---|---|---|
| NPHY-P1-037 | sanitize会静默丢弃无效/重复body/collider/joint/material，Scene回写错误也被`let _ =`吞掉 | admission返回逐entity typed diagnostic；step transaction记录partial/stale并禁止伪成功 |
| NPHY-P1-038 | body/collider默认按同entity一一对应，compound/local shape/multiple collider ownership不完整 | 建立BodyId、ShapeInstanceId、local pose、material slot和compound hierarchy稳定模型 |
| NPHY-P1-039 | mass/material字段比backend执行更宽，static friction/combine/scale等未完整进入native | artifact/backend/query/debug共用material/mass generation并做端到端字段矩阵 |
| NPHY-P1-040 | ragdoll generator只按local translation估capsule，固定mass、identity offset和default constraint | 基于skeleton bind pose、mesh/weight、body heuristic和asset policy生成可编辑proposal与诊断 |
| NPHY-P1-041 | ragdoll用字符串bone path、leaf fallback，歧义时丢失映射 | 使用stable skeleton/bone identity、schema version、retarget mapping和unknown preservation |
| NPHY-P1-042 | ragdoll spawn创建Empty node和runtime resource，没有正式asset/import/cook/provider生命周期 | RagdollProfileArtifact经Editor document、compiler和Runtime owner实例化，支持rollback/teardown/reload |
| NPHY-P1-043 | animation blend/transition是简单pose feed，没有native articulation、physical animation或ownership fence | 定义Animation↔Physics authority state machine、kinematic target、blend generation和late result拒绝 |
| NPHY-P1-044 | Physics Editor command只注册descriptor/open view，没有operation factory/controller/transaction | 每项绑定typed payload、document/job、undo/redo、cancel/deadline和terminal receipt |
| NPHY-P1-045 | 四份ZUI业务区域全为`Space`，没有真实材料/body/collider/joint/profile/ragdoll编辑 | 用document-backed controller/view model替换占位；无provider时入口隐藏或typed unavailable |
| NPHY-P1-046 | debug overlay只有Collider DTO，toggle只open view，开放failure仍无viewport provider | 注册Physics-owned ViewportOverlayProvider，读取同代debug snapshot并处理disable/stale清理 |
| NPHY-P1-047 | ragdoll create command不调用generator、不保存资产；Workbench绑定固定演示行和成功反馈 | 接入真实asset transaction/compiler/runtime preview，删除固定业务authority和伪反馈 |
| NPHY-P1-048 | 82项局部测试没有普通产品、native parity、fault/soak/scale/perf qualification | 建立source-bound BuildSet和scenario corpus，覆盖solver/query/filter/event/joint/editor/export/native |

## 10. P2：工程级能力扩展

| ID | 能力差距 | 进入条件与目标 |
|---|---|---|
| NPHY-P2-001 | Character controller与step/slope/platform interaction | G01-G24通过后，基于同一query/filter/shape owner实现并做network replay |
| NPHY-P2-002 | Vehicle、wheel、suspension与tire model | native constraint/material/telemetry成熟后建立独立vehicle subsystem和workload |
| NPHY-P2-003 | Soft body、cloth、rope与deformable collision | 不塞入rigid body manager；定义独立solver/artifact/render bridge和budget |
| NPHY-P2-004 | Destruction、fracture与geometry collection | 依赖cooked collision artifact、streaming、event和render scene增量更新 |
| NPHY-P2-005 | Network rewind、rollback、resimulation与deterministic audit | 单一clock、snapshot identity、input log和backend determinism矩阵通过后进入 |
| NPHY-P2-006 | Async physics、interpolation、extrapolation与multi-rate simulation | 建立独立clock domain和publication fence，不复活第二隐式accumulator |
| NPHY-P2-007 | Scene query acceleration specialization与GPU query | 先让CPU native broadphase正确；GPU只作为有明确latency/stale合同的可选provider |
| NPHY-P2-008 | Large world origin shift、分区world与physics streaming | 依赖coordinate/frame identity、world slot和collision artifact residency |
| NPHY-P2-009 | Deterministic replay corpus与cross-platform tolerance | 记录backend/compiler/CPU/target/build identity并区分bitwise与bounded tolerance |
| NPHY-P2-010 | Physics capture、offline inspector与solver visualization | 只读generation-bound observation，不让debug reader阻塞或改变solver |
| NPHY-P2-011 | Automated Physics Asset decomposition与ML-assisted ragdoll proposal | 输出可审计proposal、误差和人工确认，不直接覆盖source asset |
| NPHY-P2-012 | Competitive benchmark suite | 同场景/画质/solver feature/硬件下报告correctness、CPU、memory、latency和稳定性后再比较Unreal |

## 11. 目标架构与所有权

### 11.1 产品组合

~~~text
Project/Profile PhysicsSelection
  -> ProviderResolver
  -> PhysicsActivationPlan
       package + target + backend artifact + config + capability request
  -> admission
       linked + ABI + artifact + platform + budget + permission
  -> PhysicsActivationReceipt
       Active / Degraded / Unsupported / Failed with exact reason
  -> ordinary App / Editor Host / source export / NativeDynamic
       consume the same capability truth
~~~

普通App、Editor、generated export和NativeDynamic必须消费同一个resolver。source与native可以使用不同载体，但不能对backend、capability、state、query、event、failure和lifecycle给出不同事实。

### 11.2 Runtime owner

| Owner | 唯一职责 | 不拥有 |
|---|---|---|
| PhysicsRuntimeInstance | activation、backend artifact、world registry、shutdown与service publication | 不直接扫描Scene或执行Editor命令 |
| PhysicsWorldSlot | per-world solver、handle table、config generation、command ingress、query/event/debug egress | 不持有其他world全局锁 |
| PhysicsSceneBridge | Scene delta、transform authority、create/update/remove与active body回写 | 不实现碰撞算法 |
| PhysicsClockBridge | Runtime fixed tick到solver substep plan、overstep与interpolation generation | 不建立第二game clock |
| CollisionArtifactService | source mesh/profile到cooked shape/filter/material artifact、cache与residency | 不读取live Editor widget |
| PhysicsQueryService | 同一solver broad/narrow phase上的sync/batch/async query与容量 | 不从Scene snapshot重算近似空间 |
| PhysicsEventStream | native contact/trigger采集、排序、容量、cursor与generation publication | 不让consumer各自重算collision |
| PhysicsObservationStream | stats、debug shape/contact/joint、health和capacity snapshot | 不反向控制solver |

每个step在PhysicsWorldSlot内部形成一个simulation generation transaction：消费固定tick/config/scene delta/commands，推进solver，生成active state/query view/event/debug snapshot，然后一次发布。旧world、旧backend、旧artifact和旧callback只有在reader/in-flight lease归零后退役。

### 11.3 Editor owner

Physics Editor应由CollisionProfileDocument、PhysicsMaterialDocument、RigidBody/Collider/Joint property adapter、CollisionCookJob、RagdollProfileDocument和PhysicsPreviewSession组成。所有修改走共享transaction与artifact compiler；preview消费与runtime相同backend artifact/config；debug view只读PhysicsObservationStream。未安装runtime provider时可以无损编辑source，但simulate、query、overlay和ragdoll preview必须fail-close。

### 11.4 Artifact 与 identity

CollisionArtifact至少绑定source mesh/hash、import/cook settings、shape kind、material/filter profile、compiler/version、target/backend、scale policy、dependency/content hash和validation receipt。PhysicsWorldId、BodyId、ShapeInstanceId、ConstraintId、Skeleton/BoneId、QueryTicket与SimulationGeneration均需显式generation/owner；不得用entity整数、string bone path、Vec index或display name跨world/reload/ABI充当稳定身份。

## 12. 分层重构里程碑

### M0 · Truth Freeze 与红门保留

- 保持Physics experimental/partial/default-off，builtin标为ReferenceApproximation；
- 将ordinary runtime 0、editor 0、generated Disabled、native metadata shell变成机器可读红门；
- 保持debug overlay failure为open，不用PassThrough、伪geometry或静态数字关闭；
- 冻结backend/query/constraint/event/cook逐能力矩阵。

### M1 · Composition、Backend Artifact 与 Lifecycle

- 建立PhysicsActivationPlan/Receipt和唯一ProviderResolver；
- 为各target显式选择Jolt或unsupported，并绑定BuildSet/artifact identity；
- 收敛source/export/native/editor registration parity；
- 建立PhysicsRuntimeInstance shutdown、drain、world retire和DLL quiescence。

### M2 · Clock、World Slot 与增量Scene Bridge

- Runtime fixed schedule成为唯一game clock，删除/迁移manager accumulator；
- 把多张全局HashMap收敛为per-world PhysicsWorldSlot；
- scene full scan/deep clone硬切为delta和stable handle table；
- settings采用prepare-publish-retire并记录applied generation。

### M3 · Native Solver、Filter、Query、Event 与 Constraint

- collision profile编译为native filter generation；
- Jolt query使用真实broad/narrow phase和caller-owned buffer；
- contact/trigger来自native listener/manifold并有有界cursor；
- joint创建native constraint，删除production projection solver。

### M4 · Collision Artifact、Shape 与 Ragdoll

- 建立mesh/heightfield/convex/compound cook、DDC、streaming和last-good；
- 补local pose、rotation、scale、material/mass/subshape identity；
- Ragdoll使用stable skeleton identity、正式artifact和Animation↔Physics state machine；
- 完成fault/OOM/capacity与world replacement recovery。

### M5 · Editor 产品闭环

- 链接Physics Editor，创建真实document/controller/operation factory；
- 替换四份Space surface，完成edit-undo-save-reopen-cook-preview；
- 提供Physics-owned viewport overlay provider和同代observation；
- Workbench删除固定演示authority，展示真实artifact/runtime receipt。

### M6 · Qualification 与竞争基准

- 建立primitive/mesh/query/filter/contact/joint/ragdoll oracle与cross-backend parity；
- 覆盖multi-world、100k body/shape、command/query/event pressure、fault/OOM/soak；
- ordinary/editor/export/native在同一corpus上做行为与lifecycle parity；
- correctness、failure和资源预算通过后才与Unreal/Fyrox/Godot适用workload比较。

## 13. 资格门

| Gate | 验收内容 |
|---|---|
| G01 | Client、Editor Host、generated export与NativeDynamic都生成同一PhysicsActivationPlan/Receipt；required provider缺失时fail-close |
| G02 | runtime/editor catalog closure可重建，manifest package与linked provider一一对应 |
| G03 | target/backend matrix显式选择Jolt build或Unsupported，不静默落入Disabled/builtin |
| G04 | plugin options进入ValidatedPhysicsConfig并绑定desired/applied generation与source |
| G05 | source/native对registration、service、state、query、event、error、unload和capability行为parity |
| G06 | PhysicsRuntimeInstance shutdown停止admission、drain world/callback、退役artifact后卸载backend/DLL |
| G07 | Runtime fixed schedule是唯一game clock；零/一/多tick、overstep与solver substep有确定receipt |
| G08 | settings变更失败保持旧config/world可用；成功按generation迁移或重建全部world |
| G09 | 稳定scene在零dirty时无全node扫描/深拷贝，单entity变更成本与dirty frontier相关 |
| G10 | per-world slot独立并行，step后一次发布同代active/query/event/debug view |
| G11 | poison/native fault进入Faulted/LKG/recreate路径，不继续伪装Ready |
| G12 | builtin只报告ReferenceApproximation能力，生产profile不能把它当完整solver |
| G13 | collision layer/group/mask/matrix进入native response、query、event和debug同一filter generation |
| G14 | ray/sweep/overlap使用active backend broad/narrow phase，支持Any/Closest/AllSorted与stable tie-break |
| G15 | query支持caller-owned buffer、max results、overflow、batch/async、deadline和snapshot generation |
| G16 | rotated/scaled primitive、convex、triangle mesh、heightfield、compound查询与solver结果有oracle parity |
| G17 | contact Begin/Persist/End与trigger Enter/Stay/Exit来自native callback并含shape/subshape/material identity |
| G18 | event stream有bytes/items/time预算、排序/coalesce、overflow telemetry和consumer cursor |
| G19 | Fixed/Distance/Hinge/Slider/ConeTwist/6DoF可见类型均使用native constraint或admission拒绝 |
| G20 | anchor frame A/B、axis、limit、motor、collide-connected、break/projection语义端到端一致 |
| G21 | CollisionArtifact绑定source/settings/compiler/target/backend/dependency/hash并支持cache/LKG/retirement |
| G22 | mesh/heightfield cook在恶意或超量输入下先预算后分配，runtime不逐triangle临时构建compound |
| G23 | body/shape/constraint/query identity在destroy/recreate/world replace/backend reload后拒绝stale |
| G24 | native allocator/body/pair/contact/thread预算可配置、可观测，压力/OOM不会产生半发布generation |
| G25 | ragdoll profile使用stable skeleton/bone identity并完成generate-edit-undo-save-reopen-cook-spawn-teardown |
| G26 | Animation↔Physics authority、kinematic target、blend、pause、world unload和late result有状态机测试 |
| G27 | Physics Editor真实链接；公开command均有factory/receipt；四份ZUI无业务Space占位 |
| G28 | Physics-owned overlay provider发布同代collider/contact/joint geometry，disable/world unload会清除stale frame |
| G29 | Workbench数据来自document/artifact/observation，不含固定body/material/contact/warning或伪成功authority |
| G30 | test report绑定source/build/backend/target/workload；局部helper/descriptor test不能替代产品资格 |
| G31 | multi-world、100k object、query/event/command pressure、fault/OOM/device/backend reload和长时soak机器可读 |
| G32 | 同场景、shape、solver feature、tick、硬件与正确性oracle下记录CPU、latency、RSS和稳定性后才允许竞争结论 |

## 14. 禁止的临时修补

1. 不得只在某个App Cargo feature中硬塞Physics，同时保留ordinary/generated/native三套resolver。
2. 不得把capability字符串、ModuleDescriptor、非空registration或Jolt crate依赖当成provider已执行证据。
3. 不得把builtin积分器或AABB proxy改名为production solver，也不得让Unsupported静默no-op。
4. 不得继续每tick扫描全Scene、clone完整world，或增加第二份“优化cache”而保留双authority。
5. 不得从manager snapshot重算query/contact来冒充Jolt broadphase/listener结果。
6. 不得在Jolt step后用Rust投影body来冒充native constraint完成。
7. 不得用逐triangle compound临时构造替代collision cook、artifact、cache和residency。
8. 不得用string bone path、entity整数、Vec index或display name充当跨reload稳定身份。
9. 不得用Space、PassThrough provider、固定Workbench数字或只open view的command证明Editor完成。
10. 不得关闭debug overlay开放failure，除非真实provider、同代extract与stale清理产品测试通过。
11. 不得用82项局部测试或Native descriptor smoke宣称普通App、Editor、export/native parity。
12. 不得在同条件correctness、failure、soak和统计基准完成前宣称性能或表现优于Unreal。

## 15. 状态与产出边界

| 项目 | 状态 |
|---|---|
| Physics全包、普通产品装配、Editor、dist、catalog、profiles与tests E3静态审查 | review_complete |
| Unreal、Godot、Fyrox、Bevy与Unity Graphics适用边界核对 | review_complete |
| 新增finding | 0 P0 / 48 P1 / 12 P2 |
| 资格门 | 32 |
| Production / tests修改 | 无 |
| Cargo、Jolt、App、Editor、export、NativeDynamic、soak、性能验证 | 本轮未运行 |
| 实施状态 | pending |

本篇完成的是证据冻结、纵向owner边界、重构顺序与验收合同，不是Physics实现完成证明。后续实施必须从M0的产品truth和开放红门开始，按M1-M6推进；任何source drift都要求重算package fingerprint、provider closure、backend matrix、test inventory与capability truth。
