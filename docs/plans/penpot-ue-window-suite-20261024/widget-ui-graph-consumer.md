# Widget Graph：预览事件与宿主消费者 r01

接续 Actor 的 [AB-WG-01～07](actor-blueprint-shared-contracts.md#widget-graph实际可复用边界与未接通项) 和 Parent 的 [WG-C07](../../../.codex/sessions/ue-window-suite-parent-01a16d4e.md)。本稿记录 Widget 所能交付的来源、已定位的断点和最小集成要求，不声明 Graph、外部 handler 或 Definition 已接通。真实文件仍是 `b4a1e3ce-9e6a-81fe-8008-d69d60d24223` / page `b4a1e3ce-9e6a-81fe-8008-d69d60d24224`。r01建立时保存重开为 revn9，原指纹、限定搜索和验证记录保留于 [source-boundary.json](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/widget-graph-consumer-r01/source-boundary.json)；后续实际设计进度与本次只读合同接续见文末，不以r01覆盖新阶段。

## 接 Actor Definition 合同的后续候选

接 [AB-DEF-01～08](actor-blueprint-shared-contracts.md)，本域已在原 checkout 补入 [binding 源定位候选 r01](widget-ui-binding-source-locator.md)：同一 TOML 语法树的原 binding/字段范围与实际声明 owner；current / last-valid 的范围 digest 和发起缓冲 revision 分开；源修改、undo 或重开后旧 ticket 不可用。crate 内 select API 核验后移动精确 byte cursor，不调用或编译脚本。12 项已写用例未编译/执行，未接实际 Events 行或宿主 SourceLocator。上文 r01 的历史盘点保留；本段不将节点块、Source 文本、route 或 VM callback 名称升级成 Graph/外部 Definition，G10 仍 open。

## 已定位的交接位置

| 位置 | 当前实际行为 | 消费者接续要求 |
| --- | --- | --- |
| [PreviewHost](../../../zircon_editor/src/ui/asset_editor/preview/preview_host.rs:192) | 验证当前 `Arc<UiSurfaceFrame>`，保留合法 captured continuation；真实分发、`rebuild_dirty` 后保存 `Arc<UiInputDispatchResult>`，返回 `bool`。失败不保证发布新 result。 | 当前输入对应的新结果必须在这次调用中取得，不能失败后重读此前 `last_input_result`。保持现有 capture、focus、IME 和 frame 验证。 |
| [session 入口](../../../zircon_editor/src/ui/asset_editor/session/designer_state.rs:15)、[host 入口](../../../zircon_editor/src/ui/host/asset_editor_sessions/editing/palette.rs:24)、[retained 调用者](../../../zircon_editor/src/ui/retained/app/asset_events/preview.rs:83) | session/host 传回 `Result<bool, ...>`；host 释放 sessions 锁后 sync；retained 刷新本域 presentation。该调用链没有取出 action 并调用通用消费者的步骤。 | Parent 在释放文档编辑及 sessions 锁后消费拥有所有权的当前结果；回执发布前再次确认文档/预览仍是同一实例。不要在持有 session 锁时调用可重入的宿主 handler。 |
| [presentation 发布](../../../zircon_editor/src/ui/asset_editor/session/presentation/pane.rs:211)、[canvas 转换](../../../zircon_editor/src/ui/retained/ui/data/pane_ui_asset_conversion/preview.rs:41) | 结果 Arc 被复制进 presentation，再移入 canvas DTO。这是供观察的最后输入结果，不是一次性动作队列。 | 视图刷新、重绘、切 tab 和重复读取不能重新执行。消费发生在实际输入事务的交接位置；来源修订和去重状态由原宿主持有，不新增一个反射快照轮询器。 |
| [原有通用消费者](../../../zircon_editor/src/ui/retained/dispatch/common/dispatch.rs:86) | Action 只接受空 payload，转 EditorCommand；Route 解析 `EditorOperationPath`，具名 payload 转 JSON object 后调用 `invoke_operation(UiBinding)`。当前 API 最终只返回 `UiHostEventEffects` 或字符串错误。 | 复用这个分支语义及真实 controller，但 Parent 需在压缩成 effects 前保留实际 `EditorEventRecord` 或原始失败分类。不能把空 effects、`Ok` 或 toast 当作完整执行回执。 |

通用 [pointer component callback](../../../zircon_editor/src/ui/retained/app/pane_surface_actions/click.rs:5) 确有 `dispatch_template_action` 消费，但 Widget 原生预览走 [retained pane input callback](../../../zircon_editor/src/ui/retained/app/callback_wiring/pane_surface/ui_asset.rs:13)，不能因前者存在就补写一条已接通的调用边。[GeneratedBottomPane](../../../zircon_editor/src/ui/runtime/runtime/runtime_host/generated_bottom_pane.rs:310) 也有真实消费者：依赖本 owner 的 canonical slot、node incarnation、event kind 和特定 MenuAction admission，不是任意 Widget binding 的通用解析器；不复制其 slot 表或反馈模拟来关闭 G10。

## 输入报告中的来源缺口

[UiPointerComponentEvent](../../../zircon_runtime_interface/src/ui/dispatch/pointer/component_event.rs:134) 已有 `binding_id`、可选 `compiled_binding`、`event_kind`、`reason` 及 envelope。Runtime 的 [pointer 汇总](../../../zircon_runtime/src/ui/surface/input/pointer.rs:145) 将它转为 [UiComponentEventReport](../../../zircon_runtime_interface/src/ui/dispatch/input/result.rs:254) 时只保留 target、event、delivered、drag 和 template_action，没有保留前述绑定来源。`UiTemplateActionInvocation` 本身只有目标种类/ID 与 payload，没有 binding ID 或 compiled generation。

[文本事件生产](../../../zircon_runtime/src/ui/surface/input/editable_text/mutation.rs:633) 对同类事件的每条 binding 生成 action 报告，同样没有把原 binding 身份放入报告。因此 target + event + action/route 字符串不是普遍唯一的反查键，两个相同目标/参数的 binding 也不能合并成一次执行。无 action 的普通组件通知继续作为通知；不猜测其应执行哪个 binding。

`binding_reports.execution_receipt` 不能补齐这项身份：[UiBindingExecutionReceipt](../../../zircon_runtime_interface/src/ui/binding/model/execution_receipt.rs:12) 的资产/绑定字符串由工厂限长并附哈希，明确仅供诊断；`execution_count` 描述 Runtime binding target 处理，不是外部 Editor handler 完成。既有 [UiBindingMutationReceipt](../../../zircon_runtime_interface/src/ui/binding/model/mutation_receipt.rs:15) 的 committed/rolled_back 描述本地属性事务，也不替代外部调用结果。

Runtime/Parent 的最小提案是在既有 action 生产到输入报告的同一路径保留可验证的 binding 来源，优先复用上述既有字段和 compiled program 来源解析；覆盖 pointer 与 text 等实际生产者。具体共享类型/序列化兼容性由原 owner 决定，本域未新增公共字段或第二个事件协议。没有 compiled handle 的生产路径必须给出经当前真实文档验证的原 binding/owner 关联；来源缺失、重复或过期时不执行，不通过 action 名、当前列表序号或首个匹配补全。

## Widget 与 Parent 的消费合同

以下是集成要求，不是声称已有的新结构体或接口。

| 合同 | Widget 域提供 / 校验 | Parent / 依赖 owner 接续 |
| --- | --- | --- |
| WG-E01 来源 | 当前 owning document/asset、source buffer revision/hash、last-valid generation；真实 source_path/source_node_id/instance_path，原 binding ID/event/component_event、target/property/payload。owning component 从真实 asset root/components 与 invocation ancestry 解析；metadata.component 只是控件类型。 | 保留 owning 文档与当前 registry/context；不把 runtime UiNodeId、dense binding ID 或 source_binding_index 当持久 graph 身份。compiled 来源仍按 AB-WG-02 检查 generation。 |
| WG-E02 单次输入 | 原 `UiInputDispatchResult.event` 的真实 window/surface/device/sequence 等 metadata；这次调用的结果、报告位置及当前 preview/Surface incarnation。来源 metadata 可选，不单靠全局 sequence 去重。 | 同一输入结果/报告只消费一次；先记录已开始再允许重入。不同报告即便 action/payload 相同也保留。presentation Arc 的反复发布不触发执行。新来源请求必须经过现有 frame/capture 准入，不接受对旧结果任意补发。 |
| WG-E03 目标与参数 | 保留原 Route/Action 分类和具名 typed payload，不自动更名。原编辑器 chrome 的六个本地 route（如 save/undo）属于其原适配入口，不因此变成三段 Editor operation。 | 使用当前真实 registry 和 handler readiness；Action 非空 payload 按现有消费合同拒绝。Route JSON object 与 remote UiBinding JSON array 分开，不能按字符串形似选 handler。VM 三段且保留大小写、Editor 至少三段且仅小写 ASCII/数字/下划线的差异由 Parent/VM owner 统一。 |
| WG-E04 执行与回执 | 提交当前输入与源绑定关联；本地控件状态/属性写入的结果原样保留。 | Parent 在实际 controller/handler 边界关联 `EditorEventRecord` 或 `UiInvocationResult` 及原始错误，区分 registry 拒绝、执行失败和实际完成；只投影适用于该分支的结果，不能为了统一外观伪造另一分支回执。失败记录的 before/after revision 不证明提交，实际 undo 仍归原事务引擎。 |
| WG-E05 锁、代际与取消 | 文档修改/切换、PreviewHost 重建、工具模式或窗口生命周期变化使旧交接退休；保留既有 captured pointer continuation 的合法边界。 | 不在 session 锁或文档编辑 guard 内执行可重入操作。每个 action 执行前重新核对 owner/context/registry；前一 action 可能关闭文档或重建预览，不能直接继续整批。旧结果不写入新 preview。已发生外部副作用不能因结果过期而假称回滚；不支持取消的操作如实报告。 |
| WG-E06 Definition | 精确 binding 声明范围与实际owner已在[原source index/TOML定位候选](widget-ui-binding-source-locator.md)实现，12项回归未执行；Current ticket核同一buffer allocation/revision、route、last-valid generation及完整原binding，成功才选节点/binding并移动byte cursor。LastValid只读，当前脏源不能用旧offset冒充当前位置。 | Parent 在原实例/编辑guard内消费受检ticket，解锁后同步Details/Source并按当前导航generation请求焦点；当前没有宿主调用边。跨文档SourceLocator/外部Definition和真实Graph映射仍缺，Actor/VM提供callable/member/node/pin来源，Source页面不改名Graph。 |

[UiEventManager invocation](../../../zircon_runtime/src/ui/event_ui/manager/invocation.rs:159) 中 UnknownRoute、无 handler 等早退不保证广播；同步返回的 `UiInvocationResult.error` 必须被消费。`SubscribeDiffs` 仍有 `CR-UI-COMP-0002`：只有 subscription ID、receiver 被丢弃，不能只等该通知完成闭环。`delivered=true` 只代表组件报告；[effect 生产者](../../../zircon_runtime/src/ui/surface/input/effect/component_event.rs:15) 的 Queue/Coalesce 行为还保留 `CR-UI-SURFACE-0003`，本稿未修改或关闭它。

## 最小实施归属与验收输入

Widget 后续改动限定 `preview/preview_host.rs`、`session/designer_state.rs` 及其源定位缓存/域测试：提供同次输入的拥有结果和当前源关联，保留现有对外行为；精确签名要等共享来源字段和 Parent consumer 确认，避免先造一份无法兑现的来源对象。Parent 接 `host/asset_editor_sessions/editing/palette.rs`、`retained/app/asset_events/preview.rs` 和通用 dispatch/record 投影；Runtime owner 接既有输入报告/生产者的来源传递，Actor/VM 接真实 callable 和 Definition。此处只给出精确接入路径，没有向其它域授予 Widget 源文件写权。

后续有界回归应在原托管请求完成协调后声明范围，再分别证明：同一次点击的一条合法 handler 完成；同一结果再次投影不执行；相同 action 的两条不同 binding 不丢失或串接；重复组件实例对应正确来源；非空 Action payload/未知 Route/无 handler 原样失败；handler 重入导致切文档后余下旧请求退休；本地 target mutation 不生成外部成功；文本 Change/Commit 各自关联原 binding；Source 脏源/撤销/重建后旧定位拒绝。它们是待执行用例，不是已写或已跑测试。

原r01只做源码与文档结构检查，不触发插件或Rust编译，不重复提交原Jenkins identity，不启停服务或占桌面。后续精确binding源位置已形成上文候选；同次输入的真实binding来源、宿主单次执行/回执与定位焦点仍待公共owner接续。G10、W11/W12/W20/W21、正常EXE、150% DPI和双端比较仍open；passed/accepted均为0。

## 接材质 UI-SH-GRAPH-01 的源码与输入边界

本次直接消费[共享Graph原合同的Widget段](shared-node-graph-contract.md#widget-接续canonical-源版本和组件身份)及材质`widget-shared-graph-contract-04.json` / `widget-shared-graph-public-proposal-09.json`，不新增模型、合同文件或组件。本轮编辑前独立核对30条来源指纹，29条匹配；差异仅为Widget设计文档的历史hash，当前值与本域上一轮Palette候选一致，不回写材质回执或归责写入者。本轮随后依授权更新本消费者文档，另观察到AI两处重导出文件的并行变化，四项差异逐项保留。10个Parent/原pipeline逐文件提案在初读时匹配，收尾catalog发生并行变化，不能将初读结果继续称为全部当前一致；6个canonical Graph源保持原hash，原proof/prepared、retention/revision及一次permit不变。[独立对账](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/graph-consumer-reconcile-r02/reconciliation.json)记录完整身份、时点和差异。

收尾所读catalog SHA为`4c2f849c533f1f227903018eca860faadd7fa3138fdc265300913e3833ad36e5`，先前为`357e63b2…05cb`；当前定义含PropertyEditorRow的description/source/reset可选slots及NumberField参数，value仍接受原七类typed控件。本域未编辑或归责公共写入，也未据catalog元数据宣称对应源组件、插件发布或Widget实例已适配。正式source/library/typed reset与导出仍沿Parent/原pipeline交付。

最近本域实际Penpot保存重开是rev13/2978objects/8local mains/12Shared copies，见[搜索flow七图](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/penpot-search-shared-r01/review.md)。材质合同中的rev12和Parent rev9保留为各自历史观察；本轮没有API读取，不增加revision、库发布或Graph设计验收。原v2资产仍为97nodes/97唯一control IDs/6events，现catalog只有域用AnimationGraphPaneBody，未有中立GraphSurface/Node/Pin的正式wrapper或公开UUID。

### 已有宿主基础与未交付的Graph来源

| 已有实际入口 | 本次核实的用途及接入边界 |
| --- | --- |
| [binding_source_navigation.rs:51](../../../zircon_editor/src/ui/asset_editor/session/binding_source_navigation.rs:51)、[选择入口:91](../../../zircon_editor/src/ui/asset_editor/session/binding_source_navigation.rs:91) | crate内只读ticket/private字段；Current核验后只改节点/binding选择和精确cursor。Parent保持原document/instance编辑guard，解锁后同步Details/Source并请求焦点；不执行handler、不增加第二history。LastValid/stale/reopen拒绝。真实graph node/port→binding映射和resource URI provider尚缺，不能猜route.asset_id为URI。 |
| [UiAssetEditorPaneData](../../../zircon_editor/src/ui/retained/contract/data/ui_asset/pane/editor.rs:16)、[原pane输入callback](../../../zircon_editor/src/ui/retained/app/callback_wiring/pane_surface/ui_asset.rs:12) | DTO当前只有原Source/Preview/Details等投影；输入分支只有GeneratedBottom和UI Preview。它们不是已安装的Widget Graph capability；保持原真实输入，待Parent提供合法graph body/source frame后才扩展。 |
| [draw_runtime_surface_frame](../../../zircon_editor/src/ui/retained/contract/paint/render_commands.rs:39) | 真实通用绘制基础：消费发布的UiSurfaceFrame、原render command、origin/scale/clip，保留同一source Arc及command ref。它没有Graph node/pin/edge geometry生产者或Widget Graph调用链；找到此函数不等于已指定中立Graph painter owner。 |
| [hit_test_host_surface_frame](../../../zircon_editor/src/ui/retained/contract/surface_hit_test/surface_frame.rs:10)、[原retained输入](../../../zircon_editor/src/ui/retained/contract/window/retained_pane_input.rs:27) | 命中依赖真实hit_grid/control_id，retained输入保留可见来源、clip及captured continuation，再逆现有preview映射。不能从装饰wire或旧preview frame伪造Graph命中。 |
| [build_template_surface_frame](../../../zircon_editor/src/ui/retained/contract/node/surface_frame_builder/surface.rs:23) | 从既有TemplatePaneNodeData建通用模板hit frame；没有Graph稳定pin/edge、同帧测量及域编辑适配，不作为Graph geometry producer替身。 |

在`zircon_editor/src/ui/retained`、`zircon_runtime/src/ui`、`zircon_runtime_interface/src/ui`限定Rust检索中，GraphModel/GraphEditContext/GraphCanvasState/GraphNodePresentation/GraphAttachmentPresentation及route_connection_with_directions均0命中。此结论仅限列明路径和符号，不能推断整个仓库没有其它渲染实现。AI原adapter的route_edge确实调用中立方向路由，其当前SHA仍为335a5cca…a9ce9；对应4条路由调用测试只有源码证据，没有生产retained caller、paint/hit或运行图像。没有借AI整树candidate绕过retention/permit，也没有借MaterialAsset/compiler当Widget后端。原始命令和结果见[限定搜索](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/widget-ui/graph-consumer-reconcile-r02/source-boundary.json)。

### 坐标必须沿实际来源核定

本次沿真实调用追到[Runtime winit pointer转换](../../../zircon_runtime/src/ui/platform_input/winit_translation/pointer.rs:163)：PhysicalPosition仅窄化为UiPoint，没有DPI除法；[窗口CursorMoved转换](../../../zircon_runtime_interface/src/ui/window/input/window_event.rs:17)保留原位置，[normalize](../../../zircon_runtime_interface/src/ui/window/input/normalization.rs:22)也不换算坐标。Editor虽提供window metrics，[platform_pointer_input](../../../zircon_editor/src/ui/retained/contract/window/event_loop/platform_input.rs:113)并未据此换算，正常[事件循环](../../../zircon_editor/src/ui/retained/contract/window/event_loop/events/pointer.rs:48)把该point交给retained pane。因此`normalized`不能当作“输入已是pane-local DIP”的证据。

既有[Preview transform](../../../zircon_editor/src/ui/retained/contract/data/ui_asset/canvas.rs:99)只按pane/source尺寸算fit scale；[retained输入:105](../../../zircon_editor/src/ui/retained/contract/window/retained_pane_input.rs:105)逆origin/fit scale。这个scale不自动等于window DPI。Parent应由实际native source/geometry owner明确window物理点→pane DIP→graph point的唯一换算位置、源frame及transform generation；已是DIP的下游事件不再除DPI，物理入口也不从graph zoom猜DPI。本轮不修改公共输入或宣称当前Preview存在DPI故障；必须结合真实frame单位、绘制和实际150% DPI验证后判定。

Parent下一项所需交付是中立Graph native/geometry的实际owner与具体原路径、真实Widget/Actor业务图后端及同源映射；Widget随后在原session/history接域delta/inverse和checked locator。公共10路径仍是未应用提案，当前Rust/产品资产/公共文件0写入。本次只有两份本域文档接续，未编译、未插件导出、未操作Penpot/桌面/服务；W11/W12/W20与全族继续open。
