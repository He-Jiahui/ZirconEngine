# Animation / Notify：root 状态机打断重复提交同一通知窗口

**状态：独立问题定义 Draft；本问题未修改引擎或新增/执行测试，行为仍待原实现回归验证。** 不把缺少完整 Montage 当现存 bug。调查与独立反证已逐分支核对，未将静态推导当测试 RED。

ZE 固定源码基线 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37`；本地 `cb495bb1c952a24c86f4513b8099bc4942c3a87d` 的相关 crates 零差异。UE 是本机参考树 Build.version 所示 6.0.0 / BranchName UE5 / CL 0，没有已知独立 Git commit；末尾文件 SHA256 固定来源。

> 取证说明：UE 本机 E: 路径及 D: 审计记录需要本机访问，D: 材料不随本 PR 发布，也不冒充 GitHub 可读证据；UE 源码正文不分发。[首轮全模块总览与既有 Draft](https://github.com/He-Jiahui/ZirconEngine/pull/169) 供去重，不代表引擎已验收。


## 1. 不一致是什么

现有 root 状态机正处于 A→B，且中断政策允许从 B 选择 B→C。若打断帧有非零时间推进，ZE 先采旧 A→B 本帧事件，随后采新 B→C 时仍把**整个播放器前帧时间**作为新 source 的事件起点，导致 B 已消费区间被再扫一遍。

精确 owner 是 [state_machine_step:91–143](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/state_machine_step.rs#L91-L143>)、[202–219](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/state_machine_step.rs#L202-L219>) 与 `sample_state_transition_clip_events` [364–406](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/state_machine_step.rs#L364-L406>)。不是 sampler 的单范围 cursor 重复，也不是同名 notify 在不同实例合法触发。

## 2. 实际影响是什么

同一实体、同一 B 状态、同一图/唯一 clip、同一事件轨道、同一 `playback_time_seconds` 被发布两次。即使没有 Montage、没有 skeleton reorder、没有 quaternion seam、没有属性序列，这条生产链仍成立。

发布端每个采样事件分别写 neutral `AnimationEventRecord` 和 typed `AnimationClipEvent`：[events:93–118,137–143](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/events.rs#L93-L143>)。这里指**每种流内各有两份**，不是把 neutral/raw 两种 API 的设计投影算成此次 bug。

有真实 gameplay 消费者，不止测试 helper：AI [collect_bus_events:125–138](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/ai/runtime/src/plugin/registration.rs#L125-L138>) 逐项读取 neutral 记录，[hearing_event_from_animation:176–194](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/ai/runtime/src/perception/adapter.rs#L176-L194>) 把 `ai.hearing` 转为听觉刺激，再由[待处理队列:151–156](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/ai/runtime/src/plugin/registration.rs#L151-L156>)保留每份输入。因此可增加重复 ingression/处理；后续 perception 可能合并刺激，**未证明伤害翻倍、声压翻倍或最终感知状态翻倍**，不作这些扩大结论。

## 3. 根因与 UE 怎样处理

**ZE 根因**：打断分支先在 [121–129](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/state_machine_step.rs#L121-L129>) 产生旧迁移事件；新迁移由 [131–135](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/state_machine_step.rs#L131-L135>) 以候选状态的已推进时间作为 source、目标 0 秒开始。稍后 helper 只在新旧 `(from_state,to_state)` 相等时复用旧局部时钟；打断后通常不相等，遂在 [374–381](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/state_machine_step.rs#L374-L381>) 回退到 `(pending.from_time_seconds,0)`，而没有携带“旧迁移已经消费至哪个候选状态局部时间”。

**最直接的 ZE 反证/已有正确实现**：同库 layer 路径先采旧窗口后在 [state_machine_layers:166–183](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/state_machine_layers.rs#L166-L183>) 把新 `event_start` 设为 `(candidate.from_time_seconds,0)`；[264–273](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/state_machine_layers.rs#L264-L273>) 把该起点传给新迁移采样，[313–349](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/state_machine_layers.rs#L313-L349>) 分别使用 source/target 局部起点。相同 witness 在这条生产代码下新 source 是零长度范围；这不是需要设计新架构才能修的问题。

**UE 对应原则**是同一状态机 node 的一次 `Update_AnyThread` 标准 blend 路径中，对共享 state 的更新只执行一次，而不是跨所有实例全局按名字消重：

1. [FAnimNode_StateMachine::Update_AnyThread:514–529](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimNode_StateMachine.cpp#L514-L529>) 每次 `Update_AnyThread` 调用清 `StatesUpdated`，随后可遍历多个活跃 transition；[898–914](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimNode_StateMachine.cpp#L898-L914>) 的两端均通过 `UpdateState`。
2. [UpdateState:1577–1584](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimNode_StateMachine.cpp#L1577-L1584>) 先检查/登记 `StatesUpdated`，因此旧 transition 的 Next=B、新 transition 的 Previous=B 不会各更新同一 B 一遍。
3. 对单点 notify 的连续前进区间，UE [UAnimSequenceBase::GetAnimNotifiesFromDeltaPositions:430–498](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimSequenceBase.cpp#L430-L498>) 使用双向半开穿越判定。ZE 同样已有单范围半开及续读合同，不必改成别的语言模型。
4. UE [FAnimNotifyQueue::AddAnimNotifiesToDest:53–74](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimNotifyQueue.cpp#L53-L74>) 只对 notify-state 使用 unique；单点 notify 仍逐项加入，允许合法多次 loop/多源触发。故不能照此题对 ZE 下游事件流作全局 `(entity,name,time)` 去重。

UE Montage 的另一条合同是 [Stop:1566–1586](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimMontage.cpp#L1566-L1586>) 锁存 `bInterrupted`、[HandleEvents:2818–2826](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimMontage.cpp#L2818-L2826>) 对已打断 montage 跳过通知。**不把这条 Montage 停止政策强加给 ZE 的状态机 crossfade**，也不据它删除旧迁移正常扫过的本帧事件。本问题 oracle 是“同一 B 区间不应重复”，不是“打断帧必须没有旧事件”。

## 4. Rust 中怎样做

复用现有 Rust owner 和 source/target 局部时间，仿照已经存在的 layer 实现，显式携带新迁移的事件窗口起点。无需 UObject、虚调用或 UE `StatesUpdated` 容器的逐字翻译。

保持以下既有合同：同时间多个 track 可分别触发；同 clip 的不同 state/node/loop occurrence 不自动合并；每个固定范围独立的有界 cursor；同 producer/entity 原子 admission；defer 后时间、状态与 trigger 保持可重试；world replacement epoch 失效。不能把重复区间的 producer 错误转移给 sampler 或 ECS event-store。

## 5. 接口/ECS 差异是什么

- UE state-machine node 以 `StateIndex`/图更新计数约束更新；ZE 的 `MachineInstanceKey`、ECS player 和 `LevelSystem` 播放时钟/transition map 分担身份与状态。这是实现差异，只有因此重复提交同一窗口的具体分支才是错误。
- ZE [AnimationStateTransitionAsset:100–121](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/framework/animation/asset/state_machine.rs#L100-L121>) 已真实支持 None/CurrentToNext/NextToNext/Both、duration 与 exit time；不需要为本问题扩展 schema。
- ZE [AnimationEventTrackAsset:27–35](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/framework/animation/asset/clip.rs#L27-L35>) 是时间点/名称/payload，不是 UE NotifyState Begin/Tick/End、branching point 回调或完整 notify class。其没有 UE 全套 lifecycle 是明确能力差距，不是此候选的根因。
- 修正可限定在 plugin 内部 helper/参数，不变 binary asset、共享 ABI、World event 类型或 ECS 布局。

## 6. ZE 具体最小修改草案

仅作为后续评估，不在本轮执行：

1. 在 root transition 解析中显式保留事件 source/target 起点；普通首次 A→B 继续使用当前帧起点及 target=0，继续中的同一迁移使用其 previous 局部时钟。
2. **成功打断且旧迁移事件已记录后**，新 source 起点改为 `candidate.from_time_seconds`，target 起点为 0；新 source 当前终点也是该候选时间，因此不再回扫。可参考现有 layer owner，不要求抽一个新架构/公共服务。
3. 仍保留旧迁移的本帧事件、冻结的 blended source pose、trigger 消费与 admission rollback。不把所有新迁移事件删掉；[首次迁移两端事件控制:385–496](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/tests/physics_tick/clip_events.rs#L385-L496>) 必须保持。
4. 先新增“附录：对照测试与回归门”真实帧回归，获得原实现行为 RED 后才能作最小修正、验证 GREEN。helper 的具体参数改法、target 重入/相同名字边界与失败回滚由 lead 复核，不把此草案称为已验证补丁。


## 附录：源码可达见证

### 3.1 正常资产与输入

用现有 [interruptible_transition_state_machine:259–311](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/tests/animation_assets.rs#L259-L311>) 的形态：A=Idle、B=Run、C=Sprint，A→B duration=1/策略 Both，B→C duration=1/策略 None，exit time 均 None；各状态引用不同 GraphRef，每图恰有一个非循环、speed=1、weight=1 的 clip。骨架及三资产 ready，三个 clip duration=1；只有 B 在 **0.375** 有一条非空事件（可命名 `ai.hearing`、payload `0.75`）。

实体安装 skeleton 与 playing=true 的状态机 player，active_state=Idle、start=true、interrupt=false。连接现有 typed dormant subscription，并为 neutral `AnimationEventRecord` 独立连接同样的订阅以观察每种流；首帧 tick **0.25** 后分别 update/drain（应无事件）；把 interrupt 改 true，再 tick **0.25** 后分别 update/drain。所有取值可二进制精确表示，不依赖浮点近似、loop seam、零权重或同名事件碰撞。

### 3.2 源码逐分支演算（未运行）

| 时刻/分支 | 状态局部时钟与事件窗口 | B 的 0.375 事件 |
|---|---|---|
| 首帧开始 | root 播放时钟 0；无旧 transition | 无 |
| 首帧 tick 0.25 | [scan:522–556](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/parameter_apply.rs#L522-L556>) 产生 root 0→0.25；[begin:160–174](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/state_machine_step.rs#L160-L174>) 建 A→B，from=0.25、to=0.25、elapsed=0.25。A/B 都扫 (0,0.25]。 | 0 |
| 第二帧 tick 0.25，选择打断前 | root 0.25→0.5；[advance:21–37](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/state_machine_transition.rs#L21-L37>) 将旧 A→B 推到 from=to=0.5、elapsed=0.5，未完成；[候选选择:64–110,116–120](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/state_machine_transition.rs#L64-L120>) 允许从 B 选 B→C，候选 source 时间=0.5。 | 尚未发布 |
| 成功打断，旧迁移采样 | 旧 `(A,B)` 与 pending.transition 相等，起点使用 previous 的 (0.25,0.25)。采 A(0.25,0.5] 与 **B(0.25,0.5]**。 | 第 1 份 |
| 建新 B→C 并再采样 | 新 from=0.5、to=0、elapsed=0；与 pending 的 `(A,B)` 不同，helper 回退 `(pending.from=0.25,0)`。再采 **B(0.25,0.5]** 和 C(0,0]。 | 第 2 份 |
| 正常 admission / drain | 四个 ranges，低于 queue=256/drain samples=32/events=64 的默认界限；资产 ready、无旧 backlog、同 epoch；每个 range 新建自己的 cursor。 | typed 2 + neutral 2 |

这里的“两份”是**源码因果结论，未执行的待验证行为**。期望同一 B track occurrence=1；不要求全实体总 event=1（若 A/C 另有合法事件，不应被抹掉）。首帧姿态为 2.5、打断帧 frozen blended source 为 5.0（A=0/B=10/C=20 的平移夹具），可作为源姿态连续性控制，不涉及 #147 quaternion。

### 3.3 完整生产链与没有隐藏消重的证据

1. 真实场景入口：[register_runtime_system / run_animation_runtime_system:25–61](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/runtime_system.rs#L25-L61>) 将 `animation.evaluate` 注册到 PostUpdate；不是孤立 helper。
2. [tick_animation_world:32–84](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/tick.rs#L32-L84>) 取得 epoch/旧播放时钟并扫描；[resolve_machine_instance:39–75](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/nested_machine_resolve.rs#L39-L75>) 在 root 有活跃 transition 时返回本实例；本见证不需要 SubMachine。
3. 第 2、3 节的 root 双窗口进入 `sample_machine_state_events` [245–276](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/nested_machine_sample.rs#L245-L276>) → [sample_state_events:158–192](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/state_graph_sample.rs#L158-L192>) → [GraphRef 事件:264–295](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/state_graph_sample.rs#L264-L295>) → [graph event ranges:81–107](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/graph_evaluate.rs#L81-L107>)。B 只有一个图贡献，排除多节点合法多次发射。
4. [tick:237–285](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/tick.rs#L237-L285>) 直接扩展 ranges；[enqueue_clip_event_samples:25–50](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/events.rs#L25-L50>) 只按 entity 分组，无 range 去重。
5. [AnimationRuntimeState::enqueue_clip_event_sample_batches:98–113](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/level_system/frame_state.rs#L98-L113>) 对每个 range 分别创建 `at_range_start` cursor；[drain_animation_clip_events:174–232](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/level_system/animation_runtime.rs#L174-L232>) 分别采样并 `events.extend(batch.events)`。队列/cursor 本身履行现有合同；错误在上游重复范围。
6. [ProjectAnimationClipEventSampler:21–38](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/animation/clip_event.rs#L21-L38>) 加载 B；[发射:174–200](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/animation/clip_event.rs#L174-L200>) 及[半开 cursor:380–398](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/animation/clip_event.rs#L380-L398>) 会各返回一次同一个 interior event。
7. [tick:339–348](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/tick.rs#L339-L348>) 在无旧 backlog 时本帧发布；[events:93–107](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/events.rs#L93-L107>) 逐项进入 World。进一步核到 [World::send_event:16–22](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/world/events.rs#L16-L22>) → [EventStore::send/send_by_id:243–255](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/ecs/events/store.rs#L243-L255>) → [Events::send:41–45](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/ecs/events/queue.rs#L41-L45>)，最终逐份 `next.push`，没有隐藏值消重；接“实际影响是什么”节消费者。defer、退休 epoch、缺资产不是该 witness 的必要条件。


## 附录：Montage 能力与接口差异

| 契约 | 已核 UE 真入口/符号 | ZE 对照与分类 |
|---|---|---|
| Montage 播放与 group 竞争 | [UAnimInstance::Montage_PlayInternal:2879–2943](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimInstance.cpp#L2879-L2943>) 校验资产/slot、按 group 停旧实例、建立 `FAnimMontageInstance`；[Montage_StopInternal:2988–3015](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimInstance.cpp#L2988-L3015>) 是实际停止入口。 | ZE 已有 Clip/Graph/StateMachine player，而本次 framework/plugin schema 和 owner 搜索无 Montage 资产、group/slot instance 对应实现。**明确未实现能力，不立 bug。** |
| section 跳转和 next link | [FAnimMontageInstance::JumpToSectionName / SetNextSectionName / SetNextSectionID:1816–1875](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimMontage.cpp#L1816-L1875>) 区分立即定位与更新 next/prev 链；不是把所有位置变化都当连续扫掠。 | ZE 没有同等 section graph/play/jump API。graph speed/loop 或修改 player time 不等于 Montage section。**能力缺失；seek/discontinuity 完整政策仍未知，不扩大本题。** |
| 帧内分段与分支点 | [FMontageSubStepper::Advance:2158–2176,2280–2337](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimMontage.cpp#L2158-L2337>) 在 branching point 与 section 边界截断；[FAnimMontageInstance::Advance:2571–2602,2673–2748](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimMontage.cpp#L2571-L2748>) 有迭代上限、重新检查事件修改的位置与 next link。 | ZE 有事件采样预算切片/续读，但没有同步 branch callback 改下一段播放的合同。**预算分片不等于 Montage substep**；不能因缺回调重入架构而凑 bug。 |
| segment → 子动画时间 | [FAnimSegment::ConvertTrackPosToAnimPos / GetAnimNotifiesFromTrackPositions:74–151](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimCompositeBase.cpp#L74-L151>) 根据 segment trim、speed、loop count 和轨道方向换算并拆区间；[FAnimTrack:626–633](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimCompositeBase.cpp#L626-L633>) 遍历有效 segment。 | ZE GraphRef 将 from/to 乘各 clip speed，direct Clip/Graph 都没有同等 Montage segment trim/finite loop-count 资产。**部分基础对应 + 明确缺失，不要求照搬 C++ 对象层。** |
| Slot 的 update/evaluate | [FAnimNode_Slot:49–85,97–129](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/AnimGraphRuntime/Private/AnimNodes/AnimNode_Slot.cpp#L49-L129>) 更新权重、按 relevant/always-update 决定 source；[FAnimInstanceProxy::SlotEvaluatePose:1883–1965](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimInstanceProxy.cpp#L1883-L1965>) 消费 montage evaluation snapshot、按 slot 采姿态，避免并行求值访问 live instances。 | ZE layer/mask/additive 有真实功能，但不是 Slot/group 竞争；ECS snapshot 与 UE proxy 的组织不同不算错误。零权重事件政策有现存 TODO，见第 5 节；不靠 UE 阈值直接判 bug。 |
| queued notify 与同步 branching point | [HandleEvents:2828–2885](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimMontage.cpp#L2828-L2885>) 分开 Montage notify、各 Slot 子动画 notify 和即时 branching point；[FAnimNotifyQueue:138–147](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimNotifyQueue.cpp#L138-L147>) 再按 slot relevance 接入。 | ZE clip 点事件经有界 queue 进入 World/AI，属于**真实已有通知子集**，不是粒子 birth。没有 NotifyState/branching point 全套；本题是该已有子集的重复生产。 |
| 单点双向扫掠 / 相同时间续读 | UE [GetAnimNotifiesFromDeltaPositions:438–495](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimSequenceBase.cpp#L438-L495>) 双向区间；[GetAnimNotifies:383–427](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimSequenceBase.cpp#L383-L427>) 做有限 loop 拆分。 | ZE [clip_event:94–156,265–331,380–398](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/animation/clip_event.rs#L94-L398>) 已支持正/逆半开、event/byte/span budget、相同时间按名称/track-index续读；UE offset 与 ZE raw time 不是逐值相同。**已存在合同；不重报旧报告的“逆向全丢”过时描述。** |
| 停止、暂停、生命周期 | UE [Stop/Pause:1558–1681](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimMontage.cpp#L1558-L1681>) 区分停止 blend-out/interrupt latch 与 `bPlaying=false`；[TriggerAnimNotifies:1991–2099](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimInstance.cpp#L1991-L2099>) 区分 source dispatch 与 notify-state lifecycle/实例身份。 | ZE `playing`、transition continuity 与队列延迟属于不同层；没有 Montage end/blending-out/interrupted callback 等价类型。**缺完整生命周期不立题**；未证明所有 pause/cancel/queued-old-event 组合正确。 |
| 状态机交叉淡化被打断 | UE 标准 blend 路径的单 node/单次 Update_AnyThread state update guard，见“根因与 UE 怎样处理”节。 | ZE 已有 Both 等策略与 frozen blended source。**唯一已证源码错误是 root 新 source 事件起点回退；layer 同逻辑有正确起点反证。** |
| Sequencer skeletal track → 动画系统 | [MovieSceneSkeletalAnimationSystem:510–543](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/MovieSceneTracks/Private/Systems/MovieSceneSkeletalAnimationSystem.cpp#L510-L543>) 根据 Playing/Jumping/Stopped/Scrubbing 等状态决定通知政策；[945–975,1036–1067](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/MovieSceneTracks/Private/Systems/MovieSceneSkeletalAnimationSystem.cpp#L945-L1067>) 区分 Sequencer instance 与 dynamic montage fallback；[SetSequencerMontagePosition / Preview…:3052–3148](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimMontage.cpp#L3052-L3148>) 区分 forced swept range、plain position 与 preview fire toggle。 | ZE [AnimationSequenceAsset:68–76](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/framework/animation/asset/sequence.rs#L68-L76>) 是属性 binding/channel，不是 UE skeletal animation section；[TimelineModel:134–146](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/timeline/model.rs#L134-L146>) 明示基础协议不保存 montage/section asset。**没有证明完整 runtime skeletal track / timeline event→notify 分发链；不把描述 DTO 当产品。** #165 不重做。 |


## 附录：反证与既有问题去重

- **不是既有 Draft 根因**：#147 是 quaternion Base 混合；#164 是合法骨槽计算顺序；#165 是暂停属性序列前级采样准入遮蔽 binding stale；#153 是新生粒子帧内年龄。本题仅 root 迁移事件窗口，数值取全平移、单骨可比，不借这些问题成立。
- [旧 08C P1-15:186–190](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md#L186-L190>) 与 [Runtime221 RT-AN-07:193–203](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/optimize/zircon_runtime/221-runtime-animation-source-compiled-pose-skinning-ik-root-motion-event-editor-product-current-working-tree-review.md#L193-L203>) 有宽泛事件/身份/回执/实例计划；没有本题 root 打断双窗口见证。本题不扩大为这些重构方案，也不重报已落地的 heap/reverse 修正。lead 仍应按全历史/并发候选复核归属；本轮禁止远端，未刷新 PR 列表。
- [历史 M3 记录:14](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/zircon_plugins/04/2026-07-11-animation-m3-foundation-output-records.md#L14>) 曾修首次迁移事件 2→0 丢失，采用 `pending.from_time_seconds`/target 0。不能因此删除首次迁移扫掠；此次是该 fallback 被用于**旧迁移已经消费过本帧的打断分支**。历史通过不含当前非零打断帧的唯一事件计数见证，不当作反例已通过。
- **同时间多 track 合法**：[same_time_duplicate_events_resume_by_track_index:328–346](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/animation/tests/clip_event.rs#L328-L346>) 要求两个同名同时间不同 track 都发；故全局 Set/去重不是可行修法。
- **正/逆 seam 明确现合同**：[reverse_looping…:219–250](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/animation/tests/clip_event.rs#L219-L250>) 和 [forward_looping…:253–274](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/animation/tests/clip_event.rs#L253-L274>) 指定方向侧端点。UE trigger offset/segment 规则不同，不未经裁决另报 seam bug。
- **零权重**：[graph_evaluate:80](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/graph_evaluate.rs#L80>) 已有 CR-PLUGIN-ANIMATION-0004 待确认。UE 有 per-notify threshold、leader/follower、slot relevance、LOD/chance 过滤；ZE 无相同字段。无冻结的 ZE 对等政策，不据此另提问题；本 witness 每个状态图内唯一 clip 的贡献 weight=1，不依赖图内零权重政策。
- **Editor 能力与可达性分离**：[capabilities:17–57](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/animation_editor/capabilities.rs#L17-L57>) 明确 Clip node authoring/runtime preview=false。ZE Runtime 已能程序安装/加载 GraphRef 状态机，见真实现有 tick harness；没有声称用户当前 GUI 可以从零完成 Montage/Notify authoring。
- 未审完 root/nested/layer 所有 pause/seek/re-entry/hot-reload/transition-completion 组合，未验证 root motion、网络同步、sync marker、time stretch、ability montage task、NotifyState teardown/reentrant callback 的全产品后果。此次不添加第二候选，也不把“未覆盖”写成“正确”。


## 附录：对照测试与回归门

**同一 A→B→C 非零打断帧重复 notify 的 UE 原自动化用例未找到。** 限定检索为：Runtime Engine Private/Tests 的相关符号、Engine Animation 命名 Test 文件、Source/Plugins 的 `*Test*.cpp` 内 Montage/AnimNotify/GetAnimNotifies/FAnimMontageInstance 符号、AnimGraphRuntime 源的自动化宏、MovieSceneTracks Private/Tests 的 skeletal/Montage 相关符号。不是全 UE 测试内容逐行穷尽，也未运行任何 UE 测试。

| 来源 / 真实符号 | 原输入与断言 | 迁移资格 / ZE 既有落点 |
|---|---|---|
| UE UAF `FUAFNotifyHandlerPipelineTests::Validate_TransitionTarget_FirstWindowNotify_ProducedWithWeight`，[962–1056](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/Experimental/UAF/UAFTestSuites/Source/UAFCQTestSuite/Private/NotifyHandlerTests.cpp#L962-L1056>) | FalseSeq 无 notify，TrueSeq 0.05 有点 notify；transition duration 0.8，先两次 0.1 tick 后 toggle，再跑 6 ticks；只断言 fire count 增加、记录 weight>0.01。 | **真实已有近邻测试，不是同例，也没有精确“只发一次”断言**；不得拿名称中的 first-window 当本题 cardinality oracle。可作为首次迁移不能丢 target event 的参考。 |
| UE UAF `Filter_Debounce_TapDance_SameSequenceAllFire`，[3626–3671](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/Experimental/UAF/UAFTestSuites/Source/UAFCQTestSuite/Private/NotifyHandlerTests.cpp#L3626-L3671>)、[3722–3728](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/Experimental/UAF/UAFTestSuites/Source/UAFCQTestSuite/Private/NotifyHandlerTests.cpp#L3722-L3728>) | 同一 1 秒 sequence 的四条同 class notify 位于 0.05/0.15/0.25/0.35，debounce window 0.5；五次 0.1 tick 后 Count=4。 | 可迁移**四个合法 occurrence 必须全部保留**的输入/计数控制；UAF handler/debounce 本身 ZE 没有对应能力，不称完整框架迁移，更不是经典 Montage 重复问题的原测。尚未迁移。 |
| UE `FAnimNode_StateMachine::UpdateState` + `UAnimSequenceBase` 双向范围规则，见“根因与 UE 怎样处理”节 | 同一状态机 node 的一次标准 blend 更新中，重叠 transition 通过 `StatesUpdated` 不重复更新共享 state；interior 单点只因其唯一穿越窗口触发。 | **本问题的测试是源码派生，不是 UE 原测试搬运**。“附录：源码可达见证”两个 0.25 tick、B event=0.375、期望 B count=1；原实现源码预测=2，未执行。 |
| ZE `state_machine_transition_emits_from_and_to_graph_clip_events`，[385–496](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/tests/physics_tick/clip_events.rs#L385-L496>) | 首次 Idle→Run，两个不同 clip 0.05 事件，0.1 tick 后两个都发。 | **保留控制**。新回归应放同一[clip_events 测试叶](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/tests/physics_tick/clip_events.rs>)，复用 [subscribe/drain:499–524](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/tests/physics_tick/clip_events.rs#L499-L524>)，不是另造运行器。 |
| ZE `one_shot_trigger_interruption_waits_for_source_pose_then_consumes`，[330–467](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/tests/state_machine_interruption.rs#L330-L467>) | 手工放入旧 transition，缺 source 时保留 trigger；恢复后 **0.0 tick** 打断，再 0.5 tick 验姿态 12.5。 | 真实 source-failure/trigger/continuity 控制；**没有事件计数，且打断发生在零 dt，不能覆盖此次同范围双扫。** 新例使用正常两帧建 transition，不依赖手工注入。 |
| ZE `layered_state_machine_interruption_preserves_blended_pose_continuity`，[452–533](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/tests/blend_space_state.rs#L452-L533>) | 0.5→0.0→0.5 tick 验 5→5→12.5；layer 路径。 | layer 代码的正确起点是直接反证；现有测试也没有非零打断帧事件计数。建议将“附录：源码可达见证”夹具放入 layer wrapper 作相同 count=1 正向控制，不能宣称现测已涵盖。 |
| ZE [transition_policy_matrix:7–34](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/tests/animation_state_transition_contract.rs#L7-L34>)、[event sampler tests:16–55,165–216,328–346](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/animation/tests/clip_event.rs#L16-L346>) | None/CurrentToNext/NextToNext/Both；loop budget/cursor；同时间 track 稳定续读。 | 保留矩阵与 sampler 控制；不需为了 producer 起点错误改 sampler 或其同名 track 语义。 |
| ZE `one_shot_trigger_deferred_clip_event_admission_retries_before_consuming`，[471–570](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/tests/state_machine_interruption.rs#L471-L570>) | 填满 256 range 后，transition/trigger 延迟；释放 backlog 后重试。 | 最小修正必须保留 admission/rollback；应在同 harness 为打断见证补 defer→retry 不重发控制。该组合仍是**源码派生的新回归**，尚未新增/运行。 |

建议新增例名（仅草案、当前源码没有）：`state_machine_interruption_emits_shared_source_clip_occurrence_once`。关键断言是 typed 和 neutral **各自**唯一 B occurrence、非打断对照 count=1、layer 对照 count=1、无 timeline/无 quaternion 依赖；保留首次迁移双端事件、合法多 track、后续 pose continuity 与 trigger defer。


## 附录：修复可行性与未运行门

- **可行性**：root/layer 都有现成局部时钟与 `candidate.from_time_seconds`；问题可在内部 event start 参数处止损，无需新 Montage/Slot、统一 AnimationInstance 架构或公有 ABI。layer 现实现给出可复用模式，但不是 root 补丁已通过的证明。
- **第一门——lead 独立复核/去重**：复核固定 UE SHA 和 root 与 layer 分支；与已发布 #147–157/#159–168、总览 #169 及并发新候选对照，再决定独立 Draft。这里没有远端动作。
- **第二门——回归可编译前置**：新回归落现有 [physics_tick 入口:1–31](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/tests/physics_tick.rs#L1-L31>) 与其 clip event include；依赖 [runtime_helpers:10–24](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/tests/runtime_helpers.rs#L10-L24>) 的真实 animation/physics 插件。若既有 fixture/API drift 阻止编译，独立记录，不把编译失败当重复通知 RED，不顺手改其他引擎问题。
- **第三门——最小包/feature/profile**：未来只需 `zircon_plugin_animation_runtime`、`--no-default-features --features native-host`、`--test physics_tick`，test profile，筛选新 source-derived 用例及必要既有 transition/event 控制。[包清单:8–22](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/Cargo.toml#L8-L22>) 显式带 runtime animation/physics-contracts 与 physics dev dependency；当前 target 名来自自动发现文件名，不沿用历史记录中的旧 harness 名。实际执行前须按规范重新声明确切命令，不在本轮运行任何 Cargo。
- **物理目录门**：[runtime_helpers:109–144](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/tests/runtime_helpers.rs#L109-L144>) 要求 `D:/cargo-targets/zircon-local/win32/<task>/target` 与同 task 的 `build`；smoke 在 `build/fixtures/<smoke>`，TEMP/TMP/TMPDIR 同 smoke，测试二进制物理位于 target/build，无 alias/reparse 绕过。这是后续能运行的前置，本轮未创建这些目录/fixture。
- **行为门**：原实现需真实 count=2 的 RED（oracle=1），补丁后相同输入 GREEN；再验普通首次双端通知、same-time duplicates、正逆 loop/cursor、policy matrix、源姿态失败、trigger defer/retry、layer continuity。每项都没有本轮执行证据。
- **产品门独立**：runtime/Editor GUI 播放、真实 gameplay 动作、音效、AI 最终感知、性能、网络均未验收；不以窄事件计数推断全套 Montage/动画对齐。


## 附录：UE 来源文件指纹

| 文件 | SHA256 |
|---|---|
| [Build.version](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Build/Build.version>) | `3DCD8B6654872D013F2B00AA5F0EF1E0DB89D4E8627CE820554675A27F228EA6` |
| [AnimMontage.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimMontage.cpp>) | `1DBD8796D7BBDE0F0E51D00C8CFCBD758B1EA2B810119768BC4EE8B9E5C8BB10` |
| [AnimInstance.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimInstance.cpp>) | `043B137BE446571454112CA2D7717743782C892C0A720CBE0B6C2C997514D897` |
| [AnimInstanceProxy.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimInstanceProxy.cpp>) | `AD0AEB91BDEEC985A3726F4F946B4B4578E78F888C24D9DAD7D7B3FAB0070FC1` |
| [AnimCompositeBase.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimCompositeBase.cpp>) | `D44F671236F97955756C329411102CBE8A84953BEB7346314165DB71002E5F32` |
| [AnimSequenceBase.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimSequenceBase.cpp>) | `538CEDBEE8CAD1313C301078374A2750C2929E5EC198A54D2254D0C79AE6C3F7` |
| [AnimNode_StateMachine.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimNode_StateMachine.cpp>) | `8DCA0F41F0378AD8858F193D69CC8A4FDF43139C8114D20842C67D16EF8C161B` |
| [AnimNotifyQueue.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimNotifyQueue.cpp>) | `082686BC5B5EC128C919A44B92E9DFEB0DE6291E8775C9AB1C9C49714124EA8E` |
| [AnimNode_Slot.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/AnimGraphRuntime/Private/AnimNodes/AnimNode_Slot.cpp>) | `EB59616FB262756BC458CF4210FF1C63C8984E5F6E6C0C51A15024733876D525` |
| [MovieSceneSkeletalAnimationSystem.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/MovieSceneTracks/Private/Systems/MovieSceneSkeletalAnimationSystem.cpp>) | `80B2D7393C056230CD156C37BC23593E16293C541FA26F21BBBB44782FC6EC4A` |
| [NotifyHandlerTests.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Plugins/Experimental/UAF/UAFTestSuites/Source/UAFCQTestSuite/Private/NotifyHandlerTests.cpp>) | `8B4AAF5F3C7140F96B6BD0159FA0E850F7B53E9600DAC9FC7DBCEB6DA79D6D38` |


## 测试防假绿约束

先断言第一帧确实建立 A→B、第二帧确实打断成 B→C，并检查对应局部时钟；再分别计数 typed/neutral 流中的唯一 B 事件标记、目标实体/target_id、payload 及 0.375 时间。neutral 投影明确 `clip: None`，typed 消息也不携带本夹具可据此筛选的 B `clip_id`，不能靠不存在的字段筛选而得到假绿。A/C 无该标记；合法其他 track/occurrence 不能被全局消重。当前所有回归仍未新增或运行。

## 根状态打断产生重复通知的实测

三个根状态打断用例在真实物理帧中多发通知，typed 和 neutral 两条事件流读到相同的多余条目。两个新对照通过。五个既有对照中两个通过，另三个在姿势或状态访问处失败。

测试通过已注册的 Physics 和 Animation 插件加载真实场景、骨架和剪辑。共采集 16 个实际帧，每帧读取两条事件流，队列积压均为零。表中计数分别适用于每一条流。

| 打断选择 | 打断帧预期 | 打断帧实测 | 后续帧实测 |
|---|---|---|---|
| 从当前态发起 | `.375` 的 idle 一次，`.5` 的 shared 两次 | idle 两次，shared 三次 | `.625` 的 idle 与 `.125` 的 sprint 各一次 |
| 从下一态发起 | `.375` 的 run 一次，`.5` 的 shared 两次 | run 两次，shared 三次 | `.625` 的 run 与 `.125` 的 sprint 各一次 |
| 返回先前目标 | `.625` 的 idle 和 `.125` 的 run 各一次 | 同帧额外出现 `.375` 的 run | 随后一帧又正常跨过该 `.375` 通知 |

持续原过渡时，两段剪辑各自产生的同名 shared 事件都保留。分层状态机的对应路径也通过。三个失败用例均到达相同的打断帧事件断言。五个既有对照中，双端剪辑事件、延迟事件准入通过。分层姿势连续性与触发恢复在手骨平移断言失败，退出时间门在读取状态映射时失败。原生套件共 4/10 通过。

Windows x64/MSVC，集成目标为 `zircon_plugin_animation_runtime` 的 `physics_tick`，启用 `native-host`。运行环境使用五个来自既有 SDK 测试宿主的文件，并让文件系统夹具绑定当前物理编译目录。此提交中的两处变更均属 `physics_tick` 回归。

[通知采样测试](https://github.com/He-Jiahui/ZirconEngine/blob/acd348a8d0a74e398ba327683d4192af906537b4/zircon_plugins/animation/runtime/tests/physics_tick/interruption_notify.rs#L405-L437) · [既有退出时间与姿势断言](https://github.com/He-Jiahui/ZirconEngine/blob/acd348a8d0a74e398ba327683d4192af906537b4/zircon_plugins/animation/runtime/tests/state_machine_interruption.rs#L168-L179)

## 从已采样的位置继续扫描通知

修复只调整根状态机接管旧过渡时的源剪辑起点。旧过渡已经扫描到当前源时刻，新过渡的第一次扫描从该时刻开始。目标剪辑继续按自己的时间窗口产生通知。

| 真实运行时的路径 | 打断前 | 修复后 |
|---|---|---|
| 选当前态或下一态作为新源 | `.375` 的源通知重发，`.5` 的同名通知共出现三次 | 源通知出现一次，两个活动剪辑各自的 `.5` 通知均保留 |
| 打断后返回先前目标 | `.375` 的 run 通知出现在决策帧和下一帧 | 只在下一帧实际跨越该点时出现 |
| 原过渡继续、对应的分层过渡 | 两项通过 | 两项通过 |

五个通知用例在修复前通过两项，修复后全部通过。测试实际读取 16 帧，各自检查 typed 与 neutral 通知流的实体、目标、payload 和时间。另五个现有对照在两版都通过两项。剩余三个用例的名称、断言位置及诊断相同，两处手骨平移断言失败，一处状态映射缺项。十项合计由 4/10 提升至 7/10。

运行使用 Windows x64/MSVC 的 `physics_tick` 集成测试、`native-host` 特性和先前用于同一组基线的插件宿主。公开改动限于一个动画运行时文件。

[根过渡的事件扫描](https://github.com/He-Jiahui/ZirconEngine/blob/d2a296bf446c581d2ad320a8bbd08a13b6b873ad/zircon_plugins/animation/runtime/src/evaluation/pipeline/state_machine_step.rs#L205-L230)
