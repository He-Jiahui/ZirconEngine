# MAT-SHARED-001：共享节点图接入合同

牵头：材质聊天 `01a16d50-6c7f-7370-853d-4581184b66a1`；跨域协议、公共组件/主题、宿主和集成：Parent `01a16d4e-0491-7353-84fa-038bd7706546`。本合同接续 [Parent 分工](../../../.codex/sessions/ue-window-suite-parent-01a16d4e.md) 与 [材质计划](material-family.md)，不是新建第二套图模型，也不是共享源码写权、已发布组件或产品通过证明。

当前Shared边界：原PFX回执的rev2已含34色/3文字样式，consumer own styles0误写为库0的勘误与原JSON/SHA保留；当时master有效引用0仅作历史。原public owner于2026-10-25T03:45Z复读Shared rev5，Button main有1个有效颜色引用、typography0；Material69根sourceRef有效，但640对颜色/字体引用全null、有效0。发布或1个绑定不等于传播/A-B/产品通过，例子leaf真实归属TreeRow的更正见[公开引用边界](material-family.md#shared-rev5-公开引用边界)；PFX原时点见下文[PFX勘误](#pfx-接续空端口空边与无连线交互)。

公共源码先前按 Parent 授权限于 `zircon_editor/src/ui/graph/{mod,routing,node_widget}.rs` 和 `tests/cases.rs`；第五次AI接续另明确授权 `canvas.rs`、必要的 `mod.rs` 导出与原 `tests/cases.rs` 补充 bounds-fit，见下文 AI-C06 接续。已有 `GraphDeltaProofV1`、`PreparedGraphDeltaV1` 重导出保留；兼容的四向路由、归一化切线/目标箭头和附件原 ID/host 投影保留所有旧字段、默认水平结果与调用方。model、commands 及 retention/revision/prepare/一次 permit 只读不变。生产域适配/宿主仍未接入；公共 catalog、tokens、Starship、导出插件及原生捕获协议未改。源文件/hash、静态检查与真实 Penpot 观察继续登记本任务 `artifacts.json`，受影响受管编译另取实际回执，不能把辅助 API 当产品通过。桌面输入尚未授予。

## 已有基础与最小缺口

以下路径均相对 `zircon_editor/src/ui/graph/`。本次 scoped 检索仍只找到 `tests/cases.rs` 的两个 `GraphModel` 实现；另已定位 AI 域对四向路由的真实源码调用，见下文 Widget 接续的消费证据。该 AI adapter 明确不实现 retained-owner 协议，也未找到生产 retained 调用方；不能把 trait、路由调用或候选资产当任一域已接通。

| 已有来源与真实能力 | 直接复用与最小待补合同 |
| --- | --- |
| `model.rs`：`GraphModel`、`GraphNodeView`、`GraphPinView`、`GraphPortRef`、`GraphEdgeView`、`GraphAttachmentView` | 域持有唯一作者模型和 delta；表面读取节点/端口/边，不生成材质/BT/动画的第二份资产。空 pin/edge 列表可表示真实无连线图 |
| `model.rs`：`GraphDeltaProofV1`、`PreparedGraphDeltaV1` | 此前类型已定义但 private model 未重导出；本轮已仅在 `mod.rs` 增加原两类型的公开路径，model 及全部 retention/revision 合同不变。外部真实 adapter 的编译/运行证据仍需分别取得 |
| `canvas.rs`：`GraphCanvasState`、`GraphSelection`、`GraphNodeDrag` | 复用 pane-local 变换、缩放锚点、选择/框选和拖动结果；新增 `fit_graph_bounds` 消费预计算 bounds，源码及七项回归定义已写，未编译/执行或接真实宿主。当前选择是 node ID 集合，没有 lead/anchor、pin/edge/attachment 焦点；需要宿主视图投影，不伪称已有完整键盘选择模型 |
| `commands.rs`：`GraphEditContext`、`GraphDeltaCommand`、`GraphClipboardModel` | 复用现有事务引擎、revision-bound proof → permit → prepare → apply；Target 必须定位实际 asset/document/graph/subdocument。禁止 renderer 越过命令生命周期直接调用简单 `apply` |
| `routing.rs`：`route_connection`、`GraphConnectionRoute`、`GraphRouteStyle` | 原横向算法/字段/getter 保留；可选 `route_connection_with_directions` 接收独立于 Input/Output 的 Left/Right/Up/Down 朝向，`endpoint_tangents`、`target_arrow` 由同一路径返回图内投影。尚无生产 paint/hit 调用，非 UE 精确连线/调试样式验收 |
| `node_widget.rs`：`GraphNodePresentation`、`GraphAttachmentPresentation` | 原 title/pin count/attachment labels/selected 不变；新独立 `GraphAttachmentPresentation::from_view` 保留附件原 ID、host ID、名称/类别和原顺序。未生成 bounds/instancePath/焦点；实际 retained node/pin/edge/attachment 绘制/命中仍需 Parent native host 接入 |
| `model.rs`：`ConnectVerdict`、`default_connection_verdict` | 当前只有 Allowed/Rejected，默认严格同型；Tree 只额外限制同一 target port 入边。原子替换/转换的预检与 inverse 留在域，不能用默认规则代替整树合法性或所有域的类型系统 |
| `model.rs`：`GraphDiagnostic` | 当前 message 为静态字符串；异步编译错误、调用栈、source map 和动态文本由原域提供并接既有诊断呈现。不能把一个静态 port 错误类型当完整 compiler/debugger 接口 |

## 消费者与业务 authority

| 消费者、既有合同 | 域提供的真实数据与语义 | 共享表面必须兼容 |
| --- | --- | --- |
| 资源 Reference Viewer；[RT-S01～09 与只读依赖图提案](08-resource-tools-shared-contracts.md#材质-owner只读依赖图消费提案)；owner `01a16d53-60db-7d02-b03e-0042f277783d` | project activation/catalog revision、真实 asset UUID/locator/known-project 状态、有方向 references/used_by 与 provider 给出的依赖信息；资源命令中心负责双击、Find-in-Browser 和实际 Toolkit 路由 | 只读图复用 pan/zoom/frame、稳定命中、选择/焦点和菜单锚点；不消费材质端口/编译模型，不产生作者图 delta。当前未申请音频节点图，音频后端缺口保持开放 |
| Material/Instance/Function；[材质计划](material-family.md) | MaterialGraph 节点/端点、参数值、材质 Dag、单输入替换、允许的显式类型转换、真实编译/函数依赖；Instance 属性不是另一个图模型 | 水平 pin/Bezier、无结束箭头的材质连线、节点内部 flow、图参数 Details/诊断跳转。当前六节点不是任意函数或 shader graph 支持 |
| 动画；[具体设计](animation-design.md)、[共享控件与图交接](animation-shared-controls.md#已核对的域消费者)；owner `01a16d51-741b-73b3-a61a-2189cd38f81e` | 现有 Graph/StateMachine 文档/history、Clip/Blend/Additive/Mask/Output schema、entry/transition/条件、域编译和播放/source locator；稳定 edge/子目标与生产适配缺口见下文 | 按动画适配节保留 `AnimationGraphPaneBodyRoot`、`AnimationGraphCanvasSlotPanel` 与 `animation_graph_canvas_slot`；公共层复用身份/变换/焦点/事务，动画保留唯一资产/history/运行语义，不把 selection 当活跃状态 |
| 粒子；[PFX-GRAPH01 / PFX-GRAPH-01](particles-design.md#新确认的具体差异与提案)、[共享需求](particles-shared-contracts.md#给材质owneroverview与有限vfx节点)；owner `01a16d52-a4a5-7271-90a2-336091337737` | 现有 emitter/curve 作者源有稳定 key；VfxGraphAsset 五类 node 列表已有真实 lowering/prepare/executor/readback 源码，但无 stable node/edge/pin。领域持久 schema、合法连接、诊断映射和 preview 替换归粒子 | 按下文粒子适配合同复用无连线 Overview、选择/变换和事务协议；完整 node 编辑需实际 source adapter，连接能力继续关闭。旧“全部 no-op”已过期，真实 Editor/正常 EXE 仍未接通，不借 MaterialAsset/材质 compiler 充当 VFX 模型 |
| AI；[AI-C01～12](ai-design-and-acceptance.md#ai-接入现有共享图基础的合同) | 原节点 ID、单父/有序 children、root/child-count/整树约束、一次 Reparent、真实调试帧；Decorator/Service 业务语义不转交图层 | 上下布局/箭头、有序端点、独立附件命中、原子替换反馈、真实运行样式。先保留实际 wrapper 拓扑；没有完整 trace 不合成祖先 active path/bubbles |
| Widget；[UI-SH-GRAPH-01](widget-ui-design.md#共享-owner-的具体消费合同)、[W11/W12/W20](widget-ui-inventory.md)；owner `01a16d52-d54d-7ac0-b0d5-21864f383a1e` | 实际 widget/event/variable/schema key、bindings、图文档存储、命令/history 和执行/编译 source locator、Designer 选择关联 | 按下文 Widget 适配合同消费同一表面；Source 继续是文本。材质 compiler、MaterialAsset 和本域六节点 palette 均不作为跨域模型 |
| 角色 Control Rig / Retarget；[CS-GR01](character-skeleton-shared-contracts.md#cs-gr01共享节点图表面)、[设计 5.3](character-skeleton-design.md#53-ik-rig--retarget--control-rig)、[公共合同 10](character-skeleton-design.md#10-给公共-owner-的最小合同清单)；owner `01a16d52-6b1e-7543-82f0-4f05ca92ad58` | 实际 Rig/typed ports/控制/solver/执行顺序、OpStack、骨架资源 revision、pose、持久化及域命令/history；当前只核实 IK 数学子集 | 按下文角色适配合同复用稳定身份、图变换、选择/焦点与命令协议。Retarget 有序 OpStack 仍用列表/树；RigVM/Modular Rig、求解和保存保持域能力缺口，不用 MaterialAsset/材质 compiler 代替 |
| Actor/Blueprint；[AB-SHARED-02](actor-blueprint-shared-contracts.md#给材质-ownerab-shared-02) | 类/事件/构造/函数/接口语义、typed execution/data pins、动态 pin/split/recombine、域 delta、compiler/VM source map | Target 区分 asset/document/graph/subdocument；稳定 schema key 与显示名分离，支持域 verdict/转换/替换；没有后端的模式继续列缺口 |

时间轴、曲线、transport 继续归动画聊天 `01a16d51-741b-73b3-a61a-2189cd38f81e`；graph 不创建另一套时钟、轨道或播放状态。公共树、Details、菜单、弹层、Dock、文档保存及预览 renderer 归 Parent 协调；各域提供自己的真实命令、guard 和 source mapping。

区域与搜索的结构参考沿用材质计划登记的 App Shell 4、Command Menu、Settings Form；节点图的具体视觉、端口/连线、菜单和操作行为仍以本地 UE 源码为准。这里只细化既有区域和组件的接入合同，未据外部模板推定产品几何。

## 组件内容、身份与测量合同

下表是需要由 Parent 对应到既有 catalog/native host 的语义槽，不是已注册的新 component ID 或 Rust schema。复用现有 Label/Icon/Field/NumberField/PropertyEditorRow/Menu 等原件，不在各族另建同名主件或主题。

| 内容/命中目标 | 必需身份、布局和状态 |
| --- | --- |
| Graph pane | 实际 document/graph target、source revision、surface/session generation、可编辑能力和 graph transform；pane 大小由 Dock 流式分配，普通工具栏不采用图坐标 |
| Node body/title | 原 NodeId、源文件/节点/实例路径、标题和可编辑字段的真实数据；位置来自图内容坐标，size 来自文本/内容测量。内部标题、字段、pin rows 用 flow；selection/focus/error/runtime 状态分别输入 |
| Pin row/anchor | 端点使用 `GraphPortRef(node_id,direction,pin_name)`；`pin_name` 用稳定 schema key，显示 label 另行投影。重命名、Function 端口或 split pin 不能用当前显示名/数组下标恢复关联；domain 明确 remap。绘制锚点和命中 bounds 来自同次布局测量 |
| Edge/预览连线 | 身份至少是稳定两端及域的次序/连接 key；不能只用当前 Vec index。真实 edge 与尚未提交的预览线分开；绘制、命中、hover 和 sourceRef 使用同一路径/宽度/变换 |
| Attachment | 原 attachment ID、host node ID、类别、独立 bounds/text line boxes/instancePath/选择和焦点；不按重复 label 命中，不为了缩成子卡片删除实际运行节点 |
| Editable content | 显示值、draft、enabled/readonly、类型错误和 commit/cancel 由域提供；NumberField 范围/精度不能用公共样例默认值替代；IME/长中文保持实际行框 |
| Diagnostics/导航 | 当前请求与 source revision、node/pin/field/调用链定位、severity 和 stale/last-good 状态；点击只跳转真实对象，过期 source map 不按旧 index 跳新节点 |
| Palette/context menu | 实际 `GraphNodePaletteDescriptor`/域 schema 与 capability、触发 node/pin/位置和当前 selection；不从 class 名猜创建参数。Enter/拖入/快捷创建进入同一业务命令，取消还原触发焦点 |

`GraphPinView.name` 目前兼作 key 且无独立 label 字段；稳定 key+显示 label 需要 host 投影接续，不能只在 Penpot 改名后宣称产品端已经分离。是否扩展既有 projection 或通过既有属性/slot 携带 label，由 Parent/native owner 在实际 source closure 中定稿；不改变域存储的稳定身份。

## 输入、候选连接与事务

所有物理输入先减同一帧 pane 物理原点，再除当前真实 DPI，得到 pane-local 逻辑坐标，然后调用 `screen_to_graph`。绘制走 `graph_to_screen`，由 host 施加一次 DPI。DPI、zoom、坐标须有限且有效；布局、命中、context menu anchor 和证据几何共享同一 transform generation，resize/滚动后不复用旧 pin anchor。

| 手势或命令 | 表面产生的意图 | 域提交及取消约束 |
| --- | --- | --- |
| 点击/框选/键盘选择 | 原 NodeId/子目标、selection set 与合法 lead/focus；向域 selection 投影时带 generation，避免递归同步 | 选择和 pan/zoom 不产生资产 Undo；列表/Details/图保持同一文档的选择 authority。目标删除则回仍存在的节点/画布 |
| 节点拖动 | `begin_node_drag` 捕获选中节点初始位置；中途 `moved_nodes` 只做预览 | release 把 moves 转一个域 delta；Esc、capture lost、文档切换/关闭或 revision 失效终止手势且不提交旧值。drag 数据也需受既有预算约束 |
| 接线/重接 | 源/目标原端点、expected revision、候选位置；readonly preflight 返回是否可连及实际将受影响的边/转换 | single-input replace 或 AI Reparent 先在候选图移除旧边，再校验整图；成功一次提交旧边+新边+顺序/转换和 inverse。不得先断开、失败再留下半张图 |
| 类型转换 | 显示实际域允许的转换和新增节点/边影响 | 不是默认 strict type verdict 自动许可。无已实现转换能力则明确拒绝；不能画成功线后丢掉数据 |
| 搜索/创建/连接创建 | 同一 palette entry、真实 schema/defaults、graph anchor、可选来源 pin | 创建和自动接线一次域事务，失败无 mutation；名称冲突、required 参数和引用校验由域完成，不用固定名称填假节点 |
| Delete/Cut/Copy/Paste/Align | 当前文档和真实 selection，clipboard 域序列化/稳定 ID remap；Align 用既有 `aligned_node_moves` | 全部进入同一 history context；删除节点/附件/边的闭包与 inverse 由域给出。没有 codec 或外部引用闭包时不伪造可用操作 |
| 文本/弹层 | 文本框/IME/颜色或资源选择器先消费输入；图仅在实际 focus scope 内处理快捷键 | 全局 Save/Undo/Delete/Space 不被图层抢占；弹层关闭回原 pin/node/field，触发目标已删除则回合法 canvas。dirty/save/close 复用 Parent 文档 authority |

当前 `ConnectVerdict` 无 Replace/Conversion 变体；不在本域私增共享枚举。优先让域的 readonly preflight 给出 revision-bound 效果，再由同一域 delta 提交；Parent 若确认公共 host 无法呈现该结果，再共同增补中立投影。候选图/forward/inverse 的 retained pair 与 materialization peak 必须遵守原预算，错误保证模型未变，不能通过先克隆整图绕过证明。

## Actor 消费适配：AB-SHARED-02

接续 [Actor 原合同](actor-blueprint-shared-contracts.md#给材质-ownerab-shared-02) 与 [Parent 分工](../../../.codex/sessions/ue-window-suite-parent-01a16d4e.md#actorblueprint-交接后端与正常窗口配方)。Actor owner `01a16d53-8965-7472-931c-73cba748fe8e` 负责真实类、事件、构造、函数/接口 schema、持久化、delta 和 compiler/source map；Parent 接宿主及脚本/VM/场景生命周期。共享 graph 只提供中立表面，不承担 Actor VM，不以 MaterialAsset、材质 Dag 或材质 compiler 替代。本节是语义适配合同，尚无新增 public DTO/trait、Actor 图 adapter 或完整 BP backend。

| 边界 | 本次明确的消费合同 | 现有能力与待接续 scope |
| --- | --- | --- |
| Target / retention revision | Target 区分实际 asset、document、graph、subdocument，并对应唯一当前 history context；同资产不同子图不能只按窗口标题或 URI 混用模型。command/preflight/source map 使用同一 source revision，materialization/apply 校验原 retention revision；文档关闭、重载或目标失效拒绝旧 mutation | `GraphEditContext::graph_model[_mut]` 已按泛型 Target 获取模型，`GraphDeltaCommand` 要求 Target 为 ImmutableRetainedOwnerV1。具体 Target/schema 和文档生命周期由 Actor/Parent 提供，不在公共 graph 新建第二份文档存储 |
| Node / pin / edge 身份 | NodeId 来自真实作者模型；`GraphPortRef.pin_name` 约定为稳定 schema key，显示名另行投影。显示重命名不改变连接 key；真正 schema 身份迁移、动态 pin 增删、split/recombine 由 Actor 给出明确旧→新 remap、受影响边/默认值/源映射及 inverse。重复显示名、数组重排不能改变真实端点 | 当前 `GraphPinView.name` 兼作 key，没有独立 label/pin ID，`GraphEdgeView` 仅端点。先由 Parent/native owner 定稿现有投影中的显示标签和 source locator；若必须改 `model.rs` schema，先交具体兼容 diff 与消费者，不能借既有路由辅助 scope 自动扩权或在 Actor 另建端点系统 |
| 类型与连接事务 | execution/data、object/interface/class/容器/wildcard、方向、转换与替换由真实 Actor schema preflight 给出；同一 delta 提交旧边移除、新边/转换节点、顺序与 ID remap，失败无 mutation，Undo/Redo 恢复原身份和关系 | 现 `ConnectVerdict` 仅 Allowed/Rejected，默认严格同型不能证明 Actor 合法性。沿既有域 preflight+delta 呈现实际效果；缺 schema、转换或 compiler 就拒绝并保留原因，不画成功线。UE K2 响应是行为参考，不导入 K2/UObject 模型 |
| 同帧布局、DPI 与拖动 | pin anchor、path、hit bounds、菜单锚点和证据来源于同一 graph/transform generation；物理输入减 pane 原点并除实际 DPI，再走 canvas 逻辑变换，paint 由 host 只施加一次 DPI。节点内部 title/字段/pin rows 用 flow；drag 仅预览，release 一个位置 delta，Esc/capture lost/切文档/过期 revision 取消 | 现 `GraphNodeDrag::moved_nodes` 只产生位置结果，不写 DTO；实际 retained paint/input、焦点/命中与 sourceRef 仍由 Parent 指定 consumer 接通。cancel 不创建 history，不把 pan/zoom/选中写入 Actor 参数 |
| 预算与唯一 history | 复用 proof→一次 permit→prepare forward/inverse→apply prepared，Target/label/delta/selection/remap 及候选峰值遵守原 retention bound/revision；跨 scene-instance 与资产、子文档沿正确 context 提交 | 不改公共 commands/retained-memory，不直接 apply，不复制整图快照栈。域 clipboard、动态 pin/split、引用修复与 inverse 只有在真实模型和预算证明具备后才启用 |
| diagnostic / source map / 跳转 | 每个结果绑定原 compile request、target、source revision/hash、真实 node/pin schema key/field 和调用/子图定位；选中结果只跳当前可解析对象。旧结果标 stale，不按旧 index/显示名跳新 pin，不把 missing pin 当编译成功 | `GraphDiagnostic` 当前只有 node_id、port、静态 message，没有请求/修订/动态文本或 VM source map。Actor 提供真实 compiler 结果和版本映射，Parent 接既有诊断/任务呈现；中立适配不能伪造已发布 SourceLocator API |
| 菜单、文本与证据 | 创建/拖线创建/快捷键来自同一真实 catalog/schema 与 command；IME/文本 Undo/Delete 优先，弹层取消返回有效 trigger。逐 node/pin/edge/text 保留 sourcePath/sourceNodeId/instancePath、revision、transform/DPI 与 native geometry | 类/接口/构造/事件存储与执行尚缺，Source 文本或静态画线不算 Graph；断点/高亮由真实 VM feedback 提供，不从 selected/播放按钮推断。原 ZUI 的 ID/绑定/事件由 Actor 正式导出时保留 |

本次只更新本域两份文档，未写 Actor 原 ZUI、Prefab/VM/runtime、公共 graph、catalog/tokens/Starship 或 Parent 记录。通用辅助触点仍沿既有 `ui/graph/{mod,routing,node_widget}.rs` 与 `tests/cases.rs` 提案；pin label、动态诊断/目标 schema 需要的额外公共修改另交准确 diff。实际 Actor adapter/graph 文档路径由 Actor owner 提供，不把现有 `zircon_plugin_prefab_tools_editor` 注册当成完整 BP 实现。

聚焦验证分开取得证据：公共修改后用既有 `zircon_editor` lib 的 `ui::graph::` 子集复核逻辑坐标、drag、schema 与预算，不运行整个 workspace；Actor 真实 adapter 到位后验证两文档/两子图 target 隔离、retention revision/stale/预算失败不变、重复 label/改名/动态 pin/split-recombine 的 ID/remap/边/默认值及一次 Undo/Redo、release 一次 delta/Esc 原样取消、异步诊断新旧请求乱序和 source map 跳转。没有真实 adapter/测试路径前不提交空 filter，执行前由实际 owner 声明唯一包、features、target/profile、精确命令与物理构建池。正常 EXE 的三个尺寸与实际 150% DPI 再验证同帧 paint/hit、弹层返回、中文长文本和逐图 source 证据。上述均未运行，[AB-004/005/006](actor-blueprint-differences.md#开放差异) 继续 open，共享 surface 完成也不关闭它们。

## AI 消费适配：AI-C01～12

接续 [AI 的原合同](ai-design-and-acceptance.md#ai-接入现有共享图基础的合同) 和 [Parent 第二次交接](../../../.codex/sessions/ue-window-suite-parent-01a16d4e.md#ai-第二次接续共享图轴字段与实际验证入口)。早期只交付 scope 的记录保留；最新用户/Parent 已明确授予下述四文件写权。修改前只有本域已有 mod.rs 导出 diff，另外三文件无 diff；本轮未写 AI 原资产/backend，未重复原材质请求或入口修复。

| 合同 / 当前实证 | 最小源码 scope 与兼容要求 | 域/宿主依赖与未完成边界 |
| --- | --- | --- |
| C05 外 crate 可命名性 | 本轮按 Parent 后续单文件授权，已在 `zircon_editor/src/ui/graph/mod.rs` 的 `pub use model` 增加原 `GraphDeltaProofV1`、`PreparedGraphDeltaV1`；private model、字段/构造/retention 不变。原缺导出观察保留在各历史 guard/交接，实际编译结果另记 | AI plugin 已依赖 `zircon_editor`，仍须在这个真实外 crate 用公开路径导入/实现合同证明。内部 graph 测试或材质包依赖编译不代替 AI adapter/host 的实际测试；重导出不代表 C04/C06/C09 完成 |
| C02/03 单父与一次有序重接 | 不改 `model.rs` 的 Tree verdict 或私增 Replace 枚举；domain readonly preflight 描述旧父/旧索引、目标父/移除后索引、将移除/增加的边及 revision。一个 AI Reparent delta 经 `GraphDeltaCommand` 提交并一次 Undo/Redo，失败和取消不变 | AI 保证唯一 parent input、child-count、root、整树可达/环与顺序。既有 [authoring/edit.rs](../../../zircon_plugins/ai/editor/src/authoring/edit.rs) 的 `Reparent` 明确使用移除旧边后的 index，[topology.rs](../../../zircon_plugins/ai/editor/src/authoring/topology.rs) 已处理候选顺序；它们是可复用语义，不是已接文档 history 的证据 |
| C04 预算与 revision | `model.rs`、`commands.rs` 和 retained-memory 实现只读；保留 bound→一次 permit→prepare forward/inverse→apply prepared/revision 校验。Target、label、selection、候选/错误与 replay retained owner 都按既有预算纳入，禁止 UI 直接 apply 或无界整图历史 | [AiBehaviorTreeAsset::edited](../../../zircon_plugins/ai/editor/src/authoring/asset.rs) 目前先 clone descriptor，再修改、序列化和真实编译，注释明确调用方须先准入分配并经文档发布。它是候选值，不是已接通的预算 preflight/command；AI 必须证明 clone/编译峰值和 forward/inverse 的边界，不能把候选全量快照直接塞进公共 history |
| C06 单次 DPI | `canvas.rs` 维持现有逻辑坐标算法，本轮不改。宿主输入按物理 pane 原点和实际 DPI 转成 pane-local DIP，再 `screen_to_graph`；paint 先 `graph_to_screen` 再由宿主施加一次 DPI。node/pin/edge/arrow/attachment 命中和菜单锚点共享同帧 transform generation | Parent 指定实际 retained input/paint/sourceRef owner，AI 提供当前图 revision/选择映射；这里没有可据以认领的现成 native consumer。resize、pan、zoom、实际 144 DPI 及异步旧帧验证仍缺，不能从纯数学往返推断产品 DPI 正确 |
| C07 上下端点、切线与箭头 | scope 仅 `routing.rs` 的兼容辅助输入/结果与 `mod.rs` 必需导出：明确几何端点方向、切线与可选箭头；保留原 `route_connection` 签名、所有旧 getter、默认水平 Bezier/Orthogonal 及既有调用方行为。几何 Input/Output 与上/下朝向分开，不把旧固定 x 控制点当 BT。新增 API 名称由实际 diff 定稿 | 同次测量的 pin bounds 产生锚点/路径/箭头 bounds 和命中容差。UE 普通线宽 1.5、分离 4.5，单位锚点差与 `atan2` 箭头角来自真实 policy；separation 的 zoom TODO 保留差异，不能先宣称消除。AI 提供普通/反向的真实语义，动态宽线/bubbles 等 trace 尚缺 |
| C09 附件身份与命中 | scope 仅 `node_widget.rs` 的中立辅助投影、`mod.rs` 必需导出和 `tests/cases.rs`。保留 `GraphNodePresentation` 的原字段/默认行为；辅助接口从原 `GraphNodeView.attachments` 保留 attachment ID、host node ID、category 和源定位，不再只输出 label。实际 bounds、文本行框、instancePath 由 retained consumer 的同次测量/来源关联提供，不造默认几何 | Parent 首阶段保留 Decorator 原单子 wrapper、node ID、child 顺序及独立节点命中；P05 等价投影未证明前不折叠为 UE 附件。辅助 ID 投影不等于独立命中已完成。Service 模型/周期执行和完整 trace 仍归 AI 缺口，不添加假运行样式 |
| C01/08/10/11/12 配套 | 复用既有稳定 NodeId/端点、domain source locator、文本/菜单 focus、剪贴板 ID remap 和 LayoutReviewCase；本轮不扩写 graph DTO、AI 执行或公共主题。选择不合成 active path，静态 label 不代替运行反馈 | AI 拥有真实 catalog、child/参数/引用验证、任务/包装语义和文档存储；Parent 管理 overlay/native host/导出。实际 source hash/revision/instancePath、frame/DPI/world/entity 证据逐项绑定；无 trace、未知 Service 以及缺来源图像保持 open |

实际实施触点只有 `zircon_editor/src/ui/graph/{mod,routing,node_widget}.rs`、`tests/cases.rs`，未覆盖其它 owner 的改动。`model.rs`、`commands.rs`、`canvas.rs`、retention、AI source/runtime、catalog/tokens/Starship 和原 ZUI 本轮只读。若 retained consumer 需要新字段或越出这些路径，先交具体 owner/调用链与 diff，不以 helper 已返回 ID 关闭 C06/C09。

### 四文件辅助 API 实施边界

`route_connection_with_directions(source, target, source_direction, target_direction, style) -> Option<GraphConnectionRoute>` 使用四个空间朝向；两端朝向都指向节点外，因此目标 travel tangent 与目标 outward 相反。原 `route_connection` 与两个原路由函数未改，Right/Left 输入直接复用原函数；其它朝向沿真实 x/y 轴放置 Bezier control 或 Orthogonal lead/bend。返回前拒绝非有限、溢出或无法保留端点方向的几何，消费者不能把 None 当有效路径。`GraphConnectionRoute` 保留原三字段和所有 getter，新增 `endpoint_tangents() -> Option<(GraphPoint, GraphPoint)>`、`target_arrow() -> Option<GraphConnectionArrow>`；箭头包含实际目标 anchor 和归一化到达方向，使用曲线控制点/最终线段计算，不以两 anchor 的直线夹角代替到达方向。所有量只在 graph-local 空间计算，没有 DPI/zoom 或新的 line-width/theme 常量。

这是可供 BT 上/下 pin 使用的中立几何基础，尚未复刻 UE `ComputeSplineTangent` 的任意锚点差方向、4.5 侧向分离/arrow-radius length bias、实际 brush、普通1.5线宽和其缩放/命中合同。AI/Parent 必须将同次 pin bounds 和真实 style 投影到 retained paint/hit 再比较；当前不关闭 C07，更没有10宽/bubbles/反向 trace 样式。材质继续使用原无箭头水平入口；其它域不用迁移即可保留原行为。

`GraphAttachmentPresentation<NodeId>::from_view(&GraphNodeView<NodeId>) -> Vec<Self>` 逐项复制 `attachment_id`、`host_node_id`、`display_name`、`category`，不排序、不按名称合并或分配替身 ID。原 `GraphNodePresentation` 及 `attachment_labels` 字段/构造完全保留；测试保留旧 struct literal 的源码兼容性。新投影没有可据以伪造的 bounds、instancePath、selection/focus 或文本行框；这些仍需真实 consumer 绑定源文档/布局 generation，不折叠 Decorator wrapper、不宣布 Service 生命周期支持。

新增7个 `graph_surface_` 回归源码覆盖旧水平精确点/control、原调用与 opt-in 默认等价、四向组合、上下/反向/同点/极短、非有限/溢出拒绝、箭头最终到达方向、重复 label 的原 ID/host/类别/顺序及空附件。现有12个 graph 测试主体不变，`ui::graph::tests::` 聚焦批合计19项。该批现已真实提交 Jenkins #353，但在 checkout 输入身份守卫终止、`compileStarted=false`，19项仍未编译/执行；内部公开路径的类型引用也不替代真实 AI 外 crate 编译。完整前后指纹/精确 diff、只读文件与 UE policy 哈希保留在 `graph-neutral-presentation-02.json`，本次独立身份、真实归档和源冲突另见下节；没有服务修复或 native 捕获。

### AI 接续12：域内定向路由消费确认

AI owner 的 `bt-edge-routing-12-{before,source,guard,validation}.json`、handoff和累计patch已只读对账：12份本域源、8项共享graph/lock依赖和4份原ZUI均匹配；累计patch为76,784B/SHA `b00543e01a118d5109a170b37755b0b6039ce0725a8c59222f65fc839b418559`。本次AI实际修改只有 `src/lib.rs`、`src/authoring/mod.rs`、`src/authoring/graph.rs`、`src/tests/behavior_graph.rs`，本域未写这些路径或任何公共Rust。graph源仍为此前Widget接续已独立定位的`335a5cca…`，这次是owner确认来源/边界，不是新共享实现或新运行证据。完整哈希、模块边及静态检查复用身份见本域E根 `ai-directed-consumer-reconcile-06.json`。

`AiBehaviorTreeEdgeRouteRequest` / `route_edge` 在同tree/revision、真实kind、精确pin key/方向及当前`parent.children`成员校验后，才把host提供的graph-local逻辑锚点交给原四向API，source Down/target Up，返回原`GraphConnectionRoute`及其tangent/arrow；非法或溢出几何拒绝，无横向fallback或自造坐标。该方法不改作者树、camera、DPI、选择或运行高亮。**document revision不是几何世代证据**：host仍须在同文档incarnation/测量/transform generation内提供真实pin bounds/anchors，先准入分配，再使route、paint、hit、clip与实际DPI共用同一帧数据；这条真实host调用链和focus仍缺。retained预算、一次permit、prepare/revision/history不能由这个helper或`candidate_for_connection`的整树候选替代。

C05仅能登记外crate对共享路由的真实源码调用，未编译不能证明proof/prepared类型的消费者协议已实现；C07具备路由源码消费，但UE tangent/分离、线宽/brush及真实画面命中差异仍open；C09只登记`GraphAttachmentPresentation`依赖，Decorator保留原wrapper/id，未折叠为附件，Task/Service/Subtree无输出。原附件独立bounds/instancePath/命中、真实trace/运行样式和源诊断跳转仍须各自owner交付，不因helper可调用而关闭。

四项新增域测试在实际`authoring_tests::behavior_graph::`模块下；静态数目是authoring根14+graph10=24，另gameplay debugger4，共28。路由四项覆盖提供锚点/垂直切线、foreign/stale/fabricated edge、原wrapper/leaf端口及非有限/溢出测量；这只是源码定义/模块边，不是Rust发现、编译或执行。owner11个Rust格式、12源guard/diff及五文档75链接的静态证据按其原哈希复用，不拿旧`1d06a8fd…`、bt01八源或recovery07给这四份变更背书。后续沿已有`authoring_tests::`准入范围固定当前源与真实锁/source/pool/cache/native writer，不新增`tests::shared_graph_`配方、不借内部core或材质filter代验。原core#353及材质#352失败与not_run保持，无重投、source capture/freeze或服务/浏览器/桌面操作，passed/accepted仍false。

### AI-C06 接续：同一 CanvasState 的 bounds-fit

AI 的 `penpot-ai-resume-r01/current-layout-summary.json` 实际记录 rev28/855 objects：1280、900、640 板的 GraphCanvas 可用宽为895、629、447，三个190宽task的union为650；900左右各裁10.5，640各裁101.5。这是设计几何。跨板原生copy UUID各异，域fixture的稳定逻辑node IDs、有序edge和wrapper语义须保留；不能拿Penpot绝对坐标冒充引擎图坐标、删节点或挪源位置来消除裁切。

在 `zircon_editor/src` 与 `zircon_plugins` 的实际检索中，旧 `GraphCanvasState` 调用仅在共享测试；未定位生产graph的frame-all/selection-fit/source-focus路径。`EditorState::frame_selection` 属于3D SceneViewport；`DockCommand::FocusView` 的graph分支只是工作台rail控件映射；Widget的binding Source文本定位也不是图相机。以上均不能作为AI真实frame消费链。Parent的 `ai-followup-05.json/sharedFrameBoundsGrant` 已另授权三个精确文件；本域先登记before/hash/原测试来源，再在原 `canvas.rs` 实现，未改model/commands/retention、域schema或另建相机。

| 已写的兼容接口 | 语义与边界 |
| --- | --- |
| `GraphCanvasState::fit_graph_bounds(bounds, viewport_size, padding) -> Option<GraphBoundsFitResult>` | O(1)、无分配/全图clone。bounds为完整graph-local测量，viewport为pane-local逻辑尺寸，padding为四侧逻辑屏幕inset；调用方先将真实物理输入/尺寸转换一次。所有计算和投影检查完成后才一起更新原pan/zoom；不改变selection、source位置、已开始drag或作者history |
| `GraphBoundsFitResult { changed, fully_visible }`，经原 `ui::graph` 导出 | 使用既有0.2～4.0范围居中。最小zoom仍装不下时更新相机但返回`fully_visible=false`；精度造成无法完整包含也不冒报全显。相同输入重复调用返回`changed=false`。不承担宿主重绘/几何generation发布 |
| 无效输入返回`None`且原状态不变 | 非有限值、零/负bounds或viewport尺寸、负padding或耗尽viewport、bounds端点/中心pan/投影溢出、f32中塌缩的extent均拒绝。零padding合法；空selection/空图不猜bounds，由host明确选择不执行或真实all-bounds语义 |

host只从当前asset/document/graph/subdocument incarnation、source revision、selection context和同代际retained layout取**完整未裁切**bounds，包含实际node/pin/text/wrapper尺寸；仅当前可见节点的union不够。viewport origin/size/clip、transform和真实DPI须属于同一测量世代，不能以document revision代替。All/Selected/SourceLocator由域和宿主解析稳定身份；来源过期、布局未完成或文档关闭时不得使用猜测尺寸或旧pan。若需延迟，沿真实宿主布局阶段保留一个有界待处理请求，手动pan/wheel、新gesture、切文档/关闭、resize/DPI变化使旧请求取消或重新取证；不建第二clock。真实菜单/快捷键/来源跳转先经过当前graph焦点和IME/弹层路由，fit只改view，不产生资产undo或绕过作者一次permit。

UE依据是只读 `GraphEditor/Private/SNodePanel.cpp`：`GetBoundsForNodes`使用widget位置和DesiredSize，选择为空会走all分支；`Tick`等待visual update和新选中节点尺寸，`HandleZoomToFit`使用CachedGeometry，手动输入调用`CancelZoomToFit`。其默认padding为25个**图单位**，fit使用离散zoom档并以默认档为上限，还带滚动/缩放过渡。当前中立helper使用调用方逻辑屏幕padding、既有连续0.2～4.0范围且无动画；这些UE策略和host实际消费仍为明确差异，不因helper写成而宣称同等行为、1DIP或窄窗视觉已修复。640工具栏的debug selector/overflow继续由Parent统筹，不造公共UUID或删命令代替。

七项新增回归定义在原 `tests/cases.rs`：三种可用尺寸、平移内容与selection/drag保留、四侧padding、min/max与不能全显、非有限/零负输入原子拒绝、溢出/塌缩拒绝、逻辑坐标逆变换与fit后滚轮锚点。当前该模块静态定义由19变26，均未在本轮Rust发现/编译/执行；原 `canvas::tests` 的一个Default约束测试不属于此filter且未改。受影响的未来最小范围仍为`zircon_editor`、defaults off、direct features=[]、lib、`ui::graph::tests::`、Cargo test profile、jobs1/threads1、物理E池及原传递features；Parent当前只授权源码和静态/格式检查，未创建新identity或提交Cargo。原core19 `#353`/receipt `26d25ae5…` 与材质13原失败不重写、不重投；后续源revision/真实writer准入由原Jenkins owner对账。身份、before/after、guard及实际交接见本域 `ai-bounds-fit-contract-07.json`。

### AI 聚焦验证与执行前提

下表中 core graph 一项已按精确范围真实提交，终态是输入身份冲突；其它项仍为消费者候选，尚无编译/执行证明。Cargo 默认 test profile、直接 features=[]、defaults off、lib、jobs1/testThreads1；模块包装器的 management recipe 默认记 dev。仅由 Jenkins 在真实来源/锁文件/配置/依赖守卫及物理 `E:/cargo-targets` 池准入后执行。执行前重新声明精确模块/依赖/命令并匹配最终源码；不扩大到 workspace/全仓，不重放材质入口修复或服务恢复。

AI 较早交接记录曾在 cdd 时点观察 AI 包被拒、嵌套锁缺 toml 边及 resolver 回执不足；这些按原时点保留。后来的`2790fb…`阶段已在pinned client源码列入AI四个受限filter分支，不能继续把旧“仅材质”观察作为当前拒绝证明，也不能把2790当此后不变的selector。这是当时的路由源码观察，不是AI身份/锁/Cargo执行通过；本域未重投AI或修改其锁。AI owner继续沿原请求、受管锁CAS/来源稳定窗口与资源门核对当时真实selector，保留`--locked`，不借材质filter、回退旧capsule或手改共享锁；实际依赖中的target-editor-host、native-host/AI contracts和SDK editor仍由其真实recipe/闭包/池回执证明。

| 验证 owner / 范围 | 最小命令或检查点 | 通过所能证明的范围 |
| --- | --- | --- |
| 公共 graph / Parent 整合 | `tools/dev/dev-module-build.ps1 -Package zircon_editor -Action test -Features @() -LibTests -TestFilter 'ui::graph::tests::' -Jobs 1 -TestThreads 1 -JenkinsIdentityFile <独立core-graph身份>`；Cargo 默认 test profile，传递 runtime target-editor-host/native-host/test-support，以托管 recipe 闭包为准 | 现有12项与新增7项合计19项。独立 `ue-shared-graph-20261024-01` / `graph-surface-core-01a16d50-20261024` 已真实入队14515、执行管理作业#353；终态 checkout_input_identity_conflict / compileStarted=false，19项 not_run。未用材质包/过滤器代验 |
| AI 外 crate / AI owner | 接续12复用真实 `src/tests/authoring.rs` → `behavior_graph.rs` 模块及已有准入filter `authoring_tests::`；待执行argv为 `cargo test --manifest-path E:/Git/ZirconEngine/zircon_plugins/Cargo.toml --package zircon_plugin_ai_editor --no-default-features --locked --lib --profile test --jobs 1 authoring_tests:: -- --test-threads 1`。不是本轮执行命令，不增`tests::shared_graph_`配方 | 静态authoring24项，含graph10及其中路由4项；必须实际发现非零并执行，源码调用不证明proof/prepared消费者协议已实现。AI直接features=[]，插件workspace的runtime native-host及AI runtime的ai/net/physics/sound-contracts仍由真实完整依赖闭包证明；内部core或材质包不能代验。source/锁/pool/cache/native writer和原请求准入未具备前不提交 |
| AI 有序拓扑与文档 / AI owner | 原 `src/tests/authoring.rs` 中 `reordered_children_keep_ids_and_use_destination_index_after_removal`、wrapper、failed move/cycle/source-unchanged 是已有候选值回归源码；真实文档 adapter 接入后另验证一次 Reparent/Undo/Redo、stale/预算/非法父子失败不变、dirty/save token 与焦点/选择同步 | 原候选值测试不证明 command history/预算已实现；这部分 adapter 是 AI 的依赖，不修改公共 commands 来绕过。仍沿同一受管 plugin manifest 入口，不由材质代提第二个入口修复 |
| retained/native 与双端 / Parent 排队、AI 提供状态 | 同一实际 BT 图在 1280×800、900×620、640×520 与 1280×800 实际 150% DPI；上下连线/箭头、端点和 wrapper/独立目标命中、文本/IME、弹层返回、拖动取消与正常 EXE source/nonce/generation | 同一 transform 下 paint/hit/sourceRef 对账，真实注册/执行/trace 与设计分开；Penpot 保存重开、官方导出、产品资源闭包、逐图并排/差异及 1 DIP 复核才可接受。当前没有本域桌面时段或这批产品证据 |

本轮已编辑四文件中立辅助 API 和7项回归，格式/结构/静态守卫单独记录；受管 Rust、AI adapter 和 native 验证仍 pending。材质原13项回归另保留 preparation-conflict / compileStarted=false，未重放，各族 `passed/accepted` 不变。

### ai-followup-03 的实际共享身份与 core 终态

Parent 13:34Z 的 `mod.rs` / `29584afa…` 是最初 proof/prepared/GraphAttachmentView 导出阶段；当前 `2a0a4cda…` 已保留这些原导出和上节获授中立 API，不重做或扩大 model/commands/retention 范围。16:04Z～16:16Z 实际 API 确认材质原 file/page 为 rev53/1484对象，正式 linked Shared rev1/isShared=true，57个 native copy head 的 sourceFile/componentId/shapeRef 全部匹配，四个 canonical-record version1 与原ZUI sourcePath/root/controlId 可追溯；Shared 仍4主件、颜色/字体0。原 browser07 的 rev53 保存关闭重开与24图证据保留，本次只读重核没有新浏览器重开、全局A/B、正式导出或 EXE/DPI 验收。

原生 metadata 实际在 `shared/dev.zircon.zui`；首个探针误读 `devZirconZui` 别名后的空字段已保留并更正，不据此推断源或 slot 缺失。Property 主件原 `value_slot` 为 `6ddc248b-bd35-80c5-8008-d6c8c88eaacb`，五个本域 slot copy 均保持该 shapeRef，但内容仍是 Text fixture。实际源的 value slot 为 required/single/linear，只接受 WorkbenchCheckbox/Dropdown/Field/NumberField/RangeSlider/Slider/Toggle；公共库尚无这些输入主件。真实输入 slot、源 owner codec/正式导出和效果仍 open，交 Parent/公共 pipeline owner 补齐，不在本域复制 Field 或改 Shared。

本域已按用户可见声明，使用上述正式模块命令提交独立 core19；identity SHA-256 `f33013fc0d1eacda260b1581f72159d90388a8ed415e826c920ace97fb752de5`，物理根 E:/cargo-targets、sourceView=checkout、canonical external root E:/Git/zr_vm。Jenkins #353 在488.546秒后 FAILURE，同身份 query 为 complete/conflict/checkout_input_identity_conflict，`compileStarted=false`。精确冲突为 `docs/plans/penpot-ue-window-suite-20261024/ai-design-and-acceptance.md`，before `afb997cb8de8b08b7930d27b60eef39f368535c7278f26d6781466407a5be542`，after `8207d3eea0e1925b949b7acb769cefbb5045b335113192c7607a2f2c46a97552`；本域四源与九项依赖哈希终态后全部未变。原材质两次终态、identity 和日志均未重放或覆盖。

`shared-graph-managed-validation-jenkins-353.receipt.json` 的归档与 workspace 原件逐字一致，SHA-256 `26d25ae582e89199c98b819e53cf17e7ec9861d927a6ab60e5b62ffc0cda22a6`。Jenkins API building=false，不是完整 root/children/EOF/reader/waiter 证明；此回执不借 #400 或 live host 补生命周期。来源/库/slot 对账为 `shared-graph-consumer-reconcile-03.json`，完整测试过程为 `shared-graph-managed-validation-01.json`，同属本文末 E 根报告目录。官方 Parent queue 仍因容量100被拒，queued=false，消息与拒绝另存 `shared-graph-parent-handoff-03.json`。由 Parent/原 source-preparation owner 协调有界稳定窗口；不放宽守卫、不重放或创建替代 core/材质请求、不本地 Cargo/源码复制。4213插件入口当前仍 fetch failed；Parent 的 PNG404 是历史独立问题。C03/C05/C06/C07/C09 的实际消费者/产品证据继续 open。

### Parent 第四次服务器对账：combined-r2、AI 与 core353

本段消费原 owner 的 `parent-followup-04-reconciliation.json`（SHA `e7d2331c…`）及摘要（`93d2d75a…`），原件观测时间为 **2026-10-24T20:29:40.197838Z**。其当时 selector 为0b594015…；版本、磁盘与源码计数均按该时点记录，本域后续材质 r9/r10 与 a9a82c24…观察继续分列。本次只读核对七份被引用文件，包含下述原计划、准备、资源、版本 native 与本域 core353 归档，SHA 全部匹配；没有重新查询服务器、执行工具版本检查或重算 resolver。

combined-r2 已有真实服务器身份：generation `lockgen-3c23f3791938bf820ae9a2bb8a8734c9718e461fa67311cbf27e00fe`，session `jm24`、request `combined-ai-particles-lock-resolution-r2`，原 `prepare-lock-resolution-r2` / operationKey `4357a6ce…` 已 complete，authority 快照匹配。原计划 `1f8593fa…` 及其中历史 null 保留原字节，不能据它继续判断全局没有收件。原 #356/queue14661 的 FAILURE 与原生未启动失败保留；#360/queue14711 的 SUCCESS 对应 `generationStatus=blocked`；#361/queue14721 的 SUCCESS 仅为 `admission-inspected`，resourceAdmitted/soleWriter/fileWriter 均 false、reservation=null、resolverStarted=false。Parent 已核对这三份主归档与 Home/HTTP 字节一致，本域复核其对账原件，不把管理作业 SUCCESS 记成生成通过。

真实准备原件为 `preparation.json` / `fdc33cc1…`，实际工具为 Cargo1.94.1，既有 alternate-lock-path 配方要求 stable1.97+。`tool-version-r2/native-terminal.json` / `6209ba8b…` 的 exit0、active0、childrenGone、双EOF和 complete 只覆盖原 `cargo --version`。真实资源原件 `78d561b1…` 为 admitted=false；resolver 独立预算1CPU/2GiB/256MiB，35GiB reserve加256MiB最低35.25GiB，未使用编译预算。这些证据不提供 resolver、CAS发布/恢复、host 或 core353 的原生终态，也不触发本域更新工具链。

原准备的1075项 resolution metadata pin 在该次对账中有四份 editor `src/lib.rs` 变化：AI、material_editor、particles、vfx_graph；材质一项为 `7ef40319…`→`928f63be…`。target membership 未重枚举，旧 prepared 不能直接 resolve。沿原 generation/request 链先登记实际受管工具与元数据准备修订，再取得真实 resource、sole writer/file writer 与 pool/cache 准入，消费既有 admit/resolve/publish/recover。该快照的 generation_execution/candidate/file_writer 均0，实际 after/diff、resolver执行与CAS发布/恢复回执仍为空；不是 resolver 实现缺失的结论，也不创建第二套锁模型。

AI 的 `ueai24-bt01/validate` 与 `ueai24-wt01/create` 原输入、route/Records/marker 独立保留；本次精确查询未见对应记录，不推断历史从未送达或据此重放。Parent16:42的12源+8依赖/28定义属于旧快照；较晚现场已见3份AI与3份graph pin变化、额外blackboard模块，以及原三测试文件29个静态定义。AI owner须给当时完整源revision、模块membership与正常server-owned validation operation并关联旧失败；不把新源塞旧bt01、借create或jm24生成身份、补手工grant。0b阶段的四filter路由按该时点保留，不能代替后继selector的真实能力核验。

core353 的原 request/operation、queue14515及 `26d25ae5…` 归档保持 complete/conflict/checkout_input_identity_conflict、compileStarted=false、原19项未运行；冲突仍是 AI设计md `afb997…`→`8207d3…`。Parent本次再次核对本地/Home/HTTP一致；原四源已有2项、九依赖1项相对旧identity变化，后继bounds-fit七项与当前26个静态定义不能借该旧批背书。原归档没有root/children/EOF证明，native gate继续open，不能借工具版本或#422管理检查关闭。下一窗口须提供完整current membership/writer/resource/pool/cache准备及有界start/deadline/cancel/EOF；不只盯单个doc，不跳docs，不在本轮冻结或重投旧单配方。

#422的67项管理检查只覆盖其对应路由/闭包资格；AI Rust、真实resolver/CAS、共享graph消费者和正常EXE验收仍未通过。实际消费、七份文件哈希、文档before/after与独立静态守卫在本域 E 根报告 `shared-graph-parent-reconciliation-17.json`、`guard-44.json`；本轮未修改Rust、锁、管理工具、服务、Penpot或桌面，没有提交/重放/QueryOnly/Cargo/resolver或新的ACK，释放0 B。

### consumption04 源码修订与旧身份边界

原owner在21:08:49—53Z对材质r2/core353的只读观察已由本域消费，完整SHA与逐项检查见E根 `material-source-revision-review-18.json`。材质原17项匹配12项，core原13项匹配10项；当前限定读回与该观察相符，不是完整checkout、全部writer或capture准入。该时刻6d120059及E盘余量保留为历史，不覆盖较晚的材质r9/r10状态。原30a5b632/f33013fc两份identity、#352/#353失败和查询log均未改写，也未把后继源塞进旧终态。

共享三个变化pin已与17:57:43Z的 `ai-bounds-fit-contract-07.json` 逐一对应：`canvas.rs` 23ea48fa…→a072722f…新增O(1) `fit_graph_bounds`；`mod.rs` 2a0a4cda…→0ed32746…仅补公开fit结果；`tests/cases.rs` 31635777…→9f7e906a…增加七项fit回归。原model/commands、routing/node_widget的限定pin仍匹配。方法消费已计算的bounds和局部逻辑viewport/padding，验证有限/可表示几何后一次发布pan/zoom；返回 `None` 时不变，缩到最小仍裁切则 `fully_visible=false`。默认与滚轮语义、selection、drag、源坐标和history保留；不选择节点、克隆全图或再乘DPI。proof/prepared的私有构造、retention/revision及一次permit不变。

当前core cases静态定义26项，即旧19加7；七项覆盖同一650宽union在895/629/447逻辑viewport中的居中、平移/选择/拖动、padding、zoom限制、无效与溢出原子拒绝、坐标逆变换和滚轮锚点。全部未编译/发现/执行；独立 `canvas::tests` 默认值测试不在原filter内。辅助fit依然需要宿主提供同代际真实retained bounds、viewport origin/clip、文字行框、输入owner、paint/hit/focus与一次DPI转换，不能关闭正常EXE/1DIP/实际150%DPI验收。

材质另两变化pin及新增connections模块/九项测试是领域类型预检，详见[材质修订审阅](material-family.md)。它不成为Widget/AI/Rig/动画/VFX的跨域模型，也不取代其domain preflight和delta。旧材质13/core19均not_run，当前材质30和core26仅是静态定义；较新r10的20/20声明匹配属于另一基线及独立历史，不抵消旧17/13声明变化。

原#352/#353授权active与business complete不代表资源/writer/native EOF已释放；两项前端查询失败仍单列为查询失败。后续先由原owner给受支持的原链接续和完整成员/writer/物理E reservation/pool/cache/native writer，再由Parent协调有界共同ACK窗口、开始/超时/取消与真实EOF；不将原材质9root跨借给core，不发明capture时长或重投complete operation。本次仅改两份本域文档并做静态核查，未QueryOnly、Cargo、resolver、工具链/锁/共享源码/服务/桌面操作或例行ACK。

## 资源消费适配：只读 Reference Viewer

接续资源 owner 的 [已有提案](08-resource-tools-shared-contracts.md#材质-owner只读依赖图消费提案) 和 [Parent 分工](../../../.codex/sessions/ue-window-suite-parent-01a16d4e.md#资源抽屉与资源工具共享接口交接)。Static Mesh/Texture、Reference Viewer 与独立音频工具归资源；Skeletal Mesh/Skeleton/Physics Asset 归角色；Niagara 归粒子。材质仅保留本域窗口、图表面牵头、材质槽与纹理引用适配；共享图存在不证明任何缺失 Toolkit 已完成。当前资源只申请只读依赖图，没有申请音频作者图。

| 消费点 | 可复用与需要 consumer 提供的合同 | 尚未实现或待 Parent 接续 |
| --- | --- | --- |
| 文档、node 与 edge 身份 | domain 提供 project activation/catalog revision、真实 asset UUID、locator 和 known-project/缺失状态；同一资产的节点身份不取显示名或当前数组下标。边区分原始依赖方向和 references/used_by 的展示方向，携带 provider 的稳定依赖 key；没有独立 key 时只能明确采用端点对聚合并保留真实数量 | `GraphEdgeView` 仅有两端 `GraphPortRef`，没有独立 edge ID/标签或无 pin 的依赖边投影。不能造可连接材质 pin 伪装完成，也不能从 UUID 推断 hard/soft、种类、循环或依赖数量；provider 缺字段保持未知 |
| 只读能力 | 复用现有 pane-local transform、选中集合和 frame/pan/zoom；只读 scope 必须同时禁用拖节点写位置、拖线/重接、Palette 创建、Delete/Cut/Paste 和作者 Undo，不只隐藏按钮。布局或查询变化不写资产 | 原 `GraphModel` 同时带有 mutation/retention 合同，尚无经产品验证的只读 host 消费路径；不以空 Delta、无效果命令或绕开 proof/permit 的 `apply` 充数。Parent 在现有表面接只读能力投影，不新建第二套 graph |
| 变换与选择焦点 | 同一 revision/generation 的 layout、边路由、节点命中和菜单锚点复用当前 graph transform；node 集合映射资源 selection，保留 primary/lead/focus，并让选中项可见。未知外部引用可显示，不投影为可拖放项目资产 | 现有 `GraphSelection` 仅 node ID 集合，公共 lead/focus、只读边命中与实际 host 仍待接；异步 provider 返回必须匹配 project/query/catalog generation，过期结果不改变现有选择 |
| 双击与上下文导航 | consumer 发送真实 asset locator/UUID、索引类型和来源 locator；Find-in-Browser 回资源命令中心，Open 回 registry/Toolkit。成功、类型未索引、Toolkit/command 缺失及资源加载错误分开；关闭后按当前 generation 恢复源节点/菜单触发焦点 | [实际事件入口](../../../zircon_editor/src/ui/host/editor_event_execution/asset_event.rs) 的 `ActivateReference` 只定位，`OpenAsset` 才检查 registry/command 并打开 view。入口存在不等于每类 Toolkit/真实内容或焦点返回已验证；禁止用 toast、标题变化或通用编辑器替代目标 Toolkit |
| 查询、边界与诊断 | 资源 provider 拥有根资产、深度/宽度/过滤、方向、循环/截断/缺失诊断与 source revision；图只呈现已确认的数据和真实错误/导航目标。取消或项目切换丢弃旧请求，保留已知结果的明确版本 | 无真实 provider 字段就不显示编造分类或假 Find 结果；截图与正常 EXE 验证仍待实际注册、数据和 capture readiness。只读导航不进入材质 compiler/history，缺失音频执行不因本合同关闭 |

首次验证应使用当前项目真实依赖：同一 revision 的 references/used_by 方向一致；循环或自引用由实际数据证明；缺失/外部引用不可当作可打开项目资产；只读快捷键无 mutation；已注册目标实际打开正确 Toolkit，未注册目标保留原选择并显示真实原因；关闭返回与异步过期结果不抢焦点。资源域准备 provider/业务命令与证据，Parent 接公共 host/能力/注册，材质维护此表面合同，产品与双端验收均 pending。

## 动画消费适配

接续动画 owner `01a16d51-741b-73b3-a61a-2189cd38f81e` 的 [animation-shared-controls](animation-shared-controls.md) 和[域设计](animation-design.md)。材质继续牵头中立 graph 表面；动画负责 Clip/Blend/Additive/Mask/Output、状态 entry/transition/条件、源文档/命令/history、编译与播放；Parent 负责跨域接口、公共 UUID/样式/composite slot、安装和正式导出。这里没有转移动画业务 authority，也不更改其 Rust/ZUI 或公共 graph 实现。

下列来自真实源码的差异决定适配边界，不能用 Penpot fixture 或 palette 名称补齐：

| 来源 | 已有基础与待接缺口 |
| --- | --- |
| `zircon_runtime/src/core/framework/animation/asset/{graph,state_machine}.rs` | Clip/Blend/Additive/Mask 有持久 string `id`；Output 只有 `source`，没有独立 ID。State 以 `name` 被引用，entry 为 `entry_state`；Transition/Condition 为 Vec 项，无独立持久 ID。动画域须定义 Output 与 named node 不冲突的文档内定位、重命名 remap，以及 transition/condition 的稳定或明确 revision-scoped 身份，表面不分配持久 UUID |
| `zircon_runtime/src/core/framework/animation/compiler/schema.rs`、`zircon_editor/src/ui/animation_editor/capabilities.rs` | 真实 schema 有 Pose/ScalarParameter/TargetMask、direction/required/accepts_multiple 和 owner/version。当前 resolver 只允许创建 Output/Blend，Clip/Additive/Mask 明确 unavailable；`blend_space_1d/2d` 不在 Graph schema，不能因对应 State kind 存在就创建 Graph 节点 |
| `zircon_plugins/animation_graph/editor/src/plugin.rs` | palette 仍列 `blend_space_1d/2d`，与持久 Graph enum/host resolver 不符，保留原 AN-P04 gap。本次还核实 Blend palette 的 `a/b` 两输入与 compiler schema 的多值 `inputs` 不一致；域与插件 owner 须统一真实端口投影，表面不能按 caption 猜映射 |
| `zircon_editor/src/core/editing/animation_document/mutation/graph.rs` | 当前 connect/disconnect 只收两端 node ID；Additive connect 仅改 `base`，disconnect 会按同一 source 清匹配的 base/additive。尚不足以表示指定 `base` 或 `additive` pin 的独立接线/断线、replace 与 inverse；typed port delta 由动画域补，表面不把成功返回当完整 pin 编辑能力 |
| `zircon_editor/src/core/editing/animation_document/command.rs`、`zircon_editor/src/ui/host/animation_editor_sessions/editing.rs` | 已有 CAS revision、retention/一次 permit、`AnimationEditCommand`、`HistoryContextId::Document(document)` 和提交后 session 投影。不能另建图内 history 或让同次编辑分别提交 GraphDeltaCommand 与 AnimationEditCommand；需让既有作者 authority 消费中立图事务 |
| `zircon_editor/src/ui/animation_editor/session/graph.rs`、原 `zircon_editor/assets/ui/editor/host/animation_graph_body.zui` | graph.rs 当前生成节点 label；本次在 animation_editor、animation_editor_sessions、animation_document 三组路径未检索到生产 `GraphModel/GraphEditContext` 适配。原 v2 body 的 root、canvas control 和 slot 身份保留；它们存在不证明 retained 图/命中/连线已安装 |

UE 只读依据位于 `AnimationBlueprintEditor/Private/AnimGraphConnectionDrawingPolicy.cpp` 与 `StateMachineConnectionDrawingPolicy.cpp`：前者区分 pose pin 并使用动画 debug 映射，后者关联真实 transition node 和 `Graph.AnimStateNode.ConnectionArrow`。共享层须保留普通作者边、selection/focus、编译错误及真实运行反馈各自来源；不能把点击高亮或本地计时当动画权重/active transition。

以下是接现有 graph projection/context 的语义要求，不是已新增 Rust 字段或组件 props。通用几何、手势和预算协议沿前文，不再建立第二份动画图存储。

| 消费边界 | 动画提供的真实输入/authority | 表面/宿主的处理与待接项 |
| --- | --- | --- |
| target 与 source identity | 实际 asset locator、DocumentId、Graph/StateMachine kind、graph/subgraph/transition-rule target、source/model revision、schema owner/version 和 session generation | 保留原 node string ID；Output 使用域明确的有类型 locator，不能与 named `output` 混淆。State 名称若同时承担身份，改名必须由域原子重写 entry/transition/关联引用并重定位选择；label 和 Vec index 不作为跨 revision key。Target 切换/关闭/源重载取消旧手势与导航 |
| pin/edge identity、类型与方向 | `GraphPortRef` 以实际 node locator、direction、schema pin key 定位；`inputs/base/additive/pose/weight_parameter/target_ids` 按真实 schema 和资产字段映射。域定义多值顺序、连接 key、typed preflight/替换/环与资源引用校验 | 复用现有严格类型/方向基础，但完整 verdict 留在动画；默认同型不等于 Pose/参数/Mask 或多值合法。多值/平行 transition 若两端不唯一，须先有域 connection identity，再提供重排/独立删除。普通 Graph 依赖与 StateMachine 转移拓扑分别验证，不对状态机循环一律套 Dag；当前 pin label/edge key 公共投影缺口继续 open |
| selection/context/focus | 单一 document selection authority、node/pin/transition/condition/property 子目标、context-menu 命中对象与命令 selection、稳定 opener/source locator、来源 generation | 图、导航/Details/诊断共享选择并防递归同步。右键 subject 与原多选集合由域命令规则区分，不按最后一个 label 发操作。现有 GraphSelection 只有 node set，lead/anchor/子目标 focus 待 Parent 接线；文本/IME 优先，菜单关闭回合法 opener/canvas，selection 不生成资产 Undo |
| properties/diagnostics slot 与跳转 | 节点/状态/转移/条件的真实 typed property projection、enabled/readonly/draft/error/commit/cancel；`AnimationCompileDiagnostic` 的 code/severity/message/element，加同次 source/compile request revision 与 locator | Details 使用 Parent 现有属性行/字段；GraphOutput/GraphParameter/State/Layer 与 transition/condition 诊断分开定位。`transition_index/condition_index` 只按同次输入映射，重排/编辑后结果 stale；不能把旧 index 跳到新条件。静态 GraphDiagnostic 不替代域动态诊断；没有真实 rule 子文档时不打开假图 |
| 同帧 transform、命中与拖线 | 同次布局的 node bounds、pin anchors、edge path、hit bounds、文本行框及 layout/transform generation；宿主 pane 物理原点、真实 DPI/clip 和图 camera | 输入只除一次 DPI 再 screen_to_graph，绘制 graph_to_screen 后乘一次 DPI；hover、context anchor、hit 与预览线同源。resize/zoom/滚动/DPI 变化使旧测量失效；候选线与已提交边分开，drop 时重新核对端点/revision/verdict。节点位置所属 author-layout/view owner 由动画明确，不写入播放时间或 pose |
| command、retention 和历史 | 域 delta/inverse、expected revision、retained pair/materialization peak、`GraphDeltaProofV1`、`PreparedGraphDeltaV1`，以及现有作者文档/context/dirty/save | 公共图提交保留 proof → 一次 permit → prepare → apply；失败不变、拖动一次提交一次历史，Esc/capture lost/切源取消有界预览。不能简单 apply 绕过准入，也不能先完整克隆图再补预算。现有 AnimationEditCommand/文档 CAS 是同一 authority；生产 GraphModel/context 与历史桥接尚未完成，Parent/动画选定唯一提交路径，避免双份历史 |
| 编译、运行与导航反馈 | 动画实际 compiler 的当前/last-good 产品、source map；真实播放 session/实例、节点权重/active state/transition、pose/frame generation 和资源状态 | 表面只呈现匹配 revision/session 的反馈；current compile、last-good topology 和 preview frame 分别标明，旧结果不能覆盖新文档。Clip/Additive/Mask 创建、semantic/preview capability 等缺口不能靠改 palette、画节点或材质 compile 返回“通过”关闭。播放/时钟继续归动画真实 backend，图层不推进时间 |

共享 graph 的公共路径仍是原 `zircon_editor/src/ui/graph/`；`mod.rs` 已重导出既有 proof/prepared，外部域可以据公开路径命名，实际 adapter 编译与 retained 接入仍未证明。动画侧候选路径保持 `src/ui/animation_editor`、`src/ui/host/animation_editor_sessions`、`src/core/editing/animation_document`，原 `animation_graph_body.zui` 只由其 owner 按真实导出迁移。公共 properties/diagnostics/composite slot/API 由 Parent 对应现有组件，不在材质另建动画面板或修改 palette/编译器来掩盖不匹配。

材质如需曲线/transport，消费动画已交付的中立 TimelineModel/CurveModel、域 adapter、真实 backend 和 gesture/inverse-delta 合同；不强制所有消费者创建动画 World，不建立第二 clock。曲线的同步 apply→inverse 不是 graph proof/permit 的替代，也不是 seek/job 已完成的回执。公共 UUID/样式/六事件兼容路由和官方 `dev.zircon.zui` 导出链由 Parent/原插件 owner 集成；动画私有设计 metadata、shapeRef 和空态组件并不等于产品导出或绑定安装。

接续验证仍 pending：在真实可用的 Output/Blend 和域提供的有效源上验证 node/pin/edge 定位、指定 pin 的接线/拒绝/替换/断线及一次 Undo/Redo；导入已有 Clip/Additive/Mask 时保留数据，创建能力按真实 gate；重命名、Blend 多值重排、平行 transition/condition 选择与诊断过期不误定位；Graph/StateMachine/子目标导航和 Details/context focus 返回；revision 变化拒绝旧 proof/prepared/拖线且文档不变；保存关闭重开身份与 history/dirty 对应；真实编译/播放反馈与同帧命中再覆盖三尺寸和实际 150% DPI。当前仅来源/合同核对，未实现生产 adapter、正式导出或运行这些场景，不提升动画或材质整族验收。

## 粒子消费适配：PFX-GRAPH01

本节接续 Parent 转交的 PFX-GRAPH01（粒子计划表中写作 PFX-GRAPH-01）、[域设计](particles-design.md)与[给材质 owner 的合同](particles-shared-contracts.md#给材质owneroverview与有限vfx节点)。材质牵头 selection/transform/node-pin-edge identity 和既有事务适配提案；Parent 整合公共 host/组件/接口；粒子 owner `01a16d52-a4a5-7271-90a2-336091337737` 负责持久 source schema、领域命令/合法连线、编译与诊断、运行及 preview 替换。这里只细化同一 MAT-SHARED-001，不创建新的 VFX 模型、Rust trait 或公共组件，也不写粒子原 ZUI/runtime。

实际能力按源码分层记录，替换已过期的“GPU/VFX 全部 no-op”判断：

| 已核对的真实来源 | 当前能力与实际限制 |
| --- | --- |
| `zircon_plugins/rendering/features/vfx_graph/runtime/src/lib.rs` | `VfxGraphAsset { name, max_particles, nodes: Vec<VfxGraphNode> }`；只有 SpawnRate、Lifetime、Velocity、ColorOverLife、ShaderGraphMaterial 五类。没有持久 node ID、pin、edge、图位置或任意连线模型。旧 `compile_vfx_graph` 仅返回必要节点/容量等 metadata/report，不生成可执行程序 |
| 同包 `src/plugin.rs::compile_vfx_graph_program` | 真正 lower 到 `ParticleSystemAsset` / `ParticleGpuProgram`，拒绝同类重复节点、非法容量/非有限参数；SpawnRate、材质必需。材质通过 `VfxShaderGraphResolver` 解析现有 `ShaderGraphAsset`，仅接受 binding-free color Material target；TextureSample/MaterialOutput 路径拒绝，texture bindings/roughness 没有 particle consumer。此依赖不等于共享表面调用材质编辑器 compiler 或用 MaterialAsset 存 VFX 文档 |
| 同包 `src/plugin.rs::VfxRuntime`、粒子 `runtime/src/render/{runtime_prepare,executors}.rs`、`runtime/src/render/gpu/readback.rs` | 有真实 configure/tick、prepare collector、GPU simulation/transparent executor 和 readback/probe 源码；配置后的 dispatch 来自实际容量。`configure` 已配置后明确拒绝，不能靠再次 configure 完成改参。当前源码不是全 no-op，本轮也没有执行这些路径 |
| 同包 `src/tests/lib.rs`、VFX `editor/src/plugin.rs` | 已有真实 product-frame、lowering 拒绝和 emitter 隔离测试源码；本轮未运行。Editor feature 仍只返回 descriptor，不安装编辑器扩展，source session/handler/UI/undo/save/preview consumer 未因 runtime 存在而完成 |
| `zircon_plugins/particles/editor/src/source/{model,codec,lower}.rs` | 现有 ParticleSystemSource/emitter/burst/curve key 和 decode key adoption、URI-ready runtime lowering 是粒子域候选；不含上述 VFX 节点的持久 authoring graph。emitters 的 key/display_name 与五类 VFX node identity 分开，不能把前者的稳定 UUID 说成后者已实现 |

UE 只读参考 `NiagaraEditorWidgets/Private/SNiagaraOverviewGraph.cpp`：`GraphSelectionChanged` 经 guard 写 ViewModel selection，`PreClose` 读实际 view location/zoom 保存 view settings；命令和创建菜单取当前 ViewModel/edit mode。Zircon 接续同一文档选择和合法能力，不因 UE GraphEditor 有通用接线能力就为当前 VFX 列表生成 wire。

下表的 source/compile/install/frame generation、locator 和 command payload 是消费合同的语义要求，尚未注册为新公共字段/API。

| 接口边界 | 粒子域提供 | 共享表面/宿主保留与返回 |
| --- | --- | --- |
| 模式与 target | 实际 project/session、AssetUri/DocumentId、source revision、active system/emitter 或真实 VFX graph/subdocument；read/edit/create/connect/save 能力及拒绝原因 | Overview 允许 `inputs=[]/outputs=[]/edges=[]`；五类 VFX node 属性列表可投影为无 pin 节点，实际可编辑性由域 source adapter gate。没有连接模型时关闭接线手势/handles，不能把通用默认 `can_connect` 当运行语义；ready-empty、未准备、失败及无能力分别显示 |
| 稳定 node/pin/edge identity | Overview 使用真实 document/system/emitter key；VFX node 需由粒子定义持久 key、显示名、schema/field identity、迁移/序列化与引用 remap。未来有真实 pin/edge 模型时再提供稳定端点与 connection key、方向/类型、合法连接规则 | `GraphModel::NodeId` 直接保留原域类型。现阶段 VFX enum kind/Vec index/name 均不能冒充持久 node ID；只读 index locator 绑定精确 source revision，重排后失效。不得从材质 Output pin 或 Scalar/RGB 类型推导粒子端点；非唯一端点的 edge key 接入仍由 Parent 补中立投影 |
| 文档与视图坐标 | 真实节点/布局 owner、node bounds、同帧 pin/文本/命中几何；pane 物理原点、真实 DPI、clip/layout/transform generation；按实际 target 隔离的 pan/zoom | 输入 `(physical - pane_origin_physical) / dpi` 后调用 `screen_to_graph`，绘制 `graph_to_screen` 后仅施加一次 DPI；框选、拖动、hover、菜单锚点使用同一变换。VfxGraphAsset 当前无位置字段，不能新塞进枚举值或粒子模拟 transform；布局由既有 Editor view-state 或域认可的实际作者字段持有 |
| selection、focus 与跨面板定位 | 与 EmitterList/ModuleStack/Details/Curves 共用的 document selection authority、typed source node/field/curve locator、来源 generation、合法父目标和 focus opener | 现有 `GraphSelection` 的 node set 是域选择的投影，双向同步 guard 防递归；删除 node/key 后由域回有效 emitter/system/canvas。子字段焦点和 popup return 仍需 Parent host 接入；文本/IME 优先，F/A/Find/Delete 只在正确 scope 作用于真实对象，不抢全局/文本输入 |
| 事务与领域命令 | 实际 palette/schema/defaults、expected source revision、typed delta/inverse、保留预算/proof 和既有文档 history；duplicate/必需节点/参数/资源依赖校验，域 clipboard codec 与稳定 key remap | 复用 `GraphEditContext/GraphDeltaCommand` 的 proof → 一次 permit → prepare → apply。拖动中有界预览，结束一次提交；Esc/capture lost/文档或 revision 变化取消；非法输入/预检拒绝不改 source。无 VFX source codec 或真实复制闭包时不开放对应操作，不能复制 caption 或另建图内历史；若位置仅为 view state，不伪造资产 Undo/dirty |
| compiler diagnostics 与跳转 | 实际编译输入 revision/hash、request generation、源顺序到稳定 key/field 的映射、动态 severity/message；使用真正的 program compiler，明确 report 与 executable compile 的区别 | `VfxCompileError.node_index` 只按同次 compile 输入的映射解析；重排/编辑后旧结果标 stale，不能按新 Vec index 跳转。容量/缺必需节点/backend 类错误可由 compiler 返回 index0，域须标为真实 graph/global 诊断，空列表尤其不能虚构第0节点。共享层只分发有效 source locator，现有静态 `GraphDiagnostic` 尚不足以代替动态编译结果接口 |
| preview 替换、运行反馈与保存 | 粒子提供实际 candidate validation/compile/install owner、单一生产 preview session、source/compiled/installed/rendered revision、frame/readback generation、dirty/durable saved revision；动画 owner 提供共享时域表面 | 不重复调用 configure、不在图层另开 manager/clock。域与生产 preview owner 负责候选安装/受控替换、成功后切 generation、旧 job/consumer/resource 回收及关闭取消。compile 成功、install 成功、readback 和已呈现帧分别证明；失败保留可识别 last-good。已提交源的安装失败不得伪造 current 或 durable-save 成功，具体 source commit/install 原子性由域文档合同落实 |

| 可直接复用的基础 | 需粒子 consumer 完成 | 尚未支持/不得据此宣称完成 |
| --- | --- | --- |
| 现有泛型 `GraphModel`/node views、空 pin/edge、`GraphCanvasState` 的 pane-local 变换/框选/拖动预览、`GraphSelection`、`GraphEditContext/GraphDeltaCommand`、域 clipboard 接口 | 实际 Overview 与五类 node 的 source projection、持久 VFX node/schema、typed edits/inverse/history/dirty、布局 owner、选择/source locator、真实 palette/能力与编译映射 | 任意 pin/edge/script/module 图、Niagara 参数映射/VM、隐式材质类型转换、无源数据的创建/复制、未接宿主的 node 编辑；通用 surface trait 存在不构成生产 adapter |
| 既有公共 graph 路径；材质rev55的5个业务主件为3个Material图节点提案和2个Function业务主件，仅供本域几何/flow参考 | 粒子复用 Parent 实际发布的 native 图/字段/菜单身份与 slot，按真实域类型设计参数内容，接原 ZUI/event/handler 并保留其身份 | 五个业务main均不是公共Graph library UUID，不改标签冒充VFX node；Shared当前四项基础主件也没有Graph主件，正式导出和native paint/input/sourceRef仍未验证 |
| 已有 VFX lowering、particle backend/prepare/executor/readback 的源码入口 | 粒子与既有生产 viewport owner 接 configure-once 候选替换、compile/install/frame/readback 关联、实际 reset/seek/cancel/teardown 及资源保存 | runtime 源码/test 函数不是本轮执行回执；Editor handler、正常 EXE 真实粒子变化与双端验收仍 pending。材质编译结果、静态图或固定粒子数不作为替代 |

路径交接保持原归属：共享 `zircon_editor/src/ui/graph/{mod,model,canvas,commands,routing,node_widget}.rs` 由Parent整合，材质已按先前逐文件授权交付重导出、定向路由/附件投影和中立bounds-fit，model/commands/retention仍只读；本轮仅核查与补充文档，没有新增公共源码变更。`zircon_plugins/particles/editor/src/source/`和VFX feature的editor/runtime由粒子与实际feature owner对账，本文没有跨写。已有proof/prepared公开路径保留；真实域协议消费、pin label/edge key、子目标focus、动态诊断/native host仍开放。时间轴/曲线/播放归动画owner，preview框架/资源与公共导出由Parent协调；不增加第二套图系统、主题或模拟服务。

后续真实 consumer 验证包括：无 emitter 的真实文档空态（runtime 不能实例化时明确显示原因）；Overview→Stack/Details/Curves 同一稳定选择与弹层焦点返回；五类 node 真正创建/改属性/删除、duplicate/非法容量/不支持材质拒绝且文档不变；一次 Undo/Redo、保存关闭重开保留 VFX node key；先编译再重排导致旧 `node_index` 结果失效；有意缺 SpawnRate/material 的 graph 诊断不跳假节点；真实改参经受控 preview 替换后 frame/readback 变化，失败显示 last-good，旧 job/frame 及最后 consumer 正确释放。上述再覆盖三尺寸/实际 150% DPI 的绘制、命中和文本行框以及原生双端比较。本轮仅完成来源与消费合同核查，运行项全未执行，粒子与材质整族 passed/accepted 仍为 false。

### PFX 接续：空端口、空边与无连线交互

接受Parent的最小消费者需求：初期真实System/Emitter Overview和五类VFX source投影均可有`inputs=[]`、`outputs=[]`、`edges=[]`。这是合法可见状态，不意味着资源未准备，也不意味着该source可编译或有生产preview。`null link interaction`表示没有端点/连线命中目标、拖线候选或激活的连线gesture，不是把现有Vec schema改成`edges=null`、制造空`GraphPortRef`或新建NullGraphModel。当前`GraphNodePresentation::from_view`直接保留两个`.len()`为0；`required_input_diagnostics`对无input的节点不生成缺线错误；真实VFX compiler仍可报告缺SpawnRate/material或其它源错误。

空边和无连接能力须分开：仅有`edges=[]`而具备真实稳定端口、当前source/domain capability及合法候选的图仍可创建第一条边；不能按边数清空端口或禁用连接。真实required input未接线时仍保留实际诊断，不因空边状态或Overview呈现而抹除。`GraphCanvasState`只提供既有camera/selection/drag及bounds-fit，不持有source-bound link-gesture生命周期；下列候选清理、source切换、取消与焦点返回是宿主/域适配的待接合同，不是当前CanvasState已实现的能力。GraphModel/GraphDeltaCommand、retention/revision/一次permit保持原合同。

UE依据新增只读`NiagaraEditor/Public/EdGraphSchema_NiagaraSystemOverview.h:19`：`CanCreateConnection`明确DISALLOW、`TryCreateConnection`返回false；原源码拒绝文本为空且带TODO，Zircon按用户要求提供真实不可用原因，不复制空白反馈。`SNiagaraOverviewGraph`的选择guard和关闭时保存view位置/zoom仍是参考。当前VfxGraphAsset只有五类node列表，没有pin/edge/schema；无需改共享方向/arrow/attachment接口，也不创建装饰wire来获得“类似Niagara”的图。

| 状态/入口 | 最小消费与真实反馈 |
| --- | --- |
| 已就绪的无连接模型 | 绘制真实节点/标题/域提供的内容flow，pin rows、anchor、edge route、wire preview和相应hit targets均为空；selection、pan/zoom/frame与实际Details/Stack/source定位仍可按已有能力工作。不能为0计数加一个默认端口、虚构output或把视觉分组线注册成edge |
| 空source、未就绪、失败 | ready-empty保留实际空数据和域原因；loading/unready/error不能转成成功的空Vec。last-good若保留必须标记stale及source/revision，不能将旧可连线状态带入当前source。无emitter的系统可显示真实system节点，不能生成默认emitter |
| 未定义稳定端口/连接模型时的连线命令 | 同一domain eligibility用于handles/拖线、上下文菜单、工具栏、键盘和程序化命令；不暴露活动连接目标，并给出实际原因，例如“当前source未定义端口或连线”。仅edge列表为空不进入该禁用分支。不以Material类型同型verdict放行。过期/伪造的实际port引用仍校验当前source与端点并拒绝；默认`MissingPin`只证明默认校验，不替代域capability说明 |
| 旧gesture或异步结果返回 | 切文档/模式、source revision变化、对象删除及真实输入owner撤销时清除旧候选/hover/wire，取消不创建delta、inverse或undo项；按真实触发控件/pane归还焦点。无连接只关闭连接能力，其它作者命令逐项依域source/typed delta决定，不能悄悄变成无效果按钮 |
| 未来可编辑/可连线source | 粒子提供稳定node/pin/edge key、document/revision/layout、typed delta/inverse/retention及合法性；旧node_index诊断/只读snapshot locator须绑定精确source并在重排时失效，不能升格为持久身份。没有位置持久化/命令时不把拖动当作者修改，连线接口继续关闭 |

节点、选择、hit/paint/clip使用同一retained测量与transform generation；物理坐标先减pane原点、除一次真实DPI，原CanvasState不另建相机/inputowner。frame消费完整未裁切bounds，而非只对可见节点取union；空图不猜尺寸。真正作者操作仍走原GraphModel/GraphDeltaCommand的revision/proof/prepare/一次permit，不用整图clone或简单apply绕过。VFX lowering/合法性/编译诊断归粒子；preview还须真实compile→install→frame/readback关联，不能借Material模型或GPU源码存在关闭产品缺口。

原回执的实际API观察为：Material原file/page rev55/1923对象/5业务main、isShared=false；PFX原file rev30/3页388对象/1业务main、18copies其中11个引用Shared。同一原JSON的`domain=shared`已经是rev2/isShared=true/4基础main、41对象、34colors/3typographies；此段原“colors/typographies=0”及旧消息是本域误述，consumer自身styles0不能推导库为0。`pfx-empty-ports-contract-08.json`及SHA `2a950ff2d42eaab7173eefe54a375aee2ddb500727ed5d0e1426e833453ce53b`原样保留，Parent19:36–19:38Z独立读回确认同一库计数；早期rev1观察继续按历史时点保留。master有效style refs仍空，不能据发布集合授予继承/同步或全局A/B通过，当前也无公共Graph主件UUID。

原PFX已选CPU模板1280板`7f0d509f-128a-4680-a2dc-297a782bca2a`和900板`4097f2bf-5c4f-4095-8b2f-538d57877af1`已有“此source没有script pins或connections”的原生Text说明，leaf分别为`9a0c1383-4d96-44a8-ae96-8c7d01669acd`、`25a9dcd1-47f9-4859-ac1c-0a6d39735c2c`，当时640与五类真实投影尚待交付。Parent第五次接续后继报告PFX rev38/388对象/18copies/106 sourceRefs，真实System/Emitter只读投影和原640 caption已完成且Parent看过1280/640，仍有空value及文字草稿。该后继明确归粒子三源范围，不由材质重做投影、方向/arrow/attachment/fit或引入Material业务模型；本次未重新API、写Penpot或消费外域源码。禁用原因的键盘/弹层访问、真实retained geometry/hit/焦点/undo及生产preview/GPU关联仍须原域与Parent宿主交付；设计草案和只读source投影不替代正常EXE/150%DPI/双端1DIP证据。

交给粒子的最小验收是：真实source nodes可见且ports/edges=0；三种尺寸下selection/Details与同代际pan/zoom/hit一致；每条连接入口禁用且可获得同一原因；旧拖线切到无连接source立即取消、没有delta/undo且焦点返回；真实空数据与未就绪/失败区别；未来source重排使旧locator/编译结果失效。后续由粒子consumer证明actualretained/undo/EXE/DPI和生产粒子preview/GPU关联，本轮未新增Rust测试/请求、Penpot写入、服务或桌面操作；原core19#353编译前冲突、材质13#352及新增fit七项未运行分别保留。完整实际file/page/board/main/shapeRef与source hash记录在本域`pfx-empty-ports-contract-08.json`，passed/accepted仍false。

## Widget 消费适配：UI-SH-GRAPH-01、W11/W12/W20

本节接续 Widget [共享 owner 的具体消费合同](widget-ui-design.md#共享-owner-的具体消费合同)，是现有 MAT-SHARED-001 的消费者约束，不注册新 Rust trait、DTO、Graph 资产格式、组件或运行系统。Parent 负责公共接口和 native host 集成；材质负责中立图表面提案及材质适配；Widget owner `01a16d52-d54d-7ac0-b0d5-21864f383a1e` 负责自己的事件、属性绑定、存储、执行和文档/history 适配。共享层不得调用 `compile_material_graph` 或把 Widget 数据塞入 `MaterialAsset`。

实际 UE 依据为 `UMGEditor/Private/BlueprintModes/WidgetGraphApplicationMode.cpp`：Graph 模式使用 Document、My Blueprint、Details、Palette 及 Compiler/Find Results，`PostActivateMode` 在单选有效 Designer widget 时选择其关联变量；不是创建假事件节点。`Kismet/Private/BlueprintEditorModes.cpp` 注册 Find Results/Bookmarks factory，`BlueprintEditorTabFactories.cpp::FBookmarksSummoner` 返回实际 Blueprint 的书签视图。Zircon `ui/asset_editor/contract.rs::UiAssetEditorMode` 当前只有 Design/Split/Source/Preview，因此 W11 和依赖它的 W12/W20 仍为未实现目标；不更名 Source，不以绑定摘要或调试帧列表代替 Graph 文档。

下表字段是待各 owner 对齐的语义输入/输出，不冒充当前已有公开 Rust 字段或方法。适配应扩展现有 graph projection/context，保持唯一域文档；不能增加第二套可编辑节点存储。

| 接口边界 | Widget → 共享表面/宿主 | 表面 → Widget/宿主及失效行为 |
| --- | --- | --- |
| 文档 target 与能力 | 实际资产 locator、document ID、graph ID、子文档路径；当前 source generation、模型 revision；可编辑/只读及真实命令能力。Target 继续通过 `GraphEditContext<Model, Target>` 解析 | 所有命令、查询和跳转携带同一 target/revision；关闭/切换文档或源解析失效时取消手势。无 Graph 文档能力时保留缺口，不把无实现标成 Ready-empty |
| 稳定 node identity | `GraphModel::NodeId` 直接使用域的稳定 ID 类型，满足现有 Clone/Eq/Ord；标题、排序、Designer 行号和显示名不参与身份。Widget 对源 node、组件/实例路径、变量和事件的关联由域解析 | 选择/拖动/删除均返回原 NodeId；重命名或排序不更换 ID。复制/粘贴由 `GraphClipboardModel` 的域 codec 重映射新 ID 与引用；不能借用材质 node ID 或按当前索引还原 |
| 稳定 pin/edge identity | `GraphPortRef(node_id,direction,pin_name)` 中 `pin_name` 是域的不可变 schema key；另给显示 label、类型和方向。域 connection key 标识边，必要时另给稳定次序 | 当前 `GraphPinView` 没有独立 label、`GraphEdgeView` 只有两端。若域保证两端唯一，可用完整端点组合作边身份；否则 Parent 必须先接域 connection key，未接前不提供会误删/重排的操作。重接、split、改名及兼容 remap 由域决定，不用 Vec index 或显示文字匹配 |
| 文档/视图坐标 | Node bounds 使用 graph/document 单位；同帧测量提供 pin anchor、路径、命中 bounds、文本行框与 sourceRef；宿主提供 pane 物理原点、真实 DPI、裁剪区和 layout/transform generation | 输入先 `(physical - pane_origin_physical) / dpi` 得 pane-local，再调用现有 `screen_to_graph`；绘制用 `graph_to_screen = graph * zoom + pan`，宿主仅施加一次 DPI。pan 是 pane-local 逻辑单位；嵌套滚动/resize/DPI 改变使旧锚点失效，菜单与命中不得用另一套变换 |
| 选择、跨模式定位和焦点 | Widget session 持有源选择 authority，并提供 Designer widget → 实际变量/事件/Graph node 的映射；图的 node set 是它的视图投影，带来源/generation 防递归同步 | W12 激活只选择实际存在的关联目标；找不到就呈现真实缺失，不创建替身。Graph → Designer/Details/Source 跳转由域解析源 node、实例路径、属性/event/binding key 和有效 span。当前 `GraphSelection` 只有节点集合；lead/anchor、pin/edge/field 焦点和 opener-return 仍需 Parent host 接线 |
| 域命令、手势和 Undo | 提供 palette/schema、只读 `can_connect` 与诊断原因、域 delta/inverse、retention proof 和同一文档 history context；执行 pin/data pin、类型转换、替换及外部引用闭包都由 Widget 验证 | begin/update 仅持有有界预览，commit 通过现有 `GraphDeltaCommand` 的 proof → 一次 permit → prepare → apply，形成一次域事务；Esc、capture lost、revision 变化时丢弃预览，错误保持文档不变。创建并自动接线、替换边、删除闭包均为一次原子命令，不直接调用简单 `GraphModel::apply` 绕过协议 |
| 诊断与跳转 | 实际解析/编译/执行诊断、severity、请求 generation、source revision、node/pin/property/event/binding 和调用链 locator；不得送材质编译结果代替 Widget 执行结果 | 共享层只显示并分发真实定位请求，Widget 验证并解析到源；过期结果标 stale，目标已删/文档已关时返回实际不可定位原因。现有 `GraphDiagnostic` 的静态 message 不是完整异步诊断/source-map 接口，仍需接 Widget 已有 structured diagnostics 与 Parent 结果视图 |

Widget session 当前已有 `UiAssetEditorUndoStack`，尚未定位到 Widget 的 `GraphModel/GraphEditContext` 适配。接入必须由 Widget/Parent 对齐实际源文档的既有历史 authority，禁止表面再建图内 Undo 栈，也不能在一次编辑同时向两套历史各提交一次。Source 无效而保留 last-valid 文档时，应显式标明 last-good/只读或阻止编辑；不能对旧图修改后覆盖用户的新 Source buffer。dirty/save/close 与事件/绑定持久化继续由 Widget 文档 owner 执行，材质保存流程不介入。

Designer 的 Canvas anchors/offset 与 Graph 坐标分属不同视图，Graph 的 pan/zoom 按实际 graph target 隔离；跨模式只同步稳定选择与源定位，不把 Designer 布局数值当作 Graph 节点坐标。一个 widget 对应多个变量/事件/图节点时，由域返回真实候选或显式选择，不按重复显示名取第一项。

焦点意图必须区分 canvas、node、pin、edge 和真实 editable field。只有 Graph scope 实际聚焦时处理其快捷键；文本框、Source 编辑器和 IME 优先消费输入及自身 Undo/Delete/复制。节点搜索/上下文菜单使用同一 palette/schema 与域创建命令；关闭回原 opener pin/node/field，目标已删除则回仍存在的节点或 canvas。视点和选择不进入材质或 Widget 资产编辑 Undo；跨模式/子文档返回保留该文档的合法 camera/selection，不按另一个资产的 ID 恢复。

W20 Bookmarks 与 Graph Find（关联 W13）需要真实 Graph 文档后才能接入。Widget 提供真实书签集合/查询索引及存储 owner，记录实际 target、稳定 node/source locator、标签和可恢复的视点；标签不是定位 key。共享表面只接受导航/fit/camera 请求，Parent 提供文档打开和合法视点恢复。当前 `GraphCanvasState` 的 pan/zoom 及鼠标锚点缩放可复用，但没有已确认的 Bookmark 持久化/restore 接口，不能把书签写进 MaterialAsset 或另建全局存储。节点删除或源 revision 失效时先重新解析稳定 locator，不能跳到相同行号/数组索引的新对象。

Find 的 query、scope、取消和异步结果携带实际 target/source revision/request generation；每一条结果必须来自当前真实图及有效 locator。旧请求不得覆盖新查询；真实查询的零结果、未准备、无 Graph 能力及失败必须区分。Source 文本查找仍可按 Source 的真实能力存在，但不标为 Graph Find，也不填假 Bookmarks/固定搜索结果。当前这些域集合、查询适配与跳转都未实现，面板 factory 存在不构成能力通过。

| 能直接复用的现有基础 | 尚未实现、不得对外称已接通的部分 |
| --- | --- |
| `GraphModel` 泛型 NodeId/域 Delta、节点/端点视图、`GraphCanvasState` 的单一变换/框选/拖动预览、水平路由、`GraphClipboardModel`、`GraphEditContext/GraphDeltaCommand` 既有协议 | Widget Graph 文档/模式/存储/执行及生产适配；pin label、非唯一端点的 edge key、子目标/跨模式 focus；Widget 既有源历史与 command context 桥接；动态诊断和源定位；真实 Bookmarks/Find；native paint/input 与公共发布 |
| 本域 Penpot 中已有 3 个节点主件及 5 个图状态板，可作为表面几何、flow、连线和中文标题的设计依据 | 这些仍是 Material Parameter/Math/Output fixture，未发布为中立公共库，没有 Widget 原生消费者或导出证明；不能改个标签就称 Widget 事件图组件复用完成 |

交接验证按真实消费者而非示例结果进行：W11 创建/连接/移动/删除后一次 Undo/Redo、保存关闭重开保留稳定身份；W12 从真实 Designer 单选到现有变量/事件，再回 Details/Source，中文输入/弹层取消后焦点正确；W20 创建实际书签、重命名节点、保存重开及目标删除后的定位失败；Graph Find 执行真实查询、零结果和过期请求取消；接线拒绝、源 revision 失效与拖动取消均不改变文档；同一文档只产生一条历史提交。真实宿主再覆盖三尺寸与 150% DPI 的绘制/命中/文本行框和原生证据。以上运行场景均未执行，当前只完成合同/来源核对，不升级 W11/W12/W20 或整族验收。

### Widget 接续：canonical 源、版本和组件身份

接 Parent 最新 UI-SH-GRAPH-01 与 [Widget 事件/宿主消费者](widget-ui-graph-consumer.md)、[binding 本地源定位候选](widget-ui-binding-source-locator.md)。本轮只改本合同和材质台账；公共 Rust、Widget/Actor/AI 源码与原ZUI、catalog/theme/插件均只读。Widget 原资产 `zircon_editor/assets/ui/editor/ui_asset_editor.zui` 已实际解析为v2、97 nodes/97唯一control IDs/6 events，原六事件逐项登记，不重命名为Graph操作。本次读到 Widget owner 设计文档已记rev12/2978对象/8本地主件/12份Shared linked copies；Parent所述rev9是较早真实阶段，本次没有重新API读回Widget文件，不给它虚构新revision或验收。

可消费的源码入口是已有 `zircon_editor::ui::graph`，版本身份为 checkout基点 `75a76ee963792d5a316f738519e966937f770036` 加逐文件working hash；不是新发布的组件或语义版本。完整30路径指纹、原Widget六事件、主件身份及证据强度在本域 E 根 `widget-shared-graph-contract-04.json`。下表哈希只作索引，完整SHA见该回执：

| canonical 源 | 当前hash前缀 / 已有公开能力 | 本次消费边界 |
| --- | --- | --- |
| `ui/graph/mod.rs` | `2a0a4cda94a77b8f`；已有 proof/prepared、视图、canvas、command和方向辅助公开路径 | 不增加第二trait/graph模型；原V1与private构造边界保留 |
| `ui/graph/model.rs` | `ba24acaa709b08cd`；稳定域NodeId、schema-key端点、views及retention proof/prepared | 只读；pin显示label、域edge key、动态诊断/SourceLocator仍需域/host适配 |
| `ui/graph/commands.rs` | `0a94bdc81e7e817c`；`GraphEditContext` / `GraphDeltaCommand` | 只读；原文档history、revision/retention/一次permit保持，不能直接apply代替 |
| `ui/graph/canvas.rs` | `23ea48fa11978477`；pan/zoom/节点选择/框选/拖动预览 | 只读；不携带实际Surface frame、clip、pin/edge focus或生产hit数据 |
| `ui/graph/routing.rs` | `97279317298fa43a`；旧水平API与可选四向路径/tangent/arrow | 只读复用；同一测量生成route后供paint/hit，不能另算一条装饰wire |
| `ui/graph/node_widget.rs` | `91c617b4f3edafc4`；原计数/标题投影及独立附件原ID/host投影 | 不冒充native node控件；附件真实bounds/instancePath/焦点还未提供 |

当前 catalog 只检索到域专用 `AnimationGraphPaneBody`，未有中立GraphSurface/GraphNode/GraphPin canonical条目、正式wrapper、native组件UUID或发布版本。Parent14:46Z对材质rev48/isShared=false/1063对象/5main的观察保留为历史；本域较后真实设计回执到rev55/1923对象/5main/102copies/69外链实例，不将旧库存覆盖后续进度。材质 Parameter/Math/Output 的component/main分别是 `904a343b-18f1-4af9-928f-b7d5d028a69a` / `a4544d75-53ca-40e7-99c8-8dfeabdb69ed`、`f9ea0da8-6a97-4ccb-b57d-b31b273ea4f0` / `8a349678-e6b3-4dcf-99fc-c1fe51efee35`、`d7173cd6-1da7-4897-be89-08583d32f598` / `926a240d-18dd-4741-9cdc-8ce08292e721`；另两个main为Material Function业务。它们只提供已保存的域设计/flow参考，没有中立schema、adapter、正式codec source记录，不能作为Widget节点API或复制到Shared冒充发布。

真实Shared来源仍是file `b4a1e3ce-9e6a-81fe-8008-d69d607aac12` / page `b4a1e3ce-9e6a-81fe-8008-d69d607aac13`，已有canonical-record schema v1及rev1的Button/SearchInput/TreeRow/PropertyEditorRow四主件。schema版本不是组件语义版本、source hash或发布变更号。组件/主件/source路径及hash沿既有 `shared-graph-consumer-reconcile-03.json`；合同阶段只复用其有时间戳的身份。17:27Z另一次只读API核对仍为Shared rev1/41对象/4main/0色与字体、Material rev55/1923对象/5main/102copies/69份Shared实例，见 `penpot-material-export-readiness-10.json`，不是完整库/样式/A-B通过；中立图仍无可预填的公共UUID。Parent正在交付的worker R3、字体/auth/browser路径及旧runner配新dist兼容证明保留给原pipeline owner，本合同不补兼容成功。

### Widget 接续：slot、事件与 override/reset

以下“区域”是要交给Parent核准的职责，不是已经新增的ZUI slot名或Rust enum。只有表中明确存在的catalog slot可按原键消费：

| 区域 / 已有来源 | slot、参数与事件输入 | 输出、owner和缺口 |
| --- | --- | --- |
| DocumentHost / Graph pane | 已有 `DocumentHost.content` 接受PaneSurface；真正graph target、document incarnation、source/model revision、readonly/capabilities由域提供 | Parent安装实际pane/body及canonical wrapper。CanvasBox/ScrollableBox占位、Widget Source或visible窄Host候选都不构成Graph来源 |
| 节点header/body、pin行和附件 | 原NodeId及完整port key、显示名/类型、当前节点bounds；附件用原id/host和域提供的instance/source path。pin方向、执行/数据语义与样式角色分开 | 同一layout generation量出文本/anchor/clip，供共享paint/hit；标题/label不参与身份。旧计数/labels投影不补真实pin、附件独立命中 |
| 真实节点属性 / Details | 已有 `WorkbenchPropertyEditorRow.value` 必需、单值、linear，接受Checkbox/Dropdown/Field/NumberField/RangeSlider/Slider/Toggle七类原Workbench控件；typed值、草稿/错误/只读/commit/cancel来自域 | 当前5份value内容仍为Text fixture，不能算typed slot满足。Parent沿原字段主件发布和绑定；材质不复制NumberField或把文本写成参数编辑器 |
| 诊断 / 定位 | 已有 `WorkbenchDiagnosticRow.severity` / `message` 分别接受WorkbenchStatusItem；域提供真实诊断及request/source revision、node/pin/property/binding locator | Parent结果视图分发导航；现 `GraphDiagnostic` 的静态字符串不承担完整异步source map，错误/取消/过期保持可识别 |
| 搜索 / 菜单 / 弹层 | 复用已登记SearchInput、ContextMenu/PopupMenu的真实来源；仅显示域palette和当前capability，可用/不可用/零结果/失败分开 | 父host保留opener、输入scope和返回焦点；不能拿材质六节点palette、固定Find结果或Material compiler处理Widget请求 |

图事件只传真实意图：选择/定位、pan/zoom、开始/更新/提交/取消节点拖动或拖线、域创建/删除/断线/替换、搜索菜单命令及诊断导航。每条意图关联同一asset/document/graph/subdocument、model/source revision、实际source frame和layout/transform generation、稳定node/port/edge、gesture/request与opener来源；不是原Widget六条chrome route的改名。输入已经是pane-local DIP时直接 `screen_to_graph`；只有原始physical输入才先减pane物理原点并除一次实际DPI。绘制使用同一 `graph_to_screen`，由实际host施加DPI一次；clip/resize/panzoom或测量失效就拒绝旧anchor/hit，不能按坐标相近重定位。

开始/更新只持有按既有retained预算准入的有界预览，不修改作者文档。typed pin合法性、单输入替换、显式转换、源引用和完整delta/inverse归Widget/Actor；当前ConnectVerdict只有Allowed/Rejected，不由表面猜额外Replace语义。commit沿原文档 `GraphDeltaCommand` 的retention plan/proof、一次materialization permit、prepare及revision复核形成一次事务；创建后自动接线、替换旧/新边和删除闭包一起提交。Esc、capture lost、文档/源或transform失效取消并丢弃预览；失败保持作者值不变，不在hover预检分配整树候选。文本字段/Source/IME保持本身编辑、Undo和blur语义，图快捷键不能吞其输入。camera/selection/focus不另建资产Undo栈；Widget已有历史与Graph命令的桥仍未实现。

样式沿原catalog/source、tokens与Starship的实际继承和state优先级；具体precedence及缺失图角色由Parent在原源闭包确认，不在本域烘焙平行颜色/字体。允许的局部内容/尺寸/样式override须记录原source owner、slot、shapeRef及实际Penpot override group；不能借override改component/node/pin/edge身份、event/binding或readonly来源。已观察的Penpot2.17 `content-group`、`text-content-text/text-content-attribute` 和padding/gap组差异继续保留，不能以字形重置失败后的静态截图补传播成功。样式reset通过原owner删除实际存在的局部key/override并保持linked identity，不写空字符串、不detach或复制main；属性值reset则由域生成同一文档的typed delta，不能把材质实例presence规则泛化给Widget类型。二者都要在真实实例保存重开及token A/B后才能通过。

### Widget 接续：SourceLocator、Details与焦点事务

已有可复用的本地候选是 `UiAssetEditorSession::locate_binding_source(selector, Current)` → `UiAssetBindingSourceLocation` → `select_binding_source_location(location)`；类型是crate内可见、字段私有，不是已发布的SourceLocator provider或dispatch permit。实际源码检查buffer allocation/snapshot、route资产身份、last-valid generation、当前parsed文本及原binding完整值，成功才改变本节点/binding选择和精确UTF-8 byte cursor。LastValid不可选择；同ID同文本关闭重开、源修改后undo、过期ticket、owner/event不匹配均不能据旧offset降级跳转。候选12项用例已写未执行；当前没有宿主调用边或源整段selection/行框验收。

Parent/Widget接续按以下顺序完成同一导航事务，不创建第二份图或source store：

1. Widget用当前真实graph node/port→widget/owner/binding映射提供locator；一个来源对应多目标时返回真实候选，URI由宿主真实route/resource解析，不能把 `route.asset_id` 猜成路径，也不把binding声明变成外部handler Definition。
2. Parent核对目标document/graph/subdocument/incarnation、可用pane和当前source/model revision，取消已失效图手势；Widget只为Current有效源生成并核验本地ticket。当前图/Source无效、无Graph文档或无provider分别返回真实不可用原因，不创建占位节点。
3. 在已有session/编辑guard中执行已核验的selection/cursor变更，释放锁后再同步Details/Source投影并请求实际host焦点；回调或handler不在session锁内重入。导航只改视图选择/cursor，不写作者源、dirty或图Undo，也不执行Route/Action/VM函数。
4. 发布前重核同一实例、source和选择请求generation；晚到结果不能覆盖新选择。搜索/菜单关闭回仍存活的原pin/node/field，删除后回合法canvas；Source/IME或其它文档已接收焦点时，不按旧异步请求抢回。UI焦点失败不假称完成或回滚已发生的外部操作，实际导航/取消结果由Parent原history与focus owner记录。

Widget本地ticket只是这条链中一段；真实Graph映射、跨文档Definition provider、Details同步、native focus/cancel及图模式仍缺，UI-SH-GRAPH-01/W11/W12/W20不因本候选转为passed。

### 实际源码消费者与运行证据缺口

本轮只读定位到 [AI adapter](../../../zircon_plugins/ai/editor/src/authoring/graph.rs)（SHA `335a5cca98a30ce9b3b6f172e5f96f39dd9684b57e6576445279fd18d90a9ce9`）。`AiBehaviorTreeGraphAdapter::route_edge` 在核对真实tree id/revision、原typed端点和现存作者边后，确实调用已有 `route_connection_with_directions`，source Down、target Up，使用caller同次提供的graph-local anchors；没有重新施加pan/zoom/DPI，也没有制造运行trace。模块从AI `authoring/mod.rs` 和 `lib.rs` 实际重导出，是跨crate源码消费，不是把Material节点改名。

[AI现有消费测试源](../../../zircon_plugins/ai/editor/src/tests/behavior_graph.rs)有10项定义，其中4项实际调用route helper的adapter路径，覆盖提供的anchors/垂直切线、foreign/stale/fabricated edge、wrapper原身份/leaf端口及非有限/溢出测量；它们不是本域新增或本轮执行结果。scoped检索目前仅找到导入/重导出和这些测试调用，没有生产retained caller、实际frame/paint/hit或运行图像。adapter明确不实现 `GraphModel/GraphEditContext`，`candidate_for_connection` 的整树候选尚无正式retention/一次permit authority，不能在hover中直接调用或当GraphDelta提交。本条交付的是一个真实**源码级helper消费者证据**；用户要求的真实消费者行为+图像仍open，不能用材质原型图、手写geometry夹具或SourceLocator单元用例替代。

### 给Parent的公共逐文件提案

以下仅为已有系统的最小接续候选，未授予本域写权；本轮没有应用任何公共patch。每项以Owner确认及实际source closure为前提，复用现有接口足够的文件保持0 diff。结构化路径、当前SHA、依赖和验证范围在 `widget-shared-graph-public-proposal-09.json`：

| 路径 | 提案与归属 | 进入条件 / 聚焦验证 |
| --- | --- | --- |
| `zircon_editor/src/ui/graph/{mod,model,commands,canvas,routing,node_widget}.rs` | 只读复用上述已有基础；本轮公共Rust请求0行，不扩proof/permit/model或业务type | 原core19/材质13终态与身份保留；文档变更不跑Cargo |
| `zircon_editor/src/ui/retained/contract/data/ui_asset/pane/editor.rs` | Parent核准真实graph只读pane投影与target/frame/transform/能力关联；它现在没有Graph字段，不把Preview/Source数据塞入假Graph | Widget实际文档及合法投影存在后才接；缺能力显式不可用，禁止Ready-empty |
| `zircon_editor/src/ui/retained/contract/globals/pane_context/callbacks.rs` | Parent先核现有pane input能否承载真实Graph source frame；需要扩展时保留可验证来源及单次坐标转换，不伪造UiSurfaceFrame或绕visible/frame准入 | 输入/paint/hit同帧及zoom/clip/resize/DPI失效；不复制第二套dispatcher |
| `zircon_editor/src/ui/retained/ui/data/pane_ui_asset_conversion/mod.rs` | Parent在现有投影中接实际graph slot/Details关联，保持readonly借用/预算及完整sourceRef，不新建可编辑Graph存储 | 同source revision、稳定实例/typed slot及旧preview/source行为 |
| `zircon_editor/src/ui/retained/app/callback_wiring/pane_surface/ui_asset.rs` | Parent维持现有真实preview输入分支；真实Graph capability具备后才转交对应surface，不将visible窄probe或原六route当Graph执行 | 当前输入/clip/focus准入、文本/IME隔离、Esc/capture-lost、旧frame拒绝 |
| `zircon_editor/src/ui/host/asset_editor_sessions/editing/binding.rs` | Parent协调Widget现有checked本地ticket的host包装、selection/投影及解锁后导航焦点；Widget仍拥有原locator实现，Actor/VM拥有外部Definition | Current/LastValid、同ID重开、edit→undo旧ticket、重复实例与中文/CRLF；不产生文档Undo或执行handler |
| `zircon_editor/assets/ui/editor/components/catalog.toml` | Parent沿实际canonical源登记中立graph wrapper/slots版本，先有源和生产能力再分配身份；不得注册材质3原型当通用节点 | 真实native main/copy/shapeRef、slot类型与source/export guard通过后再发布/链接 |
| `zircon_editor/assets/ui/editor/theme/editor_tokens.zui` | Parent复用现token域，确缺图样式角色时在原表中补；本域不另开主题或字面值表 | 同token跨至少两消费者A/B、局部override/reset、保存重开及产品传播 |
| `zircon_editor/assets/ui/editor/unreal/starship.zui` | Parent把经确认的图状态角色接原class/state体系；端口业务类型由域适配，不导入Material语义到全局 | 选中/焦点/disabled/诊断及不同域类型保持，无复制私有主题 |
| `tools/penpot/apps/zircon-zui-plugin/src/penpot-canonical-library.ts` | 原pipeline owner核实正式composite主件、source/copy/slot/revision及局部override/reset；仅需补确证缺口时改 | 沿原worker R3及实际旧runner+新dist兼容回执；本域不改Shared/main或重启服务 |
| `tools/penpot/apps/zircon-zui-plugin/src/penpot-export-guard.ts` | 原pipeline owner保持严格source/未知字段/绑定事件/测量来源守卫，不放行手工拼shape或任意geometry重签 | 正式导出→原asset加载→normal EXE画面变化前仍pending |

实际中立node/pin/edge painter和同帧geometry producer尚未定位到可声称已实现的公共源；Parent须明确其原native owner和接入路径后才纳入逐文件写权，当前不虚构现成renderer文件或通用组件UUID。WidgetGraph schema/可编辑模型、typed连线及原子delta、持久存储/编译/执行/diagnostics归Widget/Actor；原asset97IDs/6events、source locator候选及visible Host不由材质修改。后续真实consumer仍需19项core、受影响域adapter、实际focus/事务与正常EXE三尺寸/150%DPI/逐图证据；现在只做合同链接、结构、只读指纹及精确scope guard。

## 角色消费适配：CS-GR01、CS-D02/03/04

本节接续角色计划第 5.3、10 节及 CS-GR01，仍为同一 MAT-SHARED-001 的消费者合同。材质牵头表面提案；Parent 负责公共接口、native host 与跨域集成；角色 owner `01a16d52-6b1e-7543-82f0-4f05ca92ad58` 负责实际 Rig/OpStack/typed 文档、控制与求解、pose、文档/history 和持久化。以下字段表示适配时的语义要求，不冒充已经存在的 Rust API、Rig 资产路径或能力注册。

只读 UE 依据：`ControlRigEditor/Private/Editor/ControlRigEditor.cpp` 通过 controller 的 `SetNodeSelection` 选择模型节点，并以实际 pin path 调 `SetPinDefaultValue`；`IKRigEditor/Private/RetargetEditor/SRetargetOpStack.cpp` 通过 `MoveRetargetOpInStack(DraggedElement->GetIndexInStack(), IndexToMoveTo)` 重排，选择与查找另用 `GetSelectedOpName/GetRetargetOpByName`。这些调用证明模型、pin 和有序操作的域适配必要性，不能据此假设 Zircon 已有持久 Rig UUID，也不能将显示名、当前列表索引当作跨 revision 身份。

| 接口边界 | 角色域 → 共享表面/宿主 | 表面 → 角色域/宿主及失效行为 |
| --- | --- | --- |
| target、文档与能力 | 实际 asset locator、document/graph/subgraph target、typed resource/skeleton revision、模型 revision；独立 surface/session/layout generation；真实 edit/connect/compile/evaluate/save 能力 | Target 沿用 `GraphEditContext<Model, Target>`。打开、重导入、切换骨架/子图、关闭文档及 revision 不匹配时取消旧手势/请求，按域重新解析。无 Rig 文档能力时保持未实现，不以可画的空 canvas 标为 Ready |
| 稳定 node/pin/edge identity | `GraphModel::NodeId` 使用原域稳定类型；pin 用 node ID、direction、稳定 schema key/typed field path；数组/结构子 pin 带域结构版本与明确 remap；edge 用完整端点及必要的域 connection key | 改标题、排序、切换语言不换身份。当前 `GraphPinView` 无独立显示 label，`GraphEdgeView` 无独立 edge key，Parent 仍需接该投影；端点非唯一时，未接 key 前不提供会误删/误重排的操作。复制与 ID 重映射由域 codec 负责 |
| 层级、控制和 Op 身份 | bone/control/goal/solver/op 的实际 typed key、所属资源及 revision；图节点关联的 hierarchy/source locator 由域解析。若底层只给 index，则该 handle 仅在当前 revision 有效 | 跨图/树/Details 定位用原 typed key。重排/重导入后重新解析，失效时返回真实缺失；不能按相同 label、骨骼行号、旧 op index 或主 PIE entity 找第一个替身。持久 key 迁移仍由角色 owner 提案/实现 |
| typed pins 与连接合法性 | execution/data 区别、实际 type/schema identity、端口方向、局部/全局空间和单位约束、required/default、允许转换/替换、环/执行顺序验证与拒绝原因 | 表面呈现真实 `can_connect`/preflight 结果。不得以材质 Scalar/RGB、默认字符串相等或所有工具一律 Dag 代替 Rig 类型与拓扑；`StructureConstraint::FreeGraph/Tree/Dag` 由实际域图选择。类型转换、替换与 inverse 由域组成一个 delta；当前通用 verdict 并无完整 Rig 校验器 |
| graph/document/view 变换 | 图节点 bounds/position、pin anchor、路径、命中与文本行框来自同帧测量；pane 物理原点、真实 DPI、pan/zoom、transform generation 由宿主给出 | 物理输入只除一次 DPI 后调用 `screen_to_graph`，绘制 `graph_to_screen` 后只乘一次 DPI。图坐标不写入 `body_offset`、bind pose 或 control 局部/全局变换；节点拖动和 pose gizmo 是不同域命令。Physics profile 当前无 graph position 字段，位置持久化须接既有 Editor view-layout owner 或真实作者字段 |
| 选择、焦点和 picking | 同一文档 selection authority、typed source locator、lead/anchor/子目标及来源 generation；preview 的 subject/session、对象映射和有效帧/资源 revision | 图/Hierarchy/Details/viewport 的选择通过域映射同步并防递归；`GraphSelection` 当前只有 node set，子 pin/edge/field 焦点与 opener-return 仍待宿主接入。popup、文本/IME 优先处理输入；图仅在其 focus scope 处理快捷键。删除 opener 回合法 node/canvas，selected 与 solver/runtime active 分别呈现 |
| 作者 command/history | 真实 palette/schema、read-only preflight、目标 revision、域 delta/inverse、retention proof 与同一作者文档/history context。资源编辑和 view-layout 编辑各由实际 owner 声明 dirty/history 策略 | 通过现有 `GraphDeltaCommand` 的 proof → 一次 permit → prepare → apply；创建并接线、替换、删除闭包、域持久节点移动各一次原子提交。预览不写作者模型；Esc/capture lost/revision 失效取消，错误保持原文档。pose gizmo 一次拖动一次域历史，view-only pan/zoom/选择不加入资产 Undo；不另建图内历史栈 |
| 诊断与跳转 | 真正的解析/编译/validator/solver 结果、severity、请求 generation、source/model/compiled revision，以及 graph/node/pin/op/control/bone/调用链 locator | 表面只显示与分发导航，由角色域验证并定位到实际对象。过期异步结果不能覆盖当前结果或按旧 index 跳新 op；目标删除/源不兼容时呈现实际失效原因。现有静态 `GraphDiagnostic.message` 不构成动态 Rig source map 接口，须接域诊断与 Parent 结果视图 |
| 求值、pose 与保存 | 实际 Rig/retarget 求值 owner、兼容的 source/target、求解与 pose generation、动画 owner 的真实时域、preview subject/session/frame；dirty/Apply/Save/Export 的真实资源变更与输出 locator | 依 CS-D02/03 接既有 preview/文档链，旧资源或旧 pose 帧失效；最后消费者释放真实预览。共享表面不求解、不创建另一套时钟；保存/导出须可打开并重放真实输出。材质 compiler/MaterialAsset、静态 preview 和选中高亮均不能替代 Rig/pose 结果 |

Control Rig 的图 authoring 与运行选择分离：运行时控制/活动节点必须带实际求值 session/revision，不能由点击或图表面自行计时生成。一次 pose 修改只有在真实求解、预览帧及对应 source/target 资源链成立后才可报告效果；`TwoBoneIk/LookAt` 数学测试源码的存在不能证明 RigVM、Modular Rig、执行调度或可保存 Rig 资产已接通。`ui/animation_editor/capabilities.rs` 中的 animation-editor 能力表也不能替角色域授予 Rig 能力。

Retarget OpStack 保留 ordered-list/tree、Hierarchy 和明确的 pose 编辑模式，重排使用域的有序操作命令、约束和同一历史；退出 pose 编辑按角色合同恢复对应播放状态。只有实际提供图文档/拓扑的区域才挂 graph 表面，不给 OpStack 加装饰线或用“Graph 可画”证明 retarget 成功。Apply/Export 与普通保存的语义由域定义，必须返回真实已写资源和失败原因，不能只更新一行状态文字。

CS-GR01 同时覆盖 Physics Asset Graph：当前 RagdollProfile 的每骨单 body/shape、parent constraint 只能投影其真实关系。readonly relation view 与 editable topology capability 分开；任意 joint、多 body 或拓扑编辑缺口保留。节点选择通过既有 provider/mirror、subject/session 和实际 spawn 映射接 collider overlay，不直接把主 PIE entity 当预览 entity。

| 接续路径/owner | 可复用来源 | 尚待接入与边界 |
| --- | --- | --- |
| `zircon_editor/src/ui/graph/{model,canvas,commands,routing,node_widget}.rs`、`mod.rs`；Parent 整合，材质提案 | 泛型 NodeId/Delta、端点视图、单一 pane 变换/框选/拖动预览、现有水平路由与 retention-bound command | proof/prepared 最小 re-export、pin label/edge key、typed 子目标焦点、方向/箭头/命中、动态诊断及实际 native paint/input/sourceRef 仍 pending；本轮未写这些文件 |
| 角色原资产 `zircon_editor/assets/ui/editor/animation/extension_control_rig_workspace.zui`、`extension_retarget_workspace.zui`；角色 owner | 已有 v2 ID/事件/绑定与本域设计/CS-GR01 | 角色自行接真实 typed 文档、author commands、Rig/OpStack/求解/pose/保存；15 个事件存在或 `Not implemented (preview)` 日志均不构成业务完成。本轮只读指纹，未修改资产/runtime |
| `zircon_editor/src/core/editing`、`zircon_editor/src/ui/preview_scene`；角色域适配，Parent/实际 owner 整合 | 既有作者事务、preview backend/provider 与中性 render 链；CS-D02/03/04 管理资源/history/host/选择 | 实际 Rig 文档入口、revision-bound typed mutation、source locator、GPU frame/picking 与 teardown 未由共享表面提供；新资源/注册路径由角色 owner 明确并交 Parent，本文不虚构文件或返回值 |
| `zircon_plugins/animation/runtime/tests/animation_ik_contract.rs`、`zircon_editor/src/ui/animation_editor/capabilities.rs`；只读来源 | IK 数学合同源码、特定 animation-editor 的真实能力声明 | 本轮未运行测试；上述来源不证明 Rig/retarget 产品链。预览时钟与播放控件继续归动画 owner，共享图不复制播放器 |
| 本域 Penpot file/page、3 个 Material 节点主件与 5 个图状态板；材质保留身份 | 中立表面几何、flow、端口/线及中文标题的设计依据 | Parameter/Math/Output fixture 不是 Rig typed 节点，未发布公共库或接角色实例；不得仅改标签宣称消费者复用/验收 |

角色交接验收保持 pending：真实 Rig 文档创建/typed 连接/拒绝/移动/删除分别完成一次 Undo/Redo；保存关闭重开仍能按稳定 node/pin/edge 与 bone/control/op key 定位；局部/全局空间与 execution/data 类型由域验证；图/层级/Details/picking 同步且旧 session/reimport 结果失效；OpStack 重排失败不变、一次撤销，pose 模式返回播放状态；真实求解修改 pose 后正常 EXE 帧变化、旧帧拒绝和最后消费者释放；Source revision 变化与弹层/拖动取消正确归还焦点。随后按三个尺寸和实际 150% DPI 核对绘制、命中、行框及原生双端证据。以上运行项均未执行，表面合同不提升角色、材质或整族 passed/accepted。

## 路由、运行状态与 UE 依据

材质继续对照 `MaterialGraphConnectionDrawingPolicy.cpp` 和 `MaterialNodes/SGraphNodeMaterialBase.cpp`；AI 对照只读 UE `AIGraph/Private/AIGraphConnectionDrawingPolicy.cpp`、`BehaviorTreeEditor/Private/BehaviorTreeConnectionDrawingPolicy.cpp`。AI 的 `ComputeSplineTangent` 是锚点差的安全单位方向；普通线 1.5、LineSeparationAmount 4.5，箭头按实际方向旋转，源码还显式留下 separation 是否随 zoom 缩放的 TODO。不能把未决定的缩放行为当成已经对齐。

现行 Zircon 已有上节四向 opt-in、归一化端点切线和目标箭头投影，旧水平路由保留；实际 pin/arrow bounds、绘制/命中、zoom 后线宽与 hit tolerance 仍待生产 consumer 接入。当前四向算法不宣称等于 UE 直线单位切线/brush 合成；同帧路径和来源必须与实际 retained 几何绑定。水平、垂直、反向、同点/极短的回归源码不代替缩放/真实 DPI/运行场景，不针对单图加偏移。

AI 源码 10 宽+bubbles 依真实两端 active/previous 状态；暂停搜索结果 5 宽/成功失败色/反向另有条件。选择状态不能触发这些运行样式，缺 trace 时保留普通作者边。动画权重、Blueprint 执行流、粒子运行状态同理，由真实域反馈绑定 revision/session，而非图表面自行计时/猜路径。

公共主题继续由 Parent 导入 Starship/域专用样式与 linear→sRGB 颜色链。节点标题色、pin brush、文本、阴影、网格、selected/focus/error 继承同一全局来源；本域不发明 `material-*`/`ai-*` 平行主题。公共状态优先级须保留域真实谓词，不能用 selected 覆盖运行错误或虚构 active。

## Penpot、导出与原生证据现状

此前真实只读 API 读回材质 file `b4a1e3ce-9e6a-81fe-8008-d69d6092d704`、page `b4a1e3ce-9e6a-81fe-8008-d69d6092d705` 为 revn12，后续修正并重开至 revn33；图状态阶段回执 `penpot-graph-state-04.json` 为 revn40 / 330对象 / 3主件 / 21实例 / 133flow，保留新增中文长名称、未接线根及原238对象。其后 Material rev43、Function rev45、Instance rev48 / 1063对象 / 5本域主件 / 69实例分别保留原图/关闭重开/逐图记录，最新设计事实见材质计划及 `penpot-instance-empty-design-01.json`，不把旧库存当当前状态。公共 file `b4a1e3ce-9e6a-81fe-8008-d69d607aac12`、page `b4a1e3ce-9e6a-81fe-8008-d69d607aac13` 后续实际为 rev1 / isShared=true / 4canonical主件 / colors=0 / typographies=0，全页45实例已跨文件引用；公共样式、正式导出/传播及产品复用未验收。本次四文件 Rust 辅助接口阶段未重开浏览器或修改 Penpot，不把已有设计图作为新几何/consumer验证。

原材质 board `2fa31016-62fd-45d2-82d7-83457be89e6c` 的 Parameter/Math/Output 主件 ID 分别为 `904a343b-18f1-4af9-928f-b7d5d028a69a`、`f9ea0da8-6a97-4ccb-b57d-b31b273ea4f0`、`d7173cd6-1da7-4897-be89-08583d32f598`，身份均保留。新增 graph-only 900×620 board `ce4a6bdc-5f45-4151-b8ee-e07228bf4328` 和 640×520 board `b89331c9-90de-4ee7-bd5f-ffc51cd845ad`，以同一组主件的 10 个新实例展示不同 camera；不是完整编辑器窗口或真实 DPI 验证。保存、关闭、重开、内部 pin/path 几何及逐图差异见 [材质计划](material-family.md)。后续公共library引用已存在，但公共style、正式 ZUI 导出、正常 EXE 图表面和这些图业务主件的跨域复用未通过，不授予其它域复制为新公共主件的结论。

公共图主题还需补齐具体源合同：`SNodeTitle::GetHeadTitle` 对可改名参数使用无引号 EditableTitle；`Graph.Node.NodeTitleExtraLines` 使用 9pt Italic，额外行通过 snap 补高；`SNodePanel` 网格默认 snap16、每8格rule、zoom≤.5 时 inflation 调整；ColorSpill/九宫格和 pin brush 需要真实资产。当前 Penpot team 仅有 Roboto normal 400/500/700、Noto Sans SC normal 400，缺少 Roboto Italic；本域图设计仍有副标题粗体、灰色标题、圆形 pin 近似和缺阴影等差异。Parent 负责字体、共享样式/组件身份与跨域发布；不能以本域自有组件代替公共主题传播验证。

实际组件同步验证还发现 runtime/source 差别：运行的 Penpot2.17.0 除 `content-group` 还有 `text-content-text/text-content-attribute`；缩放后的 native padding/gap 也需声明各自 override group。遗漏时真实编辑文字会将实例样式重置为主件值，导致 pin 错位或 glyph 放大。已保存 2.26/5.66 DIP 失败及“pin 通过但字形错误”的失败，修复后以键盘改名/恢复、关闭及新 context 重开证明布局与字体不再重置；见材质计划与 receipt-11/12。共享发布/导出 codec 须按真实 runtime 合同读取 override，不将本地参考树当作部署版本。字体 attribute override 粒度较宽，局部字号与全局颜色联动/reset 仍是开放依赖，不据此宣称共享样式传播通过。

每个后续 LayoutReviewCase 应将实际 document/hash/revision、graph target、Penpot file/page/board/main/instance、原 ZUI/sourceNodeId/instancePath、布局文本行框、transform/DPI、EXE/资源/主题闭包、同次 native capture nonce/generation/process/media-ready 连接起来。布局变换后 node/pin/edge/attachment 仍要逐项定位；不可编辑 bitmap、装饰 wire 或缺来源的截图不能作为图接入证据。

## 具体交接与验证准入

| 接续项 | 路径/owner 与可审查结果 | 状态 |
| --- | --- | --- |
| 外部插件命名合同 | `zircon_editor/src/ui/graph/mod.rs` 保留两个原 proof/prepared 与既有 GraphAttachmentView 导出，兼容辅助 exports 见四文件节；原 model/retention 不变，外部域 adapter 编译仍待证据 | export source edited；core#353在编译前被来源守卫拒绝，不计消费者或产品接受 |
| 方向路由/几何/命中 | 材质在四文件授权内增加四向 opt-in、路径切线/目标箭头、原附件ID/host辅助投影；Parent 对账实际 retained paint/input 与同帧来源 | source edited；7新增/19预计core测试未运行，实际命中/绘制/AI消费者仍未接通 |
| 组件/样式/宿主 | Parent 对应既有 catalog、源组件、slots/绑定与公共 Penpot UUID/version；明确 graph pane native 接入与 sourceRef，不把普通 Space/ScrollableBox 算完成 | pending；公共发布需 Parent 实际回执 |
| 域 adapter 与命令 | 各域在自己的实际模型、document/history 和 plugin 路径实现；共享 Target/revision/focus/retention 对齐后才接 host | pending；不能仅靠 descriptor 计为支持 |
| 聚焦验证 | 文档先做链接/结构/精确路径守卫；共享 Rust 变更后仅 `zircon_editor` lib 中 `ui::graph::tests::` 及受影响单个域 adapter，默认 features 关闭，实际 features/命令须在执行前声明并由 Jenkins 托管 | core19已用独立identity实际提交#353，checkout_input_identity_conflict/compileStarted=false，19项未运行；材质原13项及两次原终态分别保留，不重投 |
| 原生与双端 review | Parent 排队正常 EXE；三个尺寸及实际 150% DPI、默认/空/选中/焦点/弹层返回/滚动/中文长文本，再验证拖动、接线替换/拒绝/Undo、附件/无边模式和真实域状态 | 全部 not_run；passed=false、accepted=false |

最小跨域检查场景为：材质 Scalar→Multiply→Output 的真实编译；AI 已有 parent 的有序 Reparent/失败不变/一次 Undo；粒子无 pin 无 edge 的真实 Overview、五类节点编译映射与 configure-once 的真实替换；Widget 的 binding 源定位及文本/图快捷键隔离；角色 Rig typed 端点/选择/真实 pose 与 OpStack 有序撤销；动画状态/转换子文档返回；Actor 稳定 pin 改名/split 与 source map。这些均以各域实际具备的模型为前提，缺失的模型继续公开，不用手写演示值补齐。

本轮产物清单继续为 `E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/artifacts.json`。保留早期只读合同阶段和历史 Penpot 记录，当前明确四文件授权及上述源码改动另记指纹/精确 diff/guard，不修改消费者、公共主题/catalog 或原 ZUI。未编译/启动 EXE/占用桌面、未创建源码副本，释放空间 0 B。任务及窗口族验收保持开放。

## 材质节点搜索消费接线32

监督恢复31已交付真实leaf API，不增加另一套公共graph或MaterialAsset跨域模型。四文件来源、8项新增/55项静态定义、三个真实Penpot板及queue15814/#386见material-family.md的恢复31和material-node-search-31-handoff.json。#386仍待管理锁，不以descriptor、设计导航或管理等待计实际节点创建。

| 现有宿主输入 | 可直接消费的实际API | 真实结果 / 宿主责任 |
| --- | --- | --- |
| 在当前材质graph打开菜单 | MaterialGraphNodeSearch::new(query) | 只投影原六动作；宿主保留实际asset/document/graph/subdocument/revision、输入owner和打开者focus token |
| SearchField的query改变 / clear | set_query(query)、query()、matches() | AND过滤key/名称/category；仍可见的原selected ID保持；IME composition先由原输入host完成，不逐键造提交 |
| 上下/Home/End/PageUp/PageDown | navigate(MaterialGraphNodeSearchNavigation) | 当前真实列表夹紧、Page步15；不改源节点坐标、camera、selection或history |
| 点击或键盘选择一行 | select_id(stable_action_key) | 过滤后不存在/过期key拒绝，不能用显示文字或行索引代替identity |
| Enter / 确认所选动作 | activate() | 返回原GraphNodeDescriptor一次并关闭临时状态；这不是GraphDeltaProofV1或mutation permit |
| Esc / 真正取消菜单 | cancel() | 不产出动作；关闭后query/selection输入不重开；native popup dismiss和focus return仍由原host实现 |

宿主在activate后须用同一真实document/subdocument与revision核对当前palette owner/schema及typed创建参数，再走领域prepare → 现有GraphDeltaProofV1/PreparedGraphDeltaV1 retention → 一次permit/primary history。Search本身不存document、不写持久图、不创建快照history、不替宿主分配node/pin/edge ID；取消或revision失配不能产生部分节点。落点只消费当前同代际graph-local anchor及原CanvasState变换，DPI只转换一次；禁止把Penpot板坐标作为产品落点。修改后创建的真实源ID需进入现有Details/SourceLocator/diagnostics/focus链，未有该链时仍为open。

当前注册operation material_editor.graph.create是创建Material Graph资产，payload schema material_editor.create_graph.v1，不能拿它冒充“新增一个节点”。现leaf未注册CreateNode handler/作者session或renderer callback；具体typed创建与宿主安装须由Parent实际接续，不给不存在的callback或事件UUID。公开SearchField源root WorkbenchSearchInputRoot已有query/value_property和input_focusable；源本身没有声明专用搜索提交事件。TreeRow源root WorkbenchTreeRowRoot已有text/selected/focused/hovered/pressed与input_focusable，原单root v2中没有我们新增的域事件。以上表格是消费映射，不能伪报这些输入已进入搜索API。

真实SearchInput component/main=6ddc248b-bd35-80c5-8008-d6c8c3ce892e / 6ddc248b-bd35-80c5-8008-d6c8c0f9c196；TreeRow=6ddc248b-bd35-80c5-8008-d6c8c7a79944 / 6ddc248b-bd35-80c5-8008-d6c8c452bf0c；source file均为Shared b4a1e3ce-9e6a-81fe-8008-d69d607aac12。搜索板11个native roots保留componentFile/shapeRef，现仅局部geometry/visibility/text样式override。源SearchField为Stretch，TreeRow为Stretch/Fixed dense；原生实例必须保留override/reset和source token映射。TreeRow尾部资产控件在新增菜单副本中隐藏，不复制主件、不修改Shared来充组件语义或正式导出通过。公共owner应校验其source-bound配置/children/default与菜单平面行适配；Domain只提供action key与选中/焦点状态，不发明新的公共List主题。

材料search31只在材质域使用。Widget/AI/粒子/动画/Actor/Rig继续提供自己的真实palette、类型、创建payload、连接合法性和domain command/history；没有材质compiler依赖要求。没有ports的真实图可显示但不提供连线工具；edges=[]单独不禁用有合法ports的图。共享GraphModel/CanvasState/Delta/retention保持原合同。

本次仅补精确可消费合同和源hash；不改公共host、model/commands、ZUI、catalog、tokens、Shared或任何域runtime。实际输入/IME、popup焦点返回、真实节点创建/撤销/保存及normal EXE/144DPI/逐图≤1DIP继续open。公共组件与宿主仍Parent/原public owner统筹，goal active不因这些独立验收项等待而停工。

### 接续33：已落地的材质 typed 创建预检

原材质leaf现公开validate_material_graph_node_creation(graph, action, candidate, resolve_kind)，其中graph/candidate直接复用现MaterialGraphAsset/MaterialGraphNodeAsset，action直接复用GraphNodeDescriptor；无新graph/store/UUID分配器。它验证当前六项注册descriptor、kind/schema、稳定ID/位置、Output、参数/override及完整纹理引用的实际类型，不要求作者图已可编译，也不代宿主编辑。只有现有source node/parameter可成为冲突jump target；新候选错误定位Graph，不能跳往不存在的候选node。

resolver必须绑定真实catalog及完整UUID/locator；字段扩展名不是资源类型证据。成功只返回空diagnostics，既不写图也不携带GraphDeltaProofV1/PreparedGraphDeltaV1；结果进入既有Document/revision的一次prepare/permit/inverse-history后才可提交。弹层关闭或source/revision/transform失配不得复用结果。UE同事务自动连线仍需实际domain候选与can_connect的共同接续，当前未冒充实现。

三源码及11新增/66总静态定义、hash/最小patch/guard见material-family.md作者语义33及material-node-creation-33-handoff.json；原#386身份/旧终态不改，实际Rust与host/paint/hit/focus/undo/renderer/EXE继续open。本域未覆盖Parent公共源，其他消费者不依赖Material编译器或此业务创建规则。

### 接续34：同源创建与自动连线候选

材质 leaf 新增 plan_material_graph_node_creation(graph, action, node, optional_link, resolve_kind) → MaterialGraphNodeCreationPlan，直接借用原DTO并复用 can_connect 的同一分析器。计划只含候选node/link与原连接计划；连接必须新旧端点各一，pin使用稳定schema key。占用输入返回 exact 原旧边，先去旧边再检查类型/环；没有可选边时允许独立创建及未接齐的作者状态。没有全图快照、第二store/history/clock，也不把计划当proof/permit。

原host须绑定真实目标的asset/document/graph/subdocument/source/revision、既有lease/retention、资源resolver与当代transform；在一次prepare/permit内插入节点并替换/插入边，用同一primary history记录一个inverse。失败、Esc、popup关闭或source/geometry代际变化不得发布部分修改，成功后的真实node/pin key才可进入现有Details/SourceLocator/focus链。这段host transaction、输入/paint/hit与Undo证据尚未实现，未因本域helper关闭公共缺口。

此API属于材质注册动作与compiler的域适配，不是Widget/AI/PFX/Rig/动画/Actor通用创建模型；各域继续提供其实际typed schema/can_connect/持久化/编译。空ports/edges、已有GraphModel/CanvasState/Delta/一次retention与DPI合同保持不变。材质source34的4路径/9新增定义、75过滤器静态定义与7保护输入见 material-node-autowire-34-validation.json；格式/精确guard通过不等于Rust执行或产品验收。原#386现有真实FAILURE/management_git_busy另链保留；源码审阅时尚无本轮新POST，后继正式收件见下段。公共源和Penpot未变；full goal active，accepted=false。

后续实际正式验证已由唯一producer受理：同原request、新operation matgraph-autowire-01a16d50-20261025-34 / queue16229 / Jenkins#391，current source9＋deps21 / identity9f8f7982…。canonical entry两次capture及公开transport通过，client exit0；归档仍waiting/management_git_busy，compile/75项发现执行/native资源均未知。实际交付见material-node-autowire-34-delivery.json；仅沿此已受理身份观察/对账，不因等待再造operation，不清未知锁或用本地Cargo替代。旧source31/#386失败独立保留，goal active。

### 接续35：搜索行的稳定身份点击确认

材质host可用 MaterialGraphNodeSearch::activate_id(stable_action_key) 直接接当前列表行点击；只有当前可见key且菜单open才返回一次原注册GraphNodeDescriptor并关闭。过滤后旧key/未知key/已取消菜单返回None，保持原query/results/selected/open；键盘Enter仍用activate()。这只合并现有菜单局部选择/确认，不代host的popup incarnation/document/source/revision/IME/focus guard，不携mutation许可。

两本域源码、2新增回归及原cancel补pointer分支见 material-node-search-click-35-validation.json。当前77过滤器静态定义未Rust执行；#391原30声明仅搜索这2项为透明source35后继，原payload/identity/queue保持，旧75不报成当前77通过。不改公共Graph/主题/Shared、其它域或fork30的4个Instance SearchInput和Hierarchy；本轮无Penpot输入，真实host/renderer/EXE/DPI验收仍open。

### 接续36：弹层关闭、焦点和可逆布局消费

fork30的Hierarchy原型保存/关闭/新context重开与十三图已由原producer逐张复核，详见 material-family.md 接续36和 material-fork30-consumption-36.json。真实失败仍是外点/Esc后菜单存在、BODY焦点；返回图的按钮描边不作语义焦点证据。该失败和节点搜索消费的是同一原host输入/overlay原语，以下是给Parent的精确消费合同，不是新增公共实现或事件UUID。

| 原host已有责任 | 材质域消费与拒绝条件 |
| --- | --- |
| 打开上下文 | 保留实际asset/document/graph/subdocument/source locator/revision、当前popup incarnation、原输入owner及打开者focus token；不另建store/clock或用设计板ID替代产品source。 |
| 真实动作 | 搜索只交当前可见稳定action key；Hierarchy只交完整AssetRef并经registry/Toolkit打开。No Parent Found是不可导航的空态，不能路由到假父资源。 |
| Esc/外点/窗口切换 | 真正dismiss原overlay，调用现菜单cancel并拒绝该popup迟到输入；IME composition和快捷键归原native输入owner，不能由每次query变化创建/提交节点。 |
| 焦点返回 | 关闭后核验打开者仍存活且source上下文一致，再恢复其真实focus；若已切换/关闭，使用当前真实document的既有fallback。颜色/选中样式不证明focus owner。 |
| 布局/命中 | 同代际逻辑anchor/viewport/clip、retained bounds和同一transform；DPI只转换一次。SearchInput根Fill及原背景/文本left-right是consumer override，正式source delta须可逆、保留reset/source refs，不detach或贴元数据冒充导出。 |
| 作者事务 | 取消不改Document；确认后仍消费33/34 typed预检，走原prepare/retention/一次permit/inverse-history，完成后的真实source locator才进入Details/focus。菜单局部一次激活不替代mutation许可。 |

现有SearchInput component/main 6ddc248b-bd35-80c5-8008-d6c8c3ce892e / 6ddc248b-bd35-80c5-8008-d6c8c0f9c196及Shared file b4a1e3ce-9e6a-81fe-8008-d69d607aac12继续复用。consumer几何修正/保存重开只证明该设计切片；官方codec/source closure/override-reset、native dismiss/focus、paint/hit/真实作者修改/renderer与EXE144DPI仍open。本轮不写公共host/GraphModel/commands/retention、Shared/catalog/theme或其它域；原#391不重投，源和验证身份边界保持。

### 接续37：共享稳定端点的材质实际消费

材质leaf新 suggest_material_graph_autowire 直接消费原 GraphPortRef<String> 的node_id/pin_name/direction，在实际MaterialGraphAsset中验证唯一source节点和真实schema pin，再按原注册action对向pins顺序调用现材质can_connect分析器。第一次合法候选返回原MaterialGraphLinkAsset；无对向/无兼容pin返回None，不画装饰wire。palette中float标签不替代vector/texture业务类型；这是材质域实现，Widget/AI/PFX/Rig/动画/Actor继续提供各自真正的类型、顺序和创建/重连规则，不依赖MaterialAsset或材质compiler。

接口只建议一条边，不持document/source lease，不改原节点、selection、CanvasState或history，也不是proof/permit。已连接的拖出输入涉及UE多边InsertNewNode，本接口明确拒绝并返回现Input定位，不丢旧边；原34显式replacement语义保持。真实host仍须保留当代popup/source/revision/geometry，消费同一typed创建plan后只用一次prepare/retention/permit/inverse-history。新增3域路径和7定义/84当前静态定义、保护输入及最小patch见 material-family.md 接续37与原reports，未Rust执行，#391原身份不重投，旧75/77历史保持。公共Graph源/commands/model/DPI/焦点合同及其它域源码均未改，产品验收继续open。

### 接续38：材质已占用输入的实际域候选

本域新增 plan_material_graph_node_insertion(GraphPortRef::Input)，实际消费稳定node/schema-pin身份与原GraphPinDirection，不扩公共模型。MaterialGraphNodeInsertionPlan借用真实候选节点/旧边；links()只含一条明确替换或两条共同预检的插入边，reconnects_previous_source()明确区别结果，旧来源节点/其它fanout保留。按UE InsertNewNode选择合法输出/接收端；接收端不存在或联合类型不合法时允许原UE替换fallback，坏旧端点/多边/循环仍拒绝。不将显示名、跨域MaterialAsset或装饰wire作共享Graph语义。

旧37单边suggest仍拒绝occupied Input，不能冒充两边插入。实际host须选本域38适配，在同一source/document/retention revision上，把节点新增、exact旧边移除、一或两边新增作为同一次domain candidate，原prepare/一次permit/一次inverse-history发布；只读计划不含authority，不绕GraphDeltaProofV1/PreparedGraphDeltaV1，也不造第二history。公共source/host未改，原geometry generation/一次transform-DPI/paint-hit/focus与cancel生命周期仍需真实接线。

七项新增回归定义利用原DTO/codec/compiler，静态91项中90普通＋1原ignored，未Rust执行。source38四路径final guard/格式通过，19保护输入保持；原#391实际FAILURE而归档waiting/management_git_busy，compile/tests/native/resource unknown，不能用管理job终态替代。当前接口是材质业务适配，Widget/AI/Rig/PFX/Actor/动画仍各自负责schema与合法性；空ports/edges合同和已有公共graph API保持。详见同目录material-family.md接续38及原reports的material-node-insertion-38-validation.json；产品/正式导出/EXE/双端验收open。

### 接续39：插入候选的实际设计消费

材质原file/page追加4板、213对象、15个原本域Parameter/Math/Output实例，复用真实component/source/shapeRef及Shared颜色/Body。rev161保存关闭后独立context重开：1280/900/640为同逻辑图，900另有明确替换态；未改原主件、公共Graph或集合外对象。这些proposal UUID不成为其它域通用节点API。

同一world布局/viewport推导缩放平移；640缩放0.7189873418一次。线端点取真实实例pin中心，按schema对应的原pin shapeRef决定连接实心/空心；材质依UE不画箭头。最终四图逐图可见、文字无越板，API端点差最大0.000449逻辑像素，只证明设计几何。初稿隐藏文字/空Viewer/边穿节点/pin fill与SVG规范化检查失败另存；通过实际workspace文字测量与保存关闭重开修正，不重签正式codec。

源38同原request合法后继matgraph-insertion-01a16d50-20261025-38已queue16803/#396，checkout_source_busy具体阻塞owner交原Jenkins owner；旧#391 FAILURE及r10原生未知保留。原Document/GraphDeltaProofV1/PreparedGraphDeltaV1/retention/一次permit与真实paint/hit/focus/input-owner/cancel/undo不被设计图绕过。正式导出、正常EXE/144DPI/双端1DIP继续open。


### 接续41：实际材质源投影与可修复错误图

本域公开 `MaterialGraphPresentation<'a>` 与 `MaterialGraphEdgeProjection<'a>`，源在 [graph_presentation.rs](../../../zircon_plugins/material_editor/editor/src/graph_presentation.rs)，回归在 [tests/graph_presentation.rs](../../../zircon_plugins/material_editor/editor/src/tests/graph_presentation.rs)。这直接消费当前 MaterialGraphAsset，使用原 GraphNodeView/GraphEdgeView/GraphPortRef、GraphNodePresentation 与 GraphCanvasState；不要求其它域采用 MaterialAsset/材质编译器，不新增公共 Graph 源文件或 library UUID。

| 当前可消费接口 | 源身份与宿主责任 |
| --- | --- |
| new(graph)、nodes()/links()/node(id) | 借用实际源数组，保作者顺序和 key；重复/空 node ID 拒绝。宿主从当前 Document/asset source 取得数据并保原 revision、pane incarnation、能力和 lease；不可塞入 fixture producer。 |
| node_view(id, measured_size) | origin 取真实作者 position，size 取同 retained generation 实测逻辑尺寸；不做 DPI/相机转换。公共原 CanvasState 负责一次 paint/hit transform、clip/origin、拖动预览和 selection，不能只把 helper 测试当 native 实测。 |
| output_type / type_diagnostic | 复用域 analyzer 的 Scalar/Vector/Texture/Undetermined，不靠显示名或旧 palette float 标签推断；错误图保持可修复。完整 compile/readiness diagnostics 仍由当前真实编译接口提供，空图/required-input 不抹除。 |
| edge_view(source_index) | 保 exact 原 link 引用、本次 source occurrence 与原 node/pin schema key。端点失效报错，类型错误的有效端点 wire 可以呈现并定位修复；稳定持久 edge UUID 尚缺，index/地址不得跨 revision 当 ID。 |
| connection_plan(from, to) | 先查真实方向，再走本域 can_connect；保 scalar 广播、texture 限制、Replace/Unchanged 与 exact old edge。UI string value_type 不是 mutation 授权，不能用 shared default verdict 代域检查。 |

Parent 生产接续范围仍为其持有的原 host/retained/runtime assembly。最小调用是：从真实当前 Document/source 保留一次只读投影 → 用同代际实测 size 形成 node views 和 exact occurrence wire → 交既有 node presenter/CanvasState/selection/Details source locator → 每次输入重新核 source/revision/geometry → 转本域33/34/37/38创建或连接计划 → 原 domain delta/GraphDeltaProofV1/PreparedGraphDeltaV1预算及一次permit/inverse-history。该 adapter 不实现 GraphModel 的 mutation 生命周期，不以 apply/整图快照补空缺；缺少真正 schema/edge ID 时必须保对应能力限制和诊断。

十项新测试定义包含实际共享 presenter/canvas 的源码消费、原 TOML roundtrip 与 PBR编译器，尚未 Rust 类型检查/执行，不能称生产 consumer/renderer 通过。最终 five-source guard 和101静态定义、十六只读依赖指纹、当前 #396 的来源差额见 material-family.md 接续41及原 reports final 回执。公共 Graph/model/commands/retention/DPI/painter、其它域原资产和 fork30 scope 未写；原 ZUI29nodes/27controls/13events不变。真实 Input/paint/hit/focus/cancel/Undo、正式导出、正常 EXE/144DPI/双端比较继续 open。


### 接续42：歧义材质输入保留每条真实边，不从第一条推导类型

材质域41的`MaterialGraphPresentation`仍是现有中立GraphNodeView/GraphEdgeView的只读consumer，未另建公共模型、history、clock或IDs。42只在域适配器使用现有MaterialGraphIndex的incoming count：count=1才从原source求输入显示类型；count>1显示undetermined，保留原Input诊断和全部原occurrence。相同端点的两条link仍各自有source指针/source_index，不压成一条；这些身份仅在原文档revision借用内有效，不能冒充持久edge UUID。

四case回归检查不同类型/相同端点及各自的两种顺序，不改变`can_connect_material_graph`、合法性、compiler、shared model/commands/retention/proof/prepare/一次permit。host仍负责文档/graph/subdocument/revision与geometry generation、一次transform/DPI、独立命中、焦点/诊断跳转及原history适配。源码静态通过不是实际host/paint/EXE证据。

[42精确交付与增量patch](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-graph-ambiguity-42-handoff.json)记录两源及37项只读依赖；当前102静态定义未运行。原41唯一managed提交queue17524/#400基于当时101定义，11:43:40Z为running/management_git_busy；其身份不改写、旧操作不重投，公共源无本轮写入。


### 接续43：显式 Hermite travel tangent 的最小可选扩展提案

只读UE GraphEditorSettings默认forward/backward policy和Slate spline契约表明，现有cardinal路由不足以表达材质原始切线；不改旧API/default以影响AI、Actor、Widget等消费者。建议新增`route_connection_with_tangents(source, target, source_travel_tangent, target_travel_tangent) -> Option<GraphConnectionRoute>`。域adapter提供真实局部逻辑锚点和同一geometry generation下的travel导数；公共构造仅转换controls `(source + start/3, target - end/3)`，拒NaN/Inf或控制点溢出，无DPI/相机/主题/合法连线语义。target travel tangent不是反向的incoming handle向量。零导数仍可形成curve；原endpoint_tangents/target_arrow将其退化端点返回None，是否绘箭头由真实域样式决定，材质ArrowImage=nullptr不画箭头。

[43未应用补丁](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-spline-43-routing-proposal.patch)仅涉及routing/mod/tests-cases三路径，保留route_connection及route_connection_with_directions、现GraphModel/Canvas/commands/retention/proof/prepare/一次permit。三候选测试覆盖任意二维导数和箭头投影、无48下限/零导数、全NaN/Inf/溢出输入。精确preimage+git apply --check通过仅是可审阅性，不是类型检查或测试执行；本轮canonical公共源未写，需Parent整合。

材质只在自己文件39原四板改14条既有native paths，rev161→162；原端点/ID/15copies/共享样式/非14对象完全保留。实际save/close/各fresh context重开及四PNG逐图复核见[43证据](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-spline-43-review.json)。这是设计fidelity修正，未证明真实host geometry/paint/hit/focus/drag或产品1DIP；不将Penpot图表面算可用共享Graph library，不增加源edge UUID或另建作者store。

[43 host接线边界](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-spline-43-host-proposal.json)列现有native descriptor/result budget/codec与原Document→MaterialAsset→ResourceManager真实发布缺口，保ZMaterialDocument parent/local presence/editor metadata，不能以静态Not implemented面板或operation metadata宣称工具可执行。原41 queue17524/#400仍waiting/management_git_busy，42两源后继和102静态定义独立保留；无新Cargo派发，正式export/正常EXE/144DPI/双端比较继续open。


### 接续44：域 policy 先消费真实 edge 与原 Canvas，一次变换后计算 UE 切线

材质本轮仅改四个本域 Rust 路径，新增 `MaterialGraphEdgeSplineProjection`/`edge_spline_view`，以原 occurrence、真实 node/pin 校验和 caller 实测锚点消费现 `GraphCanvasState`。UE 默认 clamp 在绘制空间进行，因此先一次 graph→pane logical transform，再计算 forward/backward travel 导数；native DPI 留最终转换一次，host 必须校验真实 document/revision 与 retained geometry generation，revision 本身不等于几何世代。禁止从 node center/截图补造pin坐标、复用 legacy 48下限冒充UE、或把返回值再当 graph-local 缩放。

`GraphConnectionRoute` 私有字段不绕过；43 explicit-tangent constructor 仍是 [Parent 可审阅未应用补丁](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-spline-43-routing-proposal.patch)，本轮无公共 source 写入，没有改默认/cardinal API、model/commands/retention/proof/prepare/一次permit，不替 AI/Widget/Actor/PFX/Rig/Animation 定义域合法性。域 spline projection 已有6项意义回归定义与静态检查，[当前44源](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-spline-policy-44-source-final.json)；实际 host/painter/命中/附件/focus/正式导出和正常EXE未证明。

44 新 operation `matgraph-spline-01a16d50-20261025-44` 是原 material request 下当前源有限后继，108静态定义不当执行结果。原41 queue17524/#400当前 FAILURE/management_git_busy/native未知，不重放。45秒接收配置发布空档12:30:28Z–12:31:13Z已自动到期，仅限制本 producer 下一派发，不限制源/设计/文档、不推资源已释放。实际准入与构建结果见 [44 managed](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-spline-policy-44-managed.json)；没有新公共Graph系统或第二相机/store。


## 接续45：Material动作搜索既有Shared SearchInput消费约束

本轮没有扩展公共Graph API。只在本域三张既有 NodeSearch31 板消费原 Shared SearchInput，root Fill 保留，background/text 的横向约束改为 leftright，保留 icon Left、component/file/main/shapeRef 和原 touched override。6对象变化、其余2374对象不变，400→320→400的真实原生宽度输入及服务器保存确认后，全部对象恢复为约束修正后的原值；3个独立新context重开和8图复核见 [45真实设计与逐图证据](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-node-search-layout-45-review.json)。没有改 fork30 的4个 Instance copies、Shared/catalog/tokens/Starship 或消费者业务schema。

该消费只证明本域 editable flow 分配；不证明 canonical master约束已同步、reset/全局样式传播或官方导出。公共 owner 可基于原 component=`6ddc248b-bd35-80c5-8008-d6c8c3ce892e` / main=`6ddc248b-bd35-80c5-8008-d6c8c0f9c196`，以及原 search_input.zui/field-controls 的既有 leftright 原语处理正式 canonical/source-delta，不新增另一 SearchInput。图弹层仍需源绑定的 popup/document incarnation、同一 transform 的 anchor、真实 can_connect、一次 prepared permit、Esc/外点取消与原焦点返回；这些不能由 Penpot静态选中行或当前 CanvasState 自行推导。44唯一受管原操作#403仍待管理owner沿原Records/writer处理，无新增编译/producer。


## 接续46：Material搜索到创建的领域适配，公共Graph不扩权

材质自有 `plan_material_graph_node_search_creation` 将当前打开菜单可见的稳定action key与真实typed candidate/autowire接到原领域创建校验。鼠标key不由键盘selected替代；closed/hidden/unknown在resolver前拒绝，剩余合法性和旧edge保留继续沿原Material validator。该helper返回原借用plan，不变更菜单/作者图、不安装新GraphModel/store，不新增revision计数或history；共享proof/prepare/一次permit、retention预算、变换/DPI和消费者业务schema完全保留。

7回归定义与精确三路径patch见 [46域适配源码](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-search-creation-46-source.json) 及 [46静态/只读验证](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-search-creation-46-validation.json)。42原非目标输入未变，115为当前静态定义而非发现/执行。生产host仍须绑定真实 popup/document incarnation、source/revision/selection、同代际anchor 与一次域事务；本API不把label、row index或内容相等当ABA权限，不在预检阶段关闭菜单/返回焦点。没有修改公共model/commands/retention/canvas或AI/Actor/Widget/Rig/PFX/Animation源，不将Material候选作为跨域Graph模型。44/#403原managed链独立保留，未以新源改写旧输入或新起第二producer。


46实际后继已正式POST queue18854/#407（client39060 exit0），原request与当前18+25声明输入/115静态定义见 [46 managed原链](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-search-creation-46-managed.json)。44/#403已确认API FAILURE，旧497/490B等待回执与native未知分别保留；407当前building与waiting/management_git_busy不等于类型检查或测试执行。45秒发布空档已自动结束，只控制本producer下一派发。没有新增公共source写权、第二producer或共享Graph业务模型。


## MAT-SHARED-001 接续47：材质编译诊断域适配，不扩大公共图模型

材质本域新增 `compile_material_graph_with_shader_contract`，在原CPU编译结果上复用Runtime真实ShaderAsset材质契约，返回 Graph / ShaderLocatorMismatch / ShaderImportDiagnostics / ShaderContract 四类错误。Graph沿当前document的node/pin，Runtime保留原source/path；禁止把`overrides.*`、`textures.*`、`shader.kind`捏成Graph node/pin。Shader URI核对仅防所供源不同，宿主仍须完整AssetReference→原ResourceManager解析/active-version与document/request验证；不宣称GPU shader/texture ready或 renderer publication。

通用GraphModel/Canvas/Delta/proof/permit/retention未改，Widget/AI/Rig/VFX/Actor/Animation不依赖MaterialAsset或compiler。Parent的原宿主需提供per-view作者源读取/单次prepared编辑与原Toolkit保存接线；所查Material metadata/native command无已接caller，域模块不另造document store、clock、history或原生JSON后端。当前121静态定义含新6项，尚无其实际执行回执；原managed46 #407归档仍waiting，47不借其旧输入通过。域/宿主具体路径、原ZUI13事件与保留边界见[接线交接](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-host-integration-47-handoff.md)和[源码及原Runtime API指纹](E:/cargo-targets/zircon-local/reports/ue-material-family-20261024-01a16d50/material-shader-contract-47-source.json)。正常EXE、retained paint/hit/focus、导出和双端视觉验收仍open。
