---
related_code:
  - zircon_runtime/src/core/framework/ai
  - zircon_plugins/ai/plugin.toml
  - zircon_plugins/ai/runtime/Cargo.toml
  - zircon_plugins/ai/runtime/src/behavior_tree
  - zircon_plugins/ai/runtime/src/blackboard
  - zircon_plugins/ai/runtime/src/manager
  - zircon_plugins/ai/runtime/src/perception
  - zircon_plugins/ai/runtime/src/plugin/registration.rs
  - zircon_plugins/ai/runtime/src/tick_lod.rs
  - zircon_plugins/ai/runtime/src/tests
  - zircon_plugins/ai/dist/src/lib.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/zircon_plugins/06-ai.md
  - docs/plans/zircon_plugins/06/2026-07-13-ai-m1-output-records.md
  - docs/plans/zircon_plugins/06/2026-07-13-ai-m2-output-records.md
  - docs/plans/zircon_plugins/06/2026-07-15-ai-m3-integration-task-output-records.md
  - docs/plans/zircon_plugins/06/2026-07-16-ai-m3-2-patrol-detect-chase-output-records.md
  - docs/plans/zircon_plugins/06/2026-07-16-ai-m4-perception-output-records.md
  - docs/plans/zircon_plugins/06/2026-07-28-ai-m5-editor-debug-validation-manifest.md
  - docs/plans/performance/01/2026-07-30-runtime-framework-animation-ai-navigation-tasks-static-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Classes/BehaviorTree/BehaviorTreeComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/BehaviorTree/BehaviorTreeComponent.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Classes/BehaviorTree/BlackboardComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/BehaviorTree/BlackboardComponent.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Classes/BehaviorTree/BTService.h
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Classes/Perception/AIPerceptionSystem.h
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/Perception/AIPerceptionSystem.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Classes/Perception/AISense_Sight.h
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/Perception/AISense_Sight.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Classes/EnvironmentQuery/EnvQueryManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/EnvironmentQuery/EnvQueryManager.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/StateTree/Source/StateTreeModule/Public/StateTreeExecutionContext.h
  - dev/Fyrox/fyrox-impl/src/utils/behavior/mod.rs
  - dev/godot/scene/2d/navigation/navigation_agent_2d.cpp
  - dev/godot/scene/3d/navigation/navigation_agent_3d.cpp
  - dev/bevy/crates/bevy_tasks/src/lib.rs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: false
---

# 08F · AI Behavior Tree / Blackboard / Perception Runtime 工程化差距

## 1. 结论

Zircon AI runtime 不是空壳。当前实现已经具备版本化 Behavior Tree DTO、树拓扑和参数校验、前序 dense compile artifact、18 项节点目录、typed extension owner、owner revoke execution gate、每 agent 节点运行态、Blackboard schema layout、按类型 dense storage、generation 与 changed-slot observer、Self/LowerPriority/Both abort、Sight/Hearing source/receiver、分帧 pair budget、刺激遗忘、容量 1024 的 hearing backlog、Sound/Animation hearing adapter、PhysicsQuery 与 ScriptBehaviorBridge 弱依赖、Navigation/Animation/Script 集成入口，以及运行时/Editor debug event。这些基础比临时 `if/else` AI demo 更扎实，重构应保留其 typed contract、compile-time validation、owner lease、确定性排序和有界入口，不能退回脚本每帧扫描场景的简易实现。

但是当前系统仍未形成任何可由普通项目内容启动的 AI 产品闭环。全仓 production caller 搜索只找到 `AiManager` 自己的注册方法；`register_behavior_tree`、`register_blackboard_schema` 和 `set_blackboard_entries` 除 tests 外没有调用者。`.btree.toml` compiler 没有 asset importer/cook/load consumer，scene 只注册 Perception Source/Receiver，没有 Brain/Agent/BehaviorTree/Blackboard component。所谓 active agent 只能由外部先手工构造 numeric handle 并直接 `tick_agent` 才会出现。换言之，当前测试证明的是 manager 算法可被测试代码驱动，不是场景、资产、插件和产品 host 能运行 AI。

旧计划对“18 个标准节点完成”的验收也明显过宽。`SetBlackboard`、`EmitEvent` 走通用 `evaluate_task`，只读取可伪造的 `result` 参数，不写 Blackboard、不发送事件；`UpdateBlackboardDistance` 只返回 `service_result`，既没有周期、抖动、宿主 composite，也不计算距离。语义矩阵正是给这些节点注入 `result`/`service_result` 后检查返回值，因此把名字存在当成能力完成。`BehaviorNodeDescriptor` 又没有参数 schema、pin、默认值、资源类型、side-effect、thread affinity 或调试 metadata；自定义 `BehaviorNodeRuntime` 的 context 只有只读参数/Blackboard/Perception和delta，甚至没有 entity/world/command sink，无法成为可用 gameplay task 扩展点。

执行器存在会遗留副作用的正确性缺口。TimeLimit 超时只返回 Failed，没有 abort 正在运行的 child；Parallel 以 Any 成功或 Any 失败终结时不取消仍运行的兄弟；plugin owner revoke 等待 in-flight tick 后直接删除 tree/instance/report，不调用 task abort。MoveTo 的 `clear_nav_target(None)` 实际把 destination 写成 agent 当前坐标，而中立 NavMeshAgent contract 的 destination 是 `Option<Vec3>`；当前 scene property API又不能表达 optional Vec3 清除，所以成功、失败和abort后都会留下一个新目标。MoveTo 还通过“NavAgentTickReport event storage是否存在”判断Navigation能力、每帧扫描全部已有report、用 `f32::EPSILON` 比较目标且没有 request/generation/cancel/repath ID。PlayAnimation 写parameter/trigger后立即成功；ScriptTask返回Running时每帧重新同步invoke，没有durable task、取消、timeout、预算或reload generation。

性能和生命周期同样不足。每个 agent tick 都 clone 全部已注册 CompiledBehaviorTree，而非当前tree的依赖闭包，并为所有tree的所有implementation取得owner lease；递归执行没有node/time/depth budget，selector reactive probe会重复遍历，abort的 `parent_of`逐节点全树搜索。公开 `AiManager: Send + Sync` 却允许同一agent并发tick：两个caller可先后remove同一blackboard/instance、各自执行并以后写覆盖前写。world/entity despawn、world unload/replacement、project close均没有 cleanup API或generation fence，Blackboard、Perception、active tree、instance和last report可永久残留并污染复用的 `(WorldHandle, u64)`。

Perception 的“Complete”声明不成立。每帧两次 `world.node_records()`、动态JSON parse、world transform查找、Vec重建和排序，再在全局 receiver×source 笛卡尔积上以固定256 pair round-robin；没有spatial index、最大感知延迟、time/physics-query/bytes预算。Physics bridge缺失、reload或调用失败时 Sight 明确按“可见”刷新，目标会穿墙；这不是可接受的shipping降级。刺激模型没有team/affiliation/tag、sense config、dominant stimulus、success/lost reason、source generation、listener update event或Damage/Touch/Custom provider；static hearing channel与event hearing又形成两条不同语义。

`ai.behavior_tick`还用 active camera 距离决定 Full/Half/Quarter频率。这会让相机移动改变真实AI决策时序，并在dedicated server无camera时全部Full；跳帧时间无cap/substep，恢复时一次大delta可跨越Wait/Cooldown/Perception/Script的语义边界。系统随后无条件调用全局 `runtime_snapshot()`，clone全部tree descriptor、agent Blackboard和Perception，再构造并发送完整 `AiBehaviorDebugSnapshot`；没有reader gate、changed slot/node delta、entry/bytes/age budget。debug路径即使没有Editor consumer也持续进入production frame，并且HashSet汇总使agent顺序不稳定。

本轮登记20项P1、5项P2，没有新增P0。P1先恢复能力真实性、asset/scene/world owner、迭代式有预算执行器、真实标准节点、latent task/cancel、Blackboard写入、typed Navigation/Animation/Script交接、可扩展Perception和按需debug。StateTree/utility/HTN、多阶段EQS、Mass AI/Smart Object、群体知识与深度network replay进入P2。完成这些重构和产品/规模验收前，历史M1-M4的绿色记录只能证明当时的局部tests通过，不能继续作为“AI runtime已工程化完成”或“Perception Complete”的依据，更不能支持性能优于Unreal的结论。

## 2. 审查边界与覆盖

### 2.1 已读范围

| 范围 | 文件 / 行数 / bytes | `#[test]` | 证据等级 |
|---|---:|---:|---|
| `core/framework/ai` | 9 / 1,026 / 30,659 | 0 | E3：tree、blackboard、perception、tick、snapshot、manager、id/error contract逐文件 |
| AI runtime production，不含`src/tests` | 44 / 7,979 / 273,401 | 0 | E3：compiler/catalog/executor、blackboard、manager、perception、plugin/system逐文件 |
| AI runtime tests | 20 / 6,771 / 238,656 | 98 | E3 inventory与代表断言逐项核对，未把test shape当产品证据 |
| AI dist | 2 / 115 / 4,087 | 2 | E3 ABI descriptor / E1 behavior：native projection无业务registration |
| generated plugin manifest | 1 / 122 / 4,495 | 0 | E3：capability、dependency、component、event、module与dist声明 |
| selected combined scope | 76 / 16,013 / 551,298 | 100 | 当前工作树fingerprint `8ef9320eb135d40cdbd79fef85debd57be6f0fe896456c69fb4fbca594f30971`，0 ignored，范围内无在途source |

行数使用物理文本行；fingerprint按相对路径排序，为每个当前工作树文件计算SHA-256，再对 `path<TAB>hash<LF>` 清单计算SHA-256。该值只用于实施前识别审查源漂移，不是release Build Set ID。当前整个仓库仍有其他Session/用户修改，本轮没有回退、吸收或修改任何production source。

产品调用搜索覆盖 `zircon_app`、`zircon_editor`、`zircon_runtime`、全部 `zircon_plugins` 和 `tools`，并排除AI tests。没有发现Behavior Tree/Blackboard production asset importer、scene Brain component、project bootstrap registration、world cleanup、entity despawn cleanup或gameplay activation caller。`compile_behavior_tree_toml`只有导出和定义；manager registration只有trait facade与tests。该负证据只说明当前first-party产品链不存在，不禁止未来第三方通过公开manager手工注册。

现有98个runtime test和2个dist test对compiler topology、节点返回状态、Blackboard layout/write、observer abort、integration adapter、perception budget/backlog、registration和patrol/detect/chase fixture有局部价值。但标准节点矩阵通过 `result`/`service_result`旁路真实副作用；integration tests直接构造manager/world/host；perception tests使用小型冻结world和mock occlusion；没有asset cook/load、普通scene启动、world replacement/despawn、并发same-agent tick、deep/wide tree budget、10k/100k agent、consumer stall、physics provider failure policy、dedicated server determinism、plugin reload期间latent task、soak、fuzz或跨平台产品证据。

### 2.2 参考边界

- Unreal BehaviorTreeComponent拥有active instance stack、known subtree instances、execution request、pending execution、latent abort、task finish和message observer；service有独立的relevance/tick interval生命周期。Zircon不需要复制UObject类层次，但必须吸收“prepared shared tree + per-agent active stack + queued execution change + latent task request/finish/abort + auxiliary service scheduler”的语义，而不是每帧无上限递归整条路径。
- Unreal BlackboardComponent把asset schema、key ID、value memory、key instance、observer register/unregister、notification pause/resume和owner lifecycle收在同一component。Zircon现有dense slot是正确方向，但必须补stable key identity、defaults/inheritance、typed asset/object/class/nav值、per-agent owner与mutation transaction，不能继续让公开API用完整`Vec<AiBlackboardEntry>`同步覆盖。
- Unreal AIPerceptionSystem是world subsystem，维护listener/source/sense registration和stimuli aging；AISense_Sight有importance排序、pending query、同步/异步trace上限和time slice。Zircon不必复刻其所有配置，但必须至少具备per-world registry、spatial candidate generation、可证明最大延迟、time/query budget、provider failure policy和lost stimulus语义。
- Unreal EQS manager有running/external query、query ID、abort、instance cache、time budget和querier cleanup；StateTree另有独立runtime/developer/editor/test边界。它们证明高级AI不是把更多字符串节点塞进同一个递归executor。Zircon先完成P1的基础query service，再将StateTree/Mass/高级EQS作为独立prepared program和scheduler层。
- Fyrox `utils/behavior`提供typed user behavior/context、pool handle、Visitor序列化和小型recursive tree。它适合作为轻量typed leaf与可序列化结构参考，不是observer abort、Perception、EQS或大规模scheduler的工程基线；不能用其较小功能面降低Unreal级目标。
- Godot本地参考树没有first-party Behavior Tree/Blackboard/Perception runtime；NavigationAgent的target/path反馈只可用于MoveTo边界对照。Bevy主仓没有first-party游戏AI，`bevy_tasks`只可参考executor/task pool。Unity `dev/Graphics`是渲染包，不拥有AI runtime。三者的缺席不是Zircon保留空能力的理由，也不能被错误当成AI完成度证据。

### 2.3 明确未做

- 没有修改AI/runtime/editor/navigation/animation/script production code，没有运行Cargo、Editor、client、dedicated server、native dist、性能、soak、fuzz或跨平台测试。本篇是current-source静态审查与重构计划，不是动态通过证明。
- 没有否定M1-M4全部产出。compiler、node owner gate、dense Blackboard、observer abort、bounded hearing ingress和weak bridge是应保留底座；本篇撤销的是由局部test外推到产品完成、性能完成或Perception Complete的结论。
- Editor graph、Blackboard panel、Perception overlay、simulation和Workbench由后续 `zircon_editor/20` 拥有；本篇只定义Editor必须消费的runtime asset/compiler/operation/debug contract，不把运行时缺口包装成UI任务。

## 3. 当前闭环与必须保留的能力

### 3.1 编译和节点目录已建立可迁移骨架

Behavior Tree会校验format version、空/重复ID、child count、missing child/root、multiple parent、cycle、unreachable node、implementation存在性和category一致性，再编译成前序node数组与direct-child index table。Typed extension catalog按owner注册并冻结为slot；execution gate在owner revoke时等待in-flight执行结束。这些机制应继续负责prepared program、owner generation和reload fencing，不应被Editor或脚本另建第二套节点目录。

### 3.2 Dense Blackboard与changed-slot observer值得保留

Blackboard按Bool/Integer/Scalar/String/Vec3/Entity分区，schema key预解析成slot，同值写不递增generation，完整同步会先校验再提交并收集changed slot。Observer把decorator预绑定到slot并在tick入口处理Self/LowerPriority/Both。这是正确的热路径方向；目标是在此基础上消除String type schema、full snapshot API和动态fallback，并补上stable schema/key generation与真正的node write command。

### 3.3 Perception入口已有公平性与背压意识

Sight/Hearing pair采用round-robin cursor，Sound/Animation/hearing bus共享单帧ingest limit，pending bus与adapter队列各自有容量和age，stimuli输出按sense/entity排序。Physics和Script通过weak `BridgeImport`而非依赖具体manager。后续应保留这些bounded/weak-owner原则，将固定pair计数提升为spatial/time/query/bytes多维预算，并把bridge unavailable从“默认可见”改为显式质量/错误策略。

## 4. P1 差距清单

### P1-1：AI没有asset-to-scene-to-runtime产品启动链，当前active agent只能由测试或手工manager调用产生

`compile_behavior_tree_toml`无production caller，Behavior Tree和Blackboard没有asset kind/importer/cooked artifact/load generation；scene只注册两个Perception component。没有 `AiBrain`/`AiAgent` component把entity绑定到Behavior Tree、Blackboard、Perception config和start policy，也没有scene system观察add/change/remove后注册/停用agent。目标建立BehaviorTreeAsset、BlackboardSchemaAsset、PerceptionConfigAsset和`AiBrainComponent`，由cook生成prepared program，world activation按组件增量建立agent，asset unload/change与component remove产生typed stop/replace outcome。

### P1-2：manager是跨World singleton，缺少world/entity generation和卸载/替换/despawn cleanup

Blackboard、Perception、active tree、instance和last report全部以 `(WorldHandle, u64)` 放在全局HashMap；除Perception每帧replace外没有world retire，entity消失也不清。World停止tick后旧状态永久存在，数值复用可污染新entity/world。目标由 `AiWorldKey { world, replacement_epoch }` 拥有 `AiWorldRuntime`，agent使用generational entity identity。World teardown执行StopAdmission、abort/cancel latent task、retire perception/query、drop blackboard/instance/debug和late-result fence；despawn/component removal走同一agent retire路径。

### P1-3：Behavior Tree和Blackboard只有register，没有update/unregister/reload/依赖图/稳定generation

两个catalog都用单调u64和Vec，duplicate ID拒绝后无法替换。Subtree必须按注册顺序引用已存在tree，只拒绝直接self reference，不建立跨tree dependency cycle；owner revoke只删除直接使用其slot的tree，引用被删tree的parent会留到runtime Blocked。目标让asset identity、revision、compiled generation、schema hash和dependency graph成为一等合同，支持prepare -> validate closure -> atomic publish/LKG -> migrate/restart agent -> retire old generation；unregister/reload必须处理subtree依赖闭包和stale handle。

### P1-4：每个agent tick克隆全部registered tree并租用全部owner，复杂度与隔离边界错误

tick在state锁内把所有CompiledBehaviorTree完整clone，并收集所有tree的implementation slot，不只当前tree可达闭包。tree descriptor含String和parameter Vec，agent数×catalog大小会放大clone；无关插件owner revoke也可令当前agent acquisition失败。目标为每个published program保存Arc/immutable dependency closure和预解析owner lease set；agent tick只clone廉价generation handle。catalog publish/revoke以program dependency index精确retire，禁止全catalog per-agent materialization。

### P1-5：`AiManager: Send + Sync`允许同一agent并发tick，remove/execute/reinsert会丢状态

tick把agent Blackboard和instance从global map移出后在锁外执行。第二个线程可同时取得default instance/空store并独立运行，两个结果随后last-writer-wins，observer generation、timer、latent task和report都会分叉。目标由per-agent execution token或world scheduler保证单写；公开direct tick返回Busy/StaleGeneration或排队ticket。禁止用户callback持有全局锁，且所有state publish用agent epoch compare-and-swap拒绝晚到结果。

### P1-6：递归执行器没有node/time/depth预算，reactive probe和abort存在重复遍历与主线程stall风险

compile、topology visit、normal evaluation、selector eligibility和subtree evaluation均递归；输入没有node/depth/parameter/string cap。每帧无最大node transitions、wall time或yield reason。selector可能为高优先级分支重复probe，abort的每个ancestor都用 `parent_of`全树扫描。外部node和ScriptTask又可同步阻塞。目标编译parent/subtree/observer表并使用显式active stack/continuation；每world/agent tick有node ops、task calls、wall time和diagnostic budget，超限返回Yielded而非改变业务状态。import/compile硬限node/depth/children/parameters/string bytes并用iterative算法防栈耗尽。

### P1-7：节点目录没有参数/端口/side-effect/lifecycle metadata，自定义节点context无法执行真实gameplay行为

`BehaviorNodeDescriptor`只有id/display/category/semantics/recheck/factory。参数合法性散落在manager硬编码字符串；Editor、cook和第三方plugin无法从同一schema生成控件、默认值、类型约束、Blackboard selector、asset picker或compat hash。External context没有entity/world、mutable Blackboard、event/command/task broker或cancel token。目标引入versioned `BehaviorNodeSchema`：typed inputs/outputs、required/default/range、blackboard value constraint、asset/interface dependency、instancing policy、thread affinity、latent/cancel/restart、determinism和debug fields。runtime factory只通过capability-scoped context和command sink工作，不泄漏任意World写权限。

### P1-8：三个“标准节点”没有名称承诺的副作用，`result`兼容参数让tests把placeholder判成完成

SetBlackboard/EmitEvent只返回静态result；UpdateBlackboardDistance只返回静态service result。Service按validator必须0 child，不能作为composite auxiliary按interval运行。`result`还能让MoveTo/PlayAnimation/ScriptTask完全绕过integration host。目标删除shipping descriptor中的结果注入旁路，测试fixture改用专用test node。实现Blackboard typed write/clear/copy、bounded typed event emit和真正的service attachment/schedule/jitter/relevance lifecycle；任何未实现标准node必须从catalog和capability移除，不能保留同名no-op。

### P1-9：TimeLimit、Parallel终结和owner revoke不保证abort所有仍运行任务，side effect可在分支结束后残留

TimeLimit达到阈值只清自身elapsed并返回Failed；Parallel满足Any成功/失败后只清terminal cache，不abort其他Running child；owner revoke直接drop agent instance。MoveTo destination、script coroutine、animation request和external runtime资源都可能继续存在。目标定义单一terminal transition：先冻结分支admission，按确定顺序向所有active leaf发Cancel/Abort，允许latent abort有deadline，收到terminal acknowledgement后才切换分支。Parallel policy明确main/background child和finish mode；owner/world/asset revoke复用同一cancel barrier并报告timeout/leak。

### P1-10：MoveTo通过动态属性和历史event猜测请求，清理实现错误且无request生命周期

Navigation availability由event storage是否注册判断；host构造时扫描全部report并按entity只保留最后outcome。目标只有静态Vec3参数，不能读Blackboard entity/location、acceptance radius或filter。首次tick写destination，后续按destination浮点相等匹配，没有request/generation；clear(None)改写当前坐标而不是None。目标使用中立 `NavigationTaskBroker`：Start返回generational request ID，携agent/navmesh/filter/target source/acceptance/repath policy；Poll/Completion精确关联request；Cancel清真实optional target并等待ack。AI不扫描event storage、不直接写dynamic JSON，并与Navigation08D的movement authority/crowd/off-mesh合同对齐。

### P1-11：PlayAnimation和ScriptTask不是latent task，完成、取消、超时、reload和执行预算均不成立

PlayAnimation只写parameter/trigger并立即Succeeded，不区分state进入、clip完成、notify、blend interruption和失败generation；abort对animation/script不做任何事。ScriptTask每次Running都重新同步invoke provider，没有instance handle、continuation、cancel callback、deadline、instruction/time budget或late result fencing。目标通过AnimationTaskBroker和ScriptTaskBroker返回task handle；start/tick/event/finish/cancel分离，owner/world/asset generation随handle，provider reload先停admission再迁移/取消。脚本执行受instruction/time/memory和callback queue预算，不能在AI主tick同步运行任意用户代码。

### P1-12：Blackboard公共contract仍由字符串type/key和完整Vec snapshot驱动，prepared dense优势没有贯穿边界

schema descriptor的 `value_type` 是可接受别名的String；key没有stable GUID/field ID/default/description/category/instance sync/save/replication/lifetime。manager只有整Vec set/get，每次校验、HashSet和线性schema搜索；无schema agent则退回动态Vec，observer不能工作。目标schema使用typed enum和stable key ID，cook生成slot table/compat hash；runtime提供typed slot accessor与batched mutation transaction，dynamic/untyped模式只允许tooling或明确legacy profile。Agent必须显式绑定schema；debug/serialization按需投影DTO，不让hot path回到String/Vec查找。

### P1-13：Blackboard缺少工程级值域、默认/继承、引用安全、迁移和跨系统写入策略

现有六类值无法表达Name/enum/tag/object/class/asset/nav location/rotator等常用AI数据；Entity只是裸u64，无world/generation/weak invalidation。required key没有default initialization，schema不能继承，generation u32可wrap，asset reload不能迁移active store。目标定义P1最小类型集和nullable/weak ownership，schema defaults/inheritance/override与stable migration；entity/resource写入校验world和generation，despawn自动invalid/notify。每个key可声明write authority、save/replicate/debug policy，跨线程写通过world command批次在确定阶段提交。

### P1-14：Perception每帧全World重建receiver/source并扫描R×S，固定pair数不能保证时间或感知延迟

system两次调用 `node_records()`，逐entity尝试dynamic JSON/typed component、world transform，重建并排序Vec。pair cursor面对稀疏大world仍遍历所有slot；256只限制成功consume的pair，不限制world扫描、transform查询、event map、physics call时间或snapshot clone。若R×S/256超过forget window，持续可见source也会因轮不到刷新而闪烁遗忘。目标由component lifecycle增量维护listener/source registry和spatial index，dirty transform批量更新；candidate generation按cell/range/channel，预算同时限制time/pairs/physics queries/bytes并记录oldest latency。质量层必须根据agent重要性和最大感知延迟调度，而非让forget碰运气。

### P1-15：Sight在Physics缺失或query失败时默认“可见”，`Perception Complete`是错误能力声明

`is_occluded`把bridge unavailable/revoked/error压成None，scan将None作为visible并增加fallback计数。shipping场景会穿墙发现目标，而且能力manifest仍标Complete。目标将provider状态和query outcome区分为Visible/Occluded/Unknown/Unavailable/BudgetDeferred；shipping默认fail-closed或保留上次已知状态并快速过期，项目可显式选择debug cone-only profile。capability在M5验收前改Partial，诊断聚合unknown/deferred/provider generation，不能用静默降级维持绿色。

### P1-16：Perception刺激模型缺少affiliation、sense配置、lost transition和可扩展provider生命周期

Source只有两bit channels和strength，Receiver只有FOV/range/radius/forget；contract虽然枚举Damage/Touch/Custom却无provider。没有team/friendly-neutral-hostile filter、tag、socket/offset、per-sense max age、peripheral vision高度、auto success range、dominant sense、stimulus success flag、lost reason或listener/source generation。Static HEARING source和event hearing又会并行产生不同生命周期。目标以SenseRegistry和prepared SenseConfig注册provider，Stimulus包含sense/source generation、timestamp、success/lost reason、confidence/tag；listener按affiliation/channel筛选。Sight/Hearing先完整闭环，其他sense未实现就保持Partial而不是只留enum。

### P1-17：Behavior LOD由active camera驱动业务决策，server、replay和多人观察语义错误

Full/Half/Quarter阈值硬编码20/60，生产system读取 `world.active_camera()`；无camera即Full。camera切换会改变Wait/Service/Script/MoveTo评估时刻，多玩家时只认一个camera，server AI成本和客户端画面绑定。skip期间pending delta无上限，恢复tick一次吞掉全部时间。目标将simulation significance与render camera解耦：server/world scheduler根据gameplay relevance、distance set、combat/visibility/task deadline和quality profile决定频率；critical/latent deadline可唤醒。delta按fixed-step/substep/cap处理，决策顺序和seed可记录重放，client camera只能影响debug/visualization采样。

### P1-18：每帧无条件构造全量runtime/debug snapshot，debug关闭仍深clone生产状态

Behavior system先 `runtime_snapshot()` clone全部registered descriptor和全agent Blackboard/Perception，再按world/active entity过滤，再次clone进debug frame，并无条件发送complete snapshot。没有reader count、capture mode、changed slot/node、entry/bytes/age budget或consumer lag策略；HashSet使agent顺序不稳定。目标建立reader-scoped `AiDebugCaptureLease`，无reader时hot path成本接近零；producer只发generation-bound node transition、changed slot、stimulus delta和aggregate counter，按agent/filter采样。显式full capture异步分片且有entry/bytes/time/age上限，slow Editor不反压runtime。

### P1-19：缺少可取消的环境查询/目标选择服务，复杂AI只能把静态常量塞进节点参数

当前没有EQS/query ID、generator/context/test/score、cache、time slicing或querier cleanup；MoveTo target只能是descriptor中的静态Vec3。Perception snapshot也没有空间查询接口或共享world knowledge。目标先交付P1基础 `AiSpatialQueryService`：typed request、querier/world generation、candidate provider、filter/score chain、time/entry/query budget、cancel、cache key、deterministic tie-break和result handle；Behavior node通过latent broker读取Blackboard/context并提交query。高级可视化graph、多context和大规模batch留到P2，但静态target不能作为最终产品。

### P1-20：manifest/native dist/历史验收与真实能力不一致，测试缺少产品、故障和规模资格

manifest把Perception标Complete，却只有Sight/Hearing且provider failure穿墙；native projection在Rust declaration中列 `systems/events/extensions: []`，dist只证明descriptor ABI；历史M3记录称MoveTo reset target，current source实际不能写None。98项runtime test中多项用`result`旁路业务。目标建立capability truth table和generated registration parity，dist加载后必须注册与静态插件等价的manager/system/component/event/interface并支持unload。required lanes覆盖asset/scene启动、world replace/despawn、plugin reload/cancel、dedicated server、large world、consumer stall和性能曲线；output record绑定current source/Build Set，旧绿色不能自动继承。

## 5. P2 能力差距

### P2-1：缺少StateTree/utility/HTN等适合层级状态与长流程的决策runtime

Behavior Tree不应承载所有gameplay状态。建立独立prepared StateTree或等价state-selection runtime，包含state hierarchy、enter/exit、transition priority、condition/task binding、event transition、linked state、instance data和debug trace；可复用Blackboard/task broker，但不能伪装成更多BT字符串semantics。

### P2-2：缺少完整EQS级多context generator/test/score、异步trace和结果调试

在P1基础query service上增加grid/nav/actor/cover generator、多context、filter/score normalization、named parameter/data provider、async physics/nav batch、query template cache、breadth/depth调度、history和可视化结果。规模验收覆盖大量并发query、公平性、取消、cache invalidation和deterministic ranking。

### P2-3：缺少Mass AI、Smart Object、Zone/traffic、群体LOD与大世界分片

高密度crowd不能为每个entity运行相同递归tree和完整Blackboard。需要fragment/chunk批处理、shared behavior program、significance/LOD、Smart Object claim lease、zone/traffic lane、world partition activation和跨cell state迁移，并与Navigation crowd和streaming generation对齐。

### P2-4：缺少team/squad/cover/reservation/world knowledge与战术协调层

企业级AI需要可过期事实、共享/私有knowledge、team attitude、threat、cover/reservation、formation、communication和authority conflict resolution。它应建立在typed perception/query/Blackboard owner上，拥有lease、priority、timeout和debug provenance，不能由多个agent直接写全局String map。

### P2-5：缺少network authority、save/replay、deterministic simulation和离线AI质量工具

AI state尚无server authority、replication/save schema、decision trace、random seed、replay seek或hot-join恢复。目标定义可序列化最小instance state与兼容generation，网络只复制必要intent/result，server重放能解释每次transition。离线工具运行scenario corpus、coverage、stuck/oscillation、query quality和CPU/allocation regression，并与Editor debugger共用trace schema。

## 6. 目标架构

```text
BehaviorTree / Blackboard / Perception / Query source assets
        |
        v
AI Asset Import + Cook Compiler
  - stable asset/key/node identity
  - schema + dependency + capability validation
  - bounded iterative compile
  - prepared BehaviorProgram / BlackboardLayout / SenseConfig
        |
        v
AiRuntimeService
  +-- PreparedProgramCatalog { generation, dependency closure, owner leases }
  +-- AiWorldRuntime { world, replacement_epoch }
       +-- AgentRegistry { entity generation, brain config, lifecycle }
       +-- BehaviorScheduler { active stacks, node/time budget, wakeups }
       +-- BlackboardArena { typed slots, mutation batches, observers }
       +-- LatentTaskBroker { Navigation / Animation / Script / Gameplay }
       +-- PerceptionService { spatial index, senses, stimuli, budgets }
       +-- SpatialQueryService { tickets, cache, cancel, score pipeline }
       +-- DebugTap { reader leases, deltas, bounded full capture }
        |
        v
World command/event boundary + telemetry + Editor read-only mirror
```

关键所有权规则：

1. `zircon_runtime::core::framework::ai`只拥有稳定中立合同，不拥有插件executor或完整runtime snapshot热路径。
2. 每个World replacement generation只有一个 `AiWorldRuntime`，agent lifecycle来自scene component增量，不来自外部手工manager side effect。
3. source asset不可直接执行；cook/prepare生成immutable program，publish原子切generation，旧instance通过migration/restart policy处理。
4. scheduler是agent execution唯一写者；外部系统只提交typed command/ticket，late completion必须校验world/agent/task/program generation。
5. node不能直接任意写World。Blackboard mutation、event、navigation、animation、script和query走capability-scoped broker并具备cancel/timeout/backpressure。
6. Perception provider注册sense，World owner维护listener/source/spatial index；physics unavailable是typed quality state，不是默认可见。
7. debug只有reader lease时采集，delta和full capture分开；Editor消费不能改变simulation频率、锁域或结果。

## 7. 必须硬切的旧实现

- 删除shipping Behavior Tree descriptor中的通用 `result`/`service_result`旁路；test需要固定返回时注册test-only node。
- 未实现真实副作用前，从标准目录移除 `set_blackboard`、`emit_event`、`update_blackboard_distance`能力声明；实现后按typed schema重新加入。
- 删除MoveTo对NavAgentTickReport event storage全量扫描和dynamic destination写入，硬切到request/generation/cancel broker；不得保留current-position冒充None的clear。
- 删除active-camera驱动的业务AI LOD；render camera只能影响debug sampling。
- 删除每帧无reader的 `runtime_snapshot()` 和完整 `AiBehaviorDebugSnapshot`；full DTO仅保留显式capture/serialization。
- 删除无schema的production动态Blackboard Vec fallback和String value_type主合同；legacy import必须一次迁移成typed prepared schema。
- 删除register-only Vec catalog和裸u64 handle作为hot-reload身份；所有program/schema/task/query handle带generation。
- 删除“Physics不可用即Sight可见”的shipping silent fallback；用typed Unknown/Unavailable策略。
- 删除owner revoke直接drop instance而不abort的路径；所有revoke/world unload/asset replace走统一cancel barrier。
- `Perception Complete`在P1验收前改为Partial；native dist未达到静态注册等价前不得发布同名runtime capability。

## 8. 分阶段重构计划

### M0：能力真实性与生命周期止血

- 下调Perception capability，移除/隔离`result`旁路和三个假标准节点。
- 修复MoveTo真正clear、TimeLimit/Parallel/owner revoke abort语义。
- 建立world/entity retire API和same-agent execution token；先阻止跨world残留与并发覆盖。
- 加入debug reader gate，关闭无consumer的full snapshot production成本。

### M1：资产、prepared program与节点schema

- 建立BehaviorTree/Blackboard/Perception资产import/cook/load和stable generation。
- 节点目录增加typed parameter/port/lifecycle/capability/debug schema，compiler生成预解析param block、parent/subtree/dependency表。
- 加入node/depth/parameter/string上限、iterative compile和atomic publish/LKG/hot reload。

### M2：迭代式有预算executor与latent lifecycle

- 用active stack/continuation替换无界递归和重复probe；建立node/time budget与yield。
- 统一Start/Running/Finish/Cancel/Abort/Timeout状态机，修复Parallel、Service、Decorator和Subtree完整语义。
- owner/world/asset revoke等待latent cancel terminal，超时产生leak report并隔离late result。

### M3：Agent/Blackboard ECS产品闭环

- 新增AiBrain component和scene projection，支持add/change/remove/despawn/world replace。
- Blackboard typed key/default/inheritance/mutation batch/entity generation与migration落地。
- 实现真实Set/Clear/Copy Blackboard、service scheduler与typed EmitEvent。

### M4：Navigation、Animation、Script和Gameplay task broker

- 与Navigation08D收敛MoveTo request/result/cancel/repath/off-mesh/movement authority。
- Animation task等待明确terminal/notify并处理blend interrupt；Script task支持durable handle、预算、cancel和reload。
- 增加跨插件owner revoke、provider unavailable、late completion和world shutdown tests。

### M5：Perception world service

- 用component lifecycle registry和spatial index替换全World扫描与R×S候选。
- Sight/Hearing具备time/query/bytes/latency budget、affiliation、lost outcome、provider generation和fail policy。
- stimulus delta直接更新agent perception/Blackboard service，避免每帧完整snapshot clone。

### M6：基础空间查询与world knowledge

- 实现typed query ticket、candidate/filter/score、cache、cancel、querier cleanup和deterministic tie-break。
- 提供BT query/task节点与Blackboard result binding；建立最小team/threat/reservation合同。
- 高级EQS、StateTree、Mass和Smart Object按P2独立计划，不塞进MVP executor。

### M7：按需调试、native dist与Editor contract

- 建立node transition、slot delta、stimulus/query/task trace和reader-scoped full capture。
- native dist与静态plugin registration parity，支持系统/component/event/interface和unload。
- 向Editor20提供asset transaction/compiler diagnostics/runtime mirror/simulation接口，不让Editor读取manager内部HashMap。

### M8：产品、故障与性能资格

- 普通project scene从资产启动AI，覆盖client/server/editor host、PIE多world和hot reload。
- 建立10k/100k agent/source/query规模曲线、fixed workload与Unreal/Fyrox对照方法，记录CPU、allocation、latency和质量。
- 完成soak、fuzz、consumer stall、provider failure、plugin reload、world churn、deterministic replay和跨平台required lanes。

## 9. 验收门

### 9.1 正确性与生命周期

1. 普通scene只靠AiBrain/asset即可启动、切换、停用AI；测试不手工调用register/tick作为产品入口。
2. World unload/replacement和entity despawn后，agent、Blackboard、Perception、task/query/debug状态全部归零，late result被generation拒绝。
3. 同一agent并发tick不会分叉或lost update；不同agent可按scheduler策略并行且结果确定。
4. TimeLimit、Parallel Any、selector abort、tree replace、plugin revoke和world shutdown均取消所有active latent leaf并获得exactly-one terminal。
5. MoveTo success/failure/abort真正清除optional target，旧report、相同target新request和浮点扰动不能误完成。
6. PlayAnimation/ScriptTask只有真实terminal才成功；cancel/reload/timeout不会留下animation parameter、script continuation或World side effect。
7. SetBlackboard、EmitEvent和Service具有名称对应的可观察副作用；不存在shipping `result`参数旁路。
8. cross-tree cycle、deep/wide/oversized asset、unknown node generation和schema migration均typed reject或LKG，不panic/stack overflow。

### 9.2 Perception与查询

9. Physics unavailable/error/budget deferred分别可观察，shipping策略不会默认穿墙。
10. 持续可见/可听目标在最大感知延迟内刷新，不因pair backlog超过forget window而闪烁。
11. listener/source add/remove/transform/team/config变化增量生效，source/entity generation复用不继承旧stimulus。
12. lost stimulus含sense、source generation、reason和timestamp；Damage/Touch/Custom未实现时不声明Complete。
13. query有ticket、cancel、deadline、budget、deterministic ordering与querier cleanup；world replace后旧result不可写Blackboard。
14. Navigation/Physics/Script provider reload期间admission、in-flight和new generation边界可证明。

### 9.3 性能与确定性

15. 无debug reader时不构造全量runtime/debug snapshot，agent tick不clone全catalog。
16. executor每frame node/time budget可配置并产生yield telemetry；任何单agent/第三方node不能无限占用world update。
17. Perception steady state不扫描全部World node，不构造R×S矩阵；候选、physics query和oldest latency有曲线。
18. 10k active/100k LOD agent测试记录p50/p95/p99 frame time、node ops、alloc bytes、Blackboard writes和task latency。
19. 10k listener/source与事件storm测试同时验证time、pairs、physics queries、queue bytes/age/drop和RSS上限。
20. dedicated server、client和replay使用相同seed/fixed step/ordered inputs得到相同decision trace；camera移动不改变server AI。
21. asset/program/Blackboard hot reload不造成全agent同帧峰值，迁移/重启有分批预算和oldest age。
22. 性能比较固定场景、Build Set、硬件、quality与统计方法；没有可复现证据前不宣称优于Unreal。

### 9.4 插件、产品与观察性

23. Node/Sense/Task provider的register/revoke/reload均有owner generation、cancel barrier和leak report。
24. native dist加载后提供与静态AI插件一致的manager/system/component/event/interface，卸载后无悬空callback。
25. capability表逐项绑定product test；Partial/Complete由gate生成，不能手写高于实装状态。
26. Debug delta按world/agent/program generation排序，slow/absent Editor不反压runtime；full capture有entry/bytes/time/age上限。
27. telemetry至少包含agents by LOD/status、node ops/yield、task start/cancel/timeout、Blackboard changes、Perception candidate/query/deferred/fallback/lost、queue watermark/drop和debug bytes。
28. Editor20只消费公开asset/compiler/operation/debug contract，不能直接锁DefaultAiManager或复制内部state。
29. client、dedicated server、Editor Host/PIE多world、headless test和native dynamic至少各有一条真实scene产品lane。
30. world churn、entity ID复用、asset reload、plugin reload、provider failure和consumer stall组成长时间soak，无state/queue/RSS持续增长。
31. compiler/descriptor/event/script input有fuzz和hard size/depth/count budget；恶意插件node不能越过capability context。
32. output record绑定current source fingerprint、Build Set、scenario、配置和原始metrics；零测试、静态registration或历史绿色不得满足release gate。

## 10. 与既有计划的关系

`docs/plans/zircon_plugins/06-ai.md`仍是历史实现owner和局部产出索引；本篇是以2026-08-16 current source重新审查后的重开清单。M1的compiler/catalog基础、M2的dense Blackboard/observer、M4的bounded hearing ingress可迁移保留。以下完成声明必须重开：18个标准节点完成、MoveTo reset target、Perception Complete、camera-distance LOD可作为最终simulation策略、完整debug snapshot可逐帧发送。

Runtime08D Navigation拥有navmesh/query/crowd/movement authority；AI只拥有intent/task lifecycle，不再直接写NavMeshAgent dynamic property。Runtime08C Animation拥有graph/state/clip terminal；AI只持有task ticket。Runtime07 Script/Plugin拥有VM预算和provider reload；AI只消费generation-bound broker。Runtime05 Scene/ECS拥有World/entity replacement与component lifecycle；AI必须接其generation，不另造裸u64世界。Editor20后续拥有Behavior Tree/Blackboard/Perception/StateTree/EQS authoring和debug UI，并以本篇M0-M7为前置。

## 11. 完成定义

本计划只有在20项P1全部关闭、对应代码和产品lane通过第9节32项验收门、capability/native dist与真实registration一致、current source重新取证并更新output record后才能标记implemented。P2可以分期，但必须保持明确Partial和不可误触发的产品入口。仅增加节点数量、让manager unit tests绿色、注册Editor view、发送更多full snapshot或保留silent fallback，不构成工程级AI完成。
