---
title: First-Party AI Source、Runtime、Editor、Dist、Catalog、Behavior Tree、Blackboard、Perception、EQS 与 Product Integration 工程化差距
category: zircon_plugins
report_id: Plugins15
review_date: 2026-08-19
baseline_head: 25e09a23178000f2e783ce2143cf70a8b118d404
baseline_epoch: 333
related_code:
  - zircon_plugins/ai/plugin.toml
  - zircon_plugins/ai/runtime/Cargo.toml
  - zircon_plugins/ai/runtime/src
  - zircon_plugins/ai/editor/Cargo.toml
  - zircon_plugins/ai/editor/src
  - zircon_plugins/ai/editor/behavior_tree.zui
  - zircon_plugins/ai/editor/perception_debug.zui
  - zircon_plugins/ai/dist/Cargo.toml
  - zircon_plugins/ai/dist/src
  - zircon_runtime/src/core/framework/ai
  - zircon_runtime/src/core/framework/script/behavior_bridge.rs
  - zircon_runtime/src/script/vm/behavior_bridge.rs
  - zircon_plugins/first_party_runtime_catalog/Cargo.toml
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/first_party_editor_catalog/Cargo.toml
  - zircon_plugins/first_party_editor_catalog/src
  - zircon_app/Cargo.toml
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_app/src/entry/first_party_editor_plugins.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/augmentation/capabilities.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_classification/runtime/systems.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog/core_rows/runtime/systems.rs
  - examples/vampire/README.md
  - examples/vampire/assets/data/enemy_behavior_tree.toml
  - examples/vampire/assets/data/enemy_behavior_tree.toml.zmeta
  - examples/vampire/assets/scenes/main.scene.toml
  - zircon_editor/assets/icons/zircon_engine_style/graph/behavior-tree.svg
  - zircon_editor/assets/icons/zircon_engine_style/graph/blackboard.svg
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/ai/workbench_behavior_workspace.zui
  - zircon_editor/assets/ui/editor/components/workbench/modules/core/ai/workbench_perception_workspace.zui
tests:
  - zircon_plugins/ai/runtime/src/tests
  - zircon_plugins/ai/editor/src/tests.rs
  - zircon_plugins/ai/dist/src/lib.rs
  - zircon_runtime/src/dynamic_api/session/tests/vampire_gameplay.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_plugins/06-first-party-plugin-source-editor-runtime-dist-catalog-profile-capability-closure-review.md
  - docs/plans/optimize/zircon_plugins/07-first-party-asset-importer-source-dependency-subasset-artifact-determinism-sandbox-product-integration-review.md
  - docs/plans/optimize/zircon_plugins/08-first-party-editor-authoring-extension-document-operation-toolkit-runtime-contract-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/08f-ai-behavior-tree-blackboard-perception-runtime-review.md
  - docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/42-builtin-runtime-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-review.md
  - docs/plans/optimize/zircon_editor/20-ai-behavior-tree-blackboard-perception-eqs-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/50-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-product-integration-review.md
  - docs/plans/zircon_plugins/06-ai.md
  - docs/plans/zircon_plugins/06/2026-07-13-ai-m1-output-records.md
  - docs/plans/zircon_plugins/06/2026-07-13-ai-m2-output-records.md
  - docs/plans/zircon_plugins/06/2026-07-15-ai-m3-integration-task-output-records.md
  - docs/plans/zircon_plugins/06/2026-07-16-ai-m4-perception-output-records.md
  - docs/plans/zircon_plugins/06/2026-07-28-ai-m5-editor-debug-validation-manifest.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Classes/BehaviorTree/BehaviorTreeComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/BehaviorTree/BehaviorTreeComponent.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Classes/BehaviorTree/BlackboardComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/BehaviorTree/BlackboardComponent.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Classes/Perception/AIPerceptionSystem.h
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/Perception/AIPerceptionSystem.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Classes/EnvironmentQuery/EnvQueryManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/EnvironmentQuery/EnvQueryManager.cpp
  - dev/UnrealEngine/Engine/Source/Editor/BehaviorTreeEditor/Private/BehaviorTreeEditor.cpp
  - dev/UnrealEngine/Engine/Source/Editor/BehaviorTreeEditor/Private/BehaviorTreeDebugger.cpp
  - dev/UnrealEngine/Engine/Plugins/AI/EnvironmentQueryEditor/Source/EnvironmentQueryEditor/Private/EnvironmentQueryEditor.cpp
  - dev/Fyrox/fyrox-impl/src/utils/behavior/mod.rs
  - dev/Fyrox/fyrox-impl/src/utils/behavior/composite.rs
  - dev/Fyrox/fyrox-impl/src/utils/behavior/leaf.rs
  - dev/bevy/crates
  - dev/godot
  - dev/Graphics/Packages
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 15 · First-Party AI Source、Runtime、Editor、Dist、Catalog、Behavior Tree、Blackboard、Perception、EQS 与 Product Integration 工程化差距

## 1. 结论

`zircon_plugins/ai`不是完全空壳。runtime已有编译后的dense behavior tree、标准节点目录、typed dense Blackboard、generation与observer、owner-aware registration/revocation、受限hearing ingress、弱引用physics/script imports、typed Perception组件、tick LOD、debug snapshot以及98项局部测试。Editor也有operation descriptor、graph/palette contribution、runtime mirror与overlay controller。这些是值得保留的工程底座，不能因为产品链路不完整而全部推倒。

但当前AI没有进入普通产品的默认执行路径。`first-party-runtime-plugins`能够链接AI runtime，可是普通`target-client`没有启用该feature；`target-editor-host`只显式链接advanced rendering、Navigation runtime/editor与Neural editor；`target-server`也没有启用AI contracts/provider。runtime catalog在AI feature被链接时可以发现provider，editor catalog却没有AI feature或provider，App editor composition也没有AI editor分支。因此manifest声明的Client、Server、Editor支持，与实际BuildSet中的provider可用性不一致。

Source runtime与NativeDynamic也不是同一能力。`dist`只导出ABI descriptor和registration manifest，明确让runtime services继续由embedded runtime module托管；它是stateless、state schema 0，command/event manifest为空，也没有invoke、save/restore、unload、bridge或host-ready实现。runtime侧native projection同样声明空systems/events/extensions。当前NativeDynamic只能证明元数据可装载，不能独立提供行为树、Blackboard或感知语义。

最直接的产品断点在示例资产。仓内唯一作者化行为树`enemy_behavior_tree.toml`被通用TOML importer标记为`Data`，而不是AI source asset或compiled artifact；它使用`version`、`root`、`kind = "condition"`、`action`、`result`和顶层`[blackboard]`，当前descriptor却要求`format_version`、`root_node`、decorator枚举、`implementation`、`display_name`与typed `parameters`。没有生产caller加载或注册它，场景只保存`behavior_tree = "graveyard_enemy_bt"`字符串，README还明确说明脚本只是镜像这棵树。唯一相关gameplay测试被`#[ignore]`，且测试的是动态脚本，不是AI资产编译、场景装载和runtime执行。

Runtime内部也尚未形成工程级world authority。manager是单个`Arc<Mutex<AiRuntimeState>>`；行为树、schema与Blackboard只能由API/test手工注册，仓内没有scene Agent/Brain/BehaviorTree/Blackboard组件把实体生命周期投影进manager。compiler和executor递归且没有深度、节点、时间或内存预算；每次tick克隆目录并无条件构造全量debug snapshot。Perception每tick两次扫描全World、按receiver×source计算，只有固定256 pair上限；可选physics不可用时Sight反而刷新为可见。EQS则没有生产类型、query service、scheduler或artifact，只有Editor预览文字。

Editor20已经拥有AI authoring的5项P0，Runtime08F拥有runtime语义，Plugins01/06拥有package/catalog/native ABI，Plugins07拥有asset/cook，Runtime42与App拥有target/provider composition。本篇不重复累计这些父报告P0，登记 **0项新增P0、48项P1、12项P2**；本篇唯一拥有AI单包从manifest、source runtime、Editor、dist、catalog、App target、示例asset到参考后端的纵向交付合同。

## 2. 审查边界、规模与currentness

### 2.1 物理冻结

| 范围 | 文件 / 行 / bytes / tests | 冻结事实 |
|---|---:|---|
| canonical AI | 86 / 17,667 / 609,467 / 109 | package runtime 64、editor 10、dist 2、manifest 1、framework/bridge 9 |
| plugin runtime | 64 / 14,750 / 512,057 / 98 | behavior tree、Blackboard、manager、perception、plugin registration与scenario tests |
| plugin editor | 10 / 1,654 / 58,169 / 9 | 7个Rust文件、2份ZUI和Cargo manifest |
| dist / manifest | 2 / 115 / 4,087 / 2；1 / 122 / 4,495 / 0 | metadata-only NativeDynamic shell与package声明 |
| framework AI | 9 / 1,026 / 30,659 / 0 | neutral contracts、snapshot、tick、ids、error与manager API |
| 选定纵向链 | 113 / 23,528 / 824,401 / 137 | canonical AI外加App、catalog、builtin rows、Editor product assets、Vampire example与runtime integration |
| package fingerprint | `ecba84bc9e896e4c3633cd3b35e4d1780767e8b3c0c0c2b5fc0a3e05724d6f1b` | 113个selected tracked path排序，以小写path、空格与file SHA-256组成LF串，无末尾LF后重算SHA-256 |

源revision为`25e09a23178000f2e783ce2143cf70a8b118d404`，coordinator baseline epoch为333。113个selected path在冻结时没有tracked working-tree差异；但App、Editor、共享Runtime与计划文档存在其他会话或用户改动，所以本文保持`source_recheck_required: true`。实施前必须在同一BuildSet重算package、App features、runtime/editor catalog、builtin rows、示例asset和所有父owner状态。

### 2.2 历史计划完成不等于当前产品完成

`docs/plans/zircon_plugins/06-ai.md`把M1、M3、M4标为完成，M2有完成记录，M5仍未完成；目录下没有开放`failure-*.md`。历史M2曾记录58/58局部结果，后续记录也证明managed package基础曾通过验证。但这些记录没有覆盖当前普通Client/Editor/Server链接矩阵、Editor catalog provider、NativeDynamic运行能力、示例资产schema、scene binding、EQS或真实产品authoring闭环。

因此本报告不撤销历史证据，也不把它提升为当前产品资格。父计划记录回答“当时局部milestone是否通过”，本报告回答“当前源树能否从asset到target形成同一AI产品”。两个问题必须分别保留。

### 2.3 测试库存不等于交付资格

137项selected test attribute中，runtime 98、editor 9、dist 2，其余来自catalog、App与runtime integration。现有测试对compiler、node catalog、Blackboard store/observer、abort、perception、tick LOD、registration、debug frame和isolated Editor controller有价值。

仓内没有AI Criterion benchmark、property/fuzz、concurrency model、long soak、1k agents、双World/PIE、save/load、network/replay、NativeDynamic parity、Editor visual workflow或真实asset-to-scene-to-runtime测试。唯一Vampire gameplay test被ignore，且不消费AI package。本轮只做E3静态审查，没有运行Cargo、App、Editor或NativeDynamic测试。

## 3. 当前真实产品链与断点

~~~text
ordinary zircon_app Client
  -> target-client
  -X first-party-runtime-plugins/base-runtime-plugins not enabled
  -X AI source provider not linked

zircon_app Editor Host
  -> advanced rendering + Navigation + Neural composition
  -X AI runtime/editor feature and provider absent
  -> first-party editor catalog silently returns no AI contribution

zircon_app Server
  -> manifest declares AI Server support
  -X AI contracts/provider not selected by target-server

AI source package when explicitly linked
  -> runtime catalog can discover AiRuntimePlugin
  -> global Arc<Mutex<AiRuntimeState>>
  -> manual register tree/schema/blackboard/tick calls
  -X no scene Brain/Agent/Tree/Blackboard lifecycle projection

Vampire example
  -> generic Data TOML with stale/incompatible AI schema
  -> scene stores an untyped behavior_tree string
  -> scripts manually mirror behavior
  -X importer/compiler/artifact/registry/runtime chain absent

NativeDynamic dist
  -> ABI descriptor + registration manifest
  -X no commands/events/state/bridges/lifecycle/runtime behavior
~~~

这条链说明“AI crate可编译”“局部manager tests存在”“manifest声明三个target”和“普通游戏能作者化、cook、装载、调试并稳定运行AI”是四件不同的事。当前只证明了前两件的局部基础。

## 4. 应保留的底座

| 基础 | 保留理由 | 收敛条件 |
|---|---|---|
| dense compiled tree | 已把作者描述映射为紧凑node index与child range | 增加schema/version、预算、diagnostic、artifact与迁移，禁止每tick重建/克隆目录 |
| typed Blackboard layout/store | 有typed slot、generation、observer与owner-aware binding | 变成per-agent/per-world generation owner，补key selector、parent/schema migration、save/network语义 |
| node catalog与owner revocation | 比硬编码全局函数表更接近plugin lifecycle | capability必须声明节点语义、latent/abort/thread/serialization限制并支持原子generation切换 |
| execution state/abort基础 | 已有running state、condition abort与observer绑定测试 | 替换递归无预算执行，建立instance stack、node memory、latent message和scheduled continuation |
| perception typed components | source/receiver descriptor比动态字符串更可维护 | 接入稳定实体generation、sense config、listener/source lifecycle、spatial query与forgetting |
| bounded hearing ingress | 已承认外部stimulus需要容量边界 | 将items/bytes/age/priority/drop receipt统一到所有sense与debug stream |
| weak physics/script imports | 避免包间强耦合方向正确 | missing dependency必须按facet fail-close/degrade receipt，不能把Sight判成可见 |
| runtime mirror/overlay controller | 为Editor调试提供了明确入口 | 改为selection-aware、budgeted、versioned trace consumer并由产品构造 |

## 5. 参考实现约束

### 5.1 Unreal是主参考，不是类名清单

`UBehaviorTreeComponent`维护instance stack、pending execution、restart/stop/pause/resume、branch evaluation、auxiliary/parallel node、latent task message与node memory；`UBlackboardComponent`维护key memory、key instance、observer registration/notification、parent data与初始化/销毁；Perception system管理listener、source、sense、aging、forgetting、affiliation和dominant sense；`UEnvQueryManager`管理running/external query、query ID、querier cleanup、abort、instance cache与time slicing。BehaviorTreeEditor和EnvironmentQueryEditor是独立真实模块，拥有graph schema、factory、details、find/diff与debugger，而不是几份静态widget资源。

Zircon不需要逐类复制Unreal，但必须达到同级合同：instance lifecycle、bounded scheduling、latent completion、可观察Blackboard、sense lifecycle、query artifact、time slice、authoring/compile/debug闭环以及明确的world/owner/generation。

### 5.2 其他仓内参考的适用边界

Fyrox的`utils/behavior`提供serializable BehaviorTree、Sequence、Selector、Leaf、Inverter和typed context mutation，适合验证紧凑组合节点API；它不是Unreal级Perception、EQS或Editor基线。其visitor blackboard是序列化上下文，不能误写为AI Blackboard。

本地Bevy core、Godot core和Unity Graphics在本次扫描范围内没有可作为Behavior Tree、AI Perception或EQS产品基线的domain implementation。Bevy可参考调度/ECS，Godot可参考通用scene/object生命周期，Unity Graphics只适合render/debug可视化边界；任何AI能力结论都不能从这些负向匹配外推。

## 6. P1：Package、Target、Catalog 与资产产品链

| ID | 当前差异 | 必须重构 |
|---|---|---|
| NAI-P1-001 | manifest声明Client/Server/Editor，普通三个target却没有一致链接AI provider | 建立target/profile/carrier/provider矩阵和build-time admission；声明支持的target必须有同contract provider或明确fail-close |
| NAI-P1-002 | AI只随`base-runtime-plugins`可选链接，`target-client`默认不启用 | 让产品profile显式选择required/optional AI；startup receipt记录linked、selected、effective与degraded reason |
| NAI-P1-003 | `target-server`没有AI contracts/runtime provider | 定义server-authoritative AI profile、headless依赖与deterministic tick合同，并进入server qualification |
| NAI-P1-004 | Editor Host没有AI runtime/editor feature组合 | 在App composition建立唯一AI feature owner，保证Editor authoring连接同BuildSet runtime provider |
| NAI-P1-005 | runtime catalog有条件route，editor catalog完全没有AI provider | 增加AI editor feature/provider/factory并对required contribution缺失报错，禁止silent `None` |
| NAI-P1-006 | capability只表达宽泛BT/Blackboard/Perception，不能证明EQS、authoring、debug或carrier parity | 改为逐facet capability、limits、schema、provider、carrier与evidence receipt |

| NAI-P1-007 | 唯一示例树被通用TOML importer归为`Data` | 建立AI source asset importer、typed source schema、dependency graph与compiler owner |
| NAI-P1-008 | 示例TOML与当前descriptor字段和枚举不兼容 | 选择硬迁移或受版本管理迁移器；旧schema必须产生定位明确的diagnostic，不能静默忽略 |
| NAI-P1-009 | 场景只保存`behavior_tree`字符串，无stable asset handle或generation | 引入Brain/Agent组件和typed behavior artifact reference，随scene instantiate/despawn/reload投影生命周期 |
| NAI-P1-010 | README说明脚本镜像树，形成脚本与AI双真相 | 把示例迁到真实compiled tree执行；脚本只能实现task/service，不得复制决策图 |
| NAI-P1-011 | 没有tree/schema从asset registry到manager的生产caller | 建立load/validate/compile/cache/register/activate/retire链和可恢复receipt |
| NAI-P1-012 | 唯一gameplay test被ignore且不测AI plugin | 新增非ignore产品scenario：import、cook、scene load、tick、Blackboard、perception、reload与shutdown |

## 7. P1：Behavior Tree 编译、实例与执行

| ID | 当前差异 | 必须重构 |
|---|---|---|
| NAI-P1-013 | compiler递归`compile_subtree`，无节点数、深度、child、字符串或bytes预算 | admission阶段验证schema、cycle、depth、counts、bytes、parameter types与diagnostic budget |
| NAI-P1-014 | tree/schema注册后不可作为原子artifact generation替换 | 引入immutable compiled artifact、generation handle、dependency digest、last-good activation和retirement |
| NAI-P1-015 | manager每tick克隆tree/node catalog等共享数据 | 使用immutable Arc generation、stable node dispatch table与per-instance mutable state，steady tick禁止全目录克隆 |
| NAI-P1-016 | executor递归求值且没有node/time/depth budget | 改为显式execution stack和budgeted scheduler，支持yield、deadline、cancel与continuation |
| NAI-P1-017 | Parallel只是deterministic descriptor fold，不是真正并行分支生命周期 | 定义child policy、completion threshold、abort propagation、node memory与latent branch schedule |
| NAI-P1-018 | latent task没有标准request/message/completion通道 | 建立task ticket、owner/generation、message observer、timeout、cancel、finish receipt与world teardown |

| NAI-P1-019 | SetBlackboard、EmitEvent通过通用`evaluate_task`，语义可退化成静态result | 为内建节点提供typed implementation与effect receipt；静态测试结果不得替代真实副作用 |
| NAI-P1-020 | Service可由`service_result`直接决定结果，没有独立interval/phase/lifecycle | 定义service activation、interval、jitter、budget、tick、deactivation与failure policy |
| NAI-P1-021 | Wait可由`result`绕过时间语义 | 使用明确clock domain、wake deadline、pause/time-scale/save/replay语义，禁止静态成功字段 |
| NAI-P1-022 | PlayAnimation与Script调用是同步任务壳 | 接入animation/script owner的async ticket、thread affinity、completion/error/cancel和reload generation |
| NAI-P1-023 | MoveTo/UpdateBlackboardDistance依赖松散property/history event | 接入Navigation/Movement typed request与outcome，定义partial path、stuck、repath、cancel和authority |
| NAI-P1-024 | active node debug只表达tick时单点结果，无法重建决策过程 | 输出versioned execution trace：search path、condition、abort、task transition、message和timing |

## 8. P1：Blackboard、World 与生命周期

| ID | 当前差异 | 必须重构 |
|---|---|---|
| NAI-P1-025 | 单个`Arc<Mutex<AiRuntimeState>>`承载所有world/agent/tree/schema | 按World/PIE建立`AiWorld` owner和generation，registry与instance state分离，禁止跨World串扰 |
| NAI-P1-026 | 没有稳定entity generation/Brain owner绑定 | 使用stable entity handle、spawn/despawn/change projection和owner epoch，任何stale reference fail-close |
| NAI-P1-027 | tree/schema/entries主要由API/test手工调用 | 由scene/component lifecycle自动创建、绑定、更新和销毁agent instance，保留显式管理API作为工具层 |
| NAI-P1-028 | Blackboard snapshot/entry更新缺少统一transaction和provenance | 建立typed batch transaction、writer identity、generation、changed keys、observer phase与commit receipt |
| NAI-P1-029 | schema演进、parent/inheritance、key redirect与save migration未定义 | 建立schema ID/version、inheritance规则、compatible migration、default validation与拒绝路径 |
| NAI-P1-030 | Blackboard value未声明network/save/replay/ownership语义 | 每key声明authority、persistence、replication、determinism、privacy和debug projection policy |

| NAI-P1-031 | observer回调与abort调度缺少跨线程/重入/顺序合同 | 固定commit phase、observer queue、reentrancy policy、dedup和bounded cascade，配并发模型测试 |
| NAI-P1-032 | package unload/reload只覆盖registration owner，运行实例没有完整迁移 | quiesce实例、取消latent ticket、切generation或保留last-good，并输出终态receipt |
| NAI-P1-033 | world unload、agent despawn、tree replace与schema replace没有统一状态机 | 定义Created/Active/Stopping/Retired/Faulted生命周期和幂等cleanup |
| NAI-P1-034 | manager mutex把registry、tick、perception和debug串行化 | 拆分immutable catalogs、per-world scheduler、per-agent state和bounded ingress，明确锁顺序与contention预算 |
| NAI-P1-035 | tick LOD由active camera驱动，server/headless语义不稳定 | 由simulation significance owner提供target-neutral signal，server使用authority policy而非camera缺席默认Full |
| NAI-P1-036 | 无确定性RNG、clock、schedule或replay输入合同 | 接入Runtime22的clock/RNG/replay owner，记录artifact、inputs、schedule和external result provenance |

## 9. P1：Perception、EQS 与预算

| ID | 当前差异 | 必须重构 |
|---|---|---|
| NAI-P1-037 | Perception每tick两次`world.node_records()`全扫描 | 使用typed incremental source/listener projection、spatial index与dirty set，禁止steady full-world scan |
| NAI-P1-038 | receiver×source笛卡尔积只用固定256 pair截断 | 建立items/time/bytes/query budget、priority、fairness、continuation、latency SLO和truncation receipt |
| NAI-P1-039 | optional physics无结果时Sight刷新为可见 | dependency admission声明required/degraded policy；无LOS provider时返回unsupported/unknown而非visible |
| NAI-P1-040 | source/listener/sense缺少注册、更新、注销、aging和forgetting完整生命周期 | 引入stable listener/source handles、sense instances、age/forget policy、dominant sense与affiliation filter |
| NAI-P1-041 | stimulus与memory没有容量、bytes、age、dedup和drop可观察合同 | 建立bounded stimulus store、coalescing、expiration、priority、overflow metrics和drop receipt |
| NAI-P1-042 | hearing ingress虽有限制，但没有与其他sense共享统一scheduler | 统一external/internal sense ingress、phase、clock、budget、backpressure和shutdown drain |

| NAI-P1-043 | 生产代码没有EQS/query asset、manager、context、generator、test或score模型 | 建立typed EQS source/compiler/artifact/runtime service，而不是复用行为树字符串参数 |
| NAI-P1-044 | 没有running query ID、owner cleanup、abort或cache | 引入query handle/generation、querier lifetime、cancel/deadline、instance cache与terminal result |
| NAI-P1-045 | 没有time slicing和候选项/测试成本预算 | scheduler按generator/test cost和frame budget分片，提供partial/progress/truncated/fault receipt |
| NAI-P1-046 | perception与Navigation/Physics/World query没有稳定typed boundary | 定义batched spatial/LOS/path/context query接口、snapshot generation和dependency fault语义 |
| NAI-P1-047 | 每个behavior tick无条件生成全量runtime snapshot | debug订阅按需启用，使用delta/ring buffer、items/bytes/time/age预算与采样策略 |
| NAI-P1-048 | 现有qualification没有真实规模、失败与同workload基线 | 建立多agent、多sense、深树、latent、EQS、reload、双World、server与Editor trace corpus及perf archive |

## 10. P1 Owner 路由说明

本报告的48项P1由Plugins15负责纵向closure，但实现必须回到唯一owner，不能在package里复制父子系统：

| 领域 | Canonical owner | Plugins15的验收职责 |
|---|---|---|
| Behavior Tree / Blackboard / Perception runtime语义 | Runtime08F | 证明source package、scene、target和carrier实际消费同一runtime合同 |
| AI Editor authoring/debug/EQS UX | Editor20 | 证明editor catalog、factory、document、operation、asset与runtime trace形成产品闭环 |
| package、catalog、dist、Native ABI | Plugins01/06 | 证明AI的target/provider/carrier矩阵与effective capability真实一致 |
| asset import/cook/artifact | Plugins07及Runtime asset owners | 证明source tree/schema/EQS变成版本化artifact并能原子发布、装载、迁移 |
| App/builtin composition | Runtime42与App owner | 证明Client/Editor/Server各自显式选择唯一AI provider |
| identity/time/navigation/physics/script | Runtime22/24与各领域owner | 提供稳定handle、clock、movement/query/ticket合同，AI只消费接口 |

Editor20已有5项P0仍由Editor20唯一累计；AI editor catalog缺席和operation无factory也沿用该报告的P0语义。本报告没有发现需要重新编号的独立P0，不能用“0项新增P0”关闭或降低父报告阻断。

## 11. P2：竞争性能力

| ID | 能力 | 工程目标 |
|---|---|---|
| NAI-P2-001 | StateTree / data-oriented decision runtime | 在BT合同稳定后支持高并发状态树、编译数据布局、batch transition与processor schedule |
| NAI-P2-002 | Utility AI与HTN | 共享Blackboard/EQS/trace/artifact基础，提供可解释score与plan repair，不复制world/query owner |
| NAI-P2-003 | 完整EQS生态 | generator/context/test/score/debugger、async/time-sliced query、cache和authoring/diff工具 |
| NAI-P2-004 | Mass AI / crowd simulation | significance LOD、representation切换、spatial partition、group behavior与server authority |
| NAI-P2-005 | Smart Objects与交互预约 | searchable slot、claim ticket、behavior definition、animation/navigation handshake与contention policy |
| NAI-P2-006 | World knowledge与共享记忆 | squad/faction knowledge、belief age/confidence、authority、replication和privacy边界 |
| NAI-P2-007 | Learning/ML policy integration | 受版本管理model artifact、deterministic fallback、batch inference、deadline、telemetry与safety gate |
| NAI-P2-008 | Network prediction与replay | server authoritative decisions、client presentation/prediction、rollback和trace replay |
| NAI-P2-009 | Savegame与long-running world | latent task、Blackboard、perception memory、query与artifact generation的稳定序列化/迁移 |
| NAI-P2-010 | Visual AI profiler | decision flame/timeline、Blackboard diff、sense/EQS heatmap、budget overrun和remote capture |
| NAI-P2-011 | AI质量数据库 | randomized/property/fuzz、reference differential、scenario metrics、regression bisect与artifact archive |
| NAI-P2-012 | Large-world AI streaming | cell-aware brain suspend/resume、knowledge residency、query handoff、world partition与deterministic wake |

这些能力必须建立在唯一provider、真实asset、per-world generation、budgeted scheduler和产品Editor闭环上。新增空enum、没有consumer的descriptor或固定demo不计为P2进度。

## 12. 目标架构与硬切边界

~~~text
AI Source Assets
  BehaviorTree / BlackboardSchema / EQS / SenseConfig
        |
        v
AI Compiler
  schema + dependency + node capability validation
  depth/items/bytes/cost budgets + diagnostics
        |
        v
AI Cook Artifact
  version/checksum/provenance/provider ABI
  dense tree + BB layout + EQS plan + debug map
        |
        v
AI Activation Plan / Receipt
  target + carrier + provider + capability facets
        |
        v
AiWorld per World/PIE generation
  immutable artifact/catalog generations
  behavior scheduler + instance stacks + latent tickets
  blackboard transactions + observer queue
  perception listeners/sources/senses + spatial scheduler
  EQS query manager + cache + time slicing
        |
        +-> Navigation / Movement / Physics / Script typed requests
        +-> Save / Network / Replay projections
        +-> Editor trace stream with explicit budgets
~~~

硬切后只能有一个产品truth：scene引用compiled artifact，App选择唯一provider，runtime按World持有generation，Editor消费同一compiler与trace，NativeDynamic必须提供等价runtime能力或明确声明不支持。通用Data TOML、脚本镜像树、全局manager、静态result节点和metadata-only Ready都不允许作为兼容路径长期保留。

## 13. 分层重构里程碑

### M0 · Truth Freeze与Target矩阵

- 重算113文件fingerprint、父报告状态和Client/Editor/Server/carrier provider矩阵；
- 冻结新增字符串tree reference、generic Data AI asset、static-result节点和metadata-only Ready；
- 建立effective capability、缺provider fail-close与历史record/current source差异账本。

### M1 · Product Provider与per-world owner

- App/profile显式选择AI runtime/editor provider，editor catalog增加AI factory；
- manager硬切为per-World/PIE `AiWorld`，registry generation与agent instance分离；
- scene Brain/Agent组件驱动spawn/change/despawn，稳定entity generation贯穿所有ticket。

### M2 · Source Asset、Compiler与Artifact

- 建立BehaviorTree、Blackboard、EQS、SenseConfig importer和版本化source schema；
- compiler执行dependency、capability、depth/items/bytes/type/cost admission；
- 产出带checksum/provenance/debug map的immutable artifact，支持last-good、迁移与atomic publish。

### M3 · Behavior Scheduler与Blackboard

- 递归executor硬切为显式instance stack、node memory和budgeted scheduler；
- 完成latent ticket、parallel/service/abort/message/stop/restart/pause/resume合同；
- Blackboard引入transaction、observer phase、schema migration、save/network/replay policy。

### M4 · Perception与EQS

- source/listener/sense改为typed change projection与spatial scheduler；
- 实现aging/forgetting/affiliation/dominant sense和统一bounded stimulus ingress；
- 引入EQS artifact、query handle、generator/context/test/score、cache、abort与time slicing。

### M5 · Editor产品闭环

- 由产品catalog/factory构造AI toolkit、document、operation和runtime mirror；
- graph、Blackboard、Perception、EQS ZUI全部绑定真实provider和transaction；
- import/edit/compile/save/reopen/run/pause/step/debug/diff/error/retry形成同artifact闭环。

### M6 · Cross-system与Carrier

- Navigation/Movement/Physics/Animation/Script统一typed async request/outcome；
- Source/LibraryEmbed/NativeDynamic运行同一contract suite，或在admission明确拒绝缺失facet；
- world unload、plugin reload、bad artifact、timeout、fault与shutdown有终态receipt。

### M7 · Product Migration与示例

- Vampire树迁到当前source schema和AI importer，scene使用stable artifact handle；
- 删除脚本镜像决策和generic Data兼容路径；
- ordinary Client、Editor Host与Server通过同一patrol-detect-chase产品scenario。

### M8 · 规模与竞争性资格

- 建立深树、多agent、多sense、EQS、双World、reload、save/network/replay和long-soak workload；
- 归档CPU/frame latency、memory、contention、drop、query quality与failure evidence；
- 只有同场景、同硬件、同预算、同输出质量与同失败条件下才能声称达到或超过Unreal。

## 14. 资格门

| Gate | 验收内容 |
|---|---|
| G01 | Client、Editor Host、Server、Source、LibraryEmbed与NativeDynamic输出同schema `AiActivationReceipt` |
| G02 | 每World/PIE只有一个AI provider和一个`AiWorld` owner，无跨World global state串扰 |
| G03 | manifest target、App feature、catalog route与effective capability逐facet一致；required缺失fail-close |
| G04 | scene拥有typed Brain/Agent/Tree/Blackboard组件和stable artifact/entity generation |
| G05 | AI source importer拒绝generic Data和未知schema；diagnostic包含asset、field、version与迁移建议 |
| G06 | compiler验证cycle、depth、node/child/string/parameter/bytes/cost预算并输出immutable artifact |
| G07 | artifact具有magic/schema/checksum/provenance/provider ABI/dependency digest/debug map |
| G08 | tree/schema/EQS generation可原子publish、last-good fallback、retire，无tick期目录重建或全量克隆 |
| G09 | executor使用显式instance stack和node/time/depth预算，支持yield/cancel/deadline |
| G10 | parallel/service/abort/restart/stop/pause/resume具有独立状态机和组合测试 |
| G11 | latent task使用typed ticket/message/completion/timeout/cancel，并在world/plugin teardown终止 |
| G12 | SetBlackboard/EmitEvent/Wait/MoveTo/PlayAnimation/Script不存在静态result绕过真实语义 |
| G13 | Blackboard batch commit、observer ordering、reentrancy、cascade budget和writer provenance明确 |
| G14 | schema inheritance/migration/default/key redirect与save/network/replay policy通过资格 |
| G15 | agent spawn/change/despawn、tree replace、schema replace、world unload和reload幂等cleanup |
| G16 | steady tick无全World动态反序列化扫描，manager锁竞争满足明确预算 |
| G17 | AI significance是target-neutral simulation policy，server/headless不依赖active camera |
| G18 | Perception使用incremental listener/source projection与spatial index，无R×S无界扫描 |
| G19 | 所有sense共享items/bytes/time/age/priority/fairness/backpressure和drop receipt合同 |
| G20 | 缺Physics/LOS provider返回unsupported/degraded，不把未知对象判为可见 |
| G21 | listener/source/sense注册、更新、注销、aging、forgetting、affiliation与dominant sense通过 |
| G22 | EQS拥有source/compiler/artifact/runtime manager、query ID、owner cleanup、abort和terminal result |
| G23 | EQS generator/context/test/score按cost和frame budget time-slice，支持cache与partial progress |
| G24 | AI Editor由首方editor catalog发现，五个operation有真实factory/controller/provider |
| G25 | graph、Blackboard、Perception与EQS surface无`Space`/无provider占位，状态和错误可视化完整 |
| G26 | runtime trace按需、versioned、delta/bounded；overlay按selection/frustum/items/bytes/time预算 |
| G27 | Vampire asset由AI importer编译，scene装载真实artifact，脚本不再镜像决策树 |
| G28 | 非ignore产品scenario覆盖import/cook/load/tick/perception/Blackboard/reload/shutdown |
| G29 | Source/LibraryEmbed/NativeDynamic运行同一AI corpus；metadata shell不得宣称runtime Ready |
| G30 | 双World、深树、多agent、多sense、EQS、fault/reload/soak与server workload达到明确SLO |
| G31 | Editor20的5项P0和Runtime08F父finding按各自canonical gate关闭，本报告不得代替 |
| G32 | `git diff --check`、frontmatter/path、finding唯一性、fingerprint、索引/coverage与plan-output audit通过 |

## 15. 明确禁止的临时修复

1. 不通过给`target-client`随手追加feature就宣称产品闭环；target、provider、asset和资格必须一起成立。
2. 不把manifest中的Client/Server/Editor字符串或catalog可选route当作effective capability。
3. 不让NativeDynamic继续以空command/event、state schema 0和registration pointer宣称AI runtime支持。
4. 不保留通用Data TOML、scene字符串和脚本镜像树作为长期兼容路径。
5. 不为旧示例字段增加无版本serde alias来吞掉schema drift；迁移必须可诊断、可删除。
6. 不用更多静态`result`、history event或test fixture冒充SetBlackboard、EmitEvent、Wait、MoveTo等节点语义。
7. 不扩大256 pair或snapshot Vec来掩盖没有items/time/bytes/fairness合同。
8. 不让缺Physics的Sight默认可见，也不以silent fallback掩盖dependency fault。
9. 不在递归executor上叠加更多节点类型；先建立instance stack、budget、latent和abort合同。
10. 不以全局mutex、全World scan、全量clone和无条件debug snapshot换取局部测试易写性。
11. 不用`Space`、静态Table、descriptor或test-only controller冒充Editor authoring/debugger。
12. 不从Fyrox简化树、Bevy ECS、Godot core或Unity Graphics外推不存在的AI产品能力。
13. 不以137项test attribute或历史58/58记录关闭当前target/asset/product差异。
14. 不在同workload、同硬件、同预算、同质量和同失败条件前宣称超过Unreal。

## 16. 状态与产出记录

| 项目 | 状态 | 证据 |
|---|---|---|
| 物理扫描 | review_complete | canonical 86 files / 17,667 lines / 609,467 bytes；selected vertical 113 / 23,528 / 824,401 |
| 测试库存 | review_complete | selected 137项test attribute；缺产品E2E、benchmark、fuzz、soak和carrier parity |
| 产品纵向链 | review_complete | 三个ordinary target未形成AI provider闭环，editor catalog无AI，NativeDynamic为metadata-only |
| 资产链 | review_complete | Vampire generic Data TOML schema漂移、scene字符串引用、脚本镜像、无生产loader/compiler/registration |
| Runtime审查 | review_complete | 可保留dense tree/Blackboard/observer基础；global manager、递归无预算、全扫描与无EQS待重构 |
| Editor审查 | review_complete | descriptor/mirror/overlay存在；无产品factory/provider、ZUI占位、无完整authoring/debug chain |
| 参考实现 | review_complete | Unreal为主参考、Fyrox为紧凑树参考；Bevy/Godot/Unity Graphics按负向适用边界记录 |
| 本轮登记 | review_complete | 0 P0 / 48 P1 / 12 P2 / 32 gates；P0沿用Editor20与其他canonical owner |
| Production/tests修改 | pending | 本篇只写review与重构计划，没有修改production/tests或运行Cargo/App/Editor |

本报告完成的是AI纵向事实冻结和可验收重构设计，不是功能修复。下一阶段必须从M0/M1的target truth、唯一provider、per-world owner和真实asset chain开始，不能从补节点、扩固定容量或新增静态Editor资源开始。
