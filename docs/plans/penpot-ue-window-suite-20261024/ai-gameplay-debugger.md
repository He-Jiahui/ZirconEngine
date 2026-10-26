# Gameplay Debugger：OVW-42 / AI-W25

Owner `01a16d53-0693-72e2-bd58-d34a65b938fa`，2026-10-24。接续 [Parent 窗口归属](parent-window-coverage.md)、[AI 窗口盘点](ai-windows.md) 和 [设计合同](ai-design-and-acceptance.md)。OVW-42 对应已有 AI-W25，不增加第二个 Gameplay Debugger 或诊断宿主。本文是实际源码核查、设计与接入合同；没有新增产品入口、正常 EXE 图或通过案例。

## 归属与 UE 依据

- AI 负责真实 agent/gameplay 的只读数据投影、黑板/感知/行为数据解释，以及本域视图与资产跳转请求。
- Parent 负责共享 PIE/diagnostic 宿主、输入路由、生命周期、trace、源身份关联及公共组件。Actor 只提供同一真实 Actor 的关联定位。
- OVW-03/04 的 Navigation 作者、可视化切换、设置与 bake 窗口仍由 Parent 待接续；AI 只消费已证明身份的导航结果。不创建 Nav tab 代替 EQS/StateTree，不修改公共 runtime。

| 实际 UE 来源 | 已核对的布局/行为 | Zircon 接续边界 |
| --- | --- | --- |
| [GameplayDebuggerToolkit.cpp](../../../dev/UnrealEngine/Engine/Source/Editor/GameplayDebugger/Private/GameplayDebuggerToolkit.cpp) / Init、OnClickedDisableTool | Mode toolkit 的 GroupBorder、纵向状态文本、屏幕消息警告和 Disable tool；关闭实际清除 viewport DebugAI show flag | 接入 Parent 的真实工具模式/视口开关，不另造资产编辑 tab；关闭操作须清除覆盖层并返回原焦点 |
| [GameplayDebuggerEdMode.cpp](../../../dev/UnrealEngine/Engine/Source/Editor/GameplayDebugger/Private/GameplayDebuggerEdMode.cpp) | 工具模式持有 toolkit；viewport/show flag 不满足时退出模式 | 窗口、PIE 停止、世界销毁与工具关闭须有同一真实 owner；AI 不接管共享生命周期 |
| [GameplayDebuggerConfig.cpp](../../../dev/UnrealEngine/Engine/Source/Runtime/GameplayDebugger/Private/GameplayDebuggerConfig.cpp) / constructor | 默认 Apostrophe 激活、Numpad 0–9 类别、Add/Subtract 切类别行；边距来自配置 | 用 Parent 命令/焦点路由处理；文本/IME 输入时不拦截，用户重绑和无数字键盘路径需真实菜单合同 |
| [GameplayDebuggerLocalController.cpp](../../../dev/UnrealEngine/Engine/Source/Runtime/GameplayDebugger/Private/GameplayDebuggerLocalController.cpp) | Actor 选择、类别/行输入、viewport/HUD 与选择变更订阅 | 类别和选中 Actor 取真实状态；Actor 标签不能充当跨 world 的实体身份 |
| [GameplayDebuggerCategory_AI.cpp](../../../dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/GameplayDebugger/GameplayDebuggerCategory_AI.cpp) / CollectData、CollectPathData、DrawData | 真实 Pawn/Controller、移动/Brain/GameplayTask、Montage/NavData、路径和头顶信息 | 逐字段核对 provider；现有 AI frame 没有 controller、移动模式、montage 或 GameplayTask 队列，不能拿树名/事件序号补值 |
| [GameplayDebuggerCategory_BehaviorTree.cpp](../../../dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/GameplayDebugger/GameplayDebuggerCategory_BehaviorTree.cpp) | Brain/树说明及带值黑板；Shift+Subtract/Add 翻页且首尾有界；UE 实现把黑板放在第二列 | 复用真实 typed 黑板值与共享滚动；列关系可流式折叠，不把 UE 的 600 px 横坐标当跨尺寸布局规则 |
| [GameplayDebuggerCategory_EQS.cpp](../../../dev/UnrealEngine/Engine/Source/Runtime/AIModule/Private/GameplayDebugger/GameplayDebuggerCategory_EQS.cpp) | EQS category 依赖实际查询实例/结果 | EQS 后端仍待定；无 query provider 时显示真实能力缺口，不显示模拟得分/点云 |

## 可消费的数据与身份缺口

| 来源 | 真实字段或保护 | 不具有的证明 |
| --- | --- | --- |
| `AiBehaviorDebugFrame` / `AiAgentTickReport` | world、entity、真实决策 status、active_node、diagnostic、behavior_tree 字符串、typed 黑板、感知/空间采样 | 无源 URI/hash、文档 revision、runtime 注册代际、原始 tick 编号/时间、完整调用栈/历史/断点 |
| `AiPieMirror` | Parent consumer 的 begin/consume/end；错 session、非递增投递和快照内跨 world 拒绝；只替换快照所属 world | 原 `sequence()` 是整个镜像最后一次接受的投递，不等于所有保留 agent 帧的序号；不能把它显示成 runtime tick |
| `AiBtNodeResultMirror` | world/entity/node 的实际事件；快照按 active_node 裁剪；node-result 与 snapshot 各有序号 | 没有与 AI frame 共用的 source/revision/frame ID；不能拼成完整执行路径或历史，更不能从父子拓扑推测其它节点 Running/Succeeded |
| `AiPerceptionOverlayController` | 真实 frame 的位置、forward、FOV/range、stimulus；关闭发送 None 清图元 | 未证明正常 EXE 中的 viewport owner、资源闭包和选中 agent/source revision 绑定；不能拿静态形状当实时感知 |
| `NavigationPieFrame` / `NavigationOverlayFrame` | session、sequence、owner_generation、网格图元及真实 `NavAgentTickReport.debug_agents`；agent 有 position/destination/velocity/path/status | payload 本身无 WorldHandle；须由 Parent 证明 gateway/world 来源；nav owner_generation 不是 AI 源版本。debug_agents 仅在 NavigationDebugCapture 启用时填充，空列表不证明没有导航 agent |
| Actor 定位接口 | 待 Actor/Parent 提供同 session/world/entity 对应的真实 Actor/文档定位回执 | entity u64、显示名或同名树都不能自行映射为 Actor/source URI |

相关源码为 [AI snapshot](../../../zircon_runtime/src/core/framework/ai/snapshot.rs)、[AI tick](../../../zircon_runtime/src/core/framework/ai/tick.rs)、[AI mirror](../../../zircon_plugins/ai/editor/src/runtime_mirror.rs)、[AI overlay](../../../zircon_plugins/ai/editor/src/overlay.rs)、[Navigation mirror](../../../zircon_plugins/navigation/editor/src/runtime_mirror.rs)、[Navigation agent debug](../../../zircon_runtime/src/core/framework/navigation/agent.rs) 和 [Navigation overlay frame](../../../zircon_plugins/navigation/runtime/src/overlay_frame.rs)。本域没有修改这些公共 runtime/Navigation 文件。

## 本域已落地的投递来源修订

AI editor 的 `AiPieMirror` 现在随每个已接受 agent payload 保留其真实 `delivery_sequence`，不另建历史、缓存、事件消费者或第二个会话。`agent_frame_in_world(world, entity)` 返回借用的 `AiPieFrameRef`，在同一次读取中取得实际 session、该 payload 的投递序号和原 frame；旧 `agent/agent_in_world/agents/agents_in_world/sequence` 保持原调用合同。借用查询不克隆 frame/黑板或分配 Vec；内部仅在原 agent map 的每条记录增存一个 u64。

例如先收到 world 7/seq 1、再收到 world 8/seq 2，之后 world 7/seq 3：world 8 的保留帧仍标 seq 2，不能被镜像全局 seq 3 重新标记。空 world 快照清掉该 world 的帧；拒绝错 session/stale/跨 world payload 时不改变原帧及其来源；停止和新会话清除旧数据。

四项回归源在 [gameplay_debugger.rs](../../../zircon_plugins/ai/editor/src/tests/gameplay_debugger.rs)，覆盖上述跨 world、拒绝保持、空快照和会话重启。它们仅是未执行的 Rust 测试源；真实托管执行仍受 AI recipe/锁文件/准入依赖约束。新 API 不构成源资产关联证明，也不允许执行高亮。历史 `gameplay-debugger-01-*` 保留原 14+4 及十源版本；后续 BT 图候选新增六项，当前十二源/20+4 见 `bt-graph-adapter-01-*`，全部未运行，原 `ueai24-bt01` 十一项输入不变。

历史安装 `1e1b939f…`、`cdd14500…` 曾拒 AI 四 filters，原始结果保留。恢复后最新 `9c684aef…` 的 11 固定文件匹配，四项 pinned parser/实际 server recipe 构造均成功，包含 `gameplay_debugger_tests::`。沿原 bt01/validate、wt01/create 的实际 QueryOnly 均为 session authorization pending/exit 2；没有运行四项 Gameplay 回归或其它 Cargo。范围仍限定独立插件 manifest、单 AI 包/defaults off/features=[]、locked/lib/test profile/jobs1/thread1/E 根，完整传递 host features 不移除；原 Jenkins owner 对账收件和自动上下文，不能因服务恢复启用 trace/step 或执行高亮。

## 接入合同与状态

| 合同 | 本域责任及实际动作 | 依赖 / 当前状态 |
| --- | --- | --- |
| AI-GD01 会话与 agent | 只从 Parent 已开始的真实 consumer 读取；显式 world/entity 选中；按 payload 的真实投递来源展示；停止清空 | 新镜像来源 API 已落地，未编译/未接 UI；返回 None 尚不能区分未收 world 快照与已收空快照，需真实宿主 snapshot receipt；事件执行/正常窗口仍 open |
| AI-GD02 源与 revision | Parent 提供实际 frame→runtime 树注册代际→资产 URI/source hash/文档 revision 的对应回执；AI 核对相同树和节点 | 现有 DTO 未提供，open；未知身份进入 source-unresolved，不作节点执行高亮/当前源跳转 |
| AI-GD03 字段 | 只投影 frame 原有 status、黑板 typed 值、perception/diagnostic；缺 controller/movement/task/montage provider 逐项标缺 | 不合成完整 gameplay 面板数据；fixture 只能标测试用途，不作产品证据 |
| AI-GD04 Actor 定位 | 请求和回执带原 session/world/entity；失败保持原选择；返回恢复原视图/焦点 | Actor 仅提供实际关联；公共导航入口/焦点由 Parent 接入，open |
| AI-GD05 导航结果 | 先核对来源 world、session、owner_generation 与所选 entity，再展示真实 path/status；失配清掉原覆盖层 | Parent 提供缺失的 world 关联与消费能力，open；无 Navigation 编辑/bake 写入口 |
| AI-GD06 输入与模式 | 类别、选择、滚动首尾、关闭/恢复焦点；快捷键尊重文本/IME；显示开关只控制真实已实现覆盖层 | Parent 共享 mode/commands/viewport 合同，open；不新增假的 Pause/Step/Breakpoint 命令 |
| AI-GD07 设计与资源 | 在已分配 AI file/page 接续独立业务设计；复用 Parent library/styles、共享 Tree/Property/菜单及动画播放控件 | 当前 revn 19 保留原 13 业务板，另有公共实例验证板，仍无 OVW-42 专用设计；没有为本项编造 UUID、官方导出或源 ZUI 映射；open |
| AI-GD08 验证 | 同一真实 agent/frame/source 的 1280×800、900×620、640×520、1280×800@144 DPI；同状态双端图/差异/逐图复核 | 无桌面时段/正常 EXE/真实源绑定；passed=0、accepted=0 |

状态必须来自实际条件：no-session、awaiting-frame、no-agent、source-unresolved、source-mismatch、frame-resolved；这些是待产品接入的状态合同，不冒充已注册枚举。trace/stack/断点未交付时均保持 open。即使将来得到 source-resolved 的当前帧，也只允许展示该帧直接报告的节点结果，不从 active_node 补出祖先/兄弟路径、搜索历史或 Service 状态。

## 布局与验收配方

Gameplay Debugger 是共享视口中的工具模式与只读类别面板。AI 拥有 agent/黑板/感知业务区；Parent 拥有 mode chrome、PIE 控件、viewport/GPU 图像、停靠、焦点和 diagnostic/trace。顶部身份区显示真实 session/world/entity/投递序号及源关联状态；类别区域承载 AI/BT/感知与已准入的导航结果；内容依赖真实 provider，空状态保留实际缺口。复用现有 tokens/Starship，不另建主题。

1280 配置保留 UE 的说明与黑板列关系；900 允许身份行换行；640 通过共享滚动/折叠与 overflow 访问完整操作，不能删掉 debug selector 或以裁切标签代替可达性。中文/长 Actor、树名、键值须有真实换行/省略和可访问完整文本；不复制像素坐标。完成共享组件交接前，具体 min/max、折叠断点和组件 UUID 仍待实际设计/测量。

必须验证跨 world 同 entity、世界 A/B 交错快照、空快照移除、错会话/旧序号/跨 world payload 拒绝、停止再开始、Actor 被销毁/重建、源版本失配、未知 subtree、导航无来源/旧 owner generation、无 trace、长文本/IME、滚动首尾、关闭与焦点返回。LayoutReviewCase 沿原生协议登记真实 source/state/frame/resource/EXE 指纹；没有数据和有效捕获时不填写 passed，未做双端逐图比较不填写 accepted。
