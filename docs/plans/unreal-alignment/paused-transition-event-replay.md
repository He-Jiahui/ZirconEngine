# [Animation/Notify] 恢复播放不应重发已完成过渡的目标点通知

基线 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37`。固定源码复现推演，尚未编译、取得行为 RED 或实施修复。

## 1. 不一致

[现行运行合同:267–275](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/crates/zircon_plugins/animation/runtime.md#L267-L275>) 允许 transition-local 与活动状态使用不同的钟，但明确完成后不重复已扫窗口。这里是同一合法非循环窗口重扫，不要求新增 Montage/Slot/Section，也不把正常播放的双钟切换列作 bug。
## 2. 复现与影响

已装 Animation Runtime 的同一运行 World，以 [公开组件 setter:341–361](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/world/component_access.rs#L341-L361>) 装 `Idle→Run`（duration=.25，条件满足，无 exit gate/interrupt，单 Clip GraphRef，+1、非循环）；两片 ready、长 1s，仅 Run 有 `run_mid@.1875`。三帧各 .125s，仅将玩家 `playing` 设为 `true,false,true`。源码预测 F2 在 machine clock 停于 .125 时扫 Run `(.125,.25]` 并完成，F3 恢复后再扫同一段；每条事件流累计 2 次而非 1。无 seek/reload/epoch 更换/背压。
## 3. 根因与 UE 对照

[scan:522–558](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/parameter_apply.rs#L522-L558>) 冻结 machine clock 却传裸 dt；[推进:21–37](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/state_machine_transition.rs#L21-L37>) 推进目标；[完成/活动分支:239–295](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/state_machine_step.rs#L239-L295>) 退休 transition 后改用未同步推进的 machine clock。UE 本机 `6.0.0 / UE5 / CL 0`（未知 upstream revision）Montage Pause 停位置和扫掠；这是时钟原则，不是同例，更不是暂停所有 blend 权重。已存在的 UAF 原测只验首次激活帧 span-notify Begin 增、Tick>0，不测完成、暂停或点基数，未迁移/运行：

   ```text
   Engine/Plugins/Experimental/UAF/UAFTestSuites/Source/UAFCQTestSuite/Private/NotifyHandlerTests.cpp:694,1061–1155
   FUAFNotifyHandlerPipelineTests::Validate_TransitionTarget_FirstWindowStateNotify_BeginsOnActivationTick
   SHA256 8b4aaf5f3c7140f96b6bd0159fa0e850f7b53e9600dac9fc7dbceb6da79d6d38
   ```

**传统 UE 状态机的参照，不是这个 ZE 用例的通过记录：** 普通组件 `USkeletalMeshComponent::ShouldTickPose`（`Engine/Source/Runtime/Engine/Private/Components/SkeletalMeshComponent.cpp:1880–1912`）的条件包含非 `bPauseAnims`；`FAnimationActiveTransitionEntry::Update`（`Engine/Source/Runtime/Engine/Private/Animation/AnimNode_StateMachine.cpp:120–128`）与状态机自身 elapsed（同文件 617–625）都使用收到的动画更新 delta。不能把工具强制更新、一次零 delta 刷新、UAF 的相邻测试或 Montage 混合权重当成已执行本例。ZE [实际发出旧范围的两个分支](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/state_machine_step.rs#L364-L406)与[播放映射的提交](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/tick.rs#L379-L435)可独立核查。

## 4. Rust 落点

私有 scan/transition 权属统一有效 dt，保留暂停但 dirty 的合法重评。若决定暂停冻结过渡，不止修 dt：还要保证同 owner 的 active transition 经 clean-paused 帧不会被播放状态整表写回丢掉；不保留已撤销/替换实例。不同淡出政策也必须满足 no-repeat。
## 5. ECS/ABI 与范围

[发布:93–118](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/events.rs#L93-L118>) 是现有 typed/neutral 两种投影，分别判断只发一次；不改 ABI、资产、sampler、队列或全局按名消重。不是 [#171](https://github.com/He-Jiahui/ZirconEngine/pull/171) 同帧打断、[#207](https://github.com/He-Jiahui/ZirconEngine/pull/207) 单范围分页或 [#165](https://github.com/He-Jiahui/ZirconEngine/pull/165) 属性序列绑定。
## 6. 回归

复用 [生产插件夹具的首次过渡/逐帧订阅:385–524](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/tests/physics_tick/clip_events.rs#L385-L524>)，先验证 paused/resume 的唯一 `run_mid` 总数 1；冻结策略逐帧是 `[0,0,1]`，正常不暂停 `[0,1,0]`，完成后才暂停再恢复 `[0,1,0,0]`。另测多帧保持、首次正常双端点与 defer 回滚。计划用 `zircon_plugin_animation_runtime --no-default-features --features native-host --test physics_tick` 的定向 test profile，未执行。

UE 补充源身份（SHA-256）：`Engine/Source/Runtime/Engine/Private/Components/SkeletalMeshComponent.cpp` = `e1926cb602881d192763fa9ab1bd4a735a5893e5eb2accdba05a7baf43b02cff`；`Engine/Source/Runtime/Engine/Private/Animation/AnimNode_StateMachine.cpp` = `8dca0f41f0378ad8858f193d69cc8a4fdf43139c8114d20842c67d16ef8c161b`。文档不包含 Epic 源码。

## 7. 真实插件夹具：四案编译成功，尚未到达第一帧

[tests-only `d45b121315dea0e638202f2fd5c0423d48b9e139`](https://github.com/He-Jiahui/ZirconEngine/blob/d45b121315dea0e638202f2fd5c0423d48b9e139/zircon_plugins/animation/runtime/tests/physics_tick/paused_event.rs) 只新增四项逐帧见证及原 `physics_tick` 目标的一行测试挂载。输入是同一真实 Level 的公开播放器 setter `true / false / true`；两条各自独立订阅的事件流、根时钟、过渡表和待处理队列应在每帧输出。另设始终播放、淡化结束后才暂停、连续暂停四个 World 帧时刻中的两个帧等正反控制。下表把真实失败位置和**未执行到的行为**分开。

| 同一可执行文件 / 真实入口 | 观测 | 到达动画帧？ |
|---|---|---|
| 新增 `pause_fade_notify_` 四案 | **0 过 4 败**，全部在[原 SDK/插件夹具:10–19](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/tests/runtime_helpers.rs#L10-L19) 建立宿主处因 `physics.runtime: FoundationModule` 缺依赖而提前返回。 | 否，0 条帧 trace。 |
| 既有 `state_machine_transition_emits_from_and_to_graph_clip_events` | **0 过 1 败**，未改 fixture 与新增用例是同一个目录错误。 | 否。 |
| 既有 `level_tick_without_animation_plugin_does_not_advance_sequence_players` | **0 过 1 败**，走[不加载插件的原夹具:35–44](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/tests/runtime_helpers.rs#L35-L44)后在安装 World 扩展计划时报告 `AssetModule` 缺 `TasksModule`，不是上一行的错误。 | 否。 |

这是实际 Windows x64/MSVC Rust 1.94.1 的 `zircon_plugin_animation_runtime --no-default-features --features native-host --test physics_tick --no-run --profile test --locked --jobs 1 --message-format=json`。锁定依赖在全新的物理 D 盘任务池编译；按 Cargo 返回的唯一目标、包号、源和实际字节确认 exe，再在三次单独进程隔离 smoke/TEMP/config/log 中执行，没有改动物理路径保护。没有启 Jolt、运行编辑器、设备或 UE。构建回执 `d539e805ebd8097ee76c01b87d7db4d7383cdef57d221c242458bf1e73581b93`；可执行文件 SHA-256 `34370b4764899c45496db49d67371f61e7ee52d5cd233ada896ac5a8f6a53a2b`；三次执行回执按表序 `e5cb191b5f6712b78469bb480ada4c0234b8d29f7f8f47ade86cf948e6fd2d43`、`019de66ba625c13cc9627cc0c37cb90b4e8cd46d314fbe8997b293a7658dd8f9`、`ddcaa1ca7ce99a2739dc671d75c106b487db914cbc4c2950f44df11abdcadc67`。

**这不是该动画行为的 RED，也没有 GREEN。** 现有 `TestRuntime` 的基础模块组合、插件目录排序是独立的前置；不在此 PR 偷加假 Foundation、清 fatal 或替换既有 SDK 测试。对候选还有两条独立约束：正常 playing 的组件随 bundle 迁移不应重启淡化；同一机器资产热更新删除过渡端点时，不得让旧过渡永久复活。没有原帧证据之前，本 PR 不改生产。

## 原生宿主的暂停与恢复验证

以 `6d81e87ed8047a53b2133dae41f5c0f046890969` 的测试和动画实现为输入，叠加 [#216](https://github.com/He-Jiahui/ZirconEngine/pull/216) 和 [#217](https://github.com/He-Jiahui/ZirconEngine/pull/217) 中的五个宿主生产文件。本地组合提交为 `e65e9d4a6133daa088c2743ad97bbd74b06c18af`。两个独立对照分别运行有动画插件的正常过渡和无动画插件的序列播放器，两者都通过了帧后的场景断言。[原暂停用例](https://github.com/He-Jiahui/ZirconEngine/blob/6d81e87ed8047a53b2133dae41f5c0f046890969/zircon_plugins/animation/runtime/tests/physics_tick/paused_event.rs)

| 输入动作 | 每帧匹配点数量，每条事件流各自计数 | 结果 |
|---|---|---|
| 连续播放三帧 | 0、1、0 | 通过 |
| 过渡完成后暂停并恢复 | 0、1、0、0 | 通过 |
| 过渡中暂停，下一帧恢复 | 0、1、1 | 同一通知出现两次 |
| 过渡中暂停，保持一帧再恢复 | 0、1、0、1 | 暂停期间的过渡状态断言失败 |

每次实际 tick 的步长为 0.125 秒。第三组中，播放器根时间依次为 0.125、0.125、0.25 秒。暂停帧就从 Idle 进入 Run 并发出通知。恢复帧再次发出同一点。类型化和中立事件流的身份与数量逐帧一致。

四案共记录 14 帧，每案使用原测试的场景和图资源。下一项修复验证需要同时覆盖普通播放、同一实体的播放器替换与重新添加，以及使用相同资产 ID 的状态机热更新。
