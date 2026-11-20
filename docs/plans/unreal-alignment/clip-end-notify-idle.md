# 非循环动画播完后，空通知范围仍让按需宿主请求下一帧

## 1. 使用场景与合同

启用 Animation runtime 插件，在一个关卡中用 `World::set_animation_player` 播放已就绪的 **2 秒**片段。只装一个直接播放器：`playing=true`、`looping=false`、`weight=1`、速度 `8`、起点 `0.5s`。在 `0.75s`、`1.75s` 放两个点通知。一次 `0.1875s` 的游戏帧恰好推进到 `2s`。通知采样每批最多扫 `1s`，因此第一点本帧送出，第二点下一帧续读。尾通知送完后，这个已结束的片段不应再要求下一帧。[插件调度](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/runtime_system.rs#L23-L61) · [公开播放器入口](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/world/component_access.rs#L258-L281) · [预算](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/framework/animation/clip_event_sampling.rs#L50-L66) · [终点帧需求](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/requests.rs#L98-L117)

## 2. 受影响的行为

公开基线的源码推演如下。两点仍各发一次，问题是第二点发出后出现连续的空帧请求。本轮没有运行 ZE、Unreal 或桌面宿主。

| 帧 | 通知与入队 | 帧后积压 / 动画帧需求 |
|---|---|---|
| 第 1 帧 `0.1875s` | `0.5→2` 只扫到 `1.5`，发 `0.75s` 的点，保留尾部。 | `1 / true`，此时还有真实工作 |
| 第 2 帧 `0.125s` | 旧范围续读，发 `1.75s` 的点。旧范围结束后又入队 `2→3`。 | 基线 `1 / true`，应为 `0 / false` |
| 第 3 帧 `0.125s` | `2→3` 钳为 `2→2`，无通知；又入队 `3→4`。 | 基线仍为 `1 / true` |

队列不增长，但每帧都有新空范围接替旧范围。没有其他唤醒来源时，动态 Runtime 仍报告 `Immediate`，Editor 将它转为 `Continuous` 并请求下一帧。[Level 帧需求](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/level_system/animation_runtime.rs#L37-L45) · [Runtime 转换](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/dynamic_api/session/state.rs#L894-L900) · [Editor 接口](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/core/gateway/session/protocol.rs#L31-L44) · [窗口调度](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/ui/retained/contract/window/redraw.rs#L47-L75)

## 3. 根因：播放器继续提交未跨越片段的范围

`scan_clip_players` 只要 `playing=true` 就提交原始播放时钟的范围，即使两端已在非循环片段末端或之后。每帧最多 drain 一次：如果帧开始时有积压，先 drain 旧范围，本帧新范围入队后不再 drain。采样器会正确把 `2→3` 钳在终点并判空；问题发生在更早的生产者。本轮不改变末姿态、`playing` 字段、真实续读或队列预算。[直接播放器](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/parameter_apply.rs#L289-L351) · [帧首 drain](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/tick.rs#L191-L204) · [提交和帧末 drain](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/tick.rs#L275-L351) · [采样范围](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/animation/clip_event.rs#L223-L263) · [队列唤醒](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/level_system/frame_state.rs#L98-L128)

## 4. Unreal 的段边界与原测试

本机 Unreal 参考树为 `6.0.0 / UE5 / CL 0`（`Engine/Build/Build.version:2–9`）。`Engine/Source/Runtime/Engine/Private/Animation/AnimInstance.cpp:2879–2943、3179–3200` 建立 Montage 实例并提供改下一段的入口。`Engine/Source/Runtime/Engine/Private/Animation/AnimMontage.cpp:1816–1875` 分开立即跳段和改 Next/Prev 边；`2280–2339、2673–2749` 先扫到本段边界，处理实际扫过的通知，再按方向取下一段。自接时报告 `bLooped`；无有效边时停在端点并退出播放推进。默认即使帧时间恰好用完也检查段末（`44–51`）。停止且淡出完才进入实例终止（`2784–2790`），结束委托另行排队（`1781–1782`）。ZE 可用自己的预算保留尾通知，不需要复制 UE 的生命周期接口。

`Engine/Plugins/Experimental/UAF/UAFTestSuites/Source/UAFCQTestSuite/Private/NotifyHandlerTests.cpp:694、1238–1252、1289–1292、1312–1319、1382–1405` 注册的 `EndToEnd_HandlerCopiesInputToOutput_TwoStates` 用一秒 `ForceLoop` 的序列及 `0.3s` 点通知。每轮十次 `RunEvent(.1)`：先令 `Input=true, Output=false`，断言输出变为 true；再令 `Input=false, Output=true`，断言输出变为 false。它验证真实通知能执行下游，不断言次数、末段停止或宿主续帧。`Engine/Plugins/Tests/EditorTests/Source/EditorTests/Private/UnrealEd/EditorAssetAutomationTests.cpp:526–531、601–614` 的 Montage 工厂片段在 `#if 0` 内，不是运行时测试。未找到相同触发的经典 Montage 原自动化，以下是生产源码派生回归，不是原测移植。

## 5. 正控与问题归属

既有 [直接 Clip 通知测试](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/tests/physics_tick/clip_events.rs#L1-L59) 使用真实播放器和 Level tick；[Clip 帧需求测试](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/tests/physics_tick.rs#L661-L674) 已断言非循环到末端不要求续帧，但没有留下真实尾通知。本问题只改直接 Clip 的范围准入。属性序列的 binding/channel 和状态机切换是 ZE 已有的其它入口；不把缺少 Montage section/slot/完成委托列为错误。

[#165](https://github.com/He-Jiahui/ZirconEngine/pull/165) 同一文件的改动在属性序列绑定，未改直接 Clip 的扫描段；[#214](https://github.com/He-Jiahui/ZirconEngine/pull/214) 与本案共享 `physics_tick` 测试入口，但测暂停淡化。[#171](https://github.com/He-Jiahui/ZirconEngine/pull/171) 是打断迁移重提，且改的是另一测试叶；[#207](https://github.com/He-Jiahui/ZirconEngine/pull/207) 是同一循环范围内的预算续读重复，不需要改其采样器。[#147](https://github.com/He-Jiahui/ZirconEngine/pull/147) 属基础姿态混合，[#247](https://github.com/He-Jiahui/ZirconEngine/pull/247) 是音量曲线末键，[#259](https://github.com/He-Jiahui/ZirconEngine/pull/259) 是音频暂停 seek。[#169 的 R32 目录](https://github.com/He-Jiahui/ZirconEngine/blob/52f529a46493900a1c27124a2e163056bcdaf15a/docs/plans/unreal-alignment/module-coverage.md#L503-L529) 已列状态淡化的通知问题，不是本案。

## 6. 最小修复判据与回归

在直接播放器提交新通知范围前，按已就绪且时长有效的非循环 Clip 的现有采样边界判断两端是否仍有有效移动。起点还在片内的最后一次穿越必须保留。继续处理所有已准入的范围，保留相同端点的姿态和原始钟，不改 ECS 类型、ABI、续页算法或窗口策略。不足以证明空范围的未就绪资源或修订变化须保留原重试约束。

用 `physics_tick` 的真实插件帧入口验证：第一帧两个通知流各收到 `0.75s`，积压为 1、需求为真；第二帧各收到 `1.75s`，积压与需求应为 `0 / false`；显式第三帧确认仍无新通知、无积压。短步等价播放、暂停后仍能读出已准入尾通知、真正的循环继续产生事件，分别作对照。仅在获准后使用 `zircon_plugin_animation_runtime --no-default-features --features native-host --test physics_tick` 的 test profile 比对同一输入的行为 RED/GREEN；本稿没有该次运行结果，也不声称已验收桌面休眠。[现有入口与相邻用例](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/tests/physics_tick.rs#L326-L329) · [原订阅读取](https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/tests/physics_tick/clip_events.rs#L499-L525)
