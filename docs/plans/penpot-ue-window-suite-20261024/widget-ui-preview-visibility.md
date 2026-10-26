# Widget 受控可见性预览候选 r01

接续 Parent [WG-C01～08](../../../.codex/sessions/ue-window-suite-parent-01a16d4e.md) 和 [动画消费者合同](widget-ui-animation-consumer.md)。原r01在实际checkout写入13个本域Rust文件：6个既有文件的增量、7个新模块/测试文件。2026-10-24原owner对账后的[r02闭包刷新](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/preview-visibility-domain-r01/revision-r02/summary.md)只读核对现源，未修改Rust：两索引的差异精确归属后续同域binding定位登记，其余11源码、22依赖和4保护资产保持。新patch仅两处hunk行号变化，行为和11项未编译测试均未变；原patch/失败完整保留。调用生产 `UiSurface::mutate_property`，未修改共享 Runtime、v2 schema、产品 ZUI、主题、插件或 host。没有 GUI 入口、动画轨道、时钟或播放器，产品 passed/accepted 均为 0。

原源码、增量patch和检查回执登记在[r01历史产物](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/preview-visibility-domain-r01/source-candidate.json)，其SHA03616d65…patch未改；r02源闭包与归属见[r02候选](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/preview-visibility-domain-r01/revision-r02/source-candidate.json)。旧Canvas/binding改动和旧失败保留。此候选推进WG-C01/C04/C05/C06的一个允许集，不关闭W09/W10或整个窗口族。

当前接续为[sampling-r03](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/preview-visibility-domain-r01/sampling-r03/summary.md)：在原五个域内文件消费Parent已落地的公开Bool/Step采样器，并提供受检的保留当前值/结束lease入口；原11项测试不改，新增6项，共17项未编译、未运行。调用者传入的秒值没有被确认成实际clock，finish也不等同Stop。没有GUI/transport接线或present证据。

本域消费版本仍为动画owner原rev105/Transport draft2；后续r06已交105→140→146→147→148的既有保存/重开证据索引，未将这些后续owner copies升级为Widget消费。原三main/六事件保持，clock/Stop/Seek/consumer slot和正式发布仍未交付。本域已在[原动画消费者记录](widget-ui-animation-consumer.md#动画rev105接续与实际隐藏链)定位正常host隐藏链：成功布局发布后retire_hidden_retained_pane_inputs→原host/session deactivate→PreviewHost当前frame的Focused(false)；dispatch先退休lease。具体close要求恢复/保值时，须在导致hide/源替换的布局发布及deactivate之前，消费同一有效gesture的原Cancel完整Outcome或finish现frame，再进入原输入退休链；普通hide不统一Cancel/Stop。拒绝旧源/lease，不在停用后关闭守卫补旧baseline。版本来源及四类回执分离见[原r06接续](widget-ui-animation-consumer.md#动画r06回执与原消费版本)；实际clock/关闭原因/focus/capture/IME/popup和present仍由Parent接通，本次未改Rust或产品资产。

## 实际入口及允许范围

[Session 入口](../../../zircon_editor/src/ui/asset_editor/session/preview_visibility_state.rs) 仅接当前有效 v2 View。根必须为原生 Panel，目标为直接子级、非空文本的原生 Label 叶节点，只允许 `props.visible: bool`。整个文档必须无 imports、组件定义、stylesheet/token、class/inline style、bindings/events、params/slots/state、repeat 或其它属性；Host 还验证目标没有交互、局部化或运行时绑定。布局保持现有引擎实现，不把 layout/style/slot/任意 metadata 属性加入允许集。

| 入口 | 实际行为 |
| --- | --- |
| `begin_preview_visibility(source_node_id)` | 从当前 Session 生成 source/context，唯一解析当前原生节点并保存值、原 visible attribute 是否存在、当前实例证明及实际 frame；返回不可构造的 gesture lease |
| `apply_preview_visibility(gesture, frame, sequence, visible)` | 每次写前复核 source、gesture、当前 frame、严格递增 sequence、节点/根 incarnation、值及 override presence；通过生产单属性事务写入 |
| `sample_preview_visibility(gesture, frame, sequence, channel, requested_seconds)` | 由已有公开Bool/Step采样器查询，再走原apply；返回请求秒值、采样bool及完整实际事务结果，外层Ok不等于属性写入或发布成功 |
| `finish_preview_visibility(gesture, frame, sequence)` | 按同一准入校验结束当前有效lease，保留现值，返回现有frame；不写属性、history或新发布，不恢复跨手势动画基线 |
| `cancel_preview_visibility(gesture, frame, sequence)` | 只恢复仍有效的同一手势起点；保留原 attribute 的存在/缺失，成功后退休该 gesture |
| `reset_preview_visibility()` | 先退休旧手势，再由当前有效源码通过既有 v2 builder 重建原 PreviewHost，返回真实新 frame；builtin、无 preview、无效或超出允许集的源明确失败 |

无匹配、多个匹配、组件 invocation/repeat、未知类型、builtin preview、无效源或已关闭窗口均拒绝，不回退同名首节点。其它原生控件、任意组件、交互/焦点/弹层、实例 override 与 repeat stable key 仍未适配。该属性探针不变更源 buffer、dirty、文档 history，也不创建可保存动画。

## 源身份与 Host 所有权

目标由真实 route URI、document asset ID、source revision、last-valid generation、root/source node key 和显式空 `UiTemplateNodeInstancePathStep` 组成。当前 native-only 允许集不需要推测 component ancestry；非空 ancestry 拒绝，repeat 未提供 stable item key 也不接入。编译后 UiNodeId/incarnation 只作当前实例校验，不写入持久 binding。

源码检查发现原 [v2 preview builder](../../../zircon_editor/src/ui/asset_editor/session/v2_authoring.rs) 只用 `PrototypeStore::insert`，不会登记 root source path。因此本轮让现有五个 lifecycle/Palette 调用和新 Reset 显式传入各自 owning route URI，并使用已有 `insert_with_source_path`。没有事后补写 tree metadata；import aliases 维持原逻辑，未知 imported source path 仍未知。此修改影响现有原生 v2 preview 的来源投影，需托管回归，不能仅凭静态检查称兼容已通过。

[Host state/target](../../../zircon_editor/src/ui/asset_editor/preview/visibility/target.rs) 只存一个由该 Host 创建的 `Arc<()>` gesture lease；它是一次请求所有权凭证，不是另一套全局 Surface identity。Host 构造/重建不继承 lease，Clone 显式清空它，即使克隆仍共享 cached frame Arc；实际 resize 和通过原 frame 准入的输入也退休旧手势。写入口限制在 `asset_editor` 域，未开放 `cfg(test) surface_mut()`。

每次请求还比较 actual current frame Arc、根和节点 incarnation，以及预期 visible/attribute presence。重复 sequence、旧 request frame 或非本 Host lease 不会退休较新的合法请求；源、目标或实际 frame 发生变化则拒绝并退休对应手势。旧回调不能清除新 lease。source edit/undo、preset、Palette/builtin、Reset、Host Clone 或切文档均不能把旧基线恢复进新实例。

旧 [公共 identity 出口提案](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/preview-property-adapter-r01.md) 和三文件可见性 patch 仍未应用。此窄候选不依赖该出口；更一般的共享消费者仍由 Parent 决定，未借 lease 宣称公共 Surface 身份合同已完成。

## 真实回执与失败边界

[生产适配](../../../zircon_editor/src/ui/asset_editor/preview/visibility/transaction.rs) 保留原 `UiPropertyMutationReport`，不把 `Ok(report)` 当成功。Accepted/Unchanged 还核对 node/property/source、binding 状态与前后 bool 值、失效域和实际当前属性/presence；Accepted 后才请求 `rebuild_dirty` 并取得实际发布的 frame。Rejected、外层 Runtime 错误、发布错误、回执/状态不符分别返回；外层错误或 Accepted 后发布失败均不声称回滚或无副作用。

`Reported { report, publication }` 中 publication 分为 `NotRequested`、`Published(actual_frame)`、`Failed(error)`、`VerificationFailed(actual_frame)`。Published 只证明代码取得 Surface 发布帧，仍需实际执行及 renderer present 证据。单属性请求没有多属性原子语义；未实现 setter 循环、反向写值补偿或新的 runtime。

Cancel 在 source/session/request 仍有效时恢复基线；若输入先退休 lease，保留输入后的当前状态，要求显式 Reset 当前源，不能用旧 Cancel 覆盖。Reset 失败时保留当前 Host 和真实错误，旧 lease 已退休。Pause、Stop、restore-state、动画 tab/drawer 和关闭后的焦点逻辑继续遵循 Parent/动画合同，候选没有为它们伪造执行结果。

## 验证范围及待交付项

已编写的 11 项测试使用真实 v2 compiler、Session、Surface 和 frame：可见性/arranged/render extract 改变；显式/缺失 attribute 的 Cancel；Unchanged；旧 frame/重复 sequence；相同 URI 的另一 Session；source edit/undo；Reset 数字 generation 重现；preset/无效源；交互/绑定拒绝；真实 window-event dispatcher 退休；以及共享 cached frame 的 Host Clone 拒绝旧 lease。测试不是桌面输入或正常 EXE 证据。Runtime Rejected、发布失败与 renderer/readback 尚无实测覆盖。

后续最小托管输入为 `zircon_editor` lib、defaults off、features=[]、test profile、jobs=1、threads=1，filter `preview_visibility` 当前覆盖Session的16项和私有Clone一项。原r01/r02的11项测试及回执保留为历史。**本轮未提交、未启动 Cargo**；不改原layout/canvas两个request/operation的身份或过滤器，交原Jenkins owner追加明确修订/operation并核对source capture、准入和锁，执行前声明精确命令。

Parent 已明确既有域内最小 Jenkins 权限持续有效，无需再申请。14:13Z 原两份身份的只读查询仍在管理 `authorize` 前置返回 pending，公开服务 capability 同时为 ready；这是当次工具上下文，未证明用户授权缺失或已完成服务端收件。当次13个候选源码及四份保护资产无漂移的结果保留为历史，后来binding登记引入的两处索引变化由下述r02接续；未回填旧回执或修改identity/filter。原操作恢复和新增visible测试分别记账，见[历史接续回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/managed-validation-resume-r02/supported-route-review.json)。

## 原owner对账后的候选修订r02

本机核验原owner [reconciliation.json](E:/cargo-targets/zircon-local/reports/jenkins-transition-monitor/operator/widget-original-validation-20261024/reconciliation.json) SHA `9c71d77a1f9ca3efb8b5f82febc4e18cc59a611179297ca36ee17dbfcf44b7d2`及[摘要](E:/cargo-targets/zircon-local/reports/jenkins-transition-monitor/operator/widget-original-validation-20261024/summary.md) SHA `61621c058c44cd72196474419a3c81d3b44283e73c108a9fadf42fd23ac036ea`。该15:13Z只读观测及本域15:38ZQueryOnly各保留时点；没有据此创建新context、server receipt或生命周期终态。本轮未查询/捕获/投递/编译、未改grants或任何锁。

`session/mod.rs`的`mod binding_source_navigation`和`UiAssetBindingSourceLocation`导出，以及测试索引的`mod binding_source_locator`，都精确匹配同会话后续binding候选的before/after。仅在内存扣除这三个片段即恢复r01两个post哈希；在内存逆向原preview patch并再正向验证13文件，也精确恢复原字节。CRLF/LF分别记录，只有unified patch表示为LF，源码没有换行改写或副本。当前r02 patch SHA `9cdcbcfa41e1c04ca1bd39d44acaf89acc956a9cdfaa3d2b6df982d93630d95e`，50152B/13路径，只把两处hunk起点40→41、12→13；原Canvas、binding与visible注册全部保持，行为增量不变。详见[索引归属](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/preview-visibility-domain-r01/revision-r02/module-attribution.json)。

[离线验证意图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/preview-visibility-domain-r01/revision-r02/validation-intent.json)明确新增operation提案`validate-editor-preview-visibility-01`、候选revision r02，filter `preview_visibility`按字面模块边覆盖原11项测试。它不是identity/request/server回执，requestId/attemptId为空，未创建或提交。后续最小范围仍为zircon_editor/defaults off/features=[]/lib/test profile/jobs1/threads1/locked；执行前再声明实际受管命令。Layout/Canvas原两request、operation、identity、filter及ownedPaths不变，不能把这13路径和11项测试塞入旧范围。

原owner未找到两identity的自动intake/operation/route/alias/marker及本session repo aliases授权投影；原客户端在POST前source capture退出，正式自动context尚未由真实intake证明，不能resume-original或要求用户手填grant，也不由此推断所有外部历史构建不存在。E空间原15:13Z观测17671135232B、35GiB保留线37580963840B，未计recipe已差19909828608B；不是本轮资源预留。先经已有Jenkins维护/资源准入，取得target/pool/cache-writer和真实intake后，再按实际capture-ready协调有限窗口。当前captureReady=false、interval=null，无freeze，无已证明的非编译capture-only接续。

r02的13路径source guard为0error/0warning/0suppression，22只读依赖、14个binding上下文指纹和4保护资产均匹配；格式证据按相同源码分别复用原11源码和后续binding两索引的实际回执。11项Rust仍0执行，未补造GUI、动画采样/呈现、EXE或passed/accepted。两次内存审计的CRLF和重复constructor上下文断言失败已保留，修正为逐文件换行及精确hunk坐标/上下文校验，无源码写入；完整[r02摘要](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/preview-visibility-domain-r01/revision-r02/summary.md)供原owner接续。

上述r02时点仍等待sampler出口；Parent后来实际落地出口，当前接续如下。WG01/02的持久轨道、真实clock及采样到present、WG05的实际动画历史、G10的宿主完成和Definition、共享组件/样式与完整ZUI导出均open。三尺寸、实际DPI144、Penpot/产品并排差异、逐图复核和≤1 DIP目标均未取得新通过证据。

## 已有sampler消费与显式lease结束候选

Parent的[动画第三次接续](../../../.codex/sessions/ue-window-suite-parent-01a16d4e.md)已落实 `zircon_runtime::animation::sample_animation_bool_channel(&AnimationChannelAsset, Real)->Option<bool>`。[公开函数](../../../zircon_runtime/src/animation/sequence/channel_sample.rs)只接受非空Bool/Step、无切线、有限非负且严格递增key时间，以及有限采样时间；通过后委托原求值器。原端点clamp及精确内部key仍取前一区间的行为保留，未称UE边界一致。Editor既有target-editor-host已启用animation，本域未改Runtime或Cargo。

Session新增sample入口先核当前源，调用该函数，再走原apply准入/生产属性事务。结果保存 `requested_seconds`、`sampled_visible` 及完整 `UiPreviewVisibilityOutcome`；调用者仍须核Rejected、RuntimeFailed、报告status/binding/invalidation和publication。外层Ok、Accepted和Published都不代表时钟确认或renderer present。当前没有接通的Widget clock/scrub秒回读；TimelineModel的playhead与场景AnimationEditor的frame/FPS也不能代作该回执。候选不启动clock、不存轨道、不改源history，不新增插值器或播放器。

finish先复核同一source/session、lease、frame、incarnation、原值/attribute presence及递增sequence，才退休该lease并返回现有frame。它不写属性、不生成新frame，不做一次文档commit，也不恢复动画pre-animated state。原二次begin会GestureInProgress；finish后可新begin，但新的Cancel起点是该次实际现值，原手势基线不跨finish保存。Stop/Seek及restore-state策略仍由Parent/动画/Widget沿原合同集成。

焦点处理仍依现有host/输入owner决定：先让IME、popup及共享capture处理事件；只有实际归属Widget手势的取消才可在deactivate之前调用有效Cancel。若blur/deactivate/resize/Clone先退休lease，旧Cancel/finish必须拒绝；恢复只能显式Reset当前有效源，不能自动向新实例恢复旧值。候选没有重排OS输入、绑定ESC或把Reset接成Stop，focus-return和present仍须真实宿主验证。

新增六项回归覆盖公开Step边界和真实Surface值/frame、非法channel/非有限秒不消耗有效请求、finish保值及重开/旧完成拒绝、旧frame/sequence、切源后的旧sample/finish，以及Cancel与真实deactivate dispatcher的先后顺序。原十项Session测试字节前缀和私有Clone测试保持，共17项仍0执行。五文件格式检查及静态change-guard为0错误/警告；这不是编译或GUI通过。增量patch、原身份/源闭包和未投递验证意图见[sampling-r03候选](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/preview-visibility-domain-r01/sampling-r03/candidate.json)。原97control IDs/6events及四保护资产未改。

## r05只读复核与未执行验证

动画owner已读回原sampling-r03实现，见[原回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024-animation/widget-consumer-r05.json)。本域复核五个增量、十只读输入与原17261B补丁均匹配；去重37份源/资产对旧candidate仍33匹配。相较r05当前36/37匹配，新增时点差异仅公共catalog，保留当前内容及全部旧指纹。精确哈希、原两验证identity和原资产97IDs/6events检查见[本域回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/animation-transport-rev105-r05/owner-r05-reconciliation.json)，不据此刷新或重投旧identity。

现sample返回的requested_seconds是请求输入，sampled_visible是纯查询结果；外层Ok、属性Accepted、内PreviewHost Published、外retained发布和renderer present分别确认。finish不产生属性写入/history/新publication，仅保值退休有效lease；Cancel/Reset与deactivate前后的边界沿原实现，不隐式充当Stop。唯一真实clock→受检秒→Session及实际呈现仍未接通，完整消费次序留在[原动画消费者合同](widget-ui-animation-consumer.md#动画r05回执与当前闭包)。

17项Widget测试源码及3项公共Bool sampler测试源码存在，执行数均0。本次是原Session合同的静态接续，离线验证意图的request/attempt/serverReceipt保持null；未调用QueryOnly/submit、未编译或提交新operation，不以静态通过关闭产品门槛。正常EXE、实际144DPI、双端逐图及≤1DIP仍pending，整族passed/accepted仍false。
