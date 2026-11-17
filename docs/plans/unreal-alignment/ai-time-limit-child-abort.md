# AI / 行为树独立 Draft 材料：TimeLimit 超时未撤销仍运行的子任务

> 取证说明：UE 的本机路径与文件 SHA256 是用户提供参考树的依据；本轮 D: 审计快照/工作记录仅本地保存，不随本 PR 发布，也不冒充 GitHub 可读证据。UE/ZE 测试均未在本轮执行。

状态：**仅调查，未创建 PR，未改引擎或测试，UE/ZE 测试及 Cargo 均未运行**。这是旧账 **AI-P1-020** 的源码重核，不是新发现；只讨论 TimeLimit，不夹带 Parallel、owner revoke 或 MoveTo 真正清空 optional destination 的其它问题。

## 1. 两边差异、精确符号与可达性

ZE 规范树读取点为 `cb495bb1c952a24c86f4513b8099bc4942c3a87d`；只读比较确认它与报告固定基线 `e89349ca51b4e9d024c194cd3bdf645bd25b6c37` 在 `zircon_runtime`、`zircon_runtime_interface`、`zircon_plugins`、`zircon_app` 无差异。以下 ZE 链接均固定到后者，不将本地退役流程提交夹入问题。

| 侧 / 环节 | 精确源证据与结论 |
|---|---|
| ZE 标准功能确实存在 | [decorator.rs:16–21](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/ai/runtime/src/behavior_tree/nodes/decorator.rs#L16-L21>) 注册 `time_limit`；[task.rs:10–15](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/ai/runtime/src/behavior_tree/nodes/task.rs#L10-L15>) 注册 `move_to`。不是因缺少 UE 高级节点而报错。 |
| ZE 生产注册源码（非启动回执） | [plugin.rs:120–125](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/ai/runtime/src/plugin.rs#L120-L125>) 调注册函数；[registration.rs:438–479](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/ai/runtime/src/plugin/registration.rs#L438-L479>) 将 `ai.behavior_tick` 放在 ECS `Update`，构造真实 `RuntimeBehaviorIntegrationHost` 并驱动 active agents；[tick.rs:242–261](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/ai/runtime/src/manager/tick.rs#L242-L261>) 进入执行器。需已注册并激活合法树；不声称所有示例默认启用 AI。并发新建的 [#158](<https://github.com/He-Jiahui/ZirconEngine/pull/158>) 单独跟踪首方导航描述符与 Core host admission 的前置问题；其预期拒绝也仍是源码推导。真实 Runtime host 集成见证先核该前置，不能把注册源码等同于模块已经成功启动，也不将 #158 的修复夹带到本 PR。 |
| ZE 错误退出 | [executor.rs:769–795](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/ai/runtime/src/behavior_tree/executor.rs#L769-L795>) `evaluate_time_limit` 先求值子节点；子节点仍为 Running 且到期时只将装饰器 elapsed 清零并返回 Failed。没有调用子树撤销。随后 [executor.rs:499–503](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/ai/runtime/src/behavior_tree/executor.rs#L499-L503>) 只更新当前节点 `is_active`，不会递归停子节点。 |
| ZE 既有撤销合同 | [catalog.rs:98–103](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/ai/runtime/src/behavior_tree/catalog.rs#L98-L103>) 明确活跃分支停用前调用 `BehaviorNodeRuntime::on_abort`；[abort.rs:138–200](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/ai/runtime/src/behavior_tree/executor/abort.rs#L138-L200>) 已能重置子树并向活跃 integration task / external runtime 投递撤销。TimeLimit 漏接这条已有路径。 |
| ZE 有实际副作用的子节点 | [integration.rs:34–54](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/ai/runtime/src/behavior_tree/executor/integration.rs#L34-L54>) 以 `!is_active` 判 `started`；[nodes/integration.rs:255–306](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/ai/runtime/src/behavior_tree/nodes/integration.rs#L255-L306>) 的真实 MoveTo 写 NavMeshAgent destination 并等待报告；[同文件:410–416](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/ai/runtime/src/behavior_tree/nodes/integration.rs#L410-L416>) 只有收到 `abort` 才走该撤销清理。 |
| UE 对照 | 本机 [Build.version:1–10](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Build/Build.version#L1-L10>) 为 **6.0.0 / BranchName UE5**。[BTDecorator_TimeLimit.cpp:21–24,79–86](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/BehaviorTree/Decorators/BTDecorator_TimeLimit.cpp#L21-L86>) 固定 Self abort，超时使条件失效并请求重新执行；不是只合成一个 Failed 值。 |

## 2. 有界真实影响与可证伪输入

可用现成 DTO 组成 `TimeLimit(1.0s) -> MoveTo(target=[9,0,0])`，不使用静态 `task_result` 旁路。令真实 World 的 agent 当前位置为原点，导航事件通道存在，但两次 tick 均没有 Arrived / NoPath 反馈，也没有 Blackboard observer、树替换或显式停用：

| 输入阶段 | 从现有源码推导的状态（不是运行回执） |
|---|---|
| `delta_seconds=0.4` | MoveTo 新启动、写目标 `[9,0,0]`、返回 Running；装饰器累计 0.4。 |
| 再传 `delta_seconds=0.6` | MoveTo 仍 Running；装饰器累计到 1.0 并返回 Failed；子节点 `is_active` 仍为 true，宿主 abort 未调用，destination 仍是旧目标。 |
| 后续影响边界 | 导航消费方仍可读旧目标；外部节点的 `on_abort` 也未收到。若继续同树，MoveTo 的 `started` 仍可能为 false；若放入 Selector，fallback 已可运行而原子任务仍保留活跃状态。 |

这里确证的是**终止状态与撤销副作用不一致**。实际移动距离、持续帧数、性能、脚本异步资源泄漏均未实测，不作量化。显式换树/停树仍有其它撤销路径，不能说永远无法清理。当前 MoveTo 的既有清理会把目标改为当前位置，未必写成 `None`；本问题只要求 TimeLimit 确实调用已有清理，不把那个独立契约问题算入本 Draft。

## 3. 根因、UE 实际方法与源指纹

根因是 `evaluate_time_limit` 把“子任务被超时打断”当成普通值转换，绕过既有 subtree lifecycle。UE 分开处理“装饰器条件失败”“活跃分支撤销”“新分支执行”：

- [TimeLimit 实现:42–86](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/BehaviorTree/Decorators/BTDecorator_TimeLimit.cpp#L42-L86>) 在 relevant 时安排到期，超时置 elapsed 并发 execution request；cease relevant 重置该标志。
- [BehaviorTreeComponent.h:153–167](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/AIModule/Classes/BehaviorTree/BehaviorTreeComponent.h#L153-L167>) 的旧名 `RequestExecution(decorator)` 实际转 `RequestBranchEvaluation`；[BehaviorTreeComponent.cpp:910–974](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/BehaviorTree/BehaviorTreeComponent.cpp#L910-L974>) 将 Self abort 转 Failed 续搜请求。
- [BehaviorTreeComponent.cpp:2200–2239,2522–2550](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/BehaviorTree/BehaviorTreeComponent.cpp#L2200-L2550>) 先 `AbortCurrentTask` / `WrappedAbortTask`，latent abort 未结束时不启动 pending execution。
- [BTTask_MoveTo.cpp:235–256](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/BehaviorTree/Tasks/BTTask_MoveTo.cpp#L235-L256>) 撤销具体 move request 或 ExternalCancel。本文只说明方法，不复制 UE 实现或要求 ZE 引入整个 UE latent task 系统。

以下 SHA256 是本机完整源文件字节指纹，不是 UE 提交号；UE 仅留短说明与定位：

| UE 源文件 | SHA256 |
|---|---|
| [Build.version](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Build/Build.version>) | `3DCD8B6654872D013F2B00AA5F0EF1E0DB89D4E8627CE820554675A27F228EA6` |
| [BTDecorator_TimeLimit.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/BehaviorTree/Decorators/BTDecorator_TimeLimit.cpp>) | `082A1CFBE97A81E1F745305D5DCC2CEE8BC17DD38B9F3A8EEFB0FA492F766FE0` |
| [BehaviorTreeComponent.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/BehaviorTree/BehaviorTreeComponent.cpp>) | `383FF8E54CB66FF275EAFDBC94D2C4B0AF2BBE28E3E954E2D5E8091701E38865` |
| [BehaviorTreeComponent.h](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/AIModule/Classes/BehaviorTree/BehaviorTreeComponent.h>) | `F8A5036CCD4F24DBE8340228753152D65FA4CCEBB9D8394D9F055C78F609DFCE` |
| [BTTask_MoveTo.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/BehaviorTree/Tasks/BTTask_MoveTo.cpp>) | `223C309E913F135562C234E337B227C9EF0C3A8356C813D5F878352ADB09340A` |
| [BTTest.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Developer/AITestSuite/Private/Tests/BTTest.cpp>) | `DDD404134787EB98547A0C3F8ED1B2C605ECA2FD34446C283D49ADB993574864` |
| [TestBTTask_Log.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Developer/AITestSuite/Private/BehaviorTree/TestBTTask_Log.cpp>) | `867335892E78D933F4BAA6724A2D1BA81703DCF5B862EB31F8C26EA8852F927B` |
| [AITestsCommon.h](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Developer/AITestSuite/Public/AITestsCommon.h>) | `79AE768DF316F788D17F85B51F0866F2EFA5BA3C20DB7016A190703BC92A5A20` |
| [TestBTTask_TimerBasedLatent.cpp](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Developer/AITestSuite/Private/BehaviorTree/TestBTTask_TimerBasedLatent.cpp>) | `6EF5F3336EA5BBD35650467F82BEBE5950A1DBCBF7CFF36EE8C3A3EA94960D50` |

## 4. Rust 应吸收的思想

保留 Rust 的 per-agent 实例状态、编译后稠密节点索引和借用式宿主，不搬 UE UObject/NodeMemory。吸收的是：**超时是一种受控取消，先对仍活跃的 child 完成当前接口能表达的撤销，再向父返回 Failed**。直接复用既有 `abort_subtree(tree, child, context)`，以同一 `is_active` 门限确保 inactive/已成功节点不被误撤销，且新执行不继承旧 child 的 `elapsed_seconds`、`active_child` 或 external runtime。

ZE 的 `on_abort`/宿主 `abort` 当前返回 `()`，因此这只能对齐同步撤销合同，不能声称完成 UE 的异步 latent-abort acknowledgement。引入 durable task、deadline、ack 是另一个范围，不是本缺陷成立的前提。

## 5. ECS、ABI 与调度差异

- ZE 的实到路径是 ECS `Update` 内独占 World 可变借用，再调用共享 AI manager；[tick.rs:155–173,311–341](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/ai/runtime/src/manager/tick.rs#L155-L341>) 将 agent state 取出后执行、再放回。此报告不引入 task scheduler，也不将并发 tick 的另一问题夹带入内。
- UE 的逻辑在 BehaviorTreeComponent / Controller / PathFollowing component 生命周期中协作；不能把 UE 的组件回调次序直接当作 ZE ECS 帧号。测试应对齐“旧 task 已撤销后才允许 fallback 副作用”，而非逐帧照抄 UE 日志次数。
- [AI dist 入口:9–16,24–41](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/ai/dist/src/lib.rs#L9-L41>) 只是能力协商/注册清单，AI 服务实例仍在嵌入式运行时。修复 owner 是 `zircon_plugin_ai_runtime` 的 executor；无需把 Rust trait object、World 引用或新 task 状态加进中立 ABI。

## 6. 具体 ZE 草案、既有 UE 测试迁移表与去重

**顺序仍是先本问题六点 Draft、再同范围等价回归、最后可行修复；本轮只完成调查材料。** 后续最小改法是在 `evaluate_time_limit` 的超时分支：结束短暂的 state 可变借用，保存/重置装饰器 elapsed，调用 child 的现有 `abort_subtree`，再返回 Failed；不改变普通 child 成功/失败的通路，也不修改 Parallel、revoke、通用导航 setter 或 ABI。

| UE 路径 / 行 / 测试名 | 原输入 | 原预期 | ZE 既有测试落点与迁移限制 |
|---|---|---|---|
| [BTTest.cpp:858–886](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Developer/AITestSuite/Private/Tests/BTTest.cpp#L858-L886>) `FAITest_BTTimeLimitDecorator` / `System.AI.Behavior Trees.TimeLimit decorator` | Selector：首 task 需 5 execution ticks，TimeLimit 为 2.5 × TickInterval；次 task 立即成功。TickInterval 是 [AITestsCommon.h:36](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Developer/AITestSuite/Public/AITestsCommon.h#L36>) 的 1/30s。 | 日志 `[Task1Execute, Task1Tick, Task1Tick, Task2Execute]`；首 task 未正常做完而进入 fallback。 | [behavior_tree_execution.rs:278–301](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/ai/runtime/src/tests/behavior_tree_execution.rs#L278-L301>) 已有 0.4+0.6s 结果测试，可补真实有状态 child/fallback。原 UE 将 Execute 与后续 Tick 分开，ZE 在单次 evaluate 内求值，不能宣称同样 tick 次数。单独迁移状态测试不足以检出本问题。 |
| [BTTest.cpp:888–919](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Developer/AITestSuite/Private/Tests/BTTest.cpp#L888-L919>) `FAITest_BTTimeLimitInCompositeDecorator` | 同上，装饰器组合逻辑另显式加入 Test 操作。 | 同一执行日志。 | 同一 ZE 测试 owner；ZE descriptor 装饰器嵌套可覆盖等价条件链，但没有 UE DecoratorOps 字节码的逐项移植前提，不应伪称已有相同容器。 |
| [BTTest.cpp:1454–1499](<E:/Git/ZirconEngine/dev/UnrealEngine/Engine/Source/Developer/AITestSuite/Private/Tests/BTTest.cpp#L1454-L1499>) `FAITest_BTRequestExecutionOnTimerBasedLatentAbortFinished` | 首 task 执行 6 ticks，3 ticks 时 TimeLimit 请求撤销，撤销需 2 ticks；外层 13 ticks 保险阈值。 | `[FirstTaskStartExecute, FirstTaskStartAbort, FirstTaskFinishAbort, SecondTaskExecute]`，没有正常完成日志。 | [observer_abort.rs:30–45,233](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/ai/runtime/src/tests/observer_abort.rs#L30-L233>) 已有 abort-aware external runtime，可移植“撤销先于 fallback”的同步子契约；ZE 不支持 latent abort 返回值，完整 2-tick ack 场景暂不可等价迁移，不能据缺该功能再报缺陷。 |
| **未发现原 UE 测试：源码推导契约，非迁移** | 用 TimeLimit 包住真实 World-host MoveTo，0.4+0.6s 无导航反馈，再进入无导航写入的 fallback；另用 abort-aware external runtime 检查次数与重入新状态。 | 超时调用 child 撤销恰好一次；旧目标不再保持 `[9,0,0]`；fallback 副作用前已有撤销；inactive/succeeded child 不误撤销。 | [integration_tasks.rs:100–134](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/zircon_plugins/ai/runtime/src/tests/integration_tasks.rs#L100-L134>) 已有真实宿主 `move_to_abort_clears_pending_navigation_target`，可组合 TimeLimit，不另造假 task-result fixture。搜索范围为 Runtime/AIModule 的 BehaviorTree 源与 Developer/AITestSuite 全部 C++，查询 `TimeLimit`、`MoveTo`、`AbortTask`；AITestSuite 未命中 MoveTo 测试，不声称全 UE 无此测试。 |

去重：旧 [08f 审查 P1-9:149–151](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/optimize/zircon_runtime/08f-ai-behavior-tree-blackboard-perception-runtime-review.md#L149-L151>) 同时列 TimeLimit/Parallel/revoke；旧 [AI-P1-020:249–256](<https://github.com/He-Jiahui/ZirconEngine/blob/e89349ca51b4e9d024c194cd3bdf645bd25b6c37/docs/plans/optimize/zircon_runtime/100-runtime-ai-behavior-tree-blackboard-perception-eqs-state-tree-smart-object-task-navigation-network-save-scalability-editor.md#L249-L256>) 单列 TimeLimit。本稿只重核这一个子项，旧文档不是新源码证据。所读 [15 项 PR 元数据快照](<D:/CodexData/worktrees/ze-ue-audit-20261117/reports/ue-module-audit-20261117/github-pr-snapshot.json>) 未见 AI 同题，不代表扫描了所有历史 PR。领域模块、owner、已有测试及未知覆盖另见 [AI 覆盖记录](<D:/CodexData/worktrees/ze-ue-audit-20261117/reports/ue-module-audit-20261117/research/ai-domain-coverage.md>)。**所有拟迁移/回归均未编写、未运行。**
