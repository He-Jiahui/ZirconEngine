---
title: Runtime Gameplay Ability、Effect、Attribute、Tag Query、Aggregator、Capture、Execution、Cooldown、Cost、Cue、Targeting、Task、Prediction、Replication、Network、Save、Scalability、Editor 与 Product Integration 当前源码工程化差距
report_id: Runtime151
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
runtime_child_of: Runtime08G
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
  - zircon_runtime/src/asset
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/gameplay
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench
  - zircon_editor/src/ui/template_runtime/builtin/workbench_generated_bottom_template_bindings.rs
  - examples/woc/scripts/woc_game/src/combat
  - examples/woc/scripts/woc_game/src/world/state.zr
plan_sources:
  - docs/plans/mvp/index.md
  - docs/plans/optimize/zircon_runtime/08g-gameplay-ability-effect-attribute-tag-cue-prediction-runtime-review.md
  - docs/plans/optimize/zircon_editor/21-gameplay-ability-effect-attribute-tag-cue-debug-authoring-review.md
  - docs/plans/optimize/zircon_runtime/13-woc-combat-casting-effect-aura-damage-threat-death-runtime-review.md
  - docs/plans/optimize/zircon_runtime/38-gameplay-framework-game-instance-world-context-level-game-mode-game-state-local-player-controller-pawn-possession-spawn-travel-network-save-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/99zd-runtime-save-game-checkpoint-slot-participant-capture-serialization-migration-platform-cloud-async-network-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zl-runtime-animation-skeleton-clip-pose-graph-state-machine-layer-mask-blend-ik-root-motion-event-extract-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zn-runtime-audio-sound-clip-streaming-device-mixer-bus-effect-spatial-occlusion-reverb-timeline-event-voice-chat-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zo-runtime-network-transport-socket-tls-http-websocket-reliable-udp-session-rpc-replication-prediction-rollback-content-download-editor-product-integration-current-source-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/Runtime/GameplayAbilities/Source/GameplayAbilities
  - dev/UnrealEngine/Engine/Source/Runtime/GameplayTags
  - dev/bevy/crates/bevy_ecs/src
  - dev/bevy/crates/bevy_asset/src
  - dev/Fyrox/fyrox-impl/src
  - dev/godot/scene/main/node.cpp
  - dev/godot/modules/multiplayer
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Runtime/Utilities
---

# Runtime Gameplay Ability、Effect、Attribute、Tag、Cue 与 Prediction 当前源码工程化差距

## 1. 结论

当前 Zircon **没有可作为引擎产品交付的 Gameplay Ability System**。对 `zircon_runtime/src`、`zircon_plugins`、`zircon_app/src`、`zircon_editor/src` 与 `zircon_runtime_interface/src` 中排除 tests/test-named 文件后的 12,288 个生产 Rust/TOML/JSON/ZUI 文件检索，`AbilitySystemComponent`、`GameplayAbilitySpec`、`ActiveGameplayEffect`、`AttributeSet`、`GameplayTagContainer`、`GameplayTagQuery`、`GameplayCue`、`PredictionKey` 与 `ScopedPredictionWindow` 均为 0 命中。`GameplayAbility` 和 `GameplayEffect` 各 9 次命中全部来自两个 Editor 文件中的 generated-bottom control ID/binding string，不是 runtime type、asset、provider、compiler或执行入口。

当前真正执行玩法的首方入口是 `zr.zircon.gameplay` v0.1.0 脚本宿主。它注册 39 个 callback，把 input、scene transition、navigation、transform、任意 dynamic component 读写、spawn/despawn、HUD、particle 与 combat 都挂在四个粗粒度 capability 下。`expect_entity`把非负整数或 `HostHandle`直接降为裸 `u64`；它不校验当前 script entity、owner、world/entity generation、关系或网络 authority。模块文档声称调用“scoped to the active script entity”，但大多数入口实际接受调用者提供的任意 entity ID。

`damage_entity`与`heal_entity`不是 Effect/Attribute 的雏形，而是必须硬切的临时权威：二者 clone `script.bindings` JSON，选择第一个 enabled binding，读写 `properties.hp`；`heal_entity`还信任调用者传入的 `max_hp`。damage把 HP 压到 `f64::EPSILON` 后直接 despawn entity，绕过 resistance、immunity、shield、attribute hook、death/downed policy、transaction、network authority、prediction、save/replay和ordered event。`damage_entity_report`只是在同一路径外包一层 JSON report，并没有修复所有权。

WOC 样例包含大量值得保留的规则 oracle，但它进一步证明领域 authority 分裂。当前 `world/state.zr`已导入 `casting_state`与`known_ability_state`；`ability_admission`、通用`effects`、effect sequence和aura dispatch仍只被各自test main引用。它们用project-specific数组、字符串、float cooldown与分支状态表达cost、cast、effect和aura，不能替代引擎级source/compiler/artifact、per-World owner、typed transaction、prediction、replication或save participant。WOC内容迁移继续由Runtime13拥有，本篇不复制其数百项职业/战斗规则。

Workbench的Ability、Effect、Tags三个surface仍在发布产品假象：固定 `GA_DashAttack`、`GE_HealthRegen`、`GE_DamageFire`、`DefaultGameplayTags.ini`、`Server Initiated`、duration/period/stack字段，以及固定的“+50 health”“predicted activation”反馈。当前没有对应document provider、shared compiler、operation、runtime request或debug snapshot。42个selected Editor test declaration验证的是retained route/control/template surface，不是Ability产品闭环。

本轮不重复登记Runtime08G的5项P0；它们当前仍为 **5 Open / 0 Partial / 0 Closed**。本文把其30项概念账本按当前源码扩展为72项可实施Runtime P1：**57 Open / 15 Partial / 0 Closed**；新增16项P2全部Open。40项资格门为 **33 Fail / 7 Partial / 0 Pass**。Partial只表示通用asset、clock、Dynamic Scene transaction、net descriptor/budget、animation/navigation/UI和capability底座可复用，不表示任何Ability、Effect、Attribute、Tag、Cue或Prediction领域合同已经成立。

## 2. 审查边界、currentness与证据强度

### 2.1 Currentness

- 审查基线：`main@1b2684b40ae3eba7abfcdfae3fe7e341b4906ec8`。
- 读取时共享工作树有3,462个tracked changes、1,297个untracked paths；本文读取当前working bytes，不归因、不覆盖、不回退其他Session改动。
- gameplay effect ZUI、gameplay host lifecycle与combat lifecycle test含在途修改；当前差异只涉及ZUI单行和spawn `Result`传播/测试`expect`，没有建立GAS领域owner。实施前必须重新取fingerprint并重判。
- 生产词法语料冻结于`2026-08-25T15:45:03+08:00`。首轮写入后的静态复验发现共享树累计新增14个production文件，并改动selected Editor surface、Font/Material asset export与clock-domain/fixed-step底座；本文已按复验后的current bytes重冻。新增内容没有GAS领域类型或产品caller，Editor仍是momentary control加fixed feedback。最终词法结论来自逐个读取physical working-tree文件，不依赖会忽略untracked path的Git索引。
- 按用户指令，本轮不轮询协调器。写入前的精确路径登记/lease命令被服务接受但15秒内无终态，随后已停止等待；本文不引用未取得的协调baseline epoch。
- 当前MVP仍未完成。本轮属于MVP-00允许的C3 read-only audit，不实施高级Ability功能。
- 用户明确暂不优化tooling；tooling不属于本文source、finding、里程碑或资格门owner。

### 2.2 冻结范围

统计口径：repository-relative path转`/`并小写排序；逐文件取当前bytes SHA-256；聚合输入为`path|file_sha256`以LF连接且末尾无LF。Rust tests按`#[test]`声明计数；Unreal/Godot按automation/test macro及suite `Test_*`声明计数；C#按Test attribute计数；ignored/disabled单列。Zr样例没有相同attribute语法，因此单列selected test-main入口，不与Rust/C++ test declaration混算。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 工作树fingerprint与证据 |
|---|---:|---|
| Zircon gameplay/script runtime | **30 / 7,541 / 6,954 / 270,407 / 40 / 0** | host descriptor、39 callbacks、value/entity conversion、capability、plugin manifest、runtime context、scene script persistence；`8b7e4066881315bc4a56f933aa38caa5306d6cdd050b497e09ab3c9b5ef44d78` |
| Zircon Editor product surface | **16 / 6,554 / 6,152 / 265,609 / 42 / 0** | 三份ZUI、fixed feedback、field/navigation/control、generated-bottom bindings与七组surface tests；`c48445c74dc6d17d0a195df7a973f2cdbb560d59b60dedd31d28fd33a8c02aca` |
| Zircon reusable substrate | **52 / 7,818 / 6,998 / 278,448 / 60 / 0** | ResourceKind/asset、real/virtual/fixed clock、net RPC/sync、replication budget/interest/late join、Dynamic Scene prepared transaction；`bd86429bb638eb44ea7ad447562d8d73d462121f9ef654fa35fda7cc536b93f4` |
| WOC selected migration oracle | **12 / 98,698 / 95,862 / 4,300,380 / 1 test-main / n/a** | product root/world state、ability admission、casting、known ability、effects/sequence/aura及generated catalogs；`9ba98d33ac66ea5d41644d9d6e425729422e119594db0bd62f452c1827751e3f` |
| Unreal GAS与GameplayTags | **28 / 35,618 / 29,317 / 1,433,125 / 19 / 0** | ASC/spec/ability/effect/aggregator/attribute/prediction/target/task/cue/tag dictionary/query及七份test source；`e3f1b6e1949e111d16bc259e01aac14076c27dcd7e693245c74f996fc2c1dbbf` |
| Bevy、Fyrox、Godot与Unity Graphics | **23 / 21,369 / 18,618 / 772,461 / 99 / 0** | ECS hooks/schedule/asset、plugin/script/scene lifecycle、authority replication、VFX event/binding/Playable adapter；`0134465e17dffcae86abc2756780f25806ab43115fd5e091ff8c6f57fe75691c` |

Zircon选择集合计110文件、120,611行、5,114,844 bytes，包含142项Rust test declaration与1个selected Zr test-main；五引擎参考合计51文件、56,987行、2,205,586 bytes、118项test declaration。选择集是本轮可复算证据边界，不表示未列文件永远与Ability无关。

### 2.3 纵向扫描链

本轮按 product claim -> package/capability -> source/schema/resource kind -> import/cook/artifact -> Scene/project binding -> per-World/per-owner lifecycle -> tags/attributes -> effect spec/aggregation/execution -> ability grant/activation/task -> target/cue adapters -> fixed schedule -> authority/prediction/RPC/replication -> save/replay/hot reload -> diagnostics/performance -> Editor runtime projection -> focused tests逐层读取。WOC只用于验证项目级旁路与迁移边界。

参考侧以Unreal GameplayAbilities与GameplayTags为主合同；Bevy、Fyrox只验证ECS hook、schedule、asset、plugin/script/scene owner；Godot只验证node lifecycle和multiplayer authority/replication；Unity Graphics只验证Cue下游VFX event/property/playable adapter。后四者没有first-party完整GAS，不能用其缺失项降低Unreal基线。

## 3. 当前可保留的真实基础

1. Script host descriptor具备参数kind、arity、required capability和调用错误映射，可作为未来受限script adapter的承载层；但`gameplay.entity`必须硬拆，且脚本不得获得领域容器或裸World写权限。
2. Runtime已有Asset ID/handle、generic manager/import/reference resolution和部分generation/LKG语义，可承载新source与artifact；当前`ResourceKind`没有Tag Dictionary、Attribute Set、Gameplay Effect、Gameplay Ability或Gameplay Cue，不能据此判为领域资产Partial完成。
3. real/virtual/fixed clock、fixed-step plan与world generation是真实底座，可承载effect timer和ability task；当前没有Ability schedule phase、integer due key、catch-up policy或owner teardown。
4. Dynamic Scene prepared spawn transaction会检查world、schema、component registry generation和change tick。这是设计Ability scene install transaction时应复用的currentness思想，但当前Scene schema完全不保存Ability owner/spec/effect/tag状态。
5. generic net sync提供Server/ClientOwned、field descriptor、OnChange/Interval/Once、interest、snapshot/delta、late join和count/byte budget；它没有Ability RPC、prediction ledger、attribute delta、active-effect replication、cue dedup或server validation。
6. Animation、Navigation、HUD/particle dynamic component写入构成下游adapter的可替换底座；当前脚本直接修改这些域，尚无由authoritative gameplay journal驱动的Cue/Task adapter。
7. WOC的catalog、规则模块和contract tests包含大量迁移oracle；它们必须编译进入统一engine artifact，而不是继续与`WorldState`分支表并存为第二套authority。

## 4. 当前源码断路

### 4.1 Owner、身份与权限

- 没有`GameplayAbilityWorldRuntime`或等价per-World service，也没有per-owner ability/effect/tag/attribute container。
- `EntityId`仍是裸`u64`；World有全局generation，但host handle不携entity generation、owner serial、connection、authority或lease。
- `runtime_context`确实携带当前entity，绝大多数host函数却不以它限定target。模块documentation与执行事实相反。
- 一个`gameplay.entity` capability同时授权query、任意component写、transform、spawn/despawn、damage/heal和presentation写入。manifest grant并非自动给所有脚本，但一旦授予，动作面和对象面都过宽。
- `expect_float`向`f32`转换时没有统一finite/range/unit policy；combat的HP又使用`f64` JSON和`f64::EPSILON`，数值合同分裂。

### 4.2 Tag、Attribute与Effect

- 没有tag dictionary/source/redirect/stable ID/parent closure/net index，也没有container reference count或compiled query token stream。
- 没有Attribute Set schema、base/current、aggregator channel、modifier provenance、pre/post hook、capture definition或cross-attribute transaction。
- 没有Effect Definition、mutable Spec、Context、Active Effect handle/container；instant/duration/infinite/period、stack、overflow、refresh、inhibition、requirements、immunity与removal query均为空。
- damage/heal以第一份enabled `script.bindings`的`hp`键作为状态，binding顺序可改变语义；直接despawn把Effect execution与entity lifetime错误耦合。
- generic fixed clock和Dynamic Scene transaction没有生产caller把它们组成Effect scheduler/attribute commit。

### 4.3 Ability、Task、Target与Cue

- 没有Ability Definition/Spec、grant/revoke、activation handle、Can/Try/Activate/Commit/End/Cancel状态图、cost reservation、cooldown handle、charge或activation group。
- 没有Ability Task owner、await/cancel/timeout、world retire、target confirmation或replicated event cache。
- target只表现为调用者提交的裸entity ID或JSON position；没有target data variant、source snapshot/revalidate、LOS/range/team/authority policy或async confirm/cancel。
- HUD、particle、animation和navigation由脚本直接写组件/manager；没有Gameplay Cue的Executed/Added/WhileActive/Removed生命周期、dedup、prediction reconcile或presentation-only边界。
- WOC有cost/cooldown/cast/periodic规则，但依赖project-specific array/string/float状态，且关键owner仍与product reachability分裂。

### 4.4 Prediction、Replication、Save与Replay

- Workbench的`Server Initiated`与`predicted activation`只有字符串。没有role/security policy、prediction key、scoped window、dependency chain、server receipt、reject/caught-up callback或side-effect rollback。
- generic replication manager以`NetObjectId + component_type`保存snapshot，能做delta、interest、budget和late join；没有Ability owner generation、activation/effect identity、ordered attribute transaction或domain-specific replication mode。
- generic RPC仍是schema/quota/diagnostics底座，未发现World产品调用者执行Ability request、target data、cancel/end或batch。
- 没有Ability/Effect save participant、active timer/spec revision snapshot、replay journal、load migration或late result fence。

### 4.5 Product、测试与性能

- Workbench三个surface、generated bottom panel和fixed feedback形成可点击界面，却没有runtime provider/document/compiler/operation/debug projection；成功文本不能作为运行证据。
- gameplay host直接相关测试共15项，只覆盖mock-world JSON component、damage/heal/despawn、spawn/transform、HUD、navigation、scene transition和capability拒绝；selected VM/runtime集合的其余测试验证通用registry/manifest/context。
- 没有grant/activate/cancel、effect duration/period/stack、attribute aggregation、tag exact/parent/query、target authority、prediction reject/rollback、replication/late join、save/reload、fault/scale/soak或benchmark测试。
- 当前damage路径clone/parse/serialize JSON并线性找binding；tag、modifier、timer、task和target都没有prepared data structure，尚无资格谈超越Unreal的吞吐、内存或延迟。

## 5. 参考引擎的可迁移合同

### 5.1 Unreal：主合同，不做表面复刻

- `UAbilitySystemComponent`明确拥有`FActiveGameplayEffectsContainer`、replicated ability spec fast array、spawned Attribute Sets、owned tag count、owner/avatar、replication mode和prediction state。Zircon可采用data-oriented布局，但必须保留唯一owner和完整lifecycle。
- `FGameplayAbilitySpec`把definition/class与per-owner grant state分开，并以handle、level、input/source、activation info和fast-array hooks参与复制。`UGameplayAbility`明确区分Can/Try/Activate/CommitCost/CommitCooldown/End/Cancel及instancing、net execution、replication/security policy。
- `UGameplayEffect`是共享定义，`FGameplayEffectSpec`是带context/capture/set-by-caller的应用规格，`FActiveGameplayEffect`是已应用实例。duration/period/stack/overflow/inhibition/requirements/immunity/execution不能压成一次数值写入。
- `FAggregator`保存modifier channel、qualifier、source handle、dirty propagation和evaluation metadata；`UAttributeSet`提供base/current及pre/post attribute/effect hooks。Zircon应把最终提交做成确定性transaction，不复制UObject形状。
- GameplayTags维护dictionary source、explicit/parent tag、redirect、network index、container和prepared query token stream；exact与hierarchical match、空query和序列化均有测试。
- Prediction以generational key、scoped window、owner-only return replication、rejected/caught-up delegate和dependency chain表达可撤销side effect；target data、generic replicated event和RPC batch都绑定ability/prediction identity。
- Gameplay Cue区分Executed、Added、WhileActive、Removed，并允许defer/replicate；它是权威effect/event的presentation projection，不是renderer反向拥有玩法状态。

### 5.2 Bevy、Fyrox、Godot与Unity Graphics：只取各自强项

- Bevy的typed Component lifecycle hook、observer、explicit Schedule graph和typed AssetServer/Handle说明Zircon应把Ability owner变更放入可排序stage，而不是host callback立即写World。
- Fyrox的Plugin、ScriptTrait、Graph、Scene/SceneContainer/loader说明domain service、script adapter和scene ownership必须分层；它没有完整GAS，不作为效果语义基线。
- Godot Node group只是字符串membership，不具备tag dictionary/query/count/provenance；multiplayer spawner会在非authority时拒绝spawn，synchronizer/config展示了authority与property replication的最低边界。
- Unity Graphics的VFX event binder、output event handler、property binder和VisualEffectControl Playable只证明Cue下游adapter应有prepare/send/subscribe/play/stop/reinit生命周期；它不提供Ability/Effect truth。

## 6. 唯一Owner与硬边界

目标链固定为：

`versioned Tag/Attribute/Effect/Ability/Cue sources -> shared validator/compiler -> immutable GameplayBuildSetArtifact -> per-World GameplayAbilityWorldRuntime -> per-owner AbilityOwnerState -> typed command/transaction/schedule -> GameplayJournal -> network/save/replay/debug/presentation projections`。

1. Runtime151唯一拥有Gameplay Tag dictionary/query、Attribute schema/aggregator、Effect spec/container/execution、Ability spec/activation/task、target data、Cue event、prediction ledger和领域save/replication语义。
2. Editor21只拥有source document、transaction/history、picker/graph/inspector、compile diagnostics、preview/debug projection；不得保存runtime shadow state或生成固定成功反馈。
3. Runtime13只拥有WOC职业/战斗规则、content coverage和迁移oracle；所有规则最终编译到Runtime151 artifact，不保留project-side平行Ability runtime。
4. Runtime38/Gameplay Framework提供WorldContext、Player/Controller/Pawn/possession与authority主体；Runtime151引用qualified owner/avatar，不接管游戏流程。
5. Runtime99zo提供transport/RPC/snapshot/interest/budget底座；Runtime151拥有activation/effect/prediction schema、validation、ordering、reconciliation与late-join projection。
6. Runtime99zd提供save/checkpoint事务和slot；Runtime151实现participant、snapshot、migration、restore fence和replay journal。
7. Animation、Physics、Navigation、Audio、VFX、UI只消费Task/Target/Cue request或journal，不允许直接改Ability容器；Gameplay也不直接改这些域的内部状态。
8. `damage_entity`、`heal_entity`、任意target的`gameplay.entity`写入和fixed Workbench success必须在M0硬切。迁移同一里程碑更新所有首方caller；不留旧名re-export、compat module、双写、fallback JSON authority或“新系统失败后走旧路径”。

建议核心身份均为qualified generational handle：`GameplayBuildSetId`、`TagId`、`AttributeId`、`AbilityOwnerKey { world_generation, entity, entity_generation, owner_serial }`、`AbilitySpecHandle`、`ActivationHandle`、`EffectSpecHandle`、`ActiveEffectHandle`、`AbilityTaskHandle`和`PredictionKey`。所有跨线程/网络/save回执先校验build-set与owner generation，再允许提交。

每个World只创建一个`GameplayAbilityWorldRuntime`，内部按owner分片持有tag count、attribute slots/aggregators、ability specs/activations、active effects、tasks/timers和prediction ledger。authoring source与compiled artifact只读共享；热路径不解析字符串/JSON、不按asset Vec index重寻址、不持有全局mutex调用脚本或adapter。

固定simulation阶段至少为：收集/排序request -> pure admission与target validation -> prediction reservation -> atomic cost/cooldown/effect/attribute commit -> due task/effect execution -> terminal cleanup -> journal publish -> replication/save/debug/Cue projection。阶段内使用command buffer和deterministic key；listener只看完整transaction receipt。

## 7. Runtime08G父P0当前状态

| 父项 | 当前状态 | Runtime151复核 |
|---|---|---|
| P0-1 Workbench发布Ability/Effect/Tags产品但runtime无domain/artifact/provider | **Open** | 三份ZUI、generated bottom绑定和fixed success仍存在；无runtime provider |
| P0-2 `Server Initiated`/predicted activation没有authority/prediction/rollback | **Open** | 仍只有UI/feedback字符串；所有prediction核心类型0命中 |
| P0-3 `damage_entity`以JSON HP为权威并直接despawn | **Open** | combat路径未迁移，report wrapper仍调用同一实现 |
| P0-4 `gameplay.entity`允许对任意裸u64做过宽写入 | **Open** | current entity未约束target；无entity generation/owner/role/action policy |
| P0-5 无asset/cook/scene/world/plugin生命周期 | **Open** | ResourceKind、asset export、Scene project、catalog均无Ability domain |

父P0只在Runtime08G关闭；本文只记录current-source复核，不重复计数。

## 8. P1 Runtime专属重构清单

### 8.1 Owner、Source、Artifact与Product Truth

| ID | 状态 | 当前差距与目标 |
|---|---|---|
| GAS-P1-001 | Open | 建立唯一`GameplayAbilityWorldRuntime`；Editor、script、WOC、net不得维护平行容器 |
| GAS-P1-002 | Open | 建立generation-qualified owner/avatar与per-owner state；禁止裸u64成为跨帧身份 |
| GAS-P1-003 | Open | 建立Tag Dictionary source及project/plugin/native/generated precedence与provenance |
| GAS-P1-004 | Open | 定义versioned Tag、Attribute Set、Effect、Ability、Cue source schema和unknown-field policy |
| GAS-P1-005 | Partial | 复用generic ResourceId/handle/import/reference/generation底座，新增五类resource kind与typed importer/cook target |
| GAS-P1-006 | Open | shared deterministic compiler输出immutable`GameplayBuildSetArtifact`、dependency graph、diagnostics与digest |
| GAS-P1-007 | Open | first-party provider进入catalog/profile/project selection；Client/Server/Editor按required capability fail-close |
| GAS-P1-008 | Partial | 复用Scene project/Dynamic Scene transaction思想，新增Ability owner/spec binding与atomic install/uninstall |
| GAS-P1-009 | Open | provider不可用时Workbench显示Unavailable；所有command返回runtime/compiler operation receipt |

### 8.2 Gameplay Tags与Attribute Set

| ID | 状态 | 当前差距与目标 |
|---|---|---|
| GAS-P1-010 | Open | Tag canonical name、stable content ID、dense index、parent closure、dictionary generation/hash |
| GAS-P1-011 | Open | source owner、redirect/rename/delete、循环/歧义检查和跨资产migration |
| GAS-P1-012 | Open | container维护explicit/parent/source reference count，支持Exact/Any/All与确定性delta |
| GAS-P1-013 | Open | query AST经budget校验编译为无分配token program，支持all/any/none嵌套与generation invalidation |
| GAS-P1-014 | Open | Attribute Set schema定义stable ID、slot、type、unit、default、clamp、replication/save policy |
| GAS-P1-015 | Open | per-owner dense base/current storage、schema generation和compatible migration/rebuild |
| GAS-P1-016 | Open | aggregator定义add/multiply/override channel、qualifier、source provenance、reverse evaluation与dirty propagation |
| GAS-P1-017 | Open | pre/post attribute/effect hook进入bounded transaction，禁止重入失控和锁内脚本回调 |
| GAS-P1-018 | Open | finite/NaN/Inf、precision、rounding、saturation、cross-attribute constraint和deterministic order合同 |

### 8.3 Gameplay Effect、Capture与Execution

| ID | 状态 | 当前差距与目标 |
|---|---|---|
| GAS-P1-019 | Open | 分离immutable Effect Definition、mutable Spec、Context、Active Effect及各自stable handle |
| GAS-P1-020 | Open | 统一Instant/Infinite/Duration/Periodic、execute-on-apply、expiry、pause与bounded catch-up |
| GAS-P1-021 | Open | stack key/limit/overflow、duration refresh、period reset、inhibition/resume与typed outcome |
| GAS-P1-022 | Open | source/target requirements、immunity、application query和按handle/tag/query/source removal |
| GAS-P1-023 | Open | capture definition预解析source/target attribute/tag；Spec支持snapshot与execution-time live capture |
| GAS-P1-024 | Open | Execution Calculation只消费bounded immutable input并输出modifier/command；不得直接写World/despawn |
| GAS-P1-025 | Open | attribute/effect/tag/granted-ability/cue变更在单一atomic transaction提交并生成journal receipt |
| GAS-P1-026 | Partial | 复用fixed clock/plan，建立integer due key、same-tick phase、time dilation、pause、missed-tick policy |
| GAS-P1-027 | Open | ActiveEffectContainer按owner/definition/source/tag/due time索引，支持tombstone、dirty queue与幂等terminal |

### 8.4 Gameplay Ability、Cost、Cooldown与Task

| ID | 状态 | 当前差距与目标 |
|---|---|---|
| GAS-P1-028 | Open | Ability Definition/Spec、grant/revoke/upgrade/remove-on-end、dynamic tags和stable spec handle |
| GAS-P1-029 | Open | activation instance拥有generation-qualified handle、phase、target、reservation、children和单次terminal result |
| GAS-P1-030 | Open | CanActivate/Try/Activate/Commit/End/Cancel合法状态图与结构化failure tags |
| GAS-P1-031 | Open | cost先reserve后atomic commit，cancel/failure按policy refund并记录attribute transaction |
| GAS-P1-032 | Open | cooldown用Effect/spec handle表达duration、charges、query和server correction，不用float字段散落 |
| GAS-P1-033 | Open | activation group、block/cancel/replace/queue/priority/expiry政策由owner统一裁决 |
| GAS-P1-034 | Open | Task有owner、generation、await source、timeout、cancel、world/plugin retire和terminal receipt |
| GAS-P1-035 | Open | typed Gameplay Event schema、bounded payload、tag query trigger、ordering和consume policy |
| GAS-P1-036 | Open | Input Action/Input User映射到spec/activation request；不再让Gameplay直接轮询global raw key |

### 8.5 Targeting、Gameplay Cue与Domain Adapter

| ID | 状态 | 当前差距与目标 |
|---|---|---|
| GAS-P1-037 | Open | TargetData覆盖entity/point/area/hit/custom variant，带schema/version/size/depth budget |
| GAS-P1-038 | Open | admission与commit/impact分别声明alive/team/range/LOS/facing/world/authority revalidation |
| GAS-P1-039 | Open | async target acquire/confirm/cancel与replicated target data绑定activation/prediction identity |
| GAS-P1-040 | Open | Cue identity、parameters、Executed/Added/WhileActive/Removed、lifetime与parent-child provenance |
| GAS-P1-041 | Partial | VFX/Audio/Render已有通用执行底座；新增Cue adapter registry、dedup、pool、budget和terminal cleanup |
| GAS-P1-042 | Partial | Animation已有sequence/graph底座；Task只通过typed request/result/cancel驱动，不直接写parameter JSON |
| GAS-P1-043 | Partial | Navigation/Physics query作为target/task provider，返回qualified hit/path与currentness receipt |
| GAS-P1-044 | Partial | HUD/Runtime UI只消费bounded gameplay projection；删除脚本直接写`gameplay.hud_text`的shipping authority |
| GAS-P1-045 | Partial | 复用host descriptor/cap check，拆成self query、validated target command、spawn lease等typed facade |

### 8.6 Authority、Prediction与Network

| ID | 状态 | 当前差距与目标 |
|---|---|---|
| GAS-P1-046 | Open | 定义LocalOnly/ServerOnly/LocalPredicted/ServerInitiated等execution与security policy及owner connection |
| GAS-P1-047 | Open | generational PredictionKey绑定connection/activation/build set，防wrap、reuse与cross-owner提交 |
| GAS-P1-048 | Open | scoped prediction window、dependent chain、nested action与bounded outstanding ledger |
| GAS-P1-049 | Open | predicted attribute/effect/tag/cost/cooldown/cue/task side effect全量登记可逆操作 |
| GAS-P1-050 | Open | server accept/reject/caught-up、out-of-order/duplicate/late receipt与deterministic rollback/replay |
| GAS-P1-051 | Open | Cue/Task/event在prediction reconciliation中dedup或replace，terminal cleanup幂等 |
| GAS-P1-052 | Partial | 复用generic RPC schema/quota，新增activation/target/cancel/end/effect request与batch handler |
| GAS-P1-053 | Partial | 复用sync descriptor/interest/budget/late join，定义spec/effect/tag/attribute replication mode与ordering |
| GAS-P1-054 | Open | server重做target/cost/cooldown/authority validation，限制rate/bytes/depth/targets并记录拒绝原因 |

### 8.7 World、Scene、Save、Replay与Plugin Lifecycle

| ID | 状态 | 当前差距与目标 |
|---|---|---|
| GAS-P1-055 | Open | per-World create/activate/tick/quiesce/teardown；owner/avatar spawn/possession/travel/despawn完整接线 |
| GAS-P1-056 | Partial | 复用real/virtual/fixed time，固定Ability phase order、clock domain stamp和frame/tick receipt |
| GAS-P1-057 | Partial | 复用world generation，新增entity generation与owner serial；所有async/net/save结果过fence |
| GAS-P1-058 | Open | Scene/Prefab保存source binding而非runtime实例；load/duplicate/remap/merge/undo roundtrip不丢字段 |
| GAS-P1-059 | Open | Save participant捕获spec/active effect/attribute/tag/task/timer/prediction terminal状态及build-set revision |
| GAS-P1-060 | Open | gameplay journal作为replay、rollback、diagnostic与determinism oracle，事件identity和顺序稳定 |
| GAS-P1-061 | Partial | 复用asset generation/Dynamic Scene currentness，建立prepare/migrate/atomic swap/LKG与late-result reject |
| GAS-P1-062 | Open | plugin unload先quiesce extension/task/execution/cue，等待lease归零并一次性terminal cleanup |
| GAS-P1-063 | Partial | 复用generic diagnostics，新增bounded owner/activation/effect/prediction snapshot、receipt与causal trace |

### 8.8 Scheduling、Scalability、Performance与Qualification

| ID | 状态 | 当前差距与目标 |
|---|---|---|
| GAS-P1-064 | Open | prepared dense IDs、SoA/small-vector/arena与batch aggregation；ordinary hot path零JSON/字符串解析 |
| GAS-P1-065 | Open | timer wheel/heap、dirty owner/attribute/effect队列和event subscription，禁止每帧全量扫描 |
| GAS-P1-066 | Partial | 复用net count/byte budget，补owner/effect/task/event/cue/prediction/alloc/time预算与reject/degrade策略 |
| GAS-P1-067 | Open | 可并行pure evaluate与确定性ordered commit；禁止全局mutex和锁内script/plugin callback |
| GAS-P1-068 | Open | 固定1/100/1k/100k owner、effect storm、tag query、task、prediction workload及P50/P95/P99/RSS指标 |
| GAS-P1-069 | Open | malformed source/net/save、OOM/queue pressure、plugin crash/reload、disconnect/travel和long-soak矩阵 |
| GAS-P1-070 | Open | 同源规则与Unreal做correctness、activation/effect吞吐、memory、replication bytes和rollback比较 |
| GAS-P1-071 | Open | Workbench由runtime document/compiler/provider/debug snapshot驱动，删除固定asset/status/success |
| GAS-P1-072 | Open | shipping profile/cook/package/Client/Server/Editor/PIE资格证据闭环；无证据不得宣传超过Unreal |

P1复算：Partial为005、008、026、041、042、043、044、045、052、053、056、057、061、063、066，共15项；其余57项Open，0项Closed。

## 9. P2 领先性与高级产品

| ID | 状态 | 目标 |
|---|---|---|
| GAS-P2-001 | Open | SIMD/batched aggregator与cache-friendly multi-owner effect execution |
| GAS-P2-002 | Open | 多帧deterministic rollback simulation与可证明side-effect compensation |
| GAS-P2-003 | Open | temporal attribute/tag/effect history query与低成本debug time travel |
| GAS-P2-004 | Open | typed Ability graph/DSL编译、增量dependency rebuild与program diff |
| GAS-P2-005 | Open | seamless travel/server handoff下activation/effect/task/prediction迁移 |
| GAS-P2-006 | Open | domain-aware delta compression、dictionary negotiation与late-join streaming |
| GAS-P2-007 | Open | AI可查询的ability feasibility/cost/target/effect摘要，不执行副作用 |
| GAS-P2-008 | Open | active owner跨build-set live migration、dual-run oracle与atomic cutover |
| GAS-P2-009 | Open | large-world shard/interest下effect aura/target/query边界和跨区handoff |
| GAS-P2-010 | Open | GPU-friendly Cue instance batching与严格presentation-only ownership |
| GAS-P2-011 | Open | activation/effect/prediction causal graph、first divergence和bounded capture |
| GAS-P2-012 | Open | query/execution bytecode fuzz、property、metamorphic与model checking |
| GAS-P2-013 | Open | 与Unreal测试语义对齐的differential oracle和版本化偏差说明 |
| GAS-P2-014 | Open | 多用户authoring merge、stable IDs、冲突可视化与runtime-safe publish |
| GAS-P2-015 | Open | workload-aware quality/budget policy，保持simulation truth不因presentation降级改变 |
| GAS-P2-016 | Open | 持续竞争基准证明correctness相同前提下的延迟、吞吐、内存、网络bytes优势 |

## 10. 依赖顺序与实施里程碑

### M0 · Product Truth、Authority Hard Cut与RED Freeze

- Workbench在provider缺失时明确Unavailable，删除固定成功/预测/模拟反馈。
- 为现行`damage_entity`、任意entity mutation、fake prediction、WOC双authority建立RED；迁移首方caller后同里程碑删除旧shipping路径。
- 固定owner、identity、source/artifact、command/receipt与phase边界；禁止compat facade和双写。

### M1 · Gameplay Tags与Attribute Kernel

- 实现dictionary/source/redirect、container count、prepared query和测试。
- 实现Attribute Set schema、dense storage、aggregator、hooks与atomic transaction。

### M2 · Gameplay Effect Runtime

- 实现Definition/Spec/Context/Active container、capture/execution、duration/period/stack/inhibition。
- 接fixed scheduler、journal、save snapshot基础，建立deterministic effect oracle。

### M3 · Gameplay Ability、Task与Targeting

- 实现Spec grant/revoke、activation lifecycle、cost/cooldown/group/event。
- 实现Task owner/cancel/timeout与typed TargetData/validation/confirmation。

### M4 · Gameplay Cue与Domain Adapter

- 建立Cue lifecycle与Animation/Physics/Navigation/Audio/VFX/UI adapter registry。
- 所有presentation只消费journal/request，不拥有simulation truth。

### M5 · Authority、Prediction与Replication

- 实现role/security、prediction ledger、accept/reject/caught-up、rollback/replay和dedup。
- 在generic RPC/replication上定义domain schema、budget、late join、disconnect与server validation。

### M6 · Scene、Save、Replay、Reload与Plugin Lifecycle

- 完成Scene/Prefab binding roundtrip、Save participant、replay journal、travel与owner teardown。
- compiler generation、LKG、active migration、plugin quiesce和late-result fence闭环。

### M7 · WOC迁移、Editor真相与产品资格

- Runtime13的WOC规则编译到统一artifact，删除`WorldState`/独立module平行Ability authority。
- Editor21接shared document/compiler/provider/debug snapshot；Client/Server/Editor/PIE/cook/package通过端到端门。

### M8 · Scalability、Fault与领先性

- 完成1/100/1k/100k规模、prediction storm、effect/tag/task压力、fault/fuzz/soak/cross-platform determinism。
- 与Unreal同源同规则比较correctness、P50/P95/P99、RSS、alloc、network bytes和artifact size；只按raw evidence声明领先。

## 11. Runtime151复验门（40项）

### Authority、Source与Artifact

- [ ] GAS-G01 `Fail`：provider缺失时Runtime/App/Editor统一Unavailable且没有固定成功文本。
- [ ] GAS-G02 `Fail`：唯一Runtime owner持有Ability/Effect/Attribute/Tag/Prediction truth，JSON/WOC/UI无影子authority。
- [ ] GAS-G03 `Partial`：generic asset/generation可复用；五类source能否import/cook/load/LKG/retire尚未通过。
- [ ] GAS-G04 `Fail`：Scene/Prefab owner/spec binding跨save/reopen/duplicate/remap/merge无损。
- [ ] GAS-G05 `Fail`：WOC规则迁移到同一artifact且旧分支/module authority已删除。

### Tag与Attribute

- [ ] GAS-G06 `Fail`：dictionary source/redirect/stable ID/parent/net index在cook与runtime一致。
- [ ] GAS-G07 `Fail`：container count和query exact/parent/all/any/none覆盖空/深度/预算/invalid generation。
- [ ] GAS-G08 `Fail`：Attribute Set base/current/schema migration/save/replication roundtrip成立。
- [ ] GAS-G09 `Fail`：aggregator apply/remove/inhibit/rollback保持相同结果和source provenance。
- [ ] GAS-G10 `Fail`：pre/post hooks与cross-attribute transaction不暴露半提交状态且重入有界。

### Effect与Schedule

- [ ] GAS-G11 `Fail`：instant/infinite/duration/period/expiry/remove共享单一terminal lifecycle。
- [ ] GAS-G12 `Fail`：stack/overflow/refresh/period reset/inhibition矩阵可重复且幂等。
- [ ] GAS-G13 `Fail`：requirements/immunity/query removal返回structured result并无全量轮询。
- [ ] GAS-G14 `Fail`：snapshot/live capture和execution calculation在source/target/tag变化下符合oracle。
- [ ] GAS-G15 `Partial`：fixed clock/plan存在；Ability phase与Effect due/catch-up/pause尚未接入。

### Ability、Task与Target

- [ ] GAS-G16 `Fail`：grant/revoke/upgrade/remove-on-end和spec generation矩阵通过。
- [ ] GAS-G17 `Fail`：Can/Try/Activate/CommitCost/Cooldown/End/Cancel及重复terminal通过。
- [ ] GAS-G18 `Fail`：Task完成/取消/超时/world retire/plugin unload没有晚到写入。
- [ ] GAS-G19 `Fail`：Gameplay Event ordering/consume/payload budget和activation trigger通过。
- [ ] GAS-G20 `Fail`：InputUser/Action到spec request可重绑、可归属且不轮询global raw key。
- [ ] GAS-G21 `Fail`：entity/point/area/hit/custom TargetData codec、budget与unknown schema通过。
- [ ] GAS-G22 `Fail`：range/LOS/team/alive/authority在admission、commit、impact按policy重验。

### Cue、Adapter与Script

- [ ] GAS-G23 `Fail`：Cue Executed/Added/WhileActive/Removed、dedup、pool和terminal cleanup通过。
- [ ] GAS-G24 `Partial`：Animation/VFX/Audio/Nav/UI底座存在；尚无Gameplay journal驱动的typed adapter闭环。
- [ ] GAS-G25 `Fail`：script self/query/target/spawn/despawn capability和generation/owner/role矩阵拒绝越权。

### Network、Prediction与Reconciliation

- [ ] GAS-G26 `Fail`：execution/security policy与owner connection/server validation在Client/Server产品生效。
- [ ] GAS-G27 `Fail`：PredictionKey生成、scoped/dependent window、wrap/reuse/cross-owner防护通过。
- [ ] GAS-G28 `Fail`：accept/reject/caught-up、乱序/重复/丢包/late result能回滚并重放完整side effect。
- [ ] GAS-G29 `Partial`：generic RPC/sync/interest/budget/late join存在；无Ability domain schema与caller。
- [ ] GAS-G30 `Fail`：late join/reconnect能还原spec/effect/tag/attribute/cooldown并不重放一次性Cue。

### World、Save、Reload与Diagnostics

- [ ] GAS-G31 `Fail`：World/owner spawn、possession、travel、despawn、close与teardown释放所有task/timer/lease。
- [ ] GAS-G32 `Fail`：Save/reopen、checkpoint rollback和replay逐tick journal结果一致。
- [ ] GAS-G33 `Partial`：generic generation/currentness存在；active build-set migration与LKG cutover未实现。
- [ ] GAS-G34 `Fail`：plugin quiesce/unload/crash不会遗留callback、active execution或presentation实例。
- [ ] GAS-G35 `Partial`：generic diagnostics存在；无bounded owner/effect/prediction snapshot和causal receipt。

### Performance、Product与领先性

- [ ] GAS-G36 `Fail`：ordinary grant/activate/effect tick/tag query热路径无JSON、字符串解析和稳态heap allocation。
- [ ] GAS-G37 `Partial`：net已有count/byte budget；其余owner/effect/task/cue/prediction/CPU/alloc预算为空。
- [ ] GAS-G38 `Fail`：跨线程、跨平台、save/replay与network correction保持deterministic digest。
- [ ] GAS-G39 `Fail`：Editor Ability/Effect/Tags author/save/compile/play/debug全部消费Runtime truth，无固定fixture冒充结果。
- [ ] GAS-G40 `Fail`：与Unreal同源同规则correctness相同后，吞吐/延迟/RSS/network bytes有可复核领先证据。

Gate复算：Partial为G03、G15、G24、G29、G33、G35、G37，共7项；其余33项Fail，0项Pass。

## 12. 首个允许实施的测试设计

MVP-00与M0产品真相关闭后，第一批实现不能从继续扩展Workbench UI开始，应先提交Runtime RED oracle：

1. `gameplay_tag_dictionary_redirect_query_oracle`：source precedence、case conflict、redirect cycle、parent/exact和compiled query digest。
2. `attribute_aggregator_apply_remove_rollback_oracle`：base/current、多channel、qualifier、clamp、NaN/Inf和source removal可逆。
3. `effect_duration_period_stack_inhibition_matrix`：execute-on-apply、refresh/reset、missed tick、overflow、pause/resume与single terminal。
4. `effect_capture_snapshot_live_execution_oracle`：source/target attribute/tag在Spec创建与execution间变化，结果符合capture policy。
5. `ability_grant_activate_commit_cancel_matrix`：grant/revoke、requirements、cost reservation、cooldown、group、end/cancel与structured failures。
6. `ability_task_target_world_retire_oracle`：async target/task在cancel、despawn、travel、plugin unload后不能提交晚到结果。
7. `prediction_accept_reject_dependency_oracle`：nested keys、dependent side effect、乱序/重复/丢包/reject/caught-up和Cue dedup。
8. `ability_replication_late_join_oracle`：spec、active effect、attribute/tag、cooldown、task terminal与一次性Cue恢复正确。
9. `gameplay_save_reload_replay_digest_oracle`：active timer/spec revision/build set跨save/reopen和逐tick replay一致。
10. `gameplay_script_capability_authority_matrix`：self/target/query/spawn/despawn、stale generation、cross-owner/role全部fail-close。
11. `woc_same_rule_old_new_differential_oracle`：迁移期只在test harness双跑，首次差异给出ability/effect/opcode/receipt；产品不双写。
12. `gameplay_scale_and_unreal_competition_suite`：1/100/1k/100k owner、effect/prediction storm，记录P50/P95/P99、RSS、alloc、bytes和artifact size。

任何测试不得以control ID、ZUI文本、descriptor数量、DTO roundtrip、mock JSON HP或ignored benchmark作为产品通过证据。

## 13. Review closeout

| 项目 | 状态 | 证据 |
|---|---|---|
| Runtime owner split | review_complete | Runtime151拥有engine GAS执行链；Editor21、Runtime13、99zo、99zd保留各自authoring/content/substrate边界 |
| Zircon current source | review_complete | 110 selected文件；host/asset/scene/time/net/editor/WOC逐层追踪，18个领域词法命中均为Editor control string |
| Unreal primary reference | review_complete | 28文件，覆盖owner/spec/ability/effect/aggregator/attribute/tag/query/cue/task/target/prediction和tests |
| Other four references | review_complete | 只采纳ECS/asset/plugin/scene/authority/VFX adapter边界，不稀释Unreal GAS合同 |
| Runtime08G父P0 | 5 Open / 0 Partial / 0 Closed | false product、fake prediction、JSON HP、broad capability、asset/product lifecycle均未关闭 |
| Runtime151 P1 | 57 Open / 15 Partial / 0 Closed | Partial仅为可复用通用底座 |
| Runtime151 P2 | 16 Open | 无领先功能达到实现入口 |
| Runtime151 Gates | 33 Fail / 7 Partial / 0 Pass | 无端到端Ability/Effect/Tag/Attribute/Cue/Prediction gate通过 |
| 动态验证 | not_run | review-only；未运行Cargo、Editor/App、Client/Server、PIE、cook、save/replay、fault/scale/soak/profile或竞争benchmark |

实施前必须重新读取本报告、Runtime08G、Editor21、Runtime13与最新source，重算六个selected-set fingerprint，并检查相关failure handoff。任何gameplay host、ResourceKind/asset、World/Scene、fixed time、RPC/replication、save、WOC combat或Workbench变更，都必须重跑对应P1、Gate与首批RED oracle；不能根据报告日期或UI可见性推断currentness。
