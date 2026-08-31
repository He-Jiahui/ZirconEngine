---
title: Runtime AI、Behavior Tree、Blackboard、Perception、EQS、StateTree、Smart Object、Task、Navigation、Network、Save、Scalability、Editor 与 Product Integration 当前源码工程化差距
report_id: Runtime152
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
runtime_child_of: Runtime08F
related_code:
  - zircon_runtime/src/core/framework/ai
  - zircon_plugins/ai/runtime
  - zircon_plugins/ai/editor
  - zircon_plugins/ai/dist
  - zircon_plugins/ai/plugin.toml
  - zircon_plugins/first_party_runtime_catalog
  - zircon_plugins/first_party_editor_catalog
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog
  - examples/vampire
plan_sources:
  - docs/plans/mvp/index.md
  - docs/plans/optimize/zircon_runtime/08f-ai-behavior-tree-blackboard-perception-runtime-review.md
  - docs/plans/optimize/zircon_editor/20-ai-behavior-tree-blackboard-perception-eqs-debug-authoring-review.md
  - docs/plans/zircon_plugins/06-ai.md
  - docs/plans/zircon_plugins/06
  - docs/plans/optimize/zircon_runtime/99zp-runtime-navigation-navmesh-recast-detour-tilecache-crowd-query-pathfinding-obstacle-off-mesh-link-bake-streaming-world-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zl-runtime-animation-skeleton-clip-pose-graph-state-machine-layer-mask-blend-ik-root-motion-event-extract-product-integration-current-source-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/AIModule
  - dev/UnrealEngine/Engine/Plugins/Runtime/StateTree
  - dev/UnrealEngine/Engine/Plugins/Runtime/SmartObjects
  - dev/bevy/crates/bevy_ecs/src/schedule
  - dev/bevy/crates/bevy_tasks/src
  - dev/Fyrox/fyrox-impl/src/utils/behavior
  - dev/godot/scene/3d/navigation
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging
---

# Runtime AI、Behavior Tree、Blackboard、Perception 与 EQS 当前源码工程化差距

## 1. 结论

当前 Zircon 已经拥有一套**可直接单元测试的 AI 子系统原型**，但还没有可作为工程级游戏引擎产品交付的 AI Runtime。当前进展不是空壳：Behavior Tree 已编译为带 parent、subtree range、implementation slot 的密集不可变代；executor 保存 per-agent node state、部分 abort 状态与复用 scratch；Blackboard 已有按类型分区的 dense store、generation 与 changed-slot observer；Perception 已有稳定遍历、pair cursor、全局配额、有界事件 backlog；插件也进入 first-party runtime catalog，并提供 owner revoke gate、Navigation/Animation/Script adapter、runtime debug mirror。上述能力应保留，不应回退到早期 Vec 全克隆方案。

但是这套实现尚未形成 `source asset -> import/cook -> immutable program artifact -> Scene/ECS agent binding -> per-World runtime -> cancellable task/query -> network/save/replay -> runtime-backed Editor` 产品链。生产源码中没有调用者注册 Behavior Tree/Blackboard 或驱动 `tick_agent`；Vampire 样例的 `enemy_behavior_tree.toml` 使用另一套字段并被标记为普通 Data，Scene 只保存字符串，实际敌人决策由 `main.zr` 分支和动态字段执行。README 所称 Behavior Tree 因而不是当前 AI plugin 的产品证据。

executor 仍把 `SetBlackboard`、`EmitEvent` 与 service 效果简化为静态 `result`/`service_result` 参数；`PlayAnimation` 立即成功；`ScriptTask` 没有 durable handle；`MoveTo` 通过动态属性写 destination，再从历史导航事件重建状态，没有 request generation、取消确认或 moving-target/repath 合同。`TimeLimit` 超时不 abort child，`Parallel` 终止不取消仍运行 sibling。递归求值没有 node/time/depth/reentrancy budget，同一 agent 的并发 `tick_agent` 又会 remove-execute-reinsert，允许双执行和 last-writer-wins。

Perception 每帧仍扫描所有 dynamic world node，重建 receiver/source，再做笛卡尔积；256 只是全局 pair count，不保证个体最大延迟。Sight backend 缺失或报错时返回可见，simulation correctness 取决于 provider 可用性。Behavior LOD 又由 active camera 距离选择 Full/Half/Quarter，使 dedicated server、多个观察者、回放与客户端相机产生不同 gameplay truth。EQS、StateTree、Smart Object、team/squad/world knowledge、network/save/replay均没有 runtime owner。

本轮不重复创建 Editor20 的 5 项 P0 owner；current-source 复核仍为 **5 Open / 0 Partial / 0 Closed**。Runtime08F 的 20 项历史 P1 重判为 **10 Open / 10 Partial / 0 Closed**，局部性能和结构进展没有关闭产品合同。Runtime152 新账本登记 72 项 P1：**57 Open / 15 Partial / 0 Closed**；16 项 P2 全部 Open。40 项资格门为 **32 Fail / 8 Partial / 0 Pass**。任何“性能优于 Unreal”的结论都必须等同源规则、正确性、规模和平台矩阵通过后再由原始数据证明。

## 2. 审查边界、currentness 与证据强度

### 2.1 Currentness

- 审查基线：`main@1b2684b40ae3eba7abfcdfae3fe7e341b4906ec8`。
- 冻结时间：`2026-08-25T16:26:44.4219992+08:00`。
- 读取时共享工作树有 3,504 个 tracked changes、2,260 个 untracked paths；AI runtime/editor 正含大量在途重构和新增 allocation test。本文读取 physical working bytes，不归因、不覆盖、不回退其他 Session 修改。
- 当前 MVP 仍处于 MVP-00。本轮属于允许的 C3 read-only audit，只写 review/plan/index，不实现高级 AI 功能。
- 按用户要求，本轮不轮询协调器，也不因协调器状态暂停；文档不引用未取得的 lease 或 epoch。
- 本报告是 current-source refresh：Runtime08F 继续拥有历史 Runtime 账本，Editor20 拥有 authoring/product surface，Plugins06 拥有 package 实施记录；Runtime152 负责当前 Runtime 差距、依赖顺序和验收门。

### 2.2 冻结范围

统计口径：repository-relative path 转 `/` 并小写排序；逐文件取当前 bytes SHA-256；聚合输入为 `path|file_sha256` 以 LF 连接且末尾无 LF。tests 按 Rust `#[test]`、C++ automation macro 和 `TEST_CASE` 声明计数；`#[ignore]` 单列。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 工作树 fingerprint 与证据 |
|---|---:|---|
| Zircon core AI contracts | **9 / 1,040 / 956 / 31,239 / 0 / 0** | descriptor、ID、manager、tick、snapshot、blackboard、perception；`5d9772897fc38a3ff2ff55b8e66d45c74b31a49e091f0d90333ee7eb92ce707d` |
| AI runtime plugin | **84 / 21,634 / 19,893 / 777,751 / 217 / 42** | compiler、catalog、executor、node integration、blackboard store、manager、perception、plugin 与 tests；`f0d9e3f36761756e21742c6885c48f422812aca02e991e74b69de0a46936eb71` |
| AI editor plugin | **12 / 2,396 / 2,223 / 83,869 / 18 / 4** | registration、mirror、overlay、controller、ZUI 与 tests；`2fe0566021b50d87516517475c5def9b31dcf5b381f78e71a449d493c49ce81d` |
| AI manifest 与 distribution | **3 / 237 / 205 / 8,582 / 2 / 0** | capability、event/interface、runtime/editor artifact 与 dist entry；`0c21e0fec77bed20ffdcd2dd60e2ac74512fff17c1ae8cffe3c62118371d4ab4` |
| Selected catalog 与 product path | **15 / 3,843 / 3,287 / 133,287 / 6 / 0** | runtime/editor catalog、builtin classification、overlay contract、Vampire project/Scene/asset/script；`9d9a794e29bac65896d8eca629129dac0ee6e18b9c1f4940b96d2dba8f904b63` |
| Unreal AI、StateTree 与 SmartObjects | **16 / 26,956 / 22,819 / 1,091,635 / 0 / 0** | BT component、Blackboard、Perception/Sight、EQS manager、StateTree execution context、SmartObject subsystem；`ba8801549b3c1e20c6316c33387835e9eebc0703f41e0b969ce004853cdc4d48` |
| Bevy、Fyrox、Godot 与 Unity Graphics | **6 / 6,266 / 5,439 / 233,805 / 46 / 0** | schedule/task pool、轻量 behavior、NavigationAgent3D、DebugManager；`198f5e9f6a3a2ac35e4023d4e774090adeef3c546e188610ba693ba70b8ce193` |

Zircon 选择集合计 123 文件、29,150 行、26,564 非空行、1,034,728 bytes、243 项 test declaration、46 项 ignored。参考选择集合计 22 文件、33,222 行、28,258 非空行、1,325,440 bytes、46 项 test declaration。Unreal 的 AIModule、StateTree、SmartObjects 完整树还分别包含约 391、414、122 个文件；本表 fingerprint 只冻结本轮逐项对照的核心合同，不把目录总量伪装为逐文件内容统计。

### 2.3 纵向扫描链

本轮按 capability claim -> public descriptor/ID -> source/compiler -> catalog/factory -> per-agent executor -> abort/task adapter -> manager/owner/unload -> Blackboard -> Perception/EQS -> schedule/LOD/debug -> Scene/product example -> Editor mirror/overlay -> package/dist -> focused tests 的顺序读取。参考侧以 Unreal 为完整生命周期主合同；Godot 用于导航代理状态机；Bevy用于 schedule/task ownership；Fyrox用于轻量 Behavior 下界；Unity Graphics 只验证 debug data 注册/注销和 reader ownership。Unity Graphics 中的 Blackboard/StateTree 命中属于 ShaderGraph/VFX 属性 UI，不是 AI runtime，不能据此降低基线。

## 3. 当前可保留的真实基础

1. Behavior compiler 已生成 preorder dense nodes、child indices、parent indices、subtree end、compiled subtree target、unique implementation slot 和 immutable `Arc<[CompiledBehaviorTree]>` generation，方向正确。
2. extension catalog 有 typed owner、factory、frozen table、dense slot 和 revoke gate，可扩展为 engine node ABI；当前缺的是完整 descriptor 与统一执行模型，不是重写 registry。
3. per-agent executor 已有 node state、observer binding cache、scratch buffer、tree stack和部分 root/preemption abort，可作为未来 iterative VM 的迁移起点。
4. Blackboard layout/store 具备类型分区、dense slot、generation、entry-position index、sync scratch/epoch 和 changed-slot observer，是比公开 Vec DTO 更成熟的内部基础。
5. Perception 已把重复 node scan 收为单次 records pass，建立 stable world order、pair cursor、256 pair budget、1024 event backlog、aging 与确定性 stimulus ordering，应继续扩展为 World service。
6. plugin runtime 通过 first-party catalog 可按 project selection 注册，manifest 也声明 runtime/editor artifact、event consumer 与 interface；这证明 package plumbing 存在，但不证明产品 AI agent 已运行。
7. Navigation、Animation、ZR VM、Physics bridge 和 Editor event mirror 都可作为 typed adapter 的底座；必须由 task/query lifecycle 驱动，不能继续让节点猜测动态属性。
8. focused tests 与 allocation tests 数量可观，适合保留为回归资产；release-only ignored benchmark 必须进入受控资格 lane 才能成为性能证据。

## 4. 当前源码断路

### 4.1 Source、Artifact 与 Product Reachability

- `compile_behavior_tree_toml` 只解析 runtime DTO；没有 first-party asset importer/cook target、stable asset identity、source revision、dependency graph、migration、last-known-good 或 artifact receipt。
- manager 只有 append-style register/list；没有 update/unregister/reload、tree-schema dependency invalidation、active instance migration或旧 generation retirement。
- production path 没有 `register_behavior_tree`、`register_blackboard_schema`、`set_blackboard_entries`、AI `tick_agent` caller。测试直接构造 manager 不能替代 Scene/App/Server 产品接线。
- Vampire 的 TOML 使用 `id/version/root/action/result`，AI DTO需要 `format_version/display_name/root_node/implementation/parameters`；`.zmeta`又声明普通 Data。Scene 的字符串和脚本的 `vampire.behavior_node` 是第二套 authority。
- Runtime catalog 接入 AI，Editor catalog却只接 Navigation/Neural。AI editor crate仍引用仓内不存在的 `ViewportToolModeDescriptor/register_viewport_tool_mode`，因此默认未链接还掩盖 compile drift。

### 4.2 Behavior Program、Abort 与 Task

- node descriptor只有 kind、implementation、display、children、key/value parameters 和 abort policy；没有 typed parameter schema、default、pin/cardinality、resource dependency、side-effect、thread affinity、version或debug metadata。
- 18种 semantics 中，标准节点由 executor硬编码，extension factory只覆盖 External。`SetBlackboard`、`EmitEvent`和service没有真实作用，依赖静态 result 参数返回。
- `evaluate_node`递归下降，没有 node/time/depth/reentrancy budget；恶意或错误树可占满帧、栈或触发不可控重复求值。
- root变更和部分 selector lower-priority preemption会 abort，但 `TimeLimit`超时、`Parallel`终止和Loop新一轮都没有完整 child/sibling reset/abort合同。
- `BehaviorNodeTickContext`只有 params、Blackboard slice、Perception 和 delta；节点拿不到 qualified owner/world、command sink、task broker、cancel token、resource lease 或 current generation。
- `MoveTo`写 `NavMeshAgent.destination`，用当前位置伪装 clear target，精确比较 float array，再从历史 tick report推断状态。无request ID、cancel ack、filter、acceptance radius、partial path、timeout、moving target/repath。
- `PlayAnimation`设置参数/trigger后立即 Success；`ScriptTask`每个 Running tick同步重调 callback。二者均无完成通知、持久句柄、cancel、timeout、reload fence或side-effect compensation。

### 4.3 Owner、Identity、Concurrency 与 Reload

- 一个 `Arc<Mutex<AiRuntimeState>>` 同时持有所有 World；agent key只是 `(WorldHandle, EntityId)`，ID为裸u64，无entity generation、owner serial和World retirement fence。
- 没有 ECS `AiAgent/Brain/BehaviorTree/Blackboard` component 或 system自动激活/停用。active agent只是曾被手工 `tick_agent` 的map entry。
- `tick_agent`在锁内remove blackboard/instance、锁外执行、再reinsert；两个并发调用可同时从default state执行，同一 agent发生重复副作用和last-writer-wins。
- owner revoke会阻止新extension调用、等待in-flight并删除catalog/tree/agent/report，但直接drop active instance，没有先 abort navigation/animation/script task。
- register时构造所有已注册tree的implementation slot和owner lease，即使当前agent只运行一棵root；plugin/tree数量会进入普通tick成本。
- active agent HashMap遍历与跨agent commit没有稳定调度合同；panic、poison、callback重入、同帧register/revoke/tick顺序也未产品化。

### 4.4 Blackboard

- public contract仍以字符串schema/key/type和全量 `Vec<AiBlackboardEntry>` 交换；没有prepared key handle、typed reader/writer、writer provenance 或 version-qualified view。
- value只有 Bool/Integer/Scalar/String/Vec3/Entity，Entity又是裸u64。缺defaults、inheritance、enum/tag/object/resource/class、optional/weak reference和custom type provider。
- schema没有revision、compatible migration、redirect、rename/delete、live store rebuild或load/save migration。
- `set_entries`仍是full synchronization，遗漏key会被清空，边界产生changed Vec和clone；多个service/task/script writer没有transaction、priority、conflict和deterministic commit。
- executor中的 SetBlackboard/service没有调用 dense store writer；内部成熟存储与节点语义仍是两条断开的路径。

### 4.5 Perception、EQS 与 World Knowledge

- receiver/source每帧从所有World node和动态JSON-like component重建，随后做 receiver x source；没有spatial index、dirty registration、source/listener generation或分区迁移。
- 全局pair count budget不包含physics query、bytes、alloc或wall time，也不保证任一listener的最大更新延迟；高密度世界会长期饥饿。
- Sight bridge unavailable/error返回None，scan把None当visible，形成fail-open。动态组件解析失败又会静默丢receiver/source，故相同世界可因provider/数据错误改变simulation truth。
- 只有Sight/Hearing真正产生stimulus；Damage/Touch/Custom没有provider。缺per-sense config、team/affiliation/tag filter、dominant sense、success/lost/expired reason和listener callback。
- static hearing source持续刷新刺激，与一次性sound/animation event混在同一语义；没有声强、传播、遮挡、衰减或event identity。
- 没有EQS query source/compiler/manager、running query ID、abort/time-slice/cache、owner/world cleanup；也没有team/squad/cover/reservation/Smart Object/world knowledge服务。

### 4.6 Scheduling、Debug、Network、Save 与 Qualification

- AI Behavior tick按active camera距离在20/60米切换Full/Half/Quarter。相机、无相机、dedicated server和多viewer会得到不同AI更新语义；pending delta又没有上限和substep。
- targeted debug snapshot优于旧版全runtime snapshot，但每帧仍无条件clone active entity、Blackboard/Perception，发送report、node result和完整behavior snapshot；没有reader subscription、debug budget或backpressure。
- node result不是enter/exit/abort/transition因果trace，缺program revision、task/request ID、reason、duration、budget和first-divergence数据。
- 没有AI network authority/replication、save participant、replay journal、late join、rollback、hot-reload migration或deterministic digest。
- 217项runtime test和18项editor test以unit/direct manager为主；42项ignored多为release-only allocation evidence。缺same-agent concurrency、World unload/despawn、owner revoke side-effect abort、真实asset cook/product startup、dedicated server camera、network/save/replay和大规模soak。

## 5. 参考引擎可迁移合同

### 5.1 Unreal：AI Runtime 主合同

- `BehaviorTreeComponent`不是递归函数包装，而是有instance stack、execution request、pending execution、branch action、message observer、latent abort、safe/forced stop、task finish与cleanup的长期运行组件。Zircon应迁移其生命周期思想，并用prepared program和显式budget改善热路径。
- `BlackboardComponent`把key offset、value memory、key instance、initialize/uninitialize、per-key observer、pause/resume queued notification和递归安全observer removal组成一个owner。Zircon dense store方向正确，但必须补全schema、typed handle、transaction和world lifecycle。
- `AIPerceptionSystem/Component`承担listener/source register/unregister、end-play cleanup、sense registry、aging、delayed stimuli、team interface、dominant sense和success/lost/expired状态。Sight还使用importance/time-slice/pending query，而不是每帧全笛卡尔积。
- `EnvQueryManager`持有running query、query ID、abort、tick/time limit、cache、external query与owner/world cleanup。EQS不能被实现成一次同步函数或Behavior节点内临时扫描。
- `StateTreeExecutionContext`显式管理Start/Stop/Tick、event、transition request、scheduled tick、instance data和schema；StateTree应作为另一种compiled program/runtime，不是BehaviorTree enum再加几个节点。
- `SmartObjectSubsystem`有definition/slot、spatial partition、find/filter、claim priority/handle、occupy/free、invalidation和RW lock。销毁runtime instance前先abort所有使用者，因为abort流程仍需访问runtime data；这一点直接纠正Zircon owner revoke直接drop实例的问题。

### 5.2 Godot、Bevy、Fyrox 与 Unity Graphics：边界校准

- Godot `NavigationAgent3D`有target change/repath、path/target desired distance、next path position、finished/reachable、map change和avoidance safe velocity callback。MoveTo必须消费显式agent request/result/cancel，不应写destination再猜旧report。
- Bevy schedule/task pool提供system dependency、run condition、deferred apply、cleanup、ambiguity诊断和scoped task executor。它可承载AI evaluate/ordered commit，但不替代Behavior、Perception或EQS领域状态机。
- Fyrox generic Behavior支持typed mutable context、Visit/Clone和pool handle，证明小型行为树也应有typed context和可序列化状态；其轻量递归模型不满足Zircon目标规模，不能用来降低Unreal级生命周期要求。
- Unity Graphics `DebugManager`强调debug data显式register/unregister、panel/widget刷新和persistent UI。Graphics仓没有first-party AI runtime，ShaderGraph/VFX Blackboard不是AI参考；可迁移的只有reader-owned debug lifecycle。

## 6. 唯一 Owner 与硬边界

| 领域 | 唯一 owner | 禁止继续存在的旁路 |
|---|---|---|
| AI source/build set | versioned BT/BB/Perception/EQS sources + shared compiler | runtime-only TOML、Vampire自定义TOML、Editor独立parser |
| World runtime | `AiWorldRuntime` per qualified World generation | 全局manager混存所有World、App/script私有AI map |
| Agent identity/state | generational `AiAgentHandle` + ECS binding | 裸u64、只靠手工tick创建active agent |
| Behavior execution | immutable program + iterative budgeted VM | recursive enum switch、result参数伪task/service |
| Task side effect | `AiTaskBroker` + typed domain adapters | 动态属性写入、同步脚本重调、立即成功动画 |
| Blackboard | schema-qualified dense store + transaction | 字符串/全量Vec成为内部热路径、多个writer直接改值 |
| Perception | registered World service + sense providers | 每帧dynamic node全扫描、provider错误fail-open |
| EQS/world knowledge | cancellable query manager + typed result | 节点内同步扫描、Editor-only EQS文案 |
| Debug | bounded reader subscription + causal trace | 无reader逐帧全snapshot、Workbench固定数据 |
| Network/save/replay | AI journal/snapshot projection | 临时DTO、脚本字段或UI作为权威状态 |

硬切原则：新source/artifact、agent identity、task broker、perception registry和query manager必须与首方caller迁移同一里程碑落地；迁移后删除旧TOML schema、`result/service_result` shipping语义、dynamic MoveTo猜测、camera authority LOD和Vampire脚本AI旁路，不保留compat facade、fallback或双写。

## 7. Editor20 父 P0 当前状态

| 父项 | 状态 | current-source证据 |
|---|---|---|
| P0-1 AI Editor crate与当前Editor API静态不兼容 | **Open** | 仍引用仓内无定义的`ViewportToolModeDescriptor/register_viewport_tool_mode`；默认catalog未链接而掩盖drift |
| P0-2 first-party Editor catalog没有AI provider | **Open** | catalog只注册Navigation/Neural，AI project selection只有runtime provider |
| P0-3 Import/Open/Validate/Compile/Toggle无factory | **Open** | 仍是descriptor，没有`OperationCommandFactory`/handler |
| P0-4 BT/BB/Perception/Overlay无产品controller/provider | **Open** | ZUI仍为空Space/Table；overlay mode引用provider ID但provider registration为0 |
| P0-5 默认Workbench固定AI数据和成功反馈 | **Open** | 固定asset/node/agent/result仍不消费plugin runtime truth |

## 8. Runtime08F 历史 P1 重判

| 历史项 | 状态 | current-source重判 |
|---|---|---|
| 1 无asset-to-scene-to-runtime产品启动链 | **Open** | compiler只有直接函数；产品caller为0，Vampire schema不兼容 |
| 2 跨World singleton、无generation/cleanup | **Partial** | map加入World key和owner gate，但仍单mutex、裸ID、无World/despawn cleanup |
| 3 只有register，无update/unregister/reload/dependency | **Partial** | immutable compiled generation可保留；管理API仍只有register/list |
| 4 每tick clone全部tree/owner | **Partial** | tree已用Arc generation，仍为全部registered tree构造slots/leases |
| 5 并发remove/execute/reinsert丢状态 | **Open** | 同一结构仍存在且manager为Send+Sync |
| 6 递归执行无budget、重复遍历 | **Open** | parent/subtree cache改善查找，evaluate仍递归且无预算 |
| 7 catalog缺参数/端口/副作用/生命周期metadata | **Partial** | typed owner/factory/slot已建立，descriptor metadata仍不足 |
| 8 三个伪标准节点由result参数决定 | **Open** | SetBlackboard/EmitEvent/service仍依赖result/service_result |
| 9 TimeLimit/Parallel/owner revoke不abort | **Partial** | root/preemption和external abort有进展；三条终止路径仍不完整 |
| 10 MoveTo猜测动态属性/事件 | **Partial** | navigation adapter已接入，但request/cancel/repath/currentness仍缺失 |
| 11 PlayAnimation/ScriptTask不是latent task | **Open** | animation立即成功，script同步重调，无handle/cancel |
| 12 Blackboard字符串/全量Vec合同 | **Partial** | dense store已存在，public和executor边界仍是字符串/Vec |
| 13 Blackboard无defaults/inheritance/migration/write policy | **Open** | current schema仍未提供这些合同 |
| 14 Perception全World RxS | **Partial** | 增加single scan、cursor和pair budget，仍每帧重建并笛卡尔积 |
| 15 Sight fail-open | **Open** | bridge unavailable/error仍按visible处理 |
| 16 Perception无affiliation/config/lost/provider | **Open** | 仍只有Sight/Hearing的固定路径 |
| 17 camera LOD改变gameplay | **Open** | 20/60米camera distance LOD仍是权威tick策略 |
| 18 无条件full runtime/debug snapshot | **Partial** | 已按active entity targeted，仍每帧无reader发送完整agent snapshot |
| 19 无可取消EQS target service | **Open** | EQS runtime owner仍不存在 |
| 20 manifest/dist/history test不是产品/规模资格 | **Partial** | package/test覆盖显著增加，仍无真实产品、规模和shipping lane |

复算：Partial为2、3、4、7、9、10、12、14、18、20，共10项；其余10项Open，0项Closed。

## 9. Runtime152 P1 Runtime 专属重构清单

### 9.1 Source、Artifact、Agent 与 Product Truth

| ID | 状态 | 当前差距与目标 |
|---|---|---|
| AI-P1-001 | Open | 建立generation-qualified per-World `AiWorldRuntime`，禁止全局mutex成为多World owner |
| AI-P1-002 | Open | 建立`AiAgentHandle`/entity generation/owner serial，所有异步结果和snapshot过currentness fence |
| AI-P1-003 | Open | 定义versioned Behavior/Blackboard/Perception/EQS source、unknown-field和migration policy |
| AI-P1-004 | Partial | 保留TOML compile与dense topology，输出diagnostics、source revision、digest和immutable program artifact |
| AI-P1-005 | Open | 建立typed importer、cook target、dependency graph、LKG和shipping load路径 |
| AI-P1-006 | Open | tree、Blackboard、subtree、node implementation、sense/query依赖必须可解析、失效和原子重编译 |
| AI-P1-007 | Open | ECS Agent/Brain/BT/BB component驱动activate/disable/despawn/unload，不依赖手工`tick_agent` |
| AI-P1-008 | Partial | runtime catalog/package接线可保留；补Client/Server/Editor profile、required capability和fail-close truth |
| AI-P1-009 | Open | Vampire或新首方样例必须走同一asset/Scene/runtime链，并删除脚本平行AI authority |

### 9.2 Program、Catalog 与 Executor

| ID | 状态 | 当前差距与目标 |
|---|---|---|
| AI-P1-010 | Partial | 保留dense preorder/child index/subtree range，补stable node ID、debug span和artifact validation |
| AI-P1-011 | Partial | 保留parent/implementation/subtree target cache，补cycle/depth/size/budget和cross-asset identity |
| AI-P1-012 | Partial | 保留owner/factory/frozen slot/revoke gate，统一standard与extension node执行ABI |
| AI-P1-013 | Open | standard node也由versioned factory/semantics contract提供，删除executor中央硬编码的双模型 |
| AI-P1-014 | Open | node descriptor加入typed params/default/range、ports/cardinality、resource、side-effect、thread和debug metadata |
| AI-P1-015 | Open | recursive evaluate迁移为iterative stack/program counter，强制node/time/depth/reentrancy budget |
| AI-P1-016 | Partial | 保留per-agent node state/observer/scratch，补program generation migration和明确reset lifecycle |
| AI-P1-017 | Open | 同一agent只能有一个execution lease；重复、并发和reentrant tick返回结构化结果 |
| AI-P1-018 | Open | 多agent evaluate/ordered commit拥有稳定schedule、priority、starvation和deterministic digest |

### 9.3 Abort、Service 与 Latent Task

| ID | 状态 | 当前差距与目标 |
|---|---|---|
| AI-P1-019 | Partial | 保留root/preemption abort，统一enter/tick/request-abort/aborting/terminal/reset状态机 |
| AI-P1-020 | Open | TimeLimit超时必须取消child并等待/强制terminal，禁止遗留side effect |
| AI-P1-021 | Open | Parallel成功/失败时按policy abort running siblings，terminal与cache cleanup幂等 |
| AI-P1-022 | Open | Loop每轮显式reset/abort child，定义count、infinite、break和per-frame iteration budget |
| AI-P1-023 | Open | 建立generational `AiTaskHandle`、broker、await source、cancel、timeout、owner/world/plugin retire和receipt |
| AI-P1-024 | Open | MoveTo使用typed nav request ID/result/cancel ack、filter、acceptance、partial path、moving target和repath |
| AI-P1-025 | Open | PlayAnimation等待clip/montage/notify/interrupt terminal，并在abort/reload/despawn时清理 |
| AI-P1-026 | Open | ScriptTask持久化task/callback generation，限制预算，支持cancel/timeout/panic/reload late-result reject |
| AI-P1-027 | Open | Service/SetBlackboard/EmitEvent产生typed command，按phase原子commit，删除result参数伪实现 |

### 9.4 Owner、Reload、Concurrency 与 Failure

| ID | 状态 | 当前差距与目标 |
|---|---|---|
| AI-P1-028 | Partial | immutable Arc program generation可保留；补active generation pin、retire和bounded residency |
| AI-P1-029 | Open | register/update/unregister/reload经prepare/migrate/atomic swap/LKG执行，并返回receipt |
| AI-P1-030 | Partial | owner lease/revoke gate可保留；catalog/program/task/sense/query统一同一owner generation |
| AI-P1-031 | Open | revoke先quiesce并abort所有active node/task/adapter，待lease归零后再删runtime data |
| AI-P1-032 | Open | World close、Scene unload、entity despawn、agent disable、tree replacement都有一次性teardown |
| AI-P1-033 | Open | 禁止裸World/Entity/tree/schema/agent u64跨帧；handle wrap/reuse/cross-world必须fail-close |
| AI-P1-034 | Open | runtime state按World shard/partition持有，避免无关World互锁和统一大锁 |
| AI-P1-035 | Open | 明确panic/poison/OOM/callback error/reentrant mutation语义，不能丢instance或半提交Blackboard |
| AI-P1-036 | Open | plugin/script callback只在无owner锁的bounded lease中执行，late completion必须可拒绝 |

### 9.5 Blackboard Schema、Store 与 Transaction

| ID | 状态 | 当前差距与目标 |
|---|---|---|
| AI-P1-037 | Partial | 保留type partition/dense store/index/scratch，补qualified schema/program binding |
| AI-P1-038 | Partial | 保留generation和changed-slot observer，补wrap epoch、queued notification和safe unsubscribe |
| AI-P1-039 | Open | compiler生成prepared key handle/slot/type token，ordinary tick不解析字符串或全量Vec |
| AI-P1-040 | Open | schema支持default/inheritance/enum/tag/object/resource/class/optional/weak/custom provider |
| AI-P1-041 | Open | schema revision、redirect/rename/delete、compatible migration、LKG和active store rebuild闭环 |
| AI-P1-042 | Open | key声明read/write/authority/replication/save/debug policy，写入记录node/task/script provenance |
| AI-P1-043 | Open | 多writer通过transaction、phase、priority和conflict policy确定性提交，observer只见完整revision |
| AI-P1-044 | Open | 提供typed partial read/write/delta/snapshot；省略key不得隐式清空，边界不做稳态全clone |
| AI-P1-045 | Open | SetBlackboard/service/script/Editor全部消费同一store API和generation，不保留影子Vec authority |

### 9.6 Perception、EQS 与 World Knowledge

| ID | 状态 | 当前差距与目标 |
|---|---|---|
| AI-P1-046 | Partial | 保留single records pass、stable order和pair cursor，改为registered dirty source/listener owner |
| AI-P1-047 | Partial | 保留pair budget和bounded backlog，增加wall/query/byte/alloc budget、deadline与fairness SLA |
| AI-P1-048 | Open | 建立spatial acceleration、cell/region迁移、importance和incremental candidate query |
| AI-P1-049 | Open | sense provider unavailable/error/timeout按显式policy fail-close/degrade并产生diagnostic，不得默认可见 |
| AI-P1-050 | Open | per-sense config包含range/FOV/age/team/affiliation/tag/dominant sense与success/lost/expired reason |
| AI-P1-051 | Open | Sight/Hearing/Damage/Touch/Custom通过versioned provider registry和owner lifecycle扩展 |
| AI-P1-052 | Open | stimulus/source/listener均带generation和event identity，lost/stale/age transition幂等可订阅 |
| AI-P1-053 | Open | 持续acoustic emitter与一次性sound event分离，定义loudness/attenuation/occlusion/propagation |
| AI-P1-054 | Open | physics/visibility query使用async request/result/cancel、snapshot generation、budget和late-result fence |
| AI-P1-055 | Open | 建立EQS source/compiler/running query ID、time slice/cache/abort/owner cleanup和typed result |
| AI-P1-056 | Open | 建立team/squad/cover/reservation/Smart Object/world knowledge owner及与BT/EQS的只读查询边界 |

### 9.7 Scheduling、LOD、Debug 与 Performance

| ID | 状态 | 当前差距与目标 |
|---|---|---|
| AI-P1-057 | Open | gameplay tick rate由authority/importance/budget policy决定，不读取本地active camera |
| AI-P1-058 | Open | skipped delta有上限、fixed substep和deadline；LOD不能改变感知/任务terminal语义 |
| AI-P1-059 | Partial | targeted snapshot可保留，补reader subscription、field mask、rate/byte budget和backpressure |
| AI-P1-060 | Open | 无reader时不构造/clone debug payload；Editor disconnect/unload立即注销producer state |
| AI-P1-061 | Open | causal trace记录enter/exit/abort/transition/task/query、program revision、reason、duration和first divergence |
| AI-P1-062 | Open | 每agent只解析reachable program slots/owner leases，新增dense hot cache与bounded invalidation |
| AI-P1-063 | Open | node/task/query/perception/debug分别有CPU、alloc、bytes、count、latency和memory budget/receipt |
| AI-P1-064 | Open | 支持deterministic并行evaluate/ordered commit、World partition和1/100/1k/10k agent scale |

### 9.8 Domain Integration、Network、Save、Editor 与 Qualification

| ID | 状态 | 当前差距与目标 |
|---|---|---|
| AI-P1-065 | Open | Navigation adapter只消费qualified request/path/result/cancel，不读写动态属性或回放历史report猜状态 |
| AI-P1-066 | Open | Animation adapter消费typed play/notify/result/cancel并与graph/montage/root motion owner协调 |
| AI-P1-067 | Open | 定义server authority、AI command/input、relevant Blackboard/perception replication、late join和anti-cheat |
| AI-P1-068 | Open | Save/replay捕获agent/program/Blackboard/task/query/timer/RNG状态并支持revision migration/deterministic digest |
| AI-P1-069 | Open | runtime debug provider以bounded mirror服务Editor，overlay和node highlight不拥有simulation truth |
| AI-P1-070 | Open | Editor20通过shared document/compiler/operation/provider闭环创建、保存、编译、PIE和调试 |
| AI-P1-071 | Partial | 保留现有unit/allocation suite，补product、concurrency、teardown、fault、network/save和scale oracle |
| AI-P1-072 | Open | Client/Server/Editor/cook/package/PIE/soak/profile全部过门后，再与Unreal同源同规则比较领先性 |

P1复算：Partial为004、008、010、011、012、016、019、028、030、037、038、046、047、059、071，共15项；其余57项Open，0项Closed。

## 10. P2 领先性与高级产品

| ID | 状态 | 目标 |
|---|---|---|
| AI-P2-001 | Open | StateTree独立source/compiler/instance data/event/transition/scheduled tick与Behavior互操作 |
| AI-P2-002 | Open | Smart Object definition/slot/spatial query/claim/occupy/free/invalidation及跨World partition迁移 |
| AI-P2-003 | Open | EQS batch/vectorized generator/test/scorer、incremental cache和async provider融合 |
| AI-P2-004 | Open | Utility AI/HTN/planner作为immutable program与同一task/query broker的可插拔决策域 |
| AI-P2-005 | Open | Mass AI/crowd/traffic/Zone graph与individual agent层级切换和truth-preserving LOD |
| AI-P2-006 | Open | squad/formation/cover/reservation/world knowledge的分区复制与deadlock-free arbitration |
| AI-P2-007 | Open | SIMD/SoA batched condition、Blackboard query和perception scoring |
| AI-P2-008 | Open | deterministic parallel AI schedule、rollback simulation和跨平台first-divergence定位 |
| AI-P2-009 | Open | active program/schema/provider live migration、dual-run oracle和atomic cutover |
| AI-P2-010 | Open | large-world streaming、server handoff和agent/task/query/claim state迁移 |
| AI-P2-011 | Open | offline AI trace、time travel、counterfactual replay和causal graph压缩 |
| AI-P2-012 | Open | node/query bytecode fuzz、property/metamorphic/model checking与malformed asset隔离 |
| AI-P2-013 | Open | multi-user graph/Blackboard merge、stable IDs、冲突可视化和runtime-safe publish |
| AI-P2-014 | Open | learned policy/neural provider只能经bounded typed adapter进入，不绕过authority/task journal |
| AI-P2-015 | Open | workload-aware quality policy在保持simulation truth前提下降低debug、frequency或query精度 |
| AI-P2-016 | Open | 同correctness workload下持续证明相对Unreal的P50/P95/P99、RSS、alloc、吞吐和network bytes优势 |

## 11. 依赖顺序与实施里程碑

### M0 · Product Truth、Owner 与 RED Freeze

- AI Editor/provider缺失时App/Workbench显示Unavailable，删除固定成功和Vampire伪BT声明。
- 固定source/artifact、World/Agent identity、command/receipt、schedule phase和hard-cut deletion matrix。
- 为same-agent并发、TimeLimit/Parallel abort、revoke side effect、Sight fail-open和camera LOD建立RED。

### M1 · Source、Compiler、Artifact 与 ECS Binding

- 实现BT/BB/Perception/EQS versioned source、import/cook/dependency/LKG和immutable build set。
- 实现ECS Agent/Brain binding、per-World owner、generation-qualified handle和product startup。

### M2 · Iterative Behavior VM 与 Task Broker

- 实现iterative budgeted executor、统一node ABI、完整abort/reset和deterministic schedule。
- 实现task broker及Navigation/Animation/Script/Event/Blackboard typed adapter，删除伪result路径。

### M3 · Blackboard Kernel

- 完成schema type/default/inheritance、prepared key、transaction、observer、migration和save/net policy。
- 所有node/script/editor路径硬切到同一dense store和revision。

### M4 · Perception World Service

- 完成listener/source registry、spatial acceleration、sense provider、affiliation/lost/aging和async query。
- 移除每帧dynamic World全扫描与fail-open sight，建立fairness和最大延迟门。

### M5 · EQS、World Knowledge 与 Smart Object 基础

- 实现cancellable EQS manager、cache/time slice/owner cleanup和BT/StateTree只读query接口。
- 建立team/squad/cover/reservation/Smart Object的最小工程合同，不在节点内临时维护全局状态。

### M6 · Network、Save、Replay、Reload 与 Debug

- 完成AI authority、replication、late join、save participant、replay/determinism和program migration。
- debug改为reader-gated bounded causal trace，Editor20消费同一runtime provider。

### M7 · Product、Scale、Fault 与 Unreal 对标

- 首方Client/Server/Editor/Vampire场景通过asset cook、Scene roundtrip、PIE、package和teardown。
- 完成1/100/1k/10k agent、query storm、fault/fuzz/soak/profile及同源Unreal比较；只有原始证据允许领先声明。

## 12. Runtime152 复验门（40项）

### Product、Source 与 Artifact

- [ ] AI-G01 `Fail`：AI provider缺失时Runtime/App/Editor统一Unavailable，无固定成功或脚本伪BT声明。
- [ ] AI-G02 `Fail`：首方source可create/import/save/reopen/cook/load，Vampire使用同一schema和asset kind。
- [ ] AI-G03 `Partial`：direct TOML compiler与dense program存在；revision/dependency/artifact/LKG链尚未通过。
- [ ] AI-G04 `Fail`：Scene/ECS Agent binding经spawn/disable/despawn/unload/duplicate/remap无损且一次性teardown。
- [ ] AI-G05 `Fail`：Client/Server/Editor profile按required capability装配同一build set并fail-close。

### Program、Abort 与 Task

- [ ] AI-G06 `Fail`：iterative executor在depth/node/time/reentrancy budget下返回deterministic terminal/continuation。
- [ ] AI-G07 `Fail`：Selector/Sequence/Parallel/Loop/TimeLimit/RunSubtree所有abort/reset矩阵无残留side effect。
- [ ] AI-G08 `Partial`：typed extension catalog和revoke gate存在；standard node、metadata和完整ABI未统一。
- [ ] AI-G09 `Fail`：同一agent并发/reentrant tick只能有一个lease，状态和command不丢失不重复。
- [ ] AI-G10 `Fail`：MoveTo/PlayAnimation/ScriptTask完成、取消、超时、despawn、reload和late result全部有receipt。
- [ ] AI-G11 `Fail`：SetBlackboard/EmitEvent/service产生真实typed command，shipping tree不接受result参数伪语义。

### Owner、Identity 与 Reload

- [ ] AI-G12 `Fail`：World/Entity/Agent/tree/schema/task/query handle有generation、wrap/reuse/cross-world防护。
- [ ] AI-G13 `Fail`：register/update/unregister/reload经prepare/migrate/atomic swap/LKG，active instance currentness成立。
- [ ] AI-G14 `Partial`：owner gate会等待in-flight；尚未先abort active task/adapter再销毁runtime data。
- [ ] AI-G15 `Fail`：World close、plugin unload/crash和callback panic不泄漏lease、instance、task、observer或debug state。
- [ ] AI-G16 `Fail`：多World/多agent调度无全局大锁瓶颈，ordered commit和digest跨线程稳定。

### Blackboard

- [ ] AI-G17 `Partial`：dense typed partitions/generation/observer存在；prepared handle和typed public writer未闭合。
- [ ] AI-G18 `Fail`：defaults/inheritance/rich types/schema revision/redirect/migration和LKG矩阵通过。
- [ ] AI-G19 `Fail`：多writer transaction、priority/conflict、observer revision和provenance结果确定。
- [ ] AI-G20 `Fail`：ordinary tick无字符串解析/全量Vec clone，partial update不清空未提交key。

### Perception 与 EQS

- [ ] AI-G21 `Partial`：pair cursor/count budget/backlog已存在；spatial index、wall/query budget和fairness SLA未闭合。
- [ ] AI-G22 `Fail`：sense unavailable/error/timeout不默认可见，diagnostic和degrade policy可验证。
- [ ] AI-G23 `Fail`：listener/source register/unregister、generation、despawn/unload和lost/expired事件完整。
- [ ] AI-G24 `Fail`：Sight/Hearing/Damage/Touch/Custom config、team/affiliation/tag和dominant sense通过。
- [ ] AI-G25 `Fail`：10k source/listener场景不做全笛卡尔积，P95/P99 latency和starvation有界。
- [ ] AI-G26 `Fail`：EQS query compile/run/time-slice/cache/abort/owner cleanup和typed result通过。
- [ ] AI-G27 `Fail`：team/squad/cover/reservation/Smart Object claim无双owner、泄漏或销毁后晚到提交。

### Schedule、Debug、Network 与 Save

- [ ] AI-G28 `Fail`：dedicated server、零/多camera、replay和不同viewer得到相同AI simulation digest。
- [ ] AI-G29 `Partial`：targeted debug snapshot存在；无reader零构造、rate/byte budget和causal trace未通过。
- [ ] AI-G30 `Fail`：AI authority/replication/late join/reconnect只同步允许状态且不重复task/event side effect。
- [ ] AI-G31 `Fail`：save/reopen/checkpoint/replay恢复program、Blackboard、task/query/timer/RNG并拒绝stale result。
- [ ] AI-G32 `Fail`：hot reload和plugin revoke期间debug/network/save读取同一qualified generation。

### Product、Failure、Scale 与领先性

- [ ] AI-G33 `Fail`：Vampire或首方AI场景从asset cook到Server/App真实运行，脚本平行AI已删除。
- [ ] AI-G34 `Fail`：AI Editor crate进入required compile lane，catalog/provider/factory/controller/overlay均有产品caller。
- [ ] AI-G35 `Fail`：malformed source、cycle/depth、provider failure、OOM/queue pressure、panic和disconnect均fail-close。
- [ ] AI-G36 `Fail`：1/100/1k/10k agent及perception/query/task storm记录P50/P95/P99、RSS、alloc、bytes和deadline miss。
- [ ] AI-G37 `Partial`：unit/allocation test底座丰富；same-agent、teardown、product、network/save和long-soak仍缺。
- [ ] AI-G38 `Fail`：ignored release benchmark进入固定hardware/profile/threshold/variance的required qualification lane。
- [ ] AI-G39 `Partial`：runtime package/catalog/dist存在；Editor catalog与真实product startup/authoring仍未闭合。
- [ ] AI-G40 `Fail`：与Unreal同源同规则correctness相同后，吞吐/延迟/RSS/alloc/network bytes有可复核领先证据。

Gate复算：Partial为G03、G08、G14、G17、G21、G29、G37、G39，共8项；其余32项Fail，0项Pass。

## 13. 首个允许实施的测试设计

MVP-00与M0 product truth允许实施后，首批变更必须先建立Runtime RED oracle：

1. `same_agent_concurrent_tick_has_single_execution_lease`：两个线程同tick同agent，node/task只执行一次且revision单调。
2. `timeout_parallel_and_revoke_abort_all_side_effects`：TimeLimit、Parallel terminal和owner revoke都收到cancel ack，没有晚到Nav/Animation/Script写入。
3. `behavior_source_cook_scene_agent_product_roundtrip`：真实source经import/cook/Scene load自动激活agent，Server无测试helper运行并记录artifact digest。
4. `blackboard_transaction_schema_migration_oracle`：多writer冲突、observer、default/inheritance、rename/redirect、active migration与save roundtrip。
5. `perception_provider_failure_never_grants_visibility`：physics unavailable/error/timeout、source despawn、listener unload和lost/expired顺序明确。
6. `camera_independent_ai_digest`：零camera、两个不同camera、Client与dedicated Server在相同输入下digest一致。
7. `eqs_abort_reload_late_result_fence`：running query遇owner unload/program reload，旧结果不能提交到新agent generation。
8. `ten_thousand_agent_budget_and_fairness`：固定密度与query/task storm下无starvation，P95/P99、alloc、RSS和deadline可复算。

这些测试通过前，不应继续以新增node enum、Workbench面板或ignored microbenchmark作为主里程碑完成证据。

## 14. Review closeout

- 本轮只新增 review/plan/index，不修改 AI、Runtime、Editor、plugin 或example源码。
- 未运行Cargo、App/Editor、Client/Server、PIE、asset cook、Scene roundtrip、network/save/replay、fault/scale/soak/profile或竞争benchmark；这是MVP-00下的静态current-source审查，不是动态验收。
- Plugins06的“M1-M4完成”只表示其局部计划条目已有实现/测试，不等于Runtime152的产品、生命周期、正确性和规模资格已经关闭。
- 实施前必须重新取selected source fingerprint，重判在途AI refactor对Partial/Open的影响，并先关闭Editor20五项父P0及Runtime152 M0。
- 本报告不声称整个引擎review完成；它完成的是Runtime AI/BT/BB/Perception/EQS这一纵向切片，后续继续扫描其他未覆盖子系统。
