---
title: Runtime Gameplay Ability、Effect、Attribute、Tag、Cue 与 Prediction 当前工作树 authority、artifact、执行与网络边界复审
category: zircon_runtime
report_id: Runtime198
review_date: 2026-08-31
baseline_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
verification_head: 14c89f9776bed828cc85e05e4b9914b3f8d1e784
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/175-runtime-gameplay-ability-effect-attribute-tag-cue-prediction-current-working-tree-authority-artifact-execution-review.md
  - docs/plans/optimize/zircon_runtime/08g-gameplay-ability-effect-attribute-tag-cue-prediction-runtime-review.md
  - docs/plans/optimize/zircon_runtime/99zz-runtime-gameplay-ability-effect-attribute-tag-query-attribute-set-aggregator-capture-execution-cooldown-cost-cue-targeting-task-prediction-replication-network-save-scalability-editor-product-integration-current-source-review.md
related_editor_owner:
  - docs/plans/optimize/zircon_editor/143-editor-gameplay-ability-effect-attribute-set-gameplay-tags-tag-query-cue-prediction-debug-authoring-current-source-review.md
related_code:
  - zircon_runtime/src/script/vm/gameplay_host.rs
  - zircon_runtime/src/script/vm/gameplay_host
  - zircon_runtime/src/script/vm/runtime_context.rs
  - zircon_runtime/src/script/vm/capability_set.rs
  - zircon_runtime/src/script/vm/host
  - zircon_runtime/src/script/vm/plugin
  - zircon_runtime/src/scene/world/project_io
  - zircon_runtime/src/scene/dynamic_scene
  - zircon_runtime/src/core/framework/time
  - zircon_runtime/src/core/framework/net
  - zircon_plugins/net/features/replication/runtime/src
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/first_party_editor_catalog/src/catalog.rs
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/gameplay
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilities/Public/AbilitySystemComponent.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilities/Public/Abilities/GameplayAbility.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilities/Public/GameplayEffect.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilities/Public/GameplayPrediction.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayTags/Source/GameplayTags/Public/GameplayTagContainer.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayTags/Source/GameplayTags/Public/GameplayTagsManager.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayTags/Source/GameplayTags/Public/GameplayTagsSettings.h
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayTags/Source/GameplayTags/Private/GameplayTagRedirectors.cpp
  - dev/bevy/crates/bevy_asset/src
  - dev/Fyrox/fyrox-impl/src
  - dev/godot/scene/main/node.cpp
  - dev/godot/modules/multiplayer
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Runtime/Utilities
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Runtime198 · Gameplay Ability / Effect / Attribute / Tag / Cue / Prediction 当前工作树差距

## 1. 结论

当前 Zircon 仍没有可交付的 Gameplay Ability System（GAS）运行时。当前工作树的 `zircon_runtime/src/script/vm/gameplay_host.rs` 暴露的仍是脚本便利 API，而不是 Definition/Spec/Instance/Component 体系：模块版本仍是 0.1.0，能力声明只有 `gameplay.input`、`gameplay.entity`、`gameplay.navigation`、`gameplay.scene_transition`，没有 ability/effect/attribute/tag/cue/prediction capability。源码中仍不存在 `AbilitySystemComponent`、`GameplayAbilitySpec`、`ActiveGameplayEffect`、`AttributeSet`、`GameplayTagContainer`、`GameplayTagQuery`、`GameplayCue`、`PredictionKey` 或对应 typed owner。

本轮 focused 选择集为 736 个去重文件、41,246 行、36,806 非空行、1,447,644 bytes、285 个测试声明、0 个 ignored marker，指纹为 `11e07672c4f29ed8683f3af77d005a156af05880c3113b8e8e098d8ef5786c96`；其中 gameplay host 根文件及其 15 个模块共 16 个文件、3,009 行、109,202 bytes。目录测试仍只覆盖 combat_lifecycle、component_state、property_animation、spawn_transform 等通用脚本桥行为。它可以作为临时脚本互操作层的证据，不能被当作 Ability runtime 的基础完成度。

最危险的路径是 damage_entity/heal_entity。能力调用传入裸实体句柄和浮点值，combat 实现 clone script.bindings JSON，读取或改写名为 hp 的动态属性；伤害降到 f64::EPSILON 时直接 world.remove_entity(entity)，治疗还接受调用者提供的 max_health。这绕开了属性聚合、捕获快照、免疫/抗性/护盾、死亡策略、事件顺序、authority、预测回滚、复制和存档。

因此 Runtime08G 的 5 个 canonical P0 仍是 5 Open / 0 Partial / 0 Closed；Runtime151 的长账本不能因通用 ECS、clock、replication 或 Dynamic Scene 底座存在而关闭。本次重新核对后仍为 28 项 P1（25 Open / 3 Partial / 0 Closed）、12 项 P2（12 Open）和 24 道资格门（22 Fail / 2 Partial / 0 Pass），不新增独立 P0。当前工作树新增的 replication interpolation index、runtime/editor catalog provider 函数和 combat 字符串复制优化，均未建立 Ability/Effect/Attribute/Tag/Cue owner，不能改变上述判定。

## 2. 当前证据

### 2.1 Host 不是 Gameplay domain owner

- `gameplay_host.rs:38-52` 创建 `zr.zircon.gameplay` v0.1.0，并把四个宽 capability 挂在同一模块上；`gameplay_host.rs:234-256` 仍注册 damage/heal/current_hp/report，而不是 ability/effect calls。
- gameplay_host.rs:53-57 的 entity 直接返回 context.entity as i64；这不是带 World、generation、authority、owner epoch 的稳定句柄。
- gameplay_host.rs:234-256 只注册 damage_entity、heal_entity、current_hp、damage_entity_report；返回类型是 Bool/Float/String，而不是 typed effect handle、activation receipt 或 prediction key。
- `gameplay_host.rs:316-367` 一次注册约 39 个函数，把 spawn、despawn、transform、dynamic component、navigation、HUD、particle 与 combat 全部放入同一 host；没有 domain module boundary、版本化 ABI 或 capability 细分。
- components.rs 用字符串 component id 和 JSON 值调用 dynamic_component/set_dynamic_component，并按属性名扫描 rows；没有 tag index、attribute schema、query bytecode 或 read/write lease。
- `lifecycle.rs:17-95,177-184` 直接 spawn NodeKind、写入动态组件或 remove entity；ability activation 不可能从此路径获得可回滚的 spawn/effect transaction。

### 2.5 当前工作树边界与证据索引

- `zircon_runtime/src/script/vm/gameplay_host/combat.rs:55-164` 仍通过 `SCRIPT_BINDINGS_COMPONENT` 查找 `hp`，伤害归零直接 `world.remove_entity`；`heal_entity` 从调用者接收 `max_health`，没有 AttributeSet、modifier provenance 或 death transaction。
- `zircon_runtime/src/script/vm/gameplay_host/script_bindings.rs:1-75` 仍以 JSON array/property 字符串为 health storage；这不是 stable Attribute identity 或 aggregator。
- `zircon_runtime/src/script/vm/gameplay_host/lifecycle.rs:17-184` 仍直接 spawn/update/insert/remove；`components.rs:25-88` 以字符串 component id 读写动态 JSON，`navigation.rs:70-104` 也把 nav state 序列化回 dynamic component。
- `zircon_runtime/src/script/vm/runtime_context.rs:9-67,101-145` 的 runtime context 只可获得 Core、Level、Entity 与 delta，并通过短生命 reflection operation 借 World；没有 GameplayObjectId、authority、connection 或 prediction scope。
- `zircon_runtime/src/core/framework/net/sync.rs:36-82,188-243` 提供通用 SyncComponent/Field/Delta DTO；`zircon_plugins/net/features/replication/runtime/src/manager/state.rs:8-41` 与 `manager/apply.rs:49-157` 当前新增的是 component/object/field 分层插值索引和 bounded sample retention，仍没有 AbilitySpec/ActiveEffect/Attribute/Cue delta。
- `zircon_plugins/first_party_runtime_catalog/src/lib.rs:13-96` 与 `zircon_plugins/first_party_editor_catalog/src/catalog.rs:10-54` 当前只列 AI、Sound、Texture、Net、Navigation、Particles、Animation、Rendering、Importer、Neural 等 provider；未找到 Gameplay Ability provider、source/artifact compiler 或 editor/runtime conformance。

### 2.2 Combat 是不可保留的临时权威

- combat.rs:55-85 通过 expect_entity 得到实体，clone SCRIPT_BINDINGS_COMPONENT，调用 apply_damage_to_script_health，死亡后直接 remove；未检查 source/target relationship、world identity、authority、prediction window 或 effect context。
- combat.rs:87-112 的 heal_entity 使用来宾传入的 max_health，因此上限不是 AttributeSet 的聚合结果，也没有 clamp policy、change notification 或 modifier provenance。
- combat.rs:114-124 的 current_hp 只读脚本 JSON 的 hp；damage_entity_report（126-164）只是重复同一 JSON 更新并返回 report，不产生可持久、可复制、可重放的 domain event。
- f64::EPSILON 死亡阈值、hp 字符串、动态组件名和裸 u64 共同构成 ABI；任何脚本、编辑器预览或网络 replay 都无法稳定引用同一 gameplay object。

### 2.3 没有 Definition -> compiler -> artifact -> instance 链

当前 runtime 目录检索不到 Ability/Effect/Attribute/Tag/Cue/Predictive domain 类型；plugin manifest 也只有 AI 等其他 capability。first_party_runtime_catalog/src/lib.rs 目前解析 Navigation、Neural 等 provider，first_party_editor_catalog/src/catalog.rs 目前解析 Navigation、Neural editor provider，没有 gameplay provider。因而不存在：

1. versioned source asset、subobject identity、schema migration 和 source-map；
2. deterministic semantic compiler、cycle/invalid capture/stacking 诊断和 last-good artifact；
3. per-world AbilitySystemComponent、granted spec、active effect container、attribute aggregator、tag container、cue dispatcher；
4. activation admission、commit/cost/cooldown、task cancellation、prediction key、rollback/replay receipt；
5. replication delta、late join、save participant、generation/authority fence 和 runtime/editor debug snapshot。

### 2.4 可复用底座的真实边界

时间/固定步、Scene prepared transaction、ResourceKind、replication budget/interest 和 net descriptor 可以承载新 domain，但当前没有 Gameplay consumer。不能把可复用 substrate 的 Partial 误报为 Ability 已实现：必须先建立 domain owner，再接入 World schedule、asset registry、network session 与 save graph。

## 3. 与参考引擎的差异

### 3.1 Unreal GameplayAbilities / GameplayTags

Unreal 的 AbilitySystemComponent 负责授予和激活 ability spec、维护 active effects、属性聚合、tag requirements、事件和复制；GameplayAbility.h 将 activation、commit、cancel、end、task 生命周期分开；GameplayEffect.h 规定 duration/period、modifier、execution、stack、capture 与 granted tags；GameplayPrediction.h 将 prediction key、scoped window、reconciliation 和 rejected action 变成协议。Zircon 当前四条脚本函数没有任何对应 object graph 或 receipt。

GameplayTagContainer.h/GameplayTagsManager.h/GameplayTagsSettings.h 提供 canonical tag dictionary、hierarchical match、redirect、native/config source 和 validation。Zircon 只有 ZUI 中的 Ability.Activate 字符串样例，运行时没有 container、query bytecode、redirect migration 或 O(1) lookup index。

### 3.2 Bevy、Fyrox、Godot、Unity Graphics 的可借鉴约束

- Bevy asset source/handle/load state 说明 definition、dependency、artifact 和 reload 不能由一个脚本 JSON 取代；能力 artifact 应进入统一 asset lifecycle。
- Fyrox 的 scene/editor command 边界说明运行时变更必须经 command/undo/serialization owner；直接改 ECS dynamic component 不可形成编辑器可回放操作。
- Godot 的 node/undo 与 multiplayer 代码说明 authority、object lifetime、RPC 与 editor transaction 必须明确分层；remove_entity 不能承担 death/rollback/replication 多重语义。
- Unity Visual Effect Graph 的 runtime utilities 展示了 graph 编译产物、parameter binding、event context 与 GPU/CPU execution 的分离；Ability/Cue 也需要编译 artifact 和 bounded event ingress，而不是静态文本反馈。

## 4. P1 差距与重构任务

| ID | 当前问题 | 重构结果 / 验收 |
|---|---|---|
| RT-GAS-01 | 无 domain package/owner | 新建 runtime gameplay package，定义 World-owned subsystem、service handle、generation、authority 和 shutdown contract；禁止脚本 host 继续持有 combat authority。 |
| RT-GAS-02 | Ability/Effect/Attribute/Tag/Cue 无稳定 identity | 定义 versioned source schema、stable asset/subobject id、schema hash、migration 与 source-map；save/reload 后 identity 不变。 |
| RT-GAS-03 | 无 shared compiler/artifact | 实现 deterministic semantic compiler、diagnostic code、dependency/SCC、last-good artifact 与 atomic publication；editor preview/runtime 使用同一 artifact。 |
| RT-GAS-04 | 无 AbilitySystemComponent/spec | 为每个 World/actor 建立 component owner，区分 granted spec、activation instance、active effect、task、cue event，并以 generation checked handle 暴露。 |
| RT-GAS-05 | 裸 entity u64 ABI | 用 GameplayObjectId { world, entity, generation } 和 authority/owner admission 替代整数；跨 World、stale、foreign target 必须 fail-closed。 |
| RT-GAS-06 | damage/heal 写 JSON hp | 硬切为 AttributeSet + aggregator；所有 modifier、capture、execution、clamp、resistance、shield、death/downed policy 走 typed transaction。 |
| RT-GAS-07 | 无属性聚合与捕获 | 支持 base/current/final value、snapshot/live capture、operation precedence、source/target tags、dirty invalidation 和 deterministic float policy。 |
| RT-GAS-08 | 无 tag dictionary/query | 建立 canonical hierarchical tag registry、redirect/migration、compact id、container bitset、query bytecode 和 bounded lookup/diagnostic。 |
| RT-GAS-09 | 无 effect duration/period/stack | 实现 instant/duration/infinite、period timer、stack key/policy、granted/blocked tags、expiration receipt，接入 fixed clock domain。 |
| RT-GAS-10 | 无 execution/cost/cooldown | 将 CanActivate、cost check/commit、cooldown、commit failure、cancel/end 统一为 admission state machine；不允许脚本伪造成功结果。 |
| RT-GAS-11 | 无 ability task/latent ticket | 定义 task graph、typed ticket、cancel/deadline/owner teardown、bounded per-frame work 和 exactly-once end；移动/蒙太奇/等待事件必须可回收。 |
| RT-GAS-12 | 无 Gameplay Cue lifecycle | 定义 cue tag/event payload、authority->replica routing、retrigger/while-active/remove、prediction suppression 和 renderer/audio/VFX consumer。 |
| RT-GAS-13 | 无 targeting contract | 提供 typed target data、shape/query snapshot、filter/tag requirements、trace receipt 和 deterministic ordering；禁止脚本传任意 entity 绕过过滤。 |
| RT-GAS-14 | 无 prediction/rollback | 建立 per-connection prediction key、scoped window、input receipt、local speculative state、server reconciliation、reject/rollback 和 replay log。 |
| RT-GAS-15 | 无 replication delta | 为 spec/effect/attribute/tag/cue 定义 schema-versioned delta、baseline/ack、late join、relevancy、owner-only fields 和 bounded bandwidth budget。 |
| RT-GAS-16 | 无 save/replay participant | 将 granted specs、active effects、aggregator state、tags、cooldown、prediction/replay boundary 接入 snapshot/restore/migration；禁止只保存 hp JSON。 |
| RT-GAS-17 | host capability 过宽 | 拆分 gameplay.entity 为 typed domain capabilities；每个 call 校验 owner/authority/world/generation，并提供 denial receipt 与 telemetry。 |
| RT-GAS-18 | world tick 无隔离/预算 | 将 gameplay system 纳入 scheduler phase，定义 activation/effect/cue budgets、fairness、backpressure、cancel 和 per-world parallelism。 |
| RT-GAS-19 | 事件无顺序/幂等协议 | 建立 sequence/causal id、transaction boundary、exactly-once/at-least-once policy、replay verification 与 effect application receipt。 |
| RT-GAS-20 | native/plugin/catalog 缺口 | 为 runtime/editor plugin manifest、feature、catalog、App composition、dist provider 和 ABI conformance 增加 gameplay package；缺 provider 时 fail-closed。 |
| RT-GAS-21 | generic dynamic component 承载领域数据 | 领域状态迁移到 typed ECS columns/owned store；保留 JSON 仅作为显式脚本边界的版本化 adapter。 |
| RT-GAS-22 | 无跨模块 artifact 引用 | Ability/Effect/Cue/Tag dependency 使用 typed handles 与 generation；删除/重载/设备或网络断开时所有引用收到 terminal receipt。 |
| RT-GAS-23 | 无安全与资源配额 | 对 graph depth、modifier 数、tag query、target count、cue fan-out、payload bytes、prediction window 设置可配置上限并可观测。 |
| RT-GAS-24 | 无 domain tests/benchmarks | 增加 property/conformance、save/reload、replay、prediction reject、late join、authority fault、100K actors/1M effects、soak 和 deterministic hash 测试。 |
| RT-GAS-25 | script host API 与新系统并存风险 | 先建立 compatibility adapter 和 deprecation telemetry，再硬切 combat exports；禁止两套 hp/死亡 authority 同时写同一 actor。 |
| RT-GAS-26 | 无观测与诊断合同 | 为 activation/effect/attribute/tag/cue/prediction 定义结构化 trace、causal id、sampling、redaction 和 loss counters；debug 不能依赖 full clone 或字符串日志。 |
| RT-GAS-27 | reload/device/session 失效策略缺失 | 明确 source reload、artifact replacement、device loss、World teardown 时的 quiesce、handle invalidation、last-good install 与 terminal receipt。 |
| RT-GAS-28 | 跨平台数值与兼容策略缺失 | 固定 deterministic numeric policy、schema/ABI version negotiation、旧 artifact rejection 和 migration matrix；禁止平台隐式浮点差异改变战斗结果。 |

## 5. P2 质量与产品化

P2 共 12 项：1) 工具可读 trace；2) hierarchical tag usage report；3) effect heatmap；4) ability audit；5) network bandwidth profiler；6) replay diff；7) hot reload policy；8) artifact cache eviction；9) localization/editor display metadata；10) telemetry redaction；11) fuzz corpus；12) release compatibility dashboard。它们必须建立在 P1 artifact/receipt 合同之后，不能用 UI 统计替代执行证据。

## 6. 资格门（全部在实现后重跑）

| Gate | 当前 | 必须证明 |
|---|---|---|
| G1 domain owner | Fail | 每个 World 只有一个 Gameplay subsystem，shutdown 可 drain。 |
| G2 stable identity | Fail | stale/foreign/generation mismatch 全部拒绝且有 receipt。 |
| G3 compiler parity | Fail | editor/runtime 对同一 source 得到相同 artifact/hash。 |
| G4 attribute correctness | Fail | modifier/capture/stack/death 顺序可重放且无 JSON hp 写入。 |
| G5 tag correctness | Fail | hierarchy/query/redirect/rename/save-reload 结果稳定。 |
| G6 activation lifecycle | Fail | admission/commit/task/cancel/end exactly-once。 |
| G7 cue routing | Fail | authority、prediction suppression、replica 和 renderer consumer 有 trace。 |
| G8 prediction | Fail | accept/reject/reconcile/rollback 与 server receipt 可重放。 |
| G9 replication | Fail | baseline/ack/late join/relevancy 在预算内稳定。 |
| G10 persistence | Fail | save/load/migration 不丢 spec/effect/attribute/tag/cooldown。 |
| G11 scheduler budget | Partial | 通用 fixed-step/clock 存在，但没有 Gameplay work budget/fairness。 |
| G12 capability isolation | Fail | host 不可越权操作其他 World/actor。 |
| G13 editor/runtime provider | Fail | catalog/App/dist/provider 真实装配并在缺失时 fail-closed。 |
| G14 artifact lifetime | Fail | reload/device/session teardown 无 dangling handle。 |
| G15 network fault | Fail | duplicate/loss/reorder/timeout 不产生双重 effect。 |
| G16 save/replay fault | Fail | crash/restart/replay divergence 有诊断和 last-good fallback。 |
| G17 scale | Fail | 100K actor、1M active effect、tag query 和 cue fan-out 有 P99 budget。 |
| G18 deterministic | Fail | cross-platform hash/replay/rollback 稳定。 |
| G19 script compatibility | Partial | 可以有 adapter，但不能双写领域 authority。 |
| G20 security | Fail | graph/payload/tag/cue 资源上限及拒绝遥测闭合。 |
| G21 test evidence | Fail | conformance/property/fault/soak/benchmark artifact 可复算。 |
| G22 editor preview parity | Fail | preview 使用 runtime artifact，不执行静态 fixture。 |
| G23 product consumer | Fail | gameplay 结果驱动 animation/audio/VFX/UI/replication/save。 |
| G24 release packaging | Fail | plugin manifest、feature、ABI、dist、artifact provenance 可发布。 |

## 7. 实施顺序

先完成 RT-GAS-01/02/03/05/17 的 owner、identity、schema、compiler 和 capability admission；随后完成 RT-GAS-04/06/07/08/09/10/11/12 的本地 deterministic execution；再接 RT-GAS-13/14/15/16 的 targeting、prediction、replication、save；最后接 catalog/editor/debug/scale，并在兼容层退役后删除 damage_entity/heal_entity JSON authority。每一层都要提交 artifact、receipt、fault 与 benchmark 证据，不能先把 Workbench 按钮标为完成。

本轮仅完成 Runtime198 review/index/coverage 文档，没有修改 runtime、editor、tests、Cargo、ABI 或 ZUI，也没有运行 Cargo、PIE、网络 fault、save/replay、scale、soak 或 benchmark；按用户要求未查询、轮询、等待或实时跟踪协调器。工作树中已有的未提交 combat/catalog/replication 修改均保留，且本报告只评价其当前边界。
