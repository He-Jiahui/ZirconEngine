---
related_code:
  - zircon_editor/src/core/notifications
  - zircon_editor/src/core/context
  - zircon_editor/src/core/i18n
  - zircon_editor/src/core/logging
  - zircon_editor/src/core/jobs/progress.rs
  - zircon_editor/src/ui/activity
  - zircon_editor/src/ui/host/editor_host_activity_decision.rs
  - zircon_editor/src/ui/host/play_pending_decision
  - zircon_editor/src/ui/host/project_recovery_decision
  - zircon_editor/src/ui/retained_host/app/workbench_notifications.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/dispatch_effects.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/dispatch_effects/side_effects.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/notifications.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/workbench/control.rs
  - zircon_editor/src/ui/retained_host/event_bridge.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_alerts
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_notification_center
  - zircon_editor/src/ui/retained_host/ui/pane_data_conversion/pane_component_projection/notification_center
  - zircon_editor/src/ui/retained_host/ui/workbench_window_projection/notification_cache.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/notifications.rs
  - zircon_editor/src/tests/host/retained_window/native_material_painter_notification_center.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/alert_toast_visual_screenshot.rs
  - zircon_editor/assets/ui/editor/components/workbench/primitives/feedback/workbench_notification_center.zui
  - zircon_editor/assets/ui/editor/components/workbench/primitives/feedback/workbench_toast.zui
  - zircon_editor/assets/ui/editor/windows/workbench_window.zui
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md
  - docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/10-notification-center-toast-decision-history-actions-retention-accessibility-diagnostic-integration-review.md
  - docs/plans/optimize/zircon_editor/130-editor-command-registry-keymap-menu-palette-context-routing-remote-automation-current-source-review.md
  - docs/plans/optimize/zircon_editor/131-editor-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-current-source-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Notifications/NotificationManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Notifications/SNotificationList.h
  - dev/UnrealEngine/Engine/Source/Developer/MessageLog/Public/IMessageLogListing.h
  - dev/UnrealEngine/Engine/Source/Developer/MessageLog/Public/Model/MessageLogListingModel.h
  - dev/godot/editor/gui/editor_toaster.h
  - dev/godot/editor/gui/editor_toaster.cpp
  - dev/godot/editor/editor_log.h
  - dev/godot/editor/editor_log.cpp
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Util/MessageManager.cs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor/Models/VFXErrorManager.cs
  - dev/Fyrox/fyrox-core/src/log.rs
  - dev/Fyrox/fyrox-ui/src/messagebox.rs
  - dev/Fyrox/fyrox-ui/src/progress_bar.rs
  - dev/bevy/crates/bevy_diagnostic/src/diagnostic.rs
refreshes: docs/plans/optimize/zircon_editor/10-notification-center-toast-decision-history-actions-retention-accessibility-diagnostic-integration-review.md
doc_type: current-source-review-and-refactor-plan
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# 132 - Editor Notification Center、Toast、Decision、History、Actions、Retention、Accessibility 与 Diagnostic Integration 当前源码复核

## 1. 结论

Editor10识别的主体架构差距仍然成立。当前代码已经不是纯占位：notification core有有界identity/model、toast deadline索引、progress的JobId索引和定向快照、decision ticket/incarnation/receipt cursor；severity覆盖缺陷也已经被修复，项目恢复流程开始复用共享Decision center。这些是应保留的工程基础。

但是产品层仍没有工程级Notification Center。当前Workbench把“最老pending decision的选项、活跃progress、尚未过期toast”每tick重新编码为最多64条pipe string，再由retained parser还原成最多8个row。它没有独立journal authority、稳定时间序、正常用户入口、read/dismiss/action协议、terminal progress、provider invalidation、插件owner lease或durable diagnostic fallback。普通toast/progress仍令center保持closed/non-interactive；core项消失时所谓history同步消失。

本轮保留Editor10的3个P0、52个P1与10个P2编号。当前状态为：P0 `2 Open / 0 Partial / 1 Closed`，P1 `49 Open / 3 Partial / 0 Closed`，P2 `8 Open / 2 Partial / 0 Closed`。唯一关闭项是P0-03 severity被`kind`覆盖；局部性能索引和恢复Decision接入不足以关闭其余架构项。本轮只做current-source review，没有修改生产代码。

## 2. 审查边界与可复验证据

### 2.1 物理范围

| 集合 | 文件 / 行 / 非空行 / bytes | tests / ignored | fingerprint |
|---|---:|---:|---|
| notification core | 25 / 4,048 / 3,639 / 131,465 | 49 / 4 | `966ada6e8748a89e7b717f1b0d5547e885c368f03fe0115f23b7ea6574bd17f7` |
| product projection与接入 | 107 / 16,166 / 14,744 / 568,478 | 139 / 6 | `18a707f7d483cf9be47f982ba69415ab53eebc169e90401e57e50985edaf63e5` |
| status-line绕行面 | 123 / 12,378 / 11,608 / 467,486 | 64 / 0 | `27ea40342c1d042180fe9f7865629ddfecc394d0e76288dadc11e5db58481d7a` |
| selected source union | 252 / 32,305 / 29,721 / 1,156,807 | 252 / 10 | `ea99548c2c0a73e4c0a1fa43141f32d4ed8778cc914d136e817ac73e3df0699f` |
| reference engines | 14 / 4,453 / 3,806 / 157,970 | 3 / 0 | `034ed860157468889bcf25d2292202cfcc14a669958dd317a84a4d557793e57a` |
| plan sources | 9 / 3,061 / 2,381 / 432,914 | 0 / 0 | `7470df10738366a09fa78a3ecfc8e3bbeda109fa0a5049f84548e02b46c7387d` |
| total evidence union | 275 / 39,819 / 35,908 / 1,747,691 | 255 / 10 | `d895576604fbee8e6fcf7fa7d28e87ae05b561b3a3c74d690f2b696461bf2f9a` |

fingerprint按相对路径排序，对`path + NUL + per-file SHA-256 + LF`清单再做SHA-256。它标识本轮阅读集合，不是ABI或资产兼容hash。product集合是frontmatter列出的非notification-core路径；status-line集合是排除`tests/`与`test(s).rs`后，所有直接出现`set_status_line(`的生产Rust文件，当前为123个文件、331处匹配。selected source union已去重。

### 2.2 在途源码与证据等级

成文时notification core、Activity/Play decision、retained notification bridge/parser均存在共享工作树修改，`ui/activity/decision`与`ui/host/project_recovery_decision`还是未跟踪新增路径。本报告逐项读取的是这些当前文件，没有回退、格式化、暂存或提交它们。源码仍可能继续变化，因此`source_recheck_required=true`；实施前必须重取fingerprint、测试和真实窗口证据。

- E3：toast publish/expiry/snapshot/current选择、decision publish/resolve/receipt、progress observer/refill/retire、bridge/parser/painter/input完整调用链。
- E3：Workbench资产默认状态、恢复Decision coordinator、producer-to-native测试与本地参考引擎指定文件。
- E2：123文件/330处status-line绕行清单，以及未发现通知入口、action route和生产default/cancel builder的absence proof。
- 未覆盖：真实窗口键盘/读屏/UIA、长时间压力、持久化恢复、插件卸载、headless/OS sink与跨平台行为；本轮没有运行Cargo，因为只写review文档且共享生产源码在途。

## 3. 当前实现中应保留的基础

1. `NotificationId`和option ID有长度/字符约束，toast/progress/decision payload有界，center容量有界，poisoned mutex可恢复。
2. toast新增`expires_at -> ids`索引后，清理不再每次扫描全部entry；10,000项probe evidence被标为managed ignored benchmark。
3. progress center新增`JobId -> NotificationId`索引与`snapshot_for_ids`，observer补位不再为每个binding重复全表扫描。
4. decision ticket包含center instance、ID与incarnation；resolution有幂等/冲突结果，receipt journal有cursor expiry语义。
5. 项目恢复已通过`ProjectRecoveryDecisionCoordinator`逐候选发布restore/discard/compare决策，使用ticket和receipt推进，cursor丢失时重新提示而不猜测破坏性结果。
6. parser已用`has_explicit_tone`保护显式`severity/level/tone`，`kind`只在没有显式tone时兼容映射；测试覆盖error/warning/success及legacy kind。
7. parser在`visible_limit`处停止解析，cache按generation/unread/overflow/selection/focus/limit复用，toast semantic equality忽略倒计时抖动。

这些改进应被吸收到统一authority中；不应以“已有三个center”为理由保留三个平行身份空间和字符串桥接层。

## 4. P0复核

### E-NOTIFY-P0-01 - Open - error toast仍可在从未展示前过期

`ToastCenterState.entries`仍是按`NotificationId`排序的`BTreeMap`；新expiration索引只优化删除。`snapshot_at`返回`entries.values()`，`sync_toast_queue`仍取`.first()`作为唯一current toast，而`expires_at`在`publish_at(now + lifetime)`时已经启动。因此字典序靠前的长寿命info可以遮住字典序靠后的短寿命error，后者未显示就被expiration索引删除。

固定ID重复仍返回`DuplicateNotification`，`publish_activity_toasts`仍把它与`Ok(())`一起静默吞掉。部分event bridge producer改用sequence ID降低了冲突，但import/play/recovery等路径仍未形成incident identity/coalesce receipt。必须增加monotonic enqueue sequence、priority、queued/visible/ack/dismiss/expired状态；queue wait不能消耗可见时长，error必须进入durable journal或产生明确suppressed receipt。

### E-NOTIFY-P0-02 - Open - 普通中心仍不可打开且没有history authority

`sync_notification_projection`仍每tick从`pending_decisions -> progress -> toasts`重建最多64条entry。`visible = !entries.is_empty()`，但`open/popup_open/input_*`全部绑定`decision_open = !pending_decisions.is_empty()`；普通toast/progress存在时center仍是closed/non-interactive。Workbench只挂载overlay center，没有bell/status/command route。core toast过期、progress retire或decision resolve后entry立即消失，bridge还明确以core snapshot而非previous UI内容为authority。

必须把toast viewport、live-state index与bounded journal分离；Workbench需要稳定入口、unread/severity badge、open/focus/close合同以及可搜索/过滤/分页的history。popup lifetime不得决定事实保留时间。

### E-NOTIFY-P0-03 - Closed - severity不再被kind覆盖

pipe parser现在记录`has_explicit_tone`；`severity|level|tone`设置显式tone，`kind`只在此前没有显式tone时兼容映射。`pipe_string_severity_is_not_overwritten_by_notification_kind`覆盖真实字段顺序和legacy kind，tone normalization也不再分配lowercase副本。该缺陷已关闭，但pipe codec本身仍由P1-45追踪，不能把修复一个解析顺序问题解释为typed DTO已经完成。

## 5. P1差距状态

### 5.1 Authority、identity、journal与delivery

| ID | 状态 | 当前源码差距与重构要求 |
|---|---|---|
| E-NOTIFY-P1-01 | Open | toast/progress/decision仍是三个live center，无统一record/sequence/journal；建立versioned `NotificationAuthority`和typed state transition。 |
| E-NOTIFY-P1-02 | Open | `NotificationSource`只做非空/长度检查，未canonicalize namespace；改为绑定builtin subsystem或plugin package/generation的owner identity。 |
| E-NOTIFY-P1-03 | Open | ID仍是全局空间；event sequence仅改善部分producer。改为`owner + local_id/incident`或capability-bound publisher。 |
| E-NOTIFY-P1-04 | Open | publish仍主要是insert/duplicate，缺统一update/replace/coalesce/complete/revoke revision；每次操作返回record revision和delivery receipt。 |
| E-NOTIFY-P1-05 | Open | toast/progress缺created/updated/terminal sequence与查询cursor；增加单调sequence、display timestamp和source timestamp。 |
| E-NOTIFY-P1-06 | Open | 缺project/document/asset/node/job/plugin context、tag与correlation graph；建立结构化context。 |
| E-NOTIFY-P1-07 | Open | retention仍是三个固定容量，缺severity/terminal/pin/owner预算；加入entry/byte/age/per-owner/per-severity policy与eviction reason。 |
| E-NOTIFY-P1-08 | Open | 所有state仍在进程内存；增加project/session journal、schema/build/project identity和恢复/归档策略。 |
| E-NOTIFY-P1-09 | Open | UI不可用、headless或popup抑制时没有per-sink delivery状态；建立Queued/Delivered/Acknowledged/Suppressed/Failed receipt。 |

### 5.2 Toast模型、排序、去重与动作

| ID | 状态 | 当前源码差距与重构要求 |
|---|---|---|
| E-NOTIFY-P1-10 | Open | `ToastNotification`仍只有severity/title/message/lifetime；增加bounded typed command/link/dismiss/progress/completion action。 |
| E-NOTIFY-P1-11 | Open | painter仍按几何宽度固定绘制英文`UNDO`和close，不读取空的`action_label`；只有存在有效model route时才绘制，并本地化label。 |
| E-NOTIFY-P1-12 | Open | Workbench toast资产无action/close event route；补hit region、pointer/keyboard dispatch、focus/disabled/busy与exactly-once receipt。 |
| E-NOTIFY-P1-13 | Open | current toast只投影message，丢title/source/context/count；改为结构化summary与可跳转detail。 |
| E-NOTIFY-P1-14 | Open | fixed-ID duplicate仍可静默吞掉；producer必须选择Replace/CoalesceCount/Append/Reject并保留first/latest/count。 |
| E-NOTIFY-P1-15 | Open | capacity满无error reserve、severity eviction或durable fallback；加入quota、overflow journal和health metric。 |
| E-NOTIFY-P1-16 | Open | lifetime仍从publish开始，hover/focus/窗口inactive不暂停；从delivery开始计时并定义assistive/headless policy。 |
| E-NOTIFY-P1-17 | Open | model允许任意非零lifetime，与“至少1秒”错误语义不一致；以typed duration policy统一验证和文案。 |
| E-NOTIFY-P1-18 | Open | center容量128而bridge只取64，后续项无window cursor/receipt；改为authority分页和准确省略计数。 |

### 5.3 Decision语义、生命周期与安全交互

| ID | 状态 | 当前源码差距与重构要求 |
|---|---|---|
| E-NOTIFY-P1-19 | Open | core有default/cancel API，但生产builder未调用；mandatory decision必须声明default/cancel/timeout/destructive policy。 |
| E-NOTIFY-P1-20 | Open | localized model保留default/cancel，`ActivityDecisionOption`只保留selection ID/title/message；DTO必须保持group/default/cancel/destructive semantics。 |
| E-NOTIFY-P1-21 | Open | option仍无destructive、disabled reason、shortcut、icon、help、confirmation或permission；扩展typed option descriptor。 |
| E-NOTIFY-P1-22 | Open | decision无deadline/timeout/escalation；增加clock-domain deadline与显式timeout resolution。 |
| E-NOTIFY-P1-23 | Open | center按ID排序、Activity只投影最老一组但无priority/scope；引入workflow queue policy与跨域公平性。 |
| E-NOTIFY-P1-24 | Open | 缺withdraw/revoke/supersede和owner teardown；project/document/plugin scope结束必须原子撤回。 |
| E-NOTIFY-P1-25 | Open | UI提交前只重新绑定当前snapshot，没有owner generation/command authorization；复用Editor130 command capability、provenance与revision。 |
| E-NOTIFY-P1-26 | Open | UI仍把每个decision option编码成独立通知row/下拉项；改为一个semantic modal/group，内部呈现选项。 |
| E-NOTIFY-P1-27 | Partial | 除Play外，项目恢复已使用共享Decision center和ticket/receipt；dirty close、overwrite、migration、plugin reload等仍有私有确认路径，继续逐域迁移。 |

### 5.4 Progress、job结合与终态

| ID | 状态 | 当前源码差距与重构要求 |
|---|---|---|
| E-NOTIFY-P1-28 | Open | production progress主要由全局job observer自动注册，domain无法声明阶段、scope、action或聚合；让job descriptor携presentation policy。 |
| E-NOTIFY-P1-29 | Open | observer refill仍调用按JobId顺序的`snapshot_limit`，没有foreground/priority/recency选择；使用visibility scheduler。 |
| E-NOTIFY-P1-30 | Open | observer构造/parse/publish失败仍可被吞掉；记录typed observer health、drop与resync diagnostic。 |
| E-NOTIFY-P1-31 | Open | job finish立即retire，terminal success/failure/cancel不进入history；原子转为带耗时、输出和retry action的terminal record。 |
| E-NOTIFY-P1-32 | Open | row无cancel/pause/retry/open-output，也不表达cancel requested/acknowledged；复用Editor131 job control receipt。 |
| E-NOTIFY-P1-33 | Open | progress仍主要是`Option<u8>`，缺unit/phase/ETA/rate/subtask；定义versioned progress sample和phase tree。 |
| E-NOTIFY-P1-34 | Open | 缺scope aggregation、group collapse和foreground status summary；建立operation tree及聚合查询。 |
| E-NOTIFY-P1-35 | Partial | JobId定向索引、64项截断前格式化、cache与semantic equality减少部分扫描/重建；稳定tick仍snapshot/clone/format pipe string，缺typed delta、row revision和window cursor。 |

### 5.5 Center、history、interaction与accessibility

| ID | 状态 | 当前源码差距与重构要求 |
|---|---|---|
| E-NOTIFY-P1-36 | Open | 固定decision -> progress -> toast顺序可挤掉error；按mandatory/severity/recency分层并准确计数。 |
| E-NOTIFY-P1-37 | Open | bridge保留64、asset只显示8，9..64无提示；DTO区分total/window/evicted并支持scroll request。 |
| E-NOTIFY-P1-38 | Open | toast硬编码unread，缺read cursor和多窗口一致性；authority持有逐项/分组/全部read state。 |
| E-NOTIFY-P1-39 | Open | 缺dismiss/clear/pin/mute/snooze/filter/search/group/tabs；实现受policy和owner capability约束的完整query/command surface。 |
| E-NOTIFY-P1-40 | Partial | parser已在visible limit停止，painter有overscan；仍只有8行静态截断、无scroll/window request/page cursor，尚未形成规模化virtual list。 |
| E-NOTIFY-P1-41 | Open | 非decision row点击走dropdown选择并成为no-op；定义独立`OpenDetails/ExecutePrimary/Select` typed activation。 |
| E-NOTIFY-P1-42 | Open | selected始终取第一项，focused/selected又被强制false；以stable record ID建立唯一interaction owner。 |
| E-NOTIFY-P1-43 | Open | aria文本仍是资产英文，modal/live-region/severity语义与实际输入状态不一致；从typed semantics生成本地化a11y tree。 |
| E-NOTIFY-P1-44 | Open | 无键盘入口、row/action导航、copy/dismiss/mark-read/filter/focus restore验收；建立完整keyboard/UIA matrix。 |
| E-NOTIFY-P1-45 | Open | severity顺序已修复，但pipe codec仍无schema/version/budget并会sanitize丢信息；进程内直接传typed DTO，跨ABI使用versioned codec。 |

### 5.6 Diagnostic、产品adoption与插件生态

| ID | 状态 | 当前源码差距与重构要求 |
|---|---|---|
| E-NOTIFY-P1-46 | Open | 123个生产文件/331处调用仍直接写status line，而toast相关匹配仅13处且含API/测试；分类ephemeral status、durable diagnostic与user notification。 |
| E-NOTIFY-P1-47 | Open | 新增Runtime task diagnostic bridge会把有界cursor/gap投影到`EditorLogService`，但Notification Service仍与log并列且不共享record/correlation；diagnostic应成为事实authority，各UI只是sink/query。 |
| E-NOTIFY-P1-48 | Open | literal error仍可伪装成localization key；分离`LocalizedText { key, args }`与bounded/redacted `DiagnosticText`。 |
| E-NOTIFY-P1-49 | Open | 缺asset/file/line/node/job/plugin token和typed jump；复用Editor130授权命令。 |
| E-NOTIFY-P1-50 | Open | graph/compiler diagnostic缺provider/subject/generation replace/invalidate；按Unity Graphics模型建立原子刷新。 |
| E-NOTIFY-P1-51 | Open | plugin source没有SDK publisher、quota、settings、unload revoke或diagnostic page；提供scope-bound capability并保留terminal journal。 |
| E-NOTIFY-P1-52 | Open | 缺publish/deliver/suppress/drop/evict/action latency/high-water指标；建立独立health页、metric和压力验收。 |

## 6. P2差距状态

| ID | 状态 | 当前源码差距与建议 |
|---|---|---|
| E-NOTIFY-P2-01 | Open | notification generation在`i64::MAX`饱和后不再失效；checked generation耗尽时重建epoch。 |
| E-NOTIFY-P2-02 | Open | `usize -> i64` overflow count转换未形成typed checked合同；保留`u64/usize`并显式饱和/报错。 |
| E-NOTIFY-P2-03 | Open | source只trim验证而保留原字符串；构造时canonicalize分段namespace。 |
| E-NOTIFY-P2-04 | Open | lifetime错误文案与非零验证仍不一致；文案和边界由同一policy生成。 |
| E-NOTIFY-P2-05 | Open | outcome、diagnostic severity与presentation tone混在`ToastSeverity`；拆分枚举并定义unknown映射。 |
| E-NOTIFY-P2-06 | Open | decision argument仍主要是`&'static str -> u64`；使用bounded typed argument和locale formatter。 |
| E-NOTIFY-P2-07 | Partial | toast expiration与progress JobId索引避免部分全扫，但center snapshot仍锁、clone并物化Vec；以generation snapshot/delta/arena和benchmark继续收敛。 |
| E-NOTIFY-P2-08 | Open | toast queue equality仍依赖pipe字段和volatile key别名；改为presentation revision typed equality。 |
| E-NOTIFY-P2-09 | Open | `EXPIRED_TOAST_ID`只表示center变空前的current ID；terminal delta必须携具体ID/reason/sequence。 |
| E-NOTIFY-P2-10 | Partial | 已有producer-to-native severity合同、native painter与截图测试，但没有真实window action/a11y/keyboard/property组合验证。 |

## 7. 参考引擎对照与适用边界

| 参考 | 仓内证据 | Zircon应吸收 | 边界 |
|---|---|---|---|
| Unreal Slate Notification | `FNotificationInfo`承载button/hyperlink/checkbox等typed affordance；item可更新completion、expire/fade/pulse。 | notification是可更新生命周期对象，action不是绘制出来的固定文本。 | Slate toast本身不等于durable journal。 |
| Unreal Message Log | listing/model有page、filter、selection、clear、message token与事件。 | transient delivery与可查询diagnostic listing分层，导航使用typed token/action。 | 不照搬具体Slate容器或假定其retention满足Zircon。 |
| Godot EditorToaster | main button、最高severity、silence/show、重复计数、hover计时、copy/close、旧项重放与UI-thread defer。 | 必须有常驻入口、严重度、可恢复查看、真实关闭/复制和duplicate aggregation。 | Godot的临时保留上限不是大型项目持久journal目标。 |
| Godot EditorLog | 有界行数、severity count/filter、search、collapse、clear、meta click和state保存。 | 详情落到可搜索/过滤/跳转的diagnostic surface。 | 文本行log不能成为最终typed schema。 |
| Unity Graphics | ShaderGraph按provider/node持有和清除message；VFX按origin/model dirty/invalidate/regenerate。 | compiler/graph diagnostics使用owner + subject + generation replace/invalidate。 | 这些Graphics包不是Unity全局通知中心。 |
| Fyrox | log有severity/time/listener/file/stdout与one-shot ID；UI提供message box/progress primitives。 | sink分离、时间、listener和基础控件可参考；one-shot升级为明确coalesce policy。 | 不能把这些文件当完整journal。 |
| Bevy | `DiagnosticPath`标识、bounded measurement history、enable、unit和统计查询。 | progress/metric保留typed identity、history上限与enable policy。 | measurement diagnostic不是Editor decision系统。 |

目标不是复制单一引擎，而是组合Unreal的typed lifecycle与Message Log分层、Godot的产品入口/重复/hover/close、Unity Graphics的provider invalidation、Bevy的bounded typed history，再用Zircon自己的owner lease、command capability、job scope和plugin lifecycle收敛。

## 8. 目标架构与所有权

```text
Domain Producer / Plugin Publisher / Job Scope / Diagnostic Provider
                              |
                              v
             NotificationAuthority (single writer contract)
 owner lease + record key/revision + sequence + context + policy
                /               |                 \
               v                v                  v
       Live State Index   Bounded Journal   Decision Receipt Store
       progress/decision  query + cursor    workflow + idempotency
                \               |                  /
                 +--------------+-----------------+
                                |
                    Delivery Policy / Router
          toast | center | status summary | log | headless/OS
                                |
                  typed action + delivery receipt
```

硬约束：journal record是事实，toast只是sink；status line不得是error唯一存储；decision是workflow而不是severity row；progress从active转terminal record；plugin只持scope-bound publisher/action capability；typed DTO必须贯穿到native host；notification action复用Editor130的command authorization/provenance，job action复用Editor131的scope/cancellation receipt。

## 9. 分层重构路线

### M0 - 封闭现存P0和虚假affordance

1. 为toast增加sequence/priority/display clock和durable fallback，补“被前序toast遮挡”合同。
2. fixed-ID producer显式选择coalesce/append/reject；duplicate/capacity必须产生receipt和metric。
3. Workbench增加真实notification trigger，使普通toast/progress也可打开；先建立最小journal再暴露history命名。
4. 删除固定`UNDO`/close绘制，或先完成typed route、hit-test和receipt后再显示。

### M1 - 统一record、owner lease与bounded journal

1. 定义`NotificationRecordKey/Revision/Sequence/OwnerLease/Context/Policy`及versioned payload。
2. toast/progress/decision center成为authority下专用index，硬切平行identity/retention authority。
3. journal支持entry/byte/age/owner/severity policy、cursor expiry、eviction reason与metrics。
4. 引入typed localized/diagnostic text、structured tokens与redaction policy。

### M2 - Delivery router、center query与产品入口

1. toast/center/status/log/headless成为独立sink，返回delivery receipt。
2. 实现badge、open/close、search/filter/group/page/read/dismiss/pin/mute和focus restore。
3. Editor Log与notification journal共享diagnostic record，避免复制字符串事实。
4. 明确定义inactive/headless/commandlet/OS policy，不静默丢弃。

### M3 - Decision workflow、typed action与progress terminalization

1. decision保留group/default/cancel/destructive/timeout/owner generation及withdraw/revoke/supersede。
2. action绑定command ID + typed args + authorization + record revision，生成exactly-once receipt。
3. progress复用job scope/control，展示phase/unit/ETA/cancel ack，并转terminal record。
4. 把dirty close、overwrite、migration、plugin reload等逐域迁入通用workflow。

### M4 - Retained UI、键盘、读屏与规模化列表

1. typed DTO替换pipe arrays；virtual list使用total/window/cursor而不是`visible_limit`静态截断。
2. stable ID reconcile focus/selection；无关progress delta不得移动decision焦点。
3. 实现键盘入口、导航、action、dismiss、mark-read、copy、filter、Escape/cancel与focus restore。
4. accessibility tree提供本地化role/name/description/modal/live-region/severity/action state。

### M5 - 产品adoption、provider diagnostics与插件SDK

1. 对123个status-line文件分类并迁移save/import/build/export/shader/asset/plugin/play/recovery关键错误路径。
2. graph/compiler provider使用owner + subject + generation replace/invalidate模型。
3. plugin SDK提供scope publisher、quota/settings、action manifest和unload revoke。
4. 建立source health页，覆盖malicious/flood producer和owner teardown。

### M6 - 性能、恢复与工程验收

1. 压测百万event、慢consumer、64+ progress、duplicate storm、plugin flood、locale切换和journal eviction。
2. 记录publish/query/delta/paint/action的p50/p95/p99、alloc、lock contention和high-water。
3. 故障注入partial write、sink crash、plugin unload、action timeout、cursor expiry与restart recovery。
4. Windows优先做真实Editor截图、纯键盘、UIA/读屏和200%缩放验收，再补平台差异。

## 10. 验收门

1. error在任意前序toast后仍进入journal，并最终可见或返回明确suppressed receipt。
2. queue wait不减少visible lifetime；hover、focus和assistive interaction按policy暂停。
3. 同incident聚合count/first/latest；不同incident不因固定producer key静默丢失。
4. capacity满时error有reserve，所有drop/eviction可查询且metric递增。
5. `Info/Success/Warning/Error`从producer到native row保持精确，unknown kind不改severity。
6. 无pending decision时，普通toast/progress仍有入口、可打开、可键盘聚焦。
7. toast过期、progress完成、decision解决后terminal record按policy留在history。
8. total/window/evicted一致，第9..64项不再无提示消失。
9. 100,000记录查询只物化可见window，滚动稳定且focus不丢失。
10. read/unread、dismiss、clear、pin、mute与badge在多surface一致。
11. 无action model不绘制action/close；有action时pointer、键盘和读屏均可执行。
12. action验证record revision、owner lease、authorization与provenance，重复提交不重复副作用。
13. decision以一个semantic group呈现，default/cancel/destructive不在adapter丢失。
14. Escape只选择声明的cancel；无cancel的mandatory decision不伪装成可取消。
15. unrelated toast/progress更新不改变decision focus、selection或default action。
16. project/document close与plugin unload原子revoke相关pending decision/live progress。
17. receipt cursor过期返回可恢复状态；restart后不猜测或重复执行破坏性选择。
18. progress支持determinate/indeterminate、phase、unit、ETA confidence和cancel acknowledgement。
19. job terminal success/failure/cancel进入journal，包含耗时、output/retry action和correlation ID。
20. 每条用户可见error在durable diagnostic store中有对应record，status line不是唯一载体。
21. asset/file/line/node/job/plugin token可通过typed command跳转，失效target有可解释结果。
22. provider generation刷新会移除已修复diagnostic，不残留旧badge。
23. plugin flood受quota约束；disable/unload后live项撤回、action失效、terminal journal可追溯。
24. localized text、literal diagnostic和redacted data分型，缺key/超长/敏感字段有测试。
25. Windows真实Editor通过鼠标、纯键盘、UIA/读屏与200%缩放，无重叠、截断或焦点陷阱。
26. benchmark记录publish/query/delta/paint/action的p50/p95/p99、alloc/high-water，并绑定source/build/hardware profile。

## 11. 与相邻报告的所有权

- Editor131拥有job admission、cancel acknowledgement、event journal和shutdown；本文拥有progress/terminal/action的用户可见投影。
- Editor130拥有command identity、authorization、provenance与remote policy；本文所有action必须复用，不另造字符串route。
- Editor06拥有plugin lifecycle/settings/manager；本文拥有scope-bound publisher、quota和unload revoke。
- Editor07拥有Play session/world/process；本文拥有pending-edit decision的通用workflow与可访问交互。
- Editor01及Runtime UI专项拥有retained/text/input/GPU painter底座；本文定义notification产品语义和验收。
- 后续logging/diagnostic专项拥有完整retention/export/query/telemetry；本文明确notification不能替代durable log。

## 12. 本轮未实施内容

本轮没有修改notification core、retained host、UI资产、tests、logging、jobs、plugin SDK或command routing，也没有处理tooling迁移。共享工作树中的notification/decision改动均按当前源码审阅并保留。后续实施应按M0-M6分层推进，每一层重新取源码证据，先建立typed contract和失败测试，再硬切旧字符串/平行authority；不得为了短期演示继续扩展pipe字段、固定视觉action或status-line-only错误路径。
