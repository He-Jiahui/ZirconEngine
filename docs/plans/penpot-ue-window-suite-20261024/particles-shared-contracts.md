# 粒子域对共享组件的消费合同

本聊天 `01a16d52-a4a5-7271-90a2-336091337737`。依据 [Parent交接](../../../.codex/sessions/ue-window-suite-parent-01a16d4e.md)、[本域设计](particles-design.md)、[动画提案](animation-design.md)和 [MAT-SHARED-001](material-family.md)。本域现rev38，11个原生linked caption已消费Shared body/strong，Update和fresh重开均有实际回执，106 source refs保持。Shared现rev2/4基础main/34颜色/3文字样式；master有效ref仍0，typed输入/Graph公共main及产品绑定待验。旧服务拒绝/rev1消费记录保留历史。

## 归属和实际接入状态

| 负责方 | 本轮消费边界 |
| --- | --- |
| 动画 `01a16d51-741b-73b3-a61a-2189cd38f81e` | 共享timeline、curve、transport的可编辑设计、命中/变换/输入/取消、组件身份及合同；原 `WorkbenchTimelineStrip`、`WorkbenchTransportControls` 和 `ui/timeline`、`ui/curve` 为复用基础 |
| 材质 `01a16d50-6c7f-7370-853d-4581184b66a1` | 共享graph surface，复用 `ui/graph`；粒子消费节点位置、selection、pan/zoom、框选、typed属性与domain delta，不把Niagara schema塞入材质域 |
| Parent `01a16d4e-0491-7353-84fa-038bd7706546` | 公共catalog/tokens/Starship、窗口/Dock/菜单/属性行、跨域输入及事件路由、生产viewport/media、Penpot导出和原生证据集成 |
| 粒子本域 | System/Emitter/VFX源DTO/codec与唯一authority、稳定子项key、typed mutation/undo/save、实际粒子clock/preview/renderer适配、可用性和诊断 |

本域原产品路径仍是 `zircon_plugins/particles/editor/{authoring,preview,particle_system.drawer}.zui` 和 `editor/src`。Runtime若需要改动，先给出 `runtime/src/{asset,component,service,simulation/cpu}.rs` 的具体DTO/执行范围；本合同不独占或提前修改这些公共接口。共享库、图、时间轴、曲线、transport、preview_scene、binding host、keymap和导出源码均未改。

真实Penpot文件是 `b4a1e3ce-9e6a-81fe-8008-d69d60c876ee`，初始页 `b4a1e3ce-9e6a-81fe-8008-d69d60c876ef`。最初四文件仅页根，以及随后9个flow板/业务Emitter summary主件3实例的重开核验，均保留为历史；之后原页已达18板/7业务实例。最新revn19另有4个Shared rev1 copy，见 [当前消费身份](particles-design.md#shared-rev1原生消费者身份与逐图复核)。它们限于验证页，不代表完整body或共享graph/curve/timeline/transport/数值输入/样式已接入，不能从catalog名称推定未发布的Penpot身份。

## 共同的输入与反馈

沿现有文档、history、jobs及原生证据承载以下信息，不另建全局store或第二份资产。下列字段是需求概念，实际schema由对应owner沿既有类型确定：

| 输入/反馈 | 粒子提供 | 公共surface/host须保留 |
| --- | --- | --- |
| identity | project/session、AssetUri、DocumentId、source revision、preview owner/generation、active emitter/curve/node/key | 多文档隔离；原ZUI asset/node/control/slot与Penpot实例身份可回溯；关闭或换文档后的旧反馈不可覆盖新owner |
| display/value | 已本地化caption、用户原名、typed finite值、单位、真实范围、disabled reason和source location | 文本测量/ellipsis/tooltip/换行；binding不把数值或u64转换为展示字符串后丢失语义 |
| input | command eligibility、目标key、expected revision、draft gesture/cancel/commit边界 | pointer与keyboard走同一domain action；文本/IME/弹层优先；右键命中对象与当前selection分清 |
| result | unchanged/rejected、committed source revision、compile/installed/rendered revision、durable saved revision及真实diagnostic | command入队、compile成功、frame安装、保存完成分别显示；pending或last-good不能标current success |
| native review | 原始sourceRef/instancePath、实际资源及EXE身份、真实frame generation/nonce、pane逻辑坐标和文本行框 | 复用LayoutReviewCase/normal-frame协议；DPI仅应用一次；设计fixture和产品媒体来源不能混用 |

## 给动画owner：时间轴和播放

已读 `zircon_editor/src/ui/timeline/{model,ruler,keyframe_lane,track_list}.rs`。`TimelineModel`已有descriptor/catalog、range、playhead、tracks和 `apply(delta) -> inverse`；`TimelineKey`/`TimelineSection`/selection已有稳定字符串身份。粒子实现domain adapter，普通控件不复制粒子asset。

| 需求 | 粒子适配 | 动画owner实际所需合同 |
| --- | --- | --- |
| 时间语义 | System preview时钟使用秒；view range来自会话预览范围，当前asset无system duration | 显式区分view range/play range/asset duration；不能把strip默认duration1写回source；display FPS只是显示/步进设置，不成为源asset FPS |
| emitter tracks | 真实emitter stable key/name；burst key为源burst time_seconds/count | 显示实际track/key和选择；burst拖动一次事务、按真实秒snap；缺duration/section模型时不制造持续条带/section属性 |
| playhead | 仅来自成功执行的preview steps/age及当前generation | Seek请求与已完成playhead分开；pending/取消/失败不先显示目标时间已经运行；不以UI wall-clock自动推进 |
| transport | Play/Pause/Stop/ResetSimulation、受控seek/rewind、可用时fixed-step前进 | 动作资格/名称/tooltips由domain提供；Stop与Pause不同，Reset与Previous不同；record/loop/backward不能只换icon后宣称可用 |
| view state | per-document/emitter选择、scroll/pan/zoom、显式显示单位 | 切tab/文档及窄屏Dock返回不丢焦点；view state不写source dirty；滑动key和拖动playhead不会混为同一命令 |
| jobs | 长seek/warmup复用已有EditorJobSystem，有限预算/可取消 | 控件可显示requested time、progress、cancel、completed time；旧任务结果按owner/revision失效，不能在UI线程调用无界循环 |

原 `res://ui/editor/workbench/animation/transport_controls.zui#WorkbenchTransportControls` 有稳定control/event：Record、Play、Pause、Previous、Next、Loop，route均为 `workbench.extension.animation_transport.*`。本域不重命名或覆盖这些原事件。需要专用owner给出同一组件的domain action/context参数及实际安装后binding→handler映射，再复用其布局/图标/焦点；这不意味着原animation模板route已经调用particles preview。粒子尚无record/capture资产格式或loop执行语义；Previous也没有默认“上一事件”的含义，不能直接映射成reset。额外Stop/Reset/Warmup动作复用现有按钮/动作槽，由owner确认组合方式，不复制一个粒子版transport。

**实际时钟差异**：`ParticleSystemInstance::tick` 内部使用 `scaled_dt = dt * component.time_scale.max(0)`，而 `ParticlesManager::rewind_preview(handle, fixed_dt, playback_seconds)`按未缩放的remaining循环再调用tick。因此time_scale≠1时，参数 `playback_seconds` 并不等于最终simulation age，time_scale=0时更无法靠该调用前进。UI不能把请求的seconds直接当已执行age。公共surface保留明确的clock unit/反馈；粒子controller需确定preview rate与场景instance time_scale的关系后实现。新增差异PFX-RUNTIME-06保留，不擅自修改共享clock规则。

`ParticleRuntimeSnapshot`和 `ParticleEmitterState`没有age；公开GPU专用 `ParticleGpuRuntimeInstance`有 `age_seconds`，不能把后者冒称为CPU/GPU通用snapshot。Preview controller需关联权威step、time_scale、revision、backend和有效frame；跨backend切换不复用旧age/frame身份。

## 给动画owner：寿命曲线

已读 `ui/curve/{model,canvas}.rs`，复用 `CurveModel`、`CurveView`、`CurveKey`、`CurveSelection`、`CurveCanvasTransform`。现有foundation是renderer-neutral视图，提供Step/Linear/Hermite vocabulary；粒子runtime只支持normalized lifetime线性曲线，数据校验/求值/历史仍由粒子域负责。

| 需求 | 精确消费边界 |
| --- | --- |
| 横轴 | `t` 是单粒子 normalized age `[0,1]`，显示0～1或百分比并保留单位；System playhead是秒。不同birth time/lifetime的粒子同时存在，不存在唯一的 `system_seconds / selected_lifetime` 作为公共curve playhead；不可画一个看似权威的同步游标 |
| size | `size_over_lifetime.value`是初始size倍率；实际size为initial_size采样值×该倍率。字段不能写成世界尺寸或秒，reset来源遵循真实模板/继承层 |
| color | `color_over_lifetime`每源key是一个t和Vec4；可显示RGBA四条curve，但共享一个稳定source key和t，分量选择有明确channel。拖某一分量的t时四分量共同移动；改某一value只改该分量，一次domain事务。不能生成四套相互漂移的key |
| 插值/切线 | 只启用Linear，`in_tangent/out_tangent=None`；共享工具的Hermite/Step按钮须按粒子capability禁用并说明当前backend限制，不能写入丢失后又显示为已保存 |
| 选择身份 | `CurveElementRef`接领域稳定curve/key，component/channel身份独立；runtime的Vec索引没有持久key，source DTO补足后才能接增删/排序。重命名不换key，Undo恢复被删key的原身份 |
| 有效性 | `CurveBounds::new`/`CurveKey::with_tangents`等foundation归一/过滤不代替source validator；非法输入保留draft及diagnostic，不先把NaN投影为0后commit |
| CPU/GPU | CPU多key线性曲线与GPU>2key first-to-last baseline明确区分；不靠公共画布画出相同曲线便声称两个backend等价。GPU限制由domain capability/diagnostic提供 |
| 坐标/命中 | 曲线value向上为正，screen y向下；真实pointer先变pane-local逻辑坐标，再用CurveCanvasTransform；DPI只在host边界一次变换。工具栏、label和value行仍为flow |
| 撤销 | `CurveModel::apply`返回inverse是复用点，但UI直接调用它不等于已有预算/文档history；粒子typed command需走现有EditCommand和retained-memory准入，drag预览/commit/cancel边界保留 |

本域DTO/codec现已实现format_version=1、持久UUID键及往返；重复t拒绝，t严格递增且在0～1内，端点可删除、空曲线沿既有runtime语义，size非负、color允许有限HDR。测试尚未运行，源操作与公共曲线消费者尚未接入；这些业务规则不要求动画owner为粒子增加Bezier或改变动画已有插值语义。

当前已在粒子原包新增可消费的领域候选 `source/curve.rs`：`ParticleSourceCurveAdapter::new(&mut ParticleSystemSource, emitter_key)`实现现有CurveModel，CurveId为 `ParticleSourceCurveId { emitter_key, channel }`，Delta为 `ParticleSourceCurveEdit { curve_id, key_id, expected, replacement }`。size与RGBA仅Linear、无tangent；RGBA四视图使用同一个源key/t，颜色值允许有限HDR，size只接受非负；time跨过别的key时原地重排且身份不变。apply先拒绝旧值不匹配/不存在/非法/重复t，再写一次字段并返回inverse；它借用宿主authority，没有history、整文档快照或另一个store。宿主文档revision/retention/gesture准入与UI事件安装仍由原合同接续，调用trait并不自动完成撤销或dirty。8项曲线测试源包括真实CPU输出变化/恢复，曲线阶段11路径曾静态通过、Rust执行0；后续参数阶段范围和23项总数见下节。没有修改公共曲线surface或新增该阶段Cargo依赖，也没有发布组件。域内源API交接可先据此review，不能当运行态交付。

## 给材质owner：Overview与有限VFX节点

已读 `ui/graph/{model,canvas,node_widget,commands,routing}.rs`，复用 `GraphModel`、`GraphNodeView`、`GraphCanvasState`、`GraphSelection`、`GraphDeltaCommand`和clipboard接口；schema与mutation留在领域authority。GraphModel已有retention proof、revision和prepared delta；不建立整图无预算快照undo。

| 图职责 | 粒子提供 | 需要共享表面支持 |
| --- | --- | --- |
| System/Emitter Overview | system/emitter真实稳定key、名称、summary、位置、source selection；runtime字段分组不是可连线脚本 | 允许 `GraphNodeView.inputs/outputs`为空、`edges=[]` 的真实无连线mode；不为美观生成pin或假wire。节点内部标题/参数/摘要复用公共flow/文本 |
| 五类VFX节点 | SpawnRate、Lifetime、Velocity、ColorOverLife、ShaderGraphMaterial；lower到真实compiler；同类重复拒绝、容量/材质有限制 | 同一graph surface提供node selection/拖动/编辑/创建菜单/诊断定位；这仍不是Niagara通用script graph。域palette由真实schema和capability生成 |
| 连接 | 当前VfxGraphAsset只有nodes Vec，没有edge/pin模型 | 不暴露connect gesture/connection handles；不能随意把 `can_connect` 同型默认verdict视为执行语义。未来edge schema若获设计决策再接domain verdict |
| View transform | source node position为graph坐标，pan/zoom属于view state；node内部几何来自文本/控件测量 | 复用GraphCanvasState screen_to_graph/graph_to_screen，先扣pane origin、除真实DPI；框选/hover/hit统一变换；F/A在正确focus scope生效 |
| Command/history | 稳定key、expected revision、typed delta、source field定位、真正inverse及retention bounds | 共享drag preview→单次commit/Escape cancel；错误不改source。clipboard创建新key且不改变原文档；源codec尚缺时禁用而不复制caption代替资产 |
| Selection | 与EmitterList/ModuleStack/Details/Curves同一个document selection；node/key删除后选择可恢复的有效祖先 | source selection与surface view selection互相投影，更新guard/generation防递归；不再开第二个全局selection store |
| Diagnostics | compiler返回node_index等，source稳定key映射由粒子DTO/lowering保留 | 定位实际node/field并保留当前revision；source node顺序变化后不得用旧index跳到错误节点；旧编译结果显示stale/last-good |

材质需要Dag/typed pins并不要求粒子无edge模式退化为表格，粒子也不借用材质pin类型或输出字段。普通面板布局和窄屏Dock不属于画布坐标例外。真实Penpot shared graph component UUID/slots/bindings依专用owner交接，Parent统一catalog/导出接入。

## 给Parent：公共框架与预览

| 编号 | 具体接口需求 | 当前证据/边界 |
| --- | --- | --- |
| PFX-SHARED-01/11 | 同一source的DocumentHost/Dock/active focus kind/current-document Save路由、E/F/Ctrl+O作用域和Redo兼容 | 现Ctrl+O全局Open Scene、Ctrl+S全局Save Project；keymap两个候选同时enabled返回None，不会自动优先粒子。菜单/按钮/快捷键同一eligibility，未改全局keymap |
| PFX-SHARED-07 | PreviewViewport真实renderer外部图像/surface、ready/failed、frame generation/resource identity、相机输入/焦点/卸载 | 现component为静态Image，input三项false。`PreviewSceneBackend`只有create/destroy/subject/playback/invalidate/focus，`PreviewSubject`为opaque locators/string overrides，无通用frame/source revision DTO；不能以trait存在推定生产backend |
| PFX-SHARED-08 | NumberField typed props/min/max/step/precision/单位和错误draft，WorkbenchField无损u64字串 | 默认42/0～100/整数不能限制实际rate/capacity；seed可能大于2^53，payload必须精确保留。能力由领域提供，不往token放业务范围 |
| PFX-SHARED-09 | PropertyEditorRow的reset action/来源、长中文label overflow、DiagnosticRow自动高度/可展开信息及跳转 | 复用原catalog identity；不以固定48高度裁掉diagnostic，不把component实例覆盖reset和source默认reset混成一个动作 |
| PFX-SHARED-05 | 真实公共库/实例UUID、样式引用、版本、props/slots/events及导出闭包 | 当前Shared rev2/isShared=true/4原主件/34颜色/3文字样式；本域11个linked copy最后验证rev1。服务连接拒绝导致rev2 sync/save/fresh未执行；typed槽、正式export/全局A/B继续open |
| PFX-SHARED-06 | 后续桌面时段明确授予/释放；LayoutReviewCase与原生normal-frame association | 当前已有局部草案board，尚无EXE/资源就绪指纹，不申请缺少前提的捕获时段；不改变系统DPI、不操纵共享鼠标键盘、不借主工作台旧截图 |

seed编辑使用严格十进制字符串，提交时按u64范围0～18446744073709551615校验；binding、插件消息、DTO/codec和持久化不得经过JS Number或浮点转换。显示分隔符/本地化与真实payload分开，非法draft保留并给出诊断，不能截断、clamp或先转浮点再写回。该字段复用既有WorkbenchField/InputField，由Parent交接原组件的字符串payload合同，不建立粒子专用数值控件。

粒子preview适配应使用现有生产secondary-session/render生命周期，一个粒子文档group共享owner；前文 `ParticlesManager` 指域controller内的模拟权威，不授权在公共PreviewScene外隐藏启动第二个runtime。真实生产backend缺口、已保存资产与未保存source revision的传递及render consumer安装点先与Parent交接。CPU sprite extract和GPU collector/executor的存在不代表Editor窗口已加载并显示它们。

## 原control与共享消费者映射

| 原control | 消费方 | 原身份/业务语义保留 |
| --- | --- | --- |
| ParticlesEmitterList / ParticlesModuleStack | Parent TreeRow/PropertyEditorRow；graph owner提供Overview node选择联动 | 固定字段阶段不提供任意module reorder；增删/重命名/默认/diagnostic定位按粒子source command |
| ParticlesCurveEditor | 动画CurveModel/curve surface | normalized-lifetime与RGBA共同key、Linear能力、typed delta/undo；不把System秒游标接成curve t |
| ParticlesPreviewTransport | 动画transport/timeline | 同一真实preview owner；原particles.preview.*操作ID保留，映射需authored event→安装binding→handler实证 |
| ParticlesPreviewViewport / ParticlesPreviewStats | Parent生产viewport/media及粒子renderer adapter | 真正frame/clock/backend/snapshot，不保留静态默认图或固定粒子数 |
| ParticlesDiagnosticsPanel / ComponentDiagnosticsRow | Parent公共diagnostic row | source/runtime/stale/drop不同来源可辨；点击只定位存在的source key |
| ParticlesComponentAssetRow/PlaybackRow/BackendRow/SeedRow | Parent场景Inspector/资产picker；粒子领域适配 | 当前component内嵌asset，未有继承/覆盖同步层；playing/time_scale属于场景实例，不能误写source asset |

## 下一实际实施与验收

先在已分配真实文件形成flow区域设计；共享owner交接现有组件实例和样式后建立真实linked instances并核对props/slots/events。完整设计保存获ACK、关闭原context、fresh认证重开，再通过原插件导出到原ZUI，保留依赖hash/稳定ID/binding/event；不能用另一个假Penpot或截图替代。

公共owner确认前本域不改shared路径。当前已有域内source codec/资源URI-ready lowering、稳定UUID键、借用同一source的CurveModel领域适配及typed参数修改候选；12个源文件是补丁检查范围，23项是静态测试定义、执行0。旧r1拒绝、历史r2解析路径和pending预检分别保留；最新cdd capsule只资格化材质配方/source closure，不能沿用历史“manifest可解析”判定粒子可执行。原身份授权、当前完整输入、受管合并锁生成仍未就绪。接下来仍须把域typed edit接现有DocumentHost/operation/history安装点，并让单一生产preview owner消费；不新建全局文档或隐藏模拟服务。三尺寸、实际DPI144、224项产品要求、样式传播、原生EXE并排/差异和逐图复核仍全pending，passed/accepted=false。

## 专用组件实际身份与接入准入（先前阶段历史）

2026-10-24T10:59的只读API核对发现动画专属文件revn2已含native masters：Timeline `8cf70bfd-9275-4bac-849b-b79148b8a4fd`、Curve `78492c86-3bb4-47dd-97c6-752d6ff37e3d`、Transport `982ef940-2239-4167-a24a-56c80ec35fbf`。对应main shapes为`4db0829c-abc4-482d-b7b3-155a5d8d75c2`、`599963dc-b91b-4740-926b-d38154aa8f3a`、`44eb5ccd-8ac2-44f2-a4ad-d0d618daa096`。材质文件revn12含三个GraphNode提案主件。两文件当次均styles=0、linkedLibraries=[]、isShared=false；公共文件仍revn0且无组件/样式。

这些是实际设计进度，不是可消费版本交付。继续由原owner/Parent确认发布版本、跨文件实例、slot/typed props/events与源组件映射；粒子没有复制主件、发布他人库或另造共享实现。原9画板及业务摘要主件保持身份，本轮在同一文件另存9个状态板、4个该主件实例；当前revn7/18板/266对象，实际身份和修订核对见particles-design.md。Jenkins原manifest能力请求为`ue-plugin-workspace-entry-20261024`；本域预检身份`ue-particles-source-codec-20261024/validate-source-codec-r1`已拒绝且未提交，回执在本域报告根。Parent消息ACK仅表示入队。

11:16:47Z再次实际API读取：粒子revn5/103对象保持，公共文件仍revn0/1根对象/0组件/0样式；动画已到revn5/150对象，三个组件UUID/main保持，材质仍revn12/105对象。四文件均`isShared=false`、`linkedLibraries=[]`；没有据此写入跨文件引用。只读回执为`shared-readiness-codec-stage.json`。

## PFX-SHARED-10：文档与operation安装点的具体交接

| 现有真实符号 | 粒子可接续部分 | 需Parent提供的最小合同 |
| --- | --- | --- |
| `core/editor_extension.rs::register_operation_command`；`core/commands/registry.rs::register_operation` | 域内typed payload/factory、source validation与mutation | 现有command descriptor已由authoring batch注册；应明确后安装factory的受支持入口、冲突/卸载规则，不能重复注册同一command制造成功 |
| `editing/operation/registration.rs::OperationCommandFactoryRegistration::new`；`EditOperationTarget::EditDocument(DocumentId)` | 以实际document ID与expected source revision生成一次命令 | target在registration中固定；同资产多view/多文档的factory路由、close/unregister及当前focus document解析由宿主明确，不使用EditWorkspace规避文档history |
| `editing/engine/command.rs::{EditCommand,EditContext}`；`editing/context.rs::CoreEditContext` | 域内apply/revert、retained-owner证明、prepare一次性permit与同一source authority | 通用trait公开，但CoreEditContext及animation_document都是pub(crate)，没有可供粒子插件调用的动画document store；需现有宿主承载粒子domain authority的实际安装点。不得为插件复制或另建全局CoreEditContext |
| `core/extension/toolkit/document_toolkit.rs::DocumentToolkit<Host>` | 粒子源校验、TOML不可变bytes、源URI | trait公开且不拥有dirty状态；具体Host、toolkit registry注册/close、asset URI→canonical path、SaveCtx的durable writer及revision receipt须沿现有宿主交接。纯to_toml成功不调用mark_saved |
| `core/asset/dirty/save_job_adapter.rs::{SaveDirtyViewExecutor,SaveDirtyViewsJobAdapter}` | 由已准入job执行上述source保存，保留dirty generation | 保存期间新修改不能被旧完成清clean；取消、释放与原source revision通过现有authority；不在callback前构造无限制整文档快照 |
| `ui/preview_scene/preview_scene.rs::PreviewSceneBackend` | DTO→ready runtime candidate和粒子clock/reset/seek域语义 | 公开trait没有source revision/frame/media返回合同；生产secondary-session绑定、source candidate安装、last-good切换与renderer消费者仍由Parent交接，不能启动第二个隐藏manager填空 |

这是一份源接口边界核查，不是请求把private模块整体公开。粒子接续域内实现；公共安装点按最小现有扩展机制交付，并由Parent统筹多窗口、history、retention与资源生命周期。

共享锁文件的早期粒子单包提案：`zircon_plugins/Cargo.lock`里给`[[package]] name="zircon_plugin_particles_editor"`补`serde`、`toml`、`uuid`，复用1.0.228、1.1.3+spec-1.1.0、1.23.4。Parent现已明确把AI的toml依赖合入同一受管计划，最新范围是两个既有entry共四边，其余entry/version/source/checksum和无关字节保留。未生成锁diff或after SHA，也没有lock-resolution配方；不得手拼、筛除超范围diff或去掉测试`--locked`。历史单包提案不再代表当前锁owner的完整写集，粒子未来测试仍只编译粒子单包，不合并AI测试。

历史r2阶段再次通过公开capability入口实际读取ready/public-transport，driver `fc553228…`、runtime `start-1792836412672596900`保持，当时installed capsule为 `baac70c97872d3395c5c422fa44a7cdaa481a8ac4f62820d1cc2ac099ad0ac08`，formalAcceptance/nativeLifecycleVerified均false。初次只看checkout旧entry解析器而误判支持路由，随后读到Parent的 `.jenkins/records/ue-plugin-workspace-entry-20261024.route-02.json` 并更正：正式 `tools/dev/local-cargo.ps1` 当时选择capsule内 `clients/tools/jenkins-management/entry.py`，SHA-256 `ede21bdd9c072b52fc357c18140f315b975f9e9eb3a2f67d5184ad5b23c329e7`，解析层支持canonical插件manifest。旧checkout解析器的事实不等于实际wrapper入口失效；原拒绝及该历史更正分别保留，不代表最新cdd配方已准入粒子。

已先声明单包/defaults off/features=[]/lib/test profile、`source_tests::`10项、jobs1/locked及必要传递特性，再由正式wrapper执行一次`-DryRun`。同一request `ue-particles-source-codec-20261024`下显式新operation `validate-source-codec-r2`绑定当前9源文件hash与6依赖hash，sourceView=checkout、externalRoots=canonical `E:/Git/zr_vm`、buildRoot=`E:/cargo-targets`；没有源副本。实际终态exit2、pending、`Current session authorization is pending`，Cargo未开始、Rust执行0。dry-run不自动建立context；此结果既不等于测试通过，也不证明真实自动登记提交必然失败。当前锁图仍缺三项，先交接集成，不以无锁Cargo规避。新身份/预检为报告根 `source-codec-jenkins-r2-identity.json`、`source-codec-jenkins-r2-preflight.json`，r1不覆盖、不重投。

本轮实际状态设计已投递Parent，消息ACK `01a16dbb-c46f-7c42-a560-a8a1285f121e`；其中“实际入口仍不支持manifest”的判断由上段更正。随后更正投递exit1，官方返回 `queue cannot contain more than 100 submissions (code -32600)`，未获得新消息ACK；本段及r2报告为待Parent读取交接，不声称更正已送达。没有修改公共source/library、占用桌面或改DPI；正常EXE/资源闭包未就绪，尚未请求捕获时段。下一步消费正式共享组件/样式和生产宿主的最小安装合同，域DTO/handler不得绕开既有文档history或单一preview owner。

最终只读API复核：本域仍revn7/266对象，18个board均为上述尺寸及native flex，7个域实例均引用原main；无悬空child ref、无parent/child不一致。公共文件仍revn0、0组件/0 color/0 typography；动画已revn8、材质已revn33，原三个专责main各自保持，三文件均isShared=false、linkedLibraries=[]且无共享styles。本域没有为消费未发布组件而跨文件写入或复制main；各域后续进展不被旧revn5/12覆盖。实际复核存于 `penpot-state-design.json::finalReadOnlyIntegrityObservation`。

曲线增量的修改路径严格为本域 `src/source/curve.rs`、`src/tests/source_curve.rs` 及原lib.rs/source/mod.rs结构接线；未改公共source/组件、运行时或锁文件。向Parent报告该范围与正式入口更正的前轮队列投递再次exit1/100条上限，尚无ACK；失败存入 `source-curve-inspection.json`。Parent第二次接续现已确认实际读取前版交接与r2预检，这不补记失败投递为成功，也不证明其已独立读取后续曲线/参数源码。VFX编译器另在 `zircon_plugins/rendering/features/vfx_graph/runtime`，接其source/lowering需明确独立feature包依赖与owner；当前没有跨写该feature或借材质图模型填充粒子。

### Parent第二次接续后的可消费参数API

本轮新增路径仅 `zircon_plugins/particles/editor/src/source/parameter.rs`，其余修改位于Parent继续授权的lib.rs、source/**和tests/source.rs；已有tests/source_curve.rs保持原hash，只将模块接线放回source_tests入口。`ParticleSystemSource::apply_parameter_edit(ParticleSourceParameterEdit)`支持Seed、SpawnRate、Capacity、Lifetime，返回相同类型的字段inverse。每次先对同一个source做完整校验，再按稳定emitter key与expected旧值拒绝过期输入；source内部共用nonnegative/range校验，生命周期两端一次修改，不暴露中间非法范围。没有第二个source store、history或预览实例。

seed事件必须保持原十进制字符串，调用与codec共用的 `parse_particle_seed(&str)` 后才构造typed Seed edit；MAX、MAX+1、2^53+1、中文数字/小数/空白的回归源均已列入，未执行。Capacity仍为u32而非示例0～100限制；4096与0是有效source测试，0不会跳过其它校验。当前只提供四类字段修改，未实施shape/资源字段或任意module/script编辑，也没有把descriptor当handler。宿主负责实际DocumentId/revision/retention/eligibility、一次gesture、reset目标来源、dirty/history和预览安装；不得先绑定首个DocumentId或使用EditWorkspace规避。

原r2 dry-run与冻结identity保留。早先r3/source_仅为未创建计划，已因Parent后续要求标为superseded；不创建新request/operation、不手改identity/锁、本域不查询或重试授权。现有source_tests::入口对应15项直接测试和8项curve子测试的静态定义，共23项，尚非Cargo发现/运行；Jenkins owner需沿 `ue-particles-source-codec-20261024 / validate-source-codec-r2` 对账当前完整输入后再接授权/受管锁生成及真实执行。Parent已排队给原owner的消息不等于本域测试执行。本轮不再次投递满队列；此既有文件为合并交接，参数阶段实据见报告根 `source-parameter-inspection.json`。

### 原r2只读对账与合并锁计划：当前资格化状态

本节“当前”限定为13:12Z operator对账和随后13:24Z本域读取时刻。本轮14:02Z公共控件消费没有再次查询、提交或执行Jenkins，也未新核实锁生成/配方变更；旧快照不随设计修订自动升级。

已读取operator在2026-10-24T13:12:12Z写出的原身份对账 `E:/cargo-targets/zircon-local/reports/jenkins-transition-monitor/operator/ue-particles-source-codec-20261024/reconciliation.json`，SHA-256 `815e7146783ae2c6040d41899242722e723415d8cce2733fb5abcbb39d9c0660`。本域只读核对源文件和该回执，没有再发QueryOnly/DryRun/test/generation。原r2输入SHA仍为`1e5aa440b33a050e324bd180caf3c1b48a49255c66a571bad8c2190b0e7c7379`，原9项源hash有5项变化，6项依赖hash相同；curve.rs、parameter.rs、tests/source_curve.rs不在原9项内。

当前本地crate输入候选为报告根 `source-closure-candidate.json`：16个字面可达Rust模块（13生产、3测试）、6条include_str边/4个唯一模板或ZUI输入。原`tests/cases.rs`还定义1项其它测试，即使运行筛选为source_tests::也仍进入lib测试目标的编译。候选只记录路径/哈希/模块边和静态定义，不是源码副本、execution identity、完整传递闭包或capture-ready证明。12文件source guard仍匹配当前补丁hash，但不能代替本地crate16模块，更不能代替engine/SDK/宏/目标/配置/资源闭包。完整source/features/resource/pool声明由执行owner接续，既有原identity和preflight四文件字节保持。

installed文件与operator观察hash一致，当前capsule为`cdd14500fa39088ae2c9870f62825acc2e526ba647d24d77cd6287722b5d0f31`；仅材质manifest及其source closure已资格化，粒子/AI未资格化，generate尚无lock-resolution配方。operator的原身份QueryOnly exit2/pending/ordinary_entry_unproven，授权pending，5.888秒仅客户端查询时间。保留历史/当前queue直接参数未找到匹配，删除或间接历史未知，不能写成never queued；server/native receipt、执行job均null。DryRun/QueryOnly不建立自动context，不要求人为改grant。

合并锁计划为 `E:/cargo-targets/zircon-local/reports/jenkins-transition-monitor/operator/plugin-manifest-validation-20261024/workspace-entry/ai-lock-resolution/combined-lock-resolution-plan-r2.json`，SHA-256 `1f8593faa8f78998914c31ea5fa6cde84db25924ccd87d4a4fe6209b959177dc`。唯一拟写路径`zircon_plugins/Cargo.lock`、before SHA `04de41d28ef79436b8bc4c264d8fb122d303ae2a7005085522783afdeae02bbf`；AI加toml、粒子加serde/toml/uuid，共四边。原AI-only计划字节保留；实际Cargo diff、after SHA、生成receipt/request仍null。计划中的候选argv不是已验证支持的解析配方，本域不执行或手工拼锁。

后续粒子验证仍为单包/lib/test profile/defaults off/features=[]/source_tests::/jobs1/thread1/locked。Runtime native-host/target-editor-host/graphics、SDK runtime/declaration及wgpu29.0.1只是已知传递下界，未完成feature解析。captureReady=false、没有捕获时长/估计或freeze；原runtime/nativeLifecycle/formal和窗口族验收均open。原11:54及后续队列满失败原样保留，本轮不重投。目标仍blocked，当前只合并新事实，不宣称宿主/运行时接线或服务资格已解除；新增Cargo/产品源码/锁/缓存清理/服务操作均0，实际释放0 B。

### PFX-RUNTIME-06：数值不前进与生产时间接口的精确提案

接续动画owner已写入的 [AN-PFX-G01～06](animation-shared-controls.md#粒子消费合同与-pfx-runtime-0306)。本轮只读追踪发现，`service.rs:267-271` 的问题还包括非终止输入：`Real=f32`，合法有限 `playback_seconds=1_000_000` / `fixed_dt=1/60` 经binary32扣减后remaining仍等于1_000_000，while条件保持真并持续持有manager锁。1024/0.00001与1/0.00000001也不前进；后者的dt还被CPU实例按EPSILON跳过。`1_000_000.0625/0.03125`先前进一次，再因round-to-even停住，说明只检查第一次扣减不够。

证据是最多3次的IEEE-754 binary32数值轨迹及实际源hash，保存在产物根 `shared-contract-inspection.json.runtimeSeekNumericProgressAudit`。没有执行可能挂住的真实rewind、Rust测试或EXE；这是源/算术证明，不是运行通过。现有23项source回归不覆盖该runtime缺陷，也不据此新增passed或修改capability。

UE的 [NiagaraComponent.cpp:1023](../../../dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/Niagara/Private/NiagaraComponent.cpp) 区分DesiredAge和SystemInstanceController的真实GetAge，倒退时Reset，并以MultiTick结果更新bIsSeeking。[NiagaraSystemSimulation.cpp:1095](../../../dev/UnrealEngine/Engine/Plugins/FX/Niagara/Source/Niagara/Private/NiagaraSystemSimulation.cpp) 使用整数tick循环，在每步后检查MaxSimTime并返回是否仍有未完成工作。这给出了目标行为：真实age回读、分段推进和pending；UE的默认33ms不直接复制成Zircon的新全局阈值。

以下是待Parent确认的精确扩展范围，不是已授予runtime/public源码写权，也未创建这些API或新文件：

| 路径 / 符号 | 具体候选变更与边界 |
| --- | --- |
| `zircon_plugins/particles/runtime/src/service.rs` | 仅接既有ParticlesManager下的preview子模块，保留rewind_preview签名与原play/pause/stop/tick API；按真实句柄执行，不能用tick(dt)推进全部instances冒充单预览seek |
| `zircon_plugins/particles/runtime/src/service/preview.rs`（候选新子模块） | `preview_clock(handle)`直接从当前ParticleSystemInstance读取age_seconds/playing/time_scale/backend/fallback；`step_preview(handle, unscaled_dt)`只推进该实例一次并同锁返回实际clock，校验finite及可执行步长，保留调用前playing。它不建立manager、clock ticker、DocId或frame来源 |
| `zircon_plugins/particles/runtime/src/lib.rs` | 仅结构性导出实际需要的clock结果；候选`ParticlePreviewClockSnapshot`只携带runtime已拥有的handle/age_seconds/playing/time_scale/backend/fallback，不从UI填source/request/frame字段 |
| `zircon_plugins/particles/runtime/src/simulation/mod.rs` | 若需新增精确错误，范围只限不可前进步长与工作预算拒绝；保持旧错误，检查public enum的exhaustive调用方。不借InvalidAsset把时间/预算拒绝伪装成资源故障 |
| `zircon_plugins/particles/runtime/src/tests/service.rs`、`zircon_plugins/particles/runtime/src/tests/service/preview.rs`（候选测试子文件） | 真实多handle隔离、clock/playing/age读回、0/非1倍率、数值不前进/预算拒绝前无reset副作用、暂停/停止与已有replay兼容；测试树先明确后实施 |
| `zircon_plugins/particles/editor/src/preview/{mod,controller,job}.rs`（候选新域范围） | 粒子仅适配现有Editor jobs的准入/限额、CancellationToken、progress/ticket；在Parent的document/source/install generation gate下编排真实runtime调用。此目录不塞入纯source codec以绕开范围，也不新建scheduler、全局文档/history或另一manager |

执行顺序提案如下，需和Parent生产owner同一次接线，不能只发布DTO/按钮：

1. Parent在defer/retention准入前解析实际文档/source/install generation及单一preview owner。域校验requested单位、fixed_dt和总步数/资源预算；计划用有界整数步索引与较宽精度计算每步请求，不以f32 remaining减法决定是否结束。工作总量/每slice预算由实际准入合同提供，不能在UI自行填一个假完成时间。
2. 请求明确为legacy未缩放seconds或simulation age。simulation-age目标只在有效time_scale下映射一次；time_scale为0且目标age非0时明确拒绝。现有tick内部仍应用倍率一次。每步/完成反馈的actual age来自真实实例，不能以requested*scale回填。
3. 重播期间按单一controller策略暂停选中的真实实例，步骤由step_preview受控推进；其它handle保持既有状态。每slice检查真实取消/旧source/旧handle/关闭/新请求，再执行有界步骤，释放manager锁后报告进度。已有JobContext的取消只表示意图，未确认执行停止前不报cancelled。
4. 部分推进/取消必须回报实际已应用age；若Escape策略要求恢复手势起点，恢复也通过真实有界重播，确认前保持pending，不靠UI时间标签回退。source编辑历史与临时seek始终分开；正常播放恢复只消费仍有效的请求和此前playing状态。
5. rewind_preview保留兼容入口，但数值不前进输入须在reset前拒绝，长请求使用明确的有界执行/工作预算拒绝。仅加首步检查或仅把原无界循环放进后台不满足此合同；同步兼容入口的预算来源/拒绝错误需由Parent与runtime owner明确后实施。
6. Parent将runtime结果与真实document/source/install、request顺序和生产frame/resource关联。simulation完成、GPU prepare/configure/readback及已呈现frame是不同状态；GPU configure-once、CPU无replace及参数候选安装仍是独立缺口，此提案不把CPU stepping当GPU接通。

拟验证范围是`zircon_plugin_particles_runtime`单包/defaults off/features=[]/lib/test profile/`service::tests::preview::`与受影响既有replay测试，必要传递features为SDK runtime及runtime graphics；不触发editor全包测试矩阵。实际命令/筛选须在这些测试实施后由Jenkins owner声明，当前不执行、不扩展原r2的editor包或创建新身份。共享lock/授权、正式runtime范围与生产集成均未就绪；本提案只解除“需要什么实际接口”的模糊性。

## Shared rev1消费回执与PFX-SHARED-12

以下为2026-10-24T14:02:34Z的历史独立认证读回（当前发布库已rev2）：Shared file `b4a1e3ce-9e6a-81fe-8008-d69d607aac12` / page `b4a1e3ce-9e6a-81fe-8008-d69d607aac13`已revn1/vern0、isShared=true、4原生主件/41对象/styles0，旧rev0/空库不再代表当前状态。本域真实UI已连接该直接库并在自己file的独立验证页保存4个linked copies，最终revn19；两次fresh重开保留componentFile/componentId/main shapeRef和本地文字覆盖，原生Reset overrides也已回读。原页266对象和私有业务component摘要未变；消费者未写Shared或其它族。UUID、源hash、slot、覆盖/reset、HTTP保存回执与两图逐图复核见 [当前设计身份](particles-design.md#shared-rev1原生消费者身份与逐图复核) 及报告根`penpot-shared-components-consumption.json`。

| 待公共owner消费的具体合同 | 实际证据、依赖与边界 |
| --- | --- |
| PFX-SHARED-12：TreeRow的native caption覆盖后仍遵守source单行省略 | 源`zircon_editor/assets/ui/editor/components/workbench/primitives/data/tree_row.zui` SHA `c6687ef563179bfbe395917cb4805914cb9e2b3f7f0bc49091f00135be144cf2`声明text_overflow=ellipsis。真实Shared主件`6ddc248b-bd35-80c5-8008-d6c8c7a79944`、main `6ddc248b-bd35-80c5-8008-d6c8c452bf0c`；本域copy `92a91f8e-23b8-8096-8008-d6ceb4cfb381`和leaf `92a91f8e-23b8-8096-8008-d6ceb4cfe02f`保留shared ref `6ddc248b-bd35-80c5-8008-d6c8c4c69ae5`。保存文字为完整“发射器／sprite_emitter”，但128×16框实际有两条16高SVG run，240×28行框未改；60%整板与100%原图均有该差异。原因未证实，请Parent/public owner核对既有TreeRow painter、typed text映射/覆盖-reset与正式export，保留master/源/完整caption，不在消费者detach、扩大高度、裁字或栅格化。该提案未授予粒子公共source写权，也非要求另建TreeRow |
| PropertyEditorRow必需value槽及PFX-SHARED-08 | 原value槽required=true、multiple=false，接受WorkbenchField/NumberField等既有控件；4主件中未包含这些组件。验证页刻意保持空槽并标为未完；不能把空值区域当可输入参数。沿原owner发布typed输入和实际范围/字符串合同后再消费，不重复造组件 |
| PFX-SHARED-05原门槛保持 | 4主件跨文件link和本地覆盖/reset已有实际消费者证据，但canonical owner/sourceHash/slots/events/正式导出、完整body消费、colors/typographies、跨两族全局样式A/B仍open。文字改动未经过原ZUI导出/产品加载，不能充当用户要求的一次样式→导出→EXE截图变化 |

下一独立可做项是交接上述精确问题和消费身份，复用原本域source/test输入候选；没有新增Cargo、Jenkins请求/身份或授权查询。原r2和合并锁计划仍由原owner接续，不根据本轮设计进度声称其执行或配方资格变化。没有ready EXE/资源闭包或桌面时段，本轮不申请缺少前提的native捕获；passed/accepted=false。

## 恢复后原request/r2的当前配方与收件交接

Parent恢复r2的原生终态已核实，本域沿原session `01a16d52-a4a5-7271-90a2-336091337737` / request `ue-particles-source-codec-20261024` / operation `validate-source-codec-r2`只做一次正式QueryOnly。14:39:37Z wrapper exit2/pending/ordinary_entry_unproven，原文`Current session authorization is pending`，输入SHA `1e5aa440b33a050e324bd180caf3c1b48a49255c66a571bad8c2190b0e7c7379`及当前installed跨调用未变。3.594秒仅客户端查询，不是capture或服务端受理；不据此推断其它族请求，不要求用户手工grant，不重写DB、identity或换ID重投。原始argv/stdout/stderr/两个不同层次exit保存在`jenkins-r2-recovery-continuation.json`。

| 当前已验证的精确落点 | 待原Jenkins owner接续的最小范围 |
| --- | --- |
| 当前capsule `9c684aef96b8b1c94319aa8619207f8b07aefffa71d571d4f58708d2c3408233`；manifest/5 server/6 client均匹配，同driver `fc553228…` / runtime `start-1792836412672596900` | Parent的20b3是正确历史，不能继续当当前版本。当前installed SHA `c41a585f…`，本域未启停或部署任何文件；健康/收件能力与产品recipe分别核对 |
| pinned client `clients/tools/jenkins-management/entry.py:472-493`、server `runner.py:4054-4070` | 两端新增的是bounded AI filters，仍无`zircon_plugin_particles_editor/source_tests::`。只资格化真实插件workspace manifest `zircon_plugins/Cargo.toml`、该单包、lib/test/defaults off/features=[]/jobs1/thread1/locked；当前不执行候选命令或修改胶囊 |
| server `runner.py:720-733`及`788-796` | 插件source/metadata成员catalog仍仅material/prefab/AI/glTF。需沿现有closure机制纳入`particles/editor`和实际native/editor runtime特性边，覆盖源codec/curve/parameter模块、编译进入lib的原cases、include/template、外部VM及必要资源。12个owned补丁路径、16个本地模块候选和完整传递闭包分别保留，不能只加CLI白名单 |
| server `runner.py:3090-3125`；既有combined-lock-resolution-plan-r2.json | generate仍只source-index/render-template；AI toml＋粒子serde/toml/uuid四边仍缺，锁SHA保持04de41d…；由原owner提供合法source preparation/writer/资源准入和实际定向解析配方，核验真实diff/after SHA/receipt，不手拼或筛选超范围差异 |
| 原自动收件与冻结r2 identity/source hashes | 当前仍5/9 source变化、6/6依赖相同；curve/parameter/source_curve不在旧9项中。请原owner核对原operation/delivery/context，不把QueryOnly当自动登记。完整当前输入由执行owner接续，不覆写原identity、把to_toml当durable或发新ID躲开原收件 |

原NumberField/u64字符串、12 operation后端安装、document focus/save/history和Preview/Transport真实代次合同均保持。恢复不产生这些产品能力。本域保留当前source不变；待原owner给出可执行配方、完整闭包与captureReady/时长后再向Parent协调有界稳定窗口，不宣称当前已freeze。Penpot/桌面owner及时段不变，未操作Shared或其它族文件，原生passed/accepted仍false。

## 原combined-lock-resolution-plan-r2的四门实据

> 以下保留该阶段历史观察；resolver实现和JM server准备已由本轮原owner交付。当前生成/资源状态见本文末尾新增记录，旧空值不作为当前未实现的结论。

原owner的implementation-r3在2026-10-24T14:37:37Z完成AI有界入口交付，整体仍未完成。本域独立核对summary/result/generation-gate及#406 outer native四份指定SHA，并对五份受保护manifest/lock核当前hash，均匹配。9c684、entry `5663e4e9…` / runner `5f96012e…`及原driver/runtime保持；AI四filter/26项静态输入合同与81管理checks＋267 subtests不是粒子配方、AI/PFX Rust或host生命周期验收。#405错误配置断言失败和未知create收件保留，不替换原r2计划或请求。

| 原owner实据中的门 | 当前缺失内容及所需真实回执 |
| --- | --- |
| supported-targeted-resolver-transaction：not-implemented | 当前generate只source-index/render-template。原authority需交真实定向resolver事务、完整候选字节diff、原before hash CAS发布和失败恢复；候选超出四边须保留完整差异并返回，不能手筛成允许范围或把模板输出当锁解析 |
| original-generation-intake-and-preparation：no-server-issued-generation-phase-in-original-plan | 原计划generationRequestId/serverReceipt/sourcePreparationReceipt均null。null不是“从未送达”证明；未知create记录保持。由原owner沿原authority核实自动收件与server-issued phase，不能借AI/PFX validate或能力create/repair身份执行，也不伪造新request/attempt |
| generation-lock-writer：unproven-for-original-generation | 本次能力修复writer有9个工具路径，明确不含`zircon_plugins/Cargo.lock`。实际原生成phase需获得仅该锁的writer及源准备回执；用户授权合并四边的语义不等于该writer已准入。不改grants、不借其它工作树或放宽受保护manifest/root lock |
| resource-reservation：not-admitted | policy v11的reserve为37580963840 B（35 GiB），E在14:34Z观察18320347136 B（17.06 GiB）；无resolver专用可信estimate/reservationReceipt。该观察不能把非编译解析的实际需求等同编译预算，亦不能选tiny fixture/default预算或降低reserve制造准入。由原authority交实际预算和reservation；本域不清共享缓存或把差额当清理目标 |

AI toml＋PFX serde/toml/uuid四边全部仍缺，插件锁原SHA04de41d…不变，actualGeneratedDiff/actualAfterSha256/generatorExecutionReceipt/nativeGeneratorTerminalReceipt/recoveryReceipt均null、captureReady=false。事务及各phase真实就绪后，源owner再提供当前候选并声明实际包/传递features/resources和有界稳定窗口。粒子未来source_tests::仍单独原request/r2，不混入AI四filter，不去--locked。本域新查询/提交/编译/源码或锁写入均0；这轮仅消费原owner回执，官方消息ACK也不追加推断为已读。

交付原件位于`E:/cargo-targets/zircon-local/reports/jenkins-transition-monitor/operator/plugin-manifest-validation-20261024/workspace-entry/ai-lock-resolution/implementation-r3`；本域只在既有`jenkins-r2-recovery-continuation.json::combinedPlanOperatorFollowups`保存路径、hash和必要门状态，不复制工具模块或创建源码树。原owner的24576 B allocated清理单独记账，本域实际释放0 B；passed/accepted不变。


## Parent第三次接续的具体消费边界

Parent已实际读取旧rev19与验证页4原生copy并打开两PNG；此前queue满失败仍保留，不追造送达。本轮沿相同分配文件推进rev30的System Stack组合和既有参数factory；不新增文件/公共主件/主题、不改keymap/host/save route。身份与逐图见[Stack组合页](particles-design.md#system-stack组合页的真实flow与外链实例)，机器回执为报告根`penpot-stack-consumer-design.json`。

| 合同／owner | 当前域侧可交付与未满足条件 |
| --- | --- |
| PFX-SHARED-11：Parent文档输入/保存适配 | System=`particles.system`，Emitter后端真实存在后才用`particles.emitter`。focused ViewInstance→ViewDescriptor.document_kind→chrome→CommandEvalCtx，以及实际DocumentId/session/revision必须一致；当前kind/动态factory未安装，不用caption、首个DocId或EditWorkspace猜测target。辅助pane返回原doc。域新增只读`validate_parameter_edit`和严格`seed_from_decimal`，只覆盖四参数delta校验；Collapse/Rename/Delete/Clipboard的具体target/can_execute和12 operations执行器仍需真实文档消费，不把预检当host许可 |
| Parent键盘/输入owner | 保留同chord两enabled候选返回None。先选实际doc/pane/text/modal owner再求动态eligibility；粒子owns键但disabled不fallback OpenScene/SaveProject，不改全局menu When为Not(particles)。Ctrl+O须真实Stack target/factory；IME/draft/弹层先提交或消费，关闭后按trigger/doc/pane恢复焦点。Ctrl字符当前会冒泡，不笼统宣称文本自动屏蔽所有Ctrl键 |
| Parent当前文档Save／history | CtrlS现仍file.project.save→scene/project；通用粒子当前doc save route未注册。异步复用`submit_document_toolkit_save(original, instance, Explicit)`的Job/源锁/dirty save token/同generation durable receipt；native callback不调用wait wrapper。缺文档/校验失败/旧revision不fallback scene、SaveAll或codec。Redo默认CtrlShiftZ保留，CtrlY公共alias缺口不以覆盖/新history解决；S/D/ShiftD仍PFX-RUNTIME-05 disabled |
| PFX-SHARED-08／public输入owner | 新组合的4个PropertyEditorRow仍保持required value slot为空，未用右侧只读值或手绘输入补槽。7控件全部真实外部copy，不能据caption让字段可用。沿原NumberField range/u64字符串/多行Diagnostic/reset-action slots合同交付源字段、组件UUID、版本和真实export后再消费；完整控件/样式仍非accepted |
| PFX-SHARED-12／public Tree owner | 原长caption两行问题不被短caption板替代关闭。新Tree沿master默认selected=false/focused=false；源支持selected/focused等props，但当前只做text override，未有typed state投影/真实选择消费者，不能用改copy颜色伪造选中通过 |
| PNG exporter／Parent现有原owner | 15:33:15Z仅一次正常Penpot UI Add export(PNG 1x)→Export 1 element，实际POST`http://127.0.0.1:19001/api/export`返回404，等待download30秒超时，无输出PNG。请沿原服务/proxy/原生PNG exporter入口核查该具体路径；本域未重启、另开服务或借旧compiled runner绕过兼容guard。此为设计原生PNG门，另计官方ZUI插件导出/宿主编译与EXE native capture |

新页page`f2894894-da30-8095-8008-d6e405e95ace`／board`f2894894-da30-8095-8008-d6e431a06285`，1280×800；当前file rev30/3页388对象，22个native flex frames/7新copy/45个正确source refs和主件闭包。原业务页、原私有主件/7实例、旧4外链和Shared rev1/styles0保持。保存、关闭、两次fresh重开与局部flow已实证，完整官方export/多尺寸及产品操作未通过。

当前源新增只触及既有`src/source/parameter.rs`及`src/tests/source.rs`两文件，格式检查通过；新增2项定义后17＋8＝25、执行0。旧`source-closure-candidate.json`不重签，当前两hash差异追加原`source-parameter-inspection.json`。原r2请求/operation/identity、锁原字节和四生成门保持；本轮无Jenkins查询或重复测试/生成提交。无ready EXE/资源闭包、EXE指纹或本域桌面slot，不提交缺前提captureReady/时长/freeze。继续保留passed/accepted=false。


## d2873配方交付：原source owner审阅回传

> 以下保留该阶段历史观察；resolver实现和JM server准备已由本轮原owner交付。当前生成/资源状态见本文末尾新增记录，旧空值不作为当前未实现的结论。

已实际读取原Jenkins owner的`particles-editor-extension/result.json`与`summary.md`并核对通知SHA。当前selected capsule为d2873，11server/client文件一致；粒子canonical manifest、单包/lib/test/defaults off/features[]/source_tests::/jobs1/threads1/locked在client与runner及particles/editor metadata catalog均已贯通。2workspace/17manifest/31path边是管理检查的静态契约，不是已执行Cargo/完整9-root capture。9c684/cdd缺配方结论仅作历史；本轮不再把CLI/metadata catalog缺粒子列为当前阻塞。

六项变更由本域上一阶段产生并有现有记录：四文档为真实Stack/共享消费与输入保存/验收边界，两Rust为既有parameter只读预检/严格seed及2项回归。当前16路径在本轮文档更新前完全匹配15:43本域指纹及16:36owner观察。本地修订追加`jenkins-r2-recovery-continuation.json::sourceLineageRevisions`，逐路径保留旧/owner/当前hash和具体理由；旧r2输入、5b524bff候选、原失败与未知送达保持原字节，不捏造attempt、服务端revision ACK或改库。复用原session/request/operation，只向原owner交真实修订供其受支持的原lineage接续。

下一个执行前提是原owner取得合法修订/source preparation、完整9-root source/config/resource闭包与cache/pool/writer/资源phase，以及既有combined-lock-r2唯一锁的定向resolver/CAS/完整diff/恢复合同。四依赖边仍缺、04de41d原锁未改；generation/server/native/after回执为空。没有freeze或可信capture时长，E15.04GiB是ownerOS观察，不是资源拒绝receipt。不得改用material/AI recipe、重提旧identity、去locked或手拼锁。本域本轮没有新增Rust、Jenkins query/submit、Cargo/生成或共享桌面输入；25项测试仍执行0，passed/accepted=false。

## PFX-RUNTIME实例时钟/单步：Parent第四次接续的最小实现

Runtime增量只在本域获授的5路径实现并从crate root导出 `ParticlePreviewClock` / `ParticlePreviewError`。消费者复用同一 `ParticlesManager` 和实际 `ParticleEmitterHandle`：`preview_clock(handle)`是纯读，`step_preview(handle, unscaled_dt_seconds)`只调用所选实例一次tick，成功结果来自同锁内读回的实际状态；time_scale由实例tick应用一次，不把请求秒数或normalized lifetime当实际age。clock只含handle/age_seconds/playing/time_scale/backend/fallback_to_cpu，无DocId、source revision、frame或全局clock。

所有数值/backend拒绝发生在任何play/reset/invalidate之前；step不reset，临时play后先恢复原playing再传播tick错误。严格GPU无CPU fallback时返回 `GpuStepUnavailable`；fallback=true必须带原backend=Gpu及fallback标记，不能当作GPU executor完成。新错误类型不修改旧simulation错误枚举或CPU实现。10项Runtime候选回归与Editor 25项候选分包、分filter，执行均仍0。完整scope、literal边/输入、依赖和静态回执见 [runtime-preview-clock-step.json](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/particles/runtime-preview-clock-step.json)。

此接口尚无生产UI、document factory、资源/代次或render-frame消费证明。动画owner继续拥有timeline/curve/transport，粒子提供实际clock/倍率/seed/reset语义；秒时钟与0..1寿命曲线分离，RGBA共享源key/t且单次事务、仅Linear无切线。原六条animation route不改名冒充粒子动作。旧rewind不作为seek入口，后续预算/整数步/取消等合同仍open；没有新增全局manager、clock、scheduler或store。Parent当前输入/保存/viewport和公共library/styles/export缺口保持。

## combined-lock-r2最新交付：已有resolver与server准备，实际生成仍阻塞

本轮原JM request `combined-ai-particles-lock-resolution-r2` 已有受管prepare/admit/resolve/publish/recover实现和#416管理427 checks；同输入#360生成真实preparation，generationRequestId为 `lockgen-3c23f3791938bf820ae9a2bb8a8734c9718e461fa67311cbf27e00fe`，原及归档preparation SHA均 `fdc33cc1989258cea27f782acedea6f13671a4d2357223a780d6c9a5a1bf56fe`。旧四门表中的“未实现resolver/无server phase”只保留为历史。本域粒子r2的server source revision/测试回执仍为空，不能借JM生成准备或管理测试替代。

当前门槛为：generation专用受信stable>=1.97的独立锁输出能力；policy v11资源/pool与sole canonical-lock writer实际准入；按原preparation对账最新解析输入；真实resolver完整候选/原生EOF、显式候选SHA审阅、before-CAS发布与真实恢复journal。#361的真实资源检查是admitted=false，reservation/writer均null；35GiB reserve及1CPU/2GiB RAM/256MiB disk独立估计保持，14.10GiB为回执观察而非当前承诺。#417只撤销本次未启动检查等待行，不清除blocked generation或未知业务收件。

当前diff/after/native-resolver/publication/recovery均null，captureReady=false，无可信capture时长、不要求全局freeze，原04de锁与四依赖边缺口保持。旧sealed改写和verify_declared_inputs/full-source策略归属与发布验收独立open；receiver单pointer修复及11项pin完整性不授予产品source admission。本域不改policy/manifest/lock或使用旧业务payload。完整证据与原身份在 [jenkins-r2-recovery-continuation.json](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/particles/jenkins-r2-recovery-continuation.json)的currentCombinedLockResolution及combinedPlanOperatorFollowups。Runtime 10与Editor 25项继续分包、分过滤器、执行0。

## Shared rev2与catalog字段公开后的消费合同

Parent的17:49–17:59Z实际认证读回已发布原四主件的样式：Shared rev2/34 colors/3 typographies、21处master style refs，component/main身份保持。本域从Parent报告确认身份和源值，本轮没有实际get-file响应；原consumer11个copy的rev1快照原字节保留。原生library update、覆盖保留/reset、真实style refs与fresh重开仍须在同文件完成，不detach、复制main或写Shared。当前阻塞是19001/16060/19002连接拒绝，不再列“Shared无样式/未发布”。旧rev1/styles0只代表历史。

catalog的NumberField现公开10props（value_text/placeholder加value/min/max/step/large_step/precision/editable_text/layout_stepper），PropertyEditorRow现公开4slots（value/description/source/reset）。源声明审计24项通过，只证明当前catalog与原.zui一致；不代表Field/Number已发布Penpot main或输入/范围/export在产品中通过。复用原value accepts及description/source/reset接受类型；原typed空槽不计完成。域映射见 [具体设计](particles-design.md#当前共享样式和字段值的接入要求)：Seed严格u64文本、容量精确u32、Real发射率/寿命范围及单事务，实际handler需同时保留同DocumentId/session/revision与input owner。

[同任务rev2接续回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/particles/penpot-shared-rev2-sync.json)固定原file/page/2board/11copy及覆盖值和当次失败。服务由原owner核对续接；本域无生命周期/桌面新授权，未重启、续期或重投旧Rust/生成请求。动画/材质专用共享归属、Parent文档与生产viewport/input/save合同保持；发表样式不关闭CJK、正式ZUI导出、正常EXE、150%DPI或两族A/B验收。

## PFX-GRAPH01接续15：真实无连接源与样式消费的交付边界

接受 [共享合同的空端口段](shared-node-graph-contract.md#pfx-接续空端口空边与无连线交互)：ready真实nodes可有inputs/outputs/edges=[]；null interaction只表示无pin/edge hit/hover/candidate/wire，不修改Vec为null。UE Overview schema DISALLOW/false的空FText不作Zircon空白反馈。当前源级System/Emitter投影已在本域source/**实现，使用原GraphNodeView和实际measured bounds/assigned key，统一UnavailableNoSourcePortsOrEdges及原因；没有伪造GraphModel的delta proof/permit或安装新history、默认pin/wire/Material模型。

4个新 `particle_overview_` 候选位于原 `tests/source.rs`，同Editor source_tests::包内当前29项静态定义、执行0；Runtime10不合并。真实五类VFX在已有VFX runtime，缺持久key/pin/edge/layout/Serde；建议原typed依赖所在VFX editor leaf承接只读投影，待Parent明确精确路径/包及Jenkins scope，不向combined锁4边暗加第五边。未来持久schema/合法性/lowering仍PFX，程序化edit仍一次原permit/retention；只读投影不是source edit成功或GPU/frame-ready。

原640叶已补能力原因但没有Graph binding。键盘可读解释/原因弹层、菜单/toolbar/input eligibility、旧gesture取消及真实焦点返回、同代际paint/hit/clip/一次DPI仍由实际consumer/Parent host接续；不能将caption或函数枚举计为行为通过。未复制Material5个业务main、未创建Graph公共UUID。

Shared当前34颜色/3文字样式已从实际JSON响应复读。该同响应21处typographyRefId值全null，本域native Updates最初无项；在自己原11实例通过Assets使用body/strong后，出现真实update banner。Update后的library syncedAt已推进，并经save/close/fresh再次核对11个style/file/shapeRef/中文覆盖及106 source refs。公共4 main/source和自身1 main未改。master引用/自动继承/reset与本地实例styles是不同验收项；普通frame/text/颜色、CJK测量、正式export和两族全局A/B仍open。

完整API/图/leaf/board/shapeRef/override、三次隔离helper失败/恢复及实际保存回执在 [native trial](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/particles/graph-consumer-native-trial.json)。[source packet](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/particles/source-overview-projection.json)与原r2/frozen input/candidate并列追加透明lineage，无新request/attempt/server revision、Cargo或锁生成。实际EXE/资源/桌面时段仍未就绪，passed/accepted=false。

## 动画 r03：原三 main 与当前文档动作挂载

接受动画 [r03 原合同](animation-shared-controls.md#r03粒子第四次接续与实际adapter)与 [交接](animation-parent-handoff.md#粒子第四次共享消费接续r03)。owner file=`b4a1e3ce-9e6a-81fe-8008-d69d60a13f99`、page=`b4a1e3ce-9e6a-81fe-8008-d69d60a13f9a`：source-mapping 回执 rev105、最新保存/fresh rev140，仍 isShared=false；这些是 owner 回执，本轮未重新 API 读取。Parent 17:03:59Z rev100/1358 与 PFX rev30 是各自历史，本域最近已验证 rev38 的 11 Shared copy 仍只消费基础四主件。

| 原专责 main | component / main UUID | draft 与正式源边界 |
| --- | --- | --- |
| Timeline | `8cf70bfd-9275-4bac-849b-b79148b8a4fd` / `4db0829c-abc4-482d-b7b3-155a5d8d75c2` | draft2；WorkbenchTimelineStrip 原单 Canvas 仅内层，未交付外组合/command/scrub slot |
| Curve | `78492c86-3bb4-47dd-97c6-752d6ff37e3d` / `599963dc-b91b-4740-926b-d38154aa8f3a` | draft1；复用原 CurveModel/transform 与粒子 adapter，正式 ZUI URI、typed props/slots 尚未交付 |
| Transport | `982ef940-2239-4167-a24a-56c80ec35fbf` / `44eb5ccd-8ac2-44f2-a4ad-d0d618daa096` | draft2；canonical 原六按钮＋Fill；ClockStatus 为外 wrapper，不是其正式 props 或自走 clock；无公开 context slot |

三 main 不复制、detach 或伪装公共库；不创建 clock、播放器、UUID 或第二 source store。正式 publish/link、source/codec/slot/override-reset/export 由 Parent 和原 pipeline owner 交接。粒子继续负责实际秒时钟/倍率/seed/simulation/reset 与域 transaction，动画负责中立 surface；六个 animation event/route 与旧六 MenuAction scoped diff 保持，未包含 Stop/Reset 分支。

当前消费设计见 [同实例时间与动作状态](particles-design.md#动画-r03同实例时间曲线手势和动作状态)。RGBA 横移以 emitter/source key 归并一次 mutation/inverse 后重投影；分量 value 不改其它分量。adapter 不拥有 history/revision/retention/gesture，单 key 不声称 multi-key 原子批次；仅 Linear/无 tangent、HDR 有限不夹0..1。实际 simulation 秒与 normalized lifetime t 独立，不由 System 时钟推造统一粒子 cursor。

Stop/ResetSimulation 在原文档 host 组合已存在标准 Button/允许 action；PanelHeader.actions 的六类白名单不含整个 Transport。原 Space 两挂载点不是公开 slot。Stop 目前 descriptor，Reset 没有已交付 handler/正式事件，不能以旧 rewind/Previous 或 icon 替换。外层仍须真实 context（project/document/view/source、source/install generation、backend/session/handle、capability、request sequence/typed units）、eligibility、ACK、有效 opener focus 与最后消费者生命周期；Clock 六字段不扩成该 context，准入也不等于完成。

Runtime 五路径与 Parent 记录和原 packet 仍匹配，已非“没有源码”；数值/backend 拒绝先于副作用，但 Simulation 错误不保证 tick 回滚，须同 context 仍有效才做实际 clock 读回。单 tick 不是完整 bounded seek；旧持锁 f32 remaining rewind 保留且禁止生产使用，整数步/total-slice 预算、资源/取消/新请求/关闭与 frame 关联继续 open。Editor29、Runtime10 均执行0，不相加或覆盖原 r2。

本轮只读对账 r03 的20项源为18匹配：本域 tests/source.rs 的借用修正已在 stage5 记录；公共 catalog 新 e4bba09c…保留原位，旧4c2f849c…/24项历史检查不充当新整份回执。当前19项 catalog 合同与2项源一致性检查、具体 hash/原提案与未满足项在 [r03 消费机器记录](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/particles/animation-consumer-r03-reconciliation.json)。仅四文档和本地 lineage 更新，无公共/PFX Rust/ZUI 写入、新 Jenkins request/query、Penpot/桌面/服务或 capture；passed/accepted=false。

## PFX-GRAPH01 后继：VFX 三文件交付与连接能力边界

Parent 第五次指定的 `zircon_plugins/rendering/features/vfx_graph/editor/src/{lib.rs,overview.rs,tests/overview.rs}` 已完成源码切片，之前“待 leaf 授权”是历史状态。借用原 runtime payload 和共享 GraphNodeView/GraphEdgeView/GraphNodeBounds；不新增 PFX 依赖、持久 schema/DTO、codec、Material 模型、相机、clock、GraphModel 或 layout store。完整 before/after、五文件 leaf 清单、8 项原依赖与5项未运行回归在 [VFX source packet](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/particles/vfx-source-overview-projection.json)。

| 合同层 | 当前责任与证据 | 仍开放的接线 |
| --- | --- | --- |
| VFX 实际源 | 五类 typed payload、重复/顺序/空源保留；只读 locator 绑定精确 source/revision，布局由真实 host 测量并校验 | 持久 author key、typed edit/undo、真正 lowering/compile/install 及 preview/frame 关联仍属域与生产链，不能由只读 index 替代 |
| 共享图能力 | ports/edges 都为空是合法只读投影，无连接交互；空边不构成通用禁用条件 | handles/菜单/toolbar/快捷键/程序入口使用同一实际域 eligibility 和原因；其它合法 ports 的 required-input/首条边规则保持 |
| Parent 宿主 | source/revision/layout/input owner、retained geometry、一次 DPI、生命周期/焦点归真实宿主 | GraphCanvasState 没有 source-bound link-gesture 生命周期；切源取消原 gesture，不产生 delta/inverse/undo，不能只清域 locator 就称拖线已退役 |
| Jenkins 原 owner | VFX 单包/lib/defaults off/features=[]/test/locked/jobs1/threads1/overview_tests:: 仅为未来意向 | 先有受支持 recipe 及真实 source/resource/pool/cache/lock 准入；不复用原 Editor r2、Runtime10、Material 或 AI 资格，不发新身份/QueryOnly |

直接消费 [共享图 PFX 空端口段](shared-node-graph-contract.md#pfx-接续空端口空边与无连线交互)。材质勘误两源和原 JSON 已实核，原 Shared 记录34 colors/3 typographies 不回写为0；master 有效 style refs0不表示发布集合0，也不表示跨文件继承/同步/A-B通过。原 GraphModel/DeltaCommand、retention/revision/一次 permit 保持，无公共 Graph 源变更或新主件 UUID。材质 r4/#364 的 ABORTED/operation-running 冲突只归其原 Jenkins 操作核对；不能用于 PFX 准入、清 writer 或 native 完成结论。

[本域勘误消费回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/particles/material-erratum-vfx-consumer-reconciliation.json)的34项是 hash/合同检查，不是Rust或画面。原VFX源码与静态证据、PFX原r2 identity/candidate/journal 均保持；本次仅更新既有四份本域计划和验收要求，不改公共主题/catalog、三个原ZUI或任何Rust。官方导出、实际输入/帧、正常EXE/actual150%DPI/双端逐图仍open，passed/accepted=false。

## Shared rev5 后继：单项颜色引用已消费，typed/专责组件仍缺

本域20:32–20:48Z认证API确认原Shared rev5：4main身份保持、34colors/3typographies；原Button main有1处fillColorRefId绑定editor.surface.1，typography非空引用仍0。原rev2/有效refs0记录保留为历史。粒子原两个Button root经原生库“更新”继承同一style/file引用，无本地fill覆盖；11个caption本地覆盖及106个source refs不变，所有source refs当前仍有效。新增绑定不是新main，Field/Number及三动画专责发布、Graph正式main/source/codec/slot仍未交付。

原文件rev39保存/fresh证明库引用消费，随后原gap note文字纠正保存/fresh为rev40；业务页及私有main保持，未detach/复制或写Shared/Material。原生fit-all只调整view camera，完整截图实际80%缩放；100%裁切原图、selector错误与DOM正文等待超时均保留于[本轮回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/particles/penpot-shared-rev5-consumer.json)。这一项引用继承不授予跨族全局A/B、自动override reset、官方导出或EXE/150%DPI/1DIP通过；PFX Editor29/Runtime10/VFX5仍各执行0，原请求/operation/journal/锁均不变。

## Shared palette 消费：保留业务 frame 和源组件边界

原Stack页14个非组件业务frame已直接消费现有Shared file中的两项颜色：surface.0=`5edd281a-cc68-597c-a6fa-1234e164adc8`（两处#151515），surface.1=`599586e3-022c-500b-ac80-5d9c490139af`（十二处#242424），opacity均1。通过真实原生API保存rev41并关闭fresh重开；不是重新发布样式、复制main或给linked copy增加fill override。所有106个source refs、18copy和原业务main因完整masked-data比对保持。

公共owner继续拥有Shared master默认绑定、Field/Number与专责组件发布、原生slot/source/codec、全局传播/reset及导出。此次只读浏览器加载0次update-file，单个自有helper已exit0、两个context/browser关闭，tmp无文件；原报告和失败保留。其同色值board像素无变化，不证明产品A/B；真实typed操作、host/save/preview、正常EXE/150%DPI及双端逐图门均保持。详见[palette消费回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/particles/penpot-shared-palette-consumption.json)。

## 粒子曲线批量 mutation 的实际交接

粒子现提供 `ParticleSourceCurveAdapter::apply_batch(&[ParticleSourceCurveEdit]) -> Result<Vec<ParticleSourceCurveEdit>, ParticleSourceError>`。沿已有 selected emitter、stable source key/channel 和 borrowed source，不新增中立 Curve DTO、history、clock、store 或公共组件。原 `CurveModel::Delta = ParticleSourceCurveEdit` 及单点 `apply` 保持；单点现在复用 singleton batch，批量消费者须调用显式方法。

一次 gesture 的所有 expected 均对原源检查，RGBA 共用目标时间并合并各分量；相同分量重复或同 source key 目标时间不一致拒绝。校验最终 times（包括未选 key），允许同时交换合法 key，不允许重复时间；任意一项失败都在源写入前。返回同序 inverse 列表，宿主必须把整列表留作一次已准入操作，不能逐项提交 permit/history 或用四条独立撤销冒充一次 RGBA 拖动。多 emitter/多文档批量不在此接口范围。

动画 owner继续拥有中立 Curve/input/selection/gesture与专责组件；Parent继续拥有实际 document/session/revision、factory、permit/retention、dirty/save、preview install及焦点。上述 API 没有提供这些公共接线。当前宿主尚未消费该方法，真实 undo/redo/cancel、source切换拒绝旧 gesture 和保存/预览联动均 pending。

[本域增量 packet](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/particles/source-curve-batch-mutation.json) 的5个新静态定义使 Editor21＋13＝34，Rust执行0；Runtime10/VFX5另包另过滤器且未改。原 r2 ordinal6与7份历史packet保持，不重投身份或测试，源/preparation修订交原 owner。原共用库、动画三 main、六 events、三个 ZUI、锁及服务/桌面权均未修改。

## Burst 输入合同：秒、完整 u32 与一次源 pair

本域增加 `ParticleSystemSource::validate_burst_edit` / `apply_burst_edit` 和 `ParticleSourceBurstEdit`，输入为 stable emitter key、stable burst key、expected 与 replacement 的 `(time_seconds, count)`。先对原源预检、分配 inverse，再同时写两值；只读 eligibility 在 apply 时重新检查。数组索引、显示名和发生时间均不作为身份；相同秒值不合并，零 count 不删除。返回 inverse 由真实宿主按同一 document/session/revision、一次 permit/retention/history 消费，不能单凭 DTO 或方法名声称安装了 handler。

时间是未归一化 simulation 秒、有限且非负；count 是完整 u32。`parse_particle_burst_count` 接严格 unsigned decimal 文本，直接解析到整数；公共 NumberField 的 f32 value 无法保证 16777217 或 u32::MAX 精度，消费者须保留整数文本到域边界，不能先舍入再解析。此合同只消费既有公共字段能力，不增公共 master、slot 或平行字段实现；现有 u64 seed 严格字符串合同保持。

[源码 packet](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/particles/source-burst-edit-r01.json) 的5个 burst 回归只是未运行候选，Editor39/Runtime10/VFX5仍分开。模板0 burst、显式双 burst 夹具与真实 preview/frame 分列。正常 EXE 的输入、一次撤销/重做、重置、实际播放、保存重开和精确错误返焦仍需端到端证据；监督恢复允许这些依赖并行接续，不等待公共组件全部完善才推进本域。

## 曲线数组进入现有 CurveModel：本域已实现，宿主仍有明确边界

`ParticleSourceCurveAdapter` 现在实现 `CurveModel<CurveId = ParticleSourceCurveId, Delta = Vec<ParticleSourceCurveEdit>, Error = ParticleSourceError>`。原 `apply_batch` 继续作为批量实现；trait 的 `apply` 一次委派并返回完整数组。此前单点 Delta 合同由本次接续替代，原观察不回写。已有13个曲线回归已迁移，不需要另建 domain-neutral DTO、第二 adapter 或公用组件。

Parent/动画消费方应在同一真实 document/session/revision 内收集一次 gesture，先以现有 retention/proof/一次 permit 准入整个 forward/inverse/预备峰值，再消费完整数组；不能在控件 callback 中直接借源调用来绕过命令引擎。`GraphDeltaCommand` 的实际约束为 GraphModel/GraphEditContext，当前 CurveModel 不满足，不能靠加一个假的图模型取得通行。AnimationEditCommand 的现有 authority 只覆盖 animation_documents，也不能借来承载粒子文档。

共享宿主尚无本域可应用的粒子 source lookup/command authority，故本轮不伪造安装它的公共补丁。已给出[实际接口与精确调用片段](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/particles/source-curve-gesture-host-integration-r01.md)，指出真实文件、输入、提交后 dirty/preview/save 和失败焦点条件；公共 source 仍由原 owner 集成。域 inverse 保留、宿主文档 revision 和 SaveCtx durable receipt 是三种独立证据。源packet为[source-curve-gesture-boundary-r01](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/particles/source-curve-gesture-boundary-r01.json)；Rust、真实手势和正常EXE/DPI仍未运行。

## PFX-SHARED-07/10 的实际阻点：空 Slot 不能靠绘制改写 copy 结构

本轮已经实际尝试原1280板的发射率value_slot：`114b6636-4013-80fa-8008-d6e5aeb73c6e`，其linked instance为 `114b6636-4013-80fa-8008-d6e5aeb73c6c`；源component/main/slot及四实例均保留。Native Text工具没有进入文本编辑状态，尝试后关闭/fresh的PFX rev49与Shared rev5响应hash完全不变，四值槽依然空，不能把读到的真实模板值当已挂载。

已排除“slot被锁定”推测；实际slot无blocked，原生图层按钮是可锁定状态。Penpot `frontend/src/app/main/data/workspace/drawing/box.cljs:107`明确选`get-first-valid-parent`，`common/src/app/common/types/container.cljc:410`明确定义跳过component copy。这只证明该普通绘制路径不能把新child加入copy，不等于完整调查了所有Penpot slot/instance-swap功能，也不是locator超时的唯一已证原因。

需要公共owner交付的具体合同是：在原canonical PropertyEditorRow/value Slot中，怎样原生消费现有Text/实际typed控件而保留linked instance/source refs，怎样保留覆盖、保存fresh和reset，并由正式plugin导出为原`.zui`的value slot内容。当前未提供可验证recipe；不要用copy树API强写、detach、复制master、槽外覆盖文字或占位Field来替代。本域提供实际4组slot/instance身份、真实模板文本与失败原图，[回执](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/particles/penpot-source-values-r01.json)中字段行为与产品gate保持false；无需等待该共享交付才继续域代码。

## Burst 捕获/文本更新接口：复用现有 Edit DTO

本域原 `ParticleSystemSource::capture_burst_edit(&self, emitter_key, burst_key)` 现在校验真实源与stable keys，返回现有 `ParticleSourceBurstEdit`，expected/replacement均为同次捕获的time/count；不会assign_missing_keys。原DTO的 `set_replacement_from_text(&mut self, time, count)` 先解析整对，成功才替换replacement，失败不动delta且绝不重取expected。完整u32计数不经Real；时间有限非负，非零下溢到零拒绝，Real可表示的正小值没有新EPSILON限制。

宿主应在同一原document/view/source revision捕获这份payload，沿原input owner持有raw draft，并在原authority下prepare/commit一次。capture和校验本身需要覆盖keys/BTreeSet/错误路径及payload分配的真实retention峰值，方法名不授予permit。commit先复核context/revision和`validate_burst_edit`，之后才检查`is_noop`；stale的no-op也必须拒绝。只有合法non-noop才进入一次原history/dirty/preview投影，inverse/redo始终是整对；保存仍经原SaveCtx/source lock/dirty token与durable receipt。

已经有的typed调用片段（位于原文档已准入范围，非新handler）：

```rust
let mut edit = source.capture_burst_edit(emitter_key, burst_key)?;
// 原input/gesture owner保留同次context/revision和raw文本，不在commit重新capture。
edit.set_replacement_from_text(time_text, count_text)?;
// 原authority/revision已复核后，先preflight，之后才能以is_noop决定跳过作者历史。
source.validate_burst_edit(&edit)?;
```

实际一次 `apply_burst_edit(edit)` 仍只能在原已准入命令中调用，并保留完整inverse；上述片段没有实现Toolkit/host/history/save或preview。新增5个定义使Editor44静态，cases.rs1在filter外，Runtime10/VFX5另包且执行0。原r2 identity/input不改，当前19源完整观察与父合同消费登记于[本轮packet](E:/cargo-targets/zircon-local/reports/ue-window-suite-20261024/particles/source-burst-typed-caller-r01.json)，不新建server revision或重放终结operation。
