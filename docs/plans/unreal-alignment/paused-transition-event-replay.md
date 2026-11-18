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
