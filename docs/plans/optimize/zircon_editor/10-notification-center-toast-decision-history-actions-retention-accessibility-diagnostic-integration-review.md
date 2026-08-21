---
related_code:
  - zircon_editor/src/core/notifications
  - zircon_editor/src/core/context
  - zircon_editor/src/core/i18n
  - zircon_editor/src/core/logging
  - zircon_editor/src/ui/activity
  - zircon_editor/src/ui/host/play_pending_decision
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
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 10 · Notification Center、Toast、Decision、History、Actions、Retention、Accessibility 与 Diagnostic Integration 工程化差距

## 1. 结论

Zircon Editor已经有一套值得保留的通知core种子：稳定格式的`NotificationId`、builtin/plugin source分类、有界toast/progress/decision center、基于单调时钟的toast expiry、job progress自动绑定、decision ticket/incarnation/receipt cursor，以及在同一locale snapshot下完成的显示投影。这些不是临时空壳，后续重构不应推倒重来。

但当前产品并不存在工程级Notification Center。它把三种不同生命周期的对象临时拼成一个字符串数组：Play modal的每个选项、活跃job progress和仍未过期的toast。这个数组没有历史authority、没有用户入口、没有可靠排序、没有typed action、没有read/dismiss状态，也没有与Editor Log/diagnostic owner收敛。最严重的三个断点均可由当前源码闭环证明：

1. toast按`NotificationId`字典序选择当前项，lifetime却从publish时开始；一个较早、长寿命、字典序靠前的info toast可让后来的短寿命error在从未显示的情况下过期，重复固定ID又被产品当作成功静默吞掉。
2. 普通toast/progress只令center `visibility=visible`，却保持`open=false`、`popup_open=false`和全部input flag为false；窗口没有通知入口，core项过期/结束后所谓history立即清空，用户无法恢复查看失败。
3. bridge生成`severity=error|...|kind=toast`，字符串parser又把`kind`当tone并按后字段覆盖前字段；因此error/warning/success在Notification Center结构化row中全部变成`info`。

本报告记录3个P0、52个P1、10个P2，给出M0-M6重构路线与26个验收门。没有修改生产代码。由于上一轮同一工作树的`zircon_editor --lib`测试编译已经在617.2秒后被239个既有test-build错误阻断，本轮没有重复消耗相同Cargo lane；P0使用逐文件静态调用链和现有测试源码反向确认，不能写成动态测试通过。特别需要注意：现有测试明确断言普通toast时center保持closed/non-interactive，并明确断言core快照清空后“history”也清空；另有painter测试明确要求始终画出`UNDO`和close mark。也就是说，问题已进入当前测试合同，而不是未覆盖的偶发行为。

## 2. 审查边界与可复验证据

### 2.1 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| notification core | 25 / 3,469 / 110,835 | E3：identity、source、toast/progress/decision model与center、presentation、service及全部38项test attributes；fingerprint `11deead3...0c7508` |
| product projection与接入 | 60 / 9,299 / 343,875 | E3：context、activity view、Play adapter、toast producers、retained bridge、parser/cache/painter、window/component assets；fingerprint `8f778590...6e7b83` |
| status-line绕行面 | 126 / 10,985 / 408,716 | E2 inventory、代表error path E3：共317条生产匹配行；fingerprint `f2c866e9...e67836` |
| selected combined scope | 208 / 23,488 / 853,636 | 当前工作树fingerprint `b7f7e681...3b4bb2`；core 38个test attributes、0 ignored |

fingerprint按相对路径排序，将`path + NUL + per-file SHA-256 + LF`拼接后计算SHA-256。它只标识本轮阅读集合，不是ABI、schema、资源identity或兼容性hash。

product集合由所有生产notification identity/model/view/bridge/parser/painter引用组成，并显式加入Workbench toast/center/window资产和toast action/text leaf；排除dedicated test文件。status-line集合是`zircon_editor/src`中排除test路径后所有直接调用`set_status_line`的Rust文件。两个集合有交集，combined已去重。

### 2.2 在途文件与验证隔离

成文时notification core和主要production projection文件未显示修改；`ui/host/play_pending_decision/tests`有4个在途测试文件，且更大的Editor工作树有大量其他会话修改。本报告没有回退、格式化、暂存或提交这些文件。由于测试合同在途且全lib test仍有已知编译阻断，实施前必须重取源码、测试、fingerprint和产品截图，故`source_recheck_required=true`。

证据等级：

- E3：三个P0的producer -> center -> view -> string bridge -> parser -> painter/input链均逐文件闭环。
- E3：core API、容量、expiry、decision receipt、job observer、窗口资产和生产toast callers逐文件读取。
- E2：126文件status-line绕行面、全量字符串搜索的absence proof、38项core test与外围notification测试源码。
- 未覆盖：真实Editor窗口键盘/读屏运行、长时间压力、locale切换截图、插件动态卸载、跨进程恢复、OS通知和headless commandlet输出。

### 2.3 本轮追踪的产品链

1. producer构造`ToastNotification/ProgressNotification/DecisionNotification` -> context-owned `EditorNotificationService`。
2. toast center按publish epoch保存expiry；job observer按JobId自动publish/retire progress；decision center发布ticket并记录bounded receipt。
3. retained tick读取pending Play options、live toast和active progress -> `Activity*View` -> `sync_notification_projection`。
4. bridge把typed对象编码为pipe-separated string -> NotificationCenter parser重新猜测title/message/tone/unread -> structured options -> native painter。
5. option selected只对Play decision ID执行resolve；其他center row落回dropdown `options`路径并成为no-op。
6. toast overlay另取snapshot第一项，只投影message、severity和remaining lifetime；painter按几何宽度无条件画`UNDO`与close mark。
7. 绝大多数Editor成功/失败仍直接写status line，Editor Log与Notification Service没有共享record、action、owner、retention或delivery receipt。

## 3. 已有工程基础，重构时必须保留

### 3.1 Identity与bounded core不是演示代码

- `NotificationId`要求至少三段、只允许小写ASCII/数字/下划线且总长有界，避免自由字符串直接成为全局key。
- toast title/message、progress title、decision title/message/option都有限长检查；`bounded_message`按UTF-8边界截断。
- toast默认容量128、progress容量64、decision pending/receipt默认128/256；所有center在mutex poison后恢复guard，而不是直接panic。
- toast使用context-owned `Instant` epoch并用`checked_add`计算expiry，leaf host没有各造一套wall-clock。

### 3.2 Decision ticket与receipt有真实并发防线

- ticket携center instance、notification ID和incarnation；foreign/stale ticket明确拒绝。
- option集合2..16且ID唯一；default/cancel option在core model和localized presentation中都能保留。
- resolve具有幂等same-option report；不同option二次resolve返回`AlreadyResolved`，不会静默改写用户选择。
- receipt sequence使用checked increment，bounded cursor过期返回resume cursor；Play adapter不会在receipt缺失时猜测Apply/Discard结果。

### 3.3 Progress绑定与投影已有局部正确性

- progress notification只持`JobId`而不持typed result receiver，避免UI意外延长结果owner。
- 自动job observer在admit/finish/resync维护binding；snapshot捕获`NotificationId -> JobId`，避免并发复用ID时误删replacement。
- job progress projection支持determinate/indeterminate，完成百分比计算有zero-total防线并上限100。

### 3.4 Retained painter已有性能意识

- center closed时painter直接返回，不执行palette、metrics或row work。
- row painter按可见范围加一行overscan，notification cache比较generation/unread/overflow/selection/focus/limit，避免稳定帧全量重建。
- toast countdown字段被排除在semantic equality外，避免每tick只因剩余毫秒变化重建完整模板树。

这些基础应被纳入新的`NotificationAuthority`，而不是再新增第四套通知实现。

## 4. P0：可靠交付、可恢复性与严重度完整性

### E-NOTIFY-P0-01 · error toast可以在从未成为current toast前过期并永久丢失

`ToastNotificationCenter`用`BTreeMap<NotificationId, Snapshot>`保存项，`snapshot_at`返回key字典序；`sync_toast_queue`直接取`.first()`作为唯一current toast。lifetime在`publish_at`时开始，center没有queued/visible/acknowledged状态，也不会在前一个toast结束后重新授予完整显示时间。

因此序列是确定的：先发布字典序靠前、寿命较长的info；再发布字典序靠后、寿命较短的error。第二项始终不是`.first()`，但expiry持续倒数，最终在snapshot prune中消失。它既未显示，也没有历史。产品又为import、dispatch和Play outcome使用固定ID；同ID尚存时的新事件返回`DuplicateNotification`，`publish_activity_toasts`把该错误与成功同等处理。容量满则只写status line，status line本身又可被下一条状态覆盖。

这不是“toast排序不够漂亮”，而是错误交付协议缺失。必须引入单调sequence、priority、enqueue/visible/ack/dismiss/expired delivery state、每severity保留额度和durable diagnostic fallback。过期计时应由展示策略明确：queue wait不得消耗用户可见时间；错误即使toast被抑制，也必须进入可查询journal并返回typed delivery receipt。重复事件应聚合count/latest occurrence，而不是伪装为成功。

### E-NOTIFY-P0-02 · 普通通知中心不可打开，所谓history随live snapshot消失

Workbench window只挂载一个overlay center，没有bell/status入口或open route。bridge把`visible`设为“entries非空”，却把`open`、`popup_open`和全部input flags设为`decision_open`；只有pending Play decision存在时才为true。native center painter在`popup_open=false`时立即返回，所以普通toast/progress即使`visibility=visible`也不会形成可见panel。

`MAX_NOTIFICATION_HISTORY`命名掩盖了事实：entries每tick从pending decision + active progress + live toast重新构造。toast expiry、job retire或decision resolve后条目立即删除；现有测试还明确要求snapshot清空时不保留第二份history。结果是用户错过toast后没有任何产品入口恢复错误详情，长期任务完成状态也瞬间消失。

必须将toast viewport与Notification Center journal分离：toast是journal record的一种delivery sink，center是可打开、可搜索、可过滤、可标记已读的查询面。window chrome需要稳定入口、最高severity/unread badge和可测试的open/close/focus restore合同；journal retention不能由popup lifetime决定。

### E-NOTIFY-P0-03 · pipe parser把全部toast严重度覆盖为info

bridge生成的历史项顺序为：

```text
id|title=...|message=...|severity=error|unread=true|kind=toast
```

`notification_entry_from_string`逐字段解析，并把`"severity" | "level" | "kind" | "tone"`都写入同一个`entry.tone`。因此先写入`error`，随后`kind=toast`通过`normalized_tone("toast")`变成fallback `info`并覆盖前值。warning、success同样被覆盖；progress和Play decision也把业务kind误当tone。现有bridge测试只断言raw字符串仍含`severity=success`，component parser测试则单独测试`kind=done -> success`，没有覆盖真实组合顺序，形成合同间隙。

这会破坏错误颜色、图标、排序、辅助技术announcement和任何未来severity policy。修复必须先停止字符串复编码，直接投影versioned typed DTO；短期P0 guard至少要让`kind`与`severity`成为正交枚举，并增加真实producer-to-native-row测试，覆盖所有severity、字段顺序和未知kind。

## 5. P1：authority、toast、decision、progress、UI 与生态缺口

### 5.1 Authority、identity、journal与delivery

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| E-NOTIFY-P1-01 | 三个独立center只有各自live state，没有统一record、sequence或journal。 | `NotificationAuthority`拥有versioned record、monotonic sequence、typed payload和state transition；toast/progress/decision只是payload/policy。 |
| E-NOTIFY-P1-02 | `NotificationSource`只检查非空和128 bytes，保留原始空白/大小写，也不验证namespace。 | canonical `NotificationOwnerId`，绑定builtin subsystem或plugin package identity、generation和owner lease。 |
| E-NOTIFY-P1-03 | ID全局冲突，未与source/owner组成identity；任一producer可占用他人ID。 | `NotificationKey { owner, local_id }`或capability-bound publish client，跨owner不冲突也不能伪造。 |
| E-NOTIFY-P1-04 | publish只有insert/duplicate，缺update、replace、increment、resolve、withdraw的统一generation语义。 | typed `publish/update/coalesce/complete/revoke`，每次返回record revision和delivery receipt。 |
| E-NOTIFY-P1-05 | toast/progress没有created/updated/terminal timestamp或sequence，无法按时间查询和稳定分页。 | 单调sequence + wall-clock display timestamp + source event timestamp，分页cursor检测窗口过期。 |
| E-NOTIFY-P1-06 | 没有category、project/document/asset/node/job context、tags或correlation ID。 | structured context与correlation graph，支持按project、operation、asset、job和source过滤。 |
| E-NOTIFY-P1-07 | retention只有三个固定容量，不能按severity、terminal outcome、user pin或compliance policy区分。 | entry/byte/age/per-owner/per-severity多维policy，error与decision receipt有独立下限及可观测eviction。 |
| E-NOTIFY-P1-08 | 所有state仅进程内存，crash/restart后未读错误、运行结果和决策恢复全部丢失。 | project/session journal与可选持久化；启动时按schema/build/project identity恢复或明确归档。 |
| E-NOTIFY-P1-09 | 没有sink delivery状态；UI不可用、headless、窗口关闭或popup抑制时producer不知道消息去了哪里。 | sink-independent record + per-sink `Queued/Delivered/Acknowledged/Suppressed/Failed` receipt和fallback policy。 |

### 5.2 Toast模型、排序、去重与动作

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| E-NOTIFY-P1-10 | `ToastNotification`只有severity/title/message/lifetime，没有action、dismiss、progress、link或completion state。 | bounded typed actions，含command ID、arguments、authorization、enabled predicate、destructive/default metadata和expiry。 |
| E-NOTIFY-P1-11 | painter只按宽度判断action，完全不读`action_label`，固定画英文`UNDO`和close mark。 | 只有model声明且route可解析时才绘制action；label本地化，close绑定typed dismiss，缺route时构造失败。 |
| E-NOTIFY-P1-12 | window asset没有toast action/close events，生产搜索也没有对应dispatch；视觉affordance不可操作。 | hit region、pointer/keyboard dispatch、focus ring、tooltip、disabled/busy状态和exactly-once action receipt。 |
| E-NOTIFY-P1-13 | current toast只显示message；title、source、context和重复次数只存在于未消费queue string或直接丢失。 | 结构化toast layout显示短title/summary，detail跳转journal；source/context按policy呈现。 |
| E-NOTIFY-P1-14 | 固定ID重复被静默吞掉，无法区分同一incident更新与新incident。 | producer选择`Replace/CoalesceCount/Append/Reject` policy；聚合保留first/latest time和occurrence count。 |
| E-NOTIFY-P1-15 | capacity满没有severity-aware eviction、reserved error budget或durable fallback。 | per-owner quota、error reserve、lowest-priority eviction、overflow journal和health metric；不得回退到瞬时status。 |
| E-NOTIFY-P1-16 | lifetime从publish开始，UI hover/focus、读屏announcement和window inactive都不暂停或补偿。 | delivery开始后计时；hover/focus/assistive interaction暂停；inactive/headless转journal或OS sink。 |
| E-NOTIFY-P1-17 | center允许1ns lifetime，错误文本却承诺至少1秒；policy和validation不一致。 | typed duration policy与同源错误文案；按severity/action/accessibility设置合理下限。 |
| E-NOTIFY-P1-18 | center容量128，bridge queue只取64；后64项没有明确overflow、drop receipt或展示机会。 | authority分页/window request，queue仅持可显示window；所有省略都进入准确计数和cursor。 |

### 5.3 Decision语义、生命周期与安全交互

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| E-NOTIFY-P1-19 | core支持default/cancel，但唯一生产builder未调用`with_default_option/with_cancel_option`。 | 每个mandatory decision必须声明default、cancel、timeout和destructive policy，构造期验证。 |
| E-NOTIFY-P1-20 | `LocalizedDecisionNotification`保留default/cancel，Play adapter DTO却丢弃二者。 | presentation DTO完整保留decision identity、option metadata、default/cancel和owner generation。 |
| E-NOTIFY-P1-21 | adapter把每个option展开成一条“通知”，并把label拼入message；同一decision不再是一个语义group。 | 一个modal/card对应一个decision，内部呈现typed buttons/radio choices；receipt关联decision而非伪row。 |
| E-NOTIFY-P1-22 | center只有resolve/cancel；`cancel`其实选择cancel option，没有producer withdraw/revoke/update API。 | 区分user cancel、producer withdraw、owner revoke、supersede和timeout，每种有typed terminal reason。 |
| E-NOTIFY-P1-23 | project/plugin/document owner消失时无法批量撤回decision，stale mandatory modal可继续占据UI。 | owner lease teardown原子revoke其pending decisions；UI收到terminal delta并恢复焦点。 |
| E-NOTIFY-P1-24 | decision bridge每次history变化都把selected ID和focused index重置为第一项；并发progress更新可扰动用户导航。 | focus/selection由interaction state owner保存，以stable option ID reconcile；无关record更新不得移动焦点。 |
| E-NOTIFY-P1-25 | mandatory dialog禁用Escape和backdrop，但`aria_modal=false`，也没有typed cancel键或焦点恢复合同。 | 真modal semantics、focus trap/restore、Escape到cancel option、screen-reader title/description关联和无障碍超时策略。 |
| E-NOTIFY-P1-26 | receipt journal只在内存保留256项，具体side effect的幂等/恢复仍由Play adapter私有状态拼装。 | durable decision workflow receipt，绑定owner operation/generation、selected option、commit result与recovery state。 |
| E-NOTIFY-P1-27 | 只有Play pending edits使用decision center，dirty close、overwrite、plugin reload、migration等仍各走私有dialog/status路径。 | 通用decision workflow接入所有需用户确认的跨域事务，但保留domain-specific typed payload和policy。 |

### 5.4 Progress生命周期、控制与可观察性

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| E-NOTIFY-P1-28 | production progress只有全局job observer自动注册；domain无法声明阶段、标题、action或聚合关系。 | job spec/progress descriptor直接携presentation policy、scope、phase tree、cancel/open-output actions。 |
| E-NOTIFY-P1-29 | observer从`snapshot_limit(64)`补位，当前job source按最小JobId取样；没有priority、foreground或用户关注度。 | visibility scheduler按interaction priority、severity、scope与recency选择，剩余项可分页查询。 |
| E-NOTIFY-P1-30 | observer构造/parse/publish失败全部`return`或`let _ =`吞掉。 | notification delivery不能影响job正确性，但必须记录typed observer health、dropped binding和resync diagnostics。 |
| E-NOTIFY-P1-31 | job finish立即`retire_job`，完成/失败/取消结果不进入notification journal。 | active progress原子转换为terminal record，保留结果摘要、耗时、输出/重试action和retention policy。 |
| E-NOTIFY-P1-32 | progress row没有cancel/pause/retry/open output操作，也不显示cancellation requested/acknowledged。 | action调用Editor09定义的typed job control，并呈现request、ack、quiescent和terminal reason。 |
| E-NOTIFY-P1-33 | progress只有`Option<u8>`；没有work unit、phase、ETA、throughput、bytes、subtask或backward-progress解释。 | versioned progress sample含completed/total/unit、phase tree、rate/ETA confidence和indeterminate reason。 |
| E-NOTIFY-P1-34 | 多job没有scope aggregation、group collapse、foreground task或status-bar summary。 | job/operation tree聚合，center显示group；status bar只投影最高优先级摘要并可跳到详情。 |
| E-NOTIFY-P1-35 | 每tick会clone toast/progress snapshot并格式化pipe string；progress文本变化推进整个center generation。 | typed delta/cursor、stable row revision和windowed projection；只更新变化row，记录CPU/alloc/latency budget。 |

### 5.5 Center、history、interaction与accessibility

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| E-NOTIFY-P1-36 | entries固定为decision option -> progress -> toast，64个前两类可完全挤掉error toast。 | query policy先按mandatory/terminal severity/recency分层，再在组内稳定排序；不可见项准确计数。 |
| E-NOTIFY-P1-37 | bridge保留64项，asset `visible_limit=8`在parse阶段截断；overflow只计算超过64，9..64无提示消失。 | viewport/window metadata区分`total_count/window_start/window_count/evicted_count`，scroll请求下一window。 |
| E-NOTIFY-P1-38 | 每个toast硬编码`unread=true`，没有mark read/unread、read cursor或跨窗口同步。 | read state由user/session authority持有，支持逐项、按组和全部标记，并与badge一致。 |
| E-NOTIFY-P1-39 | 没有dismiss、clear、pin、mute source、snooze、filter、search、group或severity tabs。 | 完整Notification Center query/command surface；动作受policy和owner capability约束。 |
| E-NOTIFY-P1-40 | painter已有overscan，但parser先截成8项，没有scroll/window request，规模化virtualization不可达。 | retained virtual list消费paged DTO，滚动只取可见window并保持stable key/focus。 |
| E-NOTIFY-P1-41 | 非decision row点击落入dropdown `options`选择，而center只有`notifications`，结果是no-op。 | row activation是typed `OpenDetails/ExecutePrimary/Select`，与dropdown协议完全分离。 |
| E-NOTIFY-P1-42 | selected ID每次指向第一项，`FOCUSED/SELECTED`又被强制false；模型状态、视觉状态和输入状态分裂。 | 单一interaction state owner，以record/option stable ID reconcile hover/focus/selection。 |
| E-NOTIFY-P1-43 | aria label/description是资产内英文原文，不走i18n；modal标记与实际强制交互矛盾。 | localized accessible name、role、modal/live-region、severity announcement和关系ID由typed semantics生成。 |
| E-NOTIFY-P1-44 | 没有键盘打开入口、row导航、action cycle、copy、dismiss、mark-read、filter和focus restore验收。 | 完整keyboard contract和screen-reader automation matrix，鼠标不是decision唯一可靠路径。 |
| E-NOTIFY-P1-45 | pipe codec只替换`|/=/whitespace`，Play decision title/message甚至未sanitize；字段扩展无schema/version。 | 直接传typed DTO；如跨ABI则使用有length/budget/version/unknown-field policy的codec。 |

### 5.6 Diagnostic、产品 adoption 与插件生态

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| E-NOTIFY-P1-46 | 126个生产文件/317条匹配仍直接写status line，而toast publish相关生产匹配只有17行。 | 制定adoption policy：状态摘要、durable log、user notification分别有明确用途；error不得只留status。 |
| E-NOTIFY-P1-47 | `EditorLogService`与Notification Service并列，toast/error/progress没有统一record或correlation。 | diagnostic record为事实authority；log、badge、toast、center、status bar是不同query/sink。 |
| E-NOTIFY-P1-48 | toast可以携literal error作为`message_key`，i18n miss又原样返回key；key与literal/redacted diagnostic混为一型。 | `LocalizedText { key, typed_args }`与`DiagnosticText { bounded, redacted }`分型，禁止自由replace格式化。 |
| E-NOTIFY-P1-49 | 没有tokenized asset/file/line/node/job/plugin jump；详情只能是一段字符串。 | structured diagnostic tokens与typed navigation command，复用Command08的授权/provenance。 |
| E-NOTIFY-P1-50 | graph/asset/compiler diagnostics没有provider/model generation和invalidate/clear语义，无法可靠移除已修复badge。 | 参考Unity Graphics建立provider/subject/generation owner，可按provider、node、compile pass原子replace/invalidate。 |
| E-NOTIFY-P1-51 | `NotificationSource::plugin`只有构造器，没有SDK contribution、quota、unload revoke、settings或diagnostic page接入。 | plugin SDK暴露scope-bound publisher、schema、budget和actions；disable/unload原子撤回live项并保留terminal journal。 |
| E-NOTIFY-P1-52 | 没有published/delivered/suppressed/dropped/evicted/action latency/high-water指标，也无source health页。 | 内建metrics、structured audit、pressure test和operator diagnostics；notification系统自身失败不得只通知自己。 |

## 6. P2：局部一致性、边界与维护性

| ID | 当前差距 | 建议处理 |
|---|---|---|
| E-NOTIFY-P2-01 | `next_notification_generation`饱和在`i64::MAX`后，后续变化不再使cache失效。 | 使用checked generation并在耗尽时显式重建epoch，禁止静默饱和。 |
| E-NOTIFY-P2-02 | `usize overflow_count as i64`在极端值可wrap，随后被non-negative parser压成0。 | 使用checked/saturating conversion并保留`u64/usize` typed count。 |
| E-NOTIFY-P2-03 | source只用trim判断却保存原字符串，`"editor"`与`" editor "`成为不同owner。 | 构造时canonicalize并验证分段namespace。 |
| E-NOTIFY-P2-04 | toast错误文案承诺1秒下限，model只拒绝zero。 | 文案与validation同源，并加入999ms/1s边界测试。 |
| E-NOTIFY-P2-05 | toast severity缺少Debug/Critical/Fatal或可扩展policy映射，Success又与diagnostic severity混在同一枚举。 | 分离outcome、severity与presentation tone，unknown值有forward-compatible policy。 |
| E-NOTIFY-P2-06 | decision argument只能`&'static str -> u64`，不支持typed string/path/count/plural和安全格式化。 | bounded typed argument enum，locale formatter负责plural/unit，不做全局string replace。 |
| E-NOTIFY-P2-07 | center snapshot普遍clone Arc外的Vec/map/job snapshot，稳定tick仍有锁和分配。 | immutable generation snapshot、delta cursor和arena/shared rows；以benchmark决定优化优先级。 |
| E-NOTIFY-P2-08 | toast queue semantic comparison依赖字符串字段顺序并维护volatile key别名表。 | typed equality只比较presentation-relevant revision，移除字符串协议。 |
| E-NOTIFY-P2-09 | `EXPIRED_TOAST_ID`只记录center变空前的current ID，不表达具体过期/被替换/被dismiss项。 | terminal delta携record ID、reason和sequence，不借单字符串推断队列变化。 |
| E-NOTIFY-P2-10 | tests大量断言源码字符串或raw bridge属性，却缺真实producer-to-native semantic/property tests。 | contract tests围绕typed DTO、interaction、a11y tree和pixel/behavior组合，源码shape test只作辅证。 |

## 7. 参考引擎对照与适用边界

| 参考 | 仓内可验证能力 | Zircon应吸收的原则 | 不应照搬/不可推断 |
|---|---|---|---|
| Unreal Slate Notification | 可更新item、completion state、expire/fade、pulse、button callbacks、checkbox、hyperlink、copy、window anchoring；progress有handle和独立handler。 | toast/action/progress必须是typed lifecycle对象，非字符串外观；短通知能跳到durable log。 | Slate API本身不等于完整durable journal，也不能证明其所有队列都有Zircon目标的quota/persistence。 |
| Unreal Message Log | 独立listing、filter、selection、page change、clear、export string、token execution和data/selection events。 | transient notification与可查询diagnostic listing分层；navigation使用token/action。 | 不复制其具体Slate/UI容器或假定分页策略适合所有Zircon规模。 |
| Godot EditorToaster | 常驻main button、最高severity indicator、show/silence、最多5个临时可见项、重放旧项、hover重置计时、duplicate count、copy/close和线程defer。 | 至少保证入口、严重度、可恢复查看、重复聚合、真实close/copy与UI线程边界。 | Godot仅保留约两倍临时项，不是大型项目持久journal目标。 |
| Godot EditorLog | 10,000行limit、severity counts/filter buttons、search、collapse、clear、meta click与state保存。 | notification详情应落到有界、可搜索、可过滤、可跳转的诊断surface。 | 行文本log不应成为Zircon typed record的最终schema。 |
| Unity Graphics ShaderGraph/VFX | diagnostics按provider + node/model持有；severity排序；可按provider/node清除；VFX有origin、dirty/scheduled model、invalidate/regenerate。 | 编译/图形authoring诊断需要owner/subject/generation和replace/invalidate，不是只追加toast。 | 仓内Graphics包不是完整Unity Editor通知系统，不能据此声称有全局notification center。 |
| Fyrox | core log有severity、relative time、listener、file/stdout、one-shot ID；UI有message box/progress primitives。 | sink分离、时间和listener基础可参考；one-shot需升级成明确coalesce policy。 | Fyrox这些文件不提供本文目标的完整notification journal，不能当工程上限。 |
| Bevy | diagnostics以`DiagnosticPath`标识，保存有界measurement history、enable状态、单位和统计查询。 | 对progress/metrics保留typed identity、history上限和enable policy。 | Bevy diagnostic measurement不是Editor user notification或decision系统。 |

结论不是选一个引擎照抄。Zircon需要组合：Unreal的typed notification/action与Message Log分层、Godot的真实入口和重复/hover/close语义、Unity Graphics的provider invalidation、Bevy的有界typed history，再由自己的owner lease、plugin capability和job authority完成跨域收敛。

## 8. 目标架构

```text
Domain Producer / Plugin Publisher / Job Authority / Diagnostic Provider
                              |
                              v
              NotificationAuthority (single writer contract)
      identity + owner lease + revision + sequence + context + policy
                 /              |               \
                v               v                v
       Live State Index   Bounded Journal   Decision Receipt Store
       progress/decision   query/cursor      workflow/idempotency
                \               |                /
                 +--------------+---------------+
                                |
                    Delivery Policy / Router
            toast | center | status summary | log | headless/OS
                                |
                 typed action + delivery/ack receipt
```

关键约束：

- journal record是事实，toast不是事实authority；UI关闭不能删除事实。
- status bar只显示摘要，不作为error唯一存储。
- decision是workflow，不是severity row；option action只能通过ticket/revision执行。
- progress从active转换为terminal record，不在finish瞬间消失。
- plugin只拿到scope-bound publisher和action capability，卸载会revoke live state。
- typed DTO一直到native host；不得在进程内编码成pipe string再解析。
- 所有action复用Command08的authorization、provenance、remote policy与audit，不把闭包跨ABI泄漏。

## 9. 分层重构路线

### M0 · 封闭当前P0，不扩展旧字符串协议

1. 修正`kind`覆盖severity，加入真实bridge -> parser -> native row全severity合同。
2. 将toast排队改为monotonic sequence + severity policy；queue wait不消耗visible lifetime。
3. fixed-ID duplicate必须coalesce count或append新incident；capacity/drop产生durable log与metric。
4. 为Workbench添加真实notification trigger；普通记录可打开，过期toast仍可从journal查看。
5. 删除无模型支持的`UNDO`/close绘制，或先补完整typed route再恢复视觉。

### M1 · 建立统一record、owner lease与bounded journal

1. 定义`NotificationRecordId/Key/Revision/Sequence/OwnerLease/Context/Policy`和versioned payload。
2. 收敛toast/progress/decision center为一个authority下的专用index，不保留平行身份空间。
3. journal支持entry/byte/age/per-owner/per-severity policy、cursor expiry、eviction reason和metrics。
4. 引入typed text、structured diagnostic tokens和redaction policy。

### M2 · Delivery router、history query与产品入口

1. toast/center/status/log/headless成为sink；每sink返回delivery receipt。
2. center实现badge、open/close、query/filter/search/group/page/read/dismiss/pin/mute。
3. Editor Log与notification journal共享diagnostic record，避免复制两份字符串authority。
4. window inactive、headless、commandlet和OS通知走显式policy，不静默丢弃。

### M3 · Decision workflow、typed actions与progress terminalization

1. decision保留group/default/cancel/destructive/timeout/owner generation，支持withdraw/revoke/supersede。
2. action绑定command ID + args + authorization + record revision，执行生成exactly-once receipt。
3. progress与Editor09 job control结合，显示phase/unit/ETA/cancel ack，并转terminal record。
4. dirty close、overwrite、migration、plugin reload等逐个迁移到通用workflow。

### M4 · Retained UI、键盘、读屏与规模化列表

1. typed DTO替换pipe arrays，windowed virtual list使用total/window/cursor而非`visible_limit`截断。
2. stable ID reconcile focus/selection；无关progress delta不能移动decision焦点。
3. 实现键盘打开、导航、action、dismiss、mark-read、copy、filter、Escape/cancel和focus restore。
4. accessibility tree提供localized role/name/description/modal/live region/severity/action state。

### M5 · 产品adoption、provider diagnostics与插件SDK

1. 对126个status-line文件分类：ephemeral status、durable diagnostic、user notification、decision。
2. save/import/build/export/shader/asset/plugin/play/recovery逐域接入typed record和jump/action。
3. graph/compiler/provider采用owner + subject + generation replace/invalidate模型。
4. plugin SDK提供scope publisher、quota/settings、action manifest和unload revoke；验证恶意/flood producer。

### M6 · 性能、恢复与工程验收

1. 压测百万事件、慢consumer、64+ progress、重复storm、plugin flood、locale切换和journal eviction。
2. 测量publish p50/p95/p99、UI delta latency、alloc、lock contention、paint rows和disk journal成本。
3. 故障注入journal partial write、UI sink crash、plugin unload、action timeout、cursor expiry和restart recovery。
4. Windows优先做真实Editor截图/键盘/UIA验证，再按Linux/macOS平台差异补齐。

## 10. 验收门

1. error toast在前方有任意数量/寿命toast时仍保证进入journal并最终获得可见delivery或明确suppressed receipt。
2. queue wait不减少visible lifetime；hover、keyboard focus和assistive interaction按policy暂停。
3. 相同incident聚合count/first/latest time；不同incident即使共享producer local key也不被静默吞掉。
4. capacity满时error有保留策略，所有drop/eviction可查询且系统metric递增。
5. `Info/Success/Warning/Error`从producer到native row保持精确，unknown kind不改变severity。
6. 普通toast/progress存在时通知入口可见、可打开、可键盘聚焦；无pending decision也可使用。
7. toast过期、progress完成、decision解决后，terminal record按policy留在history并可搜索。
8. center显示`total_count/window_count/evicted_count`一致；第9..64项不会无提示消失。
9. 10万记录查询只物化可见window；滚动稳定，无重复、跳项或focus丢失。
10. mark read/unread、dismiss、clear、pin、mute source与badge在多窗口/多surface下一致。
11. 没有action模型时不绘制action/close；有action时pointer、Enter/Space和读屏均可执行。
12. action校验record revision、owner lease、authorization与provenance；重复提交不重复副作用。
13. decision以一个semantic group呈现，default/cancel/destructive状态不在adapter中丢失。
14. Escape严格选择声明的cancel option；无cancel的mandatory decision不会伪装成可取消。
15. unrelated toast/progress更新不改变decision focus、selection或默认动作。
16. project close、document close、plugin unload能原子revoke对应pending decision和live progress。
17. receipt cursor过期返回可恢复状态；restart后不会猜测Apply/Discard或重复执行commit。
18. progress支持determinate/indeterminate、phase、unit、ETA confidence和cancel acknowledgement。
19. job terminal success/failure/cancel进入journal，包含耗时、输出/retry action和correlation ID。
20. 每条用户可见error在durable diagnostic store中有对应record，status line不再是唯一载体。
21. asset/file/line/node/job/plugin token可通过typed command跳转，失效target给出可解释结果。
22. provider generation刷新会移除已修复diagnostic，不残留旧badge，也不误删新generation。
23. plugin flood受per-owner entry/byte/rate quota限制，builtin error reserve不被耗尽；unload后live项撤回。
24. localized text与literal/redacted diagnostic分型；缺key、超长文本和敏感字段有测试。
25. Windows真实Editor通过鼠标、纯键盘、UI Automation/读屏和200%缩放验收；无重叠、截断或焦点陷阱。
26. benchmark记录publish/delta/paint/query的p50/p95/p99、alloc和high-water；阈值绑定source/build/hardware profile并进入required validation。

## 11. 与相邻报告的所有权

- Editor09拥有job admission、cancel acknowledgement、event journal和shutdown；本文拥有其用户可见progress/terminal/action投影。
- Editor08拥有command identity、authorization、provenance与remote policy；本文所有notification action必须复用，不另造字符串route。
- Editor06拥有plugin lifecycle/settings/manager；本文拥有scope-bound notification publisher、quota和unload revoke。
- Editor07拥有Play process/world/session；本文拥有pending-edit decision的通用workflow与可访问交互。
- Editor01与Runtime11A/11B/11C拥有retained/runtime UI、text、input和GPU painter底座；本文只定义notification产品语义和验收。
- 后续diagnostic/log专项需拥有`core/logging`的完整retention/export/query/telemetry；本文先明确notification不能替代durable log。

## 12. 本轮未实施内容

本轮只做review与计划记录，没有修改notification core、retained host、UI asset、tests、logging、jobs、plugin SDK或command routing。P0虽可由静态链路确定，但真实窗口视觉、键盘/读屏、压力、恢复和跨平台结果仍必须在实施阶段按验收门生成新证据。旧测试中把closed center、无history和固定`UNDO`当作成功的断言不能直接沿用；迁移时应先建立新typed合同，再硬切旧行为，避免为演示兼容保留双authority。
