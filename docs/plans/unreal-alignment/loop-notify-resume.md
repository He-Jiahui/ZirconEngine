# 循环动画：恢复同一个事件采样区间时可能把同一圈的点通知再发一次

阶段：独立问题调查与等价回归方案。ZE 基准 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37`；本机 Unreal 参考树 `Build.version` 自报 6.0.0 / UE5 / CL 0，不能据此确定上游提交。以下 **80 对 96 是按固定源码和独立 IEEE binary32 运算复核的预测，不是已运行 UE 或 ZE 的测量**。此初始提交不改生产或测试。

## 1. 差异：分页改变了事件个数，而不是仅推迟交付

一个从 0 开始的正向循环 Clip，其长度 `0.02f32`，16 条不同名的普通点通知 `point00` 至 `point15` 都在局部 `0.01f32`。一帧推进 `0.1f32`，以下一帧 `0.00390625f32` 续读；第二次推进不会经过新的点。源不换、时间不倒退、World 不替换、没有状态机、跳段或重复入队。每条点被穿越五次，合计 **80** 个 occurrence；ZE 可以跨帧发成 **64+16**，不能把 80 变成 96。这也是 ZE [固定 range 的游标与默认预算契约][contract]：最多 64 条、64 KiB 文本、1 秒播放跨度。16 个同时间、不同名称的 track 是合法输入，不能靠删掉重合的点解决。

固定实现初页用 `f32` 逐圈加片长，第 64 条（第 4 圈的最后一个名字）的播放时间是 `0x3d8f5c28`。同一 range 恢复时从局部时刻重建第 4 圈，得到大 1 ULP 的 `0x3d8f5c29`。由于它在数值上“晚于”游标，用于排除同圈旧项的名字/轨道序号比较不再执行，16 条都再次成为候选；之后真正的第 5 圈仍在范围内。分开走两帧 `0.0625+0.0375`、避开 64 条边界的对照，独立模型得到 **48+32=80**。这是对离散触发数的判定，不是比较两台引擎的浮点时间是否逐位相同。

## 2. 影响：可在正常运行时到达，影响限定到流内的重复

- [动画插件注册的 PostUpdate][system] 从一个普通播放组件取旧/新时刻，[直接播放器的范围生产][producer] 不需要 transition。第一帧先入队并 drain；已有积压时，下一次 tick [先 drain 旧范围，再提交新范围][tick]。`LevelSystem` [真实恢复同一 range 与原游标][queue]，不是测例重复塞入两份范围。
- [注册的采样器][sampler] 读 resident clip；[发布者][publish] 把结果分别发给 typed `AnimationClipEvent` 和通用 `AnimationEventRecord`。应在**每条流内**验证逐名次数，不将两种合法投影相加报错。会多触发订阅者；没有实际测到声音播放、AI 最终状态或游戏动作倍增。
- Editor Play 可以经[真实 session tick][editor]驱动运行时，但没有证实动画编辑工作台直接订阅这一流。其 Montage Preview/Apply 有自己的能力边界；不将“未实现高级 Montage”作为本案缺陷。

## 3. Unreal 对照：按每个有效播放子区间收集点通知

下列位置相对于本地 UE 根，不复制 UE 源码。普通 Sequence 路线在 `Engine/Source/Runtime/Engine/Private/Animation/AnimSequenceBase.cpp:383–427,430–498` 划分连续区间，正向按 `(previous,current]` 处理点。此版有 `MaxLoopCount` 上限；对主例 `0.1f32 / 0.02f32` 转整数得 **5**，足够五次穿越。**不能**把同一结论直接套到首帧 `0.09375` 的普通 Sequence，该输入的上限只有 4。前述期望同时来自 ZE 的“预算续读不重发/不漏发”合同，不以 UE 运行结果作背书。

对一个有效相关 Slot、有效循环 Segment、没有跳段或中断的 Montage，另一路 `AnimInstance.cpp:2755–2780` → `AnimMontage.cpp:3237–3277,2818–2885` → `AnimCompositeBase.cpp:74–151,626–634` 将局部循环区间分别送给点通知扫描。`AnimNotifyQueue.cpp:53–87,138–147` 区分普通点与 NotifyState；这里限定触发概率 1、无筛选、有效权重，不把 NotifyState、同步标记、静默 scrub、section jump、反向或 branching point 的政策移植到 ZE。

参考来源 SHA-256：`AnimSequenceBase.cpp` `538cedbee8cad1313c301078374a2750c2929e5ec198a54d2254d0c79ae6c3f7`；`AnimCompositeBase.cpp` `d44f671236f97955756c329411102cbe8a84953beb7346314165db71002e5f32`；`AnimMontage.cpp` `1dbd8796d7bbde0f0e51d00c8cfcbd758b1ea2b810119768bc4ee8b9e5c8bb10`；`AnimInstance.cpp` `043b137be446571454112ca2d7717743782c892c0a720cbe0b6c2c997514d897`；`AnimNotifyQueue.cpp` `082686bc5b5ec128c919a44b92e9dfeb0de6291e8775c9ab1c9c49714124ea8e`。

## 4. Rust 适配：统一同一圈的身份，不用 epsilon 掩盖

ZE [页内推进和游标存储][increment] 与 [下一页候选重建、比较][seed] 对同一 occurrence 采用不同生成式。候选修复是让私有堆项的初始与后继圈使用同一稳定的圈序号和时间生成规则，并验证反推圈序号在舍入边沿的相邻项。不能仅把比较换成宽松 epsilon，或对 `(event_name,time)` 全局去重：对时间很近的合法点、同时间不同轨、下一圈同名点，前者会漏发。修前要用真实采样和队列验证反例；修后保现有半开端点、同刻 tie、倒放、超额 payload、有限性与有界堆复杂度。

## 5. 接口、ECS 与去重

UE 传 instance/slot 与 `FAnimNotifyEventReference`；ZE 的 [clip 资产点 DTO][dto]、range 方向、World replacement epoch、两条 ECS 事件流是 ZE 自己的边界。先在 sampler 私有实现寻找解，不先改序列化资产、公共游标结构、C ABI、组件或全局调度。观察点仅是累计次数；不要求两台引擎相同帧派发、相同 tie 排序或相同浮点位型。

[#171](https://github.com/He-Jiahui/ZirconEngine/pull/171) 是打断迁移的生产者提交**两个**相同窗口；本例只入队**一个** range，故 producer 去重不能修复它。[#147](https://github.com/He-Jiahui/ZirconEngine/pull/147) 是姿态混合，[#165](https://github.com/He-Jiahui/ZirconEngine/pull/165) 是暂停的属性序列绑定。均不重复。既有 [动画事件主链评审][prior] 的广义稳定身份/积压路线也不能代替这一特定预算边界回归。

## 6. 原有测试依据与落实顺序

- 未找到经典 Montage/Sequence 专门覆盖“浮点非整周期、重复多圈、64 条分页”的原测。UE 实有 `FUAFNotifyHandlerPipelineTests::Filter_Debounce_TapDance_SameSequenceAllFire`（`Engine/Plugins/Experimental/UAF/UAFTestSuites/Source/UAFCQTestSuite/Private/NotifyHandlerTests.cpp:3624–3671,3724–3728`，SHA-256 `8b4aaf5f3c7140f96b6bd0159fa0e850f7b53e9600dac9fc7dbceb6da79d6d38`），同类的四个合法点应全部触发；它是**另一子系统的计数对照**，不是本例的等价测试。`EditorAssetAutomationTests.cpp:523–533,605–613` 的 Montage factory 位于禁用的创建/复制分支，不作播放证据。
- ZE [同 owner 的既有测试][leaf] 分别测了精确可表示的循环间隔，以及同刻非循环分页；[插件测试][plugin_test] 覆盖真正的直接播放器与负速跨边界，但没有在非精确循环间隔撞上数量预算。
- 先在真实 `ProjectAnimationClipEventSampler` 和 `LevelSystem` 队列上运行同一输入。分别记录大预算单批的 80 个事件、默认预算第一次 64 条及其真实 continuation、第二次 drain 的数目和每个名的次数；第三次无新增 crossing 不得再发。加短区间、可精确表示的循环长度及相邻合法点控制。再用实际插件的两个 tick 和 World 订阅验证 typed/neutral；该门要求其原有 fixture 环境，不能用源文件 grep 或另一单测代替。**先以原版的行为失败决定修复是否成立，再在同一 PR 修并重复同一组输入。**

[contract]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/framework/animation/clip_event_sampling.rs#L28-L92
[system]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/runtime_system.rs#L25-L85
[producer]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/parameter_apply.rs#L289-L351
[tick]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/tick.rs#L192-L204
[queue]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/scene/level_system/animation_runtime.rs#L153-L235
[sampler]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/animation/clip_event.rs#L11-L40
[publish]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/src/evaluation/pipeline/events.rs#L93-L118
[editor]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_editor/src/core/gateway/session/frame.rs#L48-L60
[increment]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/animation/clip_event.rs#L184-L235
[seed]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/animation/clip_event.rs#L265-L397
[dto]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/core/framework/animation/asset/clip.rs#L27-L46
[prior]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md#L186-L190
[leaf]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_runtime/src/animation/tests/clip_event.rs#L16-L56
[plugin_test]: https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/animation/runtime/tests/physics_tick/clip_events.rs#L2-L138

## 7. 原生 Runtime：默认 64 条预算把 80 个通知变成了 96 个（2026-11-18）

[原版生产 + 仅测试提交 `ff910246d8418d0bd4d22a747864385576b3fbda`](https://github.com/He-Jiahui/ZirconEngine/blob/ff910246d8418d0bd4d22a747864385576b3fbda/zircon_runtime/src/animation/tests/clip_event.rs)。Windows x64/MSVC、Rust 1.94.1，`test -p zircon_runtime --no-default-features --features animation,native-host --lib loop_notify_resume_ --locked --offline --jobs 1 -- --test-threads 1 --show-output`。实际执行 **3 案，1 过、2 败、0 忽略、7,871 过滤，exit 101**。两处都是数量/逐名次数断言，不是夹具、类型或链接失败。

| 原版观察 | 实际输出 | 应有输出 |
|---|---|---|
| 同一 clip、预算 128，一次采样 | 80 个点、每个名字 5 次 | 80 |
| 默认预算 64，首帧终点 0.1，复用第一次返回的原游标 | 首批 64；末点 `0x3d8f5c28`；续页首个相同 occurrence 被算成 `0x3d8f5c29`，续出 32，共 **96** | 64 + 16 |
| 独立真实 `LevelSystem`，一次 admit，使用 resident clip 的 `ProjectAnimationClipEventSampler` | 三次 drain **64 / 32 / 0**，每个名字 6 次；第一次后 backlog=1，第二次后为0 | 64 / 16 / 0，每名 5 次 |
| 对照 | 同总跨度但分两帧，48+32；精确二进制片长 1/32，同样预算 64+16。两组每名均 5 次 | 各 80 |

`.09375` 的 ZE 采样器变体也实际打印了同样的重复，仍按 §3 的不同 UE 循环上限解释，不能拿它宣称 UE 对同输入一定发 80。这里原生验证了 ZE 的 sampler / queue，并未把模型当作引擎证明；也没有启动 UE、Editor 游戏世界、插件 `PostUpdate`、AI 或声音消费者。现存插件整链夹具要求特定 smoke 和临时目录布局；本次不替换或放松这些检查。下一步先证明修复能同时保住同刻不同轨、极近但合法的两个点、计数/字节分页、正反向与大播放时间的有限进度，再改 sampler；没有动公共游标或播放资产。

22 项选定来源和清单，原生回执 `f42fdf4c9a2a0f45b32a119139eda58f76fe8c5a964b30165aa156aaef94a394`；stdout `555cae1944af4ef75dc1db49cee5ebed80a553108e42f273cfff15b3a5158a4c`（2,673 B），stderr `34b538aca3094cf489426d31ac2772fc0895c4cd34627b9177ea89e4a10f57e5`（283,762 B）。
