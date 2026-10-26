# Widget 动画消费者接入合同 r01

本域 owner 为 `01a16d52-d54d-7ac0-b0d5-21864f383a1e`。本文接续 [UI-SH-TIME/CURVE/PLAY-01](widget-ui-design.md#共享-owner-的具体消费合同) 和动画 owner 的 [Widget 消费合同](animation-shared-controls.md#widget-消费合同与-w10)，形成 W09/W10 的具体输入与剩余工作。轨道/播放语义仍为原合同；[visible域内候选](widget-ui-preview-visibility.md)现沿原Session/Host消费Parent实际公开的Bool/Step采样器，并补受检的保值finish。17项测试尚未编译/运行，没有实际clock/Stop/GUI/present接线。本域未新增合同、clock、动画schema、播放器、共享Runtime改动或主件，未关闭UI-SH-WG01～07。

## 真实共享身份与源边界

2026-10-24T12:32:28Z，本聊天独立认证 API 只读取得动画文件 revn12/vern0、533 objects、3 native components；用户交接的 revn5 是较早观测。该时点main/copy身份与原[只读回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-animation-shared-readback-r01.json)保留。后续动画owner已保存关闭fresh重开到rev105/1358objects，原三个component/main UUID不变；下表Transport草稿按该真实owner回执更新为2，不把本次读取回执称为新的独立API核验或发布版本。

动画 [file/page](http://127.0.0.1:19001/#/workspace?team-id=822661f1-7ba3-81f7-8008-c6dcb4ae6a8f&file-id=b4a1e3ce-9e6a-81fe-8008-d69d60a13f99&page-id=b4a1e3ce-9e6a-81fe-8008-d69d60a13f9a)：`b4a1e3ce-9e6a-81fe-8008-d69d60a13f99` / `b4a1e3ce-9e6a-81fe-8008-d69d60a13f9a`。1280×800 owner 空态 board 为 `22192a1e-86c1-43c9-8ecf-994d44c4ee5c`，不是 Widget 消费实例。

| 合同 / owner 最新草稿版本 | component UUID | main UUID | owner 原1280空态 copy UUID |
| --- | --- | --- | --- |
| UI-SH-TIME-01 / 2 | `8cf70bfd-9275-4bac-849b-b79148b8a4fd` | `4db0829c-abc4-482d-b7b3-155a5d8d75c2` | `e0fe35ac-c072-4bd4-918d-eabce59da737` |
| UI-SH-CURVE-01 / 1 | `78492c86-3bb4-47dd-97c6-752d6ff37e3d` | `599963dc-b91b-4740-926b-d38154aa8f3a` | `8347b506-6bae-4869-8020-18f778c756e0` |
| UI-SH-PLAY-01 / 2 | `982ef940-2239-4167-a24a-56c80ec35fbf` | `44eb5ccd-8ac2-44f2-a4ad-d0d618daa096` | `2effdc5e-0b82-4413-bbbd-229016158565` |

Timeline 外组合包括 header/outliner/data surface，目前没有正式 ZUI wrapper。只有内层 `WorkbenchTimelineStripRoot` 对应 [timeline_strip.zui](../../../zircon_editor/assets/ui/editor/workbench/primitives/data/timeline_strip.zui)：main shape `02ce83da-13eb-4d16-b847-055ac73c9450`、copy `9ab062ef-8151-4006-ae98-5f83639e4caa`。该原资产只有一个 Canvas，输入三项均 false，不能以外组合导出成功或 scrub 已接通记账。Curve ZUI 表面仍缺。Transport 保留 [原六事件和 route](animation-shared-controls.md#保留现有六个-transport-身份)；Previous/Next 不暗改为单帧，不覆盖全局动画 handler。

同次Widget revn7/1434及公共revn0空库是12:32Z历史观测。后续本域已在原页保存并关闭重开到rev12/2978对象/8本地主件，实际连接Shared rev1的四个基础主件并有12个linked copies，见[原证据](widget-ui-penpot-evidence.md)。它们是Button/SearchInput/TreeRow/PropertyEditorRow，不是上述三个时间控件的Widget消费。Parent于16:19Z读回动画rev80/原3main仍isShared=false；owner r04的14:46Z动画rev31、Widget rev9/libraries=[]分别保留历史，不覆盖后续证据。本域未复制时间main或将owner copy当本域实例。时间外组合wrapper、Curve正式ZUI、原六事件/五data props/component_variant的无损导出、发布/链接及样式传播继续open。

## 目标定位、属性与稳定身份

[v2 arena](../../../zircon_runtime_interface/src/ui/v2/arena.rs) 已有 `UiTemplateNodeInstancePathStep { source_path, source_node_id }`，区分 authored invocation 与编译生成的 handle；先复用该身份，不把场景 `EntityPath` 引入 Widget。

| 输入 | 本域接入约束 | 当前缺口 |
| --- | --- | --- |
| animation / track / key | 所属 `.zui` 资产、component/view、稳定动画/Track/Key/Curve ID、source revision、preview session generation；移动、排序和 undo/redo 后仍定位同一 key | [UiV2AssetDocument](../../../zircon_runtime_interface/src/ui/v2/asset.rs) 没有专门 animation collection；持久身份及保存/导出表示未确认。不得使用 time bits、行号或每次投影重建 UUID |
| target | 源 asset/node、公开属性空间/路径、component invocation ancestry；control ID 仅辅助定位。repeat 实例还需实际稳定 item key 和失效规则 | 当前运行节点唯一匹配及 repeat locator 尚无动画 adapter；Surface node/index/handle 只用于一次当前解析，不写持久 binding |
| property | 区分 props/state/style/layout/slot/公开组件参数；保留 override/reset、原属性缺失与 token/样式来源 | 任意 metadata 字符串可写不代表目标可动画；由真实控件描述和有效写入通路给出逐属性 capability |
| selection | Designer/Hierarchy/Details/轨道/曲线共享同一源目标；轨道与曲线使用 typed key/tangent identity | source/instance 重建后重解析；目标删除或歧义时显示诊断并拒绝编辑，不回退第一个重名控件 |

[UiValue/UiValueKind](../../../zircon_runtime_interface/src/ui/component/value.rs) 是实际值体系；[CurveView](../../../zircon_editor/src/ui/curve/model.rs) 的坐标和 tangent 是 f32，插值为每条 curve 的 Step/Linear/Hermite。下面是 adapter 的类型设计，不是已可编辑轨道列表。

| 值类别 | 显示及编辑合同 |
| --- | --- |
| Float / Vec2/3/4 有限标量分量 | 校验实际属性类型、范围、单位及 f64→f32 显示损失；文档值保持原精度。完整组合值经域校验一次提交；分量映射稳定，不能逐通道形成多次 undo |
| Color / margin / transform | Color 在 UiValue 中是字符串，margin 也不是独立 UiValue 类型；先确认控件解析、色彩空间、单位和分量组合，不能按名称拆字符串或默认把 Vec4 当 Quaternion |
| Bool / Enum / Flags / Int / String / AssetRef / InstanceRef | 维持真实离散值和域定义的 step/lane；整数不经 f32 往返丢失精度，flags 不连续插值，引用不按字符串混合；未确认可动画性时只显示诊断 |
| Quaternion / 其它组合 / 缺失 / Null | 未取得实际属性与插值支持时保留 lane 和拒绝原因；Quaternion 不拆成四条独立 Hermite。原缺失与显式 Null 不合并，恢复时保留基线语义 |
| event key | 单独的事件轨道与稳定 binding/action/route/签名；没有数值 curve。编辑事件 key 与执行事件分别记回执，不将已有输入事件摘要当动画事件执行器 |

## 真实属性应用与一次文档历史

源码核查收窄 UI-SH-WG03，以下均是可复用入口的事实，未取得动画消费者运行证明：

| 实际入口 | 可复用范围与限制 |
| --- | --- |
| [UiSurface::mutate_property](../../../zircon_runtime/src/ui/surface/surface/property_transaction.rs) / [UiPropertyMutationRequest/Report](../../../zircon_runtime/src/ui/surface/property_mutation.rs) | 实际树/metadata、组件状态、runtime style base、dirty、绑定回执及焦点协调；有 Accepted/Unchanged/Rejected。它处理单个属性，通用 metadata fallback 不提供完整动画类型校验；弹层后代关闭先发生，后续拒绝不整体回滚 |
| [UiAssetPreviewHost](../../../zircon_editor/src/ui/asset_editor/preview/preview_host.rs) | production 保留只读 `surface()`，`surface_mut()` 仍仅 cfg(test)。已新增本域 visible-only 候选，复用生产事务并检查 current source、Host lease、frame、incarnation、sequence 与实际 report；无 schema/player/时钟，尚未运行，不代表动画采样能力通过 |
| [公开Bool/Step采样器](../../../zircon_runtime/src/animation/sequence/channel_sample.rs) | Parent已落实纯查询出口，原私有trait不开放。Widget的sample候选委托该出口再走原visible事务，保留请求秒值与实际结果；不提供clock、持久track、Stop或present回执 |
| [invalidation transaction](../../../zircon_runtime/src/ui/surface/surface.rs) | 验证代际与节点后提交 dirty；不写属性值，不等于跨属性原子事务。随后仍需真实 rebuild/publication/render 证据 |
| [event manager SetProperty](../../../zircon_runtime/src/ui/event_ui/manager/reflection_store.rs) / [request dispatcher](../../../zircon_runtime/src/ui/event_ui/manager/invocation.rs) | 只改反射快照并广播 diff；不按声明类型验证、不写真实 UI。成功 Ack 不能作为采样或画面生效证明 |
| [metadata batch](../../../zircon_runtime/src/ui/surface/property_mutation/metadata_batch.rs) | crate-private、同一节点 metadata 的内部更新基础；不等于含焦点/弹层/组件行为的跨目标动画 batch，不能直接开放后称原子支持 |

一次采样先校验当前 source/session、全部 locator、类型/值、逐属性能力和恢复条件，再允许实际写入。域需明确批量失败原子性；顺序调用 mutate_property 然后逆序“改回值”未证明能恢复焦点/弹层等副作用，不得这样宣称原子完成。首版允许集必须由已核实的无额外行为属性确定，其余保留拒绝原因。基线记录实际来源、原值/缺失、revision 和实例代际；不能把采样后的 runtime-style base 当作原始基线。

持久轨道编辑沿 [session command_entry](../../../zircon_editor/src/ui/asset_editor/session/command_entry.rs) 的现有文档命令与 `undo_replay/redo_replay`，复用 [source generation](../../../zircon_editor/src/ui/asset_editor/session/lifecycle/source_outline_cache.rs)。schema 尚缺，当前不能构造合法轨道 delta；后续只增加本域适配，不开另一历史栈。gesture begin 保存起点，update 只预览，commit 一次合法源变更，cancel/无效果不 dirty。Details 加 key、拖动和数值 Enter 走同一事务；IME 中不提交，Escape/capture lost/切源取消。undo/redo 后按当前有效时间重采样，源历史与瞬时属性应用分别记录。

## 播放、事件与关闭的待决行为

UE 实际依据为 [WidgetBlueprintEditor.cpp](../../../dev/UnrealEngine/Engine/Source/Editor/UMGEditor/Private/WidgetBlueprintEditor.cpp)：drawer opened 设置 Sequencer 焦点，dismissed 禁用对应 surface/auto-change 并返回编辑器焦点，独立 tab closed 切 null animation；`ChangeViewedAnimation` 重置各 Sequencer root，空动画禁用交互并关闭曲线窗。移除绑定还显式 restore pre-animated state。不能由这些分支推断一个通用 Stop。

运行时 [WidgetAnimationState.cpp](../../../dev/UnrealEngine/Engine/Source/Runtime/UMG/Private/Animation/WidgetAnimationState.cpp) 的 Stop 独立设置 Stopped、时间 offset 归零并排求值，停止完成按 `bRestoreState` 决定恢复；[UMGSequencePlayer](../../../dev/UnrealEngine/Engine/Source/Runtime/UMG/Private/Animation/UMGSequencePlayer.cpp) 单独委托 Play/Pause/Stop/SetCurrentTime。它们是参考行为，不能声称 Zircon 已有相同后端。

| 操作 | 本域具体设计输入 / 尚待域与 Parent 确认 |
| --- | --- |
| scrub | 一个真实采样时钟；requested 与 confirmed time 分开，单位/range/FPS 来自真实动画。提案为拖动时暂停、commit 保持确认值、cancel 恢复起点；原 playing 状态是否恢复需明确。无事件策略时只允许已证实的纯属性采样，不执行 event key |
| event playback / loop / reverse | 需明确 forward/reverse、seek、重复同时间、loop 越界、cancel 的事件集合/顺序和真实宿主回执；没有 VM/route consumer 与策略时 event execution capability 不可用。取消不能撤销已经发生的外部副作用 |
| Pause / Stop / Reset | Pause 保持确认时间和值；Stop 的时间位置、采样前值恢复方式以及完成回执须独立确认；Reset 是否重建 Surface、如何重解析 selection 也须独立确认。不将重新载入源偷偷当每次 Stop |
| drawer hide / tab close / document close | 分别确认播放是否继续、gesture 取消、动画选择与瞬时值恢复。具体 close 若要求恢复/保值，须在 hide/源替换布局发布及 deactivate 退休 lease 前消费原有效 Cancel 完整 Outcome 或 finish frame，随后退休 request/session；普通 hide 只退休输入。焦点返回当前有效 opener，失效回 Designer；旧源/lease 结果拒绝。关闭消费者也需结束实际作业/释放资源，不能只隐藏 UI |

上述行为是待确认的产品合同；不创建静态按钮或时间标签模拟实现。公用 transport 只发有能力的意图；Widget 负责真实值、事件、采样执行和所属源历史。Editor Utility Widget 仍受 Actor/Parent 的 editor host/VM 合同约束，不能借 runtime animation 宣称 Utility 后端完成。

## 分工、接入顺序与验收

| owner / 顺序 | 需要交付的具体内容 | 对应 open gap |
| --- | --- | --- |
| Widget + Parent，先定真实表示 | 复用现有 v2 的轨道/绑定保存与导出承载决策、稳定身份、属性允许集和源 locator；不先塞入未定义字段或 sidecar | WG01 |
| Widget/runtime，接真实实例 | 受限 preview-host 入口、复用 mutate_property 的类型校验与失败恢复、唯一时钟/采样器、基线和逐操作能力；关联 source/session/request/applied time/Surface/frame | WG02/03/04 |
| 动画 owner + Parent，接共享表面 | 复用上列三个 main；正式 Timeline 外组合 wrapper、Curve ZUI、公开 slot/参数、六路由兼容及新增命令、公共发布/样式与导出来源 | WG06 |
| Widget，接域编辑 | UI 属性/event track、Details 加 key、同一 undo/redo、选择与 revision 失效；共享表面不持文档副本 | WG05 |
| Parent + Widget，排队宿主验证 | drawer 焦点/关闭返回，正常 EXE 三尺寸和实际150% DPI、真实资源 ready、捕获关联、Penpot/产品并排差异和逐图复核 | WG07 |

最小真实消费用例依次为：保存的真实动画/目标属性 → Details 加 key → 拖动和数值编辑各一次并 undo/redo → scrub/play/pause/stop/恢复及 event 策略 → 对应 Surface 值和真实呈现帧 → 保存关闭重开 → 切源/关闭期间拒绝旧请求 → drawer 焦点返回。每步附同一 document/animation/source revision/session、稳定 key/target、requested/applied time 和真实结果；测试夹具或中性 trait 不代替正常 EXE。三尺寸、实际 DPI144 与文本/弹层/滚动覆盖沿 [验证矩阵](widget-ui-validation.md)，当前全部产品门槛 open。

## 沿原r04的采样、结束与焦点接续

Parent已在实际Runtime落地原r03公开sampler候选；本域未应用旧proposal或改共享源码。当前[sampling-r03增量](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/preview-visibility-domain-r01/sampling-r03/summary.md)限原五个域内文件：`sample_preview_visibility(gesture, frame, sequence, channel, requested_seconds)`核当前source后调用已有 `sample_animation_bool_channel`，再调用原apply。返回 `UiPreviewVisibilitySample { requested_seconds, sampled_visible, outcome }`，outcome完整保留属性报告与publication；外层Ok不可作成功ACK。现有Step内部边界和端点clamp兼容行为原样保留，不另写插值器。

`finish_preview_visibility(gesture, frame, sequence)`用原source/session及Host的lease/frame/incarnation/sequence/值与presence检查结束手势，仅保留当前值、返回原frame并释放lease。它解决原同源二次begin的活动lease前置；不写文档、不保存pre-animated baseline，不是动画commit、Stop、Seek或renderer present。Cancel仍只能恢复同一有效手势的起点/presence，finish后新的begin/Cancel以当时真实现值为起点。Parent仍须接真实唯一clock、requested/actual time及外retained publication到内preview frame的关联，不能用TimelineModel playhead或场景AnimationEditor时间代替。

焦点次序仍由原共享host/capture处理IME与popup优先级。实际Widget取消若先于deactivate，可沿有效lease Cancel；deactivate/resize/Clone/换Host先发生则旧Cancel/finish拒绝，Reset只允许当前有效源重建，旧源不得恢复进新实例。六项新增回归明确两个次序及旧source/frame/sequence边界；原11项测试保持，当前17项均未编译/运行。没有重排输入、绑定ESC/Stop、改原六events/route或占桌面。源码静态通过不关闭UI-SH-WG01～07，采样到真实clock/呈现及Stop恢复仍open。

## 动画rev105接续与实际隐藏链

已实读动画原[rev105 source-mapping](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-transport-boundary-04/source-mapping.json)、[验证](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-transport-boundary-04/validation.json)与[逐图记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/penpot-transport-boundary-04/visual-review.json)。这是owner已完成保存重开/15原图复核的证据，本聊天未重开API或独立逐张复核这些图。Transport14、Timeline6、Curve3均为动画owner copies，不记作Widget消费；9份旧Transport copy的逐叶event metadata缺失与其余5份的六事件保持原记录，仍需正式canonical shapeRef解析。

当前Transport源SHA`eec15fa9…ff7b`匹配回执，原Record/Play/Pause/Previous/Next/Loop六个source node/control/event/route逐项与原v2资产一致，main保留六按钮+Fill。clock/status原UUID外移为历史读数，不是新clock/slot；Stop/Seek、consumer context slot、正式codec/导出与三main库发布均未交付。150个Point/Rect的目标矩形误差0只属于那些原生数据图形，不扩成整窗1DIP或产品通过。限定源码与34项owner来源指纹、Parent四个sampler源的核对见[本域接续记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/animation-transport-rev105-r05/reconciliation.json)。

本次沿现有代码定位到实际隐藏/退休顺序：

| 已有原路径 | 已核实行为 / Parent接续边界 |
| --- | --- |
| [recompute_if_dirty](../../../zircon_editor/src/ui/retained/app/host_lifecycle/recompute.rs:63) | 先保留已寻址window引用，成功发布新布局后才调用retire_hidden_retained_pane_inputs。失败走原dirty/retry，不在此构造动画时钟或假恢复成功。 |
| [retire_hidden_retained_pane_inputs](../../../zircon_editor/src/ui/retained/app/asset_events/preview.rs:32)、[hidden_retained_pane_input](../../../zircon_editor/src/ui/retained/contract/window/retained_pane_input.rs:210) | 检查pane已不可见后，携真实instance/window metadata交原manager；GeneratedBottom走原分支。此时没有Widget gesture、clock/backend context或恢复结果，不能把泛用hide自动当Stop。 |
| [host deactivation](../../../zircon_editor/src/ui/host/asset_editor_sessions/editing/palette.rs:46) → [Session](../../../zircon_editor/src/ui/asset_editor/session/designer_state.rs:30) | 原document guard/session lock下选择当前条目并停用输入；释放sessions锁后sync。现签名只有instance_id/metadata，不接gesture/frame/恢复策略；Parent必须在真实context owner关联仍有效的lease及source/session，不能据旧pane ID恢复新实例。 |
| [PreviewHost::deactivate_input](../../../zircon_editor/src/ui/asset_editor/preview/preview_host.rs:287) → [dispatch_input](../../../zircon_editor/src/ui/asset_editor/preview/preview_host.rs:213) | 用当前owner frame发Focused(false)，dispatch先退休visibility lease，再由原Surface清focus/IME/capture并rebuild。后到的Cancel/finish拒绝是保护边界，不能关闭它强恢复旧baseline。 |

如果已确认的业务动作要求恢复同一有效gesture，Parent应在上述停用前、原实例/source/frame仍有效时调用既有Cancel，保留完整Outcome中的report/publication和失败，再沿原deactivate释放输入。source已换、lease已退休或帧/sequence不符时拒绝旧恢复；明确Reset只能按当前有效源重建。若动作选择保留现值，既有finish也须在退休前用当前证明完成，不保存跨手势preanimated baseline。两者都不据此实现Stop、跨目标原子事务或present ACK。

停用后sync得到的是后续输入生命周期frame，不能替代刚才Cancel的属性回执/发布frame。Parent须分别关联外retained publication、内PreviewHost frame、真实clock请求/确认及实际present；完成源校验不等于时钟已seek，Published不等于已显示。焦点/IME/popup/capture优先级与drawer hide、tab close、document close继续由原host按真实原因处理，本文没有统一重排。

限定`zircon_editor/src`检索，sample/cancel/finish现只有原实现和测试调用，没有生产Widget clock/transport/lifecycle消费者。[调用边证据](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/animation-transport-rev105-r05/source-boundary.json)不作整个外部插件范围的否定结论。17项Widget回归由主测试文件16项及Clone测试1项组成，均未运行；Parent3项sampler也无新执行。此轮只接续两份本域文档，不改Widget Rust/资产、公共handler/Runtime、时钟或schema，不触发Jenkins/Cargo、插件、服务、Penpot或桌面。UI-SH-WG01～07、W09/W10及全族继续open。

## 动画r05回执与当前闭包

已实读动画 owner 的[r05回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/widget-consumer-r05.json)与[摘要](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/widget-consumer-r05-summary.md)。原sampling-r03补丁17261B、SHA `bd1ae9af4ab6d19e6857b591b5a0471fcca04c4d74264817f89a92b2ea0dcdf3`保持，五增量和十只读输入均匹配；未重新应用。五组去重37份源/资产，对旧candidate仍33匹配，差异为Session的`mod.rs`、`lifecycle.rs`、`palette_state.rs`与公共catalog。保留当前内容，不将旧候选称为完整现闭包。

本域本次读回与r05时点为36/37匹配：catalog由r05的`4c2f849c…36e5`更新为`e4bba09c…0a99`，具体当前哈希及两时点比较见[本域复核](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/animation-transport-rev105-r05/owner-r05-reconciliation.json)。该差异不归因写者、不回滚、不覆盖旧receipt或sampling candidate；两份原Layout/Canvas identity均匹配。原产品资产仍v2、97唯一control IDs、6events。

Sample/finish已存在源码；sample外层Ok仍须消费Rejected/RuntimeFailed及report的status/binding/invalidation、内层publication/frame。finish仅保值退休当前有效lease并返回原frame，不写属性/history或发布新帧。Cancel仅恢复同一有效手势的起点/presence；旧请求、源失效及deactivate顺序仍按上节原Host处理。原17项Widget测试定义及3项公共sampler测试定义均在，执行数仍0。限定`zircon_editor/src`的sample/cancel/finish检索仍只有原Session定义与测试调用，不扩大为整个外部插件的否定结论。

动画owner本次交接为rev140、原3main、isShared=false；该revision来自owner交接，本域未重新API读取或登记为Widget消费。同一真实clock/受检秒及requested/actual/failed ACK、原Session到Surface完整回执、外retained发布与真实present仍须Parent原host接线；场景frame/FPS和表面playhead不替代它们。原六events、Stop/Seek显式context提案、UI-SH-WG01～07及W09/W10继续open。本次仅接续原两份本域文档，operation/request/attempt/serverReceipt未新增或投递，未触发编译、服务、Penpot、桌面或capture。

## 动画r06回执与原消费版本

已实读动画owner唯一[r06合并交接](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/widget-consumer-r06-handoff.json)、[版本来源](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/widget-consumer-r06-revision-lineage.json)和[原验证](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/widget-consumer-r06-validation.json)，沿原UI-SH-WG01～07/W10及WG-C05/06/08接续。Widget消费仍是rev105、Transport draft2；原Transport源`eec15fa9d009c5a67b5ab2a0a6505e9882fe355f604c4287b6286a0711dcff7b`、v2/8IDs/6events实核保持。14/6/3只指该证据中的owner copies，不是Widget linked实例，也不表示动画文件后续copy总数。

r06的105→140→146→147→148都引用既有保存/关闭重开回执：140新增53个Sections夹具对象/9 Shared Button；146同步Shared并修复caption/CJK覆盖；147八TreeRow分配及四GraphRef未换行实验；148只恢复四个GraphRef fixed框、保留TreeRow修正。五份原回执SHA本域5/5匹配，复用既有失败和逐图路径，没有新API/save/浏览器或图片实验。Parent19:20Z的147 GET仍是独立只读观测，不补作保存/视觉证明。当前两份owner文档已含后续Sequence rev180记录，故与r06冻结文档hash不同；这两处后续变动不覆盖r06或升级Widget消费，本域没有改owner文档或查询当前Penpot版本。

离线存储比较的26个head中，仅23个owner copies及Timeline/Curve两个main共25完整子树在105/140/146相同，并与147/148修改范围无交集；Transport main仍缺原105的19个后代完整基线。根引用、存储shape hash和这25个子树不能替代正式逐叶shapeRef/原owner codec、外部handler执行或runtime呈现，不能用25/26关闭Transport导出缺口。

Parent第五次的具体close顺序已纳入上表：需要恢复有效手势时，在hide/源替换布局发布和deactivate退休lease之前消费原Cancel完整Outcome；明确保值时，在同一source/instance/lease/frame/incarnation/sequence/value/presence仍有效时消费原finish现frame，再走输入退休。普通hide只释放原输入；过期源/lease继续拒绝，不重新begin旧手势绕过，不把Reset或finish当Stop。属性report/publication、deactivate输入frame、外retained publication及renderer present仍分别确认。

[本域只读对账](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/animation-transport-rev105-r05/owner-r06-reconciliation.json)实际匹配56次/48唯一源码、4份原owner回执及6份r06附件，并保留两owner文档的历史/当前hash差异。17+3回归仍执行0；唯一真实clock、close原因/focus/capture/IME/popup和生产caller归Parent，控件不持clock。未改Widget/Runtime/ZUI/公共handler，未改两原identity或投递请求，正式codec/backend ACK/EXE/DPI仍open，passed/accepted=false。
